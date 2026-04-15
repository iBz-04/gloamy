# Failure Attribution Implementation Summary

## Overview

Successfully implemented **failure cause attribution** for GUI verification in Gloamy, enabling the agent to distinguish between agent errors and environment blocks. This is based on research from "The Art of Building Verifiers for Computer Use Agents" (Apr 2025).

## What Was Implemented

### 1. Core Data Structures (src/tools/traits.rs)

#### New `FailureCause` Enum
```rust
pub enum FailureCause {
    AgentError,              // Wrong selector, bad action, reasoning error
    EnvironmentBlocked,      // Login wall, CAPTCHA, network failure, out-of-stock
    EvidenceAbsent,          // Can't verify, data missing
    HallucinationSuspected,  // Claimed success without evidence
    None,                    // Not applicable
}
```

#### Extended `ExpectationResult`
- Added `failure_cause: FailureCause` field
- Automatically inferred during verification

#### Extended `GuiActionReport`
- Added `process_score: Option<f64>` - How well did the agent execute (0.0–1.0)?
- Added `outcome_success: Option<bool>` - Did the task actually complete?
- Added `failure_cause: FailureCause` - Why did it fail overall?

### 2. Failure Detection Logic (src/tools/gui_verify.rs)

#### `infer_failure_cause()`
Determines why a verification failed based on:
- Verification status (Verified/Failed/Ambiguous)
- Evidence presence/absence
- Content patterns in actual values

#### `detect_environment_block()`
Pattern-matches against common environment blocks:

**Authentication & Authorization:**
- login, sign in, authenticate, unauthorized, 401, 403
- session expired, session timeout, access denied

**CAPTCHA Challenges:**
- captcha, recaptcha, hcaptcha, cloudflare, turnstile
- verify you're human, bot detection, security check

**Network & Server Errors:**
- timeout, network error, connection refused/failed/reset
- 502, 503, 504, 429 (rate limit)
- service unavailable, gateway timeout

**Resource Unavailability:**
- out of stock, sold out, currently unavailable
- not available, temporarily unavailable, waitlist

**URL Pattern Analysis:**
- Expected success page but got error/login page

**Window Title Analysis:**
- Title contains login, error, access denied, captcha

#### `compute_process_outcome_scores()`
Separates process from outcome:
- **Process score = 1.0**: All expectations verified
- **Process score = 0.8**: Failures are environment blocks (agent did right thing)
- **Process score = 0.0–0.99**: Proportional to agent errors
- **Process score = 0.5**: Hallucination suspected
- **Outcome success**: Binary - did task complete?

### 3. Agent Loop Integration (src/agent/loop_.rs)

#### `classify_gui_failure()`
Uses structured `GuiActionReport` instead of string parsing:
```rust
match report.failure_cause {
    FailureCause::AgentError => "gui_agent_error",
    FailureCause::EnvironmentBlocked => "gui_environment_blocked",
    FailureCause::EvidenceAbsent => "gui_evidence_absent",
    FailureCause::HallucinationSuspected => "gui_hallucination_suspected",
    FailureCause::None => "gui_ambiguous" or "gui_verification_failed",
}
```

#### Updated `recovery_hint_for_failure()`
New classifications with specific guidance:

- **gui_agent_error**: "Inspect expectation_results, retry with corrected parameters"
- **gui_environment_blocked**: "Do NOT retry - take screenshot, adjust approach, or report to user"
- **gui_evidence_absent**: "Use pre_observe:auto or explicit evidence keys"
- **gui_hallucination_suspected**: "Re-examine state, verify assumptions"
- **gui_ambiguous**: "Add more specific expectations"

## Test Coverage

### Unit Tests (src/tools/gui_verify.rs)
- `failure_cause_detects_login_wall` ✓
- `failure_cause_detects_captcha` ✓
- `failure_cause_detects_timeout` ✓
- `failure_cause_detects_out_of_stock` ✓
- `failure_cause_detects_agent_error_wrong_selector` ✓
- `failure_cause_evidence_absent_when_null` ✓
- `process_score_high_for_environment_blocks` ✓
- `process_score_low_for_agent_errors` ✓
- `process_score_perfect_for_verified` ✓

### Integration Tests (tests/failure_attribution_integration_test.rs)
- `test_environment_block_login_wall` ✓
- `test_agent_error_wrong_selector` ✓
- `test_agent_error_wrong_value` ✓
- `test_captcha_detection` ✓
- `test_out_of_stock_detection` ✓
- `test_network_timeout_detection` ✓
- `test_successful_verification` ✓
- `test_mixed_failures_prioritizes_environment_blocks` ✓

**Total: 68 tests passing (60 unit + 8 integration)**

## Backward Compatibility

✅ **Fully backward compatible:**
- New fields are `Option<T>` or have `Default` implementations
- Existing `verification_status` is preserved
- Old code continues to work without changes
- Serialization/deserialization handles missing fields gracefully

## Example Usage

### Before (String Parsing)
```rust
if error.contains("gui verification") {
    return "gui_verification_failed";  // Can't tell why it failed
}
```

### After (Structured Attribution)
```rust
if let Ok(report) = serde_json::from_str::<GuiActionReport>(output) {
    match report.failure_cause {
        FailureCause::EnvironmentBlocked => {
            // Don't retry - environment blocked
            // Process score = 0.8 (agent did right thing)
        }
        FailureCause::AgentError => {
            // Retry with corrected parameters
            // Process score = 0.0 (agent made mistake)
        }
        _ => { /* ... */ }
    }
}
```

## Benefits

1. **Better Agent Decisions**: Agent loop can distinguish "retry with different params" from "give up, environment blocked"

2. **Improved Learning**: Process scores let the agent know when it did the right thing but was blocked

3. **Reduced False Penalties**: Agent isn't penalized for login walls or CAPTCHAs it can't control

4. **Clearer Feedback**: Users see why actions failed (agent mistake vs. environment issue)

5. **Foundation for Future Work**: Enables:
   - Trajectory-level verification (multi-frame evidence)
   - Hallucination detection (double-pass with/without evidence)
   - LLM-judge fallback (VLM scoring when structured evidence is absent)
   - Backend state verification (actual side effects, not just UI state)

## Research Alignment

This implementation addresses **Recommendation #2** from the research:
> "Add failure_cause attribution — Extend ExpectationResult with a FailureCause enum: AgentError, EnvironmentBlocked, EvidenceAbsent, HallucinationSuspected. This lets the agent loop know whether to retry vs. give up vs. ask for help."

The implementation follows the research findings that:
- Process and outcome rewards are fundamentally different signals
- Controllable vs. uncontrollable failure attribution is critical
- State-of-the-art verifiers have ~30% false positive rates without proper attribution
- Environment blocks (login walls, CAPTCHAs) should not penalize the agent

## Files Modified

1. **src/tools/traits.rs** - Added `FailureCause` enum, extended structs
2. **src/tools/gui_verify.rs** - Added failure detection logic
3. **src/tools/mod.rs** - Exported `FailureCause`
4. **src/agent/loop_.rs** - Updated classification and recovery hints
5. **tests/failure_attribution_integration_test.rs** - New integration tests

## Next Steps (Future Work)

Based on the research analysis document, the next high-impact improvements would be:

1. **Conditional Criteria** (Medium effort, high correctness gains)
   - Add `condition: Option<String>` to `GuiExpectation`
   - Support "if X then check Y" logic
   - Mark expectations as `NotApplicable` when condition not met

2. **Trajectory-Level Verification** (Medium effort, high correctness gains)
   - Accumulate `Vec<GuiObservation>` across steps
   - Verify against any frame in the trajectory
   - Detect "was there a success dialog anywhere in the last N screenshots?"

3. **LLM-Judge Fallback** (Medium effort, reduces false negatives)
   - When structured evidence is missing, enqueue VLM call
   - Produce soft confidence score from screenshot
   - Significantly reduces false-negative rate for complex UI

4. **Execution-Based Side-Effect Verification** (Longer term)
   - Verify actual file changes, process exits, database state
   - Strongest signal for complex multi-step tasks
   - OSWorld approach: inspect filesystem, app config, database

## References

- Research: "The Art of Building Verifiers for Computer Use Agents" (Apr 2025)
- Analysis: `docs/failure-attribution-analysis.md`
- AGENTS.md: §7.3 (Tool extension playbook)
- Related: `docs/hardware-peripherals-design.md` (trait-driven extension pattern)
