use crate::grounding::traits::{
    state_has_signal, GroundingDiagnostics, GroundingExpert, GroundingRequest, GroundingResult,
};
use crate::perception::traits::ScreenState;
use std::sync::Arc;

/// A MoG-style router that selects and executes grounding experts by score.
pub struct MoGRouter {
    experts: Vec<Arc<dyn GroundingExpert>>,
}

impl MoGRouter {
    pub fn new(experts: Vec<Arc<dyn GroundingExpert>>) -> Self {
        Self { experts }
    }

    pub fn experts(&self) -> &[Arc<dyn GroundingExpert>] {
        &self.experts
    }

    pub async fn ground(&self, request: &GroundingRequest) -> anyhow::Result<GroundingResult> {
        let mut state = ScreenState {
            screenshot_path: None,
            widget_tree: None,
            extracted_text: Vec::new(),
        };
        self.ground_with_state(request, &mut state).await
    }

    pub async fn ground_with_state(
        &self,
        request: &GroundingRequest,
        state: &mut ScreenState,
    ) -> anyhow::Result<GroundingResult> {
        let mut diagnostics = GroundingDiagnostics {
            requested_signals: [
                crate::grounding::traits::GroundingSignal::Vision,
                crate::grounding::traits::GroundingSignal::WidgetTree,
                crate::grounding::traits::GroundingSignal::Ocr,
            ]
            .into_iter()
            .filter(|signal| request.requests(*signal))
            .collect(),
            ..GroundingDiagnostics::default()
        };

        let mut ranked = self
            .experts
            .iter()
            .map(|expert| {
                (
                    expert.routing_score(request, state),
                    Arc::clone(expert),
                    expert.name().to_string(),
                )
            })
            .filter(|(score, _, _)| *score > 0.0)
            .collect::<Vec<_>>();

        ranked.sort_by(|left, right| right.0.total_cmp(&left.0));

        for (_, expert, expert_name) in ranked {
            diagnostics.selected_experts.push(expert_name.clone());
            match expert.ground(request, state).await {
                Ok(outcome) => {
                    if outcome.updated || state_has_signal(state, outcome.signal) {
                        diagnostics.completed.push(outcome);
                    }
                }
                Err(error) => diagnostics.failures.push(format!("{expert_name}: {error}")),
            }
        }

        Ok(GroundingResult {
            state: state.clone(),
            diagnostics,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::MoGRouter;
    use crate::grounding::traits::{
        GroundingExpert, GroundingOutcome, GroundingRequest, GroundingSignal,
    };
    use crate::perception::traits::ScreenState;
    use async_trait::async_trait;
    use std::sync::Arc;

    struct TestExpert {
        name: &'static str,
        signal: GroundingSignal,
        score: f32,
        set_vision: bool,
    }

    #[async_trait]
    impl GroundingExpert for TestExpert {
        fn name(&self) -> &str {
            self.name
        }

        fn signal(&self) -> GroundingSignal {
            self.signal
        }

        fn routing_score(&self, _request: &GroundingRequest, _state: &ScreenState) -> f32 {
            self.score
        }

        async fn ground(
            &self,
            _request: &GroundingRequest,
            state: &mut ScreenState,
        ) -> anyhow::Result<GroundingOutcome> {
            if self.set_vision {
                state.screenshot_path = Some("/tmp/test.png".to_string());
            }
            Ok(GroundingOutcome {
                expert: self.name.to_string(),
                signal: self.signal,
                confidence: 0.9,
                updated: true,
            })
        }
    }

    #[tokio::test]
    async fn router_selects_experts_in_descending_score_order() {
        let router = MoGRouter::new(vec![
            Arc::new(TestExpert {
                name: "low",
                signal: GroundingSignal::Vision,
                score: 0.1,
                set_vision: false,
            }),
            Arc::new(TestExpert {
                name: "high",
                signal: GroundingSignal::WidgetTree,
                score: 0.9,
                set_vision: false,
            }),
            Arc::new(TestExpert {
                name: "mid",
                signal: GroundingSignal::Ocr,
                score: 0.5,
                set_vision: false,
            }),
        ]);

        let grounded = router.ground(&GroundingRequest::default()).await.unwrap();
        assert_eq!(
            grounded.diagnostics.selected_experts,
            vec!["high".to_string(), "mid".to_string(), "low".to_string()]
        );
        assert_eq!(grounded.diagnostics.completed.len(), 3);
    }

    #[tokio::test]
    async fn router_merges_state_updates_from_experts() {
        let router = MoGRouter::new(vec![Arc::new(TestExpert {
            name: "vision",
            signal: GroundingSignal::Vision,
            score: 1.0,
            set_vision: true,
        })]);

        let grounded = router.ground(&GroundingRequest::default()).await.unwrap();
        assert_eq!(
            grounded.state.screenshot_path.as_deref(),
            Some("/tmp/test.png")
        );
    }
}
