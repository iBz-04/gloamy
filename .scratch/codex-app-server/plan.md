# Implementation Plan: Codex App Server OAuth and Models

**Status:** implemented; OAuth and inference were not live-smoke-tested

## Scope

Add the opt-in `codex-app-server` provider through the installed Codex CLI's stdio App Server. Keep the existing `openai-codex` provider, its OAuth profiles, and their behavior intact. Do not add dependencies or config keys.

## Sequence

1. **Build the local App Server client**
   - Resolve `codex` from `PATH`, start `codex app-server` over stdio, and negotiate JSON-RPC.
   - Implement account login/status/refresh/logout, visible paginated model listing, and ephemeral inference.
   - Keep credentials inside Codex, avoid protocol payload logging, and constrain inference to an empty private working directory with read-only permissions and disabled optional tools/network access.
2. **Expose authentication in Gloamy CLI**
   - Add browser and device-code login through `account/login/start` and wait for the matching completion notification.
   - Show safe account status alongside Gloamy-managed profiles.
   - Route refresh/logout to Codex App Server; clearly reject profile switching and token-paste operations for this provider.
3. **Add provider and model selection**
   - Implement the existing `Provider` trait and register the separate provider key.
   - Preserve system/history messages and normalized image inputs while keeping tool execution with Gloamy.
   - Fetch exact visible model IDs from `model/list`, including pagination, and use `default` to defer to Codex's installation default.
   - Add onboarding choices and model refresh/cache support without an API-key prompt.
4. **Document the user contract**
   - Update provider, auth-command, and model-refresh references with setup commands, account ownership, model catalog limits, and logout effects.
5. **Review and validate**
   - Inspect the full diff for compatibility, security boundaries, and accidental data exposure.
   - Run formatting and compile/static checks; do not run tests unless requested.
   - Record commands, results, and any App Server protocol limitations in the handoff.

## Acceptance Criteria

- `codex-app-server` is selectable in config and provider discovery and needs no Gloamy API key.
- Login, account status, refresh, and logout use the local App Server; raw tokens are neither copied into Gloamy profiles nor printed.
- Model refresh returns the installed server's visible exact model IDs, handles pagination, and stores them using Gloamy's existing cache.
- Inference uses the selected exact model ID or the installation default, returns only successful completed turns, and does not expose the Gloamy workspace to Codex tools.
- Existing `openai-codex` provider/profile behavior and configuration remain unchanged.
- Runtime docs explain setup, model selection, account ownership, and logout scope.

## Validation

- `cargo fmt --all -- --check` — passed.
- `cargo check --locked` — passed; two warnings remain in untouched modules (`src/approval/mod.rs` and `src/tools/browser_services.rs`).
- `cargo clippy --lib --bin gloamy --locked -- -D warnings` — not clean because the existing baseline reports warnings across unrelated modules; no findings were reported in the new App Server/provider/onboarding/main files.
- `codex app-server ... --print-startup-config` — passed on installed Codex CLI 0.166.0 with the configured MCP/web overrides and disabled optional tools; this checks local configuration parsing, not OAuth or inference.
- `git diff --check` — passed.
- Tests were not run. Live Codex OAuth and model inference were not exercised.
