//! Lightweight tool contracts used by `gloamy-robot`.
//!
//! The trait shape intentionally mirrors the main Gloamy tool surface, but this
//! crate keeps its own contract so it can remain usable without a direct
//! dependency on the root runtime.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Standard result returned by robot tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the tool executed successfully
    pub success: bool,
    /// Output from the tool (human-readable)
    pub output: String,
    /// Error message if failed
    pub error: Option<String>,
}

impl ToolResult {
    /// Create a successful result
    pub fn success(output: impl Into<String>) -> Self {
        Self {
            success: true,
            output: output.into(),
            error: None,
        }
    }

    /// Create a failed result
    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            output: String::new(),
            error: Some(error.into()),
        }
    }

    /// Create a failed result with partial output
    pub fn partial(output: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            success: false,
            output: output.into(),
            error: Some(error.into()),
        }
    }
}

/// Serializable tool registration payload for LLM/tool-call adapters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    /// Tool name (used in function calls)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// JSON Schema for parameters
    pub parameters: Value,
}

/// Core tool trait for robot capabilities.
///
/// Implement this trait to create a new tool that can be used
/// by an AI agent to interact with the robot hardware.
///
/// # Example
///
/// ```rust,ignore
/// use gloamy_robot::{Tool, ToolResult};
/// use async_trait::async_trait;
/// use serde_json::{json, Value};
///
/// pub struct BeepTool;
///
/// #[async_trait]
/// impl Tool for BeepTool {
///     fn name(&self) -> &str { "beep" }
///
///     fn description(&self) -> &str { "Make a beep sound" }
///
///     fn parameters_schema(&self) -> Value {
///         json!({
///             "type": "object",
///             "properties": {
///                 "frequency": { "type": "number", "description": "Hz" }
///             }
///         })
///     }
///
///     async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
///         let freq = args["frequency"].as_f64().unwrap_or(440.0);
///         // Play beep...
///         Ok(ToolResult::success(format!("Beeped at {}Hz", freq)))
///     }
/// }
/// ```
#[async_trait]
pub trait Tool: Send + Sync {
    /// Stable tool name used by the caller.
    fn name(&self) -> &str;

    /// Human-readable summary of what the tool does.
    fn description(&self) -> &str;

    /// JSON Schema describing the tool's accepted arguments.
    fn parameters_schema(&self) -> Value;

    /// Execute the tool with JSON arguments that match [`Self::parameters_schema`].
    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult>;

    /// Build the full tool registration payload.
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: self.name().to_string(),
            description: self.description().to_string(),
            parameters: self.parameters_schema(),
        }
    }
}
