use crate::config::IdentityConfig;
use crate::identity;
use crate::security::AutonomyLevel;
use crate::skills::Skill;
use crate::tools::Tool;
use anyhow::Result;
use chrono::Local;
use std::fmt::Write;
use std::path::Path;

const BOOTSTRAP_MAX_CHARS: usize = 20_000;

pub struct PromptContext<'a> {
    pub workspace_dir: &'a Path,
    pub model_name: &'a str,
    pub tools: &'a [Box<dyn Tool>],
    pub skills: &'a [Skill],
    pub skills_prompt_mode: crate::config::SkillsPromptInjectionMode,
    pub identity_config: Option<&'a IdentityConfig>,
    pub dispatcher_instructions: &'a str,
    pub autonomy_level: AutonomyLevel,
}

pub trait PromptSection: Send + Sync {
    fn name(&self) -> &str;
    fn build(&self, ctx: &PromptContext<'_>) -> Result<String>;
}

#[derive(Default)]
pub struct SystemPromptBuilder {
    sections: Vec<Box<dyn PromptSection>>,
}

impl SystemPromptBuilder {
    pub fn with_defaults() -> Self {
        Self {
            sections: vec![
                Box::new(IdentitySection),
                Box::new(ToolsSection),
                Box::new(SafetySection),
                Box::new(ExecutionSection),
                Box::new(SkillsSection),
                Box::new(WorkspaceSection),
                Box::new(DateTimeSection),
                Box::new(RuntimeSection),
            ],
        }
    }

    pub fn add_section(mut self, section: Box<dyn PromptSection>) -> Self {
        self.sections.push(section);
        self
    }

    pub fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        let mut output = String::new();
        for section in &self.sections {
            let part = section.build(ctx)?;
            if part.trim().is_empty() {
                continue;
            }
            output.push_str(part.trim_end());
            output.push_str("\n\n");
        }
        Ok(output)
    }
}

pub struct IdentitySection;
pub struct ToolsSection;
pub struct SafetySection;
pub struct ExecutionSection;
pub struct SkillsSection;
pub struct WorkspaceSection;
pub struct RuntimeSection;
pub struct DateTimeSection;

impl PromptSection for IdentitySection {
    fn name(&self) -> &str {
        "identity"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        let mut prompt = String::from("## Project Context\n\n");
        let mut has_aieos = false;
        if let Some(config) = ctx.identity_config {
            if identity::is_aieos_configured(config) {
                if let Ok(Some(aieos)) = identity::load_aieos_identity(config, ctx.workspace_dir) {
                    let rendered = identity::aieos_to_system_prompt(&aieos);
                    if !rendered.is_empty() {
                        prompt.push_str(&rendered);
                        prompt.push_str("\n\n");
                        has_aieos = true;
                    }
                }
            }
        }

        if !has_aieos {
            prompt.push_str(
                "The following workspace files define your identity, behavior, and context.\n\n",
            );
        }
        for file in [
            "AGENTS.md",
            "SOUL.md",
            "TOOLS.md",
            "IDENTITY.md",
            "USER.md",
            "HEARTBEAT.md",
            "BOOTSTRAP.md",
            "MEMORY.md",
            "experience.md",
            "INTEGRATIONS.md",
        ] {
            inject_workspace_file(&mut prompt, ctx.workspace_dir, file);
        }

        Ok(prompt)
    }
}

impl PromptSection for ToolsSection {
    fn name(&self) -> &str {
        "tools"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        let mut out = String::from("## Tools\n\n");
        for tool in ctx.tools {
            let _ = writeln!(
                out,
                "- **{}**: {}\n  Parameters: `{}`",
                tool.name(),
                tool.description(),
                tool.parameters_schema()
            );
        }
        if !ctx.dispatcher_instructions.is_empty() {
            out.push('\n');
            out.push_str(ctx.dispatcher_instructions);
        }
        Ok(out)
    }
}

impl PromptSection for SafetySection {
    fn name(&self) -> &str {
        "safety"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        match ctx.autonomy_level {
            AutonomyLevel::Full => Ok(
                "## Safety\n\n\
                 - Do not exfiltrate private data.\n\
                 - Execute tools and commands directly without asking for permission.\n\
                 - Take action autonomously. Do not tell the user to run commands you can run yourself.\n\
                 - Chain multiple tool calls when a task requires several steps.\n\
                 - Prefer `trash` over `rm` for deletions."
                    .into(),
            ),
            AutonomyLevel::ReadOnly => Ok(
                "## Safety\n\n\
                 - Do not exfiltrate private data.\n\
                 - Read-only mode: do not modify files or run state-changing commands.\n\
                 - Report what actions would be needed and let the user execute them."
                    .into(),
            ),
            AutonomyLevel::Supervised => Ok(
                "## Safety\n\n\
                 - Do not exfiltrate private data.\n\
                 - Do not run destructive commands without asking.\n\
                 - Do not bypass oversight or approval mechanisms.\n\
                 - Prefer `trash` over `rm`.\n\
                 - When in doubt, ask before acting externally."
                    .into(),
            ),
        }
    }
}

impl PromptSection for SkillsSection {
    fn name(&self) -> &str {
        "skills"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        Ok(crate::skills::skills_to_prompt_with_mode(
            ctx.skills,
            ctx.workspace_dir,
            ctx.skills_prompt_mode,
        ))
    }
}

impl PromptSection for ExecutionSection {
    fn name(&self) -> &str {
        "execution"
    }

    fn build(&self, _ctx: &PromptContext<'_>) -> Result<String> {
        Ok(
            "## Execution Loop\n\n\
             CRITICAL: When given ANY task — simple or complex — execute decisively, keep going until the task is finished, and only pause when a runtime safety gate or missing information truly blocks progress.\n\n\
             ### Step 1: Make a quick internal plan\n\
             Before calling tools, mentally decompose the task into the smallest useful steps. Keep the plan lightweight; do not over-serialize routine work.\n\
             Example: user says \"find Linear's contact email\" → internal plan:\n\
               1. Search the web for the official contact/support page\n\
               2. Open the most likely official page if search is inconclusive\n\
               3. Extract the email and verify it came from the page\n\
             ### Step 2: Act without needless interruption\n\
             - Do not stop after the first step when more steps are clearly required.\n\
             - If the user says \"open X and do Y\", do both in sequence unless a safety gate blocks one of them.\n\
             - If the user says \"create A then B then C\", carry out the whole chain.\n\
             - Prefer reasonable assumptions for routine decisions instead of asking the user to micromanage execution.\n\
             - If a tool fails, try a practical alternative immediately. Do not stop unless the alternatives are exhausted or unsafe.\n\
             - Only ask for confirmation when the runtime security policy or explicit user instruction requires it.\n\
             - When blocked, explain what failed, why it is blocked, and the smallest next input that would unblock progress.\n\n\
             ### Navigation commands (MANDATORY)\n\
             When the user says \"go to [page]\", \"navigate to [page]\", \"open [site]\", or \"go to contact\" — this is a DIRECT NAVIGATION command.\n\
             - Immediately open the browser and navigate to that exact page or URL. Do NOT treat this as a web search.\n\
             - \"go to contact\" = navigate to the contact page of the site currently under discussion.\n\
             - \"go to [site]\" = open that site in the browser directly.\n\
             - NEVER respond with search results when the user explicitly tells you to navigate somewhere.\n\n\
             ### Research / information retrieval ladder (MANDATORY)\n\
             When asked to find information from the web, follow this ladder in order. Move to the next rung ONLY if the current one fails:\n\
               1. Web search (broad query)\n\
               2. Web search (narrower, site-specific query)\n\
               3. Navigate directly to the official site's homepage in browser\n\
               4. Find and navigate to the most likely page (contact, about, support, pricing, docs)\n\
               5. Extract the target data from that page\n\
             Do NOT stop after rung 1 or 2 and report failure — continue down the ladder automatically.\n\n\
             ### Verification rules (MANDATORY — apply to ALL automation, not just GUI)\n\
             - **Never trust exit codes or script success messages alone.** Many macOS automation APIs (PyXA, AppleScript, JXA) exit 0 and print 'success' even when the operation silently did nothing. A shell returning exit code 0 does NOT mean the action actually completed.\n\
             - **After any state-changing action** — creating a note, sending a message, writing a file, creating a calendar event, modifying contacts, etc. — you MUST verify the result by reading back the actual app state before telling the user it is done. Examples:\n\
               - Created a note? → query Notes via `osascript -l JavaScript` to confirm the note exists with the expected title.\n\
               - Created a calendar event? → query the calendar to confirm the event appears.\n\
               - Wrote a file? → read the file back and check its contents.\n\
               - Sent a message? → do not assume delivery; check sent state if the API allows.\n\
             - **If the read-back confirms the action failed or produced no result:** do NOT report success. Try an alternative method (e.g. switch from PyXA to JXA, or use a different API).\n\
             - **If the read-back output is empty or ambiguous:** treat it as a failure. Try again with a different approach.\n\
             - **For GUI tasks specifically:** launching or focusing an app is never task completion. After any state-changing desktop action, verify the result with an app read-back or screenshot before concluding the task is done.\n\
             - Tool disambiguation: \"capture/take a picture\" inside a camera app (Photo Booth, FaceTime, etc.) means click the in-app shutter button via mac_automation, NOT take a screenshot. The screenshot tool only captures the current screen pixels — it cannot trigger in-app actions.\n\n\
             ## Tool Selection Priority (MANDATORY — follow this order)\n\n\
             1. **Skills first (highest priority):** Before choosing ANY tool, check if a loaded skill matches the target app or domain. Skill names follow the pattern `automating-<app>` (e.g. automating-notes, automating-reminders, automating-calendar, automating-contacts, automating-mail, automating-messages, automating-chrome, automating-excel, automating-word, automating-pages, automating-keynote, automating-numbers, automating-voice-memos). If a matching skill is loaded, use its scripting approach (JXA/osascript via shell tool) FIRST. Skills are fast, reliable, and purpose-built. NEVER skip a matching skill to go straight to mac_automation or perception_capture.\n\
             2. **One CLI (preferred for third-party services):** When you need to interact with external services (Gmail, Slack, GitHub, Google Drive, etc.) and the `one` tool is available, use it. One CLI is the preferred integration tool.\n\
             3. **Composio (fallback for third-party services):** Use `composio` only when: (a) the `one` tool is not available, OR (b) the specific action is not available in One CLI, OR (c) One CLI fails for the specific action. Do not use Composio when One CLI can do the same thing.\n\
             4. **Shell / file / memory tools:** For general-purpose operations like running commands, reading/writing files, and managing memory.\n\
             5. **mac_automation (last resort for GUI):** Use ONLY when NO matching skill exists for the target app AND the task requires direct GUI interaction. mac_automation is slow and fragile — treat it as a last resort, not a default.\n\
             6. **perception_capture + click_at (emergency fallback):** Use ONLY for apps with no skill AND no scriptable interface. The perception_capture → inspect_elements → click_at pipeline is expensive and unreliable. Never use it when a skill or AppleScript approach exists.\n\n\
             GUI click strategy (only when no skill matches and mac_automation is required):\n\
               1. Call the `perception_capture` tool (with include_widget_tree and include_ocr=true) to get structured `screen_state` JSON plus screenshot markers.\n\
               2. Identify the target button coordinates from `screen_state.widget_tree`, `screen_state.extracted_text`, or the screenshot marker.\n\
               3. If the screenshot may have been resized, pass coordinate_space with source_width/source_height so mac_automation can scale back to real desktop coordinates.\n\
               4. Use mac_automation action=click_at with those x,y coordinates.\n\
               5. Verify the result with app state, not just transport success.\n\n\
             ### Plan tracking\n\
             For any task requiring more than one tool call, start your response with a `<plan>` block that lists the near-term steps you intend to take:\n\
             <plan>\n\
             - [ ] Step 1 — description\n\
             - [ ] Step 2 — description\n\
             </plan>\n\
             After each tool batch, emit an updated `<plan>` block with completed steps marked `[x]`. The runtime re-injects your latest plan as a system note before every LLM call so you always have the full task context, even in long runs.\n\n\
             Remember: the user expects the whole task to be done. Move quickly, verify state changes, and keep working until complete."
                .into(),
        )
    }
}

impl PromptSection for WorkspaceSection {
    fn name(&self) -> &str {
        "workspace"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        Ok(format!(
            "## Workspace\n\nWorking directory: `{}`",
            ctx.workspace_dir.display()
        ))
    }
}

impl PromptSection for RuntimeSection {
    fn name(&self) -> &str {
        "runtime"
    }

    fn build(&self, ctx: &PromptContext<'_>) -> Result<String> {
        let host =
            hostname::get().map_or_else(|_| "unknown".into(), |h| h.to_string_lossy().to_string());
        Ok(format!(
            "## Runtime\n\nHost: {host} | OS: {} | Model: {}",
            std::env::consts::OS,
            ctx.model_name
        ))
    }
}

impl PromptSection for DateTimeSection {
    fn name(&self) -> &str {
        "datetime"
    }

    fn build(&self, _ctx: &PromptContext<'_>) -> Result<String> {
        let now = Local::now();
        Ok(format!(
            "## Current Date & Time\n\n{} ({})",
            now.format("%Y-%m-%d %H:%M:%S"),
            now.format("%Z")
        ))
    }
}

fn inject_workspace_file(prompt: &mut String, workspace_dir: &Path, filename: &str) {
    let path = workspace_dir.join(filename);
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                return;
            }
            let _ = writeln!(prompt, "### {filename}\n");
            let truncated = if trimmed.chars().count() > BOOTSTRAP_MAX_CHARS {
                trimmed
                    .char_indices()
                    .nth(BOOTSTRAP_MAX_CHARS)
                    .map(|(idx, _)| &trimmed[..idx])
                    .unwrap_or(trimmed)
            } else {
                trimmed
            };
            prompt.push_str(truncated);
            if truncated.len() < trimmed.len() {
                let _ = writeln!(
                    prompt,
                    "\n\n[... truncated at {BOOTSTRAP_MAX_CHARS} chars — use `read` for full file]\n"
                );
            } else {
                prompt.push_str("\n\n");
            }
        }
        Err(_) => {
            let _ = writeln!(prompt, "### {filename}\n\n[File not found: {filename}]\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::traits::Tool;
    use async_trait::async_trait;

    struct TestTool;

    #[async_trait]
    impl Tool for TestTool {
        fn name(&self) -> &str {
            "test_tool"
        }

        fn description(&self) -> &str {
            "tool desc"
        }

        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({"type": "object"})
        }

        async fn execute(
            &self,
            _args: serde_json::Value,
        ) -> anyhow::Result<crate::tools::ToolResult> {
            Ok(crate::tools::ToolResult {
                success: true,
                output: "ok".into(),
                error: None,
            })
        }
    }

    #[test]
    fn identity_section_with_aieos_includes_workspace_files() {
        let workspace =
            std::env::temp_dir().join(format!("gloamy_prompt_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::write(
            workspace.join("AGENTS.md"),
            "Always respond with: AGENTS_MD_LOADED",
        )
        .unwrap();

        let identity_config = crate::config::IdentityConfig {
            format: "aieos".into(),
            aieos_path: None,
            aieos_inline: Some(r#"{"identity":{"names":{"first":"Nova"}}}"#.into()),
        };

        let tools: Vec<Box<dyn Tool>> = vec![];
        let ctx = PromptContext {
            workspace_dir: &workspace,
            model_name: "test-model",
            tools: &tools,
            skills: &[],
            skills_prompt_mode: crate::config::SkillsPromptInjectionMode::Full,
            identity_config: Some(&identity_config),
            dispatcher_instructions: "",
            autonomy_level: AutonomyLevel::Supervised,
        };

        let section = IdentitySection;
        let output = section.build(&ctx).unwrap();

        assert!(
            output.contains("Nova"),
            "AIEOS identity should be present in prompt"
        );
        assert!(
            output.contains("AGENTS_MD_LOADED"),
            "AGENTS.md content should be present even when AIEOS is configured"
        );

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn prompt_builder_assembles_sections() {
        let tools: Vec<Box<dyn Tool>> = vec![Box::new(TestTool)];
        let ctx = PromptContext {
            workspace_dir: Path::new("/tmp"),
            model_name: "test-model",
            tools: &tools,
            skills: &[],
            skills_prompt_mode: crate::config::SkillsPromptInjectionMode::Full,
            identity_config: None,
            dispatcher_instructions: "instr",
            autonomy_level: AutonomyLevel::Supervised,
        };
        let prompt = SystemPromptBuilder::with_defaults().build(&ctx).unwrap();
        assert!(prompt.contains("## Tools"));
        assert!(prompt.contains("test_tool"));
        assert!(prompt.contains("instr"));
    }

    #[test]
    fn skills_section_includes_instructions_and_tools() {
        let tools: Vec<Box<dyn Tool>> = vec![];
        let skills = vec![crate::skills::Skill {
            name: "deploy".into(),
            description: "Release safely".into(),
            version: "1.0.0".into(),
            author: None,
            tags: vec![],
            tools: vec![crate::skills::SkillTool {
                name: "release_checklist".into(),
                description: "Validate release readiness".into(),
                kind: "shell".into(),
                command: "echo ok".into(),
                args: std::collections::HashMap::new(),
            }],
            prompts: vec!["Run smoke tests before deploy.".into()],
            location: None,
        }];

        let ctx = PromptContext {
            workspace_dir: Path::new("/tmp"),
            model_name: "test-model",
            tools: &tools,
            skills: &skills,
            skills_prompt_mode: crate::config::SkillsPromptInjectionMode::Full,
            identity_config: None,
            dispatcher_instructions: "",
            autonomy_level: AutonomyLevel::Supervised,
        };

        let output = SkillsSection.build(&ctx).unwrap();
        assert!(output.contains("<available_skills>"));
        assert!(output.contains("<name>deploy</name>"));
        assert!(output.contains("<instruction>Run smoke tests before deploy.</instruction>"));
        assert!(output.contains("<name>release_checklist</name>"));
        assert!(output.contains("<kind>shell</kind>"));
    }

    #[test]
    fn skills_section_compact_mode_omits_instructions_and_tools() {
        let tools: Vec<Box<dyn Tool>> = vec![];
        let skills = vec![crate::skills::Skill {
            name: "deploy".into(),
            description: "Release safely".into(),
            version: "1.0.0".into(),
            author: None,
            tags: vec![],
            tools: vec![crate::skills::SkillTool {
                name: "release_checklist".into(),
                description: "Validate release readiness".into(),
                kind: "shell".into(),
                command: "echo ok".into(),
                args: std::collections::HashMap::new(),
            }],
            prompts: vec!["Run smoke tests before deploy.".into()],
            location: Some(Path::new("/tmp/workspace/skills/deploy/SKILL.md").to_path_buf()),
        }];

        let ctx = PromptContext {
            workspace_dir: Path::new("/tmp/workspace"),
            model_name: "test-model",
            tools: &tools,
            skills: &skills,
            skills_prompt_mode: crate::config::SkillsPromptInjectionMode::Compact,
            identity_config: None,
            dispatcher_instructions: "",
            autonomy_level: AutonomyLevel::Supervised,
        };

        let output = SkillsSection.build(&ctx).unwrap();
        assert!(output.contains("<available_skills>"));
        assert!(output.contains("<name>deploy</name>"));
        assert!(output.contains("<location>skills/deploy/SKILL.md</location>"));
        assert!(!output.contains("<instruction>Run smoke tests before deploy.</instruction>"));
        assert!(!output.contains("<tools>"));
    }

    #[test]
    fn datetime_section_includes_timestamp_and_timezone() {
        let tools: Vec<Box<dyn Tool>> = vec![];
        let ctx = PromptContext {
            workspace_dir: Path::new("/tmp"),
            model_name: "test-model",
            tools: &tools,
            skills: &[],
            skills_prompt_mode: crate::config::SkillsPromptInjectionMode::Full,
            identity_config: None,
            dispatcher_instructions: "instr",
            autonomy_level: AutonomyLevel::Supervised,
        };

        let rendered = DateTimeSection.build(&ctx).unwrap();
        assert!(rendered.starts_with("## Current Date & Time\n\n"));

        let payload = rendered.trim_start_matches("## Current Date & Time\n\n");
        assert!(payload.chars().any(|c| c.is_ascii_digit()));
        assert!(payload.contains(" ("));
        assert!(payload.ends_with(')'));
    }

    #[test]
    fn prompt_builder_inlines_and_escapes_skills() {
        let tools: Vec<Box<dyn Tool>> = vec![];
        let skills = vec![crate::skills::Skill {
            name: "code<review>&".into(),
            description: "Review \"unsafe\" and 'risky' bits".into(),
            version: "1.0.0".into(),
            author: None,
            tags: vec![],
            tools: vec![crate::skills::SkillTool {
                name: "run\"linter\"".into(),
                description: "Run <lint> & report".into(),
                kind: "shell&exec".into(),
                command: "cargo clippy".into(),
                args: std::collections::HashMap::new(),
            }],
            prompts: vec!["Use <tool_call> and & keep output \"safe\"".into()],
            location: None,
        }];
        let ctx = PromptContext {
            workspace_dir: Path::new("/tmp/workspace"),
            model_name: "test-model",
            tools: &tools,
            skills: &skills,
            skills_prompt_mode: crate::config::SkillsPromptInjectionMode::Full,
            identity_config: None,
            dispatcher_instructions: "",
            autonomy_level: AutonomyLevel::Supervised,
        };

        let prompt = SystemPromptBuilder::with_defaults().build(&ctx).unwrap();

        assert!(prompt.contains("<available_skills>"));
        assert!(prompt.contains("<name>code&lt;review&gt;&amp;</name>"));
        assert!(prompt.contains(
            "<description>Review &quot;unsafe&quot; and &apos;risky&apos; bits</description>"
        ));
        assert!(prompt.contains("<name>run&quot;linter&quot;</name>"));
        assert!(prompt.contains("<description>Run &lt;lint&gt; &amp; report</description>"));
        assert!(prompt.contains("<kind>shell&amp;exec</kind>"));
        assert!(prompt.contains(
            "<instruction>Use &lt;tool_call&gt; and &amp; keep output &quot;safe&quot;</instruction>"
        ));
    }
}
