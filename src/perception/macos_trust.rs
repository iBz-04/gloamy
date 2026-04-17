//! macOS TCC (privacy) permission probes for GUI automation.
//!
//! The `mac_automation click_at` / `move_mouse` actions post CoreGraphics HID
//! events. macOS requires the **responsible parent process** to hold the
//! Accessibility permission (System Settings → Privacy & Security →
//! Accessibility). Without it, `CGEventPost` silently no-ops — the tool thinks
//! it clicked, but nothing happened on screen.
//!
//! Similarly `screencapture` requires Screen Recording permission; without it
//! the resulting image is a solid black frame (or fails on macOS 15+).
//!
//! These probes let the daemon detect the missing permission up front and
//! surface a clear, actionable error instead of producing silent fake
//! successes. On non-macOS builds every probe returns `true` (no-op).
//!
//! Results are cached with short TTLs: positive results cached ~5 minutes
//! (permissions rarely revoked mid-session); negative results cached ~3
//! seconds so that granting the permission is detected quickly without a
//! daemon restart.
//!
//! The probes themselves rely only on `osascript -l JavaScript` which is part
//! of every macOS install; they introduce no additional system dependency.
//!
//! Design notes:
//! - `AXIsProcessTrusted()` returns the trust state for the *effective*
//!   process lineage (usually the terminal or launcher that spawned gloamy).
//! - `CGPreflightScreenCaptureAccess()` never prompts; it just reports.
//! - We do **not** call the `*WithOptions(prompt=true)` variants, because
//!   prompting would interleave with agent tool calls unpredictably.

use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const POSITIVE_TTL: Duration = Duration::from_secs(300);
const NEGATIVE_TTL: Duration = Duration::from_secs(3);

#[derive(Default)]
struct TrustCache {
    accessibility: Option<(Instant, bool)>,
    screen_recording: Option<(Instant, bool)>,
}

impl TrustCache {
    fn read(&self, slot: CacheSlot) -> Option<bool> {
        let entry = match slot {
            CacheSlot::Accessibility => self.accessibility,
            CacheSlot::ScreenRecording => self.screen_recording,
        };
        let (at, value) = entry?;
        let ttl = if value { POSITIVE_TTL } else { NEGATIVE_TTL };
        if at.elapsed() > ttl {
            None
        } else {
            Some(value)
        }
    }

    fn write(&mut self, slot: CacheSlot, value: bool) {
        let entry = Some((Instant::now(), value));
        match slot {
            CacheSlot::Accessibility => self.accessibility = entry,
            CacheSlot::ScreenRecording => self.screen_recording = entry,
        }
    }
}

#[derive(Clone, Copy)]
enum CacheSlot {
    Accessibility,
    ScreenRecording,
}

fn cache() -> &'static Mutex<TrustCache> {
    use std::sync::OnceLock;
    static CACHE: OnceLock<Mutex<TrustCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(TrustCache::default()))
}

/// Snapshot of macOS privacy trust state, used for startup diagnostics.
#[derive(Debug, Clone, Copy)]
pub struct MacTrustStatus {
    pub accessibility: bool,
    pub screen_recording: bool,
}

impl MacTrustStatus {
    pub fn all_granted(self) -> bool {
        self.accessibility && self.screen_recording
    }
}

/// Short human-readable hint describing how to grant the missing permission.
/// Shared between startup logs and tool errors so users see the same path.
pub const ACCESSIBILITY_REMEDIATION: &str =
    "Grant Accessibility permission: System Settings → Privacy & Security → \
     Accessibility → add the app that launched gloamy (Terminal.app, iTerm, \
     or the gloamy binary itself if run as a launch agent), then restart \
     gloamy. Open pane: `open 'x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility'`.";

pub const SCREEN_RECORDING_REMEDIATION: &str =
    "Grant Screen Recording permission: System Settings → Privacy & Security → \
     Screen & System Audio Recording → add the app that launched gloamy, then \
     restart gloamy. Open pane: `open 'x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture'`.";

/// Probe whether the current process lineage holds macOS Accessibility trust.
///
/// On non-macOS returns `true`.
pub async fn has_accessibility_trust() -> bool {
    if !cfg!(target_os = "macos") {
        return true;
    }

    if let Some(cached) = cache().lock().await.read(CacheSlot::Accessibility) {
        return cached;
    }

    let value = probe_accessibility().await;
    cache().lock().await.write(CacheSlot::Accessibility, value);
    value
}

/// Probe whether the current process lineage holds macOS Screen Recording trust.
///
/// On non-macOS returns `true`.
pub async fn has_screen_recording_trust() -> bool {
    if !cfg!(target_os = "macos") {
        return true;
    }

    if let Some(cached) = cache().lock().await.read(CacheSlot::ScreenRecording) {
        return cached;
    }

    let value = probe_screen_recording().await;
    cache().lock().await.write(CacheSlot::ScreenRecording, value);
    value
}

/// Convenience: run both probes and return a snapshot.
pub async fn trust_status() -> MacTrustStatus {
    let (accessibility, screen_recording) =
        tokio::join!(has_accessibility_trust(), has_screen_recording_trust());
    MacTrustStatus {
        accessibility,
        screen_recording,
    }
}

#[cfg(target_os = "macos")]
async fn probe_accessibility() -> bool {
    run_trust_probe(
        "ObjC.import('ApplicationServices'); \
         $.AXIsProcessTrusted() ? 'yes' : 'no'",
    )
    .await
}

#[cfg(not(target_os = "macos"))]
async fn probe_accessibility() -> bool {
    true
}

#[cfg(target_os = "macos")]
async fn probe_screen_recording() -> bool {
    // CGPreflightScreenCaptureAccess is symbolicated in CoreGraphics on
    // macOS 10.15+. It never prompts; it just reports current trust state.
    run_trust_probe(
        "ObjC.import('CoreGraphics'); \
         (typeof $.CGPreflightScreenCaptureAccess === 'function' ? \
            ($.CGPreflightScreenCaptureAccess() ? 'yes' : 'no') : 'yes')",
    )
    .await
}

#[cfg(not(target_os = "macos"))]
async fn probe_screen_recording() -> bool {
    true
}

#[cfg(target_os = "macos")]
async fn run_trust_probe(script: &str) -> bool {
    let output = tokio::time::timeout(
        Duration::from_secs(3),
        tokio::process::Command::new("osascript")
            .arg("-l")
            .arg("JavaScript")
            .arg("-e")
            .arg(script)
            .output(),
    )
    .await;

    let Ok(Ok(output)) = output else {
        // If the probe itself fails (timeout, osascript missing) we
        // conservatively report `false`. The caller will surface an
        // actionable error instead of a silent fake success.
        return false;
    };

    if !output.status.success() {
        return false;
    }

    String::from_utf8_lossy(&output.stdout).trim() == "yes"
}

#[cfg(not(target_os = "macos"))]
async fn run_trust_probe(_script: &str) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn non_macos_probes_return_true() {
        if cfg!(target_os = "macos") {
            return;
        }
        assert!(has_accessibility_trust().await);
        assert!(has_screen_recording_trust().await);
        assert!(trust_status().await.all_granted());
    }

    #[test]
    fn cache_entry_expires_per_polarity() {
        let mut cache = TrustCache::default();
        cache.write(CacheSlot::Accessibility, true);
        assert_eq!(cache.read(CacheSlot::Accessibility), Some(true));
        // Manually age the entry beyond POSITIVE_TTL
        if let Some(entry) = cache.accessibility.as_mut() {
            entry.0 = Instant::now() - POSITIVE_TTL - Duration::from_secs(1);
        }
        assert_eq!(cache.read(CacheSlot::Accessibility), None);

        cache.write(CacheSlot::ScreenRecording, false);
        assert_eq!(cache.read(CacheSlot::ScreenRecording), Some(false));
        if let Some(entry) = cache.screen_recording.as_mut() {
            entry.0 = Instant::now() - NEGATIVE_TTL - Duration::from_secs(1);
        }
        assert_eq!(cache.read(CacheSlot::ScreenRecording), None);
    }

    #[test]
    fn remediation_strings_are_actionable() {
        assert!(ACCESSIBILITY_REMEDIATION.contains("Privacy_Accessibility"));
        assert!(SCREEN_RECORDING_REMEDIATION.contains("Privacy_ScreenCapture"));
    }
}
