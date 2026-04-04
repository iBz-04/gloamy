use crate::perception::traits::ScreenState;
use crate::tools::traits::ToolResult;
use async_trait::async_trait;

/// Represents an AppAgent (Worker) specialized for a specific application or tactical execution.
/// Follows the AgentOS/Compositional multi-agent paradigm for cognitive separation.
#[async_trait]
pub trait AppWorker: Send + Sync {
    /// Returns the name of the worker (e.g., "browser_agent", "terminal_agent").
    fn name(&self) -> &str;

    /// Checks if this worker is equipped to handle the current application context.
    fn can_handle(&self, application_context: &str) -> bool;

    /// Executes a delegated tactical step based on the fused screen state and instructions.
    async fn execute_step(
        &self,
        task_instruction: &str,
        current_state: &ScreenState,
    ) -> anyhow::Result<ToolResult>;
}
