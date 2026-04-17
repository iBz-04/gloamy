use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::hooks::traits::{HookHandler, HookResult};

/// Blocks `mac_automation` calls when a loaded `automating-*` skill covers the
/// target app. Forces the model to use the skill (JXA/osascript via shell)
/// instead of the slow, fragile GUI automation path.
///
/// Matches both the explicit `app_name` argument (for `launch_app` /
/// `activate_app`) and any `tell application "X"` clause inside a
/// `run_applescript` script. Aliases for common macOS-visible app names
/// (e.g. "Microsoft PowerPoint" → `automating-powerpoint`, "Google Chrome"
/// → `automating-chrome`) are resolved so the match is not defeated by the
/// way Apple/Microsoft actually brand their binaries.
pub struct SkillFirstHook {
    /// Lowercased human-readable app key → `automating-<suffix>` skill name.
    app_to_skill: HashMap<String, String>,
}

impl SkillFirstHook {
    /// Build from loaded skill names. Any skill whose name starts with
    /// `automating-` is assumed to cover the macOS app derived from the
    /// suffix. We register both the canonical suffix key (e.g. "powerpoint")
    /// and a curated set of aliases reflecting how macOS actually names the
    /// app in launch services / AX / AppleScript targets.
    pub fn from_skill_names(skill_names: &[String]) -> Self {
        let mut app_to_skill = HashMap::new();
        for name in skill_names {
            let Some(suffix) = name.strip_prefix("automating-") else {
                continue;
            };

            // Canonical: "voice-memos" → "voice memos".
            let canonical = suffix.replace('-', " ");
            app_to_skill.insert(canonical.clone(), name.clone());

            for alias in canonical_aliases(suffix) {
                app_to_skill.insert(alias.to_string(), name.clone());
            }
        }
        Self { app_to_skill }
    }

    /// Check whether a given app name (as sent by the model in the
    /// `mac_automation` arguments) is covered by a loaded skill.
    fn skill_for_app(&self, app_name: &str) -> Option<&str> {
        let normalised = normalize_app_token(app_name);
        if normalised.is_empty() {
            return None;
        }
        self.app_to_skill.get(&normalised).map(String::as_str)
    }

    /// Inspect a `run_applescript` payload and return the first targeted app
    /// (from a `tell application "X"` clause) that maps to a loaded skill.
    fn skill_for_run_applescript(&self, args: &Value) -> Option<(String, String)> {
        let script = extract_run_applescript_text(args);
        if script.is_empty() {
            return None;
        }
        for target in extract_tell_application_targets(&script) {
            if let Some(skill) = self.skill_for_app(&target) {
                return Some((target, skill.to_string()));
            }
        }
        None
    }
}

/// Collapse whitespace and common app-name noise into a lowercase match token.
///
/// Handles ".app" suffixes (from `open -a "Foo.app"`), trims the "Microsoft"
/// / "Google" / "Apple" vendor prefixes that split the canonical suffix from
/// the user-visible name, and squashes runs of whitespace.
fn normalize_app_token(raw: &str) -> String {
    let mut s = raw.trim().to_ascii_lowercase();
    if let Some(stripped) = s.strip_suffix(".app") {
        s = stripped.trim().to_string();
    }
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Extract the raw AppleScript body from a `run_applescript` `args` blob,
/// tolerating both `script` (single string) and `script_lines` (array) forms
/// used by `mac_automation::run_applescript`.
fn extract_run_applescript_text(args: &Value) -> String {
    if let Some(script) = args.get("script").and_then(Value::as_str) {
        return script.to_string();
    }
    if let Some(lines) = args.get("script_lines").and_then(Value::as_array) {
        return lines
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join("\n");
    }
    String::new()
}

/// Pull every `tell application "X"` target out of an AppleScript. Returns
/// lowercase tokens already normalized for `app_to_skill` lookup. Also
/// tolerates `tell app "X"` (common shorthand) and curly/smart quotes.
fn extract_tell_application_targets(script: &str) -> Vec<String> {
    let lower = script.to_ascii_lowercase();
    let mut targets = Vec::new();
    for (prefix_match, _) in lower.match_indices("tell application") {
        let tail = &lower[prefix_match + "tell application".len()..];
        // Accept optional trailing " id " form — we only want the quoted name.
        if let Some(name) = extract_first_quoted_token(tail) {
            targets.push(name);
        }
    }
    // Also catch the 3-letter shorthand `tell app "X"`.
    for (prefix_match, _) in lower.match_indices("tell app ") {
        // Skip if this is actually `tell application` we already captured.
        let after = &lower[prefix_match + "tell app".len()..];
        if after.trim_start().starts_with("lication") {
            continue;
        }
        if let Some(name) = extract_first_quoted_token(after) {
            targets.push(name);
        }
    }
    targets
}

/// Return the first substring bounded by a pair of ASCII/curly double quotes.
fn extract_first_quoted_token(haystack: &str) -> Option<String> {
    let bytes = haystack.as_bytes();
    // Valid opening quotes: ", “, ”
    let open_positions = ['"', '\u{201C}', '\u{201D}'];
    let open_idx = haystack
        .char_indices()
        .find(|(_, ch)| open_positions.contains(ch))?
        .0;
    let after_open = &haystack[open_idx + haystack[open_idx..].chars().next()?.len_utf8()..];
    let close_idx_rel = after_open
        .char_indices()
        .find(|(_, ch)| open_positions.contains(ch))?
        .0;
    let name = &after_open[..close_idx_rel];
    let normalized = normalize_app_token(name);
    if normalized.is_empty() {
        return None;
    }
    // Paranoia: guard against accidentally matching non-ASCII via raw bytes.
    let _ = bytes; // keep `bytes` binding to silence unused warnings in some configs
    Some(normalized)
}

/// Curated aliases for common macOS-visible app names that don't match the
/// bare `automating-<suffix>` token. Keep entries lowercase.
fn canonical_aliases(suffix: &str) -> &'static [&'static str] {
    match suffix {
        "powerpoint" => &["microsoft powerpoint"],
        "word" => &["microsoft word"],
        "excel" => &["microsoft excel"],
        "chrome" => &["google chrome"],
        "mail" => &["apple mail"],
        "messages" => &["imessage"],
        "calendar" => &["ical"],
        "contacts" => &["address book"],
        "voice-memos" => &["voice memo"], // occasional singular slip
        "mac-apps" => &[], // aggregator skill — no single app
        _ => &[],
    }
}

#[async_trait]
impl HookHandler for SkillFirstHook {
    fn name(&self) -> &str {
        "skill-first"
    }

    fn priority(&self) -> i32 {
        100 // Run before most other hooks.
    }

    async fn before_tool_call(&self, name: String, args: Value) -> HookResult<(String, Value)> {
        if !name.eq_ignore_ascii_case("mac_automation") {
            return HookResult::Continue((name, args));
        }

        let action = args
            .get("action")
            .and_then(Value::as_str)
            .unwrap_or("");

        // 1) Direct app_name argument (launch_app, activate_app, click_at
        // when supplied with an explicit target).
        let explicit_app = args.get("app_name").and_then(Value::as_str).unwrap_or("");
        if let Some(skill) = self.skill_for_app(explicit_app) {
            let msg = format!(
                "Blocked by skill-first policy: the `{skill}` skill is loaded and covers \
                 {explicit_app}. Use the skill's scripting approach (JXA/osascript via the \
                 shell tool) instead of mac_automation action={action}. Read the skill \
                 instructions already in your system prompt under <available_skills> for the \
                 exact commands."
            );
            tracing::info!(
                hook = "skill-first",
                app = explicit_app,
                skill,
                action,
                source = "app_name",
                "redirecting to skill"
            );
            return HookResult::Cancel(msg);
        }

        // 2) run_applescript payload that contains `tell application "X"` for
        // an app with a loaded skill. The skill's own scripts are usually
        // invoked via the shell tool, not via mac_automation, so catching
        // this here forces the model back onto the documented path.
        if action.eq_ignore_ascii_case("run_applescript") {
            if let Some((app, skill)) = self.skill_for_run_applescript(&args) {
                let msg = format!(
                    "Blocked by skill-first policy: the `{skill}` skill is loaded and \
                     covers {app}. Do not freehand an AppleScript via mac_automation — \
                     invoke the skill's documented commands through the shell tool. Read \
                     the skill instructions already in your system prompt under \
                     <available_skills>."
                );
                tracing::info!(
                    hook = "skill-first",
                    app = app,
                    skill,
                    action,
                    source = "run_applescript",
                    "redirecting to skill"
                );
                return HookResult::Cancel(msg);
            }
        }

        HookResult::Continue((name, args))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn blocks_mac_automation_when_skill_exists() {
        let hook = SkillFirstHook::from_skill_names(&[
            "automating-notes".into(),
            "automating-reminders".into(),
        ]);

        let args = serde_json::json!({"action": "launch_app", "app_name": "Notes"});
        let result = hook
            .before_tool_call("mac_automation".into(), args)
            .await;

        assert!(result.is_cancel());
    }

    #[tokio::test]
    async fn allows_mac_automation_when_no_skill() {
        let hook = SkillFirstHook::from_skill_names(&["automating-notes".into()]);

        let args = serde_json::json!({"action": "launch_app", "app_name": "Preview"});
        let result = hook
            .before_tool_call("mac_automation".into(), args)
            .await;

        assert!(!result.is_cancel());
    }

    #[tokio::test]
    async fn ignores_non_mac_automation_tools() {
        let hook = SkillFirstHook::from_skill_names(&["automating-notes".into()]);

        let args = serde_json::json!({"cmd": "ls"});
        let result = hook.before_tool_call("shell".into(), args).await;

        assert!(!result.is_cancel());
    }

    #[tokio::test]
    async fn case_insensitive_app_match() {
        let hook = SkillFirstHook::from_skill_names(&["automating-voice-memos".into()]);

        let args = serde_json::json!({"action": "activate_app", "app_name": "Voice Memos"});
        let result = hook
            .before_tool_call("mac_automation".into(), args)
            .await;

        assert!(result.is_cancel());
    }

    #[tokio::test]
    async fn blocks_microsoft_powerpoint_alias() {
        // Regression: the skill name suffix is "powerpoint" but macOS reports
        // the app as "Microsoft PowerPoint". Without alias handling the model
        // slips past skill-first and drives the GUI by hand.
        let hook = SkillFirstHook::from_skill_names(&["automating-powerpoint".into()]);
        let args = serde_json::json!({
            "action": "activate_app",
            "app_name": "Microsoft PowerPoint"
        });
        let result = hook
            .before_tool_call("mac_automation".into(), args)
            .await;
        assert!(
            result.is_cancel(),
            "Microsoft PowerPoint should route to automating-powerpoint"
        );
    }

    #[tokio::test]
    async fn blocks_google_chrome_alias() {
        let hook = SkillFirstHook::from_skill_names(&["automating-chrome".into()]);
        let args = serde_json::json!({
            "action": "activate_app",
            "app_name": "Google Chrome"
        });
        let result = hook
            .before_tool_call("mac_automation".into(), args)
            .await;
        assert!(result.is_cancel());
    }

    #[tokio::test]
    async fn blocks_app_with_dot_app_suffix() {
        // `open -a "Microsoft Word.app"` style normalization.
        let hook = SkillFirstHook::from_skill_names(&["automating-word".into()]);
        let args = serde_json::json!({
            "action": "launch_app",
            "app_name": "Microsoft Word.app"
        });
        let result = hook
            .before_tool_call("mac_automation".into(), args)
            .await;
        assert!(result.is_cancel());
    }

    #[tokio::test]
    async fn run_applescript_tell_application_is_blocked() {
        // A freehand `run_applescript` targeting a skilled app should be
        // blocked so the model goes through the skill's documented commands
        // (usually invoked via the shell tool), not via ad-hoc mac_automation.
        let hook = SkillFirstHook::from_skill_names(&["automating-notes".into()]);
        let args = serde_json::json!({
            "action": "run_applescript",
            "script": "tell application \"Notes\"\n  make new note\nend tell"
        });
        let result = hook
            .before_tool_call("mac_automation".into(), args)
            .await;
        assert!(
            result.is_cancel(),
            "run_applescript targeting Notes should route to automating-notes"
        );
    }

    #[tokio::test]
    async fn run_applescript_script_lines_are_inspected() {
        let hook = SkillFirstHook::from_skill_names(&["automating-messages".into()]);
        let args = serde_json::json!({
            "action": "run_applescript",
            "script_lines": [
                "tell application \"Messages\"",
                "  send \"hi\" to buddy \"x\"",
                "end tell"
            ]
        });
        let result = hook
            .before_tool_call("mac_automation".into(), args)
            .await;
        assert!(result.is_cancel());
    }

    #[tokio::test]
    async fn run_applescript_for_unskilled_app_passes() {
        // `Preview` has no automating-* skill, so the script must be allowed.
        let hook = SkillFirstHook::from_skill_names(&["automating-notes".into()]);
        let args = serde_json::json!({
            "action": "run_applescript",
            "script": "tell application \"Preview\" to activate"
        });
        let result = hook
            .before_tool_call("mac_automation".into(), args)
            .await;
        assert!(!result.is_cancel());
    }

    #[tokio::test]
    async fn empty_script_passes_through() {
        let hook = SkillFirstHook::from_skill_names(&["automating-notes".into()]);
        let args = serde_json::json!({"action": "run_applescript"});
        let result = hook
            .before_tool_call("mac_automation".into(), args)
            .await;
        assert!(!result.is_cancel());
    }

    #[test]
    fn normalize_app_token_trims_and_lowercases() {
        assert_eq!(normalize_app_token("  Microsoft PowerPoint  "), "microsoft powerpoint");
        assert_eq!(normalize_app_token("Google Chrome.app"), "google chrome");
        assert_eq!(normalize_app_token("iMessage"), "imessage");
        assert_eq!(normalize_app_token(""), "");
    }

    #[test]
    fn extract_tell_application_finds_multiple_targets() {
        let script = r#"
            tell application "System Events"
              tell application "Notes"
                make new note
              end tell
            end tell
        "#;
        let targets = extract_tell_application_targets(script);
        assert!(targets.iter().any(|t| t == "system events"));
        assert!(targets.iter().any(|t| t == "notes"));
    }

    #[test]
    fn extract_tell_application_handles_shorthand() {
        let script = "tell app \"Notes\" to activate";
        let targets = extract_tell_application_targets(script);
        assert_eq!(targets, vec!["notes".to_string()]);
    }

    #[test]
    fn canonical_aliases_cover_microsoft_office() {
        assert!(canonical_aliases("powerpoint").contains(&"microsoft powerpoint"));
        assert!(canonical_aliases("word").contains(&"microsoft word"));
        assert!(canonical_aliases("excel").contains(&"microsoft excel"));
    }
}
