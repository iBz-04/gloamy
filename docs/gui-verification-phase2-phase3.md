# GUI Verification Layer — Phase 2 & Phase 3 Implementation Plan

> Builds on Phase 1 (`gui_verify.rs`, `traits.rs` GUI types, browser/mac_automation wiring).
> Date: 2026-04-03
>
> Status note (2026-04-03): this document is now a mixed implementation record and forward plan.
> Phase 2 is mostly shipped. Phase 3 core approval gates are shipped, while eval framework and
> channel-specific approval UX are still pending.
>
> Update (2026-04-03):
> - **DomEvent wait** is now backend-native in `browser.rs` (agent-browser JS eval, computer-use sidecar `wait_event`, rust-native WebDriver `execute_async`). No longer a stub sleep.
> - **AccessibilityEvent wait** is now backend-native in `mac_automation.rs` (polling-based osascript observation for `AXTitleChanged`, `AXFocusedUIElementChanged`, `AXSheetCreated`, `AXWindowCreated`, `AXValueChanged`). No longer a stub sleep.
> - **SelectorPresent wait** was already backend-native (browser `wait` action / computer-use sidecar).
> - **Approval "Always" scoping** narrowed from tool-level to `tool_name::action_summary` for GUI actions (`has_gui_preapproval`, `record_gui_decision`). Non-GUI approvals retain tool-level scoping.
> - Remaining unimplemented: eval framework (§3.3), channel-specific approval UX (Telegram/Discord/Gateway).

---

## Phase 2: Pre-observation, Event-Driven Waits, Coordinate Hit-Testing, Confidence Scoring

### Overview

Original Phase 1 gaps this plan addressed:

- **No pre-observation** — `pre_observation` was always `None`, so we could not diff before and after state.
- **No intelligent waiting** — verification ran immediately after the action, so async UI transitions could be missed.
- **No coordinate validation** — computer-use mouse actions targeted (x, y) with no verification that the intended element occupied that region.
- **No confidence scoring** — results were tri-state (`Verified`/`Failed`/`Ambiguous`) with no granularity.

Phase 2 addresses all four.

---

### Step 2.1: Pre-Observation Capture

**Goal:** Snapshot relevant UI state *before* the action executes so the report can include a meaningful diff.

#### 2.1.1 — Add `PreObservationStrategy` enum to `traits.rs`

```rust
/// How to capture pre-action state for diffing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreObservationStrategy {
    /// No pre-observation (Phase 1 behavior, default).
    None,
    /// Capture the same evidence keys that the expectations will check.
    /// The gui_verify module infers which keys to snapshot from the expectation kinds.
    Auto,
    /// Caller provides explicit evidence keys to snapshot.
    Explicit { keys: Vec<String> },
}
```

- Default: `None` (backward-compatible).
- `Auto` inspects expectation kinds → derives evidence keys (e.g., `FieldValueEquals` → read `field_values.<selector>`, `UrlIs` → read `url`, `WindowTitleContains` → read `title`).
- `Explicit` lets callers request arbitrary keys.

#### 2.1.2 — Add `pre_observe` param to `GuiExpectation` (or top-level `expect` block)

Extend the `expect` JSON contract:

```json
{
  "expect": {
    "pre_observe": "auto",
    "expectations": [
      { "kind": "url_is", "url": "https://example.com/done" }
    ]
  }
}
```

Alternatively, keep `pre_observe` as a sibling key in the tool args (simpler, avoids nesting):

```json
{
  "action": "click",
  "selector": "#submit",
  "pre_observe": "auto",
  "expect": [{ "kind": "url_is", "url": "https://example.com/done" }]
}
```

**Decision:** Use the sibling-key approach — it avoids breaking the existing `expect` array/object contract.

#### 2.1.3 — Implement `capture_pre_observation()` in `gui_verify.rs`

```rust
/// Determine which evidence keys to snapshot based on expectation kinds.
pub fn infer_pre_observation_keys(expectations: &[GuiExpectation]) -> Vec<String> {
    let mut keys = Vec::new();
    for exp in expectations {
        match &exp.kind {
            GuiExpectationKind::FieldValueEquals { selector, .. } => {
                keys.push(format!("field_values.{selector}"));
            }
            GuiExpectationKind::UrlIs { .. } | GuiExpectationKind::UrlHostIs { .. } => {
                keys.push("url".into());
            }
            GuiExpectationKind::WindowTitleContains { .. } => {
                keys.push("title".into());
            }
            GuiExpectationKind::FocusedElementIs { .. } => {
                keys.push("focused_element".into());
            }
            GuiExpectationKind::CheckboxChecked { selector, .. } => {
                keys.push(format!("checkbox_states.{selector}"));
            }
            GuiExpectationKind::DialogPresent { .. } => {
                keys.push("dialog_present".into());
            }
            // FileExists / DownloadCompleted don't need UI pre-observation
            _ => {}
        }
    }
    keys.dedup();
    keys
}
```

#### 2.1.4 — Wire pre-observation into browser.rs and mac_automation.rs

**Browser tool (`execute` method):**

1. Parse `pre_observe` from args (default: `None`).
2. If `Auto` or `Explicit`, call backend-specific evidence collection *before* dispatching the action.
   - **agent-browser backend:** Run `agent-browser get url`, `get title`, etc. for the inferred keys.
   - **computer-use backend:** POST a `query_state` action to the sidecar with requested keys.
   - **rust-native backend:** Use WebDriver queries (`current_url()`, `title()`, element value reads).
3. Wrap the result as `GuiObservation { source: "dom_pre", evidence }`.
4. Pass into `build_report(..., Some(pre_obs), ...)`.

**mac_automation tool (`execute` method):**

1. Parse `pre_observe`.
2. If `Auto`, run a lightweight AppleScript to read the relevant state:
   - `WindowTitleContains` → `tell application "System Events" to get name of first window of (first process whose frontmost is true)`
   - `DialogPresent` → check for sheet/dialog existence via accessibility.
3. Pass as pre-observation.

#### 2.1.5 — Add `diff` field to `GuiActionReport`

```rust
pub struct GuiActionReport {
    // ... existing fields ...
    /// Optional diff between pre and post observations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_diff: Option<serde_json::Value>,
}
```

Compute in `build_report()` when both pre and post observations are present:

```rust
fn compute_diff(pre: &GuiObservation, post: &GuiObservation) -> serde_json::Value {
    // JSON-level diff: for each key in pre.evidence,
    // emit { key: { "before": pre_val, "after": post_val } } when values differ.
}
```

#### 2.1.6 — Tests for pre-observation

- `pre_observation_auto_infers_url_key` — `UrlIs` expectation → inferred keys contain `"url"`.
- `pre_observation_explicit_overrides_auto` — `Explicit { keys: ["title"] }` ignores expectation-inferred keys.
- `state_diff_computed_when_both_present` — pre has `url: A`, post has `url: B` → diff shows change.
- `state_diff_none_when_pre_absent` — backward compat: no diff when `pre_observe` is `None`.

---

### Step 2.2: Event-Driven Waits (Smart Settling)

**Goal:** Instead of verifying immediately after action dispatch, wait for the UI to settle using platform-native signals.

#### 2.2.1 — Add `WaitStrategy` to the verification contract

```rust
/// How to wait for the UI to settle before collecting post-observation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaitStrategy {
    /// No wait (Phase 1 behavior).
    None,
    /// Fixed delay in milliseconds.
    FixedMs(u64),
    /// Wait for a DOM event (browser only).
    DomEvent {
        /// JS event name: "load", "networkidle", "domcontentloaded", custom.
        event: String,
        /// Max wait ms before falling back.
        timeout_ms: u64,
    },
    /// Wait for macOS accessibility notification (mac_automation only).
    AccessibilityEvent {
        /// AX notification: "AXWindowCreated", "AXFocusedUIElementChanged",
        /// "AXValueChanged", "AXTitleChanged", "AXSheetCreated", etc.
        notification: String,
        /// Max wait ms.
        timeout_ms: u64,
    },
    /// Wait until a CSS selector becomes visible/present (browser only).
    SelectorPresent {
        selector: String,
        timeout_ms: u64,
    },
    /// Wait until post-observation evidence satisfies all required expectations.
    /// Polls at `poll_interval_ms` up to `timeout_ms`.
    PollUntilVerified {
        poll_interval_ms: u64,
        timeout_ms: u64,
    },
}
```

Default: `None`. Parsed from `"wait"` key in tool args alongside `"expect"`.

#### 2.2.2 — Implement browser-side wait strategies

**agent-browser backend:**

- `DomEvent` → `agent-browser wait --event <event> --timeout <ms>`
  (If agent-browser doesn't support `--event`, fall back to: inject JS `await new Promise(r => window.addEventListener('<event>', r, {once:true}))` via `agent-browser eval`.)
- `SelectorPresent` → `agent-browser wait <selector> --timeout <ms>` (already supported).
- `PollUntilVerified` → loop: collect evidence → verify → if `Verified` break, else sleep `poll_interval_ms`, until timeout.

**computer-use backend:**

- `DomEvent` / `SelectorPresent` → POST to sidecar with `{ action: "wait", params: { ... } }`.
- `PollUntilVerified` → Gloamy-side polling loop (same as agent-browser).

**rust-native backend:**

- `DomEvent` → WebDriver `execute_async_script` with event listener.
- `SelectorPresent` → WebDriver explicit wait (`WebDriverWait::until(element_present(selector))`).
- `PollUntilVerified` → same Gloamy-side loop.

#### 2.2.3 — Implement macOS accessibility event waits

New helper in `mac_automation.rs` (or a shared `gui_wait.rs` module):

```rust
/// Wait for a macOS accessibility notification using a short-lived observer.
///
/// Implementation: Run a small Swift/ObjC helper or AppleScript that:
/// 1. Registers an AXObserver on the frontmost app's AXUIElement.
/// 2. Listens for the specified AXNotification.
/// 3. Exits with 0 when received, or 1 on timeout.
///
/// This avoids blocking the Tokio runtime — spawn as a child process with timeout.
async fn wait_for_ax_notification(
    notification: &str,
    timeout_ms: u64,
) -> anyhow::Result<bool> { ... }
```

**Concrete approach — osascript + System Events:**

For common cases (`AXTitleChanged`, `AXFocusedUIElementChanged`, `AXValueChanged`), use a polling AppleScript:

```applescript
-- Poll window title every 200ms until it changes or timeout
set startTitle to name of front window of application "System Events"
set deadline to (current date) + «timeout_secs»
repeat while (current date) < deadline
    delay 0.2
    set newTitle to name of front window of application "System Events"
    if newTitle ≠ startTitle then return newTitle
end repeat
error "timeout"
```

For more advanced cases (sheet creation, dialog appearance), use a compiled Swift CLI helper:

- File: `scripts/ax_wait` (small Swift CLI, ~80 lines).
- Registers `AXObserverAddNotification` for the target notification.
- Exits with JSON `{ "received": true, "element_description": "..." }` or `{ "received": false, "reason": "timeout" }`.
- Built during `bootstrap.sh` on macOS (optional; graceful degradation to polling if missing).

#### 2.2.4 — Implement `apply_wait_strategy()` in `gui_verify.rs`

```rust
/// Apply the configured wait strategy before collecting post-observation.
///
/// Returns the final post-action evidence blob.
pub async fn apply_wait_strategy(
    strategy: &WaitStrategy,
    collect_evidence: impl Fn() -> Pin<Box<dyn Future<Output = anyhow::Result<Value>> + Send>>,
    expectations: &[GuiExpectation],
) -> anyhow::Result<Value> {
    match strategy {
        WaitStrategy::None => collect_evidence().await,
        WaitStrategy::FixedMs(ms) => {
            tokio::time::sleep(Duration::from_millis(*ms)).await;
            collect_evidence().await
        }
        WaitStrategy::PollUntilVerified { poll_interval_ms, timeout_ms } => {
            let deadline = Instant::now() + Duration::from_millis(*timeout_ms);
            loop {
                let evidence = collect_evidence().await?;
                let (status, _) = verify_expectations(expectations, &evidence);
                if status == VerificationStatus::Verified || Instant::now() >= deadline {
                    return Ok(evidence);
                }
                tokio::time::sleep(Duration::from_millis(*poll_interval_ms)).await;
            }
        }
        // DomEvent, AccessibilityEvent, and SelectorPresent must be handled
        // by backend-native wait paths before calling this helper.
        // If they reach this function, return an explicit error.
        _ => anyhow::bail!("unsupported wait strategy for runtime wait helper"),
    }
}
```

#### 2.2.5 — Tests for wait strategies

- `wait_none_returns_immediately` — no delay, evidence returned as-is.
- `wait_fixed_ms_delays` — measure elapsed time ≥ specified ms.
- `wait_poll_until_verified_succeeds_on_second_poll` — mock evidence provider returns wrong value first, correct second → verified.
- `wait_poll_until_verified_times_out` — evidence never matches → returns last evidence at deadline.

---

### Step 2.3: Coordinate Hit-Testing

**Goal:** Before executing a mouse action at (x, y), verify that the intended target element actually occupies that coordinate region.

#### 2.3.1 — Add `CoordinateExpectation` to expectation kinds

```rust
pub enum GuiExpectationKind {
    // ... existing kinds ...

    /// Verify that the element at (x, y) matches an expected identity.
    /// Used before mouse_click / mouse_move / mouse_drag actions.
    ElementAtCoordinate {
        x: i64,
        y: i64,
        /// Expected element role, label, or selector substring.
        expected_element: String,
        /// Tolerance in pixels (element bounding box must contain (x, y) ± tolerance).
        #[serde(default)]
        tolerance_px: u32,
    },
}
```

#### 2.3.2 — Implement hit-test evidence collection

**Browser (computer-use sidecar):**

Add a `hit_test` action to the sidecar protocol:

```json
{ "action": "hit_test", "params": { "x": 450, "y": 300 } }
```

Response:

```json
{
  "element": {
    "tag": "button",
    "role": "button",
    "label": "Submit",
    "selector": "#submit-btn",
    "bounding_box": { "x": 420, "y": 280, "width": 100, "height": 40 }
  }
}
```

Implementation in the sidecar: `document.elementFromPoint(x, y)` + compute bounding rect + extract ARIA attributes.

**macOS (mac_automation):**

Use the Accessibility API:

```applescript
tell application "System Events"
    set frontApp to first process whose frontmost is true
    set elem to UI element at position {x, y} of frontApp
    return {role: elem's role, description: elem's description, title: elem's title}
end tell
```

Or via the Swift `ax_wait` helper extended with a `hit_test` subcommand:

```
ax_wait hit-test --x 450 --y 300
```

Which calls `AXUIElementCopyElementAtPosition()` and returns JSON.

#### 2.3.3 — Implement verification in `gui_verify.rs`

```rust
fn verify_element_at_coordinate(
    x: i64, y: i64,
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
            let description = elem.get("description").and_then(Value::as_str).unwrap_or("");

            let matches = [label, role, selector, description]
                .iter()
                .any(|field| field.contains(expected));

            // Also verify coordinate is within bounding box ± tolerance
            let bbox_ok = if let Some(bb) = elem.get("bounding_box") {
                let bx = bb.get("x").and_then(Value::as_i64).unwrap_or(0);
                let by = bb.get("y").and_then(Value::as_i64).unwrap_or(0);
                let bw = bb.get("width").and_then(Value::as_i64).unwrap_or(0);
                let bh = bb.get("height").and_then(Value::as_i64).unwrap_or(0);
                let tol = tolerance as i64;
                x >= bx - tol && x <= bx + bw + tol &&
                y >= by - tol && y <= by + bh + tol
            } else {
                true // no bbox info, trust the element match
            };

            if matches && bbox_ok {
                ExpectationResult { status: VerificationStatus::Verified, ... }
            } else {
                ExpectationResult { status: VerificationStatus::Failed, ... }
            }
        }
        None => ExpectationResult { status: VerificationStatus::Ambiguous, ... }
    }
}
```

#### 2.3.4 — Wire hit-testing as a **pre-check** for coordinate actions

In `browser.rs`, for `mouse_click` / `mouse_move` / `mouse_drag` actions:

1. If any expectation has kind `ElementAtCoordinate`, run the hit-test query **before** dispatching the click.
2. If hit-test fails → return `Failed` immediately **without executing the click** (safety gate).
3. If hit-test succeeds → proceed with the action → then run normal post-observation verification.

This is a **pre-check**, not a post-check. It prevents clicking the wrong element.

#### 2.3.5 — Tests

- `hit_test_verified_when_element_matches` — mock evidence with matching label.
- `hit_test_failed_when_wrong_element` — element at coordinate doesn't match expected.
- `hit_test_blocks_action_on_failure` — full integration: execute returns `Failed` without dispatching the click.
- `hit_test_tolerance_accepts_near_boundary` — coordinate 2px outside bbox, tolerance=5 → verified.

---

### Step 2.4: Confidence Scoring

**Goal:** Replace the flat tri-state with a `[0.0, 1.0]` confidence score while keeping `VerificationStatus` as the primary API.

#### 2.4.1 — Add confidence field to `ExpectationResult`

```rust
pub struct ExpectationResult {
    pub status: VerificationStatus,
    pub expected: serde_json::Value,
    pub actual: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Confidence score in [0.0, 1.0]. `None` when not applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}
```

#### 2.4.2 — Add aggregate confidence to `GuiActionReport`

```rust
pub struct GuiActionReport {
    // ... existing fields ...
    /// Aggregate confidence across all expectations. `None` if no scoring was performed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}
```

#### 2.4.3 — Scoring rules (per expectation kind)

| Kind | Verified | Failed | Ambiguous |
|------|----------|--------|-----------|
| Exact match (FieldValueEquals, UrlIs, CheckboxChecked) | 1.0 | 0.0 | 0.3 |
| Substring match (WindowTitleContains) | 0.95 (substring may be coincidental) | 0.0 | 0.3 |
| Host match (UrlHostIs) | 0.9 (correct host, path not checked) | 0.0 | 0.3 |
| File existence (FileExists, DownloadCompleted) | 0.85 (file exists, content not verified) | 0.0 | 0.3 |
| Element at coordinate (hit-test) | 0.8 (visual heuristic) | 0.05 (element may be obscured) | 0.3 |
| Dialog present | 0.9 | 0.0 | 0.3 |
| Focused element | 0.95 | 0.0 | 0.3 |

**Aggregate:** Geometric mean of per-expectation confidence scores (rewards all-high, penalizes any-low).

#### 2.4.4 — Integrate confidence into `ToolResult::from_gui_report()`

No change to `success` logic (still driven by `VerificationStatus`). Confidence is informational metadata for the LLM and observability consumers.

#### 2.4.5 — Tests

- `confidence_exact_match_is_1` — FieldValueEquals verified → 1.0.
- `confidence_ambiguous_is_low` — missing evidence → 0.3.
- `confidence_aggregate_geometric_mean` — two expectations: 1.0 and 0.9 → ~0.949.
- `confidence_serialization_omits_none` — no confidence when Phase 1 mode.

---

### Phase 2 File Change Summary

| File | Changes | Status |
|------|---------|--------|
| `src/tools/traits.rs` | `PreObservationStrategy`, `WaitStrategy`, `ElementAtCoordinate`, confidence fields, `state_diff` | Shipped |
| `src/tools/gui_verify.rs` | `infer_evidence_keys()`, runtime wait helper, coordinate verification, diffing, confidence scoring | Shipped (event waits require backend support) |
| `src/tools/browser.rs` | Pre-observation capture, selector wait hook, coordinate hit-test pre-check, GUI report wiring | Shipped |
| `src/tools/mac_automation.rs` | Pre-observation capture and GUI report wiring | Shipped (AX event waits pending) |
| `src/tools/gui_wait.rs` (new) | Shared wait helpers | Pending / not added |
| `scripts/ax_wait` (new, macOS only) | AXObserver-based event waits and hit-testing helper | Pending / not added |

---

## Phase 3: Approval Gates for Irreversible Actions & OSWorld-Style Evals

### Overview

Phase 3 adds two capabilities:

1. **Irreversible action detection + approval gates** — classify GUI actions by reversibility, inject approval prompts for destructive actions (form submission, file deletion, account changes), and block execution until approved.
2. **Execution-based evaluation framework** — OSWorld-style benchmarks that define a task, execute it via the agent, and verify success through the full verification pipeline.

---

### Step 3.1: Action Reversibility Classification

#### 3.1.1 — Add `ReversibilityLevel` enum

In `traits.rs`:

```rust
/// How reversible a GUI action is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReversibilityLevel {
    /// Fully reversible (e.g., typing in a field, navigating, scrolling).
    Reversible,
    /// Partially reversible (e.g., form submission that can be undone server-side).
    PartiallyReversible,
    /// Irreversible (e.g., payment, account deletion, sending a message).
    Irreversible,
    /// Unknown (caller should treat as potentially irreversible).
    Unknown,
}
```

#### 3.1.2 — Implement `classify_reversibility()` in `gui_verify.rs`

Static heuristic classifier based on action type + context signals:

```rust
pub fn classify_reversibility(
    tool_name: &str,
    action: &str,
    args: &Value,
    expectations: &[GuiExpectation],
) -> ReversibilityLevel {
    // Browser actions
    if tool_name == "browser" {
        return match action {
            "open" | "snapshot" | "get_text" | "get_title" | "get_url"
            | "screenshot" | "is_visible" | "hover" | "scroll"
            | "screen_capture" => ReversibilityLevel::Reversible,

            "click" => classify_click_reversibility(args, expectations),
            "fill" | "type" => ReversibilityLevel::Reversible, // text input is undoable
            "press" => classify_keypress_reversibility(args),
            "close" => ReversibilityLevel::PartiallyReversible,

            // Computer-use coordinate actions
            "mouse_click" | "mouse_drag" => ReversibilityLevel::Unknown,
            "key_type" | "key_press" => classify_keypress_reversibility(args),

            _ => ReversibilityLevel::Unknown,
        };
    }

    // mac_automation actions
    if tool_name == "mac_automation" {
        return match action {
            "launch_app" | "activate_app" => ReversibilityLevel::Reversible,
            "run_applescript" => classify_applescript_reversibility(args),
            _ => ReversibilityLevel::Unknown,
        };
    }

    ReversibilityLevel::Unknown
}

fn classify_click_reversibility(args: &Value, expectations: &[GuiExpectation]) -> ReversibilityLevel {
    let selector = args.get("selector").and_then(Value::as_str).unwrap_or("");

    // Heuristic: submit buttons, delete buttons, payment buttons are risky
    let irreversible_patterns = [
        "submit", "delete", "remove", "pay", "purchase", "send",
        "confirm", "finalize", "checkout", "destroy", "deactivate",
    ];
    let sel_lower = selector.to_lowercase();
    if irreversible_patterns.iter().any(|p| sel_lower.contains(p)) {
        return ReversibilityLevel::Irreversible;
    }

    // If expectations include URL change to a "success/confirmation" page, likely irreversible
    for exp in expectations {
        if let GuiExpectationKind::UrlIs { url } = &exp.kind {
            let url_lower = url.to_lowercase();
            if url_lower.contains("success") || url_lower.contains("confirmation")
                || url_lower.contains("thank") || url_lower.contains("receipt")
            {
                return ReversibilityLevel::Irreversible;
            }
        }
    }

    ReversibilityLevel::PartiallyReversible
}
```

#### 3.1.3 — Allow caller override via `"reversibility"` key

```json
{
  "action": "click",
  "selector": "#custom-btn",
  "reversibility": "irreversible",
  "expect": [...]
}
```

Caller-provided value overrides the heuristic. This is important because heuristics cannot be perfect — the LLM often knows the intent.

---

### Step 3.2: Approval Gate Integration

#### 3.2.1 — Add `GuiApprovalPolicy` configuration

In `config/schema.rs`, under a new `[gui_verification]` section:

```toml
[gui_verification]
# Whether to require approval for irreversible GUI actions.
# "always" | "supervised_only" | "never"
approval_gate = "supervised_only"

# Actions above this reversibility level trigger approval.
# "irreversible" (default) | "partially_reversible" | "unknown"
approval_threshold = "irreversible"

# Timeout for waiting on approval (seconds). 0 = block indefinitely.
approval_timeout_secs = 120

# Channel-specific overrides (e.g., CLI always prompts, Telegram uses inline keyboard).
# [gui_verification.channel_overrides.telegram]
# approval_gate = "always"
```

#### 3.2.2 — Extend `ApprovalRequest` for GUI context

In `approval/mod.rs`, add GUI-specific metadata:

```rust
pub struct ApprovalRequest {
    pub tool_name: String,
    pub arguments: serde_json::Value,
    /// GUI-specific: what the action will do and why it's flagged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gui_context: Option<GuiApprovalContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiApprovalContext {
    /// Human-readable description of the action.
    pub action_summary: String,
    /// Classified reversibility level.
    pub reversibility: ReversibilityLevel,
    /// Pre-observation state (what the UI looks like now).
    pub current_state: Option<serde_json::Value>,
    /// What the agent expects to happen.
    pub expected_outcome: Vec<String>,
    /// Optional screenshot path for visual confirmation.
    pub screenshot_path: Option<String>,
}
```

#### 3.2.3 — Implement the approval gate in tool execution flow

In `browser.rs` and `mac_automation.rs`, after parsing expectations and classifying reversibility, but **before** executing the action:

```rust
// Pseudocode for the gate:
let reversibility = gui_verify::classify_reversibility(
    self.name(), action, &args, &expectations
);

// Check if approval is needed
if needs_gui_approval(&config.gui_verification, &security, reversibility) {
    // Capture pre-screenshot for approval context
    let screenshot = if config.gui_verification.include_screenshot_in_approval {
        Some(capture_approval_screenshot().await?)
    } else {
        None
    };

    let approval_ctx = GuiApprovalContext {
        action_summary: format!("{} on {}", action, selector_or_target),
        reversibility,
        current_state: pre_observation.map(|o| o.evidence.clone()),
        expected_outcome: expectations.iter().map(|e| describe_expectation(e)).collect(),
        screenshot_path: screenshot,
    };

    let request = ApprovalRequest {
        tool_name: self.name().into(),
        arguments: args.clone(),
        gui_context: Some(approval_ctx),
    };

    // This call blocks until user responds (Yes/No/Always)
    let response = approval_manager.request_approval(request, channel).await?;

    match response {
        ApprovalResponse::No => {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Action blocked: user denied approval for irreversible GUI action".into()),
            });
        }
        ApprovalResponse::Yes | ApprovalResponse::Always => {
            // Proceed with execution
        }
    }
}
```

#### 3.2.4 — Channel-specific approval UX

**CLI channel:** Use the existing `prompt_cli_interactive()` with extended display:

```
⚠️  IRREVERSIBLE GUI ACTION

  Tool:    browser
  Action:  click #submit-payment
  Expects: URL changes to https://shop.example.com/receipt

  Current state:
    URL: https://shop.example.com/checkout
    Title: "Checkout — My Shop"

  [Y]es / [N]o / [A]lways for browser: _
```

**Telegram/Discord/Slack channels:** Send an inline keyboard/button message:

```
⚠️ Irreversible GUI Action

The agent wants to click #submit-payment on shop.example.com.
Expected outcome: navigates to receipt page.

[✅ Approve]  [❌ Deny]
```

Implementation: Add a `request_gui_approval()` method to the `Channel` trait (or use the existing message-and-wait pattern used by pairing).

**Gateway API:** Expose `POST /api/approval/respond` so the desktop app can render a native dialog.

#### 3.2.5 — Approval audit trail

Every GUI approval decision is logged to the existing `ApprovalManager.audit_log` with the additional `gui_context` field. The `AuditLogger` in `security/audit.rs` should also emit an `AuditEvent` for irreversible action approvals.

#### 3.2.6 — Tests

- `irreversible_click_triggers_approval` — click on `#delete-account` → approval requested.
- `reversible_action_skips_approval` — `open` action → no approval.
- `caller_override_irreversible_triggers_approval` — even benign selector, but `"reversibility": "irreversible"` → prompted.
- `approval_denied_blocks_execution` — simulate `No` response → `success: false`.
- `approval_always_adds_to_session` — after `Always`, subsequent same-tool calls skip prompt.
- `full_autonomy_skips_gui_approval` — `AutonomyLevel::Full` → no prompt even for irreversible.

---

### Step 3.3: OSWorld-Style Execution-Based Evaluation Framework

**Goal:** A test harness that defines GUI tasks with initial state, agent action sequences, and verification criteria — then runs them end-to-end and scores the results.

#### 3.3.1 — Eval task definition format

TOML-based task definitions in `tests/gui_evals/`:

```toml
# tests/gui_evals/fill_form_and_submit.toml

[task]
id = "fill_form_submit_001"
name = "Fill registration form and submit"
description = "Navigate to registration page, fill name/email, submit, verify success page"
category = "form_interaction"
difficulty = "easy"

[setup]
# Initial state: URL to navigate to before the task starts
start_url = "http://localhost:3000/register"
# Optional: pre-task commands to reset state
reset_commands = ["curl -X POST http://localhost:3000/api/reset-test-data"]

[[steps]]
tool = "browser"
action = "fill"
args = { selector = "#name", value = "Test User" }
expect = [{ kind = "field_value_equals", selector = "#name", value = "Test User" }]
wait = { strategy = "none" }

[[steps]]
tool = "browser"
action = "fill"
args = { selector = "#email", value = "test@example.com" }
expect = [{ kind = "field_value_equals", selector = "#email", value = "test@example.com" }]

[[steps]]
tool = "browser"
action = "click"
args = { selector = "#submit" }
expect = [
    { kind = "url_is", url = "http://localhost:3000/welcome" },
    { kind = "window_title_contains", substring = "Welcome" },
]
wait = { strategy = "poll_until_verified", poll_interval_ms = 200, timeout_ms = 5000 }
reversibility = "irreversible"

[scoring]
# All steps must pass for full score
method = "all_or_nothing"
# Alternative: "weighted" with per-step weights
# method = "weighted"
# weights = [0.2, 0.2, 0.6]
```

#### 3.3.2 — Eval runner implementation

New module: `src/tools/gui_eval.rs` (or `tests/gui_eval_runner.rs` if test-only):

```rust
pub struct GuiEvalRunner {
    tools: Vec<Arc<dyn Tool>>,
    config: GuiEvalConfig,
}

pub struct GuiEvalConfig {
    /// Directory containing .toml task definitions.
    pub tasks_dir: PathBuf,
    /// Whether to capture screenshots at each step for the report.
    pub capture_screenshots: bool,
    /// Whether to run approval gates (usually disabled in eval mode).
    pub skip_approval_gates: bool,
    /// Timeout per step.
    pub step_timeout_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct GuiEvalResult {
    pub task_id: String,
    pub task_name: String,
    pub total_steps: usize,
    pub passed_steps: usize,
    pub failed_steps: usize,
    pub ambiguous_steps: usize,
    pub overall_score: f64,
    pub overall_confidence: Option<f64>,
    pub step_results: Vec<GuiEvalStepResult>,
    pub duration_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct GuiEvalStepResult {
    pub step_index: usize,
    pub tool: String,
    pub action: String,
    pub verification_status: VerificationStatus,
    pub confidence: Option<f64>,
    pub report: GuiActionReport,
    pub duration_ms: u64,
    pub screenshot_path: Option<String>,
}
```

Runner flow:

1. Load task TOML → parse into `GuiEvalTask`.
2. Run `setup.reset_commands` (if any).
3. Navigate to `setup.start_url`.
4. For each step:
   a. Build tool args from step definition.
   b. Execute via the tool registry (same path as agent execution).
   c. Collect `GuiActionReport` from the result.
   d. Optionally capture screenshot.
   e. Record step result.
5. Compute overall score per `scoring.method`.
6. Emit `GuiEvalResult` as JSON.

#### 3.3.3 — CLI command for running evals

Add a `gui-eval` subcommand to `main.rs`:

```
gloamy gui-eval [--tasks-dir ./tests/gui_evals] [--filter "form_*"] [--report-format json|markdown]
```

This command:
- Loads config (needs browser config for tool construction).
- Builds tool registry.
- Instantiates `GuiEvalRunner`.
- Runs matching tasks.
- Outputs results to stdout or a report file.

#### 3.3.4 — Scoring methods

**all_or_nothing:** Score = 1.0 if all steps `Verified`, else 0.0.

**weighted:** Score = Σ(weight_i × step_score_i) where step_score = 1.0 if `Verified`, 0.5 if `Ambiguous`, 0.0 if `Failed`.

**partial_credit:** Score = passed_steps / total_steps.

**confidence_weighted:** Score = Σ(confidence_i) / N — uses Phase 2 confidence scores.

#### 3.3.5 — Example eval tasks to ship

| Task ID | Category | Description |
|---------|----------|-------------|
| `nav_001` | navigation | Open URL, verify title and URL |
| `form_001` | form_interaction | Fill a text field, verify value |
| `form_002` | form_interaction | Fill and submit a form, verify success page |
| `click_001` | click_interaction | Click a button, verify URL change |
| `dialog_001` | dialog_interaction | Trigger a dialog, verify it appears |
| `download_001` | file_interaction | Click download link, verify file exists |
| `mac_001` | macos_automation | Launch app, verify window title |
| `mac_002` | macos_automation | Run AppleScript, verify output |
| `coord_001` | coordinate_action | Mouse click at coordinate, verify element hit-test |

#### 3.3.6 — Integration with CI

Add a `gui-eval` CI job (optional, runs on macOS runners with a test web server):

```yaml
# .github/workflows/gui-eval.yml
name: GUI Eval Suite
on:
  schedule:
    - cron: '0 6 * * 1'  # Weekly Monday 6am UTC
  workflow_dispatch:

jobs:
  eval:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - name: Start test server
        run: cd tests/gui_evals/test_server && npm start &
      - name: Run GUI evals
        run: cargo run -- gui-eval --tasks-dir tests/gui_evals --report-format json > eval_results.json
      - name: Upload results
        uses: actions/upload-artifact@v4
        with:
          name: gui-eval-results
          path: eval_results.json
```

#### 3.3.7 — Tests for the eval framework itself

- `eval_task_toml_parsing` — load a sample TOML, verify struct fields.
- `eval_runner_all_or_nothing_scoring` — all steps pass → 1.0, one fails → 0.0.
- `eval_runner_weighted_scoring` — partial pass with weights → correct score.
- `eval_runner_step_timeout` — step exceeds timeout → `Failed` with timeout message.
- `eval_result_serialization` — `GuiEvalResult` round-trips through JSON.

---

### Phase 3 File Change Summary

| File | Changes | Status |
|------|---------|--------|
| `src/tools/traits.rs` | `ReversibilityLevel` enum | Shipped |
| `src/tools/gui_verify.rs` | Reversibility classification helpers, expectation descriptions, approval-threshold gate checks | Shipped |
| `src/tools/browser.rs` | GUI approval gate before execution | Shipped |
| `src/tools/mac_automation.rs` | GUI approval gate before execution | Shipped |
| `src/approval/mod.rs` | `GuiApprovalContext`, `ApprovalRequest` extension, GUI approval request path | Shipped |
| `src/config/schema.rs` | `[gui_verification]` config section | Shipped |
| `src/tools/gui_eval.rs` (new) | Eval runner, task parser, scoring | Pending / not added |
| `src/main.rs` (or `src/lib.rs`) | `gui-eval` CLI subcommand | Pending / not added |
| `tests/gui_evals/` (new dir) | TOML task definitions + optional test server | Pending / not added |
| `.github/workflows/gui-eval.yml` (new) | Optional CI job | Pending / not added |

---

## Implementation Order (Remaining Work)

| Order | Item | Depends On | Risk |
|-------|------|------------|------|
| 1 | 2.4 Confidence scoring | None (additive fields) | Low |
| 2 | 2.1 Pre-observation capture | None | Low |
| 3 | 2.2 Event-driven waits (FixedMs + PollUntilVerified first) | 2.1 (for poll loop evidence) | Medium |
| 4 | 2.2 Event-driven waits (DomEvent, AccessibilityEvent) | Platform helpers | Medium |
| 5 | 2.3 Coordinate hit-testing | Sidecar protocol change | Medium |
| 6 | 3.1 Reversibility classification | None | Low |
| 7 | 3.2 Approval gates | 3.1 + approval module | High |
| 8 | 3.3 Eval framework (task format + runner) | 2.1–2.4 for full scoring | Medium |
| 9 | 3.3 Eval CI integration | 3.3 runner + test server | Low |

Each step is independently shippable as a single PR.
