<div align="center">
   <h1>Gloamy</h1>
</div>

<p align="center">
  <img src="https://res.cloudinary.com/diekemzs9/image/upload/v1774631483/hairy_gloam-3_den5zj.png" alt="Gloamy Mascot" width="250"/>
</p>


# Intro
When the claws 🦞 are asleep the gloamies 🦉 come out to play !

Hi I'm Gloamy, I execute tasks on your behalf without deleting your stuff, leaking your private business, oh and I'm lightweight unlike that damn lobster  (openclaw).

# Overview
![Desktop](/mobile_shots.png)
![Desktop](/desktop.png)


Gloamy is built around explicit subsystem contracts:

- `Provider` for model backends
- `Channel` for messaging platforms
- `Tool` for execution surfaces
- `Memory` for persistence and recall
- `Observer` for observability
- `RuntimeAdapter` for runtime isolation
- `Peripheral` for boards and device integrations

The project goal is simple: one runtime, one configuration model, and swappable integrations without rewriting the core agent loop.

## What Gloamy Does

Gloamy can:

- chat with you in your terminal
- stay running in the background to handle messages
- control apps on your pc with permissions
- let desktop and external API clients connect through the gateway
- use different AI models to answer questions
- run tools and scripts on your computer or online, safely
- remember things between sessions
- connect to devices and smart hardware

Out of the box, the runtime supports:

- local CLI usage
- channel-based operation through providers such as Telegram, Discord, Slack, Matrix, WhatsApp, iMessage, Email, IRC, Nostr, and others
- OpenAI-compatible and non-OpenAI model providers
- SQLite, Markdown, Lucid, PostgreSQL, and no-op memory modes

## Project Shape

High-level repository map:

- [`src/main.rs`](src/main.rs): CLI entrypoint and command routing
- [`src/lib.rs`](src/lib.rs): shared exports and command enums
- [`src/agent/`](src/agent): orchestration loop
- [`src/config/`](src/config): config schema, loading, merging, env overrides
- [`src/providers/`](src/providers): model provider implementations and factory wiring
- [`src/channels/`](src/channels): channel integrations
- [`src/tools/`](src/tools): tool execution surface
- [`src/memory/`](src/memory): memory backends
- [`src/security/`](src/security): policy, pairing, secret handling
- [`src/gateway/`](src/gateway): HTTP and websocket gateway
- [`src/runtime/`](src/runtime): runtime adapters
- [`src/peripherals/`](src/peripherals): hardware integrations
- [`desktop/`](desktop): Tauri + Vue desktop application
- [`docs/`](docs): operator, reference, and contribution docs

## Key Properties

- Trait-driven architecture
- Secure-by-default runtime behavior
- Small binary and low runtime overhead
- Explicit config and CLI contracts
- Swappable providers, channels, memory backends, and tools
- Deterministic, Rust-first deployment model

## Optional Robot

The repository also includes [`crates/robot`](crates/robot), a standalone robot-control crate for motion, sensing, speech, vision, simple robot expression, and safety-gated drive control.

Use it when you want to experiment with Raspberry Pi or robot hardware integrations without wiring those surfaces directly into the main runtime. It is a workspace member, but it is not auto-registered into `gloamy`'s core tool factory.

Start here:

- [`crates/robot/README.md`](crates/robot/README.md) for the crate surface and integration model
- [`crates/robot/robot.toml`](crates/robot/robot.toml) for the sample configuration
- [`crates/robot/PI5_SETUP.md`](crates/robot/PI5_SETUP.md) for Raspberry Pi 5 setup notes

## Quick Start

### Prerequisites

You need:

- a working Rust toolchain
- standard platform build tools
- an API key or local model endpoint, depending on your provider

On macOS:

```bash
xcode-select --install
```

On Debian or Ubuntu:

```bash
sudo apt install build-essential pkg-config
```

Install Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
```

Verify:

```bash
rustc --version
cargo --version
```

### Install

Install from [crates.io](https://crates.io/crates/gloamy):

```bash
cargo install gloamy --locked
```

Then run full onboarding:

```bash
gloamy onboard --interactive
```

If you prefer to run from source, clone the repository and start onboarding:

```bash
git clone https://github.com/iBz-04/gloamy.git
cd gloamy
cargo run -- onboard --interactive
```

The crates.io install is the recommended default for end users. The source path is better when you want to develop, patch, or test unreleased changes.

The full onboarding flow lets you:

- choose your provider and default model
- configure channels
- set memory behavior
- create workspace identity files
- avoid partial setup drift

After onboarding, start a direct CLI session:

```bash
cargo run -- agent
```

If you want persistent channel operation after setup:

```bash
cargo run -- daemon
```

### Bootstrap Script

If you prefer the bootstrap path:

```bash
./bootstrap.sh --interactive-onboard
```

Useful bootstrap variants:

```bash
./bootstrap.sh
./bootstrap.sh --install-system-deps --install-rust
./bootstrap.sh --prefer-prebuilt
./bootstrap.sh --prebuilt-only
./bootstrap.sh --docker
```

Reference: [`docs/one-click-bootstrap.md`](docs/one-click-bootstrap.md)

### Non-Interactive Setup

If you already know exactly what you want, you can skip the full wizard:

```bash
cargo run -- onboard --api-key YOUR_OPENAI_KEY --provider openai --model gpt-5-mini
```

This path is faster, but the interactive onboarding flow is the better default for first setup.

## Migrating from OpenClaw

Gloamy can import memory from an existing OpenClaw workspace into your current Gloamy workspace.

Preview the migration first:

```bash
cargo run -- migrate openclaw --dry-run
```

Run the import:

```bash
cargo run -- migrate openclaw
```

Use a custom OpenClaw workspace path if needed:

```bash
cargo run -- migrate openclaw --source /path/to/openclaw/workspace
```

What the migration does:

- reads importable memory from `~/.openclaw/workspace` by default
- imports entries from `memory/brain.db`, `MEMORY.md`, and `memory/*.md`
- skips unchanged entries on re-run and renames conflicting keys deterministically
- creates a backup of the target Gloamy memory before writing

What it does not do:

- it does not migrate arbitrary workspace files
- it does not convert OpenClaw config into Gloamy `config.toml`

If you want a safe preview of the candidate entries without writing data, use `--dry-run`.

## Desktop App

This repository also includes a desktop application in [`desktop/`](desktop), built with Tauri (Rust backend) and Vue 3 (frontend).

The legacy browser dashboard has been removed. Use the desktop app for the primary UI, and use the gateway for webhook/API access.

If you want to run the desktop UI locally:

```bash
cd desktop
pnpm install
pnpm tauri dev
```

For desktop-specific setup, development, and packaging details, see [`desktop/README.md`](desktop/README.md).

## Running Modes

### Interactive CLI

Start a direct chat session:

```bash
cargo run -- agent
```

Run a one-shot prompt:

```bash
cargo run -- agent -m "Summarize today's logs"
```

Override provider and model for one run:

```bash
cargo run -- agent --provider openai --model gpt-5-mini -m "hello"
```

### Daemon

Run the long-lived runtime:

```bash
cargo run -- daemon
```

The daemon starts:

- configured channels
- gateway server
- heartbeat
- scheduler

Use this mode when you want Telegram or other channels to stay online.

### Gateway

Run only the local gateway:

```bash
cargo run -- gateway
```

The gateway exposes HTTP, webhook, and websocket endpoints for external integrations. It no longer serves a browser dashboard.

### Channels Only

Run the configured channels without the full daemon stack:

```bash
cargo run -- channel start
```

## First-Time Configuration

The primary setup command is:

```bash
gloamy onboard
```

Common variants:

```bash
gloamy onboard --interactive
gloamy onboard --channels-only
gloamy onboard --force
gloamy onboard --api-key YOUR_KEY --provider openai --model gpt-5-mini
```

If you already have a config and only want to wire channels, use:

```bash
gloamy onboard --channels-only
```

## Channels

Gloamy supports multiple inbound and outbound channels. Channel setup is stored in config, and the daemon uses that config at runtime.

Typical flow:

1. Run `gloamy onboard --channels-only`
2. Configure Telegram, Discord, Slack, WhatsApp, or another channel
3. Start `gloamy daemon`
4. Verify with `gloamy status` and `gloamy channel doctor`

Important operational note:

- `gloamy agent` is for direct CLI use
- `gloamy daemon` is what you run for persistent channel operation

Canonical reference: [`docs/channels-reference.md`](docs/channels-reference.md)

## Configuration

Default config location:

```text
~/.gloamy/config.toml
```

Default workspace:

```text
~/.gloamy/workspace
```

Minimal example:

```toml
api_key = "sk-..."
default_provider = "openai"
default_model = "gpt-5-mini"
default_temperature = 0.7

[memory]
backend = "sqlite"
auto_save = true
embedding_provider = "none"
vector_weight = 0.7
keyword_weight = 0.3
```

Notes:

- `default_provider` controls the main runtime provider
- `default_model` controls the default model for CLI, channel, and daemon flows
- `default_temperature` may be ignored by some providers or models
- several settings can also be overridden by environment variables

Canonical reference: [`docs/config-reference.md`](docs/config-reference.md)

## Provider Model

Gloamy supports:

- direct providers such as OpenAI, Anthropic, Gemini, Groq, Mistral, DeepSeek, Venice, GLM, Qwen, and others
- OpenAI-compatible custom endpoints
- provider aliases and routed model configuration
- subscription-native auth flows for supported providers

Examples:

```bash
gloamy agent --provider openai --model gpt-5-mini -m "hello"
gloamy agent --provider anthropic -m "hello"
gloamy agent --provider openai-codex -m "hello"
```

References:

- [`docs/providers-reference.md`](docs/providers-reference.md)
- [`docs/custom-providers.md`](docs/custom-providers.md)

## Security Model

Gloamy is designed to fail closed where practical.

Important defaults:

- localhost-first binding
- explicit pairing for gateway auth flows
- workspace-scoped file access
- deny-by-default channel allowlists
- encrypted secret support
- explicit security policy for tool and command execution

Operational guidance:

- do not expose the gateway directly without understanding the bind and tunnel settings
- use allowlists for channels instead of wide-open routing
- keep secrets in config or environment variables, not in workspace files
- review tool and command permissions before enabling broad autonomy

References:

- [`docs/security/README.md`](docs/security/README.md)
- [`docs/operations-runbook.md`](docs/operations-runbook.md)
- [`docs/troubleshooting.md`](docs/troubleshooting.md)

## Memory

Supported memory backends include:

- SQLite
- Markdown
- Lucid
- PostgreSQL
- `none`

SQLite is the usual default because it gives:

- local persistence
- keyword and vector search support
- practical low-friction setup

Reference: [`docs/config-reference.md`](docs/config-reference.md)

## Commands You Will Actually Use

```bash
gloamy status
gloamy doctor
gloamy channel doctor
gloamy agent
gloamy daemon
gloamy gateway
gloamy onboard --interactive
gloamy onboard --channels-only
gloamy auth status
gloamy service install
gloamy service status
gloamy service restart
```

Canonical command reference: [`docs/commands-reference.md`](docs/commands-reference.md)

## Development

Recommended local validation:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Preferred full local validation path when available:

```bash
./dev/ci.sh all
```

Useful development commands:

```bash
cargo run -- status
cargo run -- doctor
cargo run -- agent -m "hello"
cargo run -- daemon
```

If you are working on docs, start here:

- [`docs/README.md`](docs/README.md)
- [`docs/SUMMARY.md`](docs/SUMMARY.md)

## Documentation Map

Start from the docs hub:

- Docs hub: [`docs/README.md`](docs/README.md)
- Unified TOC: [`docs/SUMMARY.md`](docs/SUMMARY.md)
- Getting started: [`docs/getting-started/README.md`](docs/getting-started/README.md)
- Reference: [`docs/reference/README.md`](docs/reference/README.md)
- Operations: [`docs/operations/README.md`](docs/operations/README.md)
- Security: [`docs/security/README.md`](docs/security/README.md)
- Hardware: [`docs/hardware/README.md`](docs/hardware/README.md)
- Contributing workflow: [`docs/pr-workflow.md`](docs/pr-workflow.md)

High-signal runtime references:

- [`docs/commands-reference.md`](docs/commands-reference.md)
- [`docs/providers-reference.md`](docs/providers-reference.md)
- [`docs/channels-reference.md`](docs/channels-reference.md)
- [`docs/config-reference.md`](docs/config-reference.md)
- [`docs/operations-runbook.md`](docs/operations-runbook.md)
- [`docs/troubleshooting.md`](docs/troubleshooting.md)

## Official Repository

Official source of truth:

- [https://github.com/iBz-04/gloamy](https://github.com/iBz-04/gloamy)

If you encounter impersonation or a misleading fork, open an issue in the official repository.

## Contributing

If you want to contribute:

- review [`AGENTS.md`](AGENTS.md) for repository engineering expectations
- read [`docs/pr-workflow.md`](docs/pr-workflow.md)
- use [`docs/reviewer-playbook.md`](docs/reviewer-playbook.md) for review standards

Good entry points:

- new provider in `src/providers/`
- new channel in `src/channels/`
- new tool in `src/tools/`
- new memory backend in `src/memory/`
- new observer in `src/observability/`

## License

Gloamy is licensed under [MIT](LICENSE).
