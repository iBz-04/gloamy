use super::traits::{Observer, ObserverEvent, ObserverMetric};
use crate::config::schema::ModelPricing;
use crate::cost::CostTracker;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

fn matches_model_key(configured_key: &str, provider: &str, model: &str) -> bool {
    let configured_lower = configured_key.trim().to_ascii_lowercase();
    let provider_lower = provider.trim().to_ascii_lowercase();
    let model_lower = model.trim().to_ascii_lowercase();

    if configured_lower.is_empty() || model_lower.is_empty() {
        return false;
    }

    if configured_lower == model_lower || model_lower.starts_with(&(configured_lower.clone() + "-")) {
        return true;
    }

    let configured_tail = configured_lower
        .split('/')
        .next_back()
        .unwrap_or(configured_lower.as_str());

    if configured_tail == model_lower || model_lower.starts_with(&(configured_tail.to_string() + "-")) {
        return true;
    }

    if provider_lower.is_empty() {
        return false;
    }

    let provider_model = format!("{provider_lower}/{model_lower}");
    configured_lower == provider_model || provider_model.starts_with(&(configured_lower + "-"))
}

/// Observer wrapper that intercepts `LlmResponse` events and records token
/// usage to a [`CostTracker`], then delegates all events to an inner observer.
pub struct CostTrackingObserver {
    inner: Arc<dyn Observer>,
    cost_tracker: Arc<CostTracker>,
    prices: HashMap<String, ModelPricing>,
}

impl CostTrackingObserver {
    pub fn new(
        inner: Arc<dyn Observer>,
        cost_tracker: Arc<CostTracker>,
        prices: HashMap<String, ModelPricing>,
    ) -> Self {
        Self {
            inner,
            cost_tracker,
            prices,
        }
    }

    fn model_prices(&self, provider: &str, model: &str) -> (f64, f64) {
        let model_trimmed = model.trim();
        let provider_trimmed = provider.trim();

        if model_trimmed.is_empty() {
            return (0.0, 0.0);
        }

        let direct_keys = [
            model_trimmed.to_string(),
            model_trimmed.to_ascii_lowercase(),
            format!("{provider_trimmed}/{model_trimmed}"),
            format!(
                "{}/{}",
                provider_trimmed.to_ascii_lowercase(),
                model_trimmed.to_ascii_lowercase()
            ),
        ];

        for key in direct_keys {
            if let Some(pricing) = self.prices.get(&key) {
                return (pricing.input, pricing.output);
            }
        }

        let model_lower = model_trimmed.to_ascii_lowercase();
        for (configured_key, pricing) in &self.prices {
            if matches_model_key(configured_key, provider_trimmed, &model_lower) {
                return (pricing.input, pricing.output);
            }
        }

        (0.0, 0.0)
    }
}

impl Observer for CostTrackingObserver {
    fn record_event(&self, event: &ObserverEvent) {
        if let ObserverEvent::LlmResponse {
            provider,
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
                    let (input_price, output_price) = self.model_prices(provider, model);
                    let usage = crate::cost::TokenUsage::new(
                        model,
                        input,
                        output,
                        input_price,
                        output_price,
                    );
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

    fn observer_with_prices(
        tracker: Arc<CostTracker>,
        prices: HashMap<String, ModelPricing>,
    ) -> CostTrackingObserver {
        let inner: Arc<dyn Observer> = Arc::new(NoopObserver);
        CostTrackingObserver::new(inner, tracker, prices)
    }

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
        let mut prices = HashMap::new();
        prices.insert(
            "openai/gpt-4".to_string(),
            ModelPricing {
                input: 2.0,
                output: 4.0,
            },
        );
        let obs = observer_with_prices(Arc::clone(&tracker), prices);

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
        assert!((summary.session_cost_usd - 0.0004).abs() < 1e-10);
    }

    #[test]
    fn skips_failed_llm_responses() {
        let (_tmp, tracker) = test_tracker();
        let obs = observer_with_prices(Arc::clone(&tracker), HashMap::new());

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
        let obs = observer_with_prices(Arc::clone(&tracker), HashMap::new());

        // Should not panic on non-LLM events
        obs.record_event(&ObserverEvent::HeartbeatTick);
        obs.record_metric(&ObserverMetric::TokensUsed(42));
        obs.flush();
        assert_eq!(obs.name(), "cost-tracking");
    }

    #[test]
    fn falls_back_to_zero_price_when_model_is_unmapped() {
        let (_tmp, tracker) = test_tracker();
        let obs = observer_with_prices(Arc::clone(&tracker), HashMap::new());

        obs.record_event(&ObserverEvent::LlmResponse {
            provider: "openai".into(),
            model: "gpt-4.1-mini".into(),
            duration: Duration::from_millis(300),
            success: true,
            error_message: None,
            input_tokens: Some(100),
            output_tokens: Some(50),
        });

        let summary = tracker.get_summary().unwrap();
        assert_eq!(summary.total_tokens, 150);
        assert_eq!(summary.request_count, 1);
        assert!(summary.session_cost_usd.abs() < f64::EPSILON);
    }

    #[test]
    fn matches_snapshot_suffix_to_base_model_price() {
        let (_tmp, tracker) = test_tracker();
        let mut prices = HashMap::new();
        prices.insert(
            "openai/gpt-5.4-mini".to_string(),
            ModelPricing {
                input: 0.75,
                output: 4.5,
            },
        );
        let obs = observer_with_prices(Arc::clone(&tracker), prices);

        obs.record_event(&ObserverEvent::LlmResponse {
            provider: "openai".into(),
            model: "gpt-5.4-mini-2026-03-17".into(),
            duration: Duration::from_millis(300),
            success: true,
            error_message: None,
            input_tokens: Some(100),
            output_tokens: Some(50),
        });

        let summary = tracker.get_summary().unwrap();
        assert_eq!(summary.total_tokens, 150);
        assert_eq!(summary.request_count, 1);
        assert!((summary.session_cost_usd - 0.0003).abs() < 1e-10);
    }

    #[test]
    fn matches_provider_prefixed_snapshot_suffix_to_base_model_price() {
        let (_tmp, tracker) = test_tracker();
        let mut prices = HashMap::new();
        prices.insert(
            "openai/gpt-5.4-mini".to_string(),
            ModelPricing {
                input: 0.75,
                output: 4.5,
            },
        );
        let obs = observer_with_prices(Arc::clone(&tracker), prices);

        obs.record_event(&ObserverEvent::LlmResponse {
            provider: "openai".into(),
            model: "openai/gpt-5.4-mini-2026-03-17".into(),
            duration: Duration::from_millis(300),
            success: true,
            error_message: None,
            input_tokens: Some(100),
            output_tokens: Some(50),
        });

        let summary = tracker.get_summary().unwrap();
        assert_eq!(summary.total_tokens, 150);
        assert_eq!(summary.request_count, 1);
        assert!((summary.session_cost_usd - 0.0003).abs() < 1e-10);
    }
}
