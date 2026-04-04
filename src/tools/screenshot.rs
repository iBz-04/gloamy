use super::traits::{Tool, ToolResult};
use crate::security::SecurityPolicy;
use async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// Maximum time to wait for a screenshot command to complete.
const SCREENSHOT_TIMEOUT_SECS: u64 = 15;
const MACOS_SCREENSHOT_MAX_ATTEMPTS: usize = 2;
const SCREENSHOT_RETRY_DELAY_MS: u64 = 750;

/// Tool for capturing screenshots using platform-native commands.
///
/// macOS: `screencapture`
/// Linux: tries `gnome-screenshot`, `scrot`, `import` (`ImageMagick`) in order.
pub struct ScreenshotTool {
    security: Arc<SecurityPolicy>,
}

impl ScreenshotTool {
    pub fn new(security: Arc<SecurityPolicy>) -> Self {
        Self { security }
    }

    /// Determine the screenshot command for the current platform.
    fn screenshot_command(output_path: &str) -> Option<Vec<String>> {
        if cfg!(target_os = "macos") {
            Some(vec![
                "screencapture".into(),
                "-x".into(), // no sound
                output_path.into(),
            ])
        } else if cfg!(target_os = "linux") {
            Some(vec![
                "sh".into(),
                "-c".into(),
                format!(
                    "if command -v gnome-screenshot >/dev/null 2>&1; then \
                         gnome-screenshot -f '{output_path}'; \
                     elif command -v scrot >/dev/null 2>&1; then \
                         scrot '{output_path}'; \
                     elif command -v import >/dev/null 2>&1; then \
                         import -window root '{output_path}'; \
                     else \
                         echo 'NO_SCREENSHOT_TOOL' >&2; exit 1; \
                     fi"
                ),
            ])
        } else {
            None
        }
    }

    /// Execute the screenshot capture and return the result.
    async fn capture(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = args
            .get("filename")
            .and_then(|v| v.as_str())
            .map_or_else(|| format!("screenshot_{timestamp}.png"), String::from);

        // Sanitize filename to prevent path traversal
        let safe_name = PathBuf::from(&filename).file_name().map_or_else(
            || format!("screenshot_{timestamp}.png"),
            |n| n.to_string_lossy().to_string(),
        );

        // Reject filenames with shell-breaking characters to prevent injection in sh -c
        const SHELL_UNSAFE: &[char] = &[
            '\'', '"', '`', '$', '\\', ';', '|', '&', '\n', '\0', '(', ')',
        ];
        if safe_name.contains(SHELL_UNSAFE) {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Filename contains characters unsafe for shell execution".into()),
            });
        }

        let output_path = self.security.workspace_dir.join(&safe_name);
        let output_str = output_path.to_string_lossy().to_string();

        let Some(mut cmd_args) = Self::screenshot_command(&output_str) else {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Screenshot not supported on this platform".into()),
            });
        };

        // macOS region flags
        if cfg!(target_os = "macos") {
            if let Some(region) = args.get("region").and_then(|v| v.as_str()) {
                match region {
                    "selection" => cmd_args.insert(1, "-s".into()),
                    "window" => cmd_args.insert(1, "-w".into()),
                    _ => {} // ignore unknown regions
                }
            }
        }

        let program = cmd_args.remove(0);
        let max_attempts = if cfg!(target_os = "macos") {
            MACOS_SCREENSHOT_MAX_ATTEMPTS
        } else {
            1
        };

        for attempt in 1..=max_attempts {
            let mut command = tokio::process::Command::new(&program);
            command.kill_on_drop(true);
            command.args(&cmd_args);

            let result = tokio::time::timeout(
                Duration::from_secs(SCREENSHOT_TIMEOUT_SECS),
                command.output(),
            )
            .await;

            match result {
                Ok(Ok(output)) => {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if stderr.contains("NO_SCREENSHOT_TOOL") {
                            return Ok(ToolResult {
                                success: false,
                                output: String::new(),
                                error: Some(
                                    "No screenshot tool found. Install gnome-screenshot, scrot, or ImageMagick."
                                        .into(),
                                ),
                            });
                        }
                        return Ok(ToolResult {
                            success: false,
                            output: String::new(),
                            error: Some(format!("Screenshot command failed: {stderr}")),
                        });
                    }

                    return Self::build_image_result(&output_path).await;
                }
                Ok(Err(e)) => {
                    return Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Failed to execute screenshot command: {e}")),
                    });
                }
                Err(_) if attempt < max_attempts => {
                    tokio::time::sleep(Duration::from_millis(SCREENSHOT_RETRY_DELAY_MS)).await;
                }
                Err(_) => {
                    return Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!(
                            "Screenshot timed out after {SCREENSHOT_TIMEOUT_SECS}s"
                        )),
                    });
                }
            }
        }

        unreachable!("screenshot retry loop should always return before exhaustion")
    }

    /// Build a tool result referencing the screenshot via [IMAGE:] marker.
    ///
    /// The multimodal pipeline resolves the marker to a proper image content
    /// block for the provider, so the LLM can visually inspect the screenshot.
    /// This replaces the previous approach of dumping raw base64 into the tool
    /// output (which was invisible to the LLM since providers only process
    /// image markers in user-role messages).
    async fn build_image_result(output_path: &std::path::Path) -> anyhow::Result<ToolResult> {
        let size = match tokio::fs::metadata(output_path).await {
            Ok(meta) => meta.len(),
            Err(e) => {
                return Ok(ToolResult {
                    success: false,
                    output: format!("Screenshot saved to: {}", output_path.display()),
                    error: Some(format!("Failed to read screenshot file: {e}")),
                });
            }
        };

        Ok(ToolResult {
            success: true,
            output: format!(
                "Screenshot saved to: {path}\nSize: {size} bytes\n[IMAGE:{path}]",
                path = output_path.display(),
            ),
            error: None,
        })
    }
}

#[async_trait]
impl Tool for ScreenshotTool {
    fn name(&self) -> &str {
        "screenshot"
    }

    fn description(&self) -> &str {
        "Capture a screenshot of the current screen (pixel dump). Returns the file path and base64-encoded PNG data. This is a passive screen-capture tool — it does NOT interact with any application. To trigger an in-app action like clicking a camera shutter button (e.g. Photo Booth, FaceTime), use mac_automation with click_at or run_applescript instead."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "filename": {
                    "type": "string",
                    "description": "Optional filename (default: screenshot_<timestamp>.png). Saved in workspace."
                },
                "region": {
                    "type": "string",
                    "description": "Optional region for macOS: 'selection' for interactive crop, 'window' for front window. Ignored on Linux."
                }
            }
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
        self.capture(args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::{AutonomyLevel, SecurityPolicy};

    fn test_security() -> Arc<SecurityPolicy> {
        Arc::new(SecurityPolicy {
            autonomy: AutonomyLevel::Full,
            workspace_dir: std::env::temp_dir(),
            ..SecurityPolicy::default()
        })
    }

    #[test]
    fn screenshot_tool_name() {
        let tool = ScreenshotTool::new(test_security());
        assert_eq!(tool.name(), "screenshot");
    }

    #[test]
    fn screenshot_tool_description() {
        let tool = ScreenshotTool::new(test_security());
        assert!(!tool.description().is_empty());
        assert!(tool.description().contains("screenshot"));
    }

    #[test]
    fn screenshot_tool_schema() {
        let tool = ScreenshotTool::new(test_security());
        let schema = tool.parameters_schema();
        assert!(schema["properties"]["filename"].is_object());
        assert!(schema["properties"]["region"].is_object());
    }

    #[test]
    fn screenshot_tool_spec() {
        let tool = ScreenshotTool::new(test_security());
        let spec = tool.spec();
        assert_eq!(spec.name, "screenshot");
        assert!(spec.parameters.is_object());
    }

    #[test]
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn screenshot_command_exists() {
        let cmd = ScreenshotTool::screenshot_command("/tmp/test.png");
        assert!(cmd.is_some());
        let args = cmd.unwrap();
        assert!(!args.is_empty());
    }

    #[tokio::test]
    async fn screenshot_rejects_shell_injection_filename() {
        let tool = ScreenshotTool::new(test_security());
        let result = tool
            .execute(json!({"filename": "test'injection.png"}))
            .await
            .unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("unsafe for shell execution"));
    }

    #[test]
    fn screenshot_command_contains_output_path() {
        let cmd = ScreenshotTool::screenshot_command("/tmp/my_screenshot.png").unwrap();
        let joined = cmd.join(" ");
        assert!(
            joined.contains("/tmp/my_screenshot.png"),
            "Command should contain the output path"
        );
    }
}
