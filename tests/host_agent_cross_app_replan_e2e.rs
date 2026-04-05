//! End-to-end HostAgent tests for cross-app decomposition and proactive replanning.

use async_trait::async_trait;
use gloamy::agent::host::HostAgent;
use gloamy::agent::worker::AppWorker;
use gloamy::memory::episode::EpisodeManager;
use gloamy::memory::NoneMemory;
use gloamy::perception::traits::{PerceptionProvider, ScreenState, WidgetNode};
use gloamy::tools::traits::ToolResult;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

struct SequencePerceptionProvider {
    states: Vec<ScreenState>,
    calls: AtomicUsize,
}

#[async_trait]
impl PerceptionProvider for SequencePerceptionProvider {
    fn name(&self) -> &str {
        "integration_sequence_perception"
    }

    async fn capture_state(&self) -> anyhow::Result<ScreenState> {
        let index = self.calls.fetch_add(1, Ordering::SeqCst);
        if let Some(state) = self.states.get(index) {
            return Ok(state.clone());
        }
        self.states
            .last()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("sequence perception requires at least one state"))
    }
}

struct QueueWorker {
    name: String,
    handles: String,
    calls: Arc<AtomicUsize>,
    results: Arc<Mutex<VecDeque<ToolResult>>>,
}

#[async_trait]
impl AppWorker for QueueWorker {
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

fn episode_manager() -> EpisodeManager {
    EpisodeManager::new(
        Arc::new(NoneMemory::new()),
        "integration_session".to_string(),
        "integration_goal".to_string(),
    )
}

#[tokio::test]
async fn e2e_cross_app_decomposition_routes_to_matching_workers() {
    let perception = Arc::new(SequencePerceptionProvider {
        states: vec![
            state_for("Terminal"),
            state_for("Terminal"),
            state_for("Safari"),
            state_for("Safari"),
        ],
        calls: AtomicUsize::new(0),
    });

    let terminal_calls = Arc::new(AtomicUsize::new(0));
    let browser_calls = Arc::new(AtomicUsize::new(0));

    let terminal_worker = QueueWorker {
        name: "terminal_worker".to_string(),
        handles: "Terminal".to_string(),
        calls: Arc::clone(&terminal_calls),
        results: Arc::new(Mutex::new(VecDeque::from(vec![ToolResult {
            success: true,
            output: "terminal-done".to_string(),
            error: None,
        }]))),
    };

    let browser_worker = QueueWorker {
        name: "browser_worker".to_string(),
        handles: "Safari".to_string(),
        calls: Arc::clone(&browser_calls),
        results: Arc::new(Mutex::new(VecDeque::from(vec![ToolResult {
            success: true,
            output: "browser-done".to_string(),
            error: None,
        }]))),
    };

    let mut host = HostAgent::new(perception, episode_manager());
    host.register_worker(Arc::new(terminal_worker));
    host.register_worker(Arc::new(browser_worker));

    let result = host
        .run_task_with_result("Open terminal and then inspect browser tab.")
        .await
        .expect("host should route both app steps");

    assert_eq!(terminal_calls.load(Ordering::SeqCst), 1);
    assert_eq!(browser_calls.load(Ordering::SeqCst), 1);
    assert_eq!(result.output, "browser-done");
}

#[tokio::test]
async fn e2e_replanning_recovers_after_no_progress_step() {
    let perception = Arc::new(SequencePerceptionProvider {
        states: vec![state_for("Terminal")],
        calls: AtomicUsize::new(0),
    });

    let calls = Arc::new(AtomicUsize::new(0));
    let worker = QueueWorker {
        name: "terminal_worker".to_string(),
        handles: "Terminal".to_string(),
        calls: Arc::clone(&calls),
        results: Arc::new(Mutex::new(VecDeque::from(vec![
            ToolResult {
                success: true,
                output: String::new(),
                error: None,
            },
            ToolResult {
                success: true,
                output: "replanned-done".to_string(),
                error: None,
            },
        ]))),
    };

    let mut host = HostAgent::new(perception, episode_manager());
    host.register_worker(Arc::new(worker));

    let result = host
        .run_task_with_result("Run maintenance")
        .await
        .expect("host should replan and recover");

    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(result.output, "replanned-done");
}
