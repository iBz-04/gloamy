# ADR 0002: Codex App Server owns its OAuth account

- **Status:** Accepted
- **Date:** 2026-11-03

## Context

Gloamy already has an `openai-codex` provider that stores OAuth profiles in Gloamy and sends requests through a direct Responses API path. Codex App Server offers a separate local JSON-RPC interface with its own ChatGPT sign-in, model catalog, and inference lifecycle. Copying the App Server's tokens into Gloamy would create a second credential owner and couple Gloamy to Codex's private credential format.

## Decision

- Add `codex-app-server` as a distinct provider. Keep the existing `openai-codex` provider and Gloamy-managed profiles unchanged.
- Use the installed `codex app-server` over stdio for login, account status, proactive refresh, logout, model discovery, and inference. Let Codex own and persist its OAuth credentials. Gloamy does not read or store the raw token.
- Use the account selected by the Codex CLI's effective `CODEX_HOME`. Logout therefore signs out other Codex CLI clients that share that home.
- Follow the App Server model catalog's pagination and visibility fields. A catalog entry is a selectable model ID, not proof that an account can use it; require a completed inference turn for that.
- Start each inference in a temporary working directory and an ephemeral read-only thread. Disable shell, browser, image-viewing, app, plugin, multi-agent, skill-search, and web-search capabilities; configure no MCP servers or environments. Keep Gloamy's tool execution in Gloamy.
- Treat App Server protocol errors and failed or interrupted turns as errors. Do not silently fall back to the direct provider or expose raw protocol payloads in diagnostics.

## Consequences

- Users must install a compatible Codex CLI and sign in through `gloamy auth login --provider codex-app-server`.
- Codex App Server's protocol is experimental and tied to the installed CLI version. Gloamy fails explicitly when an operation is unsupported.
- The two OpenAI Codex providers have separate authentication and model-selection behavior; users choose `codex-app-server` when they want Codex-owned credentials and the local App Server catalog.
- The account lifecycle and tool boundary require focused fake-process tests because they cross a subprocess and JSON-RPC boundary.

## References

- [Codex App Server protocol](https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md)
- [Sign in with ChatGPT for Codex App Server](https://developers.openai.com/siwc/token-sharing-open-source/codex-app-server)
