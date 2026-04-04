use crate::agent::lesson;
use crate::agent::loop_::run_tool_call_loop;
use crate::approval::ApprovalManager;
use crate::memory::Memory;
use crate::observability::Observer;
use crate::perception::traits::ScreenState;
use crate::providers::{ChatMessage, Provider};
use crate::tools::traits::{Tool, ToolResult};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Represents an AppAgent (Worker) specialized for a specific application or tactical execution.
/// Follows the AgentOS/Compositional multi-agent paradigm for cognitive separation.
#[async_trait]
pub trait AppWorker: Send + Sync {
    /// Returns the name of the worker (e.g., "browser_agent", "terminal_agent").
    fn name(&self) -> &str;

    /// Checks if this worker is equipped to handle the current application context.
    fn can_handle(&self, application_context: &str) -> bool;

    /// Executes a delegated tactical step based on the fused screen state and instructions.
    async fn execute_step(
        &self,
        task_instruction: &str,
        current_state: &ScreenState,
    ) -> anyhow::Result<ToolResult>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkerContextStrategy {
    Any,
    TerminalLike,
    Exact(Vec<String>),
}

impl WorkerContextStrategy {
    fn matches(&self, application_context: &str) -> bool {
        match self {
            Self::Any => true,
            Self::TerminalLike => is_terminal_like_context(application_context),
            Self::Exact(contexts) => contexts
                .iter()
                .any(|context| context.eq_ignore_ascii_case(application_context.trim())),
        }
    }
}

fn is_terminal_like_context(application_context: &str) -> bool {
    let normalized = application_context.trim().to_ascii_lowercase();
    if normalized.is_empty() || normalized == "unknown" {
        return true;
    }

    const TERMINAL_NAMES: &[&str] = &[
        "terminal",
        "iterm",
        "iterm2",
        "warp",
        "wezterm",
        "alacritty",
        "kitty",
        "ghostty",
        "hyper",
        "rio",
        "tabby",
        "console",
        "shell",
        "code",
        "cursor",
        "codium",
        "zed",
    ];

    TERMINAL_NAMES.iter().any(|name| normalized == *name)
        || normalized.contains("terminal")
        || normalized.contains("shell")
        || normalized.contains("tty")
        || normalized.contains("console")
}

/// Production worker that runs the existing tool-call loop as a routed tactical step.
pub struct ToolLoopWorker {
    name: String,
    context_strategy: WorkerContextStrategy,
    provider: Arc<dyn Provider>,
    tools_registry: Arc<Vec<Box<dyn Tool>>>,
    observer: Arc<dyn Observer>,
    provider_name: String,
    model_name: String,
    temperature: f64,
    silent: bool,
    channel_name: String,
    multimodal_config: crate::config::MultimodalConfig,
    max_tool_iterations: usize,
    system_prompt: String,
    approval_manager: Option<Arc<ApprovalManager>>,
    self_learning: bool,
    memory: Arc<dyn Memory>,
}

impl ToolLoopWorker {
    #[allow(clippy::too_many_arguments)]
    fn with_context_strategy(
        name: String,
        context_strategy: WorkerContextStrategy,
        provider: Arc<dyn Provider>,
        tools_registry: Arc<Vec<Box<dyn Tool>>>,
        observer: Arc<dyn Observer>,
        provider_name: String,
        model_name: String,
        temperature: f64,
        silent: bool,
        channel_name: String,
        multimodal_config: crate::config::MultimodalConfig,
        max_tool_iterations: usize,
        system_prompt: String,
        approval_manager: Option<Arc<ApprovalManager>>,
        self_learning: bool,
        memory: Arc<dyn Memory>,
    ) -> Self {
        Self {
            name,
            context_strategy,
            provider,
            tools_registry,
            observer,
            provider_name,
            model_name,
            temperature,
            silent,
            channel_name,
            multimodal_config,
            max_tool_iterations,
            system_prompt,
            approval_manager,
            self_learning,
            memory,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider: Arc<dyn Provider>,
        tools_registry: Arc<Vec<Box<dyn Tool>>>,
        observer: Arc<dyn Observer>,
        provider_name: String,
        model_name: String,
        temperature: f64,
        silent: bool,
        channel_name: String,
        multimodal_config: crate::config::MultimodalConfig,
        max_tool_iterations: usize,
        system_prompt: String,
        approval_manager: Option<Arc<ApprovalManager>>,
        self_learning: bool,
        memory: Arc<dyn Memory>,
    ) -> Self {
        Self::with_context_strategy(
            "tool_loop_worker".to_string(),
            WorkerContextStrategy::Any,
            provider,
            tools_registry,
            observer,
            provider_name,
            model_name,
            temperature,
            silent,
            channel_name,
            multimodal_config,
            max_tool_iterations,
            system_prompt,
            approval_manager,
            self_learning,
            memory,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn for_terminal_host(
        provider: Arc<dyn Provider>,
        tools_registry: Arc<Vec<Box<dyn Tool>>>,
        observer: Arc<dyn Observer>,
        provider_name: String,
        model_name: String,
        temperature: f64,
        silent: bool,
        channel_name: String,
        multimodal_config: crate::config::MultimodalConfig,
        max_tool_iterations: usize,
        system_prompt: String,
        approval_manager: Option<Arc<ApprovalManager>>,
        self_learning: bool,
        memory: Arc<dyn Memory>,
    ) -> Self {
        Self::with_context_strategy(
            "terminal_tool_loop_worker".to_string(),
            WorkerContextStrategy::TerminalLike,
            provider,
            tools_registry,
            observer,
            provider_name,
            model_name,
            temperature,
            silent,
            channel_name,
            multimodal_config,
            max_tool_iterations,
            system_prompt,
            approval_manager,
            self_learning,
            memory,
        )
    }
}

#[async_trait]
impl AppWorker for ToolLoopWorker {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_handle(&self, application_context: &str) -> bool {
        self.context_strategy.matches(application_context)
    }

    async fn execute_step(
        &self,
        task_instruction: &str,
        _current_state: &ScreenState,
    ) -> anyhow::Result<ToolResult> {
        let mut history = vec![
            ChatMessage::system(self.system_prompt.clone()),
            ChatMessage::user(task_instruction.to_string()),
        ];
        let mut tool_outcomes = Vec::new();
        let response = run_tool_call_loop(
            self.provider.as_ref(),
            &mut history,
            self.tools_registry.as_ref(),
            self.observer.as_ref(),
            &self.provider_name,
            &self.model_name,
            self.temperature,
            self.silent,
            self.approval_manager.as_deref(),
            &self.channel_name,
            &self.multimodal_config,
            self.max_tool_iterations,
            None,
            None,
            None,
            &[],
            None,
            if self.self_learning {
                Some(&mut tool_outcomes)
            } else {
                None
            },
        )
        .await?;

        if self.self_learning && !tool_outcomes.is_empty() {
            let lessons = lesson::extract_lessons(&tool_outcomes, task_instruction);
            if !lessons.is_empty() {
                let _ = lesson::persist_lessons(self.memory.as_ref(), &lessons).await;
            }
        }

        Ok(ToolResult {
            success: true,
            output: response,
            error: None,
        })
    }
}

/// Stateful worker that preserves conversation history across HostAgent turns.
pub struct ConversationToolLoopWorker {
    name: String,
    context_strategy: WorkerContextStrategy,
    history: Arc<Mutex<Vec<ChatMessage>>>,
    provider: Arc<dyn Provider>,
    tools_registry: Arc<Vec<Box<dyn Tool>>>,
    observer: Arc<dyn Observer>,
    provider_name: String,
    model_name: String,
    temperature: f64,
    silent: bool,
    approval_manager: Option<Arc<ApprovalManager>>,
    channel_name: String,
    multimodal_config: crate::config::MultimodalConfig,
    max_tool_iterations: usize,
    self_learning: bool,
    memory: Arc<dyn Memory>,
}

impl ConversationToolLoopWorker {
    #[allow(clippy::too_many_arguments)]
    pub fn for_terminal_host(
        history: Arc<Mutex<Vec<ChatMessage>>>,
        provider: Arc<dyn Provider>,
        tools_registry: Arc<Vec<Box<dyn Tool>>>,
        observer: Arc<dyn Observer>,
        provider_name: String,
        model_name: String,
        temperature: f64,
        silent: bool,
        approval_manager: Option<Arc<ApprovalManager>>,
        channel_name: String,
        multimodal_config: crate::config::MultimodalConfig,
        max_tool_iterations: usize,
        self_learning: bool,
        memory: Arc<dyn Memory>,
    ) -> Self {
        Self {
            name: "terminal_conversation_tool_loop_worker".to_string(),
            context_strategy: WorkerContextStrategy::TerminalLike,
            history,
            provider,
            tools_registry,
            observer,
            provider_name,
            model_name,
            temperature,
            silent,
            approval_manager,
            channel_name,
            multimodal_config,
            max_tool_iterations,
            self_learning,
            memory,
        }
    }
}

#[async_trait]
impl AppWorker for ConversationToolLoopWorker {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_handle(&self, application_context: &str) -> bool {
        self.context_strategy.matches(application_context)
    }

    async fn execute_step(
        &self,
        task_instruction: &str,
        _current_state: &ScreenState,
    ) -> anyhow::Result<ToolResult> {
        let mut history = {
            let mut shared_history = self.history.lock().await;
            std::mem::take(&mut *shared_history)
        };

        let needs_user_turn = !history.last().is_some_and(|message| {
            message.role == "user" && message.content.trim() == task_instruction.trim()
        });
        if needs_user_turn {
            history.push(ChatMessage::user(task_instruction.to_string()));
        }

        let mut tool_outcomes = Vec::new();
        let response = run_tool_call_loop(
            self.provider.as_ref(),
            &mut history,
            self.tools_registry.as_ref(),
            self.observer.as_ref(),
            &self.provider_name,
            &self.model_name,
            self.temperature,
            self.silent,
            self.approval_manager.as_deref(),
            &self.channel_name,
            &self.multimodal_config,
            self.max_tool_iterations,
            None,
            None,
            None,
            &[],
            None,
            if self.self_learning {
                Some(&mut tool_outcomes)
            } else {
                None
            },
        )
        .await;

        {
            let mut shared_history = self.history.lock().await;
            *shared_history = history;
        }

        let output = response?;

        if self.self_learning && !tool_outcomes.is_empty() {
            let lessons = lesson::extract_lessons(&tool_outcomes, task_instruction);
            if !lessons.is_empty() {
                let _ = lesson::persist_lessons(self.memory.as_ref(), &lessons).await;
            }
        }

        Ok(ToolResult {
            success: true,
            output,
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::NoneMemory;
    use crate::observability::NoopObserver;
    use crate::tools::traits::ToolResult;

    struct StaticProvider;

    #[async_trait]
    impl Provider for StaticProvider {
        async fn chat_with_system(
            &self,
            _system_prompt: Option<&str>,
            _message: &str,
            _model: &str,
            _temperature: f64,
        ) -> anyhow::Result<String> {
            Ok("worker-result".to_string())
        }
    }

    struct NoopTool;

    #[async_trait]
    impl Tool for NoopTool {
        fn name(&self) -> &str {
            "noop"
        }

        fn description(&self) -> &str {
            "No-op test tool"
        }

        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({"type":"object","properties":{}})
        }

        async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<ToolResult> {
            Ok(ToolResult {
                success: true,
                output: "ok".to_string(),
                error: None,
            })
        }
    }

    #[tokio::test]
    async fn tool_loop_worker_executes_base_loop() {
        let worker = ToolLoopWorker::new(
            Arc::new(StaticProvider),
            Arc::new(vec![Box::new(NoopTool)]),
            Arc::new(NoopObserver),
            "mock-provider".to_string(),
            "mock-model".to_string(),
            0.0,
            true,
            "cli".to_string(),
            crate::config::MultimodalConfig::default(),
            8,
            "system".to_string(),
            None,
            false,
            Arc::new(NoneMemory::new()),
        );

        let result = worker
            .execute_step(
                "return final response",
                &ScreenState {
                    screenshot_path: None,
                    widget_tree: None,
                    extracted_text: Vec::new(),
                },
            )
            .await
            .expect("worker should return tool result");

        assert!(result.success);
        assert_eq!(result.output, "worker-result");
    }

    #[test]
    fn terminal_context_strategy_matches_terminal_like_inputs() {
        assert!(WorkerContextStrategy::TerminalLike.matches("Terminal"));
        assert!(WorkerContextStrategy::TerminalLike.matches("iTerm2"));
        assert!(WorkerContextStrategy::TerminalLike.matches("unknown"));
        assert!(!WorkerContextStrategy::TerminalLike.matches("Safari"));
    }

    #[tokio::test]
    async fn conversation_worker_preserves_shared_history() {
        let history = Arc::new(Mutex::new(vec![
            ChatMessage::system("system"),
            ChatMessage::user("prior turn"),
            ChatMessage::assistant("prior response"),
        ]));
        let worker = ConversationToolLoopWorker::for_terminal_host(
            Arc::clone(&history),
            Arc::new(StaticProvider),
            Arc::new(vec![Box::new(NoopTool)]),
            Arc::new(NoopObserver),
            "mock-provider".to_string(),
            "mock-model".to_string(),
            0.0,
            true,
            None,
            "cli".to_string(),
            crate::config::MultimodalConfig::default(),
            8,
            false,
            Arc::new(NoneMemory::new()),
        );

        let result = worker
            .execute_step(
                "return final response",
                &ScreenState {
                    screenshot_path: None,
                    widget_tree: None,
                    extracted_text: Vec::new(),
                },
            )
            .await
            .expect("conversation worker should return tool result");

        assert!(result.success);
        let shared_history = history.lock().await;
        assert_eq!(
            shared_history.first().map(|message| message.role.as_str()),
            Some("system")
        );
        assert!(shared_history.iter().any(|message| {
            message.role == "user" && message.content == "return final response"
        }));
    }
}
