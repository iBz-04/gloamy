# Gloamy Troubleshooting

This guide focuses on common setup/runtime failures and fast resolution paths.

Last verified: **April 5, 2026**.

## Installation / Bootstrap

### `cargo` not found

Symptom:

- bootstrap exits with `cargo is not installed`

Fix:

```bash
./bootstrap.sh --install-rust
```

Or install from <https://rustup.rs/>.

### Missing system build dependencies

Symptom:

- build fails due to compiler or `pkg-config` issues

Fix:

```bash
./bootstrap.sh --install-system-deps
```

### Build fails on low-RAM / low-disk hosts

Symptoms:

- `cargo build --release` is killed (`signal: 9`, OOM killer, or `cannot allocate memory`)
- Build crashes after adding swap because disk space runs out

Why this happens:

- Runtime memory (<5MB for common operations) is not the same as compile-time memory.
- Full source build can require **2 GB RAM + swap** and **6+ GB free disk**.
- Enabling swap on a tiny disk can avoid RAM OOM but still fail due to disk exhaustion.

Preferred path for constrained machines:

```bash
./bootstrap.sh --prefer-prebuilt
```

Binary-only mode (no source fallback):

```bash
./bootstrap.sh --prebuilt-only
```

If you must compile from source on constrained hosts:

1. Add swap only if you also have enough free disk for both swap + build output.
1. Limit cargo parallelism:

```bash
CARGO_BUILD_JOBS=1 cargo build --release --locked
```

1. Reduce heavy features when Matrix is not required:

```bash
cargo build --release --locked --features hardware
```

1. Cross-compile on a stronger machine and copy the binary to the target host.

### Build is very slow or appears stuck

Symptoms:

- `cargo check` / `cargo build` appears stuck at `Checking gloamy` for a long time
- repeated `Blocking waiting for file lock on package cache` or `build directory`

Why this happens in Gloamy:

- Matrix E2EE stack (`matrix-sdk`, `ruma`, `vodozemac`) is large and expensive to type-check.
- TLS + crypto native build scripts (`aws-lc-sys`, `ring`) add noticeable compile time.
- `rusqlite` with bundled SQLite compiles C code locally.
- Running multiple cargo jobs/worktrees in parallel causes lock contention.

Fast checks:

```bash
cargo check --timings
cargo tree -d
```

The timing report is written to `target/cargo-timings/cargo-timing.html`.

Faster local iteration (when Matrix channel is not needed):

```bash
cargo check
```

This uses the lean default feature set and can significantly reduce compile time.

To build with Matrix support explicitly enabled:

```bash
cargo check --features channel-matrix
```

To build with Matrix + Lark + hardware support:

```bash
cargo check --features hardware,channel-matrix,channel-lark
```

Lock-contention mitigation:

```bash
pgrep -af "cargo (check|build|test)|cargo check|cargo build|cargo test"
```

Stop unrelated cargo jobs before running your own build.

### `gloamy` command not found after install

Symptom:

- install succeeds but shell cannot find `gloamy`

Fix:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
which gloamy
```

Persist in your shell profile if needed.

## Runtime / Gateway

### HostAgent perception fails on macOS

Symptoms:

- `gloamy agent` fails with `HostAgent runtime perception failed`
- interactive CLI turns stop before tool execution on macOS

Why this happens:

- the HostAgent runtime treats perception as a hard dependency
- missing Accessibility or Screen Recording permission is surfaced as a turn failure instead of silently degrading to an empty screen state

Fix:

1. Grant Accessibility permission to the app hosting the CLI session (for example Terminal, iTerm, Warp, VS Code, or Cursor)
1. Grant Screen Recording permission if screenshot capture is also blocked
1. Restart the affected app after changing macOS privacy settings
1. Re-run the same `gloamy agent` command

### HostAgent perception fails on Linux or Windows

Symptoms:

- `gloamy agent` fails with `HostAgent runtime perception failed`
- interactive CLI turns stop before tool execution

Why this happens:

- HostAgent runtime perception no longer accepts empty runtime state fallback
- screenshot capture and widget-tree runtime context must produce at least one usable signal
- missing screenshot backend binaries or blocked desktop session permissions can leave runtime signals empty

Fix:

1. On Linux, install at least one screenshot backend available in PATH (`gnome-screenshot`, `scrot`, or `import` from ImageMagick)
1. Ensure the process has desktop session access (for example X11/Wayland permissions in the active user session)
1. On Windows, run in an interactive desktop session with PowerShell available
1. Re-run `gloamy agent` after confirming host desktop capture is permitted

### `mac_automation click_at` is blocked by perception policy

Symptoms:

- runtime returns `Blocked by runtime policy: call perception_capture...`

Why this happens:

- coordinate clicks now require a successful `perception_capture` preflight with `include_widget_tree=true` and `include_ocr=true`
- partial or degraded perception output no longer satisfies the click preflight

Fix:

1. Call `perception_capture` with both `include_widget_tree=true` and `include_ocr=true`
1. Confirm the tool output reports both modalities as completed
1. Retry the `mac_automation click_at` action only after the successful preflight

### OCR extraction fails during `perception_capture`

Symptoms:

- runtime returns `OCR extraction failed: ...`
- `perception_capture` diagnostics show `ocr.completed = false`

Checks:

```bash
which tesseract
tesseract --version
```

Fix:

1. Install Tesseract if it is missing from the host
1. If the runtime needs a non-default language pack or tessdata path, pass the `ocr` object in the tool call:

```json
{
  "include_widget_tree": true,
  "include_ocr": true,
  "ocr": {
    "language": "eng",
    "psm": 11,
    "oem": 1
  }
}
```

1. Set `ocr.tessdata_dir` when traineddata files live outside the system default location

### Gateway unreachable

Checks:

```bash
gloamy status
gloamy doctor
```

Verify `~/.gloamy/config.toml`:

- `[gateway].host` (default `127.0.0.1`)
- `[gateway].port` (default `42617`)
- `allow_public_bind` only when intentionally exposing LAN/public interfaces

### Gateway opens in browser but no dashboard appears

Expected behavior:

- the legacy browser dashboard has been removed
- `gloamy gateway` now serves webhook, API, and WebSocket endpoints for desktop and external clients

What to do:

- use the desktop app for the primary UI
- use gateway routes only for integrations, automation, health checks, and API traffic

### Pairing / auth failures on webhook

Checks:

1. Ensure pairing completed (`/pair` flow)
2. Ensure bearer token is current
3. Re-run diagnostics:

```bash
gloamy doctor
```

## Channel Issues

### Telegram conflict: `terminated by other getUpdates request`

Cause:

- multiple pollers using same bot token

Fix:

- keep only one active runtime for that token
- stop extra `gloamy daemon` / `gloamy channel start` processes

### Channel unhealthy in `channel doctor`

Checks:

```bash
gloamy channel doctor
```

Then verify channel-specific credentials + allowlist fields in config.

## Service Mode

### Service installed but not running

Checks:

```bash
gloamy service status
```

Recovery:

```bash
gloamy service stop
gloamy service start
```

Linux logs:

```bash
journalctl --user -u gloamy.service -f
```

## Legacy Installer Compatibility

Both still work:

```bash
curl -fsSL https://raw.githubusercontent.com/iBz-04/gloamy/main/scripts/bootstrap.sh | bash
curl -fsSL https://raw.githubusercontent.com/iBz-04/gloamy/main/scripts/install.sh | bash
```

`install.sh` is a compatibility entry and forwards/falls back to bootstrap behavior.

## Still Stuck?

Collect and include these outputs when filing an issue:

```bash
gloamy --version
gloamy status
gloamy doctor
gloamy channel doctor
```

Also include OS, install method, and sanitized config snippets (no secrets).

## Related Docs

- [operations-runbook.md](operations-runbook.md)
- [one-click-bootstrap.md](one-click-bootstrap.md)
- [channels-reference.md](channels-reference.md)
- [network-deployment.md](network-deployment.md)
