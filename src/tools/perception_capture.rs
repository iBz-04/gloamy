use crate::perception::traits::PerceptionProvider;
use crate::perception::traits::{OcrTextItem, ScreenState, WidgetNode};
use crate::perception::{MacOsPerceptionProvider, OcrConfig, TesseractOcrProvider};
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

    fn parse_ocr_config(args: &Value) -> anyhow::Result<OcrConfig> {
        let ocr_args = args.get("ocr").and_then(Value::as_object);
        let mut config = OcrConfig::default();

        if let Some(ocr_args) = ocr_args {
            if let Some(language) = ocr_args.get("language").and_then(Value::as_str) {
                config.language = language.to_string();
            }
            if let Some(psm) = ocr_args.get("psm").and_then(Value::as_u64) {
                config.psm = u8::try_from(psm)
                    .map_err(|_| anyhow::anyhow!("OCR psm must be in range 0..=13"))?;
            }
            if let Some(oem) = ocr_args.get("oem").and_then(Value::as_u64) {
                config.oem = u8::try_from(oem)
                    .map_err(|_| anyhow::anyhow!("OCR oem must be in range 0..=3"))?;
            }
            if let Some(tessdata_dir) = ocr_args.get("tessdata_dir").and_then(Value::as_str) {
                config.tessdata_dir = Some(tessdata_dir.to_string());
            }
        }

        config.validate()?;
        Ok(config)
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
                },
                "ocr": {
                    "type": "object",
                    "description": "Optional OCR config for deterministic Tesseract behavior when include_ocr=true.",
                    "properties": {
                        "language": {
                            "type": "string",
                            "description": "Tesseract language code(s). Default: 'eng'."
                        },
                        "psm": {
                            "type": "integer",
                            "minimum": 0,
                            "maximum": 13,
                            "description": "Tesseract page segmentation mode. Default: 11 (sparse text)."
                        },
                        "oem": {
                            "type": "integer",
                            "minimum": 0,
                            "maximum": 3,
                            "description": "Tesseract OCR engine mode. Default: 1 (LSTM)."
                        },
                        "tessdata_dir": {
                            "type": "string",
                            "description": "Optional tessdata directory to pin traineddata source."
                        }
                    }
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
        let ocr_config = Self::parse_ocr_config(&args)?;

        let mut widget_tree: Option<WidgetNode> = None;
        let mut ocr_items: Vec<OcrTextItem> = Vec::new();
        let mut screenshot_path: Option<String> = None;
        let mut errors = Vec::new();
        let mut completed_modalities = 0usize;
        let mut widget_tree_completed = false;
        let mut screenshot_completed = false;
        let mut ocr_completed = false;
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
                            widget_tree_completed = true;
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
                        screenshot_completed = true;
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
                match TesseractOcrProvider::extract_text_with_config(
                    std::path::Path::new(path),
                    &ocr_config,
                )
                .await
                {
                    Ok(items) => {
                        ocr_items = items;
                        ocr_completed = true;
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
                "modalities": {
                    "widget_tree": {
                        "requested": include_widget_tree,
                        "completed": widget_tree_completed,
                    },
                    "screenshot": {
                        "requested": include_screenshot || include_ocr,
                        "completed": screenshot_completed,
                    },
                    "ocr": {
                        "requested": include_ocr,
                        "completed": ocr_completed,
                        "config": {
                            "language": ocr_config.language,
                            "psm": ocr_config.psm,
                            "oem": ocr_config.oem,
                            "tessdata_dir": ocr_config.tessdata_dir,
                        }
                    }
                },
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
        assert!(schema["properties"]["ocr"]["properties"]["language"].is_object());
        assert!(schema["properties"]["ocr"]["properties"]["psm"].is_object());
        assert!(schema["properties"]["ocr"]["properties"]["oem"].is_object());
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
