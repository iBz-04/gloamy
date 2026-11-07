# Agnostic Security: Zero Impact on Portability

> ⚠️ **Status: Proposal / Roadmap**
>
> This document describes proposed approaches and may include hypothetical commands or config.
> For current runtime behavior, see [config-reference.md](config-reference.md), [operations-runbook.md](operations-runbook.md), and [troubleshooting.md](troubleshooting.md).

## Core Question: Will security features break supported host builds?

This proposal considers three maintained host targets: Linux amd64, Linux arm64, and macOS arm64. Security features must preserve those build contracts and keep platform-specific behavior explicit.

---

## 1. Build Speed: Feature-Gated Security

### Cargo.toml: Security Features Behind Features

```toml
[features]
default = ["basic-security"]

# Basic security (always on, zero overhead)
basic-security = []

# Platform-specific sandboxing (opt-in per platform)
sandbox-landlock = []   # Linux only
sandbox-firejail = []  # Linux only
sandbox-bubblewrap = []# macOS/Linux
sandbox-docker = []    # All platforms (heavy)

# Full security suite (for production builds)
security-full = [
    "basic-security",
    "sandbox-landlock",
    "resource-monitoring",
    "audit-logging",
]

# Resource & audit monitoring
resource-monitoring = []
audit-logging = []

# Development builds (fastest, no extra deps)
dev = []
```

### Build Commands (Choose Your Profile)

```bash
# Ultra-fast dev build (no security extras)
cargo build --profile dev

# Release build with basic security (default)
cargo build --release
# → Includes: allowlist, path blocking, injection protection
# → Excludes: Landlock, Firejail, audit logging

# Production build with full security
cargo build --release --features security-full
# → Includes: Everything

# Platform-specific sandbox only
cargo build --release --features sandbox-landlock  # Linux
cargo build --release --features sandbox-docker    # All platforms
```

### Conditional Compilation: Zero Overhead When Disabled

```rust
// src/security/mod.rs

#[cfg(feature = "sandbox-landlock")]
mod landlock;
#[cfg(feature = "sandbox-landlock")]
pub use landlock::LandlockSandbox;

#[cfg(feature = "sandbox-firejail")]
mod firejail;
#[cfg(feature = "sandbox-firejail")]
pub use firejail::FirejailSandbox;

// Always-include basic security (no feature flag)
pub mod policy;  // allowlist, path blocking, injection protection
```

**Result**: When features are disabled, the code isn't even compiled — **zero binary bloat**.

---

## 2. Pluggable Architecture: Security Is a Trait Too

### Security Backend Trait (Swappable Like Everything Else)

```rust
// src/security/traits.rs

#[async_trait]
pub trait Sandbox: Send + Sync {
    /// Wrap a command with sandbox protection
    fn wrap_command(&self, cmd: &mut std::process::Command) -> std::io::Result<()>;

    /// Check if sandbox is available on this platform
    fn is_available(&self) -> bool;

    /// Human-readable name
    fn name(&self) -> &str;
}

// No-op sandbox (always available)
pub struct NoopSandbox;

impl Sandbox for NoopSandbox {
    fn wrap_command(&self, _cmd: &mut std::process::Command) -> std::io::Result<()> {
        Ok(())  // Pass through unchanged
    }

    fn is_available(&self) -> bool { true }
    fn name(&self) -> &str { "none" }
}
```

### Factory Pattern: Auto-Select Based on Features

```rust
// src/security/factory.rs

pub fn create_sandbox() -> Box<dyn Sandbox> {
    #[cfg(feature = "sandbox-landlock")]
    {
        if LandlockSandbox::is_available() {
            return Box::new(LandlockSandbox::new());
        }
    }

    #[cfg(feature = "sandbox-firejail")]
    {
        if FirejailSandbox::is_available() {
            return Box::new(FirejailSandbox::new());
        }
    }

    #[cfg(feature = "sandbox-bubblewrap")]
    {
        if BubblewrapSandbox::is_available() {
            return Box::new(BubblewrapSandbox::new());
        }
    }

    #[cfg(feature = "sandbox-docker")]
    {
        if DockerSandbox::is_available() {
            return Box::new(DockerSandbox::new());
        }
    }

    // Fallback: always available
    Box::new(NoopSandbox)
}
```

**Just like providers, channels, and memory — security is pluggable!**

---

## 3. Supported Host Targets

### Build and runtime matrix

| Host label | Rust target | Runtime sandbox behavior |
|---|---|---|
| Linux amd64 | `x86_64-unknown-linux-gnu` | Landlock, then configured fallback |
| Linux arm64 | `aarch64-unknown-linux-gnu` | Landlock, then configured fallback |
| macOS arm64 | `aarch64-apple-darwin` | macOS runtime policy and configured fallback |

These are generic PC-class hosts. Embedded boards, MCU firmware, and other operating systems are outside the maintained support set.

### How It Works: Runtime Detection

```rust
// src/security/detect.rs

impl SandboxingStrategy {
    /// Choose best available sandbox AT RUNTIME
    pub fn detect() -> SandboxingStrategy {
        #[cfg(target_os = "linux")]
        {
            // Try Landlock first (kernel feature detection)
            if Self::probe_landlock() {
                return SandboxingStrategy::Landlock;
            }

            // Try Firejail (user-space tool detection)
            if Self::probe_firejail() {
                return SandboxingStrategy::Firejail;
            }
        }

        #[cfg(target_os = "macos")]
        {
            if Self::probe_bubblewrap() {
                return SandboxingStrategy::Bubblewrap;
            }
        }

        // Always available fallback
        SandboxingStrategy::ApplicationLayer
    }
}
```

**The three maintained targets share the same application policy. Runtime sandbox selection remains explicit and OS-dependent.**

---

## 5. Agnostic Swaps: Everything Remains Pluggable

### Gloamy's Core Promise: Swap Anything

```rust
// Providers (already pluggable)
Box<dyn Provider>

// Channels (already pluggable)
Box<dyn Channel>

// Memory (already pluggable)
Box<dyn MemoryBackend>

// Tunnels (already pluggable)
Box<dyn Tunnel>

// NOW ALSO: Security (newly pluggable)
Box<dyn Sandbox>
Box<dyn Auditor>
Box<dyn ResourceMonitor>
```

### Swap Security Backends via Config

```toml
# Use no sandbox (fastest, app-layer only)
[security.sandbox]
backend = "none"

# Use Landlock (Linux kernel LSM, native)
[security.sandbox]
backend = "landlock"

# Use Firejail (user-space, needs firejail installed)
[security.sandbox]
backend = "firejail"

# Use Docker (heaviest, most isolated)
[security.sandbox]
backend = "docker"
```

**Just like swapping OpenAI for Gemini, or SQLite for PostgreSQL.**

---

## 6. Dependency Impact: Minimal New Deps

### Current Dependencies (for context)
```
reqwest, tokio, serde, anyhow, uuid, chrono, rusqlite,
axum, tracing, opentelemetry, ...
```

### Security Feature Dependencies

| Feature | New Dependencies | Platform |
|---------|------------------|----------|
| **Landlock** | `landlock` crate (pure Rust) | Linux only |
| **Firejail** | None (external binary) | Linux only |
| **Bubblewrap** | None (external binary) | macOS/Linux |
| **Docker** | `bollard` crate (Docker API) | All platforms |
| **Memory monitoring** | None (std::alloc) | All platforms |
| **Audit logging** | None (already have hmac/sha2) | All platforms |

**Result**: Most features add **zero new Rust dependencies** — they either:
1. Use pure-Rust crates (landlock)
2. Wrap external binaries (Firejail, Bubblewrap)
3. Use existing deps (hmac, sha2 already in Cargo.toml)

---

## Summary: Core Value Propositions Preserved

| Value Prop | Before | After (with security) | Status |
|------------|--------|----------------------|--------|
| **<5MB RAM** | ✅ <5MB | ✅ <6MB (worst case) | ✅ Preserved |
| **<10ms startup** | ✅ <10ms | ✅ <15ms (detection) | ✅ Preserved |
| **3.4MB binary** | ✅ 3.4MB | ✅ 3.5MB (with all features) | ✅ Preserved |
| **Three supported host targets** | ✅ All | ✅ All | ✅ Preserved |
| **Pluggable everything** | ✅ Yes | ✅ Yes (security too) | ✅ Enhanced |
| **Cross-platform** | ✅ Yes | ✅ Yes | ✅ Preserved |

---

## The Key: Feature Flags + Conditional Compilation

```bash
# Developer build (fastest, no extra features)
cargo build --profile dev

# Standard release (your current build)
cargo build --release

# Production with full security
cargo build --release --features security-full

# Maintained host targets
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
cargo build --release --target aarch64-apple-darwin
```

**The three maintained host targets keep security behavior explicit and reproducible.**
