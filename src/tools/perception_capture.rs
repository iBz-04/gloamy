use crate::perception::traits::PerceptionProvider;
use crate::perception::traits::{OcrTextItem, ScreenState, WidgetNode};
use crate::perception::{MacOsPerceptionProvider, TesseractOcrProvider};
use crate::security::SecurityPolicy;
use crate::tools::screenshot::ScreenshotTool;
use crate::tools::traits::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

/// A tool that captures the multimodal perceived state of the screen,
/// including visual (screenshot), structured (widget tree), and eventually text (OCR).
pub struct PerceptionCaptureTool {
    security: Arc<SecurityPolicy>,
    screenshot_tool: ScreenshotTool,
}

impl PerceptionCaptureTool {
    pub fn new(security: Arc<SecurityPolicy>) -> Self {
        Self {
            screenshot_tool: ScreenshotTool::new(security.clone()),
            security,
        }
    }
}

#[async_trait]
impl Tool for PerceptionCaptureTool {
    fn name(&self) -> &str {
        "perception_capture"
    }

    fn description(&self) -> &str {
        "Capture the complete multimodal state of the graphical environment. Returns both a visual screenshot [IMAGE:...] and a structured UI accessibility widget tree. Use this when you need precise coordinates or hierarchical understanding of the current application state before taking a GUI action."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "include_screenshot": {
                    "type": "boolean",
                    "description": "If true, takes and returns a screenshot. Default is true."
                },
                "include_widget_tree": {
                    "type": "boolean",
                    "description": "If true, extracts and returns the structured macOS accessibility tree. Default is true."
                },
                "include_ocr": {
                    "type": "boolean",
                    "description": "If true, extracts text bounding boxes from the screenshot via Tesseract OCR. Default is false."
                }
            }
        })
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        if !self.security.can_act() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Action blocked: autonomy is read-only".into()),
            });
        }

        let include_screenshot = args
            .get("include_screenshot")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let include_widget_tree = args
            .get("include_widget_tree")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let include_ocr = args
            .get("include_ocr")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut widget_tree: Option<WidgetNode> = None;
        let mut ocr_items: Vec<OcrTextItem> = Vec::new();
        let mut screenshot_path: Option<String> = None;
        let mut errors = Vec::new();
        let mut completed_modalities = 0usize;
        let requested_modalities = usize::from(include_widget_tree)
            + usize::from(include_screenshot || include_ocr)
            + usize::from(include_ocr);

        if include_widget_tree {
            if cfg!(target_os = "macos") {
                let provider = MacOsPerceptionProvider::new();
                match provider.capture_state().await {
                    Ok(state) => {
                        if let Some(tree) = state.widget_tree {
                            widget_tree = Some(tree);
                            completed_modalities += 1;
                        } else {
                            errors.push(
                                "Widget tree capture returned no active window tree.".to_string(),
                            );
                        }
                    }
                    Err(e) => {
                        errors.push(format!("Widget tree capture failed: {}", e));
                    }
                }
            } else {
                errors.push(
                    "Widget tree extraction is currently only supported on macOS.".to_string(),
                );
            }
        }

        let screenshot_file = format!("perception_{}.png", chrono::Utc::now().timestamp());

        if include_screenshot || include_ocr {
            let capture_args = json!({"filename": screenshot_file.clone()});
            match self.screenshot_tool.execute(capture_args).await {
                Ok(res) => {
                    if res.success {
                        screenshot_path = Some(
                            self.security
                                .workspace_dir
                                .join(&screenshot_file)
                                .to_string_lossy()
                                .to_string(),
                        );
                        completed_modalities += 1;
                    } else {
                        errors.push(
                            res.error
                                .unwrap_or_else(|| "Screenshot failed without reason".to_string()),
                        );
                    }
                }
                Err(e) => {
                    errors.push(format!("Screenshot execution failed: {}", e));
                }
            }
        }

        if include_ocr {
            if let Some(path) = screenshot_path.as_deref() {
                match TesseractOcrProvider::extract_text(std::path::Path::new(path)).await {
                    Ok(items) => {
                        ocr_items = items;
                        completed_modalities += 1;
                    }
                    Err(e) => {
                        errors.push(format!("OCR extraction failed: {}", e));
                    }
                }
            } else {
                errors.push("OCR requested but screenshot capture failed".to_string());
            }
        }

        let fused_state = ScreenState {
            screenshot_path: screenshot_path.clone(),
            widget_tree,
            extracted_text: ocr_items,
        };
        let payload = json!({
            "screen_state": fused_state,
            "diagnostics": {
                "requested_modalities": requested_modalities,
                "completed_modalities": completed_modalities,
                "errors": errors.clone(),
            }
        });

        let mut output = serde_json::to_string_pretty(&payload)?;
        if let Some(path) = screenshot_path {
            output = format!("[IMAGE:{path}]\n{output}");
        }

        let success = if requested_modalities == 0 {
            true
        } else {
            completed_modalities > 0
        };

        Ok(ToolResult {
            success,
            output,
            error: if errors.is_empty() {
                None
            } else {
                Some(errors.join(" | "))
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::{AutonomyLevel, SecurityPolicy};

    fn readonly_security() -> Arc<SecurityPolicy> {
        Arc::new(SecurityPolicy {
            autonomy: AutonomyLevel::ReadOnly,
            workspace_dir: std::env::temp_dir(),
            ..SecurityPolicy::default()
        })
    }

    #[test]
    fn perception_capture_tool_name_and_schema_are_stable() {
        let tool = PerceptionCaptureTool::new(readonly_security());
        assert_eq!(tool.name(), "perception_capture");
        let schema = tool.parameters_schema();
        assert!(schema["properties"]["include_screenshot"].is_object());
        assert!(schema["properties"]["include_widget_tree"].is_object());
        assert!(schema["properties"]["include_ocr"].is_object());
    }

    #[tokio::test]
    async fn perception_capture_blocks_in_readonly_mode() {
        let tool = PerceptionCaptureTool::new(readonly_security());
        let result = tool
            .execute(json!({}))
            .await
            .expect("tool should return result");
        assert!(!result.success);
        assert!(result
            .error
            .as_deref()
            .is_some_and(|msg| msg.contains("read-only")));
    }
}
