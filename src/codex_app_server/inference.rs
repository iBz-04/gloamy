use super::{CodexAppServerClient, APP_SERVER_REQUEST_TIMEOUT};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

const APP_SERVER_TURN_TIMEOUT: Duration = Duration::from_secs(120);

impl CodexAppServerClient {
    /// Run one Gloamy response through an ephemeral, restricted Codex thread.
    pub async fn infer(
        &mut self,
        model: &str,
        base_instructions: &str,
        input: Vec<Value>,
    ) -> Result<String> {
        self.require_chatgpt_oauth().await?;
        if input.is_empty() {
            anyhow::bail!("Codex App Server inference requires at least one input item");
        }

        let thread_params =
            restricted_thread_start_params(model, base_instructions, &self.working_directory);

        let thread_result = self
            .request("thread/start", thread_params, APP_SERVER_REQUEST_TIMEOUT)
            .await?;
        let thread_id = thread_result
            .pointer("/thread/id")
            .and_then(Value::as_str)
            .context("Codex App Server did not return a thread ID")?
            .to_string();
        let turn_result = self
            .request(
                "turn/start",
                json!({
                    "threadId": thread_id,
                    "input": input
                }),
                APP_SERVER_REQUEST_TIMEOUT,
            )
            .await?;
        let turn_id = turn_result
            .pointer("/turn/id")
            .and_then(Value::as_str)
            .context("Codex App Server did not return a turn ID")?
            .to_string();

        let mut output = String::new();
        let started = Instant::now();
        loop {
            let remaining = APP_SERVER_TURN_TIMEOUT.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                anyhow::bail!("Timed out waiting for Codex App Server inference to complete");
            }
            let notification = self.next_notification(remaining).await?;
            let method = notification.get("method").and_then(Value::as_str);
            let params = notification.get("params");

            match method {
                Some("item/agentMessage/delta") => {
                    let Some(params) = params else {
                        continue;
                    };
                    if params.get("threadId").and_then(Value::as_str) == Some(&thread_id)
                        && params.get("turnId").and_then(Value::as_str) == Some(&turn_id)
                    {
                        if let Some(delta) = params.get("delta").and_then(Value::as_str) {
                            output.push_str(delta);
                        }
                    }
                }
                Some("item/completed") if output.is_empty() => {
                    if let Some(params) = params {
                        if params.get("threadId").and_then(Value::as_str) == Some(&thread_id)
                            && params.get("turnId").and_then(Value::as_str) == Some(&turn_id)
                            && params.pointer("/item/type").and_then(Value::as_str)
                                == Some("agentMessage")
                        {
                            append_completed_message_text(&mut output, params.get("item"));
                        }
                    }
                }
                Some("turn/completed") => {
                    let Some(params) = params else {
                        continue;
                    };
                    if params.get("threadId").and_then(Value::as_str) != Some(&thread_id) {
                        continue;
                    }
                    let Some(turn) = params.get("turn") else {
                        continue;
                    };
                    if turn.get("id").and_then(Value::as_str) != Some(&turn_id) {
                        continue;
                    }
                    if turn.get("status").and_then(Value::as_str) != Some("completed") {
                        let status = turn
                            .get("status")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown");
                        anyhow::bail!("Codex App Server turn ended with status `{status}`");
                    }
                    if output.trim().is_empty() {
                        anyhow::bail!(
                            "Codex App Server completed the turn without an agent message"
                        );
                    }
                    return Ok(output);
                }
                _ => {}
            }
        }
    }
}

pub(super) fn restricted_thread_start_params(
    model: &str,
    base_instructions: &str,
    working_directory: &std::path::Path,
) -> Value {
    let mut params = json!({
        "approvalPolicy": "never",
        "sandbox": "read-only",
        "ephemeral": true,
        "cwd": working_directory,
        "runtimeWorkspaceRoots": [working_directory],
        "environments": [],
        "dynamicTools": [],
        "selectedCapabilityRoots": [],
        "baseInstructions": base_instructions,
        "config": {
            "mcp_servers": {},
            "web_search": "disabled"
        }
    });
    let selected_model = model.trim();
    if !selected_model.is_empty() && !selected_model.eq_ignore_ascii_case("default") {
        params["model"] = Value::String(selected_model.to_string());
    }
    params
}

fn append_completed_message_text(output: &mut String, item: Option<&Value>) {
    let Some(item) = item else {
        return;
    };
    if let Some(text) = item.get("text").and_then(Value::as_str) {
        output.push_str(text);
        return;
    }
    let Some(content) = item.get("content").and_then(Value::as_array) else {
        return;
    };
    for part in content {
        let kind = part.get("type").and_then(Value::as_str);
        if matches!(kind, Some("text") | Some("output_text")) {
            if let Some(text) = part.get("text").and_then(Value::as_str) {
                output.push_str(text);
            }
        }
    }
}
