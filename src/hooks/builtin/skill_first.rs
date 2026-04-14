use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::hooks::traits::{HookHandler, HookResult};

/// Blocks `mac_automation` calls when a loaded `automating-*` skill covers the
/// target app.  Forces the model to use the skill (JXA/osascript via shell)
/// instead of the slow, fragile GUI automation path.
pub struct SkillFirstHook {
    /// Lowercase app-name → skill name, e.g. "notes" → "automating-notes".
    app_to_skill: HashMap<String, String>,
}

impl SkillFirstHook {
    /// Build from loaded skill names.  Any skill whose name starts with
    /// `automating-` is assumed to cover the macOS app derived from the
    /// suffix (hyphens replaced with spaces, title-cased for display).
    pub fn from_skill_names(skill_names: &[String]) -> Self {
        let mut app_to_skill = HashMap::new();
        for name in skill_names {
            if let Some(suffix) = name.strip_prefix("automating-") {
                // Normalise: "voice-memos" → "voice memos" for matching.
                let key = suffix.replace('-', " ");
                app_to_skill.insert(key, name.clone());
            }
        }
        Self { app_to_skill }
    }

    /// Check whether a given app name (as sent by the model in the
    /// `mac_automation` arguments) is covered by a loaded skill.
    fn skill_for_app(&self, app_name: &str) -> Option<&str> {
        let normalised = app_name.trim().to_ascii_lowercase();
        self.app_to_skill.get(&normalised).map(String::as_str)
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

        let app_name = args
            .get("app_name")
            .and_then(Value::as_str)
            .unwrap_or("");

        if let Some(skill) = self.skill_for_app(app_name) {
            let msg = format!(
                "Blocked by skill-first policy: the `{skill}` skill is loaded and covers \
                 {app_name}. Use the skill's scripting approach (JXA/osascript via the shell \
                 tool) instead of mac_automation. Read the skill instructions already in your \
                 system prompt under <available_skills> for the exact commands."
            );
            tracing::info!(hook = "skill-first", app = app_name, skill, "redirecting to skill");
            return HookResult::Cancel(msg);
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
    async fn no_app_name_passes_through() {
        let hook = SkillFirstHook::from_skill_names(&["automating-notes".into()]);

        let args = serde_json::json!({"action": "run_applescript", "script": "tell app \"Notes\" to activate"});
        let result = hook
            .before_tool_call("mac_automation".into(), args)
            .await;

        // No app_name field → can't determine target → allow through.
        assert!(!result.is_cancel());
    }
}
