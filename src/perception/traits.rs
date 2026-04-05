use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Represents a bounding box on the screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenBounds {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

/// A node in the UI accessibility or DOM tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WidgetNode {
    pub id: String,
    pub role: String,
    pub name: Option<String>,
    pub value: Option<String>,
    pub bounds: Option<ScreenBounds>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<WidgetNode>,
}

/// A localized text element extracted via OCR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrTextItem {
    pub text: String,
    pub bounds: ScreenBounds,
    pub confidence: f64,
}

/// Represents the fused state of the current environment (GUI screen).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenState {
    /// Path to the captured screenshot image
    pub screenshot_path: Option<String>,
    /// Structured accessibility/DOM tree
    pub widget_tree: Option<WidgetNode>,
    /// Any text dynamically extracted via OCR
    pub extracted_text: Vec<OcrTextItem>,
}

/// Core trait for a perception sub-system that extracts state from the OS or application.
#[async_trait]
pub trait PerceptionProvider: Send + Sync {
    /// The canonical name of the provider (e.g., "macos_a11y", "tesseract_ocr")
    fn name(&self) -> &str;

    /// Captures the current state of the screen. Providers can populate the fields they support.
    async fn capture_state(&self) -> anyhow::Result<ScreenState>;
}
