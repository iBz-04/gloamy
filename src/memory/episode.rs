use crate::memory::traits::{Memory, MemoryCategory};
use crate::perception::traits::ScreenState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Represents a single tactical action or turn taken by the agent during an episode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeStep {
    /// Zero-based index of this step in the episode.
    pub step_index: usize,
    /// The structured action formulation or tool call requested.
    pub action_taken: String,
    /// The string or JSON output returned by the environment.
    pub action_result: String,
    /// Perceived state before the action.
    pub screen_state_before: Option<ScreenState>,
    /// Perceived state after the action.
    pub screen_state_after: Option<ScreenState>,
    /// Any execution failure reasons.
    pub execution_error: Option<String>,
}

/// Short-term memory wrapper storing a contiguous sequence of actions (an episode).
/// This provides local grounding for the Worker/AppAgent.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EpisodeBuffer {
    /// Contextual session ID
    pub session_id: String,
    /// The high-level intent driving this episode
    pub active_goal: String,
    /// The sequential steps executed
    pub steps: Vec<EpisodeStep>,
}

/// Long-term knowledge representing a successful end-to-end task execution.
/// Used to bootstrap zero-shot or few-shot planning for similar goals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTrajectory {
    pub id: String,
    pub original_goal: String,
    /// The dominant application or context (e.g., "macOS:Safari")
    pub operating_context: String,
    pub successful_steps: Vec<EpisodeStep>,
}

/// Helper struct to manage the recording and flushing of an EpisodeBuffer
/// and promotion to TaskTrajectories.
pub struct EpisodeManager {
    memory: Arc<dyn Memory>,
    buffer: EpisodeBuffer,
}

impl EpisodeManager {
    pub fn new(memory: Arc<dyn Memory>, session_id: String, active_goal: String) -> Self {
        Self {
            memory,
            buffer: EpisodeBuffer {
                session_id,
                active_goal,
                steps: Vec::new(),
            },
        }
    }

    /// Records a new step into the episode buffer
    pub fn record_step(&mut self, step: EpisodeStep) {
        self.buffer.steps.push(step);
    }

    /// Returns the index to be used by the next recorded step.
    pub fn next_step_index(&self) -> usize {
        self.buffer.steps.len()
    }

    /// Flushes the current episode buffer to short-term memory (Session scoped)
    pub async fn flush(&self) -> anyhow::Result<()> {
        let key = format!("episode_{}", self.buffer.session_id);
        let content = serde_json::to_string_pretty(&self.buffer)?;

        self.memory
            .store(
                &key,
                &content,
                MemoryCategory::Episode,
                Some(&self.buffer.session_id),
            )
            .await
    }

    /// Promotes the current episode buffer to a successful TaskTrajectory and stores it
    pub async fn promote_to_trajectory(&self, operating_context: &str) -> anyhow::Result<()> {
        let trajectory_id = format!("traj_{}", uuid::Uuid::new_v4());
        let trajectory = TaskTrajectory {
            id: trajectory_id.clone(),
            original_goal: self.buffer.active_goal.clone(),
            operating_context: operating_context.to_string(),
            successful_steps: self.buffer.steps.clone(),
        };

        let content = serde_json::to_string_pretty(&trajectory)?;
        // Stored without session scope making it available globally for the agent
        self.memory
            .store(&trajectory_id, &content, MemoryCategory::Trajectory, None)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;

    #[derive(Default)]
    struct RecordingMemory {
        stores: Mutex<Vec<(String, MemoryCategory, Option<String>)>>,
    }

    #[async_trait]
    impl Memory for RecordingMemory {
        fn name(&self) -> &str {
            "recording"
        }

        async fn store(
            &self,
            key: &str,
            _content: &str,
            category: MemoryCategory,
            session_id: Option<&str>,
        ) -> anyhow::Result<()> {
            let mut guard = self.stores.lock().expect("lock should work");
            guard.push((
                key.to_string(),
                category,
                session_id.map(std::string::ToString::to_string),
            ));
            Ok(())
        }

        async fn recall(
            &self,
            _query: &str,
            _limit: usize,
            _session_id: Option<&str>,
        ) -> anyhow::Result<Vec<crate::memory::MemoryEntry>> {
            Ok(Vec::new())
        }

        async fn get(&self, _key: &str) -> anyhow::Result<Option<crate::memory::MemoryEntry>> {
            Ok(None)
        }

        async fn list(
            &self,
            _category: Option<&MemoryCategory>,
            _session_id: Option<&str>,
        ) -> anyhow::Result<Vec<crate::memory::MemoryEntry>> {
            Ok(Vec::new())
        }

        async fn forget(&self, _key: &str) -> anyhow::Result<bool> {
            Ok(false)
        }

        async fn count(&self) -> anyhow::Result<usize> {
            Ok(0)
        }

        async fn health_check(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn episode_manager_flush_and_promote_use_expected_categories() {
        let memory = Arc::new(RecordingMemory::default());
        let mut manager =
            EpisodeManager::new(memory.clone(), "session_1".to_string(), "goal".to_string());

        manager.record_step(EpisodeStep {
            step_index: manager.next_step_index(),
            action_taken: "action".to_string(),
            action_result: "result".to_string(),
            screen_state_before: None,
            screen_state_after: None,
            execution_error: None,
        });

        manager.flush().await.expect("flush should succeed");
        manager
            .promote_to_trajectory("Terminal")
            .await
            .expect("promote should succeed");

        let stores = memory.stores.lock().expect("lock should work");
        assert_eq!(stores.len(), 2);
        assert_eq!(stores[0].1, MemoryCategory::Episode);
        assert_eq!(stores[0].2.as_deref(), Some("session_1"));
        assert_eq!(stores[1].1, MemoryCategory::Trajectory);
        assert!(stores[1].2.is_none());
    }
}
