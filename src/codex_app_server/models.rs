use super::{CodexAppServerClient, CodexModel, APP_SERVER_REQUEST_TIMEOUT};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::HashSet;

const MODEL_PAGE_LIMIT: u64 = 100;
const MODEL_PAGE_COUNT_LIMIT: usize = 100;

impl CodexAppServerClient {
    /// Return visible model IDs from every page of the installed catalog.
    pub async fn list_models(&mut self) -> Result<Vec<CodexModel>> {
        self.require_chatgpt_oauth().await?;
        let mut models = Vec::new();
        let mut seen_ids = HashSet::new();
        let mut seen_cursors = HashSet::new();
        let mut cursor: Option<String> = None;

        for _ in 0..MODEL_PAGE_COUNT_LIMIT {
            let mut params = json!({
                "includeHidden": false,
                "limit": MODEL_PAGE_LIMIT
            });
            if let Some(cursor_value) = cursor.as_deref() {
                params["cursor"] = Value::String(cursor_value.to_string());
            }

            let result = self
                .request("model/list", params, APP_SERVER_REQUEST_TIMEOUT)
                .await?;
            let data = result
                .get("data")
                .and_then(Value::as_array)
                .context("Codex App Server model catalog response did not contain a data list")?;

            for entry in data {
                if entry.get("hidden").and_then(Value::as_bool) == Some(true) {
                    continue;
                }
                let model_id = entry
                    .get("model")
                    .and_then(Value::as_str)
                    .filter(|value| !value.trim().is_empty())
                    .or_else(|| {
                        entry
                            .get("id")
                            .and_then(Value::as_str)
                            .filter(|value| !value.trim().is_empty())
                    });
                let Some(model_id) = model_id else {
                    continue;
                };
                if seen_ids.insert(model_id.to_string()) {
                    models.push(CodexModel {
                        id: model_id.to_string(),
                        is_default: entry.get("isDefault").and_then(Value::as_bool) == Some(true),
                    });
                }
            }

            let next_cursor = result
                .get("nextCursor")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string);
            let Some(next_cursor) = next_cursor else {
                move_default_model_first(&mut models);
                return Ok(models);
            };
            if !seen_cursors.insert(next_cursor.clone()) {
                anyhow::bail!("Codex App Server model catalog repeated a pagination cursor");
            }
            cursor = Some(next_cursor);
        }

        anyhow::bail!("Codex App Server model catalog exceeded the pagination safety limit")
    }
}

fn move_default_model_first(models: &mut [CodexModel]) {
    if let Some(default_index) = models.iter().position(|model| model.is_default) {
        models.swap(0, default_index);
    }
}
