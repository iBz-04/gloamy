use super::gui_verify;
use super::traits::{GuiExpectation, PreObservationStrategy, Tool, ToolResult};
use crate::security::SecurityPolicy;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;

const MAC_AUTOMATION_TIMEOUT_SECS: u64 = 20;
const MAX_APP_NAME_CHARS: usize = 120;
const MAX_SCRIPT_CHARS: usize = 8_000;
const MAX_OUTPUT_CHARS: usize = 8_000;

/// macOS desktop automation helper (launch/activate apps and run AppleScript).
///
/// This tool provides an explicit, policy-aware path for local GUI automation
/// so the agent does not need to improvise via shell allowlist gaps.
pub struct MacAutomationTool {
    security: Arc<SecurityPolicy>,
}

impl MacAutomationTool {
    pub fn new(security: Arc<SecurityPolicy>) -> Self {
        Self { security }
    }

    fn parse_action(args: &serde_json::Value) -> anyhow::Result<&str> {
        args.get("action")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| anyhow::anyhow!("Missing 'action' parameter"))
    }

    fn parse_app_name(args: &serde_json::Value) -> anyhow::Result<String> {
        let app_name = args
            .get("app_name")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| anyhow::anyhow!("Missing 'app_name' parameter"))?;

        if app_name.chars().count() > MAX_APP_NAME_CHARS {
            anyhow::bail!("app_name is too long (max {MAX_APP_NAME_CHARS} characters)");
        }
        if app_name.chars().any(char::is_control) {
            anyhow::bail!("app_name contains control characters");
        }

        Ok(app_name.to_string())
    }

    fn parse_applescript_lines(args: &serde_json::Value) -> anyhow::Result<Vec<String>> {
        if let Some(script) = args.get("script").and_then(serde_json::Value::as_str) {
            let trimmed = script.trim();
            if trimmed.is_empty() {
                anyhow::bail!("'script' cannot be empty");
            }
            if trimmed.chars().count() > MAX_SCRIPT_CHARS {
                anyhow::bail!("script is too long (max {MAX_SCRIPT_CHARS} characters)");
            }
            if trimmed.contains('\0') {
                anyhow::bail!("script contains a null byte");
            }
            return Ok(vec![trimmed.to_string()]);
        }

        let lines = args
            .get("script_lines")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| anyhow::anyhow!("Provide either 'script' or 'script_lines'"))?;

        if lines.is_empty() {
            anyhow::bail!("'script_lines' cannot be empty");
        }

        let mut total_len = 0usize;
        let mut parsed = Vec::with_capacity(lines.len());
        for (idx, line) in lines.iter().enumerate() {
            let value = line
                .as_str()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .ok_or_else(|| anyhow::anyhow!("script_lines[{idx}] must be a non-empty string"))?;

            if value.contains('\0') {
                anyhow::bail!("script_lines[{idx}] contains a null byte");
            }

            total_len += value.len();
            if total_len > MAX_SCRIPT_CHARS {
                anyhow::bail!("script_lines total size exceeds {MAX_SCRIPT_CHARS} characters");
            }
            parsed.push(value.to_string());
        }

        Ok(parsed)
    }

    fn escape_applescript_literal(input: &str) -> String {
        input
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', " ")
            .replace('\r', " ")
    }

    fn truncate_output(raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        let mut out = trimmed.to_string();
        if out.len() > MAX_OUTPUT_CHARS {
            out.truncate(out.floor_char_boundary(MAX_OUTPUT_CHARS));
            out.push_str("\n... [output truncated]");
        }
        out
    }

    async fn run_macos_command(program: &str, args: &[String]) -> anyhow::Result<ToolResult> {
        let result = tokio::time::timeout(
            Duration::from_secs(MAC_AUTOMATION_TIMEOUT_SECS),
            tokio::process::Command::new(program).args(args).output(),
        )
        .await;

        match result {
            Ok(Ok(output)) => {
                let stdout = Self::truncate_output(&String::from_utf8_lossy(&output.stdout));
                let stderr = Self::truncate_output(&String::from_utf8_lossy(&output.stderr));
                if output.status.success() {
                    Ok(ToolResult {
                        success: true,
                        output: if stdout.is_empty() {
                            "ok".to_string()
                        } else {
                            stdout
                        },
                        error: None,
                    })
                } else {
                    Ok(ToolResult {
                        success: false,
                        output: stdout,
                        error: Some(if stderr.is_empty() {
                            format!("{program} exited with status {}", output.status)
                        } else {
                            stderr
                        }),
                    })
                }
            }
            Ok(Err(error)) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to execute {program}: {error}")),
            }),
            Err(_) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!(
                    "{program} timed out after {MAC_AUTOMATION_TIMEOUT_SECS}s"
                )),
            }),
        }
    }

    async fn collect_gui_evidence(&self, keys: &[String]) -> anyhow::Result<Value> {
        let mut evidence = json!({});

        if keys.iter().any(|key| key == "title") {
            let title = self.frontmost_window_title().await.unwrap_or_default();
            merge_json_objects(&mut evidence, &json!({ "title": title }));
        }

        if keys.iter().any(|key| key == "dialog_present") {
            let present = self.frontmost_dialog_present().await.unwrap_or(false);
            merge_json_objects(&mut evidence, &json!({ "dialog_present": present }));
        }

        Ok(evidence)
    }

    async fn maybe_collect_pre_observation(
        &self,
        strategy: &PreObservationStrategy,
        expectations: &[GuiExpectation],
    ) -> Option<super::traits::GuiObservation> {
        let keys = match strategy {
            PreObservationStrategy::None => Vec::new(),
            PreObservationStrategy::Auto => gui_verify::infer_pre_observation_keys(expectations),
            PreObservationStrategy::Explicit { keys } => keys.clone(),
        };

        if keys.is_empty() {
            return None;
        }

        match self.collect_gui_evidence(&keys).await {
            Ok(evidence) => Some(gui_verify::observation("mac_pre", evidence)),
            Err(_) => None,
        }
    }

    async fn frontmost_window_title(&self) -> anyhow::Result<String> {
        let args = vec![
            "-e".to_string(),
            "tell application \"System Events\"".to_string(),
            "-e".to_string(),
            "tell (first process whose frontmost is true)".to_string(),
            "-e".to_string(),
            "try".to_string(),
            "-e".to_string(),
            "return name of front window".to_string(),
            "-e".to_string(),
            "on error".to_string(),
            "-e".to_string(),
            "return \"\"".to_string(),
            "-e".to_string(),
            "end try".to_string(),
            "-e".to_string(),
            "end tell".to_string(),
            "-e".to_string(),
            "end tell".to_string(),
        ];

        let result = Self::run_macos_command("osascript", &args).await?;
        if !result.success {
            anyhow::bail!(result.error.unwrap_or_else(|| "osascript failed".into()));
        }
        Ok(result.output)
    }

    async fn frontmost_dialog_present(&self) -> anyhow::Result<bool> {
        let args = vec![
            "-e".to_string(),
            "tell application \"System Events\"".to_string(),
            "-e".to_string(),
            "tell (first process whose frontmost is true)".to_string(),
            "-e".to_string(),
            "try".to_string(),
            "-e".to_string(),
            "if exists (sheet 1 of front window) then return \"true\"".to_string(),
            "-e".to_string(),
            "end try".to_string(),
            "-e".to_string(),
            "return \"false\"".to_string(),
            "-e".to_string(),
            "end tell".to_string(),
            "-e".to_string(),
            "end tell".to_string(),
        ];

        let result = Self::run_macos_command("osascript", &args).await?;
        if !result.success {
            anyhow::bail!(result.error.unwrap_or_else(|| "osascript failed".into()));
        }
        Ok(result.output.trim().eq_ignore_ascii_case("true"))
    }
}

#[async_trait]
impl Tool for MacAutomationTool {
    fn name(&self) -> &str {
        "mac_automation"
    }

    fn description(&self) -> &str {
        "macOS desktop automation: launch or activate applications and run AppleScript for UI workflows. Success means the automation command ran, not that the UI state is verified. After state-changing actions, follow with an AppleScript read-back or screenshot check before concluding the task is complete."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["launch_app", "activate_app", "run_applescript"],
                    "description": "Automation action to execute"
                },
                "app_name": {
                    "type": "string",
                    "description": "Application name for launch_app/activate_app (e.g., 'MongoDB Compass')"
                },
                "script": {
                    "type": "string",
                    "description": "AppleScript source for run_applescript"
                },
                "script_lines": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "AppleScript passed as multiple lines (-e per line)"
                },
                "expect": {
                    "description": "Verification expectation(s) for mutating actions. When provided, success means the expected UI state was verified, not just that the command exited 0. Accepts a single object or array. Each object must have a 'kind' field: field_value_equals, focused_element_is, checkbox_checked, window_title_contains, dialog_present, url_is, url_host_is, file_exists, download_completed, element_at_coordinate.",
                    "oneOf": [
                        { "type": "object", "properties": { "kind": { "type": "string" } }, "required": ["kind"] },
                        { "type": "array", "items": { "type": "object", "properties": { "kind": { "type": "string" } }, "required": ["kind"] } }
                    ]
                },
                "pre_observe": {
                    "description": "Optional pre-action observation strategy. Use \"auto\" to infer evidence keys from expectations or provide {\"keys\": [...]}."
                },
                "wait": {
                    "description": "Optional wait strategy object. Example: {\"strategy\":\"fixed_ms\",\"ms\":500}."
                },
                "reversibility": {
                    "type": "string",
                    "enum": ["reversible", "partially_reversible", "irreversible", "unknown"],
                    "description": "Optional caller override for GUI action reversibility classification."
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        if !self.security.can_act() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Action blocked: autonomy is read-only".into()),
            });
        }

        if !self.security.record_action() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Action blocked: rate limit exceeded".into()),
            });
        }

        // Parse verification expectations (if any) before executing
        let expectations = match gui_verify::parse_expectations(&args) {
            Ok(exps) => exps,
            Err(e) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(e.to_string()),
                });
            }
        };

        let pre_observe = match gui_verify::parse_pre_observation_strategy(&args) {
            Ok(strategy) => strategy,
            Err(error) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(error.to_string()),
                });
            }
        };

        let wait_strategy = match gui_verify::parse_wait_strategy(&args) {
            Ok(strategy) => strategy,
            Err(error) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(error.to_string()),
                });
            }
        };

        let action = match Self::parse_action(&args) {
            Ok(action) => action,
            Err(error) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(error.to_string()),
                })
            }
        };

        let expectations = match expectations {
            Some(exps) => exps,
            None => Vec::new(),
        };
        let pre_observation = self
            .maybe_collect_pre_observation(&pre_observe, &expectations)
            .await;

        let raw_result = match action {
            "launch_app" => {
                let app_name = match Self::parse_app_name(&args) {
                    Ok(value) => value,
                    Err(error) => {
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(error.to_string()),
                        })
                    }
                };
                let launch = Self::run_macos_command(
                    "open",
                    &["-a".to_string(), app_name.clone()],
                )
                .await?;

                ToolResult {
                    success: launch.success,
                    output: if launch.success {
                        format!("Launched app: {app_name}")
                    } else {
                        launch.output
                    },
                    error: launch.error,
                }
            }
            "activate_app" => {
                let app_name = match Self::parse_app_name(&args) {
                    Ok(value) => value,
                    Err(error) => {
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(error.to_string()),
                        })
                    }
                };
                let script = format!(
                    "tell application \"{}\" to activate",
                    Self::escape_applescript_literal(&app_name)
                );
                let activation = Self::run_macos_command(
                    "osascript",
                    &["-e".to_string(), script],
                )
                .await?;

                ToolResult {
                    success: activation.success,
                    output: if activation.success {
                        format!("Activated app: {app_name}")
                    } else {
                        activation.output
                    },
                    error: activation.error,
                }
            }
            "run_applescript" => {
                let script_lines = match Self::parse_applescript_lines(&args) {
                    Ok(value) => value,
                    Err(error) => {
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(error.to_string()),
                        })
                    }
                };
                let script_args: Vec<String> = script_lines
                    .into_iter()
                    .flat_map(|line| ["-e".to_string(), line])
                    .collect();

                Self::run_macos_command("osascript", &script_args).await?
            }
            other => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!(
                        "Unsupported action '{other}'. Allowed: launch_app, activate_app, run_applescript"
                    )),
                })
            }
        };

        // If no expectations, return the raw result (unverified/raw mode)
        if expectations.is_empty() {
            return Ok(raw_result);
        }

        // ── GUI verification: build post-action evidence and verify ──
        let base_evidence = parse_tool_output(&raw_result.output);
        let post_keys = gui_verify::infer_pre_observation_keys(&expectations);
        let post_evidence = if raw_result.success {
            match gui_verify::apply_wait_strategy(
                &wait_strategy,
                || async {
                    let mut evidence = base_evidence.clone();
                    let extra = self.collect_gui_evidence(&post_keys).await?;
                    merge_json_objects(&mut evidence, &extra);
                    Ok(evidence)
                },
                &expectations,
            )
            .await
            {
                Ok(evidence) => evidence,
                Err(error) => {
                    return Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(error.to_string()),
                    });
                }
            }
        } else {
            base_evidence
        };

        let post_obs = gui_verify::observation("osascript_output", post_evidence.clone());
        let report = gui_verify::build_report(
            raw_result.success,
            pre_observation,
            Some(post_obs),
            &expectations,
            &post_evidence,
        );

        Ok(ToolResult::from_gui_report(&report))
    }
}

fn parse_tool_output(output: &str) -> Value {
    serde_json::from_str(output).unwrap_or_else(|_| json!({"output": output}))
}

fn merge_json_objects(target: &mut Value, source: &Value) {
    match (target, source) {
        (Value::Object(target_map), Value::Object(source_map)) => {
            for (key, value) in source_map {
                match target_map.get_mut(key) {
                    Some(existing) => merge_json_objects(existing, value),
                    None => {
                        target_map.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        (target_slot, source_value) => {
            *target_slot = source_value.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::{AutonomyLevel, SecurityPolicy};

    fn test_security(autonomy: AutonomyLevel) -> Arc<SecurityPolicy> {
        Arc::new(SecurityPolicy {
            autonomy,
            workspace_dir: std::env::temp_dir(),
            ..SecurityPolicy::default()
        })
    }

    #[test]
    fn mac_automation_tool_name() {
        let tool = MacAutomationTool::new(test_security(AutonomyLevel::Supervised));
        assert_eq!(tool.name(), "mac_automation");
    }

    #[test]
    fn mac_automation_schema_has_actions() {
        let tool = MacAutomationTool::new(test_security(AutonomyLevel::Supervised));
        let schema = tool.parameters_schema();
        assert!(schema["properties"]["action"].is_object());
        assert!(schema["properties"]["app_name"].is_object());
        assert!(schema["properties"]["script"].is_object());
    }

    #[tokio::test]
    async fn mac_automation_blocks_in_readonly() {
        let tool = MacAutomationTool::new(test_security(AutonomyLevel::ReadOnly));
        let result = tool
            .execute(json!({"action":"launch_app","app_name":"MongoDB Compass"}))
            .await
            .expect("readonly execution should return a structured tool result");
        assert!(!result.success);
        assert!(result.error.unwrap_or_default().contains("read-only"));
    }

    #[tokio::test]
    async fn mac_automation_requires_script_for_applescript_action() {
        let tool = MacAutomationTool::new(test_security(AutonomyLevel::Supervised));
        let result = tool
            .execute(json!({"action":"run_applescript"}))
            .await
            .expect("missing script should return a structured tool result");
        assert!(!result.success);
        assert!(result
            .error
            .unwrap_or_default()
            .contains("Provide either 'script' or 'script_lines'"));
    }
}
