//! GUI verification layer for browser and mac_automation tools.
//!
//! This module implements the `precheck -> execute -> observe -> verify` contract
//! for mutating GUI actions. It evaluates [`GuiExpectation`] templates against
//! post-action observations and produces a [`GuiActionReport`] that determines
//! whether an action truly succeeded (expected state verified) rather than just
//! whether the input event was dispatched.
//!
//! Verification prefers structured signals over screenshots:
//! 1. Browser DOM / WebDriver state
//! 2. OS accessibility tree
//! 3. Filesystem or app side effects
//! 4. Screenshot/OCR only as fallback (Phase 2)

use super::traits::{
    ExpectationResult, FailureCause, GuiActionReport, GuiExpectation, GuiExpectationKind,
    GuiObservation, PreObservationStrategy, ReversibilityLevel, VerificationStatus, WaitStrategy,
};
use crate::config::{GuiApprovalGate, GuiApprovalThreshold, GuiVerificationConfig};
use crate::security::AutonomyLevel;
use serde_json::{json, Value};
use std::future::Future;
use std::time::{Duration, Instant};

/// Parse the `expect` parameter from tool arguments into typed expectations.
///
/// Accepts either a single expectation object or an array of them.
/// Returns `None` when the key is absent (raw/unverified mode).
pub fn parse_expectations(args: &Value) -> anyhow::Result<Option<Vec<GuiExpectation>>> {
    let expect = match args.get("expect") {
        Some(v) if !v.is_null() => v,
        _ => return Ok(None),
    };

    if let Some(arr) = expect.as_array() {
        if arr.is_empty() {
            return Ok(None);
        }
        let parsed: Vec<GuiExpectation> = serde_json::from_value(Value::Array(arr.clone()))
            .map_err(|e| anyhow::anyhow!("Invalid 'expect' array: {e}"))?;
        Ok(Some(parsed))
    } else if expect.is_object() {
        let single: GuiExpectation = serde_json::from_value(expect.clone())
            .map_err(|e| anyhow::anyhow!("Invalid 'expect' object: {e}"))?;
        Ok(Some(vec![single]))
    } else {
        anyhow::bail!("'expect' must be an object or array of expectation objects")
    }
}

/// Parse the optional `pre_observe` tool argument.
pub fn parse_pre_observation_strategy(args: &Value) -> anyhow::Result<PreObservationStrategy> {
    let Some(raw) = args.get("pre_observe") else {
        return Ok(PreObservationStrategy::None);
    };

    match raw {
        Value::Null => Ok(PreObservationStrategy::None),
        Value::String(mode) => match mode.trim().to_ascii_lowercase().as_str() {
            "" | "none" => Ok(PreObservationStrategy::None),
            "auto" => Ok(PreObservationStrategy::Auto),
            other => anyhow::bail!(
                "Unsupported 'pre_observe' value '{other}'. Use 'none', 'auto', or an object with 'keys'"
            ),
        },
        Value::Object(map) => {
            if let Some(strategy) = map.get("strategy").and_then(Value::as_str) {
                match strategy.trim().to_ascii_lowercase().as_str() {
                    "none" => return Ok(PreObservationStrategy::None),
                    "auto" => return Ok(PreObservationStrategy::Auto),
                    "explicit" => {}
                    other => anyhow::bail!(
                        "Unsupported 'pre_observe.strategy' value '{other}'. Use none/auto/explicit"
                    ),
                }
            }

            let keys = map
                .get("keys")
                .and_then(Value::as_array)
                .ok_or_else(|| anyhow::anyhow!("'pre_observe' object must include a string array 'keys'"))?
                .iter()
                .enumerate()
                .map(|(idx, value)| {
                    value
                        .as_str()
                        .map(str::trim)
                        .filter(|v| !v.is_empty())
                        .map(ToOwned::to_owned)
                        .ok_or_else(|| anyhow::anyhow!("pre_observe.keys[{idx}] must be a non-empty string"))
                })
                .collect::<anyhow::Result<Vec<_>>>()?;

            Ok(PreObservationStrategy::Explicit { keys })
        }
        _ => anyhow::bail!("'pre_observe' must be a string or object"),
    }
}

/// Parse the optional `wait` tool argument.
pub fn parse_wait_strategy(args: &Value) -> anyhow::Result<WaitStrategy> {
    let Some(raw) = args.get("wait") else {
        return Ok(WaitStrategy::None);
    };

    match raw {
        Value::Null => Ok(WaitStrategy::None),
        Value::String(mode) => match mode.trim().to_ascii_lowercase().as_str() {
            "" | "none" => Ok(WaitStrategy::None),
            other => anyhow::bail!(
                "Unsupported 'wait' shorthand '{other}'. Use an object with a 'strategy' field"
            ),
        },
        Value::Object(map) => {
            let strategy = map
                .get("strategy")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .unwrap_or("none")
                .to_ascii_lowercase();

            match strategy.as_str() {
                "none" => Ok(WaitStrategy::None),
                "fixed_ms" => Ok(WaitStrategy::FixedMs {
                    ms: required_u64_field(map, "ms")?,
                }),
                "dom_event" => Ok(WaitStrategy::DomEvent {
                    event: required_string_field(map, "event")?,
                    timeout_ms: required_u64_field(map, "timeout_ms")?,
                }),
                "accessibility_event" => Ok(WaitStrategy::AccessibilityEvent {
                    notification: required_string_field(map, "notification")?,
                    timeout_ms: required_u64_field(map, "timeout_ms")?,
                }),
                "selector_present" => Ok(WaitStrategy::SelectorPresent {
                    selector: required_string_field(map, "selector")?,
                    timeout_ms: required_u64_field(map, "timeout_ms")?,
                }),
                "poll_until_verified" => Ok(WaitStrategy::PollUntilVerified {
                    poll_interval_ms: required_u64_field(map, "poll_interval_ms")?,
                    timeout_ms: required_u64_field(map, "timeout_ms")?,
                }),
                other => anyhow::bail!("Unsupported wait.strategy '{other}'"),
            }
        }
        _ => anyhow::bail!("'wait' must be an object"),
    }
}

/// Parse a caller-provided reversibility override.
pub fn parse_reversibility_override(args: &Value) -> anyhow::Result<Option<ReversibilityLevel>> {
    let Some(raw) = args.get("reversibility") else {
        return Ok(None);
    };

    match raw {
        Value::Null => Ok(None),
        Value::String(level) => {
            let parsed = match level.trim().to_ascii_lowercase().as_str() {
                "reversible" => ReversibilityLevel::Reversible,
                "partially_reversible" | "partially-reversible" => {
                    ReversibilityLevel::PartiallyReversible
                }
                "irreversible" => ReversibilityLevel::Irreversible,
                "unknown" => ReversibilityLevel::Unknown,
                other => anyhow::bail!(
                    "Unsupported 'reversibility' value '{other}'. Use reversible, partially_reversible, irreversible, or unknown"
                ),
            };
            Ok(Some(parsed))
        }
        _ => anyhow::bail!("'reversibility' must be a string"),
    }
}

/// Evaluate a set of expectations against post-action evidence.
///
/// `post_evidence` is a free-form JSON blob whose shape depends on the backend.
/// Each expectation kind knows which keys to look for.
pub fn verify_expectations(
    expectations: &[GuiExpectation],
    post_evidence: &Value,
) -> (VerificationStatus, Vec<ExpectationResult>) {
    verify_expectations_with_context(expectations, None, post_evidence)
}

/// Evaluate expectations with optional access to pre-action evidence.
pub fn verify_expectations_with_context(
    expectations: &[GuiExpectation],
    pre_evidence: Option<&Value>,
    post_evidence: &Value,
) -> (VerificationStatus, Vec<ExpectationResult>) {
    let mut results = Vec::with_capacity(expectations.len());

    for exp in expectations {
        let result = verify_single(&exp.kind, pre_evidence, post_evidence);
        results.push(result);
    }

    let overall = summarize_verification(expectations, &results);
    (overall, results)
}

/// Build a complete [`GuiActionReport`] after execution.
pub fn build_report(
    execution_ok: bool,
    pre_observation: Option<GuiObservation>,
    post_observation: Option<GuiObservation>,
    expectations: &[GuiExpectation],
    post_evidence: &Value,
) -> GuiActionReport {
    let expectation_results = if execution_ok {
        verify_expectations_with_context(
            expectations,
            pre_observation.as_ref().map(|obs| &obs.evidence),
            post_evidence,
        )
        .1
    } else {
        Vec::new()
    };

    build_report_from_results(
        execution_ok,
        pre_observation,
        post_observation,
        expectation_results,
        expectations,
    )
}

/// Build a report directly from precomputed expectation results.
pub fn build_report_from_results(
    execution_ok: bool,
    pre_observation: Option<GuiObservation>,
    post_observation: Option<GuiObservation>,
    expectation_results: Vec<ExpectationResult>,
    expectations: &[GuiExpectation],
) -> GuiActionReport {
    let verification_status = if !execution_ok && expectation_results.is_empty() {
        VerificationStatus::Failed
    } else {
        summarize_verification(expectations, &expectation_results)
    };

    let state_diff = match (&pre_observation, &post_observation) {
        (Some(pre), Some(post)) => compute_diff(pre, post),
        _ => None,
    };

    let (process_score, outcome_success, failure_cause) =
        compute_process_outcome_scores(execution_ok, &expectation_results, &verification_status);

    GuiActionReport {
        execution_ok,
        pre_observation,
        post_observation,
        verification_status,
        confidence: aggregate_confidence(&expectation_results),
        state_diff,
        expectation_results,
        process_score,
        outcome_success,
        failure_cause,
    }
}

/// Compute process score, outcome success, and overall failure cause.
///
/// Process score: How well did the agent execute (0.0–1.0)?
/// - High if failures are environment blocks (agent did the right thing but was blocked)
/// - Low if agent errors (wrong selector, bad action)
///
/// Outcome success: Did the task actually complete?
///
/// Failure cause: Why did it fail overall?
fn compute_process_outcome_scores(
    execution_ok: bool,
    expectation_results: &[ExpectationResult],
    verification_status: &VerificationStatus,
) -> (Option<f64>, Option<bool>, FailureCause) {
    if !execution_ok {
        // Execution failed at transport level - process score is 0, outcome is failure
        return (Some(0.0), Some(false), FailureCause::AgentError);
    }

    if expectation_results.is_empty() {
        // No expectations - can't determine process/outcome
        return (None, None, FailureCause::None);
    }

    // Count failure causes
    let mut agent_errors = 0;
    let mut environment_blocks = 0;
    let mut evidence_absent = 0;
    let mut hallucination_suspected = 0;
    let mut total_failures = 0;

    for result in expectation_results {
        if result.status != VerificationStatus::Verified {
            total_failures += 1;
            match result.failure_cause {
                FailureCause::AgentError => agent_errors += 1,
                FailureCause::EnvironmentBlocked => environment_blocks += 1,
                FailureCause::EvidenceAbsent => evidence_absent += 1,
                FailureCause::HallucinationSuspected => hallucination_suspected += 1,
                FailureCause::None => {}
            }
        }
    }

    // Process score: How well did the agent execute?
    // High if failures are environment blocks, low if agent errors
    let process_score = if total_failures == 0 {
        Some(1.0)
    } else if agent_errors == 0 && hallucination_suspected == 0 {
        // All failures are environment blocks or evidence absent - agent did well
        Some(0.8)
    } else if agent_errors > 0 {
        // Some agent errors - penalize proportionally
        Some(1.0 - (agent_errors as f64 / expectation_results.len() as f64))
    } else {
        // Hallucination suspected - moderate penalty
        Some(0.5)
    };

    // Outcome success: Did the task complete?
    let outcome_success = Some(*verification_status == VerificationStatus::Verified);

    // Overall failure cause: prioritize by severity
    let failure_cause = if *verification_status == VerificationStatus::Verified {
        FailureCause::None
    } else if hallucination_suspected > 0 {
        FailureCause::HallucinationSuspected
    } else if environment_blocks > 0 {
        FailureCause::EnvironmentBlocked
    } else if agent_errors > 0 {
        FailureCause::AgentError
    } else if evidence_absent > 0 {
        FailureCause::EvidenceAbsent
    } else {
        FailureCause::AgentError // Default
    };

    (process_score, outcome_success, failure_cause)
}

/// Build a simple observation from a JSON evidence blob.
pub fn observation(source: &str, evidence: Value) -> GuiObservation {
    GuiObservation {
        evidence,
        source: source.to_string(),
    }
}

/// Determine which evidence keys to snapshot based on expectation kinds.
///
/// This inference is used for both pre-observation and post-observation
/// collection, so keep it focused on expectation-driven evidence needs.
pub fn infer_evidence_keys(expectations: &[GuiExpectation]) -> Vec<String> {
    let mut keys = Vec::new();
    for exp in expectations {
        match &exp.kind {
            GuiExpectationKind::FieldValueEquals { selector, .. } => {
                push_unique_key(&mut keys, format!("field_values.{selector}"));
            }
            GuiExpectationKind::UrlIs { .. } | GuiExpectationKind::UrlHostIs { .. } => {
                push_unique_key(&mut keys, "url".into());
            }
            GuiExpectationKind::WindowTitleContains { .. } => {
                push_unique_key(&mut keys, "title".into());
            }
            GuiExpectationKind::FocusedElementIs { .. } => {
                push_unique_key(&mut keys, "focused_element".into());
            }
            GuiExpectationKind::CheckboxChecked { selector, .. } => {
                push_unique_key(&mut keys, format!("checkbox_states.{selector}"));
            }
            GuiExpectationKind::DialogPresent { .. } => {
                push_unique_key(&mut keys, "dialog_present".into());
            }
            GuiExpectationKind::ElementAtCoordinate { .. } => {
                push_unique_key(&mut keys, "hit_test_result".into());
            }
            GuiExpectationKind::AXAttributeEquals { attribute, .. } => {
                push_unique_key(&mut keys, format!("ax_attributes.{attribute}"));
            }
            GuiExpectationKind::FrontWindowElementCountChanged {
                role,
                title_contains,
                description_contains,
                value_contains,
                ..
            } => push_unique_key(
                &mut keys,
                encode_front_window_match_count_key(
                    role.as_deref(),
                    title_contains.as_deref(),
                    description_contains.as_deref(),
                    value_contains.as_deref(),
                ),
            ),
            GuiExpectationKind::FileExists { .. }
            | GuiExpectationKind::DownloadCompleted { .. } => {}
        }
    }
    keys
}

/// Backward-compatible alias kept for existing call sites.
pub fn infer_pre_observation_keys(expectations: &[GuiExpectation]) -> Vec<String> {
    infer_evidence_keys(expectations)
}

/// Whether any expectation depends on pre-action state to verify a state delta.
pub fn expectations_require_pre_observation(expectations: &[GuiExpectation]) -> bool {
    expectations.iter().any(|exp| {
        matches!(
            exp.kind,
            GuiExpectationKind::FrontWindowElementCountChanged { .. }
        )
    })
}

/// Apply a wait strategy before collecting post-observation evidence.
pub async fn apply_wait_strategy<F, Fut>(
    strategy: &WaitStrategy,
    mut collect_evidence: F,
    expectations: &[GuiExpectation],
) -> anyhow::Result<Value>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = anyhow::Result<Value>>,
{
    match strategy {
        WaitStrategy::None => collect_evidence().await,
        WaitStrategy::FixedMs { ms } => {
            tokio::time::sleep(Duration::from_millis(*ms)).await;
            collect_evidence().await
        }
        WaitStrategy::PollUntilVerified {
            poll_interval_ms,
            timeout_ms,
        } => {
            let deadline = Instant::now() + Duration::from_millis(*timeout_ms);
            let mut last_evidence = collect_evidence().await?;
            loop {
                let (status, _) = verify_expectations(expectations, &last_evidence);
                if status == VerificationStatus::Verified || Instant::now() >= deadline {
                    return Ok(last_evidence);
                }
                tokio::time::sleep(Duration::from_millis(*poll_interval_ms)).await;
                last_evidence = collect_evidence().await?;
            }
        }
        WaitStrategy::DomEvent { .. } => {
            anyhow::bail!("wait.strategy=dom_event requires backend-native event handling; no fixed-sleep fallback is allowed")
        }
        WaitStrategy::AccessibilityEvent { .. } => {
            anyhow::bail!("wait.strategy=accessibility_event requires backend-native event handling; no fixed-sleep fallback is allowed")
        }
        WaitStrategy::SelectorPresent { .. } => {
            anyhow::bail!("wait.strategy=selector_present must be resolved by the tool backend before runtime polling")
        }
    }
}

/// Compute a flattened JSON diff between two observations.
pub fn compute_diff(pre: &GuiObservation, post: &GuiObservation) -> Option<Value> {
    let mut diff = serde_json::Map::new();
    collect_differences("", &pre.evidence, &post.evidence, &mut diff);
    if diff.is_empty() {
        None
    } else {
        Some(Value::Object(diff))
    }
}

/// Classify how reversible a GUI action is.
pub fn classify_reversibility(
    tool_name: &str,
    action: &str,
    args: &Value,
    expectations: &[GuiExpectation],
) -> ReversibilityLevel {
    if let Ok(Some(override_level)) = parse_reversibility_override(args) {
        return override_level;
    }

    match tool_name {
        "browser" => match action {
            "open" | "snapshot" | "get_text" | "get_title" | "get_url" | "screenshot"
            | "is_visible" | "hover" | "scroll" | "screen_capture" => {
                ReversibilityLevel::Reversible
            }
            "click" => classify_click_reversibility(args, expectations),
            "fill" | "type" => ReversibilityLevel::Reversible,
            "press" | "key_press" | "key_type" => classify_keypress_reversibility(args),
            "close" => ReversibilityLevel::PartiallyReversible,
            "mouse_click" | "mouse_drag" => ReversibilityLevel::Unknown,
            _ => ReversibilityLevel::Unknown,
        },
        "mac_automation" => match action {
            "launch_app" | "activate_app" => ReversibilityLevel::Reversible,
            "run_applescript" => classify_applescript_reversibility(args),
            "move_mouse" => ReversibilityLevel::Reversible,
            "click_at" => ReversibilityLevel::PartiallyReversible,
            "read_focused_element" | "hit_test" => ReversibilityLevel::Reversible,
            _ => ReversibilityLevel::Unknown,
        },
        _ => ReversibilityLevel::Unknown,
    }
}

/// Describe an expectation in human-readable form for approval prompts.
pub fn describe_expectation(expectation: &GuiExpectation) -> String {
    match &expectation.kind {
        GuiExpectationKind::FieldValueEquals { selector, value } => {
            format!("field '{selector}' becomes '{value}'")
        }
        GuiExpectationKind::FocusedElementIs { selector } => {
            format!("focus moves to '{selector}'")
        }
        GuiExpectationKind::CheckboxChecked { selector, checked } => {
            format!("checkbox '{selector}' checked={checked}")
        }
        GuiExpectationKind::WindowTitleContains { substring } => {
            format!("window title contains '{substring}'")
        }
        GuiExpectationKind::DialogPresent { present } => {
            format!("dialog present={present}")
        }
        GuiExpectationKind::UrlIs { url } => format!("URL becomes '{url}'"),
        GuiExpectationKind::UrlHostIs { host } => format!("URL host becomes '{host}'"),
        GuiExpectationKind::FileExists { path } => format!("file exists at '{path}'"),
        GuiExpectationKind::DownloadCompleted { path } => {
            format!("download completes at '{path}'")
        }
        GuiExpectationKind::FrontWindowElementCountChanged {
            role,
            title_contains,
            description_contains,
            value_contains,
            min_increase,
        } => {
            let matcher = describe_front_window_matcher(
                role.as_deref(),
                title_contains.as_deref(),
                description_contains.as_deref(),
                value_contains.as_deref(),
            );
            format!("front window elements matching {matcher} increase by at least {min_increase}")
        }
        GuiExpectationKind::ElementAtCoordinate {
            x,
            y,
            expected_element,
            tolerance_px,
        } => format!(
            "element '{expected_element}' is present near ({x}, {y}) with tolerance {tolerance_px}px"
        ),
        GuiExpectationKind::AXAttributeEquals { attribute, value } => {
            format!("AX attribute '{attribute}' equals '{value}'")
        }
    }
}

/// Determine whether a GUI action should block on approval before execution.
pub fn needs_gui_approval(
    config: &GuiVerificationConfig,
    autonomy: AutonomyLevel,
    reversibility: ReversibilityLevel,
) -> bool {
    let gate_enabled = match config.approval_gate {
        GuiApprovalGate::Never => false,
        GuiApprovalGate::Always => true,
        GuiApprovalGate::SupervisedOnly => autonomy == AutonomyLevel::Supervised,
    };

    gate_enabled && reversibility_meets_threshold(reversibility, config.approval_threshold)
}

fn reversibility_meets_threshold(
    reversibility: ReversibilityLevel,
    threshold: GuiApprovalThreshold,
) -> bool {
    match threshold {
        GuiApprovalThreshold::PartiallyReversible => matches!(
            reversibility,
            ReversibilityLevel::PartiallyReversible
                | ReversibilityLevel::Irreversible
                | ReversibilityLevel::Unknown
        ),
        GuiApprovalThreshold::Irreversible => {
            matches!(reversibility, ReversibilityLevel::Irreversible)
        }
        GuiApprovalThreshold::Unknown => matches!(reversibility, ReversibilityLevel::Unknown),
    }
}

// ── Per-expectation verification ────────────────────────────────

/// Infer why a verification failed based on the expectation kind and evidence.
fn infer_failure_cause(
    kind: &GuiExpectationKind,
    result: &ExpectationResult,
    post_evidence: &Value,
) -> FailureCause {
    match result.status {
        VerificationStatus::Verified => FailureCause::None,
        VerificationStatus::Ambiguous => {
            // Evidence is missing or incomplete
            if result.actual.is_null() {
                FailureCause::EvidenceAbsent
            } else {
                // Evidence exists but doesn't match - could be environment block
                detect_environment_block(kind, result, post_evidence)
            }
        }
        VerificationStatus::Failed => {
            // Check if this looks like an environment block
            detect_environment_block(kind, result, post_evidence)
        }
    }
}

/// Detect if a failure is due to environment blocks (login walls, CAPTCHAs, network errors, etc.)
/// rather than agent errors.
///
/// Based on research from 2025-2026 on common web automation failure patterns:
/// - Authentication errors (401, 403, login required, session expired)
/// - CAPTCHAs (reCAPTCHA, hCaptcha, Cloudflare Turnstile, behavioral challenges)
/// - Network failures (timeouts, 503, 504, connection errors)
/// - Resource unavailability (out of stock, sold out, currently unavailable)
fn detect_environment_block(
    kind: &GuiExpectationKind,
    result: &ExpectationResult,
    post_evidence: &Value,
) -> FailureCause {
    let actual_str = result.actual.as_str().unwrap_or("");
    let actual_lower = actual_str.to_ascii_lowercase();

    // Check message field as well
    let message_lower = result
        .message
        .as_ref()
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    let combined = format!("{} {}", actual_lower, message_lower);

    // Authentication and authorization blocks (401, 403, login walls, session expired)
    if combined.contains("login")
        || combined.contains("sign in")
        || combined.contains("sign-in")
        || combined.contains("authenticate")
        || combined.contains("authentication required")
        || combined.contains("authorization required")
        || combined.contains("unauthorized")
        || combined.contains("forbidden")
        || combined.contains("access denied")
        || combined.contains("session expired")
        || combined.contains("session timeout")
        || combined.contains("please log in")
        || combined.contains("you must be logged in")
        || combined.contains(" 401")
        || combined.contains(" 403")
        || combined.contains("http 401")
        || combined.contains("http 403")
    {
        return FailureCause::EnvironmentBlocked;
    }

    // CAPTCHA challenges (reCAPTCHA, hCaptcha, Cloudflare, behavioral verification)
    if combined.contains("captcha")
        || combined.contains("recaptcha")
        || combined.contains("hcaptcha")
        || combined.contains("cloudflare")
        || combined.contains("turnstile")
        || combined.contains("verify you")
        || combined.contains("verify that you")
        || combined.contains("human verification")
        || combined.contains("bot detection")
        || combined.contains("security check")
        || combined.contains("prove you're human")
        || combined.contains("are you a robot")
    {
        return FailureCause::EnvironmentBlocked;
    }

    // Network and server errors (timeouts, 503, 504, connection failures)
    if combined.contains("timeout")
        || combined.contains("timed out")
        || combined.contains("time out")
        || combined.contains("network error")
        || combined.contains("connection refused")
        || combined.contains("connection failed")
        || combined.contains("connection reset")
        || combined.contains("connection timeout")
        || combined.contains("service unavailable")
        || combined.contains("server error")
        || combined.contains("gateway timeout")
        || combined.contains(" 503")
        || combined.contains(" 504")
        || combined.contains(" 502")
        || combined.contains("http 503")
        || combined.contains("http 504")
        || combined.contains("http 502")
        || combined.contains("too many requests")
        || combined.contains(" 429")
        || combined.contains("http 429")
        || combined.contains("rate limit")
    {
        return FailureCause::EnvironmentBlocked;
    }

    // Resource unavailability (out of stock, sold out, not available)
    if combined.contains("out of stock")
        || combined.contains("sold out")
        || combined.contains("currently unavailable")
        || combined.contains("not available")
        || combined.contains("no longer available")
        || combined.contains("temporarily unavailable")
        || combined.contains("item unavailable")
        || combined.contains("product unavailable")
        || combined.contains("back in stock")
        || combined.contains("notify me")
        || combined.contains("waitlist")
    {
        return FailureCause::EnvironmentBlocked;
    }

    // Check URL patterns for environment blocks
    match kind {
        GuiExpectationKind::UrlIs { url } | GuiExpectationKind::UrlHostIs { host: url } => {
            let expected_lower = url.to_ascii_lowercase();
            // If we expected a success page but got an error/login page
            if (expected_lower.contains("success")
                || expected_lower.contains("confirmation")
                || expected_lower.contains("complete")
                || expected_lower.contains("thank")
                || expected_lower.contains("receipt"))
                && (actual_lower.contains("error")
                    || actual_lower.contains("failed")
                    || actual_lower.contains("denied")
                    || actual_lower.contains("login")
                    || actual_lower.contains("signin")
                    || actual_lower.contains("auth"))
            {
                return FailureCause::EnvironmentBlocked;
            }
        }
        _ => {}
    }

    // Check window title for environment blocks
    if let Some(title) = post_evidence.get("title").and_then(Value::as_str) {
        let title_lower = title.to_ascii_lowercase();
        if title_lower.contains("login")
            || title_lower.contains("sign in")
            || title_lower.contains("error")
            || title_lower.contains("access denied")
            || title_lower.contains("unauthorized")
            || title_lower.contains("captcha")
        {
            return FailureCause::EnvironmentBlocked;
        }
    }

    // Default: assume agent error (wrong selector, bad action, incorrect parameters)
    FailureCause::AgentError
}

fn verify_single(
    kind: &GuiExpectationKind,
    pre_evidence: Option<&Value>,
    post_evidence: &Value,
) -> ExpectationResult {
    let mut result = match kind {
        GuiExpectationKind::FieldValueEquals { selector, value } => {
            verify_field_value(selector, value, post_evidence)
        }
        GuiExpectationKind::FocusedElementIs { selector } => {
            verify_focused_element(selector, post_evidence)
        }
        GuiExpectationKind::CheckboxChecked { selector, checked } => {
            verify_checkbox(selector, *checked, post_evidence)
        }
        GuiExpectationKind::WindowTitleContains { substring } => {
            verify_window_title(substring, post_evidence)
        }
        GuiExpectationKind::DialogPresent { present } => {
            verify_dialog_present(*present, post_evidence)
        }
        GuiExpectationKind::UrlIs { url } => verify_url_is(url, post_evidence),
        GuiExpectationKind::UrlHostIs { host } => verify_url_host(host, post_evidence),
        GuiExpectationKind::FileExists { path } => verify_file_exists(path),
        GuiExpectationKind::DownloadCompleted { path } => verify_file_exists(path),
        GuiExpectationKind::FrontWindowElementCountChanged {
            role,
            title_contains,
            description_contains,
            value_contains,
            min_increase,
        } => verify_front_window_element_count_changed(
            role.as_deref(),
            title_contains.as_deref(),
            description_contains.as_deref(),
            value_contains.as_deref(),
            *min_increase,
            pre_evidence,
            post_evidence,
        ),
        GuiExpectationKind::ElementAtCoordinate {
            x,
            y,
            expected_element,
            tolerance_px,
        } => verify_element_at_coordinate(*x, *y, expected_element, *tolerance_px, post_evidence),
        GuiExpectationKind::AXAttributeEquals { attribute, value } => {
            verify_ax_attribute(attribute, value, post_evidence)
        }
    };
    result.confidence = Some(confidence_for(kind, result.status));
    result.failure_cause = infer_failure_cause(kind, &result, post_evidence);
    result
}

fn verify_field_value(selector: &str, expected: &str, evidence: &Value) -> ExpectationResult {
    let actual = evidence
        .get("field_values")
        .and_then(|fv| fv.get(selector))
        .and_then(Value::as_str);

    match actual {
        Some(actual_val) if actual_val == expected => expectation_result(
            VerificationStatus::Verified,
            json!({"selector": selector, "value": expected}),
            json!(actual_val),
            None,
        ),
        Some(actual_val) => expectation_result(
            VerificationStatus::Failed,
            json!({"selector": selector, "value": expected}),
            json!(actual_val),
            Some(format!(
                "Field '{selector}' has value '{actual_val}', expected '{expected}'"
            )),
        ),
        None => expectation_result(
            VerificationStatus::Ambiguous,
            json!({"selector": selector, "value": expected}),
            Value::Null,
            Some(format!(
                "Could not read field value for '{selector}' from post-action evidence"
            )),
        ),
    }
}

fn verify_focused_element(selector: &str, evidence: &Value) -> ExpectationResult {
    let focused = evidence.get("focused_element").and_then(Value::as_str);

    match focused {
        Some(actual) if actual == selector => expectation_result(
            VerificationStatus::Verified,
            json!({"focused": selector}),
            json!(actual),
            None,
        ),
        Some(actual) => expectation_result(
            VerificationStatus::Failed,
            json!({"focused": selector}),
            json!(actual),
            Some(format!(
                "Focused element is '{actual}', expected '{selector}'"
            )),
        ),
        None => expectation_result(
            VerificationStatus::Ambiguous,
            json!({"focused": selector}),
            Value::Null,
            Some("Could not determine focused element from post-action evidence".into()),
        ),
    }
}

fn verify_checkbox(selector: &str, expected_checked: bool, evidence: &Value) -> ExpectationResult {
    let actual = evidence
        .get("checkbox_states")
        .and_then(|cs| cs.get(selector))
        .and_then(Value::as_bool);

    match actual {
        Some(actual_val) if actual_val == expected_checked => expectation_result(
            VerificationStatus::Verified,
            json!({"selector": selector, "checked": expected_checked}),
            json!(actual_val),
            None,
        ),
        Some(actual_val) => expectation_result(
            VerificationStatus::Failed,
            json!({"selector": selector, "checked": expected_checked}),
            json!(actual_val),
            Some(format!(
                "Checkbox '{selector}' checked={actual_val}, expected checked={expected_checked}"
            )),
        ),
        None => expectation_result(
            VerificationStatus::Ambiguous,
            json!({"selector": selector, "checked": expected_checked}),
            Value::Null,
            Some(format!(
                "Could not read checkbox state for '{selector}' from post-action evidence"
            )),
        ),
    }
}

fn verify_window_title(substring: &str, evidence: &Value) -> ExpectationResult {
    let title = evidence
        .get("title")
        .and_then(Value::as_str)
        .or_else(|| evidence.get("window_title").and_then(Value::as_str));

    match title {
        Some(actual) if actual.contains(substring) => expectation_result(
            VerificationStatus::Verified,
            json!({"title_contains": substring}),
            json!(actual),
            None,
        ),
        Some(actual) => expectation_result(
            VerificationStatus::Failed,
            json!({"title_contains": substring}),
            json!(actual),
            Some(format!(
                "Window title '{actual}' does not contain '{substring}'"
            )),
        ),
        None => expectation_result(
            VerificationStatus::Ambiguous,
            json!({"title_contains": substring}),
            Value::Null,
            Some("Could not read window title from post-action evidence".into()),
        ),
    }
}

fn verify_dialog_present(expected_present: bool, evidence: &Value) -> ExpectationResult {
    let present = evidence.get("dialog_present").and_then(Value::as_bool);

    match present {
        Some(actual) if actual == expected_present => expectation_result(
            VerificationStatus::Verified,
            json!({"dialog_present": expected_present}),
            json!(actual),
            None,
        ),
        Some(actual) => expectation_result(
            VerificationStatus::Failed,
            json!({"dialog_present": expected_present}),
            json!(actual),
            Some(format!(
                "Dialog present={actual}, expected present={expected_present}"
            )),
        ),
        None => expectation_result(
            VerificationStatus::Ambiguous,
            json!({"dialog_present": expected_present}),
            Value::Null,
            Some("Could not determine dialog state from post-action evidence".into()),
        ),
    }
}

fn verify_url_is(expected: &str, evidence: &Value) -> ExpectationResult {
    let url = evidence.get("url").and_then(Value::as_str);

    match url {
        Some(actual) if actual == expected => expectation_result(
            VerificationStatus::Verified,
            json!({"url": expected}),
            json!(actual),
            None,
        ),
        Some(actual) => expectation_result(
            VerificationStatus::Failed,
            json!({"url": expected}),
            json!(actual),
            Some(format!("URL is '{actual}', expected '{expected}'")),
        ),
        None => expectation_result(
            VerificationStatus::Ambiguous,
            json!({"url": expected}),
            Value::Null,
            Some("Could not read URL from post-action evidence".into()),
        ),
    }
}

fn verify_url_host(expected_host: &str, evidence: &Value) -> ExpectationResult {
    let url = evidence.get("url").and_then(Value::as_str);

    match url {
        Some(actual_url) => {
            let host = reqwest::Url::parse(actual_url)
                .ok()
                .and_then(|u| u.host_str().map(str::to_lowercase));
            match host {
                Some(h) if h == expected_host.to_lowercase() => expectation_result(
                    VerificationStatus::Verified,
                    json!({"url_host": expected_host}),
                    json!(h),
                    None,
                ),
                Some(h) => expectation_result(
                    VerificationStatus::Failed,
                    json!({"url_host": expected_host}),
                    json!(h),
                    Some(format!("URL host is '{h}', expected '{expected_host}'")),
                ),
                None => expectation_result(
                    VerificationStatus::Ambiguous,
                    json!({"url_host": expected_host}),
                    json!(actual_url),
                    Some(format!("Could not parse host from URL '{actual_url}'")),
                ),
            }
        }
        None => expectation_result(
            VerificationStatus::Ambiguous,
            json!({"url_host": expected_host}),
            Value::Null,
            Some("Could not read URL from post-action evidence".into()),
        ),
    }
}

fn verify_file_exists(path: &str) -> ExpectationResult {
    let exists = std::path::Path::new(path).exists();
    if exists {
        expectation_result(
            VerificationStatus::Verified,
            json!({"file_exists": path}),
            json!(true),
            None,
        )
    } else {
        expectation_result(
            VerificationStatus::Failed,
            json!({"file_exists": path}),
            json!(false),
            Some(format!("File does not exist: {path}")),
        )
    }
}

pub fn encode_front_window_match_count_key(
    role: Option<&str>,
    title_contains: Option<&str>,
    description_contains: Option<&str>,
    value_contains: Option<&str>,
) -> String {
    let payload = json!({
        "role": role.unwrap_or_default(),
        "title_contains": title_contains.unwrap_or_default(),
        "description_contains": description_contains.unwrap_or_default(),
        "value_contains": value_contains.unwrap_or_default(),
    });
    format!(
        "front_window_match_count::{}",
        serde_json::to_string(&payload).unwrap_or_else(|_| "{}".into())
    )
}

fn describe_front_window_matcher(
    role: Option<&str>,
    title_contains: Option<&str>,
    description_contains: Option<&str>,
    value_contains: Option<&str>,
) -> String {
    let mut parts = Vec::new();
    if let Some(role) = role.filter(|value| !value.is_empty()) {
        parts.push(format!("role '{role}'"));
    }
    if let Some(value) = title_contains.filter(|value| !value.is_empty()) {
        parts.push(format!("title containing '{value}'"));
    }
    if let Some(value) = description_contains.filter(|value| !value.is_empty()) {
        parts.push(format!("description containing '{value}'"));
    }
    if let Some(value) = value_contains.filter(|value| !value.is_empty()) {
        parts.push(format!("value containing '{value}'"));
    }

    if parts.is_empty() {
        "any accessible element".into()
    } else {
        parts.join(", ")
    }
}

fn verify_front_window_element_count_changed(
    role: Option<&str>,
    title_contains: Option<&str>,
    description_contains: Option<&str>,
    value_contains: Option<&str>,
    min_increase: u32,
    pre_evidence: Option<&Value>,
    post_evidence: &Value,
) -> ExpectationResult {
    let key = encode_front_window_match_count_key(
        role,
        title_contains,
        description_contains,
        value_contains,
    );
    let expected = json!({
        "matcher": {
            "role": role,
            "title_contains": title_contains,
            "description_contains": description_contains,
            "value_contains": value_contains,
        },
        "min_increase": min_increase,
    });

    let Some(pre_count) = front_window_match_count(pre_evidence, &key) else {
        return expectation_result(
            VerificationStatus::Ambiguous,
            expected,
            Value::Null,
            Some(
                "Could not read the pre-action front window match count. Use pre_observe:auto or explicit keys."
                    .into(),
            ),
        );
    };

    let Some(post_snapshot) = front_window_match_snapshot(Some(post_evidence), &key) else {
        return expectation_result(
            VerificationStatus::Ambiguous,
            expected,
            Value::Null,
            Some("Could not read the post-action front window match count".into()),
        );
    };

    let post_count = post_snapshot
        .get("count")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let actual = json!({
        "pre_count": pre_count,
        "post_count": post_count,
        "increase": post_count.saturating_sub(pre_count),
        "samples": post_snapshot.get("samples").cloned().unwrap_or(Value::Array(vec![])),
    });

    if post_count >= pre_count.saturating_add(u64::from(min_increase)) {
        expectation_result(VerificationStatus::Verified, expected, actual, None)
    } else {
        expectation_result(
            VerificationStatus::Failed,
            expected,
            actual,
            Some(format!(
                "Front window match count did not increase by at least {min_increase} (pre={pre_count}, post={post_count})"
            )),
        )
    }
}

fn front_window_match_count(evidence: Option<&Value>, key: &str) -> Option<u64> {
    front_window_match_snapshot(evidence, key)?
        .get("count")
        .and_then(Value::as_u64)
}

fn front_window_match_snapshot<'a>(evidence: Option<&'a Value>, key: &str) -> Option<&'a Value> {
    evidence?
        .get("front_window_match_counts")
        .and_then(|counts| counts.get(key))
}

fn verify_element_at_coordinate(
    x: i64,
    y: i64,
    expected: &str,
    tolerance: u32,
    evidence: &Value,
) -> ExpectationResult {
    let hit = evidence.get("hit_test_result");
    match hit {
        Some(elem) => {
            let label = elem.get("label").and_then(Value::as_str).unwrap_or("");
            let role = elem.get("role").and_then(Value::as_str).unwrap_or("");
            let selector = elem.get("selector").and_then(Value::as_str).unwrap_or("");
            let description = elem
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("");
            let expected_lower = expected.to_ascii_lowercase();
            let matches = [label, role, selector, description]
                .iter()
                .any(|field| field.to_ascii_lowercase().contains(expected_lower.as_str()));

            let bbox_ok = if let Some(bb) = elem.get("bounding_box") {
                let bx = bb.get("x").and_then(Value::as_i64).unwrap_or(0);
                let by = bb.get("y").and_then(Value::as_i64).unwrap_or(0);
                let bw = bb.get("width").and_then(Value::as_i64).unwrap_or(0);
                let bh = bb.get("height").and_then(Value::as_i64).unwrap_or(0);
                let tol = i64::from(tolerance);
                x >= bx - tol && x <= bx + bw + tol && y >= by - tol && y <= by + bh + tol
            } else {
                true
            };

            if matches && bbox_ok {
                expectation_result(
                    VerificationStatus::Verified,
                    json!({"x": x, "y": y, "expected_element": expected, "tolerance_px": tolerance}),
                    elem.clone(),
                    None,
                )
            } else {
                expectation_result(
                    VerificationStatus::Failed,
                    json!({"x": x, "y": y, "expected_element": expected, "tolerance_px": tolerance}),
                    elem.clone(),
                    Some(format!(
                        "Element at ({x}, {y}) does not match '{expected}' within {tolerance}px tolerance"
                    )),
                )
            }
        }
        None => expectation_result(
            VerificationStatus::Ambiguous,
            json!({"x": x, "y": y, "expected_element": expected, "tolerance_px": tolerance}),
            Value::Null,
            Some("Could not determine element at coordinate from post-action evidence".into()),
        ),
    }
}

fn verify_ax_attribute(attribute: &str, expected: &str, evidence: &Value) -> ExpectationResult {
    let actual = evidence
        .get("ax_attributes")
        .and_then(|attrs| attrs.get(attribute))
        .and_then(Value::as_str);

    match actual {
        Some(actual_val) if actual_val == expected => expectation_result(
            VerificationStatus::Verified,
            json!({"attribute": attribute, "value": expected}),
            json!(actual_val),
            None,
        ),
        Some(actual_val) => expectation_result(
            VerificationStatus::Failed,
            json!({"attribute": attribute, "value": expected}),
            json!(actual_val),
            Some(format!(
                "AX attribute '{attribute}' is '{actual_val}', expected '{expected}'"
            )),
        ),
        None => expectation_result(
            VerificationStatus::Ambiguous,
            json!({"attribute": attribute, "value": expected}),
            Value::Null,
            Some(format!(
                "Could not read AX attribute '{attribute}' from post-action evidence"
            )),
        ),
    }
}

fn expectation_result(
    status: VerificationStatus,
    expected: Value,
    actual: Value,
    message: Option<String>,
) -> ExpectationResult {
    ExpectationResult {
        status,
        expected,
        actual,
        message,
        confidence: None,
        failure_cause: FailureCause::None,
    }
}

fn summarize_verification(
    expectations: &[GuiExpectation],
    results: &[ExpectationResult],
) -> VerificationStatus {
    if results.is_empty() {
        return VerificationStatus::Verified;
    }

    let mut any_required_failed = false;
    let mut any_ambiguous = false;

    for (exp, result) in expectations.iter().zip(results.iter()) {
        match (exp.required, result.status) {
            (true, VerificationStatus::Failed) => any_required_failed = true,
            (false, VerificationStatus::Failed) => any_ambiguous = true,
            (_, VerificationStatus::Ambiguous) => any_ambiguous = true,
            _ => {}
        }
    }

    if any_required_failed {
        VerificationStatus::Failed
    } else if any_ambiguous {
        VerificationStatus::Ambiguous
    } else {
        VerificationStatus::Verified
    }
}

fn aggregate_confidence(results: &[ExpectationResult]) -> Option<f64> {
    let confidences: Vec<f64> = results
        .iter()
        .filter_map(|result| result.confidence)
        .collect();
    if confidences.is_empty() {
        return None;
    }

    let product = confidences.iter().copied().product::<f64>();
    Some(product.powf(1.0 / confidences.len() as f64))
}

fn confidence_for(kind: &GuiExpectationKind, status: VerificationStatus) -> f64 {
    match status {
        VerificationStatus::Failed => match kind {
            GuiExpectationKind::ElementAtCoordinate { .. } => 0.05,
            _ => 0.0,
        },
        VerificationStatus::Ambiguous => 0.3,
        VerificationStatus::Verified => match kind {
            GuiExpectationKind::FieldValueEquals { .. }
            | GuiExpectationKind::UrlIs { .. }
            | GuiExpectationKind::CheckboxChecked { .. } => 1.0,
            GuiExpectationKind::WindowTitleContains { .. }
            | GuiExpectationKind::FocusedElementIs { .. } => 0.95,
            GuiExpectationKind::UrlHostIs { .. } | GuiExpectationKind::DialogPresent { .. } => 0.9,
            GuiExpectationKind::FileExists { .. }
            | GuiExpectationKind::DownloadCompleted { .. } => 0.85,
            GuiExpectationKind::FrontWindowElementCountChanged { .. } => 0.9,
            GuiExpectationKind::ElementAtCoordinate { .. } => 0.8,
            GuiExpectationKind::AXAttributeEquals { .. } => 0.9,
        },
    }
}

fn required_string_field(
    map: &serde_json::Map<String, Value>,
    key: &str,
) -> anyhow::Result<String> {
    map.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid wait.{key}"))
}

fn required_u64_field(map: &serde_json::Map<String, Value>, key: &str) -> anyhow::Result<u64> {
    map.get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid wait.{key}"))
}

fn push_unique_key(keys: &mut Vec<String>, candidate: String) {
    if !keys.iter().any(|existing| existing == &candidate) {
        keys.push(candidate);
    }
}

fn collect_differences(
    prefix: &str,
    before: &Value,
    after: &Value,
    output: &mut serde_json::Map<String, Value>,
) {
    match (before, after) {
        (Value::Object(before_map), Value::Object(after_map)) => {
            let mut keys: Vec<String> = before_map.keys().cloned().collect();
            for key in after_map.keys() {
                if !keys.iter().any(|existing| existing == key) {
                    keys.push(key.clone());
                }
            }
            for key in keys {
                let child_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                collect_differences(
                    &child_prefix,
                    before_map.get(&key).unwrap_or(&Value::Null),
                    after_map.get(&key).unwrap_or(&Value::Null),
                    output,
                );
            }
        }
        _ if before != after => {
            output.insert(
                prefix.to_string(),
                json!({
                    "before": before,
                    "after": after,
                }),
            );
        }
        _ => {}
    }
}

fn classify_click_reversibility(
    args: &Value,
    expectations: &[GuiExpectation],
) -> ReversibilityLevel {
    let selector = args
        .get("selector")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let irreversible_patterns = [
        "submit",
        "delete",
        "remove",
        "pay",
        "purchase",
        "send",
        "confirm",
        "finalize",
        "checkout",
        "destroy",
        "deactivate",
    ];
    if irreversible_patterns
        .iter()
        .any(|pattern| selector.contains(pattern))
    {
        return ReversibilityLevel::Irreversible;
    }

    for exp in expectations {
        if let GuiExpectationKind::UrlIs { url } = &exp.kind {
            let url_lower = url.to_ascii_lowercase();
            if ["success", "confirmation", "thank", "receipt"]
                .iter()
                .any(|needle| url_lower.contains(needle))
            {
                return ReversibilityLevel::Irreversible;
            }
        }
    }

    ReversibilityLevel::PartiallyReversible
}

fn classify_keypress_reversibility(args: &Value) -> ReversibilityLevel {
    let key = args
        .get("key")
        .and_then(Value::as_str)
        .or_else(|| args.get("text").and_then(Value::as_str))
        .unwrap_or_default()
        .to_ascii_lowercase();

    match key.as_str() {
        "enter" | "return" => ReversibilityLevel::PartiallyReversible,
        _ if key.is_empty() => ReversibilityLevel::Unknown,
        _ => ReversibilityLevel::Reversible,
    }
}

fn classify_applescript_reversibility(args: &Value) -> ReversibilityLevel {
    let script = args
        .get("script")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .or_else(|| {
            args.get("script_lines")
                .and_then(Value::as_array)
                .map(|lines| {
                    lines
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join("\n")
                })
        })
        .unwrap_or_default()
        .to_ascii_lowercase();

    if script.is_empty() {
        return ReversibilityLevel::Unknown;
    }

    if ["delete", "remove", "empty trash", "send", "click menu item"]
        .iter()
        .any(|needle| script.contains(needle))
    {
        ReversibilityLevel::Irreversible
    } else if ["keystroke return", "key code 36", "confirm", "save"]
        .iter()
        .any(|needle| script.contains(needle))
    {
        ReversibilityLevel::PartiallyReversible
    } else {
        ReversibilityLevel::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GuiApprovalGate, GuiApprovalThreshold, GuiVerificationConfig};
    use crate::security::AutonomyLevel;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[test]
    fn parse_expectations_absent_returns_none() {
        let args = json!({"action": "click", "selector": "#btn"});
        assert!(parse_expectations(&args).unwrap().is_none());
    }

    #[test]
    fn parse_expectations_null_returns_none() {
        let args = json!({"action": "click", "expect": null});
        assert!(parse_expectations(&args).unwrap().is_none());
    }

    #[test]
    fn parse_expectations_single_object() {
        let args = json!({
            "action": "fill",
            "expect": {
                "kind": "field_value_equals",
                "selector": "#name",
                "value": "Alice"
            }
        });
        let exps = parse_expectations(&args).unwrap().unwrap();
        assert_eq!(exps.len(), 1);
        assert!(exps[0].required);
    }

    #[test]
    fn parse_expectations_array() {
        let args = json!({
            "action": "click",
            "expect": [
                {"kind": "url_is", "url": "https://example.com/done"},
                {"kind": "window_title_contains", "substring": "Done", "required": false}
            ]
        });
        let exps = parse_expectations(&args).unwrap().unwrap();
        assert_eq!(exps.len(), 2);
        assert!(exps[0].required);
        assert!(!exps[1].required);
    }

    #[test]
    fn parse_expectations_window_title_contains_accepts_title_contains_alias() {
        let args = json!({
            "action": "click",
            "expect": {
                "kind": "window_title_contains",
                "title_contains": "Photo Booth"
            }
        });

        let exps = parse_expectations(&args).unwrap().unwrap();
        assert_eq!(exps.len(), 1);
        match &exps[0].kind {
            GuiExpectationKind::WindowTitleContains { substring } => {
                assert_eq!(substring, "Photo Booth");
            }
            other => panic!("expected window_title_contains expectation, got {other:?}"),
        }
    }

    #[test]
    fn parse_expectations_empty_array_returns_none() {
        let args = json!({"action": "click", "expect": []});
        assert!(parse_expectations(&args).unwrap().is_none());
    }

    #[test]
    fn parse_expectations_invalid_kind_errors() {
        let args = json!({"expect": {"kind": "bogus_kind"}});
        assert!(parse_expectations(&args).is_err());
    }

    #[test]
    fn parse_pre_observe_auto() {
        let args = json!({"pre_observe": "auto"});
        assert_eq!(
            parse_pre_observation_strategy(&args).unwrap(),
            PreObservationStrategy::Auto
        );
    }

    #[test]
    fn parse_wait_poll_until_verified() {
        let args = json!({
            "wait": {
                "strategy": "poll_until_verified",
                "poll_interval_ms": 10,
                "timeout_ms": 30
            }
        });
        assert_eq!(
            parse_wait_strategy(&args).unwrap(),
            WaitStrategy::PollUntilVerified {
                poll_interval_ms: 10,
                timeout_ms: 30,
            }
        );
    }

    #[test]
    fn verify_field_value_equals_verified() {
        let evidence = json!({"field_values": {"#name": "Alice"}});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::FieldValueEquals {
                selector: "#name".into(),
                value: "Alice".into(),
            },
            required: true,
        }];
        let (status, results) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Verified);
        assert_eq!(results[0].status, VerificationStatus::Verified);
    }

    #[test]
    fn verify_field_value_equals_failed() {
        let evidence = json!({"field_values": {"#name": "Bob"}});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::FieldValueEquals {
                selector: "#name".into(),
                value: "Alice".into(),
            },
            required: true,
        }];
        let (status, _) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Failed);
    }

    #[test]
    fn verify_field_value_missing_is_ambiguous() {
        let evidence = json!({});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::FieldValueEquals {
                selector: "#name".into(),
                value: "Alice".into(),
            },
            required: true,
        }];
        let (status, _) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Ambiguous);
    }

    #[test]
    fn verify_url_is_verified() {
        let evidence = json!({"url": "https://example.com/done"});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::UrlIs {
                url: "https://example.com/done".into(),
            },
            required: true,
        }];
        let (status, _) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Verified);
    }

    #[test]
    fn verify_url_host_verified() {
        let evidence = json!({"url": "https://example.com/some/path"});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::UrlHostIs {
                host: "example.com".into(),
            },
            required: true,
        }];
        let (status, _) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Verified);
    }

    #[test]
    fn verify_window_title_contains() {
        let evidence = json!({"title": "My App - Settings"});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::WindowTitleContains {
                substring: "Settings".into(),
            },
            required: true,
        }];
        let (status, _) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Verified);
    }

    #[test]
    fn verify_checkbox_checked() {
        let evidence = json!({"checkbox_states": {"#agree": true}});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::CheckboxChecked {
                selector: "#agree".into(),
                checked: true,
            },
            required: true,
        }];
        let (status, _) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Verified);
    }

    #[test]
    fn verify_file_exists_verified() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::FileExists { path: path.clone() },
            required: true,
        }];
        let (status, _) = verify_expectations(&exps, &json!({}));
        assert_eq!(status, VerificationStatus::Verified);
    }

    #[test]
    fn verify_file_exists_failed() {
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::FileExists {
                path: "/tmp/nonexistent_gui_verify_test_file_12345".into(),
            },
            required: true,
        }];
        let (status, _) = verify_expectations(&exps, &json!({}));
        assert_eq!(status, VerificationStatus::Failed);
    }

    #[test]
    fn verify_front_window_element_count_changed_verified() {
        let key =
            encode_front_window_match_count_key(Some("AXImage"), None, Some("thumbnail"), None);
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::FrontWindowElementCountChanged {
                role: Some("AXImage".into()),
                title_contains: None,
                description_contains: Some("thumbnail".into()),
                value_contains: None,
                min_increase: 1,
            },
            required: true,
        }];
        let pre = json!({
            "front_window_match_counts": {
                key.clone(): {
                    "count": 2,
                    "samples": []
                }
            }
        });
        let post = json!({
            "front_window_match_counts": {
                key: {
                    "count": 3,
                    "samples": [{"role": "AXImage", "description": "thumbnail"}]
                }
            }
        });
        let (status, results) = verify_expectations_with_context(&exps, Some(&pre), &post);
        assert_eq!(status, VerificationStatus::Verified);
        assert_eq!(results[0].confidence, Some(0.9));
    }

    #[test]
    fn verify_front_window_element_count_changed_is_ambiguous_without_pre_state() {
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::FrontWindowElementCountChanged {
                role: Some("AXImage".into()),
                title_contains: None,
                description_contains: Some("thumbnail".into()),
                value_contains: None,
                min_increase: 1,
            },
            required: true,
        }];
        let (status, _) = verify_expectations_with_context(&exps, None, &json!({}));
        assert_eq!(status, VerificationStatus::Ambiguous);
        assert!(expectations_require_pre_observation(&exps));
    }

    #[test]
    fn verify_dialog_present_verified() {
        let evidence = json!({"dialog_present": true});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::DialogPresent { present: true },
            required: true,
        }];
        let (status, _) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Verified);
    }

    #[test]
    fn optional_failure_does_not_fail_overall() {
        let evidence = json!({"url": "https://other.com"});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::UrlIs {
                url: "https://example.com".into(),
            },
            required: false,
        }];
        let (status, results) = verify_expectations(&exps, &evidence);
        // Non-required failure → overall is Ambiguous, not Failed
        assert_eq!(results[0].status, VerificationStatus::Failed);
        assert_eq!(status, VerificationStatus::Ambiguous);
    }

    #[test]
    fn build_report_execution_failed() {
        let report = build_report(false, None, None, &[], &json!({}));
        assert!(!report.execution_ok);
        assert_eq!(report.verification_status, VerificationStatus::Failed);
    }

    #[test]
    fn build_report_verified() {
        let evidence = json!({"url": "https://example.com/done"});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::UrlIs {
                url: "https://example.com/done".into(),
            },
            required: true,
        }];
        let report = build_report(true, None, None, &exps, &evidence);
        assert!(report.execution_ok);
        assert_eq!(report.verification_status, VerificationStatus::Verified);
        assert_eq!(report.expectation_results.len(), 1);
    }

    #[test]
    fn pre_observation_auto_infers_url_key() {
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::UrlIs {
                url: "https://example.com/done".into(),
            },
            required: true,
        }];
        assert_eq!(infer_evidence_keys(&exps), vec!["url".to_string()]);
        assert_eq!(infer_pre_observation_keys(&exps), vec!["url".to_string()]);
    }

    #[test]
    fn state_diff_computed_when_both_present() {
        let pre = observation("pre", json!({"url": "https://example.com/start"}));
        let post = observation("post", json!({"url": "https://example.com/done"}));
        let diff = compute_diff(&pre, &post).unwrap();
        assert_eq!(
            diff["url"],
            json!({
                "before": "https://example.com/start",
                "after": "https://example.com/done"
            })
        );
    }

    #[test]
    fn verify_element_at_coordinate_verified() {
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::ElementAtCoordinate {
                x: 450,
                y: 300,
                expected_element: "Submit".into(),
                tolerance_px: 5,
            },
            required: true,
        }];
        let evidence = json!({
            "hit_test_result": {
                "label": "Submit",
                "role": "button",
                "selector": "#submit",
                "bounding_box": { "x": 420, "y": 280, "width": 100, "height": 40 }
            }
        });
        let (status, results) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Verified);
        assert_eq!(results[0].confidence, Some(0.8));
    }

    #[test]
    fn verify_element_at_coordinate_failed() {
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::ElementAtCoordinate {
                x: 450,
                y: 300,
                expected_element: "Delete".into(),
                tolerance_px: 0,
            },
            required: true,
        }];
        let evidence = json!({
            "hit_test_result": {
                "label": "Submit",
                "role": "button",
                "selector": "#submit"
            }
        });
        let (status, _) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Failed);
    }

    #[test]
    fn tool_result_from_gui_report_success_follows_verification() {
        use super::super::traits::ToolResult;

        let report = build_report(true, None, None, &[], &json!({}));
        assert_eq!(report.verification_status, VerificationStatus::Verified);
        let result = ToolResult::from_gui_report(&report);
        assert!(result.success);
        assert!(result.error.is_none());

        let evidence = json!({"url": "https://wrong.com"});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::UrlIs {
                url: "https://expected.com".into(),
            },
            required: true,
        }];
        let report = build_report(true, None, None, &exps, &evidence);
        assert_eq!(report.verification_status, VerificationStatus::Failed);
        let result = ToolResult::from_gui_report(&report);
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn confidence_exact_match_is_1() {
        let evidence = json!({"field_values": {"#name": "Alice"}});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::FieldValueEquals {
                selector: "#name".into(),
                value: "Alice".into(),
            },
            required: true,
        }];
        let (_, results) = verify_expectations(&exps, &evidence);
        assert_eq!(results[0].confidence, Some(1.0));
    }

    #[test]
    fn confidence_aggregate_geometric_mean() {
        let report = build_report_from_results(
            true,
            None,
            None,
            vec![
                ExpectationResult {
                    status: VerificationStatus::Verified,
                    expected: Value::Null,
                    actual: Value::Null,
                    message: None,
                    confidence: Some(1.0),
                    failure_cause: FailureCause::None,
                },
                ExpectationResult {
                    status: VerificationStatus::Verified,
                    expected: Value::Null,
                    actual: Value::Null,
                    message: None,
                    confidence: Some(0.9),
                    failure_cause: FailureCause::None,
                },
            ],
            &[
                GuiExpectation {
                    kind: GuiExpectationKind::UrlIs {
                        url: "https://example.com".into(),
                    },
                    required: true,
                },
                GuiExpectation {
                    kind: GuiExpectationKind::UrlHostIs {
                        host: "example.com".into(),
                    },
                    required: true,
                },
            ],
        );
        let confidence = report.confidence.unwrap();
        assert!((confidence - 0.948_683).abs() < 0.000_01);
    }

    #[tokio::test]
    async fn wait_poll_until_verified_succeeds_on_second_poll() {
        let polls = Arc::new(AtomicUsize::new(0));
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::UrlIs {
                url: "https://example.com/done".into(),
            },
            required: true,
        }];

        let evidence = apply_wait_strategy(
            &WaitStrategy::PollUntilVerified {
                poll_interval_ms: 1,
                timeout_ms: 20,
            },
            {
                let polls = Arc::clone(&polls);
                move || {
                    let polls = Arc::clone(&polls);
                    async move {
                        let count = polls.fetch_add(1, Ordering::SeqCst);
                        Ok(if count == 0 {
                            json!({"url": "https://example.com/start"})
                        } else {
                            json!({"url": "https://example.com/done"})
                        })
                    }
                }
            },
            &exps,
        )
        .await
        .unwrap();

        assert_eq!(evidence["url"], "https://example.com/done");
        assert!(polls.load(Ordering::SeqCst) >= 2);
    }

    #[tokio::test]
    async fn wait_selector_present_errors_without_backend_handling() {
        let err = apply_wait_strategy(
            &WaitStrategy::SelectorPresent {
                selector: "#submit".into(),
                timeout_ms: 100,
            },
            || async { Ok(json!({})) },
            &[],
        )
        .await
        .expect_err("selector_present should require backend wait handling");
        assert!(err.to_string().contains("wait.strategy=selector_present"));
    }

    #[test]
    fn classify_reversibility_respects_override() {
        let args = json!({"reversibility": "irreversible"});
        assert_eq!(
            classify_reversibility("browser", "click", &args, &[]),
            ReversibilityLevel::Irreversible
        );
    }

    #[test]
    fn classify_reversibility_marks_submit_click_irreversible() {
        let args = json!({"selector": "#submit-payment"});
        assert_eq!(
            classify_reversibility("browser", "click", &args, &[]),
            ReversibilityLevel::Irreversible
        );
    }

    #[test]
    fn gui_approval_default_threshold_does_not_prompt_for_unknown_in_supervised_mode() {
        let config = GuiVerificationConfig::default();
        assert!(!needs_gui_approval(
            &config,
            AutonomyLevel::Supervised,
            ReversibilityLevel::Unknown
        ));
    }

    #[test]
    fn gui_approval_unknown_threshold_prompts_for_unknown_actions() {
        let config = GuiVerificationConfig {
            approval_gate: GuiApprovalGate::Always,
            approval_threshold: GuiApprovalThreshold::Unknown,
            approval_timeout_secs: 30,
            click_at_preflight: crate::config::ClickAtPreflightMode::default(),
        };
        assert!(needs_gui_approval(
            &config,
            AutonomyLevel::Full,
            ReversibilityLevel::Unknown
        ));
        assert!(!needs_gui_approval(
            &config,
            AutonomyLevel::Full,
            ReversibilityLevel::Irreversible
        ));
    }

    #[test]
    fn gui_approval_supervised_only_skips_full_autonomy() {
        let config = GuiVerificationConfig::default();
        assert!(!needs_gui_approval(
            &config,
            AutonomyLevel::Full,
            ReversibilityLevel::Irreversible
        ));
    }

    #[test]
    fn gui_approval_partially_reversible_threshold_catches_partial_actions() {
        let config = GuiVerificationConfig {
            approval_gate: GuiApprovalGate::Always,
            approval_threshold: GuiApprovalThreshold::PartiallyReversible,
            approval_timeout_secs: 30,
            click_at_preflight: crate::config::ClickAtPreflightMode::default(),
        };
        assert!(needs_gui_approval(
            &config,
            AutonomyLevel::Full,
            ReversibilityLevel::PartiallyReversible
        ));
    }

    #[test]
    fn describe_expectation_renders_url_host_expectation() {
        let expectation = GuiExpectation {
            kind: GuiExpectationKind::UrlHostIs {
                host: "example.com".into(),
            },
            required: true,
        };
        assert_eq!(
            describe_expectation(&expectation),
            "URL host becomes 'example.com'"
        );
    }

    #[test]
    fn non_required_failure_yields_ambiguous_not_verified() {
        let evidence = json!({"title": "Wrong Title"});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::WindowTitleContains {
                substring: "Expected".into(),
            },
            required: false,
        }];
        let (status, _) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Ambiguous);
    }

    #[test]
    fn verify_ax_attribute_equals_verified() {
        let evidence = json!({"ax_attributes": {"AXRole": "AXButton"}});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::AXAttributeEquals {
                attribute: "AXRole".into(),
                value: "AXButton".into(),
            },
            required: true,
        }];
        let (status, results) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Verified);
        assert_eq!(results[0].confidence, Some(0.9));
    }

    #[test]
    fn verify_ax_attribute_equals_failed() {
        let evidence = json!({"ax_attributes": {"AXRole": "AXStaticText"}});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::AXAttributeEquals {
                attribute: "AXRole".into(),
                value: "AXButton".into(),
            },
            required: true,
        }];
        let (status, _) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Failed);
    }

    #[test]
    fn verify_ax_attribute_equals_missing_is_ambiguous() {
        let evidence = json!({});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::AXAttributeEquals {
                attribute: "AXRole".into(),
                value: "AXButton".into(),
            },
            required: true,
        }];
        let (status, _) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Ambiguous);
    }

    #[test]
    fn describe_ax_attribute_equals() {
        let expectation = GuiExpectation {
            kind: GuiExpectationKind::AXAttributeEquals {
                attribute: "AXTitle".into(),
                value: "Take Photo".into(),
            },
            required: true,
        };
        assert_eq!(
            describe_expectation(&expectation),
            "AX attribute 'AXTitle' equals 'Take Photo'"
        );
    }

    #[test]
    fn failure_cause_detects_login_wall() {
        let evidence = json!({"url": "https://example.com/login"});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::UrlIs {
                url: "https://example.com/success".into(),
            },
            required: true,
        }];
        let (status, results) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Failed);
        assert_eq!(results[0].failure_cause, FailureCause::EnvironmentBlocked);
    }

    #[test]
    fn failure_cause_detects_captcha() {
        let evidence = json!({"title": "Please verify you're human - reCAPTCHA"});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::WindowTitleContains {
                substring: "Dashboard".into(),
            },
            required: true,
        }];
        let (status, results) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Failed);
        assert_eq!(results[0].failure_cause, FailureCause::EnvironmentBlocked);
    }

    #[test]
    fn failure_cause_detects_timeout() {
        let evidence = json!({"url": "https://example.com/error?code=504"});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::UrlIs {
                url: "https://example.com/success".into(),
            },
            required: true,
        }];
        let (status, results) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Failed);
        assert_eq!(results[0].failure_cause, FailureCause::EnvironmentBlocked);
    }

    #[test]
    fn failure_cause_detects_out_of_stock() {
        let evidence = json!({"title": "Product Currently Unavailable"});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::WindowTitleContains {
                substring: "Add to Cart".into(),
            },
            required: true,
        }];
        let (status, results) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Failed);
        assert_eq!(results[0].failure_cause, FailureCause::EnvironmentBlocked);
    }

    #[test]
    fn failure_cause_detects_agent_error_wrong_selector() {
        let evidence = json!({"field_values": {"#email": "test@example.com"}});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::FieldValueEquals {
                selector: "#username".into(),
                value: "test@example.com".into(),
            },
            required: true,
        }];
        let (status, results) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Ambiguous);
        assert_eq!(results[0].failure_cause, FailureCause::EvidenceAbsent);
    }

    #[test]
    fn failure_cause_evidence_absent_when_null() {
        let evidence = json!({});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::UrlIs {
                url: "https://example.com/success".into(),
            },
            required: true,
        }];
        let (status, results) = verify_expectations(&exps, &evidence);
        assert_eq!(status, VerificationStatus::Ambiguous);
        assert_eq!(results[0].failure_cause, FailureCause::EvidenceAbsent);
    }

    #[test]
    fn process_score_high_for_environment_blocks() {
        let evidence = json!({"url": "https://example.com/login"});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::UrlIs {
                url: "https://example.com/success".into(),
            },
            required: true,
        }];
        let report = build_report(true, None, None, &exps, &evidence);
        assert_eq!(report.verification_status, VerificationStatus::Failed);
        assert_eq!(report.failure_cause, FailureCause::EnvironmentBlocked);
        assert_eq!(report.process_score, Some(0.8)); // High score - agent did right thing
        assert_eq!(report.outcome_success, Some(false));
    }

    #[test]
    fn process_score_low_for_agent_errors() {
        let evidence = json!({"field_values": {"#name": "Bob"}});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::FieldValueEquals {
                selector: "#name".into(),
                value: "Alice".into(),
            },
            required: true,
        }];
        let report = build_report(true, None, None, &exps, &evidence);
        assert_eq!(report.verification_status, VerificationStatus::Failed);
        assert_eq!(report.failure_cause, FailureCause::AgentError);
        assert_eq!(report.process_score, Some(0.0)); // Low score - agent made mistake
        assert_eq!(report.outcome_success, Some(false));
    }

    #[test]
    fn process_score_perfect_for_verified() {
        let evidence = json!({"url": "https://example.com/success"});
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::UrlIs {
                url: "https://example.com/success".into(),
            },
            required: true,
        }];
        let report = build_report(true, None, None, &exps, &evidence);
        assert_eq!(report.verification_status, VerificationStatus::Verified);
        assert_eq!(report.failure_cause, FailureCause::None);
        assert_eq!(report.process_score, Some(1.0));
        assert_eq!(report.outcome_success, Some(true));
    }

    #[test]
    fn describe_front_window_element_count_changed() {
        let expectation = GuiExpectation {
            kind: GuiExpectationKind::FrontWindowElementCountChanged {
                role: Some("AXImage".into()),
                title_contains: None,
                description_contains: Some("thumbnail".into()),
                value_contains: None,
                min_increase: 1,
            },
            required: true,
        };
        assert_eq!(
            describe_expectation(&expectation),
            "front window elements matching role 'AXImage', description containing 'thumbnail' increase by at least 1"
        );
    }

    #[test]
    fn classify_reversibility_click_at_partially_reversible() {
        let args = json!({"x": 100, "y": 200});
        assert_eq!(
            classify_reversibility("mac_automation", "click_at", &args, &[]),
            ReversibilityLevel::PartiallyReversible
        );
    }

    #[test]
    fn classify_reversibility_move_mouse_reversible() {
        let args = json!({"x": 100, "y": 200});
        assert_eq!(
            classify_reversibility("mac_automation", "move_mouse", &args, &[]),
            ReversibilityLevel::Reversible
        );
    }

    #[test]
    fn classify_reversibility_read_actions_reversible() {
        let args = json!({});
        assert_eq!(
            classify_reversibility("mac_automation", "read_focused_element", &args, &[]),
            ReversibilityLevel::Reversible
        );
        assert_eq!(
            classify_reversibility("mac_automation", "hit_test", &args, &[]),
            ReversibilityLevel::Reversible
        );
    }

    #[test]
    fn infer_evidence_keys_for_ax_attribute() {
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::AXAttributeEquals {
                attribute: "AXRole".into(),
                value: "AXButton".into(),
            },
            required: true,
        }];
        assert_eq!(
            infer_evidence_keys(&exps),
            vec!["ax_attributes.AXRole".to_string()]
        );
    }

    #[test]
    fn infer_evidence_keys_for_front_window_count_change() {
        let exps = vec![GuiExpectation {
            kind: GuiExpectationKind::FrontWindowElementCountChanged {
                role: Some("AXImage".into()),
                title_contains: None,
                description_contains: Some("thumbnail".into()),
                value_contains: None,
                min_increase: 1,
            },
            required: true,
        }];
        assert_eq!(
            infer_evidence_keys(&exps),
            vec![encode_front_window_match_count_key(
                Some("AXImage"),
                None,
                Some("thumbnail"),
                None
            )]
        );
    }
}
