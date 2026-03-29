pub mod cost_tracking;
pub mod log;
pub mod multi;
pub mod noop;
#[cfg(feature = "observability-otel")]
pub mod otel;
pub mod prometheus;
pub mod runtime_trace;
pub mod traits;
pub mod verbose;

#[allow(unused_imports)]
pub use self::log::LogObserver;
#[allow(unused_imports)]
pub use self::multi::MultiObserver;
pub use cost_tracking::CostTrackingObserver;
pub use noop::NoopObserver;
#[cfg(feature = "observability-otel")]
pub use otel::OtelObserver;
pub use prometheus::PrometheusObserver;
pub use traits::{Observer, ObserverEvent};
#[allow(unused_imports)]
pub use verbose::VerboseObserver;

use crate::config::schema::CostConfig;
use crate::config::ObservabilityConfig;
use crate::cost::CostTracker;
use std::path::Path;
use std::sync::Arc;

/// Factory: create the right observer from config
pub fn create_observer(config: &ObservabilityConfig) -> Box<dyn Observer> {
    match config.backend.as_str() {
        "log" => Box::new(LogObserver::new()),
        "prometheus" => Box::new(PrometheusObserver::new()),
        "otel" | "opentelemetry" | "otlp" => {
            #[cfg(feature = "observability-otel")]
            match OtelObserver::new(
                config.otel_endpoint.as_deref(),
                config.otel_service_name.as_deref(),
            ) {
                Ok(obs) => {
                    tracing::info!(
                        endpoint = config
                            .otel_endpoint
                            .as_deref()
                            .unwrap_or("http://localhost:4318"),
                        "OpenTelemetry observer initialized"
                    );
                    Box::new(obs)
                }
                Err(e) => {
                    tracing::error!("Failed to create OTel observer: {e}. Falling back to noop.");
                    Box::new(NoopObserver)
                }
            }
            #[cfg(not(feature = "observability-otel"))]
            {
                tracing::warn!(
                    "OpenTelemetry backend requested but this build was compiled without `observability-otel`; falling back to noop."
                );
                Box::new(NoopObserver)
            }
        }
        "none" | "noop" => Box::new(NoopObserver),
        _ => {
            tracing::warn!(
                "Unknown observability backend '{}', falling back to noop",
                config.backend
            );
            Box::new(NoopObserver)
        }
    }
}

/// Create an observer and wrap it with cost tracking when enabled.
pub fn create_observer_with_cost(
    observability_config: &ObservabilityConfig,
    cost_config: &CostConfig,
    workspace_dir: &Path,
) -> (Arc<dyn Observer>, Option<Arc<CostTracker>>) {
    let observer: Arc<dyn Observer> = Arc::from(create_observer(observability_config));
    wrap_with_cost_tracking(observer, cost_config, workspace_dir)
}

/// Wrap an existing observer with cost tracking when enabled.
pub fn wrap_with_cost_tracking(
    observer: Arc<dyn Observer>,
    cost_config: &CostConfig,
    workspace_dir: &Path,
) -> (Arc<dyn Observer>, Option<Arc<CostTracker>>) {
    if !cost_config.enabled {
        return (observer, None);
    }

    match CostTracker::new(cost_config.clone(), workspace_dir) {
        Ok(tracker) => {
            let tracker = Arc::new(tracker);
            let wrapped: Arc<dyn Observer> = Arc::new(CostTrackingObserver::new(
                observer,
                Arc::clone(&tracker),
                cost_config.prices.clone(),
            ));
            (wrapped, Some(tracker))
        }
        Err(e) => {
            tracing::warn!("Failed to initialize cost tracker: {e}");
            (observer, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::{CostConfig, ModelPricing};
    use std::collections::HashMap;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn factory_none_returns_noop() {
        let cfg = ObservabilityConfig {
            backend: "none".into(),
            ..ObservabilityConfig::default()
        };
        assert_eq!(create_observer(&cfg).name(), "noop");
    }

    #[test]
    fn factory_noop_returns_noop() {
        let cfg = ObservabilityConfig {
            backend: "noop".into(),
            ..ObservabilityConfig::default()
        };
        assert_eq!(create_observer(&cfg).name(), "noop");
    }

    #[test]
    fn factory_log_returns_log() {
        let cfg = ObservabilityConfig {
            backend: "log".into(),
            ..ObservabilityConfig::default()
        };
        assert_eq!(create_observer(&cfg).name(), "log");
    }

    #[test]
    fn factory_prometheus_returns_prometheus() {
        let cfg = ObservabilityConfig {
            backend: "prometheus".into(),
            ..ObservabilityConfig::default()
        };
        assert_eq!(create_observer(&cfg).name(), "prometheus");
    }

    #[test]
    fn factory_otel_returns_otel() {
        let cfg = ObservabilityConfig {
            backend: "otel".into(),
            otel_endpoint: Some("http://127.0.0.1:19999".into()),
            otel_service_name: Some("test".into()),
            ..ObservabilityConfig::default()
        };
        let expected = if cfg!(feature = "observability-otel") {
            "otel"
        } else {
            "noop"
        };
        assert_eq!(create_observer(&cfg).name(), expected);
    }

    #[test]
    fn factory_opentelemetry_alias() {
        let cfg = ObservabilityConfig {
            backend: "opentelemetry".into(),
            otel_endpoint: Some("http://127.0.0.1:19999".into()),
            otel_service_name: Some("test".into()),
            ..ObservabilityConfig::default()
        };
        let expected = if cfg!(feature = "observability-otel") {
            "otel"
        } else {
            "noop"
        };
        assert_eq!(create_observer(&cfg).name(), expected);
    }

    #[test]
    fn factory_otlp_alias() {
        let cfg = ObservabilityConfig {
            backend: "otlp".into(),
            otel_endpoint: Some("http://127.0.0.1:19999".into()),
            otel_service_name: Some("test".into()),
            ..ObservabilityConfig::default()
        };
        let expected = if cfg!(feature = "observability-otel") {
            "otel"
        } else {
            "noop"
        };
        assert_eq!(create_observer(&cfg).name(), expected);
    }

    #[test]
    fn factory_unknown_falls_back_to_noop() {
        let cfg = ObservabilityConfig {
            backend: "xyzzy_unknown".into(),
            ..ObservabilityConfig::default()
        };
        assert_eq!(create_observer(&cfg).name(), "noop");
    }

    #[test]
    fn factory_empty_string_falls_back_to_noop() {
        let cfg = ObservabilityConfig {
            backend: String::new(),
            ..ObservabilityConfig::default()
        };
        assert_eq!(create_observer(&cfg).name(), "noop");
    }

    #[test]
    fn factory_garbage_falls_back_to_noop() {
        let cfg = ObservabilityConfig {
            backend: "xyzzy_garbage_123".into(),
            ..ObservabilityConfig::default()
        };
        assert_eq!(create_observer(&cfg).name(), "noop");
    }

    #[test]
    fn wrap_with_cost_tracking_records_usage_when_enabled() {
        let tmp = TempDir::new().unwrap();
        let mut prices = HashMap::new();
        prices.insert(
            "openai/gpt-4".to_string(),
            ModelPricing {
                input: 1.0,
                output: 2.0,
            },
        );
        let cost = CostConfig {
            enabled: true,
            prices,
            ..CostConfig::default()
        };
        let base: Arc<dyn Observer> = Arc::new(NoopObserver);

        let (observer, tracker) = wrap_with_cost_tracking(base, &cost, tmp.path());
        let tracker = tracker.expect("cost tracker should be created when enabled");

        observer.record_event(&ObserverEvent::LlmResponse {
            provider: "openai".into(),
            model: "gpt-4".into(),
            duration: Duration::from_millis(100),
            success: true,
            error_message: None,
            input_tokens: Some(12),
            output_tokens: Some(8),
        });

        let summary = tracker.get_summary().unwrap();
        assert_eq!(summary.total_tokens, 20);
        assert_eq!(summary.request_count, 1);
    }
}
