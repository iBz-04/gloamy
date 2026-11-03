//! Provider backed by the locally installed Codex App Server.
//!
//! Codex owns OAuth credentials and inference. Gloamy sends messages over a
//! short-lived local JSON-RPC process and never reads Codex's credential files.

use crate::codex_app_server::CodexAppServerClient;
use crate::multimodal;
use crate::providers::traits::{ChatMessage, Provider, ProviderCapabilities};
use async_trait::async_trait;
use serde_json::{json, Value};

const DEFAULT_INSTRUCTIONS: &str = "You are Gloamy, a concise and helpful assistant.";
const APP_SERVER_BOUNDARY_INSTRUCTIONS: &str = "Do not access files, run commands, or use network tools directly. If Gloamy tool instructions are present, request those tools through their documented Gloamy tool-call format.";

pub struct CodexAppServerProvider;

impl CodexAppServerProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Provider for CodexAppServerProvider {
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            native_tool_calling: false,
            vision: true,
        }
    }

    async fn chat_with_system(
        &self,
        system_prompt: Option<&str>,
        message: &str,
        model: &str,
        _temperature: f64,
    ) -> anyhow::Result<String> {
        let mut messages = Vec::with_capacity(2);
        if let Some(system_prompt) = system_prompt {
            messages.push(ChatMessage::system(system_prompt));
        }
        messages.push(ChatMessage::user(message));
        self.chat_with_history(&messages, model, _temperature).await
    }

    async fn chat_with_history(
        &self,
        messages: &[ChatMessage],
        model: &str,
        _temperature: f64,
    ) -> anyhow::Result<String> {
        let multimodal_config = crate::config::MultimodalConfig::default();
        let prepared =
            multimodal::prepare_messages_for_provider(messages, &multimodal_config).await?;
        let (instructions, input) = build_app_server_request(&prepared.messages);
        let mut client = CodexAppServerClient::connect().await?;
        client.infer(model, &instructions, input).await
    }
}

fn build_app_server_request(messages: &[ChatMessage]) -> (String, Vec<Value>) {
    let mut system_parts = Vec::new();
    let mut input = Vec::new();

    for message in messages {
        if message.role == "system" {
            if !message.content.trim().is_empty() {
                system_parts.push(message.content.trim());
            }
            continue;
        }

        let (text, images) = if message.role == "user" {
            multimodal::parse_image_markers(&message.content)
        } else {
            (message.content.clone(), Vec::new())
        };

        let role = match message.role.as_str() {
            "user" => "User",
            "assistant" => "Assistant",
            "tool" => "Gloamy tool result",
            other => other,
        };
        let mut text_item = format!("{role}:\n{text}");
        if !input.is_empty() {
            text_item.insert_str(0, "\n\n");
        }
        input.push(json!({ "type": "text", "text": text_item }));
        input.extend(
            images
                .into_iter()
                .map(|url| json!({ "type": "image", "url": url })),
        );
    }

    if input.is_empty() {
        input.push(json!({ "type": "text", "text": "User:\n" }));
    }

    let mut instructions = if system_parts.is_empty() {
        DEFAULT_INSTRUCTIONS.to_string()
    } else {
        system_parts.join("\n\n")
    };
    instructions.push_str("\n\n");
    instructions.push_str(APP_SERVER_BOUNDARY_INSTRUCTIONS);

    (instructions, input)
}
