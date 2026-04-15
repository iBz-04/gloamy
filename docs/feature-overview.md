# Feature Overview

This page gives a high-level map of what Gloamy can do beyond authentication.

## Core Product Areas

### CLI and onboarding

Gloamy provides a Rust-first CLI for first-time setup, direct agent sessions, daemon mode, and operational commands.

Typical entry points:

- `gloamy onboard` — guided setup and workspace initialization
- `gloamy agent` — direct interactive or one-shot chat sessions
- `gloamy daemon` — long-running runtime for channels and gateway services
- `gloamy gateway` — local HTTP/WebSocket gateway for external integrations

### Providers

Gloamy supports multiple model providers and provider styles:

- direct providers such as OpenAI, Anthropic, Gemini, Groq, Mistral, DeepSeek, Venice, GLM, and Qwen
- OpenAI-compatible custom endpoints
- subscription-style auth flows for supported providers
- profile-based switching when you want separate credentials for different tasks

### Channels

The runtime can stay online and interact through messaging channels such as Telegram, Discord, Slack, Matrix, WhatsApp, iMessage, Email, IRC, and Nostr.

Channel setup is stored in config, and the daemon uses that configuration at runtime.

### Tools and automation

Gloamy can run tools and scripts on your machine or online, with explicit permissions and policy boundaries.

Common capability groups include:

- shell execution
- browser automation
- file and workspace access
- memory and retrieval tooling
- desktop/runtime perception for host interactions

### Memory

Gloamy supports local-first memory backends for persistence and retrieval, including:

- SQLite
- Markdown
- Lucid
- PostgreSQL
- no-op memory

### Security and runtime control

Gloamy is designed to fail closed where practical.

Important security and runtime properties include:

- localhost-first gateway binding by default
- explicit pairing for gateway auth flows
- workspace-scoped file access
- deny-by-default channel allowlists
- explicit policy for tool and command execution

### Desktop app

The repository also includes a desktop app in `desktop/`, built with Tauri and Vue 3.

It is the main UI for people who want a graphical workflow rather than the terminal-first path.

### Hardware and peripherals

Gloamy also includes hardware/peripheral integration paths for boards and device workflows.

These are documented separately because they are a distinct subsystem with different setup and safety constraints.

## Best entry points

- Quick start: [`../README.md`](../README.md)
- Command lookup: [`commands-reference.md`](commands-reference.md)
- Provider details: [`providers-reference.md`](providers-reference.md)
- Channel details: [`channels-reference.md`](channels-reference.md)
- Config details: [`config-reference.md`](config-reference.md)
- Docs hub: [`README.md`](README.md)
- Full TOC: [`SUMMARY.md`](SUMMARY.md)
