use crate::agent::worker::AppWorker;
use crate::memory::episode::{EpisodeManager, EpisodeStep};
use crate::perception::traits::{PerceptionProvider, ScreenState};
use crate::tools::traits::ToolResult;
use anyhow::{anyhow, Result};
use std::sync::Arc;

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

    /// Plans and routes a single tactical step based on global awareness.
    pub async fn run_task(&mut self, user_goal: &str) -> Result<()> {
        self.run_task_with_result(user_goal).await.map(|_| ())
    }

    /// Plans and routes a single tactical step based on global awareness and returns
    /// the worker result for callers that need direct response content.
    pub async fn run_task_with_result(&mut self, user_goal: &str) -> Result<ToolResult> {
        if user_goal.trim().is_empty() {
            return Err(anyhow!("user goal cannot be empty"));
        }

        let state_before = self.perception.capture_state().await?;
        let active_app = self.infer_active_application(&state_before)?;
        let worker = self
            .workers
            .iter()
            .find(|candidate| candidate.can_handle(&active_app))
            .cloned()
            .ok_or_else(|| {
                anyhow!(
                    "no registered worker can handle application context '{}'",
                    active_app
                )
            })?;

        let step_index = self.episode_manager.next_step_index();
        let execution = worker.execute_step(user_goal, &state_before).await;
        let state_after = self.perception.capture_state().await.ok();

        let (step, worker_result, worker_error) = match execution {
            Ok(result) => {
                let failure_reason = if result.success {
                    None
                } else {
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
                };
                let error = failure_reason
                    .as_ref()
                    .map(|reason| anyhow!(reason.clone()));

                (
                    EpisodeStep {
                        step_index,
                        action_taken: user_goal.to_string(),
                        action_result: result.output.clone(),
                        screen_state_before: Some(state_before),
                        screen_state_after: state_after,
                        execution_error: failure_reason.clone(),
                    },
                    Some(result),
                    error,
                )
            }
            Err(err) => {
                let error_string = err.to_string();
                (
                    EpisodeStep {
                        step_index,
                        action_taken: user_goal.to_string(),
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

        let completed_without_error = worker_error.is_none();
        self.episode_manager.record_step(step);
        self.episode_manager.flush().await?;

        if completed_without_error {
            self.episode_manager
                .promote_to_trajectory(&active_app)
                .await?;
            return Ok(worker_result.unwrap_or(ToolResult {
                success: true,
                output: String::new(),
                error: None,
            }));
        }

        if let Some(err) = worker_error {
            return Err(err.context(format!(
                "worker '{}' failed while executing task for '{}'",
                worker.name(),
                active_app
            )));
        }

        Err(anyhow!(
            "worker '{}' failed while executing task for '{}'",
            worker.name(),
            active_app
        ))
    }

    fn infer_active_application(&self, state: &ScreenState) -> Result<String> {
        if let Some(tree) = &state.widget_tree {
            if let Some(name) = tree
                .name
                .as_deref()
                .map(str::trim)
                .filter(|name| !name.is_empty())
            {
                return Ok(name.to_string());
            }
            let id = tree.id.trim();
            if !id.is_empty() {
                return Ok(id.to_string());
            }
        }
        Ok("unknown".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::NoneMemory;
    use crate::perception::traits::WidgetNode;
    use crate::tools::traits::ToolResult;
    use async_trait::async_trait;

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

    fn perception_for(app_name: &str) -> Arc<dyn PerceptionProvider> {
        Arc::new(StaticPerceptionProvider {
            state: ScreenState {
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
            },
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
        assert!(err
            .to_string()
            .contains("worker 'terminal_worker' failed while executing task"));
    }
}
