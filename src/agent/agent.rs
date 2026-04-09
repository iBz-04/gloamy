use crate::agent::dispatcher::{
    NativeToolDispatcher, ParsedToolCall, ToolDispatcher, ToolExecutionResult, XmlToolDispatcher,
};
use crate::agent::memory_loader::{DefaultMemoryLoader, MemoryLoader};
use crate::agent::prompt::{PromptContext, SystemPromptBuilder};
use crate::agent::task_store::{self, TaskCheckpointUpdate, TaskSnapshot, TaskStatus, TaskStore};
use crate::config::Config;
use crate::memory::{self, Memory, MemoryCategory};
use crate::observability::{self, Observer, ObserverEvent};
use crate::providers::{self, ChatMessage, ChatRequest, ConversationMessage, Provider};
use crate::runtime;
use crate::security::{AutonomyLevel, SecurityPolicy};
use crate::tools::{self, Tool, ToolSpec};
use anyhow::Result;
use std::collections::HashMap;
use std::io::Write as IoWrite;
use std::sync::Arc;
use std::time::Instant;

pub struct Agent {
    provider: Box<dyn Provider>,
    tools: Vec<Box<dyn Tool>>,
    tool_specs: Vec<ToolSpec>,
    memory: Arc<dyn Memory>,
    observer: Arc<dyn Observer>,
    prompt_builder: SystemPromptBuilder,
    tool_dispatcher: Box<dyn ToolDispatcher>,
    memory_loader: Box<dyn MemoryLoader>,
    config: crate::config::AgentConfig,
    model_name: String,
    temperature: f64,
    workspace_dir: std::path::PathBuf,
    identity_config: crate::config::IdentityConfig,
    skills: Vec<crate::skills::Skill>,
    skills_prompt_mode: crate::config::SkillsPromptInjectionMode,
    auto_save: bool,
    history: Vec<ConversationMessage>,
    classification_config: crate::config::QueryClassificationConfig,
    available_hints: Vec<String>,
    route_model_by_hint: HashMap<String, String>,
    autonomy_level: AutonomyLevel,
    provider_name: String,
    task_store: Option<Box<dyn TaskStore>>,
    task_session_id: Option<String>,
    resume_persisted_session: bool,
    persisted_state_hydrated: bool,
    latest_checkpoint_note: Option<String>,
}

pub struct AgentBuilder {
    provider: Option<Box<dyn Provider>>,
    tools: Option<Vec<Box<dyn Tool>>>,
    memory: Option<Arc<dyn Memory>>,
    observer: Option<Arc<dyn Observer>>,
    prompt_builder: Option<SystemPromptBuilder>,
    tool_dispatcher: Option<Box<dyn ToolDispatcher>>,
    memory_loader: Option<Box<dyn MemoryLoader>>,
    config: Option<crate::config::AgentConfig>,
    model_name: Option<String>,
    temperature: Option<f64>,
    workspace_dir: Option<std::path::PathBuf>,
    identity_config: Option<crate::config::IdentityConfig>,
    skills: Option<Vec<crate::skills::Skill>>,
    skills_prompt_mode: Option<crate::config::SkillsPromptInjectionMode>,
    auto_save: Option<bool>,
    classification_config: Option<crate::config::QueryClassificationConfig>,
    available_hints: Option<Vec<String>>,
    route_model_by_hint: Option<HashMap<String, String>>,
    autonomy_level: Option<AutonomyLevel>,
    provider_name: Option<String>,
    task_store: Option<Box<dyn TaskStore>>,
    task_session_id: Option<String>,
    resume_persisted_session: Option<bool>,
}

impl AgentBuilder {
    pub fn new() -> Self {
        Self {
            provider: None,
            tools: None,
            memory: None,
            observer: None,
            prompt_builder: None,
            tool_dispatcher: None,
            memory_loader: None,
            config: None,
            model_name: None,
            temperature: None,
            workspace_dir: None,
            identity_config: None,
            skills: None,
            skills_prompt_mode: None,
            auto_save: None,
            classification_config: None,
            available_hints: None,
            route_model_by_hint: None,
            autonomy_level: None,
            provider_name: None,
            task_store: None,
            task_session_id: None,
            resume_persisted_session: None,
        }
    }

    pub fn provider(mut self, provider: Box<dyn Provider>) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn tools(mut self, tools: Vec<Box<dyn Tool>>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn memory(mut self, memory: Arc<dyn Memory>) -> Self {
        self.memory = Some(memory);
        self
    }

    pub fn observer(mut self, observer: Arc<dyn Observer>) -> Self {
        self.observer = Some(observer);
        self
    }

    pub fn prompt_builder(mut self, prompt_builder: SystemPromptBuilder) -> Self {
        self.prompt_builder = Some(prompt_builder);
        self
    }

    pub fn tool_dispatcher(mut self, tool_dispatcher: Box<dyn ToolDispatcher>) -> Self {
        self.tool_dispatcher = Some(tool_dispatcher);
        self
    }

    pub fn memory_loader(mut self, memory_loader: Box<dyn MemoryLoader>) -> Self {
        self.memory_loader = Some(memory_loader);
        self
    }

    pub fn config(mut self, config: crate::config::AgentConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn model_name(mut self, model_name: String) -> Self {
        self.model_name = Some(model_name);
        self
    }

    pub fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn workspace_dir(mut self, workspace_dir: std::path::PathBuf) -> Self {
        self.workspace_dir = Some(workspace_dir);
        self
    }

    pub fn identity_config(mut self, identity_config: crate::config::IdentityConfig) -> Self {
        self.identity_config = Some(identity_config);
        self
    }

    pub fn skills(mut self, skills: Vec<crate::skills::Skill>) -> Self {
        self.skills = Some(skills);
        self
    }

    pub fn skills_prompt_mode(
        mut self,
        skills_prompt_mode: crate::config::SkillsPromptInjectionMode,
    ) -> Self {
        self.skills_prompt_mode = Some(skills_prompt_mode);
        self
    }

    pub fn auto_save(mut self, auto_save: bool) -> Self {
        self.auto_save = Some(auto_save);
        self
    }

    pub fn classification_config(
        mut self,
        classification_config: crate::config::QueryClassificationConfig,
    ) -> Self {
        self.classification_config = Some(classification_config);
        self
    }

    pub fn available_hints(mut self, available_hints: Vec<String>) -> Self {
        self.available_hints = Some(available_hints);
        self
    }

    pub fn route_model_by_hint(mut self, route_model_by_hint: HashMap<String, String>) -> Self {
        self.route_model_by_hint = Some(route_model_by_hint);
        self
    }

    pub fn autonomy_level(mut self, autonomy_level: AutonomyLevel) -> Self {
        self.autonomy_level = Some(autonomy_level);
        self
    }

    pub fn provider_name(mut self, provider_name: String) -> Self {
        self.provider_name = Some(provider_name);
        self
    }

    pub(crate) fn task_store(mut self, task_store: Box<dyn TaskStore>) -> Self {
        self.task_store = Some(task_store);
        self
    }

    pub fn task_session_id(mut self, task_session_id: String) -> Self {
        self.task_session_id = Some(task_session_id);
        self
    }

    pub fn resume_persisted_session(mut self, resume_persisted_session: bool) -> Self {
        self.resume_persisted_session = Some(resume_persisted_session);
        self
    }

    pub fn build(self) -> Result<Agent> {
        let tools = self
            .tools
            .ok_or_else(|| anyhow::anyhow!("tools are required"))?;
        let tool_specs = tools.iter().map(|tool| tool.spec()).collect();

        Ok(Agent {
            provider: self
                .provider
                .ok_or_else(|| anyhow::anyhow!("provider is required"))?,
            tools,
            tool_specs,
            memory: self
                .memory
                .ok_or_else(|| anyhow::anyhow!("memory is required"))?,
            observer: self
                .observer
                .ok_or_else(|| anyhow::anyhow!("observer is required"))?,
            prompt_builder: self
                .prompt_builder
                .unwrap_or_else(SystemPromptBuilder::with_defaults),
            tool_dispatcher: self
                .tool_dispatcher
                .ok_or_else(|| anyhow::anyhow!("tool_dispatcher is required"))?,
            memory_loader: self
                .memory_loader
                .unwrap_or_else(|| Box::new(DefaultMemoryLoader::default())),
            config: self.config.unwrap_or_default(),
            model_name: self
                .model_name
                .unwrap_or_else(|| "anthropic/claude-sonnet-4-20250514".into()),
            temperature: self.temperature.unwrap_or(0.7),
            workspace_dir: self
                .workspace_dir
                .unwrap_or_else(|| std::path::PathBuf::from(".")),
            identity_config: self.identity_config.unwrap_or_default(),
            skills: self.skills.unwrap_or_default(),
            skills_prompt_mode: self.skills_prompt_mode.unwrap_or_default(),
            auto_save: self.auto_save.unwrap_or(false),
            history: Vec::new(),
            classification_config: self.classification_config.unwrap_or_default(),
            available_hints: self.available_hints.unwrap_or_default(),
            route_model_by_hint: self.route_model_by_hint.unwrap_or_default(),
            autonomy_level: self.autonomy_level.unwrap_or_default(),
            provider_name: self.provider_name.unwrap_or_else(|| "unknown".into()),
            task_store: self.task_store,
            task_session_id: self.task_session_id,
            resume_persisted_session: self.resume_persisted_session.unwrap_or(false),
            persisted_state_hydrated: false,
            latest_checkpoint_note: None,
        })
    }
}

impl Agent {
    pub fn builder() -> AgentBuilder {
        AgentBuilder::new()
    }

    pub fn history(&self) -> &[ConversationMessage] {
        &self.history
    }

    pub async fn clear_history(&mut self) {
        self.history.clear();
        self.latest_checkpoint_note = None;
        self.persisted_state_hydrated = true;

        let (Some(store), Some(session_id)) =
            (self.task_store.as_deref(), self.task_session_id.as_deref())
        else {
            return;
        };

        if let Err(err) = task_store::clear_session(store, session_id).await {
            tracing::warn!(thread_id = %session_id, "Failed to clear durable task state: {err}");
        }
    }

    fn has_task_persistence(&self) -> bool {
        self.task_store.is_some() && self.task_session_id.is_some()
    }

    fn cli_session_key(&self) -> String {
        format!("cli_interactive:{}", self.workspace_dir.display())
    }

    fn ensure_cli_persistence_session(&mut self) {
        if self.task_store.is_none() {
            return;
        }
        if self.task_session_id.is_none() {
            self.task_session_id = Some(self.cli_session_key());
        }
        self.resume_persisted_session = true;
        self.persisted_state_hydrated = false;
    }

    fn execution_history_for_persistence(&self) -> Vec<ChatMessage> {
        self.tool_dispatcher.to_provider_messages(&self.history)
    }

    fn resumable_history_for_persistence(&self) -> Vec<ChatMessage> {
        task_store::conversation_messages_to_resumable_turns(&self.history)
    }

    async fn hydrate_persisted_history_if_needed(&mut self) {
        if !self.resume_persisted_session
            || self.persisted_state_hydrated
            || !self.history.is_empty()
            || !self.has_task_persistence()
        {
            return;
        }

        self.persisted_state_hydrated = true;
        let (Some(store), Some(session_id)) =
            (self.task_store.as_deref(), self.task_session_id.as_deref())
        else {
            return;
        };

        match task_store::load_resumable_state(store, session_id).await {
            Ok(Some((turns, checkpoint_note))) => {
                self.history = task_store::resumable_turns_to_conversation_messages(turns);
                self.latest_checkpoint_note = checkpoint_note;
            }
            Ok(None) => {
                self.latest_checkpoint_note = None;
            }
            Err(err) => {
                tracing::warn!(thread_id = %session_id, "Failed to load durable task state: {err}");
            }
        }
    }

    async fn persist_task_snapshot(
        &self,
        model: &str,
        status: TaskStatus,
        latest_checkpoint_note: Option<String>,
        final_response: Option<String>,
        last_error: Option<String>,
    ) {
        let (Some(store), Some(session_id)) =
            (self.task_store.as_deref(), self.task_session_id.as_deref())
        else {
            return;
        };

        let snapshot = TaskSnapshot {
            task_id: session_id.to_string(),
            thread_id: session_id.to_string(),
            channel: "cli".to_string(),
            provider: self.provider_name.clone(),
            model: model.to_string(),
            status,
            execution_history: self.execution_history_for_persistence(),
            resumable_history: self.resumable_history_for_persistence(),
            latest_checkpoint_note,
            final_response,
            last_error,
        };

        if let Err(err) = task_store::persist_snapshot(store, snapshot).await {
            tracing::warn!(thread_id = %session_id, "Failed to persist task snapshot: {err}");
        }
    }

    async fn persist_task_checkpoint(
        &self,
        model: &str,
        step_index: usize,
        checkpoint_note: Option<String>,
        items: Vec<crate::agent::loop_::ExecutionCheckpointItem>,
    ) {
        let (Some(store), Some(session_id)) =
            (self.task_store.as_deref(), self.task_session_id.as_deref())
        else {
            return;
        };

        if let Err(err) = store
            .record_checkpoint(TaskCheckpointUpdate {
                task_id: session_id.to_string(),
                thread_id: session_id.to_string(),
                channel: "cli".to_string(),
                provider: self.provider_name.clone(),
                model: model.to_string(),
                step_index,
                execution_history: self.execution_history_for_persistence(),
                checkpoint_note,
                items,
            })
            .await
        {
            tracing::warn!(thread_id = %session_id, "Failed to record task checkpoint: {err}");
        }
    }

    pub fn from_config(config: &Config) -> Result<Self> {
        let observer: Arc<dyn Observer> =
            Arc::from(observability::create_observer(&config.observability));
        let runtime: Arc<dyn runtime::RuntimeAdapter> =
            Arc::from(runtime::create_runtime(&config.runtime)?);
        let security = Arc::new(SecurityPolicy::from_config(
            &config.autonomy,
            &config.workspace_dir,
        ));

        let memory: Arc<dyn Memory> = Arc::from(memory::create_memory_with_storage_and_routes(
            &config.memory,
            &config.embedding_routes,
            Some(&config.storage.provider.config),
            &config.workspace_dir,
            config.api_key.as_deref(),
        )?);

        let composio_key = if config.composio.enabled {
            config.composio.api_key.as_deref()
        } else {
            None
        };
        let composio_entity_id = if config.composio.enabled {
            Some(config.composio.entity_id.as_str())
        } else {
            None
        };
        let one_key = if config.one.enabled {
            config.one.api_key.as_deref()
        } else {
            None
        };

        let tools = tools::all_tools_with_runtime(
            Arc::new(config.clone()),
            &security,
            runtime,
            memory.clone(),
            composio_key,
            composio_entity_id,
            one_key,
            &config.browser,
            &config.http_request,
            &config.web_fetch,
            &config.workspace_dir,
            &config.agents,
            config.api_key.as_deref(),
            config,
        );

        let provider_name = config.default_provider.as_deref().unwrap_or("openai");

        let model_name = config
            .default_model
            .as_deref()
            .unwrap_or("anthropic/claude-sonnet-4-20250514")
            .to_string();

        let provider: Box<dyn Provider> = providers::create_routed_provider(
            provider_name,
            config.api_key.as_deref(),
            config.api_url.as_deref(),
            &config.reliability,
            &config.model_routes,
            &model_name,
        )?;

        let task_store = match task_store::create_task_store(&config.workspace_dir) {
            Ok(store) => Some(store),
            Err(err) => {
                tracing::warn!(
                    workspace = %config.workspace_dir.display(),
                    "Failed to initialize task persistence store: {err}"
                );
                None
            }
        };

        let dispatcher_choice = config.agent.tool_dispatcher.as_str();
        let tool_dispatcher: Box<dyn ToolDispatcher> = match dispatcher_choice {
            "native" => Box::new(NativeToolDispatcher),
            "xml" => Box::new(XmlToolDispatcher),
            _ if provider.supports_native_tools() => Box::new(NativeToolDispatcher),
            _ => Box::new(XmlToolDispatcher),
        };

        let route_model_by_hint: HashMap<String, String> = config
            .model_routes
            .iter()
            .map(|route| (route.hint.clone(), route.model.clone()))
            .collect();
        let available_hints: Vec<String> = route_model_by_hint.keys().cloned().collect();

        let builder = Agent::builder()
            .provider(provider)
            .tools(tools)
            .memory(memory)
            .observer(observer)
            .tool_dispatcher(tool_dispatcher)
            .memory_loader(Box::new(DefaultMemoryLoader::new(
                5,
                config.memory.min_relevance_score,
            )))
            .prompt_builder(SystemPromptBuilder::with_defaults())
            .config(config.agent.clone())
            .model_name(model_name)
            .temperature(config.default_temperature)
            .workspace_dir(config.workspace_dir.clone())
            .classification_config(config.query_classification.clone())
            .available_hints(available_hints)
            .route_model_by_hint(route_model_by_hint)
            .identity_config(config.identity.clone())
            .skills(crate::skills::load_skills_with_runtime_context(
                &config.workspace_dir,
                config,
            ))
            .skills_prompt_mode(config.skills.prompt_injection_mode)
            .auto_save(config.memory.auto_save)
            .autonomy_level(config.autonomy.level)
            .provider_name(provider_name.to_string());

        let builder = if let Some(store) = task_store {
            builder.task_store(store)
        } else {
            builder
        };

        builder.build()
    }

    fn trim_history(&mut self) {
        let max = self.config.max_history_messages;
        if self.history.len() <= max {
            return;
        }

        let mut system_messages = Vec::new();
        let mut other_messages = Vec::new();

        for msg in self.history.drain(..) {
            match &msg {
                ConversationMessage::Chat(chat) if chat.role == "system" => {
                    system_messages.push(msg);
                }
                _ => other_messages.push(msg),
            }
        }

        if other_messages.len() > max {
            let drop_count = other_messages.len() - max;
            other_messages.drain(0..drop_count);
        }

        self.history = system_messages;
        self.history.extend(other_messages);
    }

    fn build_system_prompt(&self) -> Result<String> {
        let instructions = self.tool_dispatcher.prompt_instructions(&self.tools);
        let ctx = PromptContext {
            workspace_dir: &self.workspace_dir,
            model_name: &self.model_name,
            tools: &self.tools,
            skills: &self.skills,
            skills_prompt_mode: self.skills_prompt_mode,
            identity_config: Some(&self.identity_config),
            dispatcher_instructions: &instructions,
            autonomy_level: self.autonomy_level,
        };
        self.prompt_builder.build(&ctx)
    }

    async fn execute_tool_call(&self, call: &ParsedToolCall) -> ToolExecutionResult {
        let start = Instant::now();

        let (result, success) =
            if let Some(tool) = self.tools.iter().find(|t| t.name() == call.name) {
                match tool.execute(call.arguments.clone()).await {
                    Ok(r) => {
                        self.observer.record_event(&ObserverEvent::ToolCall {
                            tool: call.name.clone(),
                            duration: start.elapsed(),
                            success: r.success,
                        });
                        if r.success {
                            (r.output, true)
                        } else {
                            (format!("Error: {}", r.error.unwrap_or(r.output)), false)
                        }
                    }
                    Err(e) => {
                        self.observer.record_event(&ObserverEvent::ToolCall {
                            tool: call.name.clone(),
                            duration: start.elapsed(),
                            success: false,
                        });
                        (format!("Error executing {}: {e}", call.name), false)
                    }
                }
            } else {
                (format!("Unknown tool: {}", call.name), false)
            };

        ToolExecutionResult {
            name: call.name.clone(),
            output: result,
            success,
            tool_call_id: call.tool_call_id.clone(),
        }
    }

    async fn execute_tools(&self, calls: &[ParsedToolCall]) -> Vec<ToolExecutionResult> {
        if !self.config.parallel_tools {
            let mut results = Vec::with_capacity(calls.len());
            for call in calls {
                results.push(self.execute_tool_call(call).await);
            }
            return results;
        }

        let futs: Vec<_> = calls
            .iter()
            .map(|call| self.execute_tool_call(call))
            .collect();
        futures_util::future::join_all(futs).await
    }

    fn classify_model(&self, user_message: &str) -> String {
        if let Some(decision) =
            super::classifier::classify_with_decision(&self.classification_config, user_message)
        {
            if self.available_hints.contains(&decision.hint) {
                let resolved_model = self
                    .route_model_by_hint
                    .get(&decision.hint)
                    .map(String::as_str)
                    .unwrap_or("unknown");
                tracing::info!(
                    target: "query_classification",
                    hint = decision.hint.as_str(),
                    model = resolved_model,
                    rule_priority = decision.priority,
                    message_length = user_message.len(),
                    "Classified message route"
                );
                return format!("hint:{}", decision.hint);
            }
        }
        self.model_name.clone()
    }

    pub async fn turn(&mut self, user_message: &str) -> Result<String> {
        self.hydrate_persisted_history_if_needed().await;

        let has_system_prompt = self.history.iter().any(|msg| {
            matches!(
                msg,
                ConversationMessage::Chat(chat) if chat.role == "system"
            )
        });
        if !has_system_prompt {
            let system_prompt = self.build_system_prompt()?;
            self.history.insert(
                0,
                ConversationMessage::Chat(ChatMessage::system(system_prompt)),
            );
        }

        if self.auto_save {
            let _ = self
                .memory
                .store("user_msg", user_message, MemoryCategory::Conversation, None)
                .await;
        }

        let context = self
            .memory_loader
            .load_context(self.memory.as_ref(), user_message)
            .await
            .unwrap_or_default();

        let lesson_context = if self.config.self_learning {
            crate::agent::lesson::build_lesson_context(
                self.memory.as_ref(),
                user_message,
                self.config.max_lessons_per_query,
            )
            .await
        } else {
            String::new()
        };

        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S %Z");
        let mut enriched = if context.is_empty() && lesson_context.is_empty() {
            format!("[{now}] {user_message}")
        } else {
            format!("{context}{lesson_context}[{now}] {user_message}")
        };
        if let Some(note) = self.latest_checkpoint_note.as_deref() {
            if !note.trim().is_empty() {
                enriched = format!("[Task checkpoint]\n{note}\n\n[User message]\n{enriched}");
            }
        }

        self.history
            .push(ConversationMessage::Chat(ChatMessage::user(enriched)));

        let effective_model = self.classify_model(user_message);
        self.persist_task_snapshot(
            &effective_model,
            TaskStatus::Running,
            self.latest_checkpoint_note.clone(),
            None,
            None,
        )
        .await;

        let mut tool_outcomes: Vec<crate::agent::lesson::ToolOutcome> = Vec::new();
        let mut execution_checkpoint_note = self.latest_checkpoint_note.clone();

        for iteration in 0..self.config.max_tool_iterations {
            let mut messages = self.tool_dispatcher.to_provider_messages(&self.history);
            if let Some(note) = execution_checkpoint_note.as_deref() {
                crate::agent::loop_::inject_ephemeral_system_note(&mut messages, note);
            }
            let response = match self
                .provider
                .chat(
                    ChatRequest {
                        messages: &messages,
                        tools: if self.tool_dispatcher.should_send_tool_specs() {
                            Some(&self.tool_specs)
                        } else {
                            None
                        },
                    },
                    &effective_model,
                    self.temperature,
                )
                .await
            {
                Ok(resp) => resp,
                Err(err) => {
                    let error_text = err.to_string();
                    self.latest_checkpoint_note = execution_checkpoint_note.clone();
                    self.persist_task_snapshot(
                        &effective_model,
                        TaskStatus::Failed,
                        execution_checkpoint_note.clone(),
                        None,
                        Some(error_text),
                    )
                    .await;
                    return Err(err);
                }
            };

            let (text, calls) = self.tool_dispatcher.parse_response(&response);
            if calls.is_empty() {
                let final_text = if text.is_empty() {
                    response.text.unwrap_or_default()
                } else {
                    text
                };

                self.history
                    .push(ConversationMessage::Chat(ChatMessage::assistant(
                        final_text.clone(),
                    )));
                self.trim_history();

                self.latest_checkpoint_note = execution_checkpoint_note.clone();
                self.persist_task_snapshot(
                    &effective_model,
                    TaskStatus::Completed,
                    execution_checkpoint_note,
                    Some(final_text.clone()),
                    None,
                )
                .await;

                return Ok(final_text);
            }

            if !text.is_empty() {
                self.history
                    .push(ConversationMessage::Chat(ChatMessage::assistant(
                        text.clone(),
                    )));
                print!("{text}");
                let _ = std::io::stdout().flush();
            }

            self.history.push(ConversationMessage::AssistantToolCalls {
                text: response.text.clone(),
                tool_calls: response.tool_calls.clone(),
                reasoning_content: response.reasoning_content.clone(),
            });

            let results = self.execute_tools(&calls).await;

            // Track tool outcomes for self-learning (persist after each batch)
            if self.config.self_learning {
                for (call, result) in calls.iter().zip(&results) {
                    tool_outcomes.push(crate::agent::lesson::ToolOutcome {
                        tool_name: call.name.clone(),
                        arguments: call.arguments.clone(),
                        success: result.success,
                        output: result.output.clone(),
                    });
                }
                let stored = crate::agent::lesson::persist_lessons_from_outcomes(
                    self.memory.as_ref(),
                    &tool_outcomes,
                    user_message,
                )
                .await;
                if stored > 0 {
                    tracing::info!(count = stored, "Self-learning: persisted new lessons");
                }
            }

            let checkpoint_items: Vec<crate::agent::loop_::ExecutionCheckpointItem> = calls
                .iter()
                .zip(&results)
                .map(
                    |(call, result)| crate::agent::loop_::ExecutionCheckpointItem {
                        tool_name: call.name.clone(),
                        arguments: call.arguments.clone(),
                        success: result.success,
                        output: result.output.clone(),
                    },
                )
                .collect();
            execution_checkpoint_note =
                crate::agent::loop_::build_execution_checkpoint_note(&checkpoint_items);

            let formatted = self.tool_dispatcher.format_results(&results);
            self.history.push(formatted);

            // When using native tool calling, tool results go as `tool` role
            // messages which providers skip for multimodal processing. If any
            // tool result contains [IMAGE:] markers (e.g. screenshots), inject
            // them as a user message so the LLM can actually see the images.
            if self.tool_dispatcher.should_send_tool_specs() {
                let image_refs: Vec<String> = results
                    .iter()
                    .filter(|r| r.success)
                    .flat_map(|r| crate::multimodal::parse_image_markers(&r.output).1)
                    .collect();
                if !image_refs.is_empty() {
                    let markers: String = image_refs
                        .iter()
                        .map(|r| format!("[IMAGE:{r}]"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    self.history
                        .push(ConversationMessage::Chat(ChatMessage::user(format!(
                            "[Tool screenshot attached]\n{markers}"
                        ))));
                }
            }

            self.trim_history();

            self.persist_task_checkpoint(
                &effective_model,
                iteration + 1,
                execution_checkpoint_note.clone(),
                checkpoint_items,
            )
            .await;
        }

        self.latest_checkpoint_note = execution_checkpoint_note.clone();
        let loop_error = format!(
            "Agent exceeded maximum tool iterations ({})",
            self.config.max_tool_iterations
        );
        self.persist_task_snapshot(
            &effective_model,
            TaskStatus::Failed,
            execution_checkpoint_note,
            None,
            Some(loop_error),
        )
        .await;

        anyhow::bail!(
            "Agent exceeded maximum tool iterations ({})",
            self.config.max_tool_iterations
        )
    }

    pub async fn run_single(&mut self, message: &str) -> Result<String> {
        self.turn(message).await
    }

    pub async fn run_interactive(&mut self) -> Result<()> {
        self.ensure_cli_persistence_session();
        println!("🦀 Gloamy Interactive Mode");
        println!("Type /help for commands.\n");

        let (tx, mut rx) = tokio::sync::mpsc::channel(32);
        let cli = crate::channels::CliChannel::new();

        let listen_handle = tokio::spawn(async move {
            let _ = crate::channels::Channel::listen(&cli, tx).await;
        });

        while let Some(msg) = rx.recv().await {
            match msg.content.trim() {
                "/help" => {
                    println!("Available commands:");
                    println!("  /help        Show this help message");
                    println!("  /clear /new  Clear conversation history");
                    println!("  /quit /exit  Exit interactive mode\n");
                    continue;
                }
                "/clear" | "/new" => {
                    self.clear_history().await;
                    println!("Conversation cleared.\n");
                    continue;
                }
                _ => {}
            }

            let response = match self.turn(&msg.content).await {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!("\nError: {e}\n");
                    continue;
                }
            };
            println!("\n{response}\n");
        }

        listen_handle.abort();
        Ok(())
    }
}

pub async fn run(
    config: Config,
    message: Option<String>,
    provider_override: Option<String>,
    model_override: Option<String>,
    temperature: f64,
) -> Result<()> {
    let start = Instant::now();

    let mut effective_config = config;
    if let Some(p) = provider_override {
        effective_config.default_provider = Some(p);
    }
    if let Some(m) = model_override {
        effective_config.default_model = Some(m);
    }
    effective_config.default_temperature = temperature;

    let mut agent = Agent::from_config(&effective_config)?;

    let provider_name = effective_config
        .default_provider
        .as_deref()
        .unwrap_or("openai")
        .to_string();
    let model_name = effective_config
        .default_model
        .as_deref()
        .unwrap_or("anthropic/claude-sonnet-4-20250514")
        .to_string();

    agent.observer.record_event(&ObserverEvent::AgentStart {
        provider: provider_name.clone(),
        model: model_name.clone(),
    });

    if let Some(msg) = message {
        let response = agent.run_single(&msg).await?;
        println!("{response}");
    } else {
        agent.run_interactive().await?;
    }

    agent.observer.record_event(&ObserverEvent::AgentEnd {
        provider: provider_name,
        model: model_name,
        duration: start.elapsed(),
        tokens_used: None,
        cost_usd: None,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use parking_lot::Mutex;
    use std::collections::HashMap;

    struct MockProvider {
        responses: Mutex<Vec<crate::providers::ChatResponse>>,
    }

    #[async_trait]
    impl Provider for MockProvider {
        async fn chat_with_system(
            &self,
            _system_prompt: Option<&str>,
            _message: &str,
            _model: &str,
            _temperature: f64,
        ) -> Result<String> {
            Ok("ok".into())
        }

        async fn chat(
            &self,
            _request: ChatRequest<'_>,
            _model: &str,
            _temperature: f64,
        ) -> Result<crate::providers::ChatResponse> {
            let mut guard = self.responses.lock();
            if guard.is_empty() {
                return Ok(crate::providers::ChatResponse {
                    text: Some("done".into()),
                    tool_calls: vec![],
                    usage: None,
                    reasoning_content: None,
                });
            }
            Ok(guard.remove(0))
        }
    }

    struct ModelCaptureProvider {
        responses: Mutex<Vec<crate::providers::ChatResponse>>,
        seen_models: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl Provider for ModelCaptureProvider {
        async fn chat_with_system(
            &self,
            _system_prompt: Option<&str>,
            _message: &str,
            _model: &str,
            _temperature: f64,
        ) -> Result<String> {
            Ok("ok".into())
        }

        async fn chat(
            &self,
            _request: ChatRequest<'_>,
            model: &str,
            _temperature: f64,
        ) -> Result<crate::providers::ChatResponse> {
            self.seen_models.lock().push(model.to_string());
            let mut guard = self.responses.lock();
            if guard.is_empty() {
                return Ok(crate::providers::ChatResponse {
                    text: Some("done".into()),
                    tool_calls: vec![],
                    usage: None,
                    reasoning_content: None,
                });
            }
            Ok(guard.remove(0))
        }
    }

    struct MockTool;

    struct FailingMockTool;

    #[async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str {
            "echo"
        }

        fn description(&self) -> &str {
            "echo"
        }

        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({"type": "object"})
        }

        async fn execute(&self, _args: serde_json::Value) -> Result<crate::tools::ToolResult> {
            Ok(crate::tools::ToolResult {
                success: true,
                output: "tool-out".into(),
                error: None,
            })
        }
    }

    #[async_trait]
    impl Tool for FailingMockTool {
        fn name(&self) -> &str {
            "always_fail"
        }

        fn description(&self) -> &str {
            "always fails"
        }

        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({"type": "object"})
        }

        async fn execute(&self, _args: serde_json::Value) -> Result<crate::tools::ToolResult> {
            Ok(crate::tools::ToolResult {
                success: false,
                output: String::new(),
                error: Some("simulated failure".into()),
            })
        }
    }

    #[tokio::test]
    async fn turn_without_tools_returns_text() {
        let provider = Box::new(MockProvider {
            responses: Mutex::new(vec![crate::providers::ChatResponse {
                text: Some("hello".into()),
                tool_calls: vec![],
                usage: None,
                reasoning_content: None,
            }]),
        });

        let memory_cfg = crate::config::MemoryConfig {
            backend: "none".into(),
            ..crate::config::MemoryConfig::default()
        };
        let mem: Arc<dyn Memory> = Arc::from(
            crate::memory::create_memory(&memory_cfg, std::path::Path::new("/tmp"), None)
                .expect("memory creation should succeed with valid config"),
        );

        let observer: Arc<dyn Observer> = Arc::from(crate::observability::NoopObserver {});
        let mut agent = Agent::builder()
            .provider(provider)
            .tools(vec![Box::new(MockTool)])
            .memory(mem)
            .observer(observer)
            .tool_dispatcher(Box::new(XmlToolDispatcher))
            .workspace_dir(std::path::PathBuf::from("/tmp"))
            .build()
            .expect("agent builder should succeed with valid config");

        let response = agent.turn("hi").await.unwrap();
        assert_eq!(response, "hello");
    }

    #[tokio::test]
    async fn turn_with_native_dispatcher_handles_tool_results_variant() {
        let provider = Box::new(MockProvider {
            responses: Mutex::new(vec![
                crate::providers::ChatResponse {
                    text: Some(String::new()),
                    tool_calls: vec![crate::providers::ToolCall {
                        id: "tc1".into(),
                        name: "echo".into(),
                        arguments: "{}".into(),
                    }],
                    usage: None,
                    reasoning_content: None,
                },
                crate::providers::ChatResponse {
                    text: Some("done".into()),
                    tool_calls: vec![],
                    usage: None,
                    reasoning_content: None,
                },
            ]),
        });

        let memory_cfg = crate::config::MemoryConfig {
            backend: "none".into(),
            ..crate::config::MemoryConfig::default()
        };
        let mem: Arc<dyn Memory> = Arc::from(
            crate::memory::create_memory(&memory_cfg, std::path::Path::new("/tmp"), None)
                .expect("memory creation should succeed with valid config"),
        );

        let observer: Arc<dyn Observer> = Arc::from(crate::observability::NoopObserver {});
        let mut agent = Agent::builder()
            .provider(provider)
            .tools(vec![Box::new(MockTool)])
            .memory(mem)
            .observer(observer)
            .tool_dispatcher(Box::new(NativeToolDispatcher))
            .workspace_dir(std::path::PathBuf::from("/tmp"))
            .build()
            .expect("agent builder should succeed with valid config");

        let response = agent.turn("hi").await.unwrap();
        assert_eq!(response, "done");
        assert!(agent
            .history()
            .iter()
            .any(|msg| matches!(msg, ConversationMessage::ToolResults(_))));
    }

    #[tokio::test]
    async fn turn_with_xml_dispatcher_marks_failed_tool_results_as_error() {
        let provider = Box::new(MockProvider {
            responses: Mutex::new(vec![
                crate::providers::ChatResponse {
                    text: Some(
                        "<tool_call>{\"name\":\"always_fail\",\"arguments\":{}}</tool_call>".into(),
                    ),
                    tool_calls: vec![],
                    usage: None,
                    reasoning_content: None,
                },
                crate::providers::ChatResponse {
                    text: Some("done".into()),
                    tool_calls: vec![],
                    usage: None,
                    reasoning_content: None,
                },
            ]),
        });

        let memory_cfg = crate::config::MemoryConfig {
            backend: "none".into(),
            ..crate::config::MemoryConfig::default()
        };
        let mem: Arc<dyn Memory> = Arc::from(
            crate::memory::create_memory(&memory_cfg, std::path::Path::new("/tmp"), None)
                .expect("memory creation should succeed with valid config"),
        );

        let observer: Arc<dyn Observer> = Arc::from(crate::observability::NoopObserver {});
        let mut agent = Agent::builder()
            .provider(provider)
            .tools(vec![Box::new(FailingMockTool)])
            .memory(mem)
            .observer(observer)
            .tool_dispatcher(Box::new(XmlToolDispatcher))
            .workspace_dir(std::path::PathBuf::from("/tmp"))
            .build()
            .expect("agent builder should succeed with valid config");

        let response = agent.turn("fail once").await.unwrap();
        assert_eq!(response, "done");
        assert!(agent.history().iter().any(|msg| {
            matches!(
                msg,
                ConversationMessage::Chat(chat)
                    if chat.role == "user"
                        && chat.content.contains(
                            "<tool_result name=\"always_fail\" status=\"error\">"
                        )
            )
        }));
    }

    #[tokio::test]
    async fn turn_routes_with_hint_when_query_classification_matches() {
        let seen_models = Arc::new(Mutex::new(Vec::new()));
        let provider = Box::new(ModelCaptureProvider {
            responses: Mutex::new(vec![crate::providers::ChatResponse {
                text: Some("classified".into()),
                tool_calls: vec![],
                usage: None,
                reasoning_content: None,
            }]),
            seen_models: seen_models.clone(),
        });

        let memory_cfg = crate::config::MemoryConfig {
            backend: "none".into(),
            ..crate::config::MemoryConfig::default()
        };
        let mem: Arc<dyn Memory> = Arc::from(
            crate::memory::create_memory(&memory_cfg, std::path::Path::new("/tmp"), None)
                .expect("memory creation should succeed with valid config"),
        );

        let observer: Arc<dyn Observer> = Arc::from(crate::observability::NoopObserver {});
        let mut route_model_by_hint = HashMap::new();
        route_model_by_hint.insert("fast".to_string(), "anthropic/claude-haiku-4-5".to_string());
        let mut agent = Agent::builder()
            .provider(provider)
            .tools(vec![Box::new(MockTool)])
            .memory(mem)
            .observer(observer)
            .tool_dispatcher(Box::new(NativeToolDispatcher))
            .workspace_dir(std::path::PathBuf::from("/tmp"))
            .classification_config(crate::config::QueryClassificationConfig {
                enabled: true,
                rules: vec![crate::config::ClassificationRule {
                    hint: "fast".to_string(),
                    keywords: vec!["quick".to_string()],
                    patterns: vec![],
                    min_length: None,
                    max_length: None,
                    priority: 10,
                }],
            })
            .available_hints(vec!["fast".to_string()])
            .route_model_by_hint(route_model_by_hint)
            .build()
            .expect("agent builder should succeed with valid config");

        let response = agent.turn("quick summary please").await.unwrap();
        assert_eq!(response, "classified");
        let seen = seen_models.lock();
        assert_eq!(seen.as_slice(), &["hint:fast".to_string()]);
    }
}
