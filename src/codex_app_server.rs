//! Client for the locally installed Codex App Server.
//!
//! Authentication remains in Codex's own credential store. The client uses
//! newline-delimited JSON-RPC over stdio and deliberately does not log protocol
//! payloads, which may contain authorization URLs or user content.

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::task::JoinHandle;
use uuid::Uuid;

const CODEX_EXECUTABLE: &str = "codex";
const APP_SERVER_STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
const APP_SERVER_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const APP_SERVER_TURN_TIMEOUT: Duration = Duration::from_secs(120);
const APP_SERVER_LOGIN_TIMEOUT: Duration = Duration::from_secs(600);
const MODEL_PAGE_LIMIT: u64 = 100;
const MODEL_PAGE_COUNT_LIMIT: usize = 100;
const QUEUED_NOTIFICATION_LIMIT: usize = 4096;

/// A model exposed by the installed Codex model catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodexModel {
    pub id: String,
    pub is_default: bool,
}

/// Public account details that are safe for Gloamy's status output.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AccountStatus {
    pub account_type: Option<String>,
}

impl AccountStatus {
    pub fn is_chatgpt_oauth(&self) -> bool {
        self.account_type.as_deref() == Some("chatgpt")
    }
}

/// The browser or device-code prompt returned by Codex App Server.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoginPrompt {
    Browser {
        login_id: String,
        authorization_url: String,
    },
    DeviceCode {
        login_id: String,
        verification_url: String,
        user_code: String,
    },
}

/// One connected Codex App Server process.
pub struct CodexAppServerClient {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    stderr_task: JoinHandle<()>,
    working_directory: PathBuf,
    next_request_id: u64,
    notifications: VecDeque<Value>,
}

impl CodexAppServerClient {
    /// Check whether a Codex executable can be resolved from the current PATH.
    pub fn is_installed() -> bool {
        which::which(CODEX_EXECUTABLE).is_ok()
    }

    /// Start the local App Server and complete its initialization handshake.
    pub async fn connect() -> Result<Self> {
        let codex_path = which::which(CODEX_EXECUTABLE).with_context(|| {
            "Codex CLI was not found on PATH; install Codex CLI to use the codex-app-server provider"
        })?;
        let working_directory = create_private_working_directory()?;

        let mut command = Command::new(codex_path);
        command
            .arg("app-server")
            .arg("--listen")
            .arg("stdio://")
            .arg("-c")
            .arg("mcp_servers={}")
            .arg("-c")
            .arg("web_search=\"disabled\"")
            .arg("--disable")
            .arg("shell_tool")
            .arg("--disable")
            .arg("browser_use")
            .arg("--disable")
            .arg("browser_use_external")
            .arg("--disable")
            .arg("computer_use")
            .arg("--disable")
            .arg("view_image")
            .arg("--disable")
            .arg("browser_annotation_api")
            .arg("--disable")
            .arg("apps")
            .arg("--disable")
            .arg("plugins")
            .arg("--disable")
            .arg("multi_agent")
            .arg("--disable")
            .arg("skill_search")
            .arg("--disable")
            .arg("image_generation")
            .current_dir(&working_directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        apply_sanitized_environment(&mut command);

        let mut child = command
            .spawn()
            .context("failed to start `codex app-server`")?;
        let stdin = child
            .stdin
            .take()
            .context("Codex App Server did not expose its stdio input")?;
        let stdout = child
            .stdout
            .take()
            .context("Codex App Server did not expose its stdio output")?;
        let stderr = child
            .stderr
            .take()
            .context("Codex App Server did not expose its stderr stream")?;
        let stderr_task = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut discarded = Vec::new();
            loop {
                discarded.clear();
                match reader.read_until(b'\n', &mut discarded).await {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {}
                }
            }
        });

        let mut client = Self {
            child,
            stdin: BufWriter::new(stdin),
            stdout: BufReader::new(stdout),
            stderr_task,
            working_directory,
            next_request_id: 1,
            notifications: VecDeque::new(),
        };

        client
            .request(
                "initialize",
                json!({
                    "clientInfo": {
                        "name": "gloamy",
                        "title": "Gloamy",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }),
                APP_SERVER_STARTUP_TIMEOUT,
            )
            .await
            .context("Codex App Server initialization failed")?;
        client.notify("initialized", None).await?;

        Ok(client)
    }

    /// Read the active Codex account without returning email or credential data.
    pub async fn account_status(&mut self) -> Result<AccountStatus> {
        let result = self
            .request("account/read", json!({}), APP_SERVER_REQUEST_TIMEOUT)
            .await?;
        Ok(AccountStatus {
            account_type: result
                .pointer("/account/type")
                .and_then(Value::as_str)
                .map(ToString::to_string),
        })
    }

    /// Start a Codex-managed ChatGPT OAuth flow.
    pub async fn start_login(&mut self, use_device_code: bool) -> Result<LoginPrompt> {
        let params = if use_device_code {
            json!({ "type": "chatgptDeviceCode" })
        } else {
            json!({ "type": "chatgpt" })
        };
        let result = self
            .request("account/login/start", params, APP_SERVER_REQUEST_TIMEOUT)
            .await?;
        let login_id = required_string(&result, "loginId", "Codex App Server login response")?;

        if use_device_code {
            Ok(LoginPrompt::DeviceCode {
                login_id,
                verification_url: required_string(
                    &result,
                    "verificationUrl",
                    "Codex App Server device-code response",
                )?,
                user_code: required_string(
                    &result,
                    "userCode",
                    "Codex App Server device-code response",
                )?,
            })
        } else {
            Ok(LoginPrompt::Browser {
                login_id,
                authorization_url: required_string(
                    &result,
                    "authUrl",
                    "Codex App Server browser-login response",
                )?,
            })
        }
    }

    /// Wait for the matching App Server login completion notification.
    pub async fn wait_for_login(&mut self, login_id: &str) -> Result<()> {
        let started = Instant::now();
        loop {
            let remaining = APP_SERVER_LOGIN_TIMEOUT.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                anyhow::bail!("Timed out waiting for Codex App Server OAuth login");
            }

            let notification = self.next_notification(remaining).await?;
            if notification.get("method").and_then(Value::as_str) != Some("account/login/completed")
            {
                continue;
            }

            let Some(params) = notification.get("params") else {
                continue;
            };
            if params.get("loginId").and_then(Value::as_str) != Some(login_id) {
                continue;
            }
            if params.get("success").and_then(Value::as_bool) == Some(true) {
                let status = self.account_status().await?;
                if status.is_chatgpt_oauth() {
                    return Ok(());
                }
                anyhow::bail!(
                    "Codex App Server login completed without an active ChatGPT OAuth account"
                );
            }

            anyhow::bail!(
                "Codex App Server reported that ChatGPT OAuth login did not complete successfully"
            );
        }
    }

    /// Refresh the managed Codex account's access token without exposing it.
    pub async fn refresh_account(&mut self) -> Result<AccountStatus> {
        let result = self
            .request(
                "account/read",
                json!({ "refreshToken": true }),
                APP_SERVER_REQUEST_TIMEOUT,
            )
            .await?;
        let status = AccountStatus {
            account_type: result
                .pointer("/account/type")
                .and_then(Value::as_str)
                .map(ToString::to_string),
        };
        if !status.is_chatgpt_oauth() {
            anyhow::bail!("No ChatGPT OAuth account is signed in through Codex App Server");
        }
        Ok(status)
    }

    /// Sign out of the account managed by Codex App Server.
    pub async fn logout(&mut self) -> Result<()> {
        self.request("account/logout", Value::Null, APP_SERVER_REQUEST_TIMEOUT)
            .await?;
        Ok(())
    }

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

    /// Run one Gloamy response through an ephemeral, restricted Codex thread.
    pub async fn infer(
        &mut self,
        model: &str,
        base_instructions: &str,
        input: Vec<Value>,
    ) -> Result<String> {
        self.require_chatgpt_oauth().await?;
        if input.is_empty() {
            anyhow::bail!("Codex App Server inference requires at least one input item");
        }

        let mut thread_params = json!({
            "approvalPolicy": "never",
            "sandbox": "read-only",
            "ephemeral": true,
            "cwd": self.working_directory,
            "runtimeWorkspaceRoots": [self.working_directory],
            "environments": [],
            "dynamicTools": [],
            "selectedCapabilityRoots": [],
            "baseInstructions": base_instructions,
            "config": {
                "mcp_servers": {},
                "web_search": "disabled"
            }
        });
        let selected_model = model.trim();
        if !selected_model.is_empty() && !selected_model.eq_ignore_ascii_case("default") {
            thread_params["model"] = Value::String(selected_model.to_string());
        }

        let thread_result = self
            .request("thread/start", thread_params, APP_SERVER_REQUEST_TIMEOUT)
            .await?;
        let thread_id = thread_result
            .pointer("/thread/id")
            .and_then(Value::as_str)
            .context("Codex App Server did not return a thread ID")?
            .to_string();
        let turn_result = self
            .request(
                "turn/start",
                json!({
                    "threadId": thread_id,
                    "input": input
                }),
                APP_SERVER_REQUEST_TIMEOUT,
            )
            .await?;
        let turn_id = turn_result
            .pointer("/turn/id")
            .and_then(Value::as_str)
            .context("Codex App Server did not return a turn ID")?
            .to_string();

        let mut output = String::new();
        let started = Instant::now();
        loop {
            let remaining = APP_SERVER_TURN_TIMEOUT.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                anyhow::bail!("Timed out waiting for Codex App Server inference to complete");
            }
            let notification = self.next_notification(remaining).await?;
            let method = notification.get("method").and_then(Value::as_str);
            let params = notification.get("params");

            match method {
                Some("item/agentMessage/delta") => {
                    let Some(params) = params else {
                        continue;
                    };
                    if params.get("threadId").and_then(Value::as_str) == Some(&thread_id)
                        && params.get("turnId").and_then(Value::as_str) == Some(&turn_id)
                    {
                        if let Some(delta) = params.get("delta").and_then(Value::as_str) {
                            output.push_str(delta);
                        }
                    }
                }
                Some("item/completed") if output.is_empty() => {
                    if let Some(params) = params {
                        if params.get("threadId").and_then(Value::as_str) == Some(&thread_id)
                            && params.get("turnId").and_then(Value::as_str) == Some(&turn_id)
                            && params.pointer("/item/type").and_then(Value::as_str)
                                == Some("agentMessage")
                        {
                            append_completed_message_text(&mut output, params.get("item"));
                        }
                    }
                }
                Some("turn/completed") => {
                    let Some(params) = params else {
                        continue;
                    };
                    if params.get("threadId").and_then(Value::as_str) != Some(&thread_id) {
                        continue;
                    }
                    let Some(turn) = params.get("turn") else {
                        continue;
                    };
                    if turn.get("id").and_then(Value::as_str) != Some(&turn_id) {
                        continue;
                    }
                    if turn.get("status").and_then(Value::as_str) != Some("completed") {
                        let status = turn
                            .get("status")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown");
                        anyhow::bail!("Codex App Server turn ended with status `{status}`");
                    }
                    if output.trim().is_empty() {
                        anyhow::bail!(
                            "Codex App Server completed the turn without an agent message"
                        );
                    }
                    return Ok(output);
                }
                _ => {}
            }
        }
    }

    async fn require_chatgpt_oauth(&mut self) -> Result<()> {
        let status = self.account_status().await?;
        if !status.is_chatgpt_oauth() {
            anyhow::bail!("No ChatGPT OAuth account is signed in; run `gloamy auth login --provider codex-app-server`");
        }
        Ok(())
    }

    async fn request(&mut self, method: &str, params: Value, timeout: Duration) -> Result<Value> {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1);
        let mut message = json!({ "id": request_id, "method": method });
        if !params.is_null() {
            message["params"] = params;
        }
        self.write_message(&message).await?;

        tokio::time::timeout(timeout, self.wait_for_response(request_id, method))
            .await
            .with_context(|| {
                format!("timed out waiting for Codex App Server `{method}` response")
            })?
    }

    async fn notify(&mut self, method: &str, params: Option<Value>) -> Result<()> {
        let mut message = json!({ "method": method });
        if let Some(params) = params {
            message["params"] = params;
        }
        self.write_message(&message).await
    }

    async fn write_message(&mut self, message: &Value) -> Result<()> {
        let serialized = serde_json::to_vec(message)
            .context("failed to encode a Codex App Server protocol message")?;
        self.stdin
            .write_all(&serialized)
            .await
            .context("failed to write to Codex App Server")?;
        self.stdin
            .write_all(b"\n")
            .await
            .context("failed to terminate a Codex App Server protocol message")?;
        self.stdin
            .flush()
            .await
            .context("failed to flush a Codex App Server protocol message")
    }

    async fn wait_for_response(&mut self, request_id: u64, method: &str) -> Result<Value> {
        loop {
            let message = self.read_message().await?;
            if message.get("id").and_then(Value::as_u64) == Some(request_id) {
                if let Some(error) = message.get("error") {
                    let code = error
                        .get("code")
                        .and_then(Value::as_i64)
                        .unwrap_or_default();
                    anyhow::bail!("Codex App Server `{method}` request failed (JSON-RPC {code})");
                }
                return Ok(message.get("result").cloned().unwrap_or(Value::Null));
            }

            if message.get("id").is_some() && message.get("method").is_some() {
                self.reject_server_request(&message).await?;
                anyhow::bail!("Codex App Server sent an unsupported server-initiated request");
            }
            self.queue_notification(message)?;
        }
    }

    async fn read_message(&mut self) -> Result<Value> {
        let mut line = String::new();
        let bytes_read = self
            .stdout
            .read_line(&mut line)
            .await
            .context("failed to read from Codex App Server")?;
        if bytes_read == 0 {
            let status = self
                .child
                .try_wait()
                .context("failed to inspect Codex App Server process status")?;
            anyhow::bail!("Codex App Server closed its stdio stream (process status: {status:?})");
        }
        serde_json::from_str(line.trim_end())
            .context("Codex App Server emitted an invalid JSON-RPC message")
    }

    async fn reject_server_request(&mut self, message: &Value) -> Result<()> {
        let error = json!({
            "id": message.get("id").cloned().unwrap_or(Value::Null),
            "error": {
                "code": -32601,
                "message": "Gloamy does not support server-initiated App Server requests"
            }
        });
        self.write_message(&error).await
    }

    fn queue_notification(&mut self, notification: Value) -> Result<()> {
        if self.notifications.len() >= QUEUED_NOTIFICATION_LIMIT {
            anyhow::bail!("Codex App Server notification queue exceeded its safety limit");
        }
        self.notifications.push_back(notification);
        Ok(())
    }

    async fn next_notification(&mut self, timeout: Duration) -> Result<Value> {
        if let Some(notification) = self.notifications.pop_front() {
            return Ok(notification);
        }

        tokio::time::timeout(timeout, async {
            loop {
                let message = self.read_message().await?;
                if message.get("method").and_then(Value::as_str).is_some() {
                    return Ok(message);
                }
                if message.get("id").is_some() {
                    self.reject_server_request(&message).await?;
                    anyhow::bail!("Codex App Server sent an unsupported server-initiated request");
                }
            }
        })
        .await
        .context("timed out waiting for a Codex App Server notification")?
    }
}

impl Drop for CodexAppServerClient {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
        self.stderr_task.abort();
        let working_directory = self.working_directory.clone();
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn(async move {
                let _ = tokio::fs::remove_dir_all(working_directory).await;
            });
        } else {
            let _ = std::fs::remove_dir_all(working_directory);
        }
    }
}

fn required_string(value: &Value, field: &str, context: &str) -> Result<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|string| !string.trim().is_empty())
        .map(ToString::to_string)
        .with_context(|| format!("{context} did not include `{field}`"))
}

fn move_default_model_first(models: &mut [CodexModel]) {
    if let Some(default_index) = models.iter().position(|model| model.is_default) {
        models.swap(0, default_index);
    }
}

fn append_completed_message_text(output: &mut String, item: Option<&Value>) {
    let Some(item) = item else {
        return;
    };
    if let Some(text) = item.get("text").and_then(Value::as_str) {
        output.push_str(text);
        return;
    }
    let Some(content) = item.get("content").and_then(Value::as_array) else {
        return;
    };
    for part in content {
        let kind = part.get("type").and_then(Value::as_str);
        if matches!(kind, Some("text") | Some("output_text")) {
            if let Some(text) = part.get("text").and_then(Value::as_str) {
                output.push_str(text);
            }
        }
    }
}

fn create_private_working_directory() -> Result<PathBuf> {
    let working_directory = std::env::temp_dir().join(format!("gloamy-codex-{}", Uuid::new_v4()));
    let mut builder = std::fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder
        .create(&working_directory)
        .context("failed to create a private temporary Codex App Server directory")?;
    Ok(working_directory)
}

fn apply_sanitized_environment(command: &mut Command) {
    const ALLOWED_ENVIRONMENT: &[&str] = &[
        "ALL_PROXY",
        "APPDATA",
        "CODEX_HOME",
        "COMSPEC",
        "HOME",
        "HTTPS_PROXY",
        "HTTP_PROXY",
        "LANG",
        "LC_ALL",
        "LOCALAPPDATA",
        "NO_PROXY",
        "PATH",
        "PATHEXT",
        "SSL_CERT_DIR",
        "SSL_CERT_FILE",
        "SYSTEMROOT",
        "TEMP",
        "TMP",
        "TMPDIR",
        "USERPROFILE",
        "WINDIR",
    ];

    let environment: Vec<_> = std::env::vars_os()
        .filter(|(name, _)| {
            let name = name.to_string_lossy();
            ALLOWED_ENVIRONMENT
                .iter()
                .any(|allowed| name.eq_ignore_ascii_case(allowed))
        })
        .collect();
    command.env_clear().envs(environment);
}
