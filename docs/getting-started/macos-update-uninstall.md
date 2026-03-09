# macOS Update and Uninstall Guide

This page documents supported update and uninstall procedures for Gloamy on macOS (OS X).

Last verified: **February 22, 2026**.

## 1) Check current install method

```bash
which gloamy
gloamy --version
```

Typical locations:

- Homebrew: `/opt/homebrew/bin/gloamy` (Apple Silicon) or `/usr/local/bin/gloamy` (Intel)
- Cargo/bootstrap/manual: `~/.cargo/bin/gloamy`

If both exist, your shell `PATH` order decides which one runs.

## 2) Update on macOS

### A) Homebrew install

```bash
brew update
brew upgrade gloamy
gloamy --version
```

### B) Clone + bootstrap install

From your local repository checkout:

```bash
git pull --ff-only
./bootstrap.sh --prefer-prebuilt
gloamy --version
```

If you want source-only update:

```bash
git pull --ff-only
cargo install --path . --force --locked
gloamy --version
```

### C) Manual prebuilt binary install

Re-run your download/install flow with the latest release asset, then verify:

```bash
gloamy --version
```

## 3) Uninstall on macOS

### A) Stop and remove background service first

This prevents the daemon from continuing to run after binary removal.

```bash
gloamy service stop || true
gloamy service uninstall || true
```

Service artifacts removed by `service uninstall`:

- `~/Library/LaunchAgents/com.gloamy.daemon.plist`

### B) Remove the binary by install method

Homebrew:

```bash
brew uninstall gloamy
```

Cargo/bootstrap/manual (`~/.cargo/bin/gloamy`):

```bash
cargo uninstall gloamy || true
rm -f ~/.cargo/bin/gloamy
```

### C) Optional: remove local runtime data

Only run this if you want a full cleanup of config, auth profiles, logs, and workspace state.

```bash
rm -rf ~/.gloamy
```

## 4) Verify uninstall completed

```bash
command -v gloamy || echo "gloamy binary not found"
pgrep -fl gloamy || echo "No running gloamy process"
```

If `pgrep` still finds a process, stop it manually and re-check:

```bash
pkill -f gloamy
```

## Related docs

- [One-Click Bootstrap](../one-click-bootstrap.md)
- [Commands Reference](../commands-reference.md)
- [Troubleshooting](../troubleshooting.md)
