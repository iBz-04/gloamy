//! Client for the locally installed Codex App Server.
//!
//! Authentication remains in Codex's own credential store. The client uses
//! newline-delimited JSON-RPC over stdio and deliberately does not log protocol
//! payloads, which may contain authorization URLs or user content.

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::task::JoinHandle;
use uuid::Uuid;

const CODEX_EXECUTABLE: &str = "codex";
const APP_SERVER_STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
const APP_SERVER_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
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

mod account;
mod inference;
mod models;

#[cfg(all(test, unix))]
mod tests;

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
        Self::connect_with_binary(codex_path).await
    }

    async fn connect_with_binary(codex_path: PathBuf) -> Result<Self> {
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

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                let _ = std::fs::remove_dir_all(&working_directory);
                return Err(error).context("failed to start `codex app-server`");
            }
        };
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
                    },
                    "capabilities": {
                        "experimentalApi": true
                    }
                }),
                APP_SERVER_STARTUP_TIMEOUT,
            )
            .await
            .context("Codex App Server initialization failed")?;
        client.notify("initialized", None).await?;

        Ok(client)
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
