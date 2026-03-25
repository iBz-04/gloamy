use super::traits::{Observer, ObserverEvent, ObserverMetric};
use crate::cost::CostTracker;
use std::any::Any;
use std::sync::Arc;

/// Observer wrapper that intercepts `LlmResponse` events and records token
/// usage to a [`CostTracker`], then delegates all events to an inner observer.
pub struct CostTrackingObserver {
    inner: Arc<dyn Observer>,
    cost_tracker: Arc<CostTracker>,
}

impl CostTrackingObserver {
    pub fn new(inner: Arc<dyn Observer>, cost_tracker: Arc<CostTracker>) -> Self {
        Self {
            inner,
            cost_tracker,
        }
    }
}

impl Observer for CostTrackingObserver {
    fn record_event(&self, event: &ObserverEvent) {
        if let ObserverEvent::LlmResponse {
            model,
            input_tokens,
            output_tokens,
            success,
            ..
        } = event
        {
            if *success {
                let input = input_tokens.unwrap_or(0);
                let output = output_tokens.unwrap_or(0);
                if input > 0 || output > 0 {
                    // Use zero pricing when exact per-model pricing is unknown;
                    // token counts are still tracked accurately for the dashboard.
                    let usage = crate::cost::TokenUsage::new(model, input, output, 0.0, 0.0);
                    if let Err(e) = self.cost_tracker.record_usage(usage) {
                        tracing::warn!("Failed to record cost usage: {e}");
                    }
                }
            }
        }
        self.inner.record_event(event);
    }

    fn record_metric(&self, metric: &ObserverMetric) {
        self.inner.record_metric(metric);
    }

    fn flush(&self) {
        self.inner.flush();
    }

    fn name(&self) -> &str {
        "cost-tracking"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::NoopObserver;
    use std::time::Duration;
    use tempfile::TempDir;

    fn test_tracker() -> (TempDir, Arc<CostTracker>) {
        let tmp = TempDir::new().unwrap();
        let config = crate::config::schema::CostConfig {
            enabled: true,
            ..Default::default()
        };
        let tracker = CostTracker::new(config, tmp.path()).unwrap();
        (tmp, Arc::new(tracker))
    }

    #[test]
    fn records_usage_on_successful_llm_response() {
        let (_tmp, tracker) = test_tracker();
        let inner: Arc<dyn Observer> = Arc::new(NoopObserver);
        let obs = CostTrackingObserver::new(inner, Arc::clone(&tracker));

        obs.record_event(&ObserverEvent::LlmResponse {
            provider: "openai".into(),
            model: "gpt-4".into(),
            duration: Duration::from_millis(500),
            success: true,
            error_message: None,
            input_tokens: Some(100),
            output_tokens: Some(50),
        });

        let summary = tracker.get_summary().unwrap();
        assert_eq!(summary.total_tokens, 150);
        assert_eq!(summary.request_count, 1);
    }

    #[test]
    fn skips_failed_llm_responses() {
        let (_tmp, tracker) = test_tracker();
        let inner: Arc<dyn Observer> = Arc::new(NoopObserver);
        let obs = CostTrackingObserver::new(inner, Arc::clone(&tracker));

        obs.record_event(&ObserverEvent::LlmResponse {
            provider: "openai".into(),
            model: "gpt-4".into(),
            duration: Duration::from_millis(100),
            success: false,
            error_message: Some("rate limited".into()),
            input_tokens: Some(100),
            output_tokens: None,
        });

        let summary = tracker.get_summary().unwrap();
        assert_eq!(summary.total_tokens, 0);
        assert_eq!(summary.request_count, 0);
    }

    #[test]
    fn delegates_all_events_to_inner() {
        let (_tmp, tracker) = test_tracker();
        let inner: Arc<dyn Observer> = Arc::new(NoopObserver);
        let obs = CostTrackingObserver::new(inner, Arc::clone(&tracker));

        // Should not panic on non-LLM events
        obs.record_event(&ObserverEvent::HeartbeatTick);
        obs.record_metric(&ObserverMetric::TokensUsed(42));
        obs.flush();
        assert_eq!(obs.name(), "cost-tracking");
    }
}
