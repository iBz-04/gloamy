use crate::grounding::traits::{
    GroundingExpert, GroundingOutcome, GroundingRequest, GroundingSignal,
};
use crate::perception::traits::{PerceptionProvider, ScreenState, WidgetNode};
use crate::perception::{MacOsPerceptionProvider, OcrConfig, TesseractOcrProvider};
use anyhow::Context;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use uuid::Uuid;

const SCREENSHOT_TIMEOUT_SECS: u64 = 15;

pub struct WidgetTreeGroundingExpert;

#[async_trait]
impl GroundingExpert for WidgetTreeGroundingExpert {
    fn name(&self) -> &str {
        "widget_tree_grounding_expert"
    }

    fn signal(&self) -> GroundingSignal {
        GroundingSignal::WidgetTree
    }

    fn routing_score(&self, request: &GroundingRequest, state: &ScreenState) -> f32 {
        if request.include_widget_tree && state.widget_tree.is_none() {
            0.95
        } else {
            0.0
        }
    }

    async fn ground(
        &self,
        request: &GroundingRequest,
        state: &mut ScreenState,
    ) -> anyhow::Result<GroundingOutcome> {
        if !request.include_widget_tree {
            return Ok(GroundingOutcome {
                expert: self.name().to_string(),
                signal: self.signal(),
                confidence: 0.0,
                updated: false,
            });
        }

        if !cfg!(target_os = "macos") {
            anyhow::bail!("widget tree capture is currently supported on macOS only");
        }

        let provider = MacOsPerceptionProvider::new();
        let result = provider
            .capture_state()
            .await
            .context("macOS accessibility capture failed")?;
        let updated = result.widget_tree.is_some();
        state.widget_tree = result.widget_tree;

        Ok(GroundingOutcome {
            expert: self.name().to_string(),
            signal: self.signal(),
            confidence: if updated { 0.92 } else { 0.0 },
            updated,
        })
    }
}

pub struct RuntimeContextGroundingExpert;

impl RuntimeContextGroundingExpert {
    fn infer_runtime_app_name() -> String {
        #[cfg(target_os = "windows")]
        {
            if std::env::var("WT_SESSION").is_ok() {
                return "Windows Terminal".to_string();
            }
            if std::env::var("TERM_PROGRAM").is_ok() {
                return std::env::var("TERM_PROGRAM").unwrap_or_else(|_| "PowerShell".to_string());
            }
            if std::env::var("ComSpec").is_ok() {
                return "Command Prompt".to_string();
            }
            return "Windows Runtime".to_string();
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(value) = std::env::var("TERM_PROGRAM") {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
            return "Terminal".to_string();
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(value) = std::env::var("TERM_PROGRAM") {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
            if let Ok(value) = std::env::var("TERM") {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
            return "Terminal".to_string();
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            if let Ok(value) = std::env::var("TERM_PROGRAM") {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
            "Runtime".to_string()
        }
    }

    fn normalize_identifier(value: &str) -> String {
        let mut id = String::with_capacity(value.len());
        for ch in value.chars() {
            if ch.is_ascii_alphanumeric() {
                id.push(ch.to_ascii_lowercase());
            } else if ch.is_ascii_whitespace() || ch == '-' || ch == '_' {
                id.push('_');
            }
        }
        let collapsed = id
            .split('_')
            .filter(|chunk| !chunk.is_empty())
            .collect::<Vec<_>>()
            .join("_");
        if collapsed.is_empty() {
            "runtime_context".to_string()
        } else {
            collapsed
        }
    }
}

#[async_trait]
impl GroundingExpert for RuntimeContextGroundingExpert {
    fn name(&self) -> &str {
        "runtime_context_grounding_expert"
    }

    fn signal(&self) -> GroundingSignal {
        GroundingSignal::WidgetTree
    }

    fn routing_score(&self, request: &GroundingRequest, state: &ScreenState) -> f32 {
        if request.include_widget_tree && state.widget_tree.is_none() {
            0.35
        } else {
            0.0
        }
    }

    async fn ground(
        &self,
        request: &GroundingRequest,
        state: &mut ScreenState,
    ) -> anyhow::Result<GroundingOutcome> {
        if !request.include_widget_tree || state.widget_tree.is_some() {
            return Ok(GroundingOutcome {
                expert: self.name().to_string(),
                signal: self.signal(),
                confidence: 0.0,
                updated: false,
            });
        }

        let app_name = Self::infer_runtime_app_name();
        let id = Self::normalize_identifier(&app_name);
        state.widget_tree = Some(WidgetNode {
            id,
            role: "application".to_string(),
            name: Some(app_name),
            value: None,
            bounds: None,
            children: Vec::new(),
        });

        Ok(GroundingOutcome {
            expert: self.name().to_string(),
            signal: self.signal(),
            confidence: 0.45,
            updated: true,
        })
    }
}

pub struct VisionRoutingGroundingExpert {
    output_dir: PathBuf,
}

impl VisionRoutingGroundingExpert {
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }

    fn build_output_path(&self) -> PathBuf {
        let file_name = format!("grounding_{}.png", Uuid::new_v4());
        self.output_dir.join(file_name)
    }

    async fn capture_macos(output_path: &Path) -> anyhow::Result<()> {
        let result = tokio::time::timeout(
            Duration::from_secs(SCREENSHOT_TIMEOUT_SECS),
            Command::new("screencapture")
                .arg("-x")
                .arg(output_path)
                .output(),
        )
        .await
        .context("screencapture command timed out")??;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            anyhow::bail!("screencapture failed: {stderr}");
        }

        Ok(())
    }

    async fn try_capture_linux(output_path: &Path) -> anyhow::Result<()> {
        let candidates = [
            ("gnome-screenshot", vec!["-f".to_string()]),
            ("scrot", Vec::new()),
            ("import", vec!["-window".to_string(), "root".to_string()]),
        ];

        for (program, prefix_args) in candidates {
            let mut command = Command::new(program);
            for arg in &prefix_args {
                command.arg(arg);
            }
            command.arg(output_path);

            let result = tokio::time::timeout(
                Duration::from_secs(SCREENSHOT_TIMEOUT_SECS),
                command.output(),
            )
            .await;

            if let Ok(Ok(output)) = result {
                if output.status.success() {
                    return Ok(());
                }
            }
        }

        anyhow::bail!(
            "no Linux screenshot backend succeeded (tried gnome-screenshot, scrot, import)"
        )
    }

    async fn capture_windows(output_path: &Path) -> anyhow::Result<()> {
        let output = output_path.to_string_lossy().replace('\'', "''");
        let script = format!(
            "$ErrorActionPreference='Stop'; \
             Add-Type -AssemblyName System.Windows.Forms; \
             Add-Type -AssemblyName System.Drawing; \
             $bounds=[System.Windows.Forms.Screen]::PrimaryScreen.Bounds; \
             $bitmap=New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height; \
             $graphics=[System.Drawing.Graphics]::FromImage($bitmap); \
             $graphics.CopyFromScreen($bounds.Location,[System.Drawing.Point]::Empty,$bounds.Size); \
             $bitmap.Save('{output}', [System.Drawing.Imaging.ImageFormat]::Png); \
             $graphics.Dispose(); \
             $bitmap.Dispose();"
        );

        let result = tokio::time::timeout(
            Duration::from_secs(SCREENSHOT_TIMEOUT_SECS),
            Command::new("powershell")
                .arg("-NoProfile")
                .arg("-Command")
                .arg(script)
                .output(),
        )
        .await
        .context("PowerShell screenshot command timed out")??;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            anyhow::bail!("PowerShell screenshot failed: {stderr}");
        }

        Ok(())
    }
}

#[async_trait]
impl GroundingExpert for VisionRoutingGroundingExpert {
    fn name(&self) -> &str {
        "vision_routing_grounding_expert"
    }

    fn signal(&self) -> GroundingSignal {
        GroundingSignal::Vision
    }

    fn routing_score(&self, request: &GroundingRequest, state: &ScreenState) -> f32 {
        if !request.requests(self.signal()) || state.screenshot_path.is_some() {
            return 0.0;
        }
        if request.include_ocr {
            1.0
        } else {
            0.7
        }
    }

    async fn ground(
        &self,
        request: &GroundingRequest,
        state: &mut ScreenState,
    ) -> anyhow::Result<GroundingOutcome> {
        if !request.requests(self.signal()) {
            return Ok(GroundingOutcome {
                expert: self.name().to_string(),
                signal: self.signal(),
                confidence: 0.0,
                updated: false,
            });
        }

        tokio::fs::create_dir_all(&self.output_dir)
            .await
            .with_context(|| format!("failed to create {}", self.output_dir.display()))?;

        // Cleanup old screenshot artifacts to prevent unbounded disk usage
        if let Ok(mut entries) = tokio::fs::read_dir(&self.output_dir).await {
            let threshold = std::time::Duration::from_secs(300); // 5 minutes
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(metadata) = entry.metadata().await {
                    if let Ok(modified) = metadata.modified() {
                        if modified.elapsed().unwrap_or(std::time::Duration::from_secs(0)) > threshold {
                            let _ = tokio::fs::remove_file(entry.path()).await;
                        }
                    }
                }
            }
        }

        let screenshot_path = self.build_output_path();

        if cfg!(target_os = "macos") {
            Self::capture_macos(&screenshot_path).await?;
        } else if cfg!(target_os = "linux") {
            Self::try_capture_linux(&screenshot_path).await?;
        } else if cfg!(target_os = "windows") {
            Self::capture_windows(&screenshot_path).await?;
        } else {
            anyhow::bail!("vision grounding is unsupported on this operating system");
        }

        state.screenshot_path = Some(screenshot_path.to_string_lossy().to_string());
        Ok(GroundingOutcome {
            expert: self.name().to_string(),
            signal: self.signal(),
            confidence: 0.9,
            updated: true,
        })
    }
}

pub struct OcrGroundingExpert {
    config: OcrConfig,
}

impl OcrGroundingExpert {
    pub fn new(config: OcrConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl GroundingExpert for OcrGroundingExpert {
    fn name(&self) -> &str {
        "ocr_grounding_expert"
    }

    fn signal(&self) -> GroundingSignal {
        GroundingSignal::Ocr
    }

    fn routing_score(&self, request: &GroundingRequest, state: &ScreenState) -> f32 {
        if !request.include_ocr {
            return 0.0;
        }
        if state.screenshot_path.is_some() {
            0.85
        } else {
            0.2
        }
    }

    async fn ground(
        &self,
        request: &GroundingRequest,
        state: &mut ScreenState,
    ) -> anyhow::Result<GroundingOutcome> {
        if !request.include_ocr {
            return Ok(GroundingOutcome {
                expert: self.name().to_string(),
                signal: self.signal(),
                confidence: 0.0,
                updated: false,
            });
        }

        let screenshot_path = state
            .screenshot_path
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("ocr grounding requires a screenshot signal first"))?;
        let ocr_items = TesseractOcrProvider::extract_text_with_config(
            Path::new(screenshot_path),
            &self.config,
        )
        .await
        .context("OCR extraction failed")?;
        let average_confidence = if ocr_items.is_empty() {
            0.0
        } else {
            ocr_items.iter().map(|item| item.confidence).sum::<f64>() / ocr_items.len() as f64
        };
        state.extracted_text = ocr_items;

        Ok(GroundingOutcome {
            expert: self.name().to_string(),
            signal: self.signal(),
            confidence: (average_confidence / 100.0).clamp(0.0, 1.0),
            updated: true,
        })
    }
}

pub fn default_runtime_experts(grounding_dir: PathBuf) -> Vec<Arc<dyn GroundingExpert>> {
    vec![
        Arc::new(VisionRoutingGroundingExpert::new(grounding_dir)),
        Arc::new(WidgetTreeGroundingExpert),
        Arc::new(RuntimeContextGroundingExpert),
        Arc::new(OcrGroundingExpert::new(OcrConfig::default())),
    ]
}

#[cfg(test)]
mod tests {
    use super::{default_runtime_experts, OcrGroundingExpert, RuntimeContextGroundingExpert};
    use crate::grounding::traits::GroundingExpert;
    use crate::grounding::traits::GroundingRequest;
    use crate::perception::traits::ScreenState;
    use crate::perception::OcrConfig;
    use std::path::PathBuf;

    #[tokio::test]
    async fn ocr_expert_requires_screenshot_signal() {
        let expert = OcrGroundingExpert::new(OcrConfig::default());
        let request = GroundingRequest {
            include_vision: false,
            include_widget_tree: false,
            include_ocr: true,
        };
        let mut state = ScreenState {
            screenshot_path: None,
            widget_tree: None,
            extracted_text: Vec::new(),
        };

        let err = expert.ground(&request, &mut state).await.unwrap_err();
        assert!(err.to_string().contains("requires a screenshot"));
    }

    #[tokio::test]
    async fn runtime_context_expert_populates_widget_tree_fallback() {
        let expert = RuntimeContextGroundingExpert;
        let request = GroundingRequest::host_runtime_default();
        let mut state = ScreenState {
            screenshot_path: None,
            widget_tree: None,
            extracted_text: Vec::new(),
        };

        let outcome = expert
            .ground(&request, &mut state)
            .await
            .expect("runtime context fallback should succeed");
        assert!(outcome.updated);
        assert!(state.widget_tree.is_some());
    }

    #[test]
    fn default_runtime_experts_register_runtime_context_expert() {
        let experts = default_runtime_experts(PathBuf::from("/tmp/gloamy_grounding_test"));
        let names: Vec<String> = experts
            .iter()
            .map(|expert| expert.name().to_string())
            .collect();
        assert!(names
            .iter()
            .any(|name| name == "runtime_context_grounding_expert"));
    }
}
