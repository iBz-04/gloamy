use crate::agent::loop_::ExecutionCheckpointItem;
use crate::providers::{ChatMessage, ConversationMessage};
use anyhow::Context;
use async_trait::async_trait;
use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use std::sync::Arc;

const TASK_STORE_REL_PATH: &str = "state/tasks.db";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TaskStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

impl TaskStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
        }
    }

    fn from_str(raw: &str) -> Self {
        match raw {
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "cancelled" => Self::Cancelled,
            "timed_out" => Self::TimedOut,
            _ => Self::Running,
        }
    }

    fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::TimedOut
        )
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct TaskCheckpointRecord {
    pub step_index: usize,
    pub checkpoint_note: Option<String>,
    pub(crate) items: Vec<ExecutionCheckpointItem>,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct TaskRecord {
    pub task_id: String,
    pub thread_id: String,
    pub channel: String,
    pub provider: String,
    pub model: String,
    pub status: TaskStatus,
    pub execution_history: Vec<ChatMessage>,
    pub resumable_history: Vec<ChatMessage>,
    pub latest_checkpoint_note: Option<String>,
    pub checkpoint_count: usize,
    pub final_response: Option<String>,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub(crate) checkpoints: Vec<TaskCheckpointRecord>,
}

#[derive(Debug, Clone)]
pub(crate) struct TaskSnapshot {
    pub task_id: String,
    pub thread_id: String,
    pub channel: String,
    pub provider: String,
    pub model: String,
    pub status: TaskStatus,
    pub execution_history: Vec<ChatMessage>,
    pub resumable_history: Vec<ChatMessage>,
    pub latest_checkpoint_note: Option<String>,
    pub final_response: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct TaskCheckpointUpdate {
    pub task_id: String,
    pub thread_id: String,
    pub channel: String,
    pub provider: String,
    pub model: String,
    pub step_index: usize,
    pub execution_history: Vec<ChatMessage>,
    pub checkpoint_note: Option<String>,
    pub(crate) items: Vec<ExecutionCheckpointItem>,
}

fn strip_tool_call_tags(message: &str) -> String {
    const TOOL_CALL_OPEN_TAGS: [&str; 7] = [
        "<function_calls>",
        "<function_call>",
        "<tool_call>",
        "<toolcall>",
        "<tool-call>",
        "<tool>",
        "<invoke>",
    ];

    fn find_first_tag<'a>(haystack: &str, tags: &'a [&'a str]) -> Option<(usize, &'a str)> {
        tags.iter()
            .filter_map(|tag| haystack.find(tag).map(|idx| (idx, *tag)))
            .min_by_key(|(idx, _)| *idx)
    }

    fn matching_close_tag(open_tag: &str) -> Option<&'static str> {
        match open_tag {
            "<function_calls>" => Some("</function_calls>"),
            "<function_call>" => Some("</function_call>"),
            "<tool_call>" => Some("</tool_call>"),
            "<toolcall>" => Some("</toolcall>"),
            "<tool-call>" => Some("</tool-call>"),
            "<tool>" => Some("</tool>"),
            "<invoke>" => Some("</invoke>"),
            _ => None,
        }
    }

    fn extract_first_json_end(input: &str) -> Option<usize> {
        let trimmed = input.trim_start();
        let trim_offset = input.len().saturating_sub(trimmed.len());

        for (byte_idx, ch) in trimmed.char_indices() {
            if ch != '{' && ch != '[' {
                continue;
            }

            let slice = &trimmed[byte_idx..];
            let mut stream =
                serde_json::Deserializer::from_str(slice).into_iter::<serde_json::Value>();
            if let Some(Ok(_value)) = stream.next() {
                let consumed = stream.byte_offset();
                if consumed > 0 {
                    return Some(trim_offset + byte_idx + consumed);
                }
            }
        }

        None
    }

    fn strip_leading_close_tags(mut input: &str) -> &str {
        loop {
            let trimmed = input.trim_start();
            if !trimmed.starts_with("</") {
                return trimmed;
            }

            let Some(close_end) = trimmed.find('>') else {
                return "";
            };
            input = &trimmed[close_end + 1..];
        }
    }

    let mut kept_segments = Vec::new();
    let mut remaining = message;

    while let Some((start, open_tag)) = find_first_tag(remaining, &TOOL_CALL_OPEN_TAGS) {
        let before = &remaining[..start];
        if !before.is_empty() {
            kept_segments.push(before.to_string());
        }

        let Some(close_tag) = matching_close_tag(open_tag) else {
            break;
        };
        let after_open = &remaining[start + open_tag.len()..];

        if let Some(close_idx) = after_open.find(close_tag) {
            remaining = &after_open[close_idx + close_tag.len()..];
            continue;
        }

        if let Some(consumed_end) = extract_first_json_end(after_open) {
            remaining = strip_leading_close_tags(&after_open[consumed_end..]);
            continue;
        }

        kept_segments.push(remaining[start..].to_string());
        remaining = "";
        break;
    }

    if !remaining.is_empty() {
        kept_segments.push(remaining.to_string());
    }

    let mut result = kept_segments.concat();
    while result.contains("\n\n\n") {
        result = result.replace("\n\n\n", "\n\n");
    }

    result.trim().to_string()
}

fn normalize_resumable_turns(turns: Vec<ChatMessage>) -> Vec<ChatMessage> {
    let mut normalized = Vec::with_capacity(turns.len());
    let mut expecting_user = true;

    for turn in turns {
        match (expecting_user, turn.role.as_str()) {
            (true, "user") => {
                normalized.push(turn);
                expecting_user = false;
            }
            (false, "assistant") => {
                normalized.push(turn);
                expecting_user = true;
            }
            (false, "user") | (true, "assistant") => {
                if let Some(last_turn) = normalized.last_mut() {
                    if !turn.content.is_empty() {
                        if !last_turn.content.is_empty() {
                            last_turn.content.push_str("\n\n");
                        }
                        last_turn.content.push_str(&turn.content);
                    }
                }
            }
            _ => {}
        }
    }

    normalized
}

pub(crate) fn resumable_turns_from_task_record(record: &TaskRecord) -> Vec<ChatMessage> {
    normalize_resumable_turns(
        record
            .resumable_history
            .iter()
            .filter_map(|msg| match msg.role.as_str() {
                "user" => Some(msg.clone()),
                "assistant" => {
                    let cleaned = strip_tool_call_tags(&msg.content).trim().to_string();
                    if cleaned.is_empty() {
                        None
                    } else {
                        Some(ChatMessage::assistant(cleaned))
                    }
                }
                _ => None,
            })
            .collect(),
    )
}

pub(crate) fn conversation_messages_to_resumable_turns(
    history: &[ConversationMessage],
) -> Vec<ChatMessage> {
    normalize_resumable_turns(
        history
            .iter()
            .filter_map(|message| match message {
                ConversationMessage::Chat(chat) if chat.role == "user" => Some(chat.clone()),
                ConversationMessage::Chat(chat) if chat.role == "assistant" => {
                    let cleaned = strip_tool_call_tags(&chat.content).trim().to_string();
                    if cleaned.is_empty() {
                        None
                    } else {
                        Some(ChatMessage::assistant(cleaned))
                    }
                }
                _ => None,
            })
            .collect(),
    )
}

pub(crate) fn resumable_turns_to_conversation_messages(
    turns: Vec<ChatMessage>,
) -> Vec<ConversationMessage> {
    turns.into_iter().map(ConversationMessage::Chat).collect()
}

pub(crate) async fn load_resumable_state(
    store: &dyn TaskStore,
    session_key: &str,
) -> anyhow::Result<Option<(Vec<ChatMessage>, Option<String>)>> {
    let record = store.load_by_thread_id(session_key).await?;
    let Some(record) = record else {
        return Ok(None);
    };

    let turns = resumable_turns_from_task_record(&record);
    if turns.is_empty() {
        Ok(None)
    } else {
        Ok(Some((turns, record.latest_checkpoint_note)))
    }
}

pub(crate) async fn persist_snapshot(
    store: &dyn TaskStore,
    snapshot: TaskSnapshot,
) -> anyhow::Result<()> {
    store.save_snapshot(snapshot).await
}

pub(crate) async fn clear_session(store: &dyn TaskStore, session_key: &str) -> anyhow::Result<()> {
    store.delete_by_thread_id(session_key).await
}

#[async_trait]
pub(crate) trait TaskStore: Send + Sync {
    async fn load_by_thread_id(&self, thread_id: &str) -> anyhow::Result<Option<TaskRecord>>;
    async fn save_snapshot(&self, snapshot: TaskSnapshot) -> anyhow::Result<()>;
    async fn record_checkpoint(&self, update: TaskCheckpointUpdate) -> anyhow::Result<()>;
    async fn delete_by_thread_id(&self, thread_id: &str) -> anyhow::Result<()>;
}

pub(crate) struct SqliteTaskStore {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl SqliteTaskStore {
    pub fn new(workspace_dir: &Path) -> anyhow::Result<Self> {
        let db_path = workspace_dir.join(TASK_STORE_REL_PATH);
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path).context("failed to open task store SQLite DB")?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA temp_store = MEMORY;",
        )?;
        Self::init_schema(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        })
    }

    fn init_schema(conn: &Connection) -> anyhow::Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS task_records (
                task_id TEXT PRIMARY KEY,
                thread_id TEXT NOT NULL,
                channel TEXT NOT NULL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                status TEXT NOT NULL,
                execution_history_json TEXT NOT NULL,
                resumable_history_json TEXT NOT NULL,
                latest_checkpoint_note TEXT,
                checkpoint_count INTEGER NOT NULL DEFAULT 0,
                final_response TEXT,
                last_error TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                completed_at TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_task_records_thread
                ON task_records(thread_id, updated_at DESC);

            CREATE TABLE IF NOT EXISTS task_checkpoints (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id TEXT NOT NULL,
                step_index INTEGER NOT NULL,
                checkpoint_note TEXT,
                items_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(task_id) REFERENCES task_records(task_id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_task_checkpoints_task_step
                ON task_checkpoints(task_id, step_index);",
        )?;
        Ok(())
    }

    fn now_rfc3339() -> String {
        Utc::now().to_rfc3339()
    }

    fn serialize_history(history: &[ChatMessage]) -> anyhow::Result<String> {
        serde_json::to_string(history).context("failed to serialize task history")
    }

    fn deserialize_history(raw: &str) -> anyhow::Result<Vec<ChatMessage>> {
        serde_json::from_str(raw).context("failed to deserialize task history")
    }

    fn deserialize_items(raw: &str) -> anyhow::Result<Vec<ExecutionCheckpointItem>> {
        serde_json::from_str(raw).context("failed to deserialize task checkpoint items")
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }
}

#[async_trait]
impl TaskStore for SqliteTaskStore {
    async fn load_by_thread_id(&self, thread_id: &str) -> anyhow::Result<Option<TaskRecord>> {
        let conn = Arc::clone(&self.conn);
        let thread_id = thread_id.to_string();

        tokio::task::spawn_blocking(move || -> anyhow::Result<Option<TaskRecord>> {
            let conn = conn.lock();
            let mut stmt = conn.prepare(
                "SELECT
                    task_id,
                    thread_id,
                    channel,
                    provider,
                    model,
                    status,
                    execution_history_json,
                    resumable_history_json,
                    latest_checkpoint_note,
                    checkpoint_count,
                    final_response,
                    last_error,
                    created_at,
                    updated_at,
                    completed_at
                 FROM task_records
                 WHERE thread_id = ?1
                 ORDER BY updated_at DESC
                 LIMIT 1",
            )?;

            let row = stmt.query_row(params![thread_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, i64>(9)?,
                    row.get::<_, Option<String>>(10)?,
                    row.get::<_, Option<String>>(11)?,
                    row.get::<_, String>(12)?,
                    row.get::<_, String>(13)?,
                    row.get::<_, Option<String>>(14)?,
                ))
            });

            let Ok((
                task_id,
                thread_id,
                channel,
                provider,
                model,
                status,
                execution_history_json,
                resumable_history_json,
                latest_checkpoint_note,
                checkpoint_count,
                final_response,
                last_error,
                created_at,
                updated_at,
                completed_at,
            )) = row
            else {
                return Ok(None);
            };

            let mut checkpoints_stmt = conn.prepare(
                "SELECT step_index, checkpoint_note, items_json, created_at
                 FROM task_checkpoints
                 WHERE task_id = ?1
                 ORDER BY step_index ASC, id ASC",
            )?;
            let checkpoints = checkpoints_stmt
                .query_map(params![task_id.clone()], |row| {
                    Ok(TaskCheckpointRecord {
                        step_index: row.get::<_, i64>(0)? as usize,
                        checkpoint_note: row.get(1)?,
                        items: Self::deserialize_items(&row.get::<_, String>(2)?).map_err(
                            |err| {
                                rusqlite::Error::FromSqlConversionFailure(
                                    2,
                                    rusqlite::types::Type::Text,
                                    Box::new(std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        err.to_string(),
                                    )),
                                )
                            },
                        )?,
                        created_at: row.get(3)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Some(TaskRecord {
                task_id,
                thread_id,
                channel,
                provider,
                model,
                status: TaskStatus::from_str(&status),
                execution_history: Self::deserialize_history(&execution_history_json)?,
                resumable_history: Self::deserialize_history(&resumable_history_json)?,
                latest_checkpoint_note,
                checkpoint_count: checkpoint_count.max(0) as usize,
                final_response,
                last_error,
                created_at,
                updated_at,
                completed_at,
                checkpoints,
            }))
        })
        .await?
    }

    async fn save_snapshot(&self, snapshot: TaskSnapshot) -> anyhow::Result<()> {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            let now = Self::now_rfc3339();
            let completed_at = snapshot.status.is_terminal().then(|| now.clone());

            let execution_history_json = Self::serialize_history(&snapshot.execution_history)?;
            let resumable_history_json = Self::serialize_history(&snapshot.resumable_history)?;

            let conn = conn.lock();
            let should_reset_checkpoints = if snapshot.status == TaskStatus::Running {
                conn.query_row(
                    "SELECT status FROM task_records WHERE task_id = ?1",
                    params![snapshot.task_id.clone()],
                    |row| row.get::<_, String>(0),
                )
                .optional()?
                .map(|status| TaskStatus::from_str(&status).is_terminal())
                .unwrap_or(false)
            } else {
                false
            };

            let tx = conn.unchecked_transaction()?;
            if should_reset_checkpoints {
                tx.execute(
                    "DELETE FROM task_checkpoints WHERE task_id = ?1",
                    params![snapshot.task_id.clone()],
                )?;
            }

            tx.execute(
                "INSERT INTO task_records (
                    task_id,
                    thread_id,
                    channel,
                    provider,
                    model,
                    status,
                    execution_history_json,
                    resumable_history_json,
                    latest_checkpoint_note,
                    checkpoint_count,
                    final_response,
                    last_error,
                    created_at,
                    updated_at,
                    completed_at
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, ?11, ?12, ?12, ?13)
                 ON CONFLICT(task_id) DO UPDATE SET
                    thread_id = excluded.thread_id,
                    channel = excluded.channel,
                    provider = excluded.provider,
                    model = excluded.model,
                    status = excluded.status,
                    execution_history_json = excluded.execution_history_json,
                    resumable_history_json = excluded.resumable_history_json,
                    latest_checkpoint_note = excluded.latest_checkpoint_note,
                    checkpoint_count = CASE
                        WHEN ?14 THEN 0
                        ELSE task_records.checkpoint_count
                    END,
                    final_response = excluded.final_response,
                    last_error = excluded.last_error,
                    updated_at = excluded.updated_at,
                    completed_at = excluded.completed_at",
                params![
                    snapshot.task_id,
                    snapshot.thread_id,
                    snapshot.channel,
                    snapshot.provider,
                    snapshot.model,
                    snapshot.status.as_str(),
                    execution_history_json,
                    resumable_history_json,
                    snapshot.latest_checkpoint_note,
                    snapshot.final_response,
                    snapshot.last_error,
                    now,
                    completed_at,
                    should_reset_checkpoints,
                ],
            )?;
            tx.commit()?;
            Ok(())
        })
        .await?
    }

    async fn record_checkpoint(&self, update: TaskCheckpointUpdate) -> anyhow::Result<()> {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            let now = Self::now_rfc3339();
            let execution_history_json = Self::serialize_history(&update.execution_history)?;
            let items_json =
                serde_json::to_string(&update.items).context("failed to serialize checkpoint")?;

            let conn = conn.lock();
            let tx = conn.unchecked_transaction()?;
            tx.execute(
                "INSERT INTO task_records (
                    task_id,
                    thread_id,
                    channel,
                    provider,
                    model,
                    status,
                    execution_history_json,
                    resumable_history_json,
                    latest_checkpoint_note,
                    checkpoint_count,
                    created_at,
                    updated_at
                 )
                 VALUES (
                    ?1,
                    ?2,
                    ?3,
                    ?4,
                    ?5,
                    ?6,
                    ?7,
                    COALESCE(
                        (SELECT resumable_history_json FROM task_records WHERE task_id = ?1),
                        '[]'
                    ),
                    ?8,
                    1,
                    ?9,
                    ?9
                 )
                 ON CONFLICT(task_id) DO UPDATE SET
                    thread_id = excluded.thread_id,
                    channel = excluded.channel,
                    provider = excluded.provider,
                    model = excluded.model,
                    status = excluded.status,
                    execution_history_json = excluded.execution_history_json,
                    latest_checkpoint_note = excluded.latest_checkpoint_note,
                    checkpoint_count = task_records.checkpoint_count + 1,
                    updated_at = excluded.updated_at",
                params![
                    update.task_id,
                    update.thread_id,
                    update.channel,
                    update.provider,
                    update.model,
                    TaskStatus::Running.as_str(),
                    execution_history_json,
                    update.checkpoint_note,
                    now,
                ],
            )?;
            tx.execute(
                "INSERT INTO task_checkpoints (
                    task_id,
                    step_index,
                    checkpoint_note,
                    items_json,
                    created_at
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    update.task_id,
                    update.step_index as i64,
                    update.checkpoint_note,
                    items_json,
                    now,
                ],
            )?;
            tx.commit()?;
            Ok(())
        })
        .await?
    }

    async fn delete_by_thread_id(&self, thread_id: &str) -> anyhow::Result<()> {
        let conn = Arc::clone(&self.conn);
        let thread_id = thread_id.to_string();
        tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            let conn = conn.lock();
            let tx = conn.unchecked_transaction()?;
            tx.execute(
                "DELETE FROM task_checkpoints
                 WHERE task_id IN (
                    SELECT task_id FROM task_records WHERE thread_id = ?1
                 )",
                params![thread_id.clone()],
            )?;
            tx.execute(
                "DELETE FROM task_records WHERE thread_id = ?1",
                params![thread_id],
            )?;
            tx.commit()?;
            Ok(())
        })
        .await?
    }
}

pub(crate) fn create_task_store(workspace_dir: &Path) -> anyhow::Result<Box<dyn TaskStore>> {
    Ok(Box::new(SqliteTaskStore::new(workspace_dir)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_message(role: &str, content: &str) -> ChatMessage {
        ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
        }
    }

    fn sample_snapshot(status: TaskStatus) -> TaskSnapshot {
        TaskSnapshot {
            task_id: "task-1".into(),
            thread_id: "thread-1".into(),
            channel: "telegram".into(),
            provider: "openai".into(),
            model: "gpt-test".into(),
            status,
            execution_history: vec![sample_message("user", "hello")],
            resumable_history: vec![
                sample_message("user", "hello"),
                sample_message("assistant", "working on it"),
            ],
            latest_checkpoint_note: Some("Resume from the latest confirmed step.".into()),
            final_response: None,
            last_error: None,
        }
    }

    #[tokio::test]
    async fn sqlite_task_store_persists_snapshots_and_checkpoints() {
        let tmp = TempDir::new().unwrap();
        let store = SqliteTaskStore::new(tmp.path()).unwrap();

        store
            .save_snapshot(sample_snapshot(TaskStatus::Running))
            .await
            .unwrap();

        store
            .record_checkpoint(TaskCheckpointUpdate {
                task_id: "task-1".into(),
                thread_id: "thread-1".into(),
                channel: "telegram".into(),
                provider: "openai".into(),
                model: "gpt-test".into(),
                step_index: 1,
                execution_history: vec![
                    sample_message("user", "hello"),
                    sample_message("assistant", "<tool_call>{}</tool_call>"),
                ],
                checkpoint_note: Some("Validated file edit; next confirm command output.".into()),
                items: vec![ExecutionCheckpointItem {
                    tool_name: "file_write".into(),
                    arguments: serde_json::json!({"path":"src/main.rs"}),
                    success: true,
                    output: "ok".into(),
                }],
            })
            .await
            .unwrap();

        store
            .save_snapshot(TaskSnapshot {
                final_response: Some("done".into()),
                status: TaskStatus::Completed,
                ..sample_snapshot(TaskStatus::Completed)
            })
            .await
            .unwrap();

        let record = store
            .load_by_thread_id("thread-1")
            .await
            .unwrap()
            .expect("task record should exist");

        assert_eq!(record.status, TaskStatus::Completed);
        assert_eq!(record.checkpoint_count, 1);
        assert_eq!(record.final_response.as_deref(), Some("done"));
        assert_eq!(record.resumable_history.len(), 2);
        assert_eq!(record.checkpoints.len(), 1);
        assert!(record
            .latest_checkpoint_note
            .as_deref()
            .unwrap_or_default()
            .contains("latest confirmed step"));
    }

    #[tokio::test]
    async fn sqlite_task_store_deletes_by_thread_id() {
        let tmp = TempDir::new().unwrap();
        let store = SqliteTaskStore::new(tmp.path()).unwrap();
        store
            .save_snapshot(sample_snapshot(TaskStatus::Running))
            .await
            .unwrap();

        store.delete_by_thread_id("thread-1").await.unwrap();
        assert!(store.load_by_thread_id("thread-1").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn sqlite_task_store_resets_old_checkpoints_when_new_running_task_starts() {
        let tmp = TempDir::new().unwrap();
        let store = SqliteTaskStore::new(tmp.path()).unwrap();

        store
            .record_checkpoint(TaskCheckpointUpdate {
                task_id: "task-1".into(),
                thread_id: "thread-1".into(),
                channel: "telegram".into(),
                provider: "openai".into(),
                model: "gpt-test".into(),
                step_index: 1,
                execution_history: vec![sample_message("user", "hello")],
                checkpoint_note: Some("old checkpoint".into()),
                items: vec![ExecutionCheckpointItem {
                    tool_name: "shell".into(),
                    arguments: serde_json::json!({"command":"cargo test"}),
                    success: true,
                    output: "ok".into(),
                }],
            })
            .await
            .unwrap();
        store
            .save_snapshot(TaskSnapshot {
                final_response: Some("done".into()),
                status: TaskStatus::Completed,
                ..sample_snapshot(TaskStatus::Completed)
            })
            .await
            .unwrap();

        store
            .save_snapshot(TaskSnapshot {
                latest_checkpoint_note: Some("resume from fresh turn".into()),
                status: TaskStatus::Running,
                ..sample_snapshot(TaskStatus::Running)
            })
            .await
            .unwrap();

        let record = store
            .load_by_thread_id("thread-1")
            .await
            .unwrap()
            .expect("task record should exist");

        assert_eq!(record.status, TaskStatus::Running);
        assert_eq!(record.checkpoint_count, 0);
        assert!(record.checkpoints.is_empty());
        assert_eq!(
            record.latest_checkpoint_note.as_deref(),
            Some("resume from fresh turn")
        );
        assert!(record.final_response.is_none());
        assert!(record.completed_at.is_none());
    }
}
