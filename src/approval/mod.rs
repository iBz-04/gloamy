//! Interactive approval workflow for supervised mode.
//!
//! Provides a pre-execution hook that prompts the user before tool calls,
//! with session-scoped "Always" allowlists and audit logging.

use crate::config::AutonomyConfig;
use crate::security::AutonomyLevel;
use crate::tools::traits::ReversibilityLevel;
use chrono::Utc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::{self, BufRead, IsTerminal, Write};
use std::process::Command;

// ── Types ────────────────────────────────────────────────────────

/// A request to approve a tool call before execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub tool_name: String,
    pub arguments: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gui_context: Option<GuiApprovalContext>,
}

/// GUI-specific metadata attached to an approval request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiApprovalContext {
    pub action_summary: String,
    pub reversibility: ReversibilityLevel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_state: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_outcome: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot_path: Option<String>,
}

/// The user's response to an approval request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalResponse {
    /// Execute this one call.
    Yes,
    /// Deny this call.
    No,
    /// Execute and add tool to session-scoped allowlist.
    Always,
}

/// A single audit log entry for an approval decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalLogEntry {
    pub timestamp: String,
    pub tool_name: String,
    pub arguments_summary: String,
    pub decision: ApprovalResponse,
    pub channel: String,
}

// ── ApprovalManager ──────────────────────────────────────────────

/// Manages the interactive approval workflow.
///
/// - Checks config-level `auto_approve` / `always_ask` lists
/// - Maintains a session-scoped "always" allowlist
/// - Records an audit trail of all decisions
pub struct ApprovalManager {
    /// Tools that never need approval (from config).
    auto_approve: HashSet<String>,
    /// Tools that always need approval, ignoring session allowlist.
    always_ask: HashSet<String>,
    /// Autonomy level from config.
    autonomy_level: AutonomyLevel,
    /// Session-scoped allowlist built from "Always" responses.
    session_allowlist: Mutex<HashSet<String>>,
    /// Audit trail of approval decisions.
    audit_log: Mutex<Vec<ApprovalLogEntry>>,
}

impl ApprovalManager {
    /// Create from autonomy config.
    pub fn from_config(config: &AutonomyConfig) -> Self {
        Self {
            auto_approve: config.auto_approve.iter().cloned().collect(),
            always_ask: config.always_ask.iter().cloned().collect(),
            autonomy_level: config.level,
            session_allowlist: Mutex::new(HashSet::new()),
            audit_log: Mutex::new(Vec::new()),
        }
    }

    /// Check whether a tool call requires interactive approval.
    ///
    /// Returns `true` if the call needs a prompt, `false` if it can proceed.
    pub fn needs_approval(&self, tool_name: &str) -> bool {
        // Full autonomy never prompts.
        if self.autonomy_level == AutonomyLevel::Full {
            return false;
        }

        // ReadOnly blocks everything — handled elsewhere; no prompt needed.
        if self.autonomy_level == AutonomyLevel::ReadOnly {
            return false;
        }

        // always_ask overrides everything.
        if self.always_ask.contains(tool_name) {
            return true;
        }

        // auto_approve skips the prompt.
        if self.auto_approve.contains(tool_name) {
            return false;
        }

        // Session allowlist (from prior "Always" responses).
        let allowlist = self.session_allowlist.lock();
        if allowlist.contains(tool_name) {
            return false;
        }

        // Default: supervised mode requires approval.
        true
    }

    /// Check whether a tool is already preapproved by config or session state.
    ///
    /// This ignores autonomy level so tool-specific gates can still enforce
    /// prompts when configured with `always`.
    pub fn has_preapproval(&self, tool_name: &str) -> bool {
        if self.always_ask.contains(tool_name) {
            return false;
        }

        if self.auto_approve.contains(tool_name) {
            return true;
        }

        self.session_allowlist.lock().contains(tool_name)
    }

    /// Check whether a GUI action is preapproved using scoped key
    /// (`tool_name::action_summary`).
    ///
    /// GUI "Always" approvals are scoped to the specific action context,
    /// not the entire tool, to prevent a blanket allowlist entry from
    /// suppressing approval on unrelated actions.
    pub fn has_gui_preapproval(&self, tool_name: &str, action_summary: &str) -> bool {
        if self.always_ask.contains(tool_name) {
            return false;
        }

        // GUI actions are never auto-approved by tool name alone.
        let scoped_key = scoped_allowlist_key(tool_name, action_summary);
        self.session_allowlist.lock().contains(&scoped_key)
    }

    /// Record an approval decision and update session state.
    ///
    /// For non-GUI tool calls, "Always" scopes to the tool name.
    /// For GUI tool calls, use [`record_gui_decision`] instead to scope
    /// the allowlist entry to the specific action.
    pub fn record_decision(
        &self,
        tool_name: &str,
        args: &serde_json::Value,
        decision: ApprovalResponse,
        channel: &str,
    ) {
        // If "Always", add to session allowlist.
        if decision == ApprovalResponse::Always {
            let mut allowlist = self.session_allowlist.lock();
            allowlist.insert(tool_name.to_string());
        }

        // Append to audit log.
        let summary = summarize_args(args);
        let entry = ApprovalLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            tool_name: tool_name.to_string(),
            arguments_summary: summary,
            decision,
            channel: channel.to_string(),
        };
        let mut log = self.audit_log.lock();
        log.push(entry);
    }

    /// Record a GUI approval decision with action-scoped "Always".
    ///
    /// The allowlist key is `tool_name::action_summary`, so "Always" only
    /// suppresses future prompts for the *same* action context (e.g.
    /// `browser::click #submit-payment`), not all calls to the tool.
    pub fn record_gui_decision(
        &self,
        tool_name: &str,
        action_summary: &str,
        args: &serde_json::Value,
        decision: ApprovalResponse,
        channel: &str,
    ) {
        if decision == ApprovalResponse::Always {
            let scoped_key = scoped_allowlist_key(tool_name, action_summary);
            let mut allowlist = self.session_allowlist.lock();
            allowlist.insert(scoped_key);
        }

        let summary = summarize_args(args);
        let entry = ApprovalLogEntry {
            timestamp: Utc::now().to_rfc3339(),
            tool_name: tool_name.to_string(),
            arguments_summary: summary,
            decision,
            channel: channel.to_string(),
        };
        let mut log = self.audit_log.lock();
        log.push(entry);
    }

    /// Get a snapshot of the audit log.
    pub fn audit_log(&self) -> Vec<ApprovalLogEntry> {
        self.audit_log.lock().clone()
    }

    /// Get the current session allowlist.
    pub fn session_allowlist(&self) -> HashSet<String> {
        self.session_allowlist.lock().clone()
    }

    /// Prompt the user on the CLI and return their decision.
    ///
    /// For non-CLI channels, returns `Yes` automatically (interactive
    /// approval is only supported on CLI for now).
    pub fn prompt_cli(&self, request: &ApprovalRequest) -> ApprovalResponse {
        prompt_cli_interactive(request)
    }

    /// Request approval for a GUI action, using CLI when attached to a terminal
    /// and a native dialog when running detached.
    ///
    /// "Always" responses are scoped to `tool_name::action_summary`.
    pub fn request_gui_approval(
        &self,
        request: &ApprovalRequest,
        timeout_secs: u64,
    ) -> ApprovalResponse {
        let stdin = io::stdin();
        let (decision, channel) = if stdin.is_terminal() {
            (prompt_cli_interactive(request), "cli")
        } else if let Some(decision) = prompt_native_gui(request, timeout_secs) {
            (decision, "gui")
        } else {
            (ApprovalResponse::No, "gui_unavailable")
        };

        let action_summary = request
            .gui_context
            .as_ref()
            .map(|ctx| ctx.action_summary.as_str())
            .unwrap_or("");
        self.record_gui_decision(
            &request.tool_name,
            action_summary,
            &request.arguments,
            decision,
            channel,
        );
        decision
    }
}

/// Build a scoped allowlist key for GUI actions: `tool_name::action_summary`.
fn scoped_allowlist_key(tool_name: &str, action_summary: &str) -> String {
    format!("{tool_name}::{action_summary}")
}

// ── CLI prompt ───────────────────────────────────────────────────

/// Display the approval prompt and read user input from stdin.
fn prompt_cli_interactive(request: &ApprovalRequest) -> ApprovalResponse {
    let summary = summarize_args(&request.arguments);
    eprintln!();
    if let Some(gui) = &request.gui_context {
        eprintln!("GUI approval required for: {}", request.tool_name);
        eprintln!("   Action: {}", gui.action_summary);
        eprintln!(
            "   Reversibility: {}",
            format_reversibility(gui.reversibility)
        );
        if !gui.expected_outcome.is_empty() {
            eprintln!("   Expected:");
            for outcome in &gui.expected_outcome {
                eprintln!("     - {outcome}");
            }
        }
        if let Some(current_state) = &gui.current_state {
            let rendered = summarize_current_state(current_state);
            if !rendered.is_empty() {
                eprintln!("   Current state: {rendered}");
            }
        }
        if let Some(path) = &gui.screenshot_path {
            eprintln!("   Screenshot: {path}");
        }
    } else {
        eprintln!("Agent wants to execute: {}", request.tool_name);
        eprintln!("   {summary}");
    }
    if let Some(gui) = &request.gui_context {
        eprint!("   [Y]es / [N]o / [A]lways for '{}': ", gui.action_summary);
    } else {
        eprint!("   [Y]es / [N]o / [A]lways for {}: ", request.tool_name);
    }
    let _ = io::stderr().flush();

    let stdin = io::stdin();
    let mut line = String::new();
    if stdin.lock().read_line(&mut line).is_err() {
        return ApprovalResponse::No;
    }

    match line.trim().to_ascii_lowercase().as_str() {
        "y" | "yes" => ApprovalResponse::Yes,
        "a" | "always" => ApprovalResponse::Always,
        _ => ApprovalResponse::No,
    }
}

/// Produce a short human-readable summary of tool arguments.
fn summarize_args(args: &serde_json::Value) -> String {
    match args {
        serde_json::Value::Object(map) => {
            let parts: Vec<String> = map
                .iter()
                .map(|(k, v)| {
                    let val = match v {
                        serde_json::Value::String(s) => truncate_for_summary(s, 80),
                        other => {
                            let s = other.to_string();
                            truncate_for_summary(&s, 80)
                        }
                    };
                    format!("{k}: {val}")
                })
                .collect();
            parts.join(", ")
        }
        other => {
            let s = other.to_string();
            truncate_for_summary(&s, 120)
        }
    }
}

fn truncate_for_summary(input: &str, max_chars: usize) -> String {
    let mut chars = input.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{truncated}…")
    } else {
        input.to_string()
    }
}

fn format_reversibility(level: ReversibilityLevel) -> &'static str {
    match level {
        ReversibilityLevel::Reversible => "reversible",
        ReversibilityLevel::PartiallyReversible => "partially_reversible",
        ReversibilityLevel::Irreversible => "irreversible",
        ReversibilityLevel::Unknown => "unknown",
    }
}

fn summarize_current_state(value: &serde_json::Value) -> String {
    let rendered = serde_json::to_string(value).unwrap_or_else(|_| value.to_string());
    truncate_for_summary(&rendered, 240)
}

fn build_native_gui_message(request: &ApprovalRequest) -> String {
    let mut parts = vec![format!("Tool: {}", request.tool_name)];

    if let Some(gui) = &request.gui_context {
        parts.push(format!("Action: {}", gui.action_summary));
        parts.push(format!(
            "Reversibility: {}",
            format_reversibility(gui.reversibility)
        ));
        if !gui.expected_outcome.is_empty() {
            parts.push(format!(
                "Expected: {}",
                truncate_for_summary(&gui.expected_outcome.join("; "), 240)
            ));
        }
        if let Some(current_state) = &gui.current_state {
            let rendered = summarize_current_state(current_state);
            if !rendered.is_empty() {
                parts.push(format!("Current state: {rendered}"));
            }
        }
    } else {
        parts.push(format!("Arguments: {}", summarize_args(&request.arguments)));
    }

    truncate_for_summary(&parts.join(". "), 900)
}

fn prompt_native_gui(request: &ApprovalRequest, timeout_secs: u64) -> Option<ApprovalResponse> {
    #[cfg(target_os = "macos")]
    {
        prompt_native_gui_macos(request, timeout_secs)
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (request, timeout_secs);
        None
    }
}

#[cfg(target_os = "macos")]
fn prompt_native_gui_macos(
    request: &ApprovalRequest,
    timeout_secs: u64,
) -> Option<ApprovalResponse> {
    let message = escape_applescript_string(&build_native_gui_message(request));
    let args = vec![
        "-e".to_string(),
        format!("set approval_text to \"{message}\""),
        "-e".to_string(),
        format!(
            "set approval_result to display dialog approval_text buttons {{\"Deny\", \"Approve\", \"Always\"}} default button \"Approve\" cancel button \"Deny\" with title \"Gloamy approval\" with icon caution{}",
            if timeout_secs == 0 {
                String::new()
            } else {
                format!(" giving up after {timeout_secs}")
            }
        ),
        "-e".to_string(),
        "set button_name to button returned of approval_result".to_string(),
        "-e".to_string(),
        "set gave_up to gave up of approval_result".to_string(),
        "-e".to_string(),
        "return button_name & \",gave_up:\" & (gave_up as string)".to_string(),
    ];

    let output = Command::new("osascript").args(&args).output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        if stdout.contains("gave_up:true") {
            return Some(ApprovalResponse::No);
        }
        if stdout.starts_with("Always") {
            return Some(ApprovalResponse::Always);
        }
        if stdout.starts_with("Approve") {
            return Some(ApprovalResponse::Yes);
        }
        return Some(ApprovalResponse::No);
    }

    if stderr.contains("User canceled") || stderr.contains("(-128)") {
        return Some(ApprovalResponse::No);
    }

    None
}

#[cfg(target_os = "macos")]
fn escape_applescript_string(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ")
        .replace('\r', " ")
}

// ── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AutonomyConfig;

    fn supervised_config() -> AutonomyConfig {
        AutonomyConfig {
            level: AutonomyLevel::Supervised,
            auto_approve: vec!["file_read".into(), "memory_recall".into()],
            always_ask: vec!["shell".into()],
            ..AutonomyConfig::default()
        }
    }

    fn full_config() -> AutonomyConfig {
        AutonomyConfig {
            level: AutonomyLevel::Full,
            ..AutonomyConfig::default()
        }
    }

    // ── needs_approval ───────────────────────────────────────

    #[test]
    fn auto_approve_tools_skip_prompt() {
        let mgr = ApprovalManager::from_config(&supervised_config());
        assert!(!mgr.needs_approval("file_read"));
        assert!(!mgr.needs_approval("memory_recall"));
    }

    #[test]
    fn always_ask_tools_always_prompt() {
        let mgr = ApprovalManager::from_config(&supervised_config());
        assert!(mgr.needs_approval("shell"));
    }

    #[test]
    fn unknown_tool_needs_approval_in_supervised() {
        let mgr = ApprovalManager::from_config(&supervised_config());
        assert!(mgr.needs_approval("file_write"));
        assert!(mgr.needs_approval("http_request"));
    }

    #[test]
    fn full_autonomy_never_prompts() {
        let mgr = ApprovalManager::from_config(&full_config());
        assert!(!mgr.needs_approval("shell"));
        assert!(!mgr.needs_approval("file_write"));
        assert!(!mgr.needs_approval("anything"));
    }

    #[test]
    fn readonly_never_prompts() {
        let config = AutonomyConfig {
            level: AutonomyLevel::ReadOnly,
            ..AutonomyConfig::default()
        };
        let mgr = ApprovalManager::from_config(&config);
        assert!(!mgr.needs_approval("shell"));
    }

    // ── session allowlist ────────────────────────────────────

    #[test]
    fn always_response_adds_to_session_allowlist() {
        let mgr = ApprovalManager::from_config(&supervised_config());
        assert!(mgr.needs_approval("file_write"));

        mgr.record_decision(
            "file_write",
            &serde_json::json!({"path": "test.txt"}),
            ApprovalResponse::Always,
            "cli",
        );

        // Now file_write should be in session allowlist.
        assert!(!mgr.needs_approval("file_write"));
    }

    #[test]
    fn always_ask_overrides_session_allowlist() {
        let mgr = ApprovalManager::from_config(&supervised_config());

        // Even after "Always" for shell, it should still prompt.
        mgr.record_decision(
            "shell",
            &serde_json::json!({"command": "ls"}),
            ApprovalResponse::Always,
            "cli",
        );

        // shell is in always_ask, so it still needs approval.
        assert!(mgr.needs_approval("shell"));
    }

    #[test]
    fn yes_response_does_not_add_to_allowlist() {
        let mgr = ApprovalManager::from_config(&supervised_config());
        mgr.record_decision(
            "file_write",
            &serde_json::json!({}),
            ApprovalResponse::Yes,
            "cli",
        );
        assert!(mgr.needs_approval("file_write"));
    }

    #[test]
    fn has_preapproval_reads_session_allowlist() {
        let mgr = ApprovalManager::from_config(&supervised_config());
        assert!(!mgr.has_preapproval("file_write"));

        mgr.record_decision(
            "file_write",
            &serde_json::json!({"path": "note.txt"}),
            ApprovalResponse::Always,
            "gui",
        );

        assert!(mgr.has_preapproval("file_write"));
    }

    // ── audit log ────────────────────────────────────────────

    #[test]
    fn audit_log_records_decisions() {
        let mgr = ApprovalManager::from_config(&supervised_config());

        mgr.record_decision(
            "shell",
            &serde_json::json!({"command": "rm -rf ./build/"}),
            ApprovalResponse::No,
            "cli",
        );
        mgr.record_decision(
            "file_write",
            &serde_json::json!({"path": "out.txt", "content": "hello"}),
            ApprovalResponse::Yes,
            "cli",
        );

        let log = mgr.audit_log();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].tool_name, "shell");
        assert_eq!(log[0].decision, ApprovalResponse::No);
        assert_eq!(log[1].tool_name, "file_write");
        assert_eq!(log[1].decision, ApprovalResponse::Yes);
    }

    #[test]
    fn audit_log_contains_timestamp_and_channel() {
        let mgr = ApprovalManager::from_config(&supervised_config());
        mgr.record_decision(
            "shell",
            &serde_json::json!({"command": "ls"}),
            ApprovalResponse::Yes,
            "telegram",
        );

        let log = mgr.audit_log();
        assert_eq!(log.len(), 1);
        assert!(!log[0].timestamp.is_empty());
        assert_eq!(log[0].channel, "telegram");
    }

    // ── summarize_args ───────────────────────────────────────

    #[test]
    fn summarize_args_object() {
        let args = serde_json::json!({"command": "ls -la", "cwd": "/tmp"});
        let summary = summarize_args(&args);
        assert!(summary.contains("command: ls -la"));
        assert!(summary.contains("cwd: /tmp"));
    }

    #[test]
    fn summarize_args_truncates_long_values() {
        let long_val = "x".repeat(200);
        let args = serde_json::json!({ "content": long_val });
        let summary = summarize_args(&args);
        assert!(summary.contains('…'));
        assert!(summary.len() < 200);
    }

    #[test]
    fn summarize_args_unicode_safe_truncation() {
        let long_val = "🦀".repeat(120);
        let args = serde_json::json!({ "content": long_val });
        let summary = summarize_args(&args);
        assert!(summary.contains("content:"));
        assert!(summary.contains('…'));
    }

    #[test]
    fn summarize_args_non_object() {
        let args = serde_json::json!("just a string");
        let summary = summarize_args(&args);
        assert!(summary.contains("just a string"));
    }

    // ── scoped GUI approval ──────────────────────────────────

    #[test]
    fn gui_always_scopes_to_action_summary() {
        let mgr = ApprovalManager::from_config(&supervised_config());
        assert!(!mgr.has_gui_preapproval("browser", "click #submit"));

        mgr.record_gui_decision(
            "browser",
            "click #submit",
            &serde_json::json!({}),
            ApprovalResponse::Always,
            "cli",
        );

        // Same action summary → preapproved.
        assert!(mgr.has_gui_preapproval("browser", "click #submit"));
        // Different action summary → NOT preapproved.
        assert!(!mgr.has_gui_preapproval("browser", "click #delete-account"));
        // Tool-level preapproval is NOT granted by GUI always.
        assert!(!mgr.has_preapproval("browser"));
    }

    #[test]
    fn gui_always_ask_overrides_gui_preapproval() {
        let mgr = ApprovalManager::from_config(&supervised_config());

        mgr.record_gui_decision(
            "shell",
            "run rm -rf",
            &serde_json::json!({}),
            ApprovalResponse::Always,
            "cli",
        );

        // shell is in always_ask → still not preapproved.
        assert!(!mgr.has_gui_preapproval("shell", "run rm -rf"));
    }

    #[test]
    fn non_gui_record_decision_still_tool_level() {
        let mgr = ApprovalManager::from_config(&supervised_config());
        mgr.record_decision(
            "file_write",
            &serde_json::json!({}),
            ApprovalResponse::Always,
            "cli",
        );

        // Tool-level preapproval is granted.
        assert!(mgr.has_preapproval("file_write"));
        assert!(!mgr.needs_approval("file_write"));
    }

    #[test]
    fn gui_decision_appears_in_audit_log() {
        let mgr = ApprovalManager::from_config(&supervised_config());
        mgr.record_gui_decision(
            "browser",
            "click #pay",
            &serde_json::json!({"action": "click", "selector": "#pay"}),
            ApprovalResponse::Yes,
            "gui",
        );

        let log = mgr.audit_log();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].tool_name, "browser");
        assert_eq!(log[0].decision, ApprovalResponse::Yes);
        assert_eq!(log[0].channel, "gui");
    }

    // ── ApprovalResponse serde ───────────────────────────────

    #[test]
    fn approval_response_serde_roundtrip() {
        let json = serde_json::to_string(&ApprovalResponse::Always).unwrap();
        assert_eq!(json, "\"always\"");
        let parsed: ApprovalResponse = serde_json::from_str("\"no\"").unwrap();
        assert_eq!(parsed, ApprovalResponse::No);
    }

    // ── ApprovalRequest ──────────────────────────────────────

    #[test]
    fn approval_request_serde() {
        let req = ApprovalRequest {
            tool_name: "shell".into(),
            arguments: serde_json::json!({"command": "echo hi"}),
            gui_context: Some(GuiApprovalContext {
                action_summary: "click #submit".into(),
                reversibility: ReversibilityLevel::Irreversible,
                current_state: Some(serde_json::json!({"title": "Checkout"})),
                expected_outcome: vec!["navigates to receipt page".into()],
                screenshot_path: None,
            }),
        };
        let json = serde_json::to_string(&req).unwrap();
        let parsed: ApprovalRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.tool_name, "shell");
        assert!(parsed.gui_context.is_some());
    }
}
