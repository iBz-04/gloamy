# Failure Attribution Analysis: Connecting the Dots

## Executive Summary

This document analyzes what needs to be connected to implement **failure cause attribution** (Research Recommendation #2) in Gloamy's GUI verification system. The goal is to distinguish between:

- **Agent errors** (wrong action, bad reasoning, hallucination)
- **Environment blocks** (login walls, CAPTCHAs, out-of-stock, network failures)
- **Evidence absence** (can't verify because data is missing)
- **Hallucination suspicion** (agent claims success but evidence doesn't support it)

## Current Architecture: Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. Tool Execution (browser.rs, mac_automation.rs)              │
│    - Dispatches GUI action (click, fill, type, etc.)           │
│    - Collects pre/post observations                            │
│    - Calls gui_verify::build_report()                          │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. Verification Layer (gui_verify.rs)                          │
│    - Evaluates expectations against evidence                    │
│    - Returns GuiActionReport with:                             │
│      • execution_ok: bool                                       │
│      • verification_status: Verified/Failed/Ambiguous          │
│      • expectation_results: Vec<ExpectationResult>             │
│      • confidence: Option<f64>                                  │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. ToolResult Conversion (traits.rs)                           │
│    - ToolResult::from_gui_report()                             │
│    - Maps verification_status to success: bool                  │
│    - Serializes full report to output: String                   │
│    - Sets error: Option<String> for failures                    │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4. Agent Loop (loop_.rs)                                       │
│    - Receives ToolResult                                        │
│    - classify_tool_failure() parses error string               │
│    - Returns classification: &str                               │
│      • "gui_verification_failed"                                │
│      • "gui_element_not_found"                                  │
│      • "permission_or_policy"                                   │
│      • "transient_or_rate_limit"                                │
│      • "invalid_arguments"                                      │
│      • "missing_target"                                         │
│      • "validation_failed"                                      │
│      • "execution_error"                                        │
│    - recovery_hint_for_failure() generates guidance            │
└─────────────────────────────────────────────────────────────────┘
```

## The Gap: What's Missing

### Current State
- **Single verification_status enum**: `Verified | Failed | Ambiguous`
- **No failure attribution**: All failures look the same to the agent loop
- **String-based classification**: Agent loop parses error messages (brittle)
- **No process/outcome separation**: Can't distinguish "I did the right thing but was blocked" from "I did the wrong thing"

### What Research Says We Need
1. **Process score** (0.0–1.0): Did the agent execute correctly, regardless of outcome?
2. **Outcome success** (bool): Did the task actually complete?
3. **Failure cause** (enum): Why did it fail?
   - `AgentError`: Wrong selector, bad reasoning, incorrect action
   - `EnvironmentBlocked`: Login wall, CAPTCHA, out-of-stock, network timeout
   - `EvidenceAbsent`: Can't verify because structured data is missing
   - `HallucinationSuspected`: Agent claims success but evidence contradicts

## What Needs to Be Connected

### 1. Extend `ExpectationResult` (traits.rs)

**Current:**
```rust
pub struct ExpectationResult {
    pub status: VerificationStatus,
    pub expected: serde_json::Value,
    pub actual: serde_json::Value,
    pub message: Option<String>,
    pub confidence: Option<f64>,
}
```

**Proposed Addition:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureCause {
    /// Agent made a mistake (wrong selector, bad action, reasoning error)
    AgentError,
    /// Environment blocked the action (login wall, CAPTCHA, network failure)
    EnvironmentBlocked,
    /// Cannot verify because evidence is missing/incomplete
    EvidenceAbsent,
    /// Agent claimed success but evidence doesn't support it
    HallucinationSuspected,
    /// Not applicable (expectation verified or not evaluated)
    None,
}

pub struct ExpectationResult {
    pub status: VerificationStatus,
    pub expected: serde_json::Value,
    pub actual: serde_json::Value,
    pub message: Option<String>,
    pub confidence: Option<f64>,
    pub failure_cause: FailureCause,  // NEW
}
```

### 2. Extend `GuiActionReport` (traits.rs)

**Current:**
```rust
pub struct GuiActionReport {
    pub execution_ok: bool,
    pub pre_observation: Option<GuiObservation>,
    pub post_observation: Option<GuiObservation>,
    pub verification_status: VerificationStatus,
    pub expectation_results: Vec<ExpectationResult>,
    pub state_diff: Option<serde_json::Value>,
    pub confidence: Option<f64>,
}
```

**Proposed Addition:**
```rust
pub struct GuiActionReport {
    pub execution_ok: bool,
    pub pre_observation: Option<GuiObservation>,
    pub post_observation: Option<GuiObservation>,
    pub verification_status: VerificationStatus,
    pub expectation_results: Vec<ExpectationResult>,
    pub state_diff: Option<serde_json::Value>,
    pub confidence: Option<f64>,
    
    // NEW: Process/outcome separation
    pub process_score: Option<f64>,  // 0.0–1.0: Did agent execute correctly?
    pub outcome_success: Option<bool>,  // Did the task actually complete?
    pub failure_cause: FailureCause,  // Why did it fail?
}
```

### 3. Update Verification Logic (gui_verify.rs)

**Key Functions to Modify:**

#### `verify_single()` - Add failure cause inference
```rust
fn verify_single(
    kind: &GuiExpectationKind,
    pre_evidence: Option<&Value>,
    post_evidence: &Value,
) -> ExpectationResult {
    let mut result = match kind {
        // ... existing verification logic ...
    };
    
    result.confidence = Some(confidence_for(kind, result.status));
    result.failure_cause = infer_failure_cause(kind, &result, post_evidence);  // NEW
    result
}
```

#### New function: `infer_failure_cause()`
```rust
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
```

#### New function: `detect_environment_block()`
```rust
fn detect_environment_block(
    kind: &GuiExpectationKind,
    result: &ExpectationResult,
    post_evidence: &Value,
) -> FailureCause {
    // Check for common environment block patterns
    let actual_str = result.actual.as_str().unwrap_or("");
    let actual_lower = actual_str.to_ascii_lowercase();
    
    // Login/auth walls
    if actual_lower.contains("login")
        || actual_lower.contains("sign in")
        || actual_lower.contains("authenticate")
        || actual_lower.contains("unauthorized")
        || actual_lower.contains("403")
        || actual_lower.contains("401")
    {
        return FailureCause::EnvironmentBlocked;
    }
    
    // CAPTCHAs
    if actual_lower.contains("captcha")
        || actual_lower.contains("recaptcha")
        || actual_lower.contains("verify you're human")
    {
        return FailureCause::EnvironmentBlocked;
    }
    
    // Network/timeout
    if actual_lower.contains("timeout")
        || actual_lower.contains("network error")
        || actual_lower.contains("connection refused")
        || actual_lower.contains("503")
        || actual_lower.contains("504")
    {
        return FailureCause::EnvironmentBlocked;
    }
    
    // Out of stock / unavailable
    if actual_lower.contains("out of stock")
        || actual_lower.contains("unavailable")
        || actual_lower.contains("sold out")
        || actual_lower.contains("not available")
    {
        return FailureCause::EnvironmentBlocked;
    }
    
    // Check URL patterns for environment blocks
    if let GuiExpectationKind::UrlIs { url } | GuiExpectationKind::UrlHostIs { host: url } = kind {
        let expected_lower = url.to_ascii_lowercase();
        // If we expected a success page but got an error page
        if (expected_lower.contains("success")
            || expected_lower.contains("confirmation")
            || expected_lower.contains("complete"))
            && (actual_lower.contains("error")
                || actual_lower.contains("failed")
                || actual_lower.contains("denied"))
        {
            return FailureCause::EnvironmentBlocked;
        }
    }
    
    // Default: assume agent error (wrong selector, bad action)
    FailureCause::AgentError
}
```

#### `build_report()` - Compute process/outcome scores
```rust
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

    let verification_status = if !execution_ok && expectation_results.is_empty() {
        VerificationStatus::Failed
    } else {
        summarize_verification(expectations, &expectation_results)
    };

    let state_diff = match (&pre_observation, &post_observation) {
        (Some(pre), Some(post)) => compute_diff(pre, post),
        _ => None,
    };

    // NEW: Compute process/outcome scores
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
        process_score,      // NEW
        outcome_success,    // NEW
        failure_cause,      // NEW
    }
}
```

#### New function: `compute_process_outcome_scores()`
```rust
fn compute_process_outcome_scores(
    execution_ok: bool,
    expectation_results: &[ExpectationResult],
    verification_status: &VerificationStatus,
) -> (Option<f64>, Option<bool>, FailureCause) {
    if !execution_ok {
        // Execution failed - process score is 0, outcome is failure
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
    let mut total_failures = 0;

    for result in expectation_results {
        if result.status != VerificationStatus::Verified {
            total_failures += 1;
            match result.failure_cause {
                FailureCause::AgentError => agent_errors += 1,
                FailureCause::EnvironmentBlocked => environment_blocks += 1,
                FailureCause::EvidenceAbsent => evidence_absent += 1,
                _ => {}
            }
        }
    }

    // Process score: How well did the agent execute?
    // High if failures are environment blocks, low if agent errors
    let process_score = if total_failures == 0 {
        Some(1.0)
    } else if agent_errors == 0 {
        // All failures are environment blocks - agent did well
        Some(0.8)
    } else {
        // Some agent errors - penalize proportionally
        Some(1.0 - (agent_errors as f64 / expectation_results.len() as f64))
    };

    // Outcome success: Did the task complete?
    let outcome_success = Some(*verification_status == VerificationStatus::Verified);

    // Overall failure cause: prioritize by severity
    let failure_cause = if *verification_status == VerificationStatus::Verified {
        FailureCause::None
    } else if environment_blocks > 0 {
        FailureCause::EnvironmentBlocked
    } else if agent_errors > 0 {
        FailureCause::AgentError
    } else if evidence_absent > 0 {
        FailureCause::EvidenceAbsent
    } else {
        FailureCause::AgentError  // Default
    };

    (process_score, outcome_success, failure_cause)
}
```

### 4. Update Agent Loop Classification (loop_.rs)

**Current:**
```rust
fn classify_tool_failure(error: &str, tool_name: &str) -> &'static str {
    let lower = error.to_ascii_lowercase();
    // ... string parsing ...
    if lower.contains("gui verification") || lower.contains("verification_status") {
        return "gui_verification_failed";
    }
    // ...
}
```

**Proposed:**
```rust
fn classify_tool_failure(result: &ToolResult, tool_name: &str) -> &'static str {
    // Try to parse GuiActionReport from output
    if let Ok(report) = serde_json::from_str::<GuiActionReport>(&result.output) {
        return classify_gui_failure(&report);
    }
    
    // Fallback to string parsing for non-GUI tools
    let error = result.error.as_deref().unwrap_or("");
    let lower = error.to_ascii_lowercase();
    // ... existing string parsing ...
}

fn classify_gui_failure(report: &GuiActionReport) -> &'static str {
    match report.failure_cause {
        FailureCause::AgentError => "gui_agent_error",
        FailureCause::EnvironmentBlocked => "gui_environment_blocked",
        FailureCause::EvidenceAbsent => "gui_evidence_absent",
        FailureCause::HallucinationSuspected => "gui_hallucination_suspected",
        FailureCause::None => {
            if report.verification_status == VerificationStatus::Ambiguous {
                "gui_ambiguous"
            } else {
                "gui_verification_failed"
            }
        }
    }
}
```

### 5. Update Recovery Hints (loop_.rs)

```rust
fn recovery_hint_for_failure(classification: &str, tool_name: &str) -> String {
    match classification {
        "gui_agent_error" => format!(
            "{tool_name}: the action executed but verification failed due to incorrect parameters. \
            Inspect the expectation_results to see which expectations failed, then retry with corrected selectors/values."
        ),
        "gui_environment_blocked" => format!(
            "{tool_name}: the action was blocked by the environment (login wall, CAPTCHA, network error, or unavailable resource). \
            Do NOT retry the same action. Instead: \
            1) Take a screenshot to see what blocked the action, \
            2) Adjust your approach (e.g., handle login, skip unavailable items, wait for network), \
            3) Report the block to the user if it cannot be resolved."
        ),
        "gui_evidence_absent" => format!(
            "{tool_name}: the action executed but verification could not complete because evidence is missing. \
            This may indicate the backend didn't collect the required state. \
            Try using pre_observe:auto or explicit evidence keys, or switch to a different verification approach."
        ),
        "gui_hallucination_suspected" => format!(
            "{tool_name}: you claimed success but the evidence doesn't support it. \
            Re-examine the post-action state carefully and verify your assumptions."
        ),
        // ... existing classifications ...
    }
}
```

## Impact Analysis

### Files to Modify

1. **src/tools/traits.rs**
   - Add `FailureCause` enum
   - Extend `ExpectationResult` with `failure_cause` field
   - Extend `GuiActionReport` with `process_score`, `outcome_success`, `failure_cause`
   - Update `ToolResult::from_gui_report()` to preserve new fields

2. **src/tools/gui_verify.rs**
   - Add `infer_failure_cause()` function
   - Add `detect_environment_block()` function
   - Add `compute_process_outcome_scores()` function
   - Update `verify_single()` to set `failure_cause`
   - Update `build_report()` to compute process/outcome scores
   - Update `expectation_result()` helper to include `failure_cause`

3. **src/agent/loop_.rs**
   - Update `classify_tool_failure()` to parse `GuiActionReport`
   - Add `classify_gui_failure()` function
   - Update `recovery_hint_for_failure()` with new classifications

4. **Tests**
   - Add tests for `infer_failure_cause()`
   - Add tests for `detect_environment_block()`
   - Add tests for `compute_process_outcome_scores()`
   - Update existing tests to handle new fields

### Backward Compatibility

- **Serialization**: New fields are `Option<T>` or have defaults, so old reports deserialize correctly
- **Existing code**: `verification_status` is preserved, so existing consumers still work
- **Migration path**: Tools can gradually adopt the new fields; old tools continue to work

### Benefits

1. **Better agent decisions**: Agent loop can distinguish "retry with different params" from "give up, environment blocked"
2. **Improved learning**: Process scores let the agent know when it did the right thing but was blocked
3. **Reduced false penalties**: Agent isn't penalized for login walls or CAPTCHAs it can't control
4. **Clearer feedback**: Users see why actions failed (agent mistake vs. environment issue)
5. **Foundation for future work**: Enables trajectory-level verification, hallucination detection, LLM-judge fallback

## Next Steps

1. **Phase 1**: Implement `FailureCause` enum and extend structs (backward compatible)
2. **Phase 2**: Add failure cause inference in `gui_verify.rs`
3. **Phase 3**: Update agent loop to use structured failure causes
4. **Phase 4**: Add tests and validate against real scenarios
5. **Phase 5**: Extend to trajectory-level verification (future work)

## References

- Research: "The Art of Building Verifiers for Computer Use Agents" (Apr 2025)
- Current implementation: `src/tools/gui_verify.rs`, `src/tools/traits.rs`, `src/agent/loop_.rs`
- Related: `docs/hardware-peripherals-design.md` (similar trait-driven extension pattern)
