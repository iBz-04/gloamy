# Specification: Codex App Server OAuth and Models

**Status:** implemented; OAuth and inference were not live-smoke-tested

## Problem Statement

Gloamy already has a direct OpenAI Codex provider with OAuth profiles, token refresh, and a Responses-style request path. Its model setup is separate from the locally installed Codex CLI, however: Gloamy keeps a short curated Codex model list and does not consume the model catalog supplied by that installation. Users who want models exposed by their installed Codex version must manually keep Gloamy's model choices current, and they cannot use Codex App Server's managed ChatGPT OAuth account as a Gloamy provider.

## Solution

Add a separate `codex-app-server` provider that uses the locally installed `codex app-server` for ChatGPT OAuth, model catalog discovery, and inference. Gloamy exposes the account through its auth commands, supports model refresh and selection using the catalog returned by the installed server, and can send an agent request through that provider. The existing `openai-codex` provider and its Gloamy-managed OAuth profiles continue to work unchanged.

## User Stories

1. As a Gloamy operator, I want to select `codex-app-server` as a provider, so that I can use the Codex installation's ChatGPT OAuth account and model support.
2. As a Gloamy operator, I want `gloamy auth login --provider codex-app-server` to start ChatGPT OAuth through the local Codex App Server, so that I do not need to copy OAuth credentials into Gloamy.
3. As a Gloamy operator, I want the app-server browser login flow to report completion or failure clearly, so that I know whether authentication succeeded.
4. As a Gloamy operator on a device without a convenient browser, I want the provider's device-code login option to work through the App Server when supported, so that I can authorize the account from another device.
5. As a Gloamy operator, I want the auth status and list commands to show whether the App Server has a ChatGPT account, so that I can diagnose account setup without exposing credentials.
6. As a Gloamy operator, I want auth refresh and logout for `codex-app-server` to use the App Server's account operations, so that token lifecycle stays with the installation that owns the credentials.
7. As a Gloamy operator, I want unsupported per-profile operations to fail with a clear explanation, so that Gloamy does not imply it can switch Codex-managed accounts when it cannot.
8. As a Gloamy operator, I want a missing or incompatible local Codex installation to produce an actionable error, so that the provider does not silently fall back to a different authentication path.
9. As a Gloamy operator, I want `gloamy models refresh --provider codex-app-server` to read model IDs from the installed App Server, so that model choices match the Codex installation.
10. As a Gloamy operator, I want model refresh to handle every page of the App Server catalog, so that large catalogs are not silently truncated.
11. As a Gloamy operator, I want hidden models excluded from the normal selectable catalog, so that the model list reflects the App Server's standard picker.
12. As a Gloamy operator, I want the discovered model IDs to use the exact values accepted by Codex App Server, so that selecting `gpt-6.1-sol`, `gpt-6-astra`, `gpt-6-terra`, `gpt-6-luna`, or another returned model does not require aliases or manual renaming.
13. As a Gloamy operator, I want model refresh to cache discovered models using Gloamy's existing cache behavior, so that model selection remains available between refreshes.
14. As a Gloamy operator, I want to choose a discovered model through existing model configuration and CLI options, so that the new provider fits the current provider and routing workflow.
15. As a Gloamy operator, I want a simple agent prompt to run through the Codex App Server provider, so that I can use the selected Codex model from Gloamy.
16. As a Gloamy operator, I want prior conversation messages and system instructions preserved when Gloamy sends a request, so that the new provider fits the existing agent loop.
17. As a Gloamy operator, I want Gloamy's existing prompt-guided tool behavior to remain the source of tool actions, so that selecting this provider does not unexpectedly grant Codex local execution access.
18. As a Gloamy operator, I want the App Server request to run in an ephemeral, restricted context with no Gloamy workspace access, so that provider inference does not persist extra conversation state or inspect local project files.
19. As a Gloamy operator, I want App Server errors, failed turns, and interrupted turns to return explicit provider errors, so that Gloamy does not treat incomplete output as a successful answer.
20. As a Gloamy operator, I want App Server protocol errors and process failures to avoid printing OAuth tokens or raw credential payloads, so that diagnostics remain safe to share.
21. As an existing Gloamy user, I want the current `openai-codex` provider, profiles, and configured model IDs to keep their existing behavior, so that adopting App Server support is opt-in.
22. As a Gloamy maintainer, I want the provider to use existing async process and serialization dependencies, so that this feature does not add a broad dependency or configuration surface.

## Implementation Decisions

- Add `codex-app-server` as a distinct provider key and leave the current `openai-codex` provider, aliases, and Gloamy OAuth profile behavior intact.
- Resolve the local `codex` executable through the process environment and use its stdio App Server transport. Fail explicitly when the executable or required protocol methods are unavailable.
- Let Codex App Server own ChatGPT OAuth tokens and their refresh lifecycle. Gloamy must not copy raw App Server credentials into its own profile store or logs.
- Use App Server account login, account status, refresh, and logout operations for the new provider. Treat the Codex installation's active account as the provider account; do not emulate Gloamy's named-profile switching.
- Use the App Server model catalog, following its pagination and visibility fields. Preserve returned model identifiers exactly in Gloamy's existing model cache and selection paths.
- Treat the model catalog as a selectable catalog only. A completed inference request is the signal that the selected model is usable by the current account.
- Implement the provider through the existing `Provider` trait and factory. Keep native tool calling disabled, preserve Gloamy's existing prompt-guided tool contract, and return an error unless the App Server turn reaches a successful terminal state.
- Start App Server calls with ephemeral threads, a restricted temporary working directory, no configured MCP servers or optional tool integrations, and a sanitized child environment. Do not expose the Gloamy workspace to the App Server provider.
- Treat the App Server RPC contract as experimental and supplied by the installed Codex CLI version; surface incompatibility errors rather than falling back to a different auth or inference path.
- Keep the App Server JSON-RPC boundary in a focused client module shared by auth commands, model discovery, and provider inference. Reuse existing Tokio process and serde JSON support rather than adding a new dependency.
- Update provider and command references for the new provider. Do not change configuration schema keys or the default provider as part of this feature.

## Testing Decisions

- Tests should assert externally observable CLI and provider behavior, not internal JSON-RPC helper structure.
- Use a deterministic fake `codex` executable at the process boundary to exercise browser/device-code login completion, account status/logout, paginated model discovery, successful streamed inference, failed turns, and process/protocol failures without contacting OpenAI.
- Exercise the highest practical seam: the auth command, `models refresh`, and an agent request configured with `codex-app-server`; confirm that a returned model identifier can be selected and used.
- Test the provider factory and model-cache behavior using the same provider ID that a user configures.
- Existing prior art includes OpenAI OAuth flow coverage, provider factory coverage, and model refresh/cache behavior in the current provider and onboarding subsystems.

## Out of Scope

- Replacing or migrating the existing `openai-codex` provider or its Gloamy-managed OAuth profiles.
- Changing the default provider or modifying OpenAI API-key authentication.
- Reading or importing the Codex installation's raw OAuth token file.
- Implementing Codex-native tools, MCP bridges, file editing, shell execution, browser access, or multi-agent behavior inside Gloamy.
- Guaranteeing that a listed model is entitled for the signed-in account before the first inference request.
- Adding Gloamy configuration keys for the Codex binary, model catalog, or App Server process.
- Upgrading unrelated dependencies as part of the provider implementation.

## Further Notes

The existing provider architecture is trait- and factory-based. OpenAI Codex OAuth currently uses Gloamy auth profiles, while onboarding and model refresh use a separate curated/live-model path. The new provider must remain additive so existing configurations do not change.

Codex App Server exposes an experimental JSON-RPC protocol over newline-delimited stdio. Its model catalog can be bundled or cached and is not an account entitlement check. The provider must therefore report failed or interrupted inference distinctly and must not claim that a model is available solely because it appeared in `model/list`.
