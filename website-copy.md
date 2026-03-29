# Gloamy Website Copy

This file contains concise copy for the public website.
Scope: main landing page and download page.

## Main Page

### Hero

**Headline**

Gloamy is the Rust-first agent runtime for your desktop, terminal, channels, and local tools.

**Subheadline**

Run one agent across CLI, desktop, gateway, messaging channels, memory, and device workflows without rebuilding your stack around a single provider or UI.

**Primary CTA**

Download Gloamy

**Secondary CTA**

View Docs

### Short Value Strip

- Lightweight runtime with low overhead
- Secure-by-default execution model
- Swappable models, channels, memory, and tools
- CLI, desktop, gateway, and daemon modes

### What Gloamy Does

Gloamy gives you one runtime for real agent workflows.

You can use it to:

- chat directly in the terminal
- run a long-lived agent in the background
- connect Telegram, Discord, Slack, Matrix, WhatsApp, Email, IRC, Nostr, and more
- use different AI providers without rewriting the core setup
- execute tools and scripts with explicit permissions
- keep memory across sessions
- expose a local gateway for API, webhook, and websocket integrations
- connect hardware and device workflows when needed

### Why Gloamy

**Built for control, not hype**

Gloamy is designed for people who want an agent they can actually run, inspect, configure, and keep under control.

**Secure by default**

Localhost-first binding, explicit pairing, workspace-scoped file access, deny-by-default channel allowlists, and explicit tool security policies help keep the runtime constrained.

**Modular by design**

Providers, channels, tools, memory backends, observability, runtime adapters, and peripherals are all explicit subsystems instead of hidden product magic.

**One configuration model**

Use the same runtime across direct CLI use, background daemon mode, gateway access, and channel-based operation.

### Feature Section

**One agent, multiple ways to run**

- Interactive CLI for direct local use
- Daemon mode for always-on operation
- Gateway mode for HTTP, webhook, and websocket integrations
- Desktop app as the primary graphical interface

**Bring your own model setup**

- OpenAI and OpenAI-compatible endpoints
- Anthropic, Gemini, Groq, Mistral, DeepSeek, GLM, Qwen, and more
- Local or hosted model routing depending on your setup

**Choose the memory backend that fits**

- SQLite
- Markdown
- PostgreSQL
- Lucid
- No-op mode when you want stateless operation

### Use Cases

- Personal desktop agent with explicit local permissions
- Team-operated daemon connected to messaging channels
- API-connected assistant behind a local gateway
- Experimental hardware or device control workflows
- Multi-provider setup without provider lock-in

### Trust Section

Gloamy is built in Rust with explicit subsystem boundaries and a fail-closed bias where practical. It is meant to stay understandable under real operational pressure, not just demo well.

### Main Page Footer CTA

Install Gloamy, run onboarding, and start with a local CLI session in minutes.

Button: Download

Button: Read the Getting Started Guide

## Download Page

### Hero

**Headline**

Download Gloamy

**Subheadline**

Choose the install path that matches how you want to run Gloamy: package manager, bootstrap script, GitHub release asset, or source.

### Recommended Install Paths

**Option 1: Homebrew**

Best for macOS and Linuxbrew users who want the fastest standard install.

```bash
brew install gloamy
```

**Option 2: crates.io**

Best for Rust users who want the packaged CLI directly from crates.io.

```bash
cargo install gloamy --locked
gloamy onboard --interactive
```

**Option 3: Bootstrap Script**

Best when you want the guided install path with support for prebuilt binaries, source builds, Docker onboarding, and fresh-machine setup.

```bash
./bootstrap.sh
```

Useful variants:

```bash
./bootstrap.sh --prefer-prebuilt
./bootstrap.sh --prebuilt-only
./bootstrap.sh --install-system-deps --install-rust
./bootstrap.sh --interactive-onboard
```

**Option 4: Source**

Best for contributors, unreleased testing, and local development.

```bash
git clone https://github.com/iBz-04/gloamy.git
cd gloamy
cargo run -- onboard --interactive
```

### Supported Release Targets

Current release automation targets:

- Linux x86_64
- Linux arm64
- macOS arm64
- Windows x86_64

Additional Android release assets and setup documentation are available for supported Android targets.

### After Download

Run onboarding first:

```bash
gloamy onboard --interactive
```

Then choose how you want to run it:

```bash
gloamy agent
gloamy daemon
gloamy gateway
```

### Download Page FAQ

**What is the best default for most users?**

Homebrew or crates.io for packaged installs. Bootstrap is the best guided path when you want prebuilt detection or a more flexible setup flow.

**Should I install from source?**

Only if you want to develop, patch, or test unreleased changes.

**Does Gloamy have a GUI?**

Yes. The desktop app is the primary graphical interface. The legacy browser dashboard has been removed.

**Can I run it only in the terminal?**

Yes. `gloamy agent` is the direct CLI mode.

**Can I keep it running in the background?**

Yes. `gloamy daemon` is the long-lived runtime for channels, gateway, heartbeat, and scheduling.

### Download Page Footer CTA

Start with the standard install, run onboarding, and expand into channels, gateway, memory, and desktop workflows only when you need them.

Button: Install Now

Button: Open Documentation

## Optional Short Taglines

- One agent runtime. Multiple surfaces.
- Local-first agent control without provider lock-in.
- Rust-first automation for terminal, desktop, channels, and tools.
- Run agents with explicit permissions and real operational boundaries.

## Optional Homepage Section Titles

- One Runtime, Many Surfaces
- Built to Stay Under Control
- Connect Models, Channels, and Tools
- Start Local, Expand When Needed
- Download Fast, Configure Once
