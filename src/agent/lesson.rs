use crate::memory::{Memory, MemoryCategory};
use std::fmt::Write;
use uuid::Uuid;

/// Category key used for all lesson memories.
pub(crate) const LESSON_CATEGORY: &str = "lesson";

/// A single tool outcome tracked during a tool-call loop iteration.
#[derive(Debug, Clone)]
pub struct ToolOutcome {
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub success: bool,
    pub output: String,
}

/// A structured lesson extracted from a fail→success pattern.
#[derive(Debug, Clone)]
pub struct Lesson {
    pub tool_name: String,
    pub error_summary: String,
    pub correction: String,
    pub task_keywords: String,
}

impl Lesson {
    /// Serialize to a compact memory-friendly string.
    pub fn to_memory_content(&self) -> String {
        format!(
            "TOOL: {} | ERROR: {} | FIX: {} | CONTEXT: {}",
            self.tool_name, self.error_summary, self.correction, self.task_keywords
        )
    }

    /// Generate a unique memory key for this lesson.
    pub fn memory_key() -> String {
        format!("lesson_{}", Uuid::new_v4())
    }
}

/// Extract lessons from a sequence of tool outcomes within a single turn.
///
/// Looks for patterns where:
/// 1. A tool call failed, then the same tool succeeded later (corrected approach).
/// 2. A tool call failed, then a different tool succeeded (alternative approach).
///
/// Only extracts lessons when the turn ultimately succeeded (i.e. the agent
/// recovered from the failure).
pub fn extract_lessons(outcomes: &[ToolOutcome], user_message: &str) -> Vec<Lesson> {
    let mut lessons = Vec::new();
    let task_keywords = extract_task_keywords(user_message);

    for (i, outcome) in outcomes.iter().enumerate() {
        if outcome.success {
            continue;
        }

        // Look ahead for a subsequent success with the same tool name
        let correction = outcomes[i + 1..]
            .iter()
            .find(|later| later.tool_name == outcome.tool_name && later.success);

        if let Some(success_outcome) = correction {
            let error_summary = truncate_error(&outcome.output, 200);
            let correction_desc =
                describe_correction(&outcome.arguments, &success_outcome.arguments);

            lessons.push(Lesson {
                tool_name: outcome.tool_name.clone(),
                error_summary,
                correction: correction_desc,
                task_keywords: task_keywords.clone(),
            });
        }
    }

    // Deduplicate lessons by (tool_name, error_summary prefix)
    lessons.dedup_by(|a, b| {
        a.tool_name == b.tool_name && truncate_error(&a.error_summary, 60) == truncate_error(&b.error_summary, 60)
    });

    lessons
}

/// Persist extracted lessons to memory, skipping duplicates.
pub async fn persist_lessons(
    memory: &dyn Memory,
    lessons: &[Lesson],
) -> usize {
    let mut stored = 0;
    let category = MemoryCategory::Custom(LESSON_CATEGORY.to_string());

    for lesson in lessons {
        // Check for existing similar lesson to avoid duplicates
        let query = format!("{} {}", lesson.tool_name, truncate_error(&lesson.error_summary, 80));
        if let Ok(existing) = memory.recall(&query, 3, None).await {
            let dominated = existing.iter().any(|entry| {
                entry.category == category
                    && entry.score.unwrap_or(0.0) > 0.7
                    && entry.content.contains(&lesson.tool_name)
            });
            if dominated {
                tracing::debug!(
                    tool = %lesson.tool_name,
                    "Skipping duplicate lesson — similar lesson already exists"
                );
                continue;
            }
        }

        let key = Lesson::memory_key();
        let content = lesson.to_memory_content();
        match memory.store(&key, &content, category.clone(), None).await {
            Ok(()) => {
                tracing::info!(
                    tool = %lesson.tool_name,
                    key = %key,
                    "Stored self-improvement lesson"
                );
                stored += 1;
            }
            Err(e) => {
                tracing::warn!(
                    tool = %lesson.tool_name,
                    error = %e,
                    "Failed to persist lesson"
                );
            }
        }
    }

    stored
}

/// Recall relevant lessons from memory and format them for context injection.
pub async fn build_lesson_context(
    memory: &dyn Memory,
    user_message: &str,
    max_lessons: usize,
) -> String {
    let category = MemoryCategory::Custom(LESSON_CATEGORY.to_string());
    let entries = match memory.recall(user_message, max_lessons + 2, None).await {
        Ok(entries) => entries,
        Err(_) => return String::new(),
    };

    let relevant: Vec<_> = entries
        .into_iter()
        .filter(|e| e.category == category)
        .filter(|e| e.score.map_or(true, |s| s >= 0.3))
        .take(max_lessons)
        .collect();

    if relevant.is_empty() {
        return String::new();
    }

    let mut context = String::from("[Lessons learned — avoid repeating these mistakes]\n");
    for entry in &relevant {
        let _ = writeln!(context, "- {}", entry.content);
    }
    context.push('\n');
    context
}

// ── Helpers ───────────────────────────────────────────────────────────

fn truncate_error(s: &str, max_chars: usize) -> String {
    let first_line = s.lines().next().unwrap_or(s);
    if first_line.chars().count() <= max_chars {
        first_line.to_string()
    } else {
        let truncated: String = first_line.chars().take(max_chars.saturating_sub(3)).collect();
        format!("{truncated}...")
    }
}

fn describe_correction(
    failed_args: &serde_json::Value,
    success_args: &serde_json::Value,
) -> String {
    // Try to find the key difference between failed and successful args
    if let (Some(fail_map), Some(success_map)) = (failed_args.as_object(), success_args.as_object())
    {
        let mut diffs = Vec::new();
        for (key, success_val) in success_map {
            match fail_map.get(key) {
                Some(fail_val) if fail_val != success_val => {
                    let success_str = value_preview(success_val, 80);
                    diffs.push(format!("Changed {key} to: {success_str}"));
                }
                None => {
                    let success_str = value_preview(success_val, 80);
                    diffs.push(format!("Added {key}: {success_str}"));
                }
                _ => {}
            }
        }
        if !diffs.is_empty() {
            return diffs.join("; ");
        }
    }

    // Fallback: describe the successful args directly
    let preview = value_preview(success_args, 150);
    format!("Use: {preview}")
}

fn value_preview(val: &serde_json::Value, max_len: usize) -> String {
    let s = match val {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    };
    if s.chars().count() <= max_len {
        s
    } else {
        let truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
        format!("{truncated}...")
    }
}

fn extract_task_keywords(user_message: &str) -> String {
    // Extract meaningful words (>3 chars, lowercase, deduplicated)
    let stopwords = [
        "the", "and", "for", "that", "this", "with", "from", "have", "will",
        "what", "when", "where", "which", "your", "about", "been", "could",
        "would", "should", "their", "there", "these", "those", "than",
        "them", "then", "they", "were", "also", "into", "just", "some",
        "very", "make", "like", "please", "want", "need", "can",
    ];

    let words: Vec<String> = user_message
        .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .filter(|w| w.len() > 3)
        .map(|w| w.to_lowercase())
        .filter(|w| !stopwords.contains(&w.as_str()))
        .collect();

    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    let unique: Vec<String> = words
        .into_iter()
        .filter(|w| seen.insert(w.clone()))
        .take(8)
        .collect();

    unique.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_lessons_finds_fail_then_success() {
        let outcomes = vec![
            ToolOutcome {
                tool_name: "shell".into(),
                arguments: serde_json::json!({"command": "pip install foo"}),
                success: false,
                output: "Error: pip not found".into(),
            },
            ToolOutcome {
                tool_name: "shell".into(),
                arguments: serde_json::json!({"command": "pip3 install foo"}),
                success: true,
                output: "Successfully installed foo".into(),
            },
        ];

        let lessons = extract_lessons(&outcomes, "install the foo package");
        assert_eq!(lessons.len(), 1);
        assert_eq!(lessons[0].tool_name, "shell");
        assert!(lessons[0].error_summary.contains("pip not found"));
        assert!(lessons[0].correction.contains("pip3"));
    }

    #[test]
    fn extract_lessons_no_lesson_when_all_succeed() {
        let outcomes = vec![
            ToolOutcome {
                tool_name: "shell".into(),
                arguments: serde_json::json!({"command": "ls"}),
                success: true,
                output: "file1 file2".into(),
            },
        ];

        let lessons = extract_lessons(&outcomes, "list files");
        assert!(lessons.is_empty());
    }

    #[test]
    fn extract_lessons_no_lesson_when_no_recovery() {
        let outcomes = vec![
            ToolOutcome {
                tool_name: "shell".into(),
                arguments: serde_json::json!({"command": "bad_cmd"}),
                success: false,
                output: "command not found".into(),
            },
            ToolOutcome {
                tool_name: "file_read".into(),
                arguments: serde_json::json!({"path": "/tmp/x"}),
                success: true,
                output: "contents".into(),
            },
        ];

        // No lesson because the success is a different tool, not recovery of same tool
        let lessons = extract_lessons(&outcomes, "do something");
        assert!(lessons.is_empty());
    }

    #[test]
    fn extract_lessons_deduplicates() {
        let outcomes = vec![
            ToolOutcome {
                tool_name: "shell".into(),
                arguments: serde_json::json!({"command": "npm run build"}),
                success: false,
                output: "Error: missing dependency X".into(),
            },
            ToolOutcome {
                tool_name: "shell".into(),
                arguments: serde_json::json!({"command": "npm run build"}),
                success: false,
                output: "Error: missing dependency X".into(),
            },
            ToolOutcome {
                tool_name: "shell".into(),
                arguments: serde_json::json!({"command": "npm install X && npm run build"}),
                success: true,
                output: "Build succeeded".into(),
            },
        ];

        let lessons = extract_lessons(&outcomes, "build the project");
        // Both failures match the same success, but should be deduped to 1
        assert_eq!(lessons.len(), 1);
    }

    #[test]
    fn extract_task_keywords_filters_stopwords() {
        let keywords = extract_task_keywords("please make the build work for this project");
        assert!(!keywords.contains("please"));
        assert!(!keywords.contains("the"));
        assert!(keywords.contains("build"));
        assert!(keywords.contains("work"));
        assert!(keywords.contains("project"));
    }

    #[test]
    fn lesson_to_memory_content_format() {
        let lesson = Lesson {
            tool_name: "shell".into(),
            error_summary: "pip not found".into(),
            correction: "Use pip3 instead".into(),
            task_keywords: "install package".into(),
        };
        let content = lesson.to_memory_content();
        assert!(content.starts_with("TOOL: shell"));
        assert!(content.contains("ERROR: pip not found"));
        assert!(content.contains("FIX: Use pip3 instead"));
        assert!(content.contains("CONTEXT: install package"));
    }

    #[test]
    fn truncate_error_short_unchanged() {
        assert_eq!(truncate_error("short", 100), "short");
    }

    #[test]
    fn truncate_error_long_truncated() {
        let long = "a".repeat(300);
        let result = truncate_error(&long, 50);
        assert!(result.ends_with("..."));
        assert!(result.chars().count() <= 50);
    }

    #[test]
    fn truncate_error_multiline_uses_first() {
        assert_eq!(truncate_error("first\nsecond\nthird", 100), "first");
    }

    #[test]
    fn describe_correction_shows_diffs() {
        let failed = serde_json::json!({"command": "pip install foo"});
        let success = serde_json::json!({"command": "pip3 install foo"});
        let desc = describe_correction(&failed, &success);
        assert!(desc.contains("command"));
        assert!(desc.contains("pip3"));
    }
}
