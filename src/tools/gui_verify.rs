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
    ExpectationResult, GuiActionReport, GuiExpectation, GuiExpectationKind, GuiObservation,
    PreObservationStrategy, ReversibilityLevel, VerificationStatus, WaitStrategy,
};
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
    let mut results = Vec::with_capacity(expectations.len());

    for exp in expectations {
        let result = verify_single(&exp.kind, post_evidence);
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
        verify_expectations(expectations, post_evidence).1
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

    GuiActionReport {
        execution_ok,
        pre_observation,
        post_observation,
        verification_status,
        confidence: aggregate_confidence(&expectation_results),
        state_diff,
        expectation_results,
    }
}

/// Build a simple observation from a JSON evidence blob.
pub fn observation(source: &str, evidence: Value) -> GuiObservation {
    GuiObservation {
        evidence,
        source: source.to_string(),
    }
}

/// Determine which evidence keys to snapshot based on expectation kinds.
pub fn infer_pre_observation_keys(expectations: &[GuiExpectation]) -> Vec<String> {
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
            GuiExpectationKind::FileExists { .. }
            | GuiExpectationKind::DownloadCompleted { .. } => {}
        }
    }
    keys
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
        WaitStrategy::DomEvent { timeout_ms, .. }
        | WaitStrategy::AccessibilityEvent { timeout_ms, .. }
        | WaitStrategy::SelectorPresent { timeout_ms, .. } => {
            tokio::time::sleep(Duration::from_millis(*timeout_ms)).await;
            collect_evidence().await
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
            _ => ReversibilityLevel::Unknown,
        },
        _ => ReversibilityLevel::Unknown,
    }
}

// ── Per-expectation verification ────────────────────────────────

fn verify_single(kind: &GuiExpectationKind, evidence: &Value) -> ExpectationResult {
    let mut result = match kind {
        GuiExpectationKind::FieldValueEquals { selector, value } => {
            verify_field_value(selector, value, evidence)
        }
        GuiExpectationKind::FocusedElementIs { selector } => {
            verify_focused_element(selector, evidence)
        }
        GuiExpectationKind::CheckboxChecked { selector, checked } => {
            verify_checkbox(selector, *checked, evidence)
        }
        GuiExpectationKind::WindowTitleContains { substring } => {
            verify_window_title(substring, evidence)
        }
        GuiExpectationKind::DialogPresent { present } => verify_dialog_present(*present, evidence),
        GuiExpectationKind::UrlIs { url } => verify_url_is(url, evidence),
        GuiExpectationKind::UrlHostIs { host } => verify_url_host(host, evidence),
        GuiExpectationKind::FileExists { path } => verify_file_exists(path),
        GuiExpectationKind::DownloadCompleted { path } => verify_file_exists(path),
        GuiExpectationKind::ElementAtCoordinate {
            x,
            y,
            expected_element,
            tolerance_px,
        } => verify_element_at_coordinate(*x, *y, expected_element, *tolerance_px, evidence),
    };
    result.confidence = Some(confidence_for(kind, result.status));
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
            GuiExpectationKind::ElementAtCoordinate { .. } => 0.8,
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
                },
                ExpectationResult {
                    status: VerificationStatus::Verified,
                    expected: Value::Null,
                    actual: Value::Null,
                    message: None,
                    confidence: Some(0.9),
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
}
