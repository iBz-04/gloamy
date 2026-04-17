use crate::agent::lesson;
use crate::agent::loop_::run_tool_call_loop;
use crate::approval::ApprovalManager;
use crate::memory::Memory;
use crate::observability::Observer;
use crate::perception::traits::ScreenState;
use crate::providers::{ChatMessage, Provider};
use crate::tools::traits::{Tool, ToolResult};
use async_trait::async_trait;
use std::fmt::Write as _;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerCapabilityContract {
    pub app_family: &'static str,
    pub preferred_tools: &'static [&'static str],
    pub restricted_tools: &'static [&'static str],
    pub escalation_policy: &'static str,
}

impl WorkerCapabilityContract {
    fn generic() -> Self {
        Self {
            app_family: "generic",
            preferred_tools: &[],
            restricted_tools: &[],
            escalation_policy: "Escalate when no safe tool path can complete the step.",
        }
    }
}

/// Represents an AppAgent (Worker) specialized for a specific application.
#[async_trait]
pub trait AppWorker: Send + Sync {
    /// Returns the stable worker identifier.
    fn name(&self) -> &str;

    /// Checks if this worker is equipped to handle the current application context.
    fn can_handle(&self, application_context: &str) -> bool;

    /// Returns the explicit capability contract used by this worker.
    fn capability_contract(&self) -> WorkerCapabilityContract {
        WorkerCapabilityContract::generic()
    }

    /// Executes one delegated tactical step.
    async fn execute_step(
        &self,
        task_instruction: &str,
        current_state: &ScreenState,
    ) -> anyhow::Result<ToolResult>;
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
    ];

    TERMINAL_NAMES.iter().any(|name| normalized == *name)
        || normalized.contains("terminal")
        || normalized.contains("shell")
        || normalized.contains("tty")
        || normalized.contains("console")
}

fn is_browser_like_context(application_context: &str) -> bool {
    let normalized = application_context.trim().to_ascii_lowercase();
    if normalized.is_empty() || normalized == "unknown" {
        return false;
    }

    const BROWSER_NAMES: &[&str] = &[
        "safari",
        "google chrome",
        "chrome",
        "chromium",
        "arc",
        "firefox",
        "brave",
        "edge",
        "opera",
        "vivaldi",
    ];

    BROWSER_NAMES.iter().any(|name| normalized == *name)
        || normalized.contains("browser")
        || normalized.contains("chrome")
        || normalized.contains("safari")
        || normalized.contains("firefox")
}

fn is_editor_like_context(application_context: &str) -> bool {
    let normalized = application_context.trim().to_ascii_lowercase();
    if normalized.is_empty() || normalized == "unknown" {
        return false;
    }

    const EDITOR_NAMES: &[&str] = &[
        "code",
        "visual studio code",
        "cursor",
        "codium",
        "zed",
        "jetbrains",
        "intellij idea",
        "clion",
        "goland",
        "pycharm",
        "webstorm",
        "vim",
        "nvim",
        "neovim",
        "emacs",
    ];

    EDITOR_NAMES.iter().any(|name| normalized == *name)
        || normalized.contains("editor")
        || normalized.contains("code")
        || normalized.contains("cursor")
        || normalized.contains("idea")
        || normalized.contains("jetbrains")
}

fn csv_or_none(items: &[&str]) -> String {
    if items.is_empty() {
        "none".to_string()
    } else {
        items.join(", ")
    }
}

fn augment_instruction_with_contract(
    task_instruction: &str,
    contract: &WorkerCapabilityContract,
    policy_prompt: &str,
) -> String {
    let trimmed = task_instruction.trim();
    let mut instruction = if trimmed.is_empty() {
        String::new()
    } else {
        trimmed.to_string()
    };

    if !instruction.is_empty() {
        instruction.push_str("\n\n");
    }

    instruction.push_str("[Worker capability contract]\n");
    writeln!(instruction, "- App family: {}", contract.app_family)
        .expect("writing app family to worker instruction");
    writeln!(
        instruction,
        "- Preferred tools: {}",
        csv_or_none(contract.preferred_tools)
    )
    .expect("writing preferred tools to worker instruction");
    writeln!(
        instruction,
        "- Restricted tools: {}",
        csv_or_none(contract.restricted_tools)
    )
    .expect("writing restricted tools to worker instruction");
    writeln!(
        instruction,
        "- Escalation policy: {}",
        contract.escalation_policy
    )
    .expect("writing escalation policy to worker instruction");
    instruction.push_str("\n[Worker execution policy]\n");
    instruction.push_str(policy_prompt.trim());
    instruction
}

const TERMINAL_POLICY_PROMPT: &str = "Operate as a terminal specialist. Prefer shell and file operations. Avoid GUI and browser tools unless the current step explicitly requires UI interaction.";
const BROWSER_POLICY_PROMPT: &str = "Operate as a browser specialist. Prefer the browser tool for web interactions. Use snapshot/refs when possible; when the UI requires visual navigation, use the browser tool's computer-use actions (mouse_move, mouse_click, mouse_drag, key_type, key_press, screen_capture) instead of inventing a separate mouse workflow. Use perception_capture only when you need a fresh screen readback. Avoid shell-heavy actions unless local validation is required by the step.";
const EDITOR_POLICY_PROMPT: &str = "Operate as an editor specialist. Prefer file_read, file_write, and validation commands in the active workspace. Avoid unrelated browser navigation unless the step explicitly asks for it.";
const FALLBACK_POLICY_PROMPT: &str = "Operate as a general recovery specialist. Use the safest available tool sequence and escalate when no reliable app-specific route exists.";

fn terminal_contract() -> WorkerCapabilityContract {
    WorkerCapabilityContract {
        app_family: "terminal",
        preferred_tools: &["shell", "file_read", "file_write", "memory_recall"],
        restricted_tools: &["browser_open", "mac_automation"],
        escalation_policy:
            "Escalate when command execution is blocked by policy, permission, or missing runtime dependencies.",
    }
}

fn browser_contract() -> WorkerCapabilityContract {
    WorkerCapabilityContract {
        app_family: "browser",
        preferred_tools: &["browser", "perception_capture"],
        restricted_tools: &["shell"],
        escalation_policy:
            "Escalate when browser navigation depends on unavailable credentials, permissions, blocked domains, or the browser tool cannot access the required UI state.",
    }
}

fn editor_contract() -> WorkerCapabilityContract {
    WorkerCapabilityContract {
        app_family: "editor",
        preferred_tools: &["file_read", "file_write", "shell", "memory_recall"],
        restricted_tools: &["browser_open"],
        escalation_policy:
            "Escalate when workspace writes are blocked or when validation fails without a safe automated repair path.",
    }
}

fn fallback_contract() -> WorkerCapabilityContract {
    WorkerCapabilityContract {
        app_family: "fallback",
        preferred_tools: &["memory_recall", "file_read", "shell"],
        restricted_tools: &[],
        escalation_policy:
            "Escalate when context classification remains ambiguous after one recovery attempt.",
    }
}

fn format_screen_state(state: &ScreenState) -> String {
    let mut out = String::from("\n\n[Current Screen State]\n");
    if let Some(path) = &state.screenshot_path {
        let _ = writeln!(out, "Screenshot: ![]({path})");
    }
    if let Some(tree) = &state.widget_tree {
        out.push_str("Widget Tree (Accessibility):\n```json\n");
        if let Ok(json) = serde_json::to_string_pretty(tree) {
            out.push_str(&json);
        }
        out.push_str("\n```\n");
    }
    if !state.extracted_text.is_empty() {
        out.push_str("Extracted Text (OCR):\n```json\n");
        if let Ok(json) = serde_json::to_string_pretty(&state.extracted_text) {
            out.push_str(&json);
        }
        out.push_str("\n```\n");
    }
    out
}

struct ToolLoopWorkerCore {
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
    click_at_preflight: crate::config::ClickAtPreflightMode,
}

impl ToolLoopWorkerCore {
    #[allow(clippy::too_many_arguments)]
    fn new(
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
        click_at_preflight: crate::config::ClickAtPreflightMode,
    ) -> Self {
        Self {
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
            click_at_preflight,
        }
    }

    async fn execute_step(
        &self,
        task_instruction: &str,
        current_state: &ScreenState,
    ) -> anyhow::Result<ToolResult> {
        let instruction_with_state =
            format!("{}{}", task_instruction, format_screen_state(current_state));
        let mut history = vec![
            ChatMessage::system(self.system_prompt.clone()),
            ChatMessage::user(instruction_with_state),
        ];
        let mut tool_outcomes = Vec::new();
        let lesson_ctx = lesson::LessonPersistCtx {
            memory: self.memory.as_ref(),
            user_message: task_instruction,
        };
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
            self.click_at_preflight,
            if self.self_learning {
                Some(&lesson_ctx)
            } else {
                None
            },
        )
        .await?;

        Ok(ToolResult {
            success: true,
            output: response,
            error: None,
        })
    }
}

struct ConversationToolLoopWorkerCore {
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
    click_at_preflight: crate::config::ClickAtPreflightMode,
}

impl ConversationToolLoopWorkerCore {
    #[allow(clippy::too_many_arguments)]
    fn new(
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
        click_at_preflight: crate::config::ClickAtPreflightMode,
    ) -> Self {
        Self {
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
            click_at_preflight,
        }
    }

    async fn execute_step(
        &self,
        task_instruction: &str,
        current_state: &ScreenState,
    ) -> anyhow::Result<ToolResult> {
        let mut history = {
            let mut shared_history = self.history.lock().await;
            std::mem::take(&mut *shared_history)
        };

        let instruction_with_state =
            format!("{}{}", task_instruction, format_screen_state(current_state));
        let needs_user_turn = !history.last().is_some_and(|message| {
            message.role == "user" && message.content.trim() == instruction_with_state.trim()
        });
        if needs_user_turn {
            history.push(ChatMessage::user(instruction_with_state));
        }

        let mut tool_outcomes = Vec::new();
        let lesson_ctx = lesson::LessonPersistCtx {
            memory: self.memory.as_ref(),
            user_message: task_instruction,
        };
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
            self.click_at_preflight,
            if self.self_learning {
                Some(&lesson_ctx)
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

        Ok(ToolResult {
            success: true,
            output,
            error: None,
        })
    }
}

pub struct TerminalToolLoopWorker {
    core: ToolLoopWorkerCore,
}

impl TerminalToolLoopWorker {
    #[allow(clippy::too_many_arguments)]
    fn new(
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
        click_at_preflight: crate::config::ClickAtPreflightMode,
    ) -> Self {
        Self {
            core: ToolLoopWorkerCore::new(
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
                click_at_preflight,
            ),
        }
    }
}

#[async_trait]
impl AppWorker for TerminalToolLoopWorker {
    fn name(&self) -> &str {
        "terminal_tool_loop_worker"
    }

    fn can_handle(&self, application_context: &str) -> bool {
        is_terminal_like_context(application_context)
    }

    fn capability_contract(&self) -> WorkerCapabilityContract {
        terminal_contract()
    }

    async fn execute_step(
        &self,
        task_instruction: &str,
        current_state: &ScreenState,
    ) -> anyhow::Result<ToolResult> {
        let instruction = augment_instruction_with_contract(
            task_instruction,
            &self.capability_contract(),
            TERMINAL_POLICY_PROMPT,
        );
        self.core.execute_step(&instruction, current_state).await
    }
}

pub struct BrowserToolLoopWorker {
    core: ToolLoopWorkerCore,
}

impl BrowserToolLoopWorker {
    #[allow(clippy::too_many_arguments)]
    fn new(
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
        click_at_preflight: crate::config::ClickAtPreflightMode,
    ) -> Self {
        Self {
            core: ToolLoopWorkerCore::new(
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
                click_at_preflight,
            ),
        }
    }
}

#[async_trait]
impl AppWorker for BrowserToolLoopWorker {
    fn name(&self) -> &str {
        "browser_tool_loop_worker"
    }

    fn can_handle(&self, application_context: &str) -> bool {
        is_browser_like_context(application_context)
    }

    fn capability_contract(&self) -> WorkerCapabilityContract {
        browser_contract()
    }

    async fn execute_step(
        &self,
        task_instruction: &str,
        current_state: &ScreenState,
    ) -> anyhow::Result<ToolResult> {
        let instruction = augment_instruction_with_contract(
            task_instruction,
            &self.capability_contract(),
            BROWSER_POLICY_PROMPT,
        );
        self.core.execute_step(&instruction, current_state).await
    }
}

pub struct EditorToolLoopWorker {
    core: ToolLoopWorkerCore,
}

impl EditorToolLoopWorker {
    #[allow(clippy::too_many_arguments)]
    fn new(
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
        click_at_preflight: crate::config::ClickAtPreflightMode,
    ) -> Self {
        Self {
            core: ToolLoopWorkerCore::new(
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
                click_at_preflight,
            ),
        }
    }
}

#[async_trait]
impl AppWorker for EditorToolLoopWorker {
    fn name(&self) -> &str {
        "editor_tool_loop_worker"
    }

    fn can_handle(&self, application_context: &str) -> bool {
        is_editor_like_context(application_context)
    }

    fn capability_contract(&self) -> WorkerCapabilityContract {
        editor_contract()
    }

    async fn execute_step(
        &self,
        task_instruction: &str,
        current_state: &ScreenState,
    ) -> anyhow::Result<ToolResult> {
        let instruction = augment_instruction_with_contract(
            task_instruction,
            &self.capability_contract(),
            EDITOR_POLICY_PROMPT,
        );
        self.core.execute_step(&instruction, current_state).await
    }
}

pub struct FallbackToolLoopWorker {
    core: ToolLoopWorkerCore,
}

impl FallbackToolLoopWorker {
    #[allow(clippy::too_many_arguments)]
    fn new(
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
        click_at_preflight: crate::config::ClickAtPreflightMode,
    ) -> Self {
        Self {
            core: ToolLoopWorkerCore::new(
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
                click_at_preflight,
            ),
        }
    }
}

#[async_trait]
impl AppWorker for FallbackToolLoopWorker {
    fn name(&self) -> &str {
        "fallback_tool_loop_worker"
    }

    fn can_handle(&self, _application_context: &str) -> bool {
        true
    }

    fn capability_contract(&self) -> WorkerCapabilityContract {
        fallback_contract()
    }

    async fn execute_step(
        &self,
        task_instruction: &str,
        current_state: &ScreenState,
    ) -> anyhow::Result<ToolResult> {
        let instruction = augment_instruction_with_contract(
            task_instruction,
            &self.capability_contract(),
            FALLBACK_POLICY_PROMPT,
        );
        self.core.execute_step(&instruction, current_state).await
    }
}

pub struct TerminalConversationToolLoopWorker {
    core: ConversationToolLoopWorkerCore,
}

impl TerminalConversationToolLoopWorker {
    #[allow(clippy::too_many_arguments)]
    fn new(
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
        click_at_preflight: crate::config::ClickAtPreflightMode,
    ) -> Self {
        Self {
            core: ConversationToolLoopWorkerCore::new(
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
                click_at_preflight,
            ),
        }
    }
}

#[async_trait]
impl AppWorker for TerminalConversationToolLoopWorker {
    fn name(&self) -> &str {
        "terminal_conversation_tool_loop_worker"
    }

    fn can_handle(&self, application_context: &str) -> bool {
        is_terminal_like_context(application_context)
    }

    fn capability_contract(&self) -> WorkerCapabilityContract {
        terminal_contract()
    }

    async fn execute_step(
        &self,
        task_instruction: &str,
        current_state: &ScreenState,
    ) -> anyhow::Result<ToolResult> {
        let instruction = augment_instruction_with_contract(
            task_instruction,
            &self.capability_contract(),
            TERMINAL_POLICY_PROMPT,
        );
        self.core.execute_step(&instruction, current_state).await
    }
}

pub struct BrowserConversationToolLoopWorker {
    core: ConversationToolLoopWorkerCore,
}

impl BrowserConversationToolLoopWorker {
    #[allow(clippy::too_many_arguments)]
    fn new(
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
        click_at_preflight: crate::config::ClickAtPreflightMode,
    ) -> Self {
        Self {
            core: ConversationToolLoopWorkerCore::new(
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
                click_at_preflight,
            ),
        }
    }
}

#[async_trait]
impl AppWorker for BrowserConversationToolLoopWorker {
    fn name(&self) -> &str {
        "browser_conversation_tool_loop_worker"
    }

    fn can_handle(&self, application_context: &str) -> bool {
        is_browser_like_context(application_context)
    }

    fn capability_contract(&self) -> WorkerCapabilityContract {
        browser_contract()
    }

    async fn execute_step(
        &self,
        task_instruction: &str,
        current_state: &ScreenState,
    ) -> anyhow::Result<ToolResult> {
        let instruction = augment_instruction_with_contract(
            task_instruction,
            &self.capability_contract(),
            BROWSER_POLICY_PROMPT,
        );
        self.core.execute_step(&instruction, current_state).await
    }
}

pub struct EditorConversationToolLoopWorker {
    core: ConversationToolLoopWorkerCore,
}

impl EditorConversationToolLoopWorker {
    #[allow(clippy::too_many_arguments)]
    fn new(
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
        click_at_preflight: crate::config::ClickAtPreflightMode,
    ) -> Self {
        Self {
            core: ConversationToolLoopWorkerCore::new(
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
                click_at_preflight,
            ),
        }
    }
}

#[async_trait]
impl AppWorker for EditorConversationToolLoopWorker {
    fn name(&self) -> &str {
        "editor_conversation_tool_loop_worker"
    }

    fn can_handle(&self, application_context: &str) -> bool {
        is_editor_like_context(application_context)
    }

    fn capability_contract(&self) -> WorkerCapabilityContract {
        editor_contract()
    }

    async fn execute_step(
        &self,
        task_instruction: &str,
        current_state: &ScreenState,
    ) -> anyhow::Result<ToolResult> {
        let instruction = augment_instruction_with_contract(
            task_instruction,
            &self.capability_contract(),
            EDITOR_POLICY_PROMPT,
        );
        self.core.execute_step(&instruction, current_state).await
    }
}

pub struct FallbackConversationToolLoopWorker {
    core: ConversationToolLoopWorkerCore,
}

impl FallbackConversationToolLoopWorker {
    #[allow(clippy::too_many_arguments)]
    fn new(
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
        click_at_preflight: crate::config::ClickAtPreflightMode,
    ) -> Self {
        Self {
            core: ConversationToolLoopWorkerCore::new(
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
                click_at_preflight,
            ),
        }
    }
}

#[async_trait]
impl AppWorker for FallbackConversationToolLoopWorker {
    fn name(&self) -> &str {
        "fallback_conversation_tool_loop_worker"
    }

    fn can_handle(&self, _application_context: &str) -> bool {
        true
    }

    fn capability_contract(&self) -> WorkerCapabilityContract {
        fallback_contract()
    }

    async fn execute_step(
        &self,
        task_instruction: &str,
        current_state: &ScreenState,
    ) -> anyhow::Result<ToolResult> {
        let instruction = augment_instruction_with_contract(
            task_instruction,
            &self.capability_contract(),
            FALLBACK_POLICY_PROMPT,
        );
        self.core.execute_step(&instruction, current_state).await
    }
}

/// Factory namespace for stateless host workers.
pub struct ToolLoopWorker;

impl ToolLoopWorker {
    #[allow(clippy::new_ret_no_self)]
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
        click_at_preflight: crate::config::ClickAtPreflightMode,
    ) -> FallbackToolLoopWorker {
        FallbackToolLoopWorker::new(
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
            click_at_preflight,
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
        click_at_preflight: crate::config::ClickAtPreflightMode,
    ) -> TerminalToolLoopWorker {
        TerminalToolLoopWorker::new(
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
            click_at_preflight,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn for_browser_host(
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
        click_at_preflight: crate::config::ClickAtPreflightMode,
    ) -> BrowserToolLoopWorker {
        BrowserToolLoopWorker::new(
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
            click_at_preflight,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn for_editor_host(
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
        click_at_preflight: crate::config::ClickAtPreflightMode,
    ) -> EditorToolLoopWorker {
        EditorToolLoopWorker::new(
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
            click_at_preflight,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn for_fallback_host(
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
        click_at_preflight: crate::config::ClickAtPreflightMode,
    ) -> FallbackToolLoopWorker {
        FallbackToolLoopWorker::new(
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
            click_at_preflight,
        )
    }
}

/// Factory namespace for stateful conversation host workers.
pub struct ConversationToolLoopWorker;

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
        click_at_preflight: crate::config::ClickAtPreflightMode,
    ) -> TerminalConversationToolLoopWorker {
        TerminalConversationToolLoopWorker::new(
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
            click_at_preflight,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn for_browser_host(
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
        click_at_preflight: crate::config::ClickAtPreflightMode,
    ) -> BrowserConversationToolLoopWorker {
        BrowserConversationToolLoopWorker::new(
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
            click_at_preflight,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn for_editor_host(
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
        click_at_preflight: crate::config::ClickAtPreflightMode,
    ) -> EditorConversationToolLoopWorker {
        EditorConversationToolLoopWorker::new(
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
            click_at_preflight,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn for_fallback_host(
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
        click_at_preflight: crate::config::ClickAtPreflightMode,
    ) -> FallbackConversationToolLoopWorker {
        FallbackConversationToolLoopWorker::new(
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
            click_at_preflight,
        )
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
            crate::config::ClickAtPreflightMode::default(),
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
    fn terminal_context_detection_matches_terminal_like_inputs() {
        assert!(is_terminal_like_context("Terminal"));
        assert!(is_terminal_like_context("iTerm2"));
        assert!(is_terminal_like_context("unknown"));
        assert!(!is_terminal_like_context("Safari"));
        assert!(!is_terminal_like_context("Cursor"));
    }

    #[test]
    fn browser_context_detection_matches_browser_like_inputs() {
        assert!(is_browser_like_context("Safari"));
        assert!(is_browser_like_context("Google Chrome"));
        assert!(!is_browser_like_context("Cursor"));
    }

    #[test]
    fn editor_context_detection_matches_editor_like_inputs() {
        assert!(is_editor_like_context("Cursor"));
        assert!(is_editor_like_context("Visual Studio Code"));
        assert!(!is_editor_like_context("Safari"));
    }

    #[test]
    fn terminal_worker_declares_browser_tools_as_restricted() {
        let worker = ToolLoopWorker::for_terminal_host(
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
            crate::config::ClickAtPreflightMode::default(),
        );
        let contract = worker.capability_contract();
        assert_eq!(contract.app_family, "terminal");
        assert!(contract.restricted_tools.contains(&"browser_open"));
    }

    #[test]
    fn browser_worker_prefers_browser_tool_for_mouse_navigation() {
        let worker = ToolLoopWorker::for_browser_host(
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
            crate::config::ClickAtPreflightMode::default(),
        );

        let contract = worker.capability_contract();
        assert_eq!(contract.app_family, "browser");
        assert_eq!(contract.preferred_tools.first().copied(), Some("browser"));
        assert!(contract.preferred_tools.contains(&"perception_capture"));
        assert!(!contract.preferred_tools.contains(&"mac_automation"));
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
            crate::config::ClickAtPreflightMode::default(),
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
        assert!(shared_history
            .iter()
            .any(|message| message.role == "user"
                && message.content.contains("return final response")));
    }
}
