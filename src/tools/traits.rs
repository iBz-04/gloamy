use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Result of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

impl ToolResult {
    /// Build a `ToolResult` from a [`GuiActionReport`].
    ///
    /// `success` follows `verification_status == Verified`, not mere execution
    /// acknowledgment. The report is serialized into `output` as structured JSON
    /// so the LLM and downstream consumers can inspect verification evidence.
    pub fn from_gui_report(report: &GuiActionReport) -> Self {
        let success = report.verification_status == VerificationStatus::Verified;
        let output = serde_json::to_string_pretty(report).unwrap_or_default();
        let error = if success {
            None
        } else {
            Some(format!(
                "GUI verification: {:?}",
                report.verification_status
            ))
        };
        Self {
            success,
            output,
            error,
        }
    }

    /// Return the most actionable text representation of this result.
    ///
    /// For failures, prefer the explicit error reason and keep output as
    /// supplemental context when it adds distinct information.
    pub fn diagnostic_output(&self) -> String {
        if self.success {
            return self.output.clone();
        }

        let error = self
            .error
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let output = self.output.trim();
        let output = (!output.is_empty()).then_some(output);

        match (error, output) {
            (Some(error), Some(output))
                if output != error
                    && output != format!("Error: {error}")
                    && !output.ends_with(error) =>
            {
                format!("Error: {error}\n{output}")
            }
            (Some(error), _) => format!("Error: {error}"),
            (None, Some(output)) => output.to_string(),
            (None, None) => "Error: tool returned no detail".to_string(),
        }
    }
}

// ── GUI verification contract ───────────────────────────────────

/// Whether the post-action UI state matches the expectation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Verified,
    Failed,
    Ambiguous,
}

/// Why a verification failed (for failure attribution).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureCause {
    /// Agent made a mistake (wrong selector, bad action, reasoning error).
    AgentError,
    /// Environment blocked the action (login wall, CAPTCHA, network failure, out-of-stock).
    EnvironmentBlocked,
    /// Cannot verify because evidence is missing or incomplete.
    EvidenceAbsent,
    /// Agent claimed success but evidence doesn't support it.
    HallucinationSuspected,
    /// Not applicable (expectation verified or not evaluated).
    None,
}

impl Default for FailureCause {
    fn default() -> Self {
        Self::None
    }
}

/// How to capture pre-action state for diffing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PreObservationStrategy {
    /// No pre-observation (Phase 1 behavior, default).
    #[default]
    None,
    /// Capture evidence keys inferred from the supplied expectations.
    Auto,
    /// Capture an explicit set of evidence keys.
    Explicit { keys: Vec<String> },
}

/// How to wait for the UI to settle before collecting post-observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WaitStrategy {
    /// No wait (Phase 1 behavior, default).
    #[default]
    None,
    /// Fixed delay in milliseconds.
    FixedMs { ms: u64 },
    /// Wait for a DOM event (browser-oriented backends).
    DomEvent { event: String, timeout_ms: u64 },
    /// Wait for a macOS accessibility notification.
    AccessibilityEvent {
        notification: String,
        timeout_ms: u64,
    },
    /// Wait until a selector becomes present/visible.
    SelectorPresent { selector: String, timeout_ms: u64 },
    /// Poll evidence collection until expectations verify or the timeout expires.
    PollUntilVerified {
        poll_interval_ms: u64,
        timeout_ms: u64,
    },
}

/// How reversible a GUI action is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReversibilityLevel {
    Reversible,
    PartiallyReversible,
    Irreversible,
    Unknown,
}

/// What kind of end-state the caller expects after a GUI action.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GuiExpectationKind {
    FieldValueEquals {
        selector: String,
        value: String,
    },
    FocusedElementIs {
        selector: String,
    },
    CheckboxChecked {
        selector: String,
        checked: bool,
    },
    WindowTitleContains {
        #[serde(alias = "title_contains")]
        substring: String,
    },
    DialogPresent {
        present: bool,
    },
    UrlIs {
        url: String,
    },
    UrlHostIs {
        host: String,
    },
    FileExists {
        path: String,
    },
    DownloadCompleted {
        path: String,
    },
    FrontWindowElementCountChanged {
        #[serde(default)]
        role: Option<String>,
        #[serde(default)]
        title_contains: Option<String>,
        #[serde(default)]
        description_contains: Option<String>,
        #[serde(default)]
        value_contains: Option<String>,
        #[serde(default = "default_min_increase")]
        min_increase: u32,
    },
    ElementAtCoordinate {
        x: i64,
        y: i64,
        expected_element: String,
        #[serde(default)]
        tolerance_px: u32,
    },
    /// Verify a macOS accessibility attribute on the focused or targeted element.
    AXAttributeEquals {
        /// AX attribute name (e.g. "AXRole", "AXTitle", "AXValue", "AXDescription").
        attribute: String,
        /// Expected string value of the attribute.
        value: String,
    },
}

/// A single expectation attached to a GUI action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiExpectation {
    #[serde(flatten)]
    pub kind: GuiExpectationKind,
    /// If `true`, verification failure makes the whole action fail.
    /// If `false`, a mismatch is reported as `Ambiguous` but `success` can
    /// still be `true` when `execution_ok` is `true`.
    #[serde(default = "default_required")]
    pub required: bool,
}

fn default_required() -> bool {
    true
}

fn default_min_increase() -> u32 {
    1
}

/// A point-in-time observation of GUI state used for pre/post comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiObservation {
    /// Free-form evidence captured from DOM, accessibility tree, filesystem, etc.
    pub evidence: serde_json::Value,
    /// Source of the observation (e.g. "dom", "accessibility", "filesystem", "screenshot").
    pub source: String,
}

/// Structured report returned by GUI tools when verification is requested.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiActionReport {
    /// Whether the input event was dispatched without transport error.
    pub execution_ok: bool,
    /// Observation captured *before* the action.
    pub pre_observation: Option<GuiObservation>,
    /// Observation captured *after* the action.
    pub post_observation: Option<GuiObservation>,
    /// Overall verification outcome.
    pub verification_status: VerificationStatus,
    /// Per-expectation detail (index-aligned with the input expectations).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expectation_results: Vec<ExpectationResult>,
    /// Optional diff between pre and post observations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_diff: Option<serde_json::Value>,
    /// Aggregate confidence across all expectations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    /// Process score: how well did the agent execute (0.0–1.0)?
    /// High if failures are environment blocks, low if agent errors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_score: Option<f64>,
    /// Outcome success: did the task actually complete?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome_success: Option<bool>,
    /// Why the verification failed (for failure attribution).
    #[serde(default, skip_serializing_if = "is_failure_cause_none")]
    pub failure_cause: FailureCause,
}

/// Result of evaluating a single expectation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectationResult {
    pub status: VerificationStatus,
    pub expected: serde_json::Value,
    pub actual: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Confidence score in [0.0, 1.0].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    /// Why this expectation failed (for failure attribution).
    #[serde(default, skip_serializing_if = "is_failure_cause_none")]
    pub failure_cause: FailureCause,
}

fn is_failure_cause_none(cause: &FailureCause) -> bool {
    *cause == FailureCause::None
}

/// Description of a tool for the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Core tool trait — implement for any capability
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name (used in LLM function calling)
    fn name(&self) -> &str;

    /// Human-readable description
    fn description(&self) -> &str;

    /// JSON schema for parameters
    fn parameters_schema(&self) -> serde_json::Value;

    /// Execute the tool with given arguments
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult>;

    /// Get the full spec for LLM registration
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: self.parameters_schema(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyTool;

    #[async_trait]
    impl Tool for DummyTool {
        fn name(&self) -> &str {
            "dummy_tool"
        }

        fn description(&self) -> &str {
            "A deterministic test tool"
        }

        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "value": { "type": "string" }
                }
            })
        }

        async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
            Ok(ToolResult {
                success: true,
                output: args
                    .get("value")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                error: None,
            })
        }
    }

    #[test]
    fn spec_uses_tool_metadata_and_schema() {
        let tool = DummyTool;
        let spec = tool.spec();

        assert_eq!(spec.name, "dummy_tool");
        assert_eq!(spec.description, "A deterministic test tool");
        assert_eq!(spec.parameters["type"], "object");
        assert_eq!(spec.parameters["properties"]["value"]["type"], "string");
    }

    #[tokio::test]
    async fn execute_returns_expected_output() {
        let tool = DummyTool;
        let result = tool
            .execute(serde_json::json!({ "value": "hello-tool" }))
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.output, "hello-tool");
        assert!(result.error.is_none());
    }

    #[test]
    fn tool_result_serialization_roundtrip() {
        let result = ToolResult {
            success: false,
            output: String::new(),
            error: Some("boom".into()),
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: ToolResult = serde_json::from_str(&json).unwrap();

        assert!(!parsed.success);
        assert_eq!(parsed.error.as_deref(), Some("boom"));
    }

    #[test]
    fn diagnostic_output_prefers_error_for_failures() {
        let result = ToolResult {
            success: false,
            output: String::new(),
            error: Some("Missing 'action' parameter".into()),
        };

        assert_eq!(
            result.diagnostic_output(),
            "Error: Missing 'action' parameter"
        );
    }

    #[test]
    fn diagnostic_output_keeps_distinct_output_context() {
        let result = ToolResult {
            success: false,
            output: "stdout details".into(),
            error: Some("osascript failed".into()),
        };

        assert_eq!(
            result.diagnostic_output(),
            "Error: osascript failed\nstdout details"
        );
    }

    #[test]
    fn diagnostic_output_avoids_duplicate_error_text() {
        let result = ToolResult {
            success: false,
            output: "Error: osascript failed".into(),
            error: Some("osascript failed".into()),
        };

        assert_eq!(result.diagnostic_output(), "Error: osascript failed");
    }
}
