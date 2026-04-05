use crate::perception::traits::{OcrTextItem, ScreenState, WidgetNode};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroundingSignal {
    Vision,
    WidgetTree,
    Ocr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingRequest {
    pub include_vision: bool,
    pub include_widget_tree: bool,
    pub include_ocr: bool,
}

impl GroundingRequest {
    pub fn host_runtime_default() -> Self {
        Self {
            include_vision: true,
            include_widget_tree: true,
            include_ocr: false,
        }
    }

    pub fn requests(&self, signal: GroundingSignal) -> bool {
        match signal {
            GroundingSignal::Vision => self.include_vision || self.include_ocr,
            GroundingSignal::WidgetTree => self.include_widget_tree,
            GroundingSignal::Ocr => self.include_ocr,
        }
    }
}

impl Default for GroundingRequest {
    fn default() -> Self {
        Self::host_runtime_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingOutcome {
    pub expert: String,
    pub signal: GroundingSignal,
    pub confidence: f64,
    pub updated: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GroundingDiagnostics {
    pub requested_signals: Vec<GroundingSignal>,
    pub selected_experts: Vec<String>,
    pub completed: Vec<GroundingOutcome>,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundingResult {
    pub state: ScreenState,
    pub diagnostics: GroundingDiagnostics,
}

impl GroundingResult {
    pub fn empty() -> Self {
        Self {
            state: ScreenState {
                screenshot_path: None,
                widget_tree: None,
                extracted_text: Vec::new(),
            },
            diagnostics: GroundingDiagnostics::default(),
        }
    }
}

#[async_trait]
pub trait GroundingExpert: Send + Sync {
    fn name(&self) -> &str;
    fn signal(&self) -> GroundingSignal;

    /// Return a score in [0.0, 1.0] that indicates how strongly this expert
    /// should be selected for the current request and partially grounded state.
    fn routing_score(&self, request: &GroundingRequest, state: &ScreenState) -> f32;

    /// Enrich the shared screen state with this expert's grounded signal.
    async fn ground(
        &self,
        request: &GroundingRequest,
        state: &mut ScreenState,
    ) -> anyhow::Result<GroundingOutcome>;
}

pub(crate) fn state_has_signal(state: &ScreenState, signal: GroundingSignal) -> bool {
    match signal {
        GroundingSignal::Vision => state
            .screenshot_path
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty()),
        GroundingSignal::WidgetTree => state.widget_tree.is_some(),
        GroundingSignal::Ocr => !state.extracted_text.is_empty(),
    }
}

pub(crate) fn signal_payload_size(
    screenshot_path: Option<&str>,
    widget_tree: Option<&WidgetNode>,
    ocr_items: &[OcrTextItem],
) -> usize {
    let screenshot_count = usize::from(screenshot_path.is_some());
    let widget_count = usize::from(widget_tree.is_some());
    screenshot_count + widget_count + ocr_items.len()
}
