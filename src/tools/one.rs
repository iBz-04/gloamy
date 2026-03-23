// One Tool Provider — CLI-based integration with 200+ third-party platforms.
//
// Uses the One CLI (`one --agent`) to execute actions on Gmail, Slack, GitHub, etc.
// The CLI handles OAuth, request building, and execution through One's passthrough proxy.
//
// This is opt-in. Users who prefer direct API integrations can skip this.
// The One API key is stored in the encrypted secret store.

use super::traits::{Tool, ToolResult};
use crate::security::SecurityPolicy;
use anyhow::Context;
use async_trait::async_trait;
use serde_json::json;
use std::process::Command;
use std::sync::Arc;

/// A tool that proxies actions to the One managed tool platform via CLI.
pub struct OneTool {
    api_key: String,
    security: Arc<SecurityPolicy>,
}

impl OneTool {
    pub fn new(api_key: &str, security: Arc<SecurityPolicy>) -> Self {
        Self {
            api_key: api_key.to_string(),
            security,
        }
    }

    /// Run a One CLI command and return parsed JSON output.
    async fn run_one_command(&self, args: &[&str]) -> anyhow::Result<serde_json::Value> {
        let mut cmd = Command::new("one");
        cmd.env("ONE_API_KEY", &self.api_key)
            .arg("--agent")
            .args(args);

        let output = cmd.output().context("Failed to execute One CLI")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("One CLI failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value =
            serde_json::from_str(&stdout).context("Failed to parse One CLI output as JSON")?;

        if let Some(error) = json.get("error").and_then(|e| e.as_str()) {
            anyhow::bail!("One API error: {}", error);
        }

        Ok(json)
    }

    async fn list_connections(&self) -> anyhow::Result<serde_json::Value> {
        self.run_one_command(&["connection", "list"]).await
    }

    async fn search_actions(
        &self,
        platform: &str,
        query: &str,
        action_type: &str,
    ) -> anyhow::Result<serde_json::Value> {
        self.run_one_command(&[
            "actions",
            "search",
            platform,
            query,
            "-t",
            action_type,
        ])
        .await
    }

    async fn get_action_knowledge(
        &self,
        platform: &str,
        action_id: &str,
    ) -> anyhow::Result<serde_json::Value> {
        self.run_one_command(&["actions", "knowledge", platform, action_id])
            .await
    }

    async fn execute_action(
        &self,
        platform: &str,
        action_id: &str,
        connection_key: &str,
        data: Option<&str>,
        path_vars: Option<&str>,
        query_params: Option<&str>,
        dry_run: bool,
    ) -> anyhow::Result<serde_json::Value> {
        let mut args = vec![
            "actions",
            "execute",
            platform,
            action_id,
            connection_key,
        ];

        if dry_run {
            args.push("--dry-run");
        }

        if let Some(d) = data {
            args.push("-d");
            args.push(d);
        }

        if let Some(pv) = path_vars {
            args.push("--path-vars");
            args.push(pv);
        }

        if let Some(qp) = query_params {
            args.push("--query-params");
            args.push(qp);
        }

        self.run_one_command(&args).await
    }
}

#[async_trait]
impl Tool for OneTool {
    fn name(&self) -> &str {
        "one"
    }

    fn description(&self) -> &str {
        "Execute actions on 200+ third-party platforms (Gmail, Slack, GitHub, etc.) through the One CLI. \
        Workflow: list_connections, search_actions, get_action_knowledge, execute_action."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["list_connections", "search_actions", "get_action_knowledge", "execute_action"],
                    "description": "The One operation to perform"
                },
                "platform": {
                    "type": "string",
                    "description": "Platform name in kebab-case (e.g., 'gmail', 'github', 'slack')"
                },
                "query": {
                    "type": "string",
                    "description": "Search query for actions (search_actions)"
                },
                "action_type": {
                    "type": "string",
                    "enum": ["execute", "knowledge"],
                    "default": "execute",
                    "description": "Type of action search (search_actions)"
                },
                "action_id": {
                    "type": "string",
                    "description": "Action ID from search results (get_action_knowledge, execute_action)"
                },
                "connection_key": {
                    "type": "string",
                    "description": "Connection key from list_connections (execute_action)"
                },
                "data": {
                    "type": "string",
                    "description": "JSON string of request body (execute_action)"
                },
                "path_vars": {
                    "type": "string",
                    "description": "JSON string of path variables (execute_action)"
                },
                "query_params": {
                    "type": "string",
                    "description": "JSON string of query parameters (execute_action)"
                },
                "dry_run": {
                    "type": "boolean",
                    "default": false,
                    "description": "Show request without executing (execute_action)"
                }
            },
            "required": ["operation"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let operation = args
            .get("operation")
            .and_then(|v| v.as_str())
            .context("operation parameter is required")?;

        let result = match operation {
            "list_connections" => self.list_connections().await,
            "search_actions" => {
                let platform = args
                    .get("platform")
                    .and_then(|v| v.as_str())
                    .context("platform is required for search_actions")?;
                let query = args
                    .get("query")
                    .and_then(|v| v.as_str())
                    .context("query is required for search_actions")?;
                let action_type = args
                    .get("action_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("execute");
                self.search_actions(platform, query, action_type).await
            }
            "get_action_knowledge" => {
                let platform = args
                    .get("platform")
                    .and_then(|v| v.as_str())
                    .context("platform is required for get_action_knowledge")?;
                let action_id = args
                    .get("action_id")
                    .and_then(|v| v.as_str())
                    .context("action_id is required for get_action_knowledge")?;
                self.get_action_knowledge(platform, action_id).await
            }
            "execute_action" => {
                let platform = args
                    .get("platform")
                    .and_then(|v| v.as_str())
                    .context("platform is required for execute_action")?;
                let action_id = args
                    .get("action_id")
                    .and_then(|v| v.as_str())
                    .context("action_id is required for execute_action")?;
                let connection_key = args
                    .get("connection_key")
                    .and_then(|v| v.as_str())
                    .context("connection_key is required for execute_action")?;
                let data = args.get("data").and_then(|v| v.as_str());
                let path_vars = args.get("path_vars").and_then(|v| v.as_str());
                let query_params = args.get("query_params").and_then(|v| v.as_str());
                let dry_run = args.get("dry_run").and_then(|v| v.as_bool()).unwrap_or(false);

                self.execute_action(
                    platform,
                    action_id,
                    connection_key,
                    data,
                    path_vars,
                    query_params,
                    dry_run,
                )
                .await
            }
            _ => anyhow::bail!("Unknown operation: {}", operation),
        };

        match result {
            Ok(json) => Ok(ToolResult {
                success: true,
                output: json.to_string(),
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(e.to_string()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_tool_has_correct_name() {
        let security = Arc::new(SecurityPolicy::default());
        let tool = OneTool::new("test_key", security);
        assert_eq!(tool.name(), "one");
    }

    #[test]
    fn one_tool_has_description() {
        let security = Arc::new(SecurityPolicy::default());
        let tool = OneTool::new("test_key", security);
        assert!(!tool.description().is_empty());
        assert!(tool.description().contains("One"));
    }

    #[test]
    fn one_tool_has_valid_schema() {
        let security = Arc::new(SecurityPolicy::default());
        let tool = OneTool::new("test_key", security);
        let schema = tool.parameters_schema();
        assert!(schema.is_object());
        assert!(schema["properties"].is_object());
        assert!(schema["properties"]["operation"]["enum"].is_array());
    }
}
