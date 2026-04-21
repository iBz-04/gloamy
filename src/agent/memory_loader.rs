use crate::memory::{self, Memory};
use async_trait::async_trait;
use std::fmt::Write;

/// Hard cap on per-entry content length when building the memory context
/// block. Prevents a single bloated memory (e.g. an entire persisted tool
/// output or assistant transcript) from dominating the prompt and pushing
/// the payload over the model's context window. Entries longer than this
/// are tail-truncated with a visible marker.
const MAX_CONTEXT_ENTRY_CHARS: usize = 2000;
const ENTRY_TRUNC_MARK: &str = " […truncated…]";

#[async_trait]
pub trait MemoryLoader: Send + Sync {
    async fn load_context(&self, memory: &dyn Memory, user_message: &str)
        -> anyhow::Result<String>;
}

pub struct DefaultMemoryLoader {
    limit: usize,
    min_relevance_score: f64,
}

impl Default for DefaultMemoryLoader {
    fn default() -> Self {
        Self {
            limit: 5,
            min_relevance_score: 0.4,
        }
    }
}

impl DefaultMemoryLoader {
    pub fn new(limit: usize, min_relevance_score: f64) -> Self {
        Self {
            limit: limit.max(1),
            min_relevance_score,
        }
    }
}

#[async_trait]
impl MemoryLoader for DefaultMemoryLoader {
    async fn load_context(
        &self,
        memory: &dyn Memory,
        user_message: &str,
    ) -> anyhow::Result<String> {
        let entries = memory.recall(user_message, self.limit, None).await?;
        if entries.is_empty() {
            return Ok(String::new());
        }

        let mut context = String::from("[Memory context]\n");
        for entry in entries {
            if memory::is_assistant_autosave_key(&entry.key) {
                continue;
            }
            if let Some(score) = entry.score {
                if score < self.min_relevance_score {
                    continue;
                }
            }
            // Tail-truncate any single memory whose content is larger than
            // the per-entry cap. Without this, a huge recalled entry (e.g.
            // a past tool result stored verbatim) silently bloats every
            // subsequent prompt until it alone overflows the context.
            let trimmed = if entry.content.chars().count() > MAX_CONTEXT_ENTRY_CHARS {
                let head: String = entry
                    .content
                    .chars()
                    .take(MAX_CONTEXT_ENTRY_CHARS)
                    .collect();
                format!("{head}{ENTRY_TRUNC_MARK}")
            } else {
                entry.content
            };
            let _ = writeln!(context, "- {}: {}", entry.key, trimmed);
        }

        // If all entries were below threshold, return empty
        if context == "[Memory context]\n" {
            return Ok(String::new());
        }

        context.push('\n');
        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{Memory, MemoryCategory, MemoryEntry};
    use std::sync::Arc;

    struct MockMemory;
    struct MockMemoryWithEntries {
        entries: Arc<Vec<MemoryEntry>>,
    }

    #[async_trait]
    impl Memory for MockMemory {
        async fn store(
            &self,
            _key: &str,
            _content: &str,
            _category: MemoryCategory,
            _session_id: Option<&str>,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn recall(
            &self,
            _query: &str,
            limit: usize,
            _session_id: Option<&str>,
        ) -> anyhow::Result<Vec<MemoryEntry>> {
            if limit == 0 {
                return Ok(vec![]);
            }
            Ok(vec![MemoryEntry {
                id: "1".into(),
                key: "k".into(),
                content: "v".into(),
                category: MemoryCategory::Conversation,
                timestamp: "now".into(),
                session_id: None,
                score: None,
            }])
        }

        async fn get(&self, _key: &str) -> anyhow::Result<Option<MemoryEntry>> {
            Ok(None)
        }

        async fn list(
            &self,
            _category: Option<&MemoryCategory>,
            _session_id: Option<&str>,
        ) -> anyhow::Result<Vec<MemoryEntry>> {
            Ok(vec![])
        }

        async fn forget(&self, _key: &str) -> anyhow::Result<bool> {
            Ok(true)
        }

        async fn count(&self) -> anyhow::Result<usize> {
            Ok(0)
        }

        async fn health_check(&self) -> bool {
            true
        }

        fn name(&self) -> &str {
            "mock"
        }
    }

    #[async_trait]
    impl Memory for MockMemoryWithEntries {
        async fn store(
            &self,
            _key: &str,
            _content: &str,
            _category: MemoryCategory,
            _session_id: Option<&str>,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn recall(
            &self,
            _query: &str,
            _limit: usize,
            _session_id: Option<&str>,
        ) -> anyhow::Result<Vec<MemoryEntry>> {
            Ok(self.entries.as_ref().clone())
        }

        async fn get(&self, _key: &str) -> anyhow::Result<Option<MemoryEntry>> {
            Ok(None)
        }

        async fn list(
            &self,
            _category: Option<&MemoryCategory>,
            _session_id: Option<&str>,
        ) -> anyhow::Result<Vec<MemoryEntry>> {
            Ok(vec![])
        }

        async fn forget(&self, _key: &str) -> anyhow::Result<bool> {
            Ok(true)
        }

        async fn count(&self) -> anyhow::Result<usize> {
            Ok(self.entries.len())
        }

        async fn health_check(&self) -> bool {
            true
        }

        fn name(&self) -> &str {
            "mock-with-entries"
        }
    }

    #[tokio::test]
    async fn default_loader_truncates_huge_entry_content() {
        let loader = DefaultMemoryLoader::new(5, 0.0);
        let huge = "x".repeat(MAX_CONTEXT_ENTRY_CHARS * 3);
        let memory = MockMemoryWithEntries {
            entries: Arc::new(vec![MemoryEntry {
                id: "1".into(),
                key: "bloated".into(),
                content: huge,
                category: MemoryCategory::Conversation,
                timestamp: "now".into(),
                session_id: None,
                score: Some(0.9),
            }]),
        };

        let context = loader.load_context(&memory, "q").await.unwrap();
        // Context must be bounded — header(~20) + key prefix(~12) + capped
        // body + marker — well under 1.5× the per-entry cap.
        assert!(
            context.len() < MAX_CONTEXT_ENTRY_CHARS + 200,
            "expected bounded context, got {} chars",
            context.len()
        );
        assert!(context.contains(ENTRY_TRUNC_MARK));
        assert!(context.contains("bloated"));
    }

    #[tokio::test]
    async fn default_loader_formats_context() {
        let loader = DefaultMemoryLoader::default();
        let context = loader.load_context(&MockMemory, "hello").await.unwrap();
        assert!(context.contains("[Memory context]"));
        assert!(context.contains("- k: v"));
    }

    #[tokio::test]
    async fn default_loader_skips_legacy_assistant_autosave_entries() {
        let loader = DefaultMemoryLoader::new(5, 0.0);
        let memory = MockMemoryWithEntries {
            entries: Arc::new(vec![
                MemoryEntry {
                    id: "1".into(),
                    key: "assistant_resp_legacy".into(),
                    content: "fabricated detail".into(),
                    category: MemoryCategory::Daily,
                    timestamp: "now".into(),
                    session_id: None,
                    score: Some(0.95),
                },
                MemoryEntry {
                    id: "2".into(),
                    key: "user_fact".into(),
                    content: "User prefers concise answers".into(),
                    category: MemoryCategory::Conversation,
                    timestamp: "now".into(),
                    session_id: None,
                    score: Some(0.9),
                },
            ]),
        };

        let context = loader.load_context(&memory, "answer style").await.unwrap();
        assert!(context.contains("user_fact"));
        assert!(!context.contains("assistant_resp_legacy"));
        assert!(!context.contains("fabricated detail"));
    }
}
