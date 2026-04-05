use crate::grounding::experts::default_runtime_experts;
use crate::grounding::router::MoGRouter;
use crate::grounding::traits::{GroundingRequest, GroundingResult};
use crate::perception::traits::{PerceptionProvider, ScreenState};
use async_trait::async_trait;
use std::path::PathBuf;

/// Runtime perception provider backed by a MoG-style grounding subsystem.
pub struct GroundedRuntimePerceptionProvider {
    request: GroundingRequest,
    router: MoGRouter,
}

impl GroundedRuntimePerceptionProvider {
    pub fn new(request: GroundingRequest, grounding_dir: PathBuf) -> Self {
        Self {
            request,
            router: MoGRouter::new(default_runtime_experts(grounding_dir)),
        }
    }

    pub fn for_host_runtime() -> Self {
        let grounding_dir = std::env::temp_dir().join("gloamy_grounding");
        Self::new(GroundingRequest::host_runtime_default(), grounding_dir)
    }
}

impl Default for GroundedRuntimePerceptionProvider {
    fn default() -> Self {
        Self::for_host_runtime()
    }
}

fn validate_grounding_result(
    request: &GroundingRequest,
    grounded: &GroundingResult,
) -> anyhow::Result<()> {
    let state = &grounded.state;
    let has_any_signal = state
        .screenshot_path
        .as_deref()
        .is_some_and(|path| !path.trim().is_empty())
        || state.widget_tree.is_some()
        || !state.extracted_text.is_empty();

    if !has_any_signal {
        let failure_detail = if grounded.diagnostics.failures.is_empty() {
            "no grounding expert produced usable runtime signals".to_string()
        } else {
            grounded.diagnostics.failures.join(" | ")
        };
        anyhow::bail!("HostAgent runtime perception failed: {failure_detail}");
    }

    if request.include_widget_tree && grounded.state.widget_tree.is_none() {
        let failure_detail = if grounded.diagnostics.failures.is_empty() {
            "widget_tree signal unavailable".to_string()
        } else {
            grounded.diagnostics.failures.join(" | ")
        };
        anyhow::bail!("HostAgent runtime perception failed: {failure_detail}");
    }

    Ok(())
}

#[async_trait]
impl PerceptionProvider for GroundedRuntimePerceptionProvider {
    fn name(&self) -> &str {
        "grounded_runtime_perception"
    }

    async fn capture_state(&self) -> anyhow::Result<ScreenState> {
        let grounded = self.router.ground(&self.request).await?;
        if !grounded.diagnostics.failures.is_empty() {
            tracing::debug!(
                failures = ?grounded.diagnostics.failures,
                "grounding completed with partial failures"
            );
        }
        validate_grounding_result(&self.request, &grounded)?;
        Ok(grounded.state)
    }
}

#[cfg(test)]
mod tests {
    use super::validate_grounding_result;
    use super::GroundedRuntimePerceptionProvider;
    use crate::grounding::traits::{GroundingDiagnostics, GroundingRequest, GroundingResult};
    use crate::perception::traits::PerceptionProvider;
    use crate::perception::traits::ScreenState;

    #[tokio::test]
    async fn grounded_runtime_provider_has_stable_name() {
        let provider = GroundedRuntimePerceptionProvider::default();
        assert_eq!(provider.name(), "grounded_runtime_perception");
    }

    #[test]
    fn validate_grounding_result_rejects_empty_state() {
        let request = GroundingRequest::host_runtime_default();
        let grounded = GroundingResult {
            state: ScreenState {
                screenshot_path: None,
                widget_tree: None,
                extracted_text: Vec::new(),
            },
            diagnostics: GroundingDiagnostics {
                failures: vec!["vision: denied".to_string()],
                ..GroundingDiagnostics::default()
            },
        };

        let err = validate_grounding_result(&request, &grounded).unwrap_err();
        assert!(err
            .to_string()
            .contains("HostAgent runtime perception failed"));
    }
}
