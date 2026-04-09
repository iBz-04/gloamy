# Gloamy Commands Reference

This reference is derived from the current CLI surface (`gloamy --help`).

Last verified: **April 5, 2026**.

## Top-Level Commands

| Command | Purpose |
|---|---|
| `onboard` | Initialize workspace/config quickly or interactively |
| `agent` | Run interactive chat or single-message mode |
| `auth` | Manage authentication profiles and tokens |
| `gateway` | Start webhook, API, and WebSocket gateway for external clients |
| `daemon` | Start supervised runtime (gateway + channels + optional heartbeat/scheduler) |
| `service` | Manage user-level OS service lifecycle |
| `doctor` | Run diagnostics and freshness checks |
| `status` | Print current configuration and system summary |
| `estop` | Engage/resume emergency stop levels and inspect estop state |
| `cron` | Manage scheduled tasks |
| `models` | Refresh provider model catalogs and set default model |
| `providers` | List provider IDs, aliases, and active provider |
| `channel` | Manage channels and channel health checks |
| `integrations` | Inspect integration details |
| `skills` | List/install/remove skills |
| `migrate` | Import from external runtimes (currently OpenClaw) |
| `config` | Export machine-readable config schema |
| `completions` | Generate shell completion scripts to stdout |
| `hardware` | Discover and introspect USB hardware |
| `peripheral` | Configure and flash peripherals |

## Command Groups

### `onboard`

- `gloamy onboard`
- `gloamy onboard --interactive`
- `gloamy onboard --channels-only`
- `gloamy onboard --force`
- `gloamy onboard --api-key <KEY> --provider <ID> --memory <sqlite|lucid|markdown|none>`
- `gloamy onboard --api-key <KEY> --provider <ID> --model <MODEL_ID> --memory <sqlite|lucid|markdown|none>`
- `gloamy onboard --api-key <KEY> --provider <ID> --model <MODEL_ID> --memory <sqlite|lucid|markdown|none> --force`

`onboard` safety behavior:

- If `config.toml` already exists and you run `--interactive`, onboarding now offers two modes:
  - Full onboarding (overwrite `config.toml`)
  - Provider-only update (update provider/model/API key while preserving existing channels, tunnel, memory, hooks, and other settings)
- In non-interactive environments, existing `config.toml` causes a safe refusal unless `--force` is passed.
- Use `gloamy onboard --channels-only` when you only need to rotate channel tokens/allowlists.

### `agent`

- `gloamy agent`
- `gloamy agent -m "Hello"`
- `gloamy agent --provider <ID> --model <MODEL> --temperature <0.0-2.0>`
- `gloamy agent --peripheral <board:path>`

Tip:

- In interactive chat, you can ask for route changes in natural language (for example “conversation uses kimi, coding uses gpt-5.3-codex”); the assistant can persist this via tool `model_routing_config`.

Runtime notes:

- Interactive CLI and single-message CLI both execute through the HostAgent worker path.
- Interactive CLI keeps a stable host episode per workspace until you reset the session with `/clear` or `/new`.
- HostAgent now routes through app-specialized workers (terminal, browser, editor, fallback), each with explicit tool-policy and capability contracts.
- HostAgent runs a proactive observe and replan loop after each step (`continue` / `replan` / `escalate`) instead of only executing a fixed static plan.
- HostAgent runtime perception is strict on all supported host OS targets. Empty runtime perception is rejected instead of silently degrading to an empty screen state.
- When native accessibility tree capture is unavailable, runtime context fallback still provides a widget-tree application context so cross-app routing remains deterministic.
- Before `mac_automation click_at`, the runtime requires a successful `perception_capture` preflight with `include_widget_tree=true` and `include_ocr=true`.
- `perception_capture` accepts optional OCR overrides under `ocr`: `language`, `psm`, `oem`, and `tessdata_dir`. These are per-call runtime arguments, not persistent config keys.

### `auth`

Authentication management for providers and services.

- `gloamy auth status`
- `gloamy auth use <provider> <profile>`
- `gloamy auth refresh [--provider <ID>]`
- `gloamy auth logout [--provider <ID>]`

**Commands:**

| Command | Purpose |
|---------|---------|
| `auth status` | Show authentication status for all providers, active profile, and token expiry |
| `auth use <provider> <profile>` | Set active authentication profile for a provider |
| `auth refresh` | Refresh authentication tokens (OpenAI Codex, Gemini OAuth) |
| `auth logout` | Remove authentication profile and clear stored tokens |

**Examples:**

```bash
gloamy auth status                    # View all auth status
gloamy auth use openai personal       # Switch to 'personal' profile
gloamy auth refresh --provider gemini # Refresh Gemini tokens
gloamy auth logout --provider openai  # Clear OpenAI auth
```

### `gateway` / `daemon`

- `gloamy gateway [--host <HOST>] [--port <PORT>]`
- `gloamy daemon [--host <HOST>] [--port <PORT>]`

Notes:

- `gateway` exposes webhook, API, and WebSocket endpoints for integrations and desktop clients.
- The legacy browser dashboard has been removed; `gateway` no longer serves a browser UI.

### `estop`

- `gloamy estop` (engage `kill-all`)
- `gloamy estop --level network-kill`
- `gloamy estop --level domain-block --domain "*.chase.com" [--domain "*.paypal.com"]`
- `gloamy estop --level tool-freeze --tool shell [--tool browser]`
- `gloamy estop status`
- `gloamy estop resume`
- `gloamy estop resume --network`
- `gloamy estop resume --domain "*.chase.com"`
- `gloamy estop resume --tool shell`
- `gloamy estop resume --otp <123456>`

Notes:

- `estop` commands require `[security.estop].enabled = true`.
- When `[security.estop].require_otp_to_resume = true`, `resume` requires OTP validation.
- OTP prompt appears automatically if `--otp` is omitted.

### `service`

- `gloamy service install`
- `gloamy service install --service-init <auto|systemd|openrc>`
- `gloamy service start`
- `gloamy service stop`
- `gloamy service restart`
- `gloamy service status`
- `gloamy service uninstall`

**Options:**

| Option | Purpose |
|--------|---------|
| `--service-init <type>` | Service initialization system: `auto` (default), `systemd`, or `openrc` |

**Examples:**

```bash
gloamy service --service-init systemd install   # Use systemd explicitly
gloamy service --service-init openrc install    # Use OpenRC for Alpine/embedded
```

### `cron`

- `gloamy cron list`
- `gloamy cron add <expr> [--tz <IANA_TZ>] <command>`
- `gloamy cron add-at <rfc3339_timestamp> <command>`
- `gloamy cron add-every <every_ms> <command>`
- `gloamy cron once <delay> <command>`
- `gloamy cron remove <id>`
- `gloamy cron pause <id>`
- `gloamy cron resume <id>`
- `gloamy cron update <id> [--expr <new_expr>] [--command <new_cmd>] [--tz <new_tz>]`

Notes:

- Mutating schedule/cron actions require `cron.enabled = true`.
- Shell command payloads for schedule creation (`create` / `add` / `once`) are validated by security command policy before job persistence.
- `cron update` modifies only specified fields; unspecified fields remain unchanged.

### `models`

- `gloamy models refresh`
- `gloamy models refresh --provider <ID>`
- `gloamy models refresh --all`
- `gloamy models refresh --force`
- `gloamy models set <model_id>`

`models refresh` currently supports live catalog refresh for provider IDs: `openrouter`, `openai`, `anthropic`, `groq`, `mistral`, `deepseek`, `xai`, `together-ai`, `gemini`, `ollama`, `llamacpp`, `sglang`, `vllm`, `astrai`, `venice`, `fireworks`, `cohere`, `moonshot`, `glm`, `zai`, `qwen`, and `nvidia`.

**Options:**

| Option | Purpose |
|--------|---------|
| `--all` | Refresh all providers that support live model discovery |
| `--force` | Force refresh bypassing cache |

`models set <model_id>` updates `default_model` in config and activates immediately.

### `doctor`

- `gloamy doctor`
- `gloamy doctor models [--provider <ID>] [--use-cache]`
- `gloamy doctor traces [--limit <N>] [--event <TYPE>] [--contains <TEXT>]`
- `gloamy doctor traces --id <TRACE_ID>`

`doctor traces` reads runtime tool/model diagnostics from `observability.runtime_trace_path`.

**Subcommands:**

| Command | Purpose |
|---------|---------|
| `doctor models` | Probe model catalogs across providers, verify connectivity |
| `doctor traces` | Query runtime trace events with filtering |

**Trace Filtering Options:**

| Option | Purpose |
|--------|---------|
| `--limit <N>` | Maximum events to return (default: 50) |
| `--event <TYPE>` | Filter by event type (`tool_call`, `tool_call_result`, `model_request`, etc.) |
| `--contains <TEXT>` | Filter events containing text |
| `--id <TRACE_ID>` | Retrieve specific trace by ID |

### `channel`

- `gloamy channel list`
- `gloamy channel start`
- `gloamy channel doctor`
- `gloamy channel bind-telegram <IDENTITY>`
- `gloamy channel add <type> <json>`
- `gloamy channel remove <name>`

Runtime in-chat commands (Telegram/Discord while channel server is running):

- `/models`
- `/models <provider>`
- `/model`
- `/model <model-id>`
- `/new`

Channel runtime also watches `config.toml` and hot-applies updates to:
- `default_provider`
- `default_model`
- `default_temperature`
- `api_key` / `api_url` (for the default provider)
- `reliability.*` provider retry settings

`add/remove` currently route you back to managed setup/manual config paths (not full declarative mutators yet).

### `integrations`

- `gloamy integrations info <name>`

### `skills`

- `gloamy skills list`
- `gloamy skills audit <source_or_name>`
- `gloamy skills install <source>`
- `gloamy skills remove <name>`

`<source>` accepts git remotes (`https://...`, `http://...`, `ssh://...`, and `git@host:owner/repo.git`) or a local filesystem path.

`skills install` always runs a built-in static security audit before the skill is accepted. The audit blocks:
- symlinks inside the skill package
- script-like files (`.sh`, `.bash`, `.zsh`, `.ps1`, `.bat`, `.cmd`)
- high-risk command snippets (for example pipe-to-shell payloads)
- markdown links that escape the enclosing skill package or shared `skills/` collection, point to remote markdown, or target script files

Use `skills audit` to manually validate a candidate skill directory (or an installed skill by name) before sharing it.

Skill manifests (`SKILL.toml`) support `prompts` and `[[tools]]`; both are injected into the agent system prompt at runtime, so the model can follow skill instructions without manually reading skill files.

At runtime, Gloamy merges skills from the configured workspace `skills/` directory and, when you launch it from a project/worktree that contains its own `skills/` directory, that project-local collection as well.

Gloamy also ships built-in `docx`, `xlsx`, and `pptx` document skills. They are materialized under the user's Gloamy config directory on first run, appear in `gloamy skills list`, and can be audited by name with `gloamy skills audit docx` (or `xlsx` / `pptx`) without a separate install step.

### `migrate`

- `gloamy migrate openclaw [--source <path>] [--dry-run]`

`migrate openclaw` imports memory from an OpenClaw workspace into the current Gloamy workspace.

- Default source path: `~/.openclaw/workspace`
- Imported sources: `memory/brain.db`, `MEMORY.md`, `memory/*.md`
- `--dry-run` previews candidate entries without writing data
- Re-runs skip unchanged entries and rename conflicting keys deterministically
- A backup of the target memory is created before writes

This command does not migrate arbitrary workspace files and does not convert OpenClaw config into `config.toml`.

### `config`

- `gloamy config schema`

`config schema` prints a JSON Schema (draft 2020-12) for the full `config.toml` contract to stdout.

### `completions`

- `gloamy completions bash`
- `gloamy completions fish`
- `gloamy completions zsh`
- `gloamy completions powershell`
- `gloamy completions elvish`

`completions` is stdout-only by design so scripts can be sourced directly without log/warning contamination.

### `hardware`

- `gloamy hardware discover`
- `gloamy hardware introspect <path>`
- `gloamy hardware info [--chip <chip_name>]`

### `peripheral`

- `gloamy peripheral list`
- `gloamy peripheral add <board> <path>`
- `gloamy peripheral flash [--port <serial_port>]`
- `gloamy peripheral setup-uno-q [--host <ip_or_host>]`
- `gloamy peripheral flash-nucleo`

## Validation Tip

To verify docs against your current binary quickly:

```bash
gloamy --help
gloamy <command> --help
```
