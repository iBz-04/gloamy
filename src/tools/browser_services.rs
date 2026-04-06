use crate::config::BrowserConfig;
use anyhow::{bail, Context};
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::cmp::Ordering;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::net::TcpListener;
use tokio::process::Command;
use tokio::sync::OnceCell;
use tokio::time::sleep;
use tokio::time::timeout;
use tracing::{debug, info, warn};

static STARTUP_ONCE: OnceLock<OnceCell<()>> = OnceLock::new();
const CHROME_FOR_TESTING_INDEX_URL: &str =
    "https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json";

#[cfg(target_os = "windows")]
const CHROMEDRIVER_FILENAME: &str = "chromedriver.exe";
#[cfg(not(target_os = "windows"))]
const CHROMEDRIVER_FILENAME: &str = "chromedriver";

#[derive(Clone, Debug)]
struct SidecarState {
    default_session_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SidecarActionRequest {
    action: String,
    #[serde(default)]
    params: Value,
    #[serde(default)]
    metadata: SidecarMetadata,
}

#[derive(Debug, Default, Deserialize)]
struct SidecarMetadata {
    session_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct SidecarActionResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

pub async fn ensure_started(config: &BrowserConfig) {
    if !config.enabled {
        return;
    }

    let once = STARTUP_ONCE.get_or_init(OnceCell::new);
    let config_snapshot = config.clone();
    once.get_or_init(|| async move {
        bootstrap_services(config_snapshot).await;
    })
    .await;
}

pub fn ensure_started_detached(config: &BrowserConfig) {
    if !config.enabled {
        return;
    }

    let config_snapshot = config.clone();
    let Ok(runtime) = tokio::runtime::Handle::try_current() else {
        debug!("Skipping browser sidecar bootstrap: no active Tokio runtime");
        return;
    };

    runtime.spawn(async move {
        ensure_started(&config_snapshot).await;
    });
}

async fn bootstrap_services(config: BrowserConfig) {
    ensure_native_webdriver(&config).await;
    warmup_agent_browser(&config).await;
    ensure_embedded_computer_use_sidecar(&config).await;
}

async fn warmup_agent_browser(config: &BrowserConfig) {
    let mut version_cmd = Command::new("agent-browser");
    version_cmd.arg("--version");
    version_cmd.stdout(Stdio::piped());
    version_cmd.stderr(Stdio::piped());
    let version_output = timeout(Duration::from_secs(6), version_cmd.output()).await;
    match version_output {
        Ok(Ok(output)) if output.status.success() => {
            let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !ver.is_empty() {
                info!("agent-browser detected: {ver}");
            }
        }
        Ok(Ok(output)) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            warn!("agent-browser --version failed: {stderr}");
            return;
        }
        Ok(Err(err)) => {
            warn!("agent-browser not available: {err}");
            return;
        }
        Err(_) => {
            warn!("agent-browser --version timed out");
            return;
        }
    }

    let mut warmup_cmd = Command::new("agent-browser");
    if let Some(session_name) = config.session_name.as_deref() {
        warmup_cmd.arg("--session").arg(session_name);
    }
    warmup_cmd.arg("open").arg("about:blank").arg("--json");
    warmup_cmd.stdout(Stdio::piped());
    warmup_cmd.stderr(Stdio::piped());

    match timeout(Duration::from_secs(20), warmup_cmd.output()).await {
        Ok(Ok(output)) if output.status.success() => {
            debug!("agent-browser warmup completed");
        }
        Ok(Ok(output)) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if !stderr.is_empty() {
                warn!("agent-browser warmup failed: {stderr}");
            }
        }
        Ok(Err(err)) => {
            warn!("agent-browser warmup failed: {err}");
        }
        Err(_) => {
            warn!("agent-browser warmup timed out");
        }
    }
}

#[derive(Debug, Deserialize)]
struct CftIndex {
    versions: Vec<CftVersionEntry>,
}

#[derive(Debug, Deserialize)]
struct CftVersionEntry {
    version: String,
    downloads: CftDownloads,
}

#[derive(Debug, Deserialize)]
struct CftDownloads {
    #[serde(default)]
    chromedriver: Vec<CftDownload>,
}

#[derive(Debug, Deserialize)]
struct CftDownload {
    platform: String,
    url: String,
}

async fn ensure_native_webdriver(config: &BrowserConfig) {
    let Some(bind_addr) = loopback_bind_addr_for_webdriver(&config.native_webdriver_url) else {
        debug!(
            endpoint = %config.native_webdriver_url,
            "Skipping native WebDriver bootstrap: endpoint is not local loopback http://host:port"
        );
        return;
    };

    if endpoint_reachable(bind_addr).await {
        info!(
            endpoint = %config.native_webdriver_url,
            "native WebDriver already reachable"
        );
        return;
    }

    let chromedriver = match resolve_or_install_chromedriver().await {
        Ok(Some(path)) => path,
        Ok(None) => {
            warn!(
                endpoint = %config.native_webdriver_url,
                "native WebDriver endpoint is down and chromedriver could not be resolved; install chromedriver or set PATH"
            );
            return;
        }
        Err(err) => {
            warn!(
                endpoint = %config.native_webdriver_url,
                "native WebDriver bootstrap failed while preparing chromedriver: {err}"
            );
            return;
        }
    };

    start_chromedriver(chromedriver.as_path(), bind_addr).await;
}

fn loopback_bind_addr_for_webdriver(endpoint: &str) -> Option<SocketAddr> {
    let parsed = reqwest::Url::parse(endpoint).ok()?;
    if parsed.scheme() != "http" {
        return None;
    }

    let host = parsed.host_str()?;
    let port = parsed.port().unwrap_or(9515);
    let normalized_host = host
        .strip_prefix('[')
        .and_then(|trimmed| trimmed.strip_suffix(']'))
        .unwrap_or(host);

    let ip = if normalized_host.eq_ignore_ascii_case("localhost") {
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    } else {
        normalized_host.parse::<IpAddr>().ok()?
    };

    ip.is_loopback().then_some(SocketAddr::new(ip, port))
}

async fn resolve_or_install_chromedriver() -> anyhow::Result<Option<PathBuf>> {
    if command_available("chromedriver").await {
        info!("Browser bootstrap: using chromedriver from PATH");
        return Ok(Some(PathBuf::from("chromedriver")));
    }

    let managed = managed_chromedriver_path()?;
    if command_available(managed.as_path()).await {
        info!(path = %managed.display(), "Browser bootstrap: using managed chromedriver binary");
        return Ok(Some(managed));
    }

    install_managed_chromedriver().await
}

async fn command_available(program: impl AsRef<Path>) -> bool {
    let mut cmd = Command::new(program.as_ref());
    cmd.arg("--version");
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    matches!(
        timeout(Duration::from_secs(5), cmd.output()).await,
        Ok(Ok(output)) if output.status.success()
    )
}

fn managed_chromedriver_path() -> anyhow::Result<PathBuf> {
    let home = UserDirs::new()
        .map(|dirs| dirs.home_dir().to_path_buf())
        .context("Unable to resolve home directory for managed chromedriver path")?;
    Ok(home
        .join(".gloamy")
        .join("bin")
        .join("chromedriver")
        .join(CHROMEDRIVER_FILENAME))
}

#[cfg(target_os = "macos")]
async fn install_managed_chromedriver() -> anyhow::Result<Option<PathBuf>> {
    let chrome_major = match detect_local_chrome_major().await {
        Some(major) => major,
        None => {
            warn!("Unable to detect local Chrome major version; skipping managed chromedriver install");
            return Ok(None);
        }
    };

    let platform = if cfg!(target_arch = "aarch64") {
        "mac-arm64"
    } else {
        "mac-x64"
    };
    let download_url = match resolve_cft_chromedriver_url(chrome_major, platform).await {
        Ok(Some(url)) => url,
        Ok(None) => {
            warn!(
                "No Chrome-for-Testing chromedriver found for Chrome {chrome_major} on {platform}"
            );
            return Ok(None);
        }
        Err(err) => {
            warn!("Failed to resolve Chrome-for-Testing chromedriver index: {err}");
            return Ok(None);
        }
    };

    let destination = managed_chromedriver_path()?;
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).await.with_context(|| {
            format!(
                "Failed to create managed chromedriver directory: {}",
                parent.display()
            )
        })?;
    }

    let temp_root =
        std::env::temp_dir().join(format!("gloamy-chromedriver-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_root).await.with_context(|| {
        format!(
            "Failed to create temporary directory {}",
            temp_root.display()
        )
    })?;
    let zip_path = temp_root.join("chromedriver.zip");
    let unzip_dir = temp_root.join("unpacked");

    let download = async {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .context("Failed to build HTTP client for chromedriver download")?;
        let response = client
            .get(download_url.as_str())
            .send()
            .await
            .with_context(|| format!("Failed to download chromedriver from {download_url}"))?;
        if !response.status().is_success() {
            bail!(
                "Failed to download chromedriver from {download_url}: HTTP {}",
                response.status()
            );
        }
        let body = response
            .bytes()
            .await
            .context("Failed to read downloaded chromedriver archive")?;
        fs::write(&zip_path, &body)
            .await
            .with_context(|| format!("Failed to write {}", zip_path.display()))?;
        Ok::<(), anyhow::Error>(())
    };

    if let Err(err) = download.await {
        let _ = fs::remove_dir_all(&temp_root).await;
        return Err(err);
    }

    fs::create_dir_all(&unzip_dir)
        .await
        .with_context(|| format!("Failed to create {}", unzip_dir.display()))?;

    let mut unzip_cmd = Command::new("unzip");
    unzip_cmd
        .arg("-o")
        .arg(&zip_path)
        .arg("-d")
        .arg(&unzip_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let unzip_out = unzip_cmd
        .output()
        .await
        .context("Failed to run unzip for chromedriver archive")?;
    if !unzip_out.status.success() {
        let stderr = String::from_utf8_lossy(&unzip_out.stderr)
            .trim()
            .to_string();
        let _ = fs::remove_dir_all(&temp_root).await;
        bail!("Failed to unzip chromedriver archive: {stderr}");
    }

    let extracted = unzip_dir
        .join(format!("chromedriver-{platform}"))
        .join(CHROMEDRIVER_FILENAME);
    if !extracted.exists() {
        let _ = fs::remove_dir_all(&temp_root).await;
        bail!(
            "Downloaded chromedriver archive did not contain expected binary at {}",
            extracted.display()
        );
    }

    fs::copy(&extracted, &destination).await.with_context(|| {
        format!(
            "Failed to copy chromedriver binary from {} to {}",
            extracted.display(),
            destination.display()
        )
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        fs::set_permissions(&destination, perms)
            .await
            .with_context(|| {
                format!(
                    "Failed to set executable permissions on {}",
                    destination.display()
                )
            })?;
    }

    let _ = fs::remove_dir_all(&temp_root).await;
    info!(
        path = %destination.display(),
        "Installed managed chromedriver for native WebDriver"
    );
    Ok(Some(destination))
}

#[cfg(not(target_os = "macos"))]
async fn install_managed_chromedriver() -> anyhow::Result<Option<PathBuf>> {
    Ok(None)
}

#[cfg(target_os = "macos")]
async fn detect_local_chrome_major() -> Option<u32> {
    for candidate in [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "google-chrome",
        "chromium",
        "chromium-browser",
    ] {
        let mut cmd = Command::new(candidate);
        cmd.arg("--version");
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::null());
        let output = timeout(Duration::from_secs(4), cmd.output()).await;
        let Ok(Ok(output)) = output else {
            continue;
        };
        if !output.status.success() {
            continue;
        }
        let version_text = String::from_utf8_lossy(&output.stdout);
        if let Some(major) = parse_major_from_version_text(&version_text) {
            return Some(major);
        }
    }
    None
}

#[cfg(not(target_os = "macos"))]
async fn detect_local_chrome_major() -> Option<u32> {
    None
}

async fn resolve_cft_chromedriver_url(
    chrome_major: u32,
    platform: &str,
) -> anyhow::Result<Option<String>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .context("Failed to build HTTP client for Chrome-for-Testing index")?;
    let response = client
        .get(CHROME_FOR_TESTING_INDEX_URL)
        .send()
        .await
        .context("Failed to fetch Chrome-for-Testing version index")?;
    if !response.status().is_success() {
        bail!(
            "Chrome-for-Testing index returned HTTP {}",
            response.status()
        );
    }

    let index: CftIndex = response
        .json()
        .await
        .context("Failed to parse Chrome-for-Testing version index")?;
    let mut best: Option<(VersionKey, String)> = None;

    for entry in index.versions {
        let Some(version_key) = VersionKey::parse(&entry.version) else {
            continue;
        };
        if version_key.major != chrome_major {
            continue;
        }
        let Some(download) = entry
            .downloads
            .chromedriver
            .iter()
            .find(|item| item.platform == platform)
        else {
            continue;
        };

        match &best {
            Some((best_key, _)) if version_key <= *best_key => {}
            _ => best = Some((version_key, download.url.clone())),
        }
    }

    Ok(best.map(|(_, url)| url))
}

async fn start_chromedriver(binary: &Path, bind_addr: SocketAddr) {
    info!(
        binary = %binary.display(),
        endpoint = %bind_addr,
        "Browser bootstrap: starting chromedriver"
    );

    let mut cmd = Command::new(binary);
    cmd.arg(format!("--port={}", bind_addr.port()));
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(err) => {
            warn!(
                binary = %binary.display(),
                endpoint = %bind_addr,
                "Failed to start chromedriver: {err}"
            );
            return;
        }
    };

    for _ in 0..20 {
        if endpoint_reachable(bind_addr).await {
            info!(
                binary = %binary.display(),
                endpoint = %bind_addr,
                "Native WebDriver started"
            );
            return;
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                warn!(
                    binary = %binary.display(),
                    endpoint = %bind_addr,
                    "chromedriver exited before endpoint became ready: {status}"
                );
                return;
            }
            Ok(None) => {
                sleep(Duration::from_millis(150)).await;
            }
            Err(err) => {
                warn!(
                    binary = %binary.display(),
                    endpoint = %bind_addr,
                    "Failed to poll chromedriver process status: {err}"
                );
                return;
            }
        }
    }

    warn!(
        binary = %binary.display(),
        endpoint = %bind_addr,
        "chromedriver did not become reachable before timeout"
    );
}

fn parse_major_from_version_text(text: &str) -> Option<u32> {
    text.split_whitespace().find_map(|token| {
        let cleaned = token.trim_matches(|c: char| !c.is_ascii_digit() && c != '.');
        let major = cleaned.split('.').next()?;
        if major.is_empty() || !major.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        major.parse::<u32>().ok()
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct VersionKey {
    major: u32,
    minor: u32,
    build: u32,
    patch: u32,
}

impl VersionKey {
    fn parse(version: &str) -> Option<Self> {
        let mut parts = version.split('.');
        Some(Self {
            major: parts.next()?.parse().ok()?,
            minor: parts.next()?.parse().ok()?,
            build: parts.next()?.parse().ok()?,
            patch: parts.next()?.parse().ok()?,
        })
    }
}

impl PartialOrd for VersionKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VersionKey {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.build, self.patch).cmp(&(
            other.major,
            other.minor,
            other.build,
            other.patch,
        ))
    }
}

async fn ensure_embedded_computer_use_sidecar(config: &BrowserConfig) {
    let Some(bind_addr) = loopback_bind_addr_for_endpoint(&config.computer_use.endpoint) else {
        debug!(
            endpoint = %config.computer_use.endpoint,
            "Skipping embedded computer-use sidecar: endpoint is not local loopback /v1/actions"
        );
        return;
    };

    if endpoint_reachable(bind_addr).await {
        info!(
            endpoint = %config.computer_use.endpoint,
            "computer-use sidecar already reachable"
        );
        return;
    }

    let listener = match TcpListener::bind(bind_addr).await {
        Ok(listener) => listener,
        Err(err) => {
            if endpoint_reachable(bind_addr).await {
                info!(
                    endpoint = %config.computer_use.endpoint,
                    "computer-use sidecar became reachable while binding"
                );
            } else {
                warn!(
                    endpoint = %config.computer_use.endpoint,
                    "Failed to bind embedded computer-use sidecar: {err}"
                );
            }
            return;
        }
    };

    let state = SidecarState {
        default_session_name: config.session_name.clone(),
    };
    let app = Router::new()
        .route("/health", get(sidecar_health_handler))
        .route(
            "/v1/actions",
            get(sidecar_health_handler).post(sidecar_action_handler),
        )
        .with_state(state);

    tokio::spawn(async move {
        if let Err(err) = axum::serve(listener, app).await {
            warn!("Embedded computer-use sidecar server stopped: {err}");
        }
    });

    if endpoint_reachable(bind_addr).await {
        info!(
            endpoint = %config.computer_use.endpoint,
            "Embedded computer-use sidecar started"
        );
    }
}

fn loopback_bind_addr_for_endpoint(endpoint: &str) -> Option<SocketAddr> {
    let parsed = reqwest::Url::parse(endpoint).ok()?;
    if parsed.scheme() != "http" {
        return None;
    }
    if parsed.path() != "/v1/actions" {
        return None;
    }

    let host = parsed.host_str()?;
    let port = parsed.port_or_known_default()?;
    let normalized_host = host
        .strip_prefix('[')
        .and_then(|trimmed| trimmed.strip_suffix(']'))
        .unwrap_or(host);

    let ip = if normalized_host.eq_ignore_ascii_case("localhost") {
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    } else {
        normalized_host.parse::<IpAddr>().ok()?
    };

    ip.is_loopback().then_some(SocketAddr::new(ip, port))
}

async fn endpoint_reachable(addr: SocketAddr) -> bool {
    timeout(
        Duration::from_millis(500),
        tokio::net::TcpStream::connect(addr),
    )
    .await
    .is_ok_and(|result| result.is_ok())
}

async fn sidecar_health_handler() -> Json<Value> {
    Json(json!({
        "ok": true,
        "service": "gloamy-computer-use-sidecar",
    }))
}

async fn sidecar_action_handler(
    State(state): State<SidecarState>,
    Json(request): Json<SidecarActionRequest>,
) -> Json<SidecarActionResponse> {
    let session_name = request
        .metadata
        .session_name
        .or(state.default_session_name.clone());

    let params = request.params.as_object().cloned().unwrap_or_else(Map::new);
    match dispatch_action(&request.action, &params, session_name.as_deref()).await {
        Ok(data) => Json(SidecarActionResponse {
            success: true,
            data: Some(data),
            error: None,
        }),
        Err(err) => Json(SidecarActionResponse {
            success: false,
            data: None,
            error: Some(err.to_string()),
        }),
    }
}

async fn dispatch_action(
    action: &str,
    params: &Map<String, Value>,
    session_name: Option<&str>,
) -> anyhow::Result<Value> {
    match action {
        "open" => {
            let url = required_string(params, "url")?;
            run_agent_browser(session_name, vec!["open".into(), url]).await
        }
        "snapshot" => {
            let mut args = vec!["snapshot".to_string()];
            if params
                .get("interactive_only")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                args.push("-i".into());
            }
            if params
                .get("compact")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                args.push("-c".into());
            }
            if let Some(depth) = params.get("depth").and_then(Value::as_u64) {
                args.push("-d".into());
                args.push(depth.to_string());
            }
            run_agent_browser(session_name, args).await
        }
        "click" => {
            let selector = required_string(params, "selector")?;
            run_agent_browser(session_name, vec!["click".into(), selector]).await
        }
        "fill" => {
            let selector = required_string(params, "selector")?;
            let value = required_string(params, "value")?;
            run_agent_browser(session_name, vec!["fill".into(), selector, value]).await
        }
        "type" => {
            let selector = required_string(params, "selector")?;
            let text = required_string(params, "text")?;
            run_agent_browser(session_name, vec!["type".into(), selector, text]).await
        }
        "get_text" => {
            let selector = required_string(params, "selector")?;
            run_agent_browser(session_name, vec!["get".into(), "text".into(), selector]).await
        }
        "get_title" => run_agent_browser(session_name, vec!["get".into(), "title".into()]).await,
        "get_url" => run_agent_browser(session_name, vec!["get".into(), "url".into()]).await,
        "screenshot" | "screen_capture" => {
            let path = params
                .get("path")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .unwrap_or_else(default_screenshot_path);
            let mut data =
                run_agent_browser(session_name, vec!["screenshot".into(), path.clone()]).await?;
            if let Value::Object(ref mut obj) = data {
                obj.entry("path").or_insert_with(|| Value::String(path));
            }
            Ok(data)
        }
        "wait" => {
            if let Some(selector) = params.get("selector").and_then(Value::as_str) {
                return run_agent_browser(session_name, vec!["wait".into(), selector.into()]).await;
            }
            if let Some(ms) = params
                .get("timeout_ms")
                .or_else(|| params.get("ms"))
                .and_then(Value::as_u64)
            {
                return run_agent_browser(session_name, vec!["wait".into(), ms.to_string()]).await;
            }
            if let Some(text) = params.get("text").and_then(Value::as_str) {
                return run_agent_browser(
                    session_name,
                    vec!["wait".into(), "--text".into(), text.into()],
                )
                .await;
            }
            Ok(json!({ "waited": true }))
        }
        "press" => {
            let key = required_string(params, "key")?;
            run_agent_browser(session_name, vec!["press".into(), key]).await
        }
        "hover" => {
            let selector = required_string(params, "selector")?;
            run_agent_browser(session_name, vec!["hover".into(), selector]).await
        }
        "scroll" => {
            let direction = required_string(params, "direction")?;
            let mut args = vec!["scroll".to_string(), direction];
            if let Some(pixels) = params.get("pixels").and_then(Value::as_u64) {
                args.push(pixels.to_string());
            }
            run_agent_browser(session_name, args).await
        }
        "is_visible" => {
            let selector = required_string(params, "selector")?;
            run_agent_browser(session_name, vec!["is".into(), "visible".into(), selector]).await
        }
        "close" => run_agent_browser(session_name, vec!["close".into()]).await,
        "find" => {
            let by = required_string(params, "by")?;
            let value = required_string(params, "value")?;
            let find_action = params
                .get("find_action")
                .or_else(|| params.get("action"))
                .and_then(Value::as_str)
                .filter(|text| !text.trim().is_empty())
                .context("Missing 'find_action' for find action")?;
            let mut args = vec!["find".to_string(), by, value, find_action.to_string()];
            if let Some(fill_value) = params.get("fill_value").and_then(Value::as_str) {
                args.push(fill_value.to_string());
            }
            run_agent_browser(session_name, args).await
        }
        "mouse_move" => {
            let x = required_i64(params, "x")?;
            let y = required_i64(params, "y")?;
            run_agent_browser(
                session_name,
                vec!["mouse".into(), "move".into(), x.to_string(), y.to_string()],
            )
            .await
        }
        "mouse_click" => {
            let x = required_i64(params, "x")?;
            let y = required_i64(params, "y")?;
            let button = map_mouse_button(params.get("button").and_then(Value::as_str));

            run_agent_browser(
                session_name,
                vec!["mouse".into(), "move".into(), x.to_string(), y.to_string()],
            )
            .await?;
            run_agent_browser(
                session_name,
                vec!["mouse".into(), "down".into(), button.into()],
            )
            .await?;
            run_agent_browser(
                session_name,
                vec!["mouse".into(), "up".into(), button.into()],
            )
            .await
        }
        "mouse_drag" => {
            let from_x = required_i64(params, "from_x")?;
            let from_y = required_i64(params, "from_y")?;
            let to_x = required_i64(params, "to_x")?;
            let to_y = required_i64(params, "to_y")?;

            run_agent_browser(
                session_name,
                vec![
                    "mouse".into(),
                    "move".into(),
                    from_x.to_string(),
                    from_y.to_string(),
                ],
            )
            .await?;
            run_agent_browser(
                session_name,
                vec!["mouse".into(), "down".into(), "left".into()],
            )
            .await?;
            run_agent_browser(
                session_name,
                vec![
                    "mouse".into(),
                    "move".into(),
                    to_x.to_string(),
                    to_y.to_string(),
                ],
            )
            .await?;
            run_agent_browser(
                session_name,
                vec!["mouse".into(), "up".into(), "left".into()],
            )
            .await
        }
        "key_type" => {
            let text = params
                .get("text")
                .or_else(|| params.get("value"))
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .context("Missing 'text' for key_type action")?;
            run_agent_browser(
                session_name,
                vec!["keyboard".into(), "type".into(), text.to_string()],
            )
            .await
        }
        "key_press" => {
            let key = params
                .get("key")
                .or_else(|| params.get("keys"))
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .context("Missing 'key' for key_press action")?;
            run_agent_browser(session_name, vec!["press".into(), key.to_string()]).await
        }
        "query_state" => {
            let mut state = Map::new();
            let keys = params
                .get("keys")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for key in keys.iter().filter_map(Value::as_str) {
                match key {
                    "url" => {
                        state.insert(
                            "url".into(),
                            run_agent_browser(session_name, vec!["get".into(), "url".into()])
                                .await?,
                        );
                    }
                    "title" => {
                        state.insert(
                            "title".into(),
                            run_agent_browser(session_name, vec!["get".into(), "title".into()])
                                .await?,
                        );
                    }
                    "snapshot" => {
                        state.insert(
                            "snapshot".into(),
                            run_agent_browser(
                                session_name,
                                vec!["snapshot".into(), "-i".into(), "-c".into()],
                            )
                            .await?,
                        );
                    }
                    _ => {}
                }
            }
            Ok(Value::Object(state))
        }
        "wait_event" => {
            let timeout_ms = params
                .get("timeout_ms")
                .and_then(Value::as_u64)
                .unwrap_or(1000);
            let bounded = timeout_ms.min(5000);
            tokio::time::sleep(Duration::from_millis(bounded)).await;
            Ok(json!({
                "waited_ms": bounded,
                "event": params.get("event").cloned().unwrap_or(Value::Null),
            }))
        }
        "hit_test" => {
            let x = required_i64(params, "x")?;
            let y = required_i64(params, "y")?;
            Ok(json!({
                "x": x,
                "y": y,
                "hit": Value::Null,
            }))
        }
        _ => bail!("Unsupported action: {action}"),
    }
}

async fn run_agent_browser(session_name: Option<&str>, args: Vec<String>) -> anyhow::Result<Value> {
    let mut command = Command::new("agent-browser");
    if let Some(session_name) = session_name {
        command.arg("--session").arg(session_name);
    }
    command.args(args);
    command.arg("--json");
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let output = timeout(Duration::from_secs(45), command.output())
        .await
        .context("agent-browser invocation timed out")?
        .context("failed to execute agent-browser")?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !stdout.is_empty() {
        if let Ok(parsed) = serde_json::from_str::<Value>(&stdout) {
            match parsed.get("success").and_then(Value::as_bool) {
                Some(true) => return Ok(parsed.get("data").cloned().unwrap_or(Value::Null)),
                Some(false) => {
                    let error = parsed
                        .get("error")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                        .unwrap_or_else(|| "agent-browser command failed".to_string());
                    bail!("{error}");
                }
                None => return Ok(parsed),
            }
        }
    }

    if output.status.success() {
        if stdout.is_empty() {
            return Ok(Value::Object(Map::new()));
        }
        return Ok(json!({ "output": stdout }));
    }

    let error = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        format!("agent-browser exited with status {}", output.status)
    };
    bail!("{error}");
}

fn required_string(params: &Map<String, Value>, key: &str) -> anyhow::Result<String> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .with_context(|| format!("Missing '{key}'"))
}

fn required_i64(params: &Map<String, Value>, key: &str) -> anyhow::Result<i64> {
    params
        .get(key)
        .and_then(Value::as_i64)
        .with_context(|| format!("Missing or invalid '{key}'"))
}

fn default_screenshot_path() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("/tmp/gloamy-screen-{millis}.png")
}

fn map_mouse_button(raw: Option<&str>) -> &'static str {
    match raw.unwrap_or("left").trim().to_ascii_lowercase().as_str() {
        "right" | "3" => "right",
        "middle" | "2" => "middle",
        _ => "left",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_bind_addr_accepts_loopback_hosts() {
        assert_eq!(
            loopback_bind_addr_for_endpoint("http://127.0.0.1:8787/v1/actions")
                .unwrap()
                .ip(),
            IpAddr::V4(Ipv4Addr::LOCALHOST)
        );
        assert_eq!(
            loopback_bind_addr_for_endpoint("http://localhost:8787/v1/actions")
                .unwrap()
                .port(),
            8787
        );
        assert!(loopback_bind_addr_for_endpoint("http://[::1]:8787/v1/actions").is_some());
    }

    #[test]
    fn webdriver_bind_addr_accepts_loopback_hosts() {
        assert_eq!(
            loopback_bind_addr_for_webdriver("http://127.0.0.1:9515")
                .unwrap()
                .port(),
            9515
        );
        assert_eq!(
            loopback_bind_addr_for_webdriver("http://localhost:9515/status")
                .unwrap()
                .ip(),
            IpAddr::V4(Ipv4Addr::LOCALHOST)
        );
        assert!(loopback_bind_addr_for_webdriver("http://[::1]:9515").is_some());
    }

    #[test]
    fn webdriver_bind_addr_rejects_non_loopback_or_https() {
        assert!(loopback_bind_addr_for_webdriver("https://127.0.0.1:9515").is_none());
        assert!(loopback_bind_addr_for_webdriver("http://192.168.1.10:9515").is_none());
    }

    #[test]
    fn endpoint_bind_addr_rejects_non_loopback_or_invalid_paths() {
        assert!(loopback_bind_addr_for_endpoint("https://127.0.0.1:8787/v1/actions").is_none());
        assert!(loopback_bind_addr_for_endpoint("http://192.168.1.10:8787/v1/actions").is_none());
        assert!(loopback_bind_addr_for_endpoint("http://127.0.0.1:8787/health").is_none());
    }

    #[test]
    fn mouse_button_mapping_is_stable() {
        assert_eq!(map_mouse_button(None), "left");
        assert_eq!(map_mouse_button(Some("left")), "left");
        assert_eq!(map_mouse_button(Some("right")), "right");
        assert_eq!(map_mouse_button(Some("middle")), "middle");
        assert_eq!(map_mouse_button(Some("3")), "right");
    }

    #[test]
    fn parse_major_version_from_browser_output() {
        assert_eq!(
            parse_major_from_version_text("Google Chrome 146.0.7680.178"),
            Some(146)
        );
        assert_eq!(
            parse_major_from_version_text("ChromeDriver 146.0.7680.165 (...)"),
            Some(146)
        );
        assert_eq!(parse_major_from_version_text("no version here"), None);
    }

    #[test]
    fn version_key_ordering_prefers_newest_patch() {
        let older = VersionKey::parse("146.0.7680.153").unwrap();
        let newer = VersionKey::parse("146.0.7680.165").unwrap();
        assert!(newer > older);
    }
}
