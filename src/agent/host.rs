use crate::agent::worker::AppWorker;
use crate::memory::episode::{EpisodeManager, EpisodeStep};
use crate::perception::traits::{PerceptionProvider, ScreenState};
use crate::tools::traits::ToolResult;
use anyhow::{anyhow, Result};
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::sync::Arc;

const MAX_PLANNED_STEPS: usize = 8;
const MAX_REPLAN_ROUNDS: usize = 3;
const MAX_EXECUTION_STEPS: usize = 24;

#[derive(Debug, Clone, PartialEq, Eq)]
enum PostStepDecision {
    Continue,
    Replan(String),
    Escalate(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
struct ScreenStateDiff {
    app_before: String,
    app_after: Option<String>,
    app_changed: bool,
    widget_tree_changed: bool,
    ocr_count_changed: bool,
    capture_failed: bool,
}

impl ScreenStateDiff {
    fn has_meaningful_change(&self) -> bool {
        self.app_changed || self.widget_tree_changed || self.ocr_count_changed
    }
}

/// The HostAgent is the global planner and router in the AgentOS architecture.
/// It decomposes tasks and dispatches them to appropriate AppAgents (Workers).
pub struct HostAgent {
    workers: Vec<Arc<dyn AppWorker>>,
    perception: Arc<dyn PerceptionProvider>,
    episode_manager: EpisodeManager,
}

impl HostAgent {
    pub fn new(perception: Arc<dyn PerceptionProvider>, episode_manager: EpisodeManager) -> Self {
        Self {
            workers: Vec::new(),
            perception,
            episode_manager,
        }
    }

    pub fn register_worker(&mut self, worker: Arc<dyn AppWorker>) {
        self.workers.push(worker);
    }

    /// Plans and routes tactical steps based on global awareness.
    pub async fn run_task(&mut self, user_goal: &str) -> Result<()> {
        self.run_task_with_result(user_goal).await.map(|_| ())
    }

    /// Plans and routes multiple tactical steps based on global awareness and returns
    /// the final worker result for callers that need direct response content.
    pub async fn run_task_with_result(&mut self, user_goal: &str) -> Result<ToolResult> {
        if user_goal.trim().is_empty() {
            return Err(anyhow!("user goal cannot be empty"));
        }

        let mut pending_steps: VecDeque<String> = Self::plan_steps(user_goal).into_iter().collect();
        if pending_steps.is_empty() {
            pending_steps.push_back(user_goal.trim().to_string());
        }

        let mut final_result: Option<ToolResult> = None;
        let mut previous_step_output: Option<String> = None;
        let mut trajectory_context: Option<String> = None;
        let mut execution_count = 0usize;
        let mut replan_rounds = 0usize;

        while let Some(planned_step) = pending_steps.pop_front() {
            execution_count += 1;
            if execution_count > MAX_EXECUTION_STEPS {
                return Err(anyhow!(
                    "host execution exceeded maximum step budget ({MAX_EXECUTION_STEPS})"
                ));
            }

            let total_steps = execution_count + pending_steps.len();
            let state_before = self.perception.capture_state().await?;
            let active_app = self.infer_active_application(&state_before)?;
            if trajectory_context.is_none() {
                trajectory_context = Some(active_app.clone());
            }
            let worker = self.select_worker_for_context(&active_app)?;
            let step_instruction = Self::build_step_instruction(
                user_goal,
                &planned_step,
                execution_count,
                total_steps,
                previous_step_output.as_deref(),
            );

            let step_index = self.episode_manager.next_step_index();
            let execution = worker.execute_step(&step_instruction, &state_before).await;
            let state_after = self.perception.capture_state().await.ok();
            let state_diff = Self::diff_screen_state(&state_before, state_after.as_ref());

            let (step, worker_result, worker_error) = match execution {
                Ok(result) => {
                    let failure_reason = Self::failure_reason_from_result(&result);

                    (
                        EpisodeStep {
                            step_index,
                            action_taken: planned_step.clone(),
                            action_result: result.output.clone(),
                            screen_state_before: Some(state_before),
                            screen_state_after: state_after,
                            execution_error: failure_reason.clone(),
                        },
                        Some(result),
                        None,
                    )
                }
                Err(err) => {
                    let error_string = err.to_string();
                    (
                        EpisodeStep {
                            step_index,
                            action_taken: planned_step.clone(),
                            action_result: String::new(),
                            screen_state_before: Some(state_before),
                            screen_state_after: state_after,
                            execution_error: Some(error_string),
                        },
                        None,
                        Some(err),
                    )
                }
            };

            self.episode_manager.record_step(step);
            self.episode_manager.flush().await?;

            if let Some(err) = worker_error {
                return Err(err.context(format!(
                    "worker '{}' failed while executing planned step {execution_count}/{total_steps} for '{}'",
                    worker.name(),
                    active_app
                )));
            }

            if let Some(result) = worker_result {
                let decision = Self::decide_post_step_action(&result, &state_diff);
                previous_step_output = Some(result.output.clone());
                final_result = Some(result.clone());

                match decision {
                    PostStepDecision::Continue => {}
                    PostStepDecision::Replan(reason) => {
                        if replan_rounds >= MAX_REPLAN_ROUNDS {
                            return Err(anyhow!(
                                "replanning exhausted after {MAX_REPLAN_ROUNDS} rounds: {reason}"
                            ));
                        }

                        replan_rounds += 1;
                        let replanned = Self::build_replanned_steps(
                            user_goal,
                            &planned_step,
                            &result,
                            &state_diff,
                            &pending_steps,
                        );
                        if replanned.is_empty() {
                            return Err(anyhow!(
                                "replanning produced no remaining executable steps"
                            ));
                        }
                        pending_steps = replanned;
                    }
                    PostStepDecision::Escalate(reason) => {
                        return Err(anyhow!(reason).context(format!(
                            "worker '{}' escalated while executing planned step {execution_count}/{total_steps} for '{}'",
                            worker.name(),
                            active_app
                        )));
                    }
                }
            }
        }

        self.episode_manager
            .promote_to_trajectory(trajectory_context.as_deref().unwrap_or("unknown"))
            .await?;

        Ok(final_result.unwrap_or(ToolResult {
            success: true,
            output: String::new(),
            error: None,
        }))
    }

    fn select_worker_for_context(&self, active_app: &str) -> Result<Arc<dyn AppWorker>> {
        self.workers
            .iter()
            .find(|candidate| candidate.can_handle(active_app))
            .cloned()
            .ok_or_else(|| {
                anyhow!(
                    "no registered worker can handle application context '{}'",
                    active_app
                )
            })
    }

    fn failure_reason_from_result(result: &ToolResult) -> Option<String> {
        if result.success {
            return None;
        }
        result
            .error
            .as_deref()
            .map(str::trim)
            .filter(|msg| !msg.is_empty())
            .map(ToString::to_string)
            .or_else(|| {
                let output = result.output.trim();
                (!output.is_empty()).then_some(output.to_string())
            })
            .or_else(|| Some("worker reported unsuccessful execution".to_string()))
    }

    fn build_step_instruction(
        user_goal: &str,
        planned_step: &str,
        step_number: usize,
        total_steps: usize,
        previous_step_output: Option<&str>,
    ) -> String {
        if total_steps <= 1 && planned_step.trim() == user_goal.trim() {
            return user_goal.trim().to_string();
        }

        let mut instruction = if total_steps <= 1 {
            format!("Overall goal:\n{user_goal}\n\nCurrent planned step:\n{planned_step}")
        } else {
            format!(
                "Overall goal:\n{user_goal}\n\nCurrent planned step {step_number}/{total_steps}:\n{planned_step}"
            )
        };

        if let Some(previous_output) = previous_step_output {
            let trimmed = previous_output.trim();
            if !trimmed.is_empty() {
                instruction.push_str("\n\nPrevious step outcome:\n");
                instruction.push_str(trimmed);
            }
        }

        instruction.push_str(
            "\n\nExecute this step now. Observe concrete state changes after execution. Continue if progress is real, otherwise replan safely or escalate when blocked.",
        );
        instruction
    }

    fn plan_steps(user_goal: &str) -> Vec<String> {
        let explicit_steps = Self::extract_explicit_list_steps(user_goal);
        if !explicit_steps.is_empty() {
            return explicit_steps.into_iter().take(MAX_PLANNED_STEPS).collect();
        }

        let normalized = user_goal.replace('\n', " ");
        let sentence_steps: Vec<String> = normalized
            .split(['.', ';'])
            .flat_map(Self::split_by_connectors)
            .map(|step| step.trim().to_string())
            .filter(|step| !step.is_empty())
            .take(MAX_PLANNED_STEPS)
            .collect();

        if sentence_steps.is_empty() {
            vec![user_goal.trim().to_string()]
        } else {
            sentence_steps
        }
    }

    fn extract_explicit_list_steps(user_goal: &str) -> Vec<String> {
        user_goal
            .lines()
            .filter_map(Self::strip_list_marker)
            .map(str::trim)
            .filter(|step| !step.is_empty())
            .map(ToString::to_string)
            .collect()
    }

    fn strip_list_marker(line: &str) -> Option<&str> {
        let trimmed = line.trim();
        if let Some(rest) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
        {
            return Some(rest);
        }

        let mut seen_digit = false;
        for (idx, ch) in trimmed.char_indices() {
            if ch.is_ascii_digit() {
                seen_digit = true;
                continue;
            }

            if seen_digit && (ch == '.' || ch == ')') {
                let rest_start = idx + ch.len_utf8();
                let rest = trimmed[rest_start..].trim_start();
                return (!rest.is_empty()).then_some(rest);
            }

            break;
        }

        None
    }

    fn split_by_connectors(segment: &str) -> Vec<String> {
        if segment.trim().is_empty() {
            return Vec::new();
        }

        let mut steps = vec![segment.to_string()];
        const CONNECTORS: [&str; 4] = [" and then ", " then ", " after that ", " afterwards "];

        for connector in CONNECTORS {
            let mut next_steps = Vec::new();
            for part in steps {
                let pieces: Vec<String> = part
                    .split(connector)
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(ToString::to_string)
                    .collect();
                if pieces.len() <= 1 {
                    next_steps.push(part);
                    continue;
                }
                next_steps.extend(pieces);
            }
            steps = next_steps;
        }

        steps
    }

    fn decide_post_step_action(
        result: &ToolResult,
        state_diff: &ScreenStateDiff,
    ) -> PostStepDecision {
        let diagnostic = result.diagnostic_output();
        let diagnostic_lower = diagnostic.to_ascii_lowercase();

        if !result.success {
            if Self::is_transient_failure(&diagnostic_lower) {
                return PostStepDecision::Replan(format!(
                    "transient failure detected after step: {}",
                    diagnostic.trim()
                ));
            }
            return PostStepDecision::Escalate(format!(
                "worker failure requires escalation: {}",
                diagnostic.trim()
            ));
        }

        if Self::is_permission_or_policy_failure(&diagnostic_lower) {
            return PostStepDecision::Escalate(format!(
                "execution blocked by runtime policy or permissions: {}",
                diagnostic.trim()
            ));
        }

        if result.output.trim().is_empty() && !state_diff.has_meaningful_change() {
            return PostStepDecision::Replan(
                "step produced no output and no detectable runtime state delta".to_string(),
            );
        }

        if Self::signals_incomplete_progress(&diagnostic_lower) {
            return PostStepDecision::Replan(
                "worker output indicates incomplete progress requiring a revised next step"
                    .to_string(),
            );
        }

        PostStepDecision::Continue
    }

    fn build_replanned_steps(
        user_goal: &str,
        completed_step: &str,
        result: &ToolResult,
        state_diff: &ScreenStateDiff,
        remaining_steps: &VecDeque<String>,
    ) -> VecDeque<String> {
        let mut next_steps = VecDeque::new();

        let mut from_output = Self::extract_explicit_list_steps(&result.output);
        if from_output.is_empty() {
            from_output = Self::split_by_connectors(&result.output.replace('\n', " "));
        }
        let has_output_derived_steps = from_output.iter().any(|step| !step.trim().is_empty());

        for step in from_output {
            Self::push_unique_step(&mut next_steps, &step);
            if next_steps.len() >= MAX_PLANNED_STEPS {
                return next_steps;
            }
        }

        if next_steps.is_empty() {
            let mut recovery_step =
                format!("Recover and re-attempt safely: {}", completed_step.trim());
            if state_diff.app_changed {
                if let Some(app_after) = state_diff.app_after.as_deref() {
                    write!(
                        recovery_step,
                        " (runtime app moved from '{}' to '{}')",
                        state_diff.app_before, app_after
                    )
                    .expect("writing runtime app transition to string");
                }
            } else if state_diff.capture_failed {
                recovery_step.push_str(" (state-after capture failed; validate before retry)");
            }
            Self::push_unique_step(&mut next_steps, &recovery_step);
        }

        for step in remaining_steps {
            if next_steps.len() >= MAX_PLANNED_STEPS {
                break;
            }
            Self::push_unique_step(&mut next_steps, step);
        }

        if next_steps.len() < MAX_PLANNED_STEPS
            && (has_output_derived_steps || !remaining_steps.is_empty())
        {
            Self::push_unique_step(
                &mut next_steps,
                &format!("Validate completion of overall goal: {}", user_goal.trim()),
            );
        }

        next_steps
    }

    fn push_unique_step(steps: &mut VecDeque<String>, step: &str) {
        let trimmed = step.trim();
        if trimmed.is_empty() {
            return;
        }
        if steps
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(trimmed))
        {
            return;
        }
        steps.push_back(trimmed.to_string());
    }

    fn is_transient_failure(lower: &str) -> bool {
        lower.contains("timeout")
            || lower.contains("timed out")
            || lower.contains("temporarily unavailable")
            || lower.contains("rate limit")
            || lower.contains("connection reset")
            || lower.contains("connection refused")
            || lower.contains("try again")
            || lower.contains("retry")
    }

    fn is_permission_or_policy_failure(lower: &str) -> bool {
        lower.contains("blocked by runtime policy")
            || lower.contains("permission denied")
            || lower.contains("denied by user")
            || lower.contains("not allowed")
            || lower.contains("forbidden")
            || lower.contains("approval")
    }

    fn signals_incomplete_progress(lower: &str) -> bool {
        lower.contains("unable to")
            || lower.contains("could not")
            || lower.contains("incomplete")
            || lower.contains("needs manual")
            || lower.contains("still pending")
    }

    fn diff_screen_state(before: &ScreenState, after: Option<&ScreenState>) -> ScreenStateDiff {
        let app_before = Self::state_application_name(before);
        let app_after = after.map(Self::state_application_name);

        match after {
            Some(after_state) => ScreenStateDiff {
                app_changed: app_after
                    .as_deref()
                    .is_some_and(|after_app| !after_app.eq_ignore_ascii_case(&app_before)),
                widget_tree_changed: before.widget_tree != after_state.widget_tree,
                ocr_count_changed: before.extracted_text.len() != after_state.extracted_text.len(),
                capture_failed: false,
                app_before,
                app_after,
            },
            None => ScreenStateDiff {
                app_before,
                app_after,
                app_changed: false,
                widget_tree_changed: false,
                ocr_count_changed: false,
                capture_failed: true,
            },
        }
    }

    fn infer_active_application(&self, state: &ScreenState) -> Result<String> {
        Ok(Self::state_application_name(state))
    }

    fn state_application_name(state: &ScreenState) -> String {
        if let Some(tree) = &state.widget_tree {
            if let Some(name) = tree
                .name
                .as_deref()
                .map(str::trim)
                .filter(|name| !name.is_empty())
            {
                return name.to_string();
            }
            let id = tree.id.trim();
            if !id.is_empty() {
                return id.to_string();
            }
        }
        "unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::NoneMemory;
    use crate::perception::traits::WidgetNode;
    use crate::tools::traits::ToolResult;
    use async_trait::async_trait;
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    struct StaticPerceptionProvider {
        state: ScreenState,
    }

    #[async_trait]
    impl PerceptionProvider for StaticPerceptionProvider {
        fn name(&self) -> &str {
            "test_perception"
        }

        async fn capture_state(&self) -> anyhow::Result<ScreenState> {
            Ok(self.state.clone())
        }
    }

    struct SequencePerceptionProvider {
        states: Vec<ScreenState>,
        calls: AtomicUsize,
    }

    #[async_trait]
    impl PerceptionProvider for SequencePerceptionProvider {
        fn name(&self) -> &str {
            "sequence_perception"
        }

        async fn capture_state(&self) -> anyhow::Result<ScreenState> {
            let index = self.calls.fetch_add(1, Ordering::SeqCst);
            if let Some(state) = self.states.get(index) {
                return Ok(state.clone());
            }
            self.states
                .last()
                .cloned()
                .ok_or_else(|| anyhow!("sequence perception requires at least one state"))
        }
    }

    struct StubWorker {
        name: String,
        handles: String,
        result: ToolResult,
    }

    #[async_trait]
    impl AppWorker for StubWorker {
        fn name(&self) -> &str {
            &self.name
        }

        fn can_handle(&self, application_context: &str) -> bool {
            application_context == self.handles
        }

        async fn execute_step(
            &self,
            _task_instruction: &str,
            _current_state: &ScreenState,
        ) -> anyhow::Result<ToolResult> {
            Ok(self.result.clone())
        }
    }

    struct RecordingWorker {
        name: String,
        handles: String,
        output_prefix: String,
        calls: Arc<AtomicUsize>,
        instructions: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl AppWorker for RecordingWorker {
        fn name(&self) -> &str {
            &self.name
        }

        fn can_handle(&self, application_context: &str) -> bool {
            application_context == self.handles
        }

        async fn execute_step(
            &self,
            task_instruction: &str,
            _current_state: &ScreenState,
        ) -> anyhow::Result<ToolResult> {
            let call_index = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            self.instructions
                .lock()
                .expect("instruction lock should work")
                .push(task_instruction.to_string());
            Ok(ToolResult {
                success: true,
                output: format!("{}-{call_index}", self.output_prefix),
                error: None,
            })
        }
    }

    struct SequencedResultWorker {
        name: String,
        handles: String,
        calls: Arc<AtomicUsize>,
        results: Arc<Mutex<VecDeque<ToolResult>>>,
    }

    #[async_trait]
    impl AppWorker for SequencedResultWorker {
        fn name(&self) -> &str {
            &self.name
        }

        fn can_handle(&self, application_context: &str) -> bool {
            application_context == self.handles
        }

        async fn execute_step(
            &self,
            _task_instruction: &str,
            _current_state: &ScreenState,
        ) -> anyhow::Result<ToolResult> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let mut queue = self.results.lock().expect("result queue lock should work");
            let result = queue
                .pop_front()
                .or_else(|| queue.back().cloned())
                .unwrap_or(ToolResult {
                    success: true,
                    output: "default".to_string(),
                    error: None,
                });
            Ok(result)
        }
    }

    fn state_for(app_name: &str) -> ScreenState {
        ScreenState {
            screenshot_path: None,
            widget_tree: Some(WidgetNode {
                id: "window_id".to_string(),
                role: "window".to_string(),
                name: Some(app_name.to_string()),
                value: None,
                bounds: None,
                children: Vec::new(),
            }),
            extracted_text: Vec::new(),
        }
    }

    fn perception_for(app_name: &str) -> Arc<dyn PerceptionProvider> {
        Arc::new(StaticPerceptionProvider {
            state: state_for(app_name),
        })
    }

    fn sequence_perception(states: Vec<ScreenState>) -> Arc<dyn PerceptionProvider> {
        Arc::new(SequencePerceptionProvider {
            states,
            calls: AtomicUsize::new(0),
        })
    }

    fn episode_manager() -> EpisodeManager {
        EpisodeManager::new(
            Arc::new(NoneMemory::new()),
            "session_test".to_string(),
            "goal".to_string(),
        )
    }

    #[tokio::test]
    async fn host_agent_run_task_succeeds_for_matching_worker() {
        let mut host = HostAgent::new(perception_for("Terminal"), episode_manager());
        host.register_worker(Arc::new(StubWorker {
            name: "terminal_worker".to_string(),
            handles: "Terminal".to_string(),
            result: ToolResult {
                success: true,
                output: "ok".to_string(),
                error: None,
            },
        }));

        let result = host.run_task("type command").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn host_agent_run_task_with_result_returns_worker_output() {
        let mut host = HostAgent::new(perception_for("Terminal"), episode_manager());
        host.register_worker(Arc::new(StubWorker {
            name: "terminal_worker".to_string(),
            handles: "Terminal".to_string(),
            result: ToolResult {
                success: true,
                output: "ok".to_string(),
                error: None,
            },
        }));

        let result = host
            .run_task_with_result("type command")
            .await
            .expect("host execution should succeed");
        assert_eq!(result.output, "ok");
    }

    #[tokio::test]
    async fn host_agent_decomposes_goal_into_multiple_steps() {
        let calls = Arc::new(AtomicUsize::new(0));
        let instructions = Arc::new(Mutex::new(Vec::new()));

        let mut host = HostAgent::new(perception_for("Terminal"), episode_manager());
        host.register_worker(Arc::new(RecordingWorker {
            name: "terminal_worker".to_string(),
            handles: "Terminal".to_string(),
            output_prefix: "step".to_string(),
            calls: Arc::clone(&calls),
            instructions: Arc::clone(&instructions),
        }));

        let result = host
            .run_task_with_result("Open settings and then enable notifications.")
            .await
            .expect("host execution should succeed");

        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(result.output, "step-2");
        let instructions = instructions
            .lock()
            .expect("instruction lock should work")
            .clone();
        assert_eq!(instructions.len(), 2);
        assert!(instructions[0].contains("Current planned step 1/2"));
        assert!(instructions[1].contains("Current planned step 2/2"));
    }

    #[tokio::test]
    async fn host_agent_routes_cross_app_steps_with_runtime_context_switch() {
        let terminal_calls = Arc::new(AtomicUsize::new(0));
        let browser_calls = Arc::new(AtomicUsize::new(0));
        let terminal_instructions = Arc::new(Mutex::new(Vec::new()));
        let browser_instructions = Arc::new(Mutex::new(Vec::new()));

        let perception = sequence_perception(vec![
            state_for("Terminal"),
            state_for("Terminal"),
            state_for("Safari"),
            state_for("Safari"),
        ]);

        let mut host = HostAgent::new(perception, episode_manager());
        host.register_worker(Arc::new(RecordingWorker {
            name: "terminal_worker".to_string(),
            handles: "Terminal".to_string(),
            output_prefix: "terminal".to_string(),
            calls: Arc::clone(&terminal_calls),
            instructions: Arc::clone(&terminal_instructions),
        }));
        host.register_worker(Arc::new(RecordingWorker {
            name: "browser_worker".to_string(),
            handles: "Safari".to_string(),
            output_prefix: "browser".to_string(),
            calls: Arc::clone(&browser_calls),
            instructions: Arc::clone(&browser_instructions),
        }));

        let result = host
            .run_task_with_result("Open terminal and then check docs in browser.")
            .await
            .expect("host execution should succeed");

        assert_eq!(terminal_calls.load(Ordering::SeqCst), 1);
        assert_eq!(browser_calls.load(Ordering::SeqCst), 1);
        assert_eq!(result.output, "browser-1");

        let terminal_seen = terminal_instructions
            .lock()
            .expect("terminal instruction lock should work")
            .clone();
        let browser_seen = browser_instructions
            .lock()
            .expect("browser instruction lock should work")
            .clone();
        assert_eq!(terminal_seen.len(), 1);
        assert_eq!(browser_seen.len(), 1);
    }

    #[tokio::test]
    async fn host_agent_replans_after_success_without_progress_signal() {
        let calls = Arc::new(AtomicUsize::new(0));
        let results = Arc::new(Mutex::new(VecDeque::from(vec![
            ToolResult {
                success: true,
                output: String::new(),
                error: None,
            },
            ToolResult {
                success: true,
                output: "recovered".to_string(),
                error: None,
            },
        ])));

        let mut host = HostAgent::new(perception_for("Terminal"), episode_manager());
        host.register_worker(Arc::new(SequencedResultWorker {
            name: "terminal_worker".to_string(),
            handles: "Terminal".to_string(),
            calls: Arc::clone(&calls),
            results,
        }));

        let result = host
            .run_task_with_result("Run maintenance")
            .await
            .expect("host should recover through replanning");

        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(result.output, "recovered");
    }

    #[test]
    fn plan_steps_prefers_explicit_list_markers() {
        let goal = "1. Open browser\n2. Search docs\n3. Summarize findings";
        let steps = HostAgent::plan_steps(goal);
        assert_eq!(
            steps,
            vec![
                "Open browser".to_string(),
                "Search docs".to_string(),
                "Summarize findings".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn host_agent_run_task_with_result_uses_first_matching_worker() {
        let mut host = HostAgent::new(perception_for("Terminal"), episode_manager());
        host.register_worker(Arc::new(StubWorker {
            name: "first_terminal_worker".to_string(),
            handles: "Terminal".to_string(),
            result: ToolResult {
                success: true,
                output: "first".to_string(),
                error: None,
            },
        }));
        host.register_worker(Arc::new(StubWorker {
            name: "second_terminal_worker".to_string(),
            handles: "Terminal".to_string(),
            result: ToolResult {
                success: true,
                output: "second".to_string(),
                error: None,
            },
        }));

        let result = host
            .run_task_with_result("type command")
            .await
            .expect("host execution should succeed");
        assert_eq!(result.output, "first");
    }

    #[tokio::test]
    async fn host_agent_run_task_fails_when_no_worker_can_handle_context() {
        let mut host = HostAgent::new(perception_for("Terminal"), episode_manager());
        host.register_worker(Arc::new(StubWorker {
            name: "browser_worker".to_string(),
            handles: "Browser".to_string(),
            result: ToolResult {
                success: true,
                output: "ok".to_string(),
                error: None,
            },
        }));

        let err = host.run_task("type command").await.unwrap_err();
        assert!(err.to_string().contains("no registered worker can handle"));
    }

    #[tokio::test]
    async fn host_agent_run_task_fails_when_worker_returns_unsuccessful_without_error() {
        let mut host = HostAgent::new(perception_for("Terminal"), episode_manager());
        host.register_worker(Arc::new(StubWorker {
            name: "terminal_worker".to_string(),
            handles: "Terminal".to_string(),
            result: ToolResult {
                success: false,
                output: "worker returned unsuccessful".to_string(),
                error: None,
            },
        }));

        let err = host.run_task("type command").await.unwrap_err();
        let message = err.to_string();
        assert!(
            message.contains("escalated while executing planned step")
                || message.contains("worker returned unsuccessful")
        );
    }
}
