# Tools Reference

Agent-callable tools and their configuration.

Last verified: **April 8, 2026**.

## Tool Categories

| Category | Tools |
|----------|-------|
| **Core** | `shell`, `file_read`, `file_write`, `file_edit`, `glob_search` |
| **Memory** | `memory_store`, `memory_recall`, `memory_forget` |
| **Browser** | `browser_open`, `browser`, `perception_capture`, `screenshot`, `web_fetch` |
| **Integration** | `composio`, `one`, `web_search_tool`, `pushover`, `http_request` |
| **SOP** | `sop_list`, `sop_execute`, `sop_status`, `sop_advance`, `sop_approve` |
| **System** | `cron_add`, `cron_list`, `cron_remove`, `schedule`, `delegate` |

---

## Web Search Tool

Tool name: `web_search_tool`

Search the internet using DuckDuckGo (free) or Brave (requires API key).

### Configuration

```toml
[web_search]
provider = "duckduckgo"  # or "brave"
brave_api_key = "your-brave-api-key"  # Required for Brave provider
max_results = 5            # Results per search (1-10)
timeout_secs = 30          # Request timeout
```

### Environment Variables

| Variable | Purpose |
|----------|---------|
| `BRAVE_API_KEY` | Brave Search API key |

### Usage

The agent calls `web_search_tool` with:

```json
{
  "query": "latest rust programming features"
}
```

**Providers:**

- **DuckDuckGo** (default): Free, no API key required. May be rate-limited.
- **Brave**: Requires API key. Higher rate limits, better result quality.

---

## Pushover Notifications

Tool name: `pushover`

Send push notifications to mobile devices via Pushover service.

### Setup

Create a `.env` file in your workspace:

```bash
PUSHOVER_TOKEN=your-app-token
PUSHOVER_USER_KEY=your-user-key
```

### Usage

```json
{
  "message": "Build completed successfully",
  "title": "CI Notification",
  "priority": 0
}
```

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `message` | string | Yes | Notification message body |
| `title` | string | No | Notification title |
| `priority` | integer | No | `-2` to `2` (low to emergency) |
| `url` | string | No | URL to open when notification tapped |
| `url_title` | string | No | Title for the URL |
| `sound` | string | No | Notification sound name |

**Priority Levels:**

| Value | Level | Behavior |
|-------|-------|----------|
| `-2` | Lowest | No notification, only in history |
| `-1` | Low | No sound/vibration |
| `0` | Normal | Standard notification |
| `1` | High | Bypass quiet hours |
| `2` | Emergency | Require acknowledgment |

---

## PDF Read Tool

Tool name: `pdf_read`

Extract text content from PDF files.

### Requirements

Requires `rag-pdf` feature at compile time:

```bash
cargo build --features rag-pdf
```

### Usage

```json
{
  "path": "/path/to/document.pdf",
  "page_range": "1-10",
  "extract_text": true
}
```

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `path` | string | Yes | Path to PDF file |
| `page_range` | string | No | Pages to extract (e.g., `1-10`, `5`) |
| `extract_text` | boolean | No | Extract text content (default: true) |
| `max_chars` | integer | No | Maximum characters to extract |

---

## Browser Tool

Tool name: `browser`

Automate browser actions with GUI verification support.

### GUI Verification

The browser tool supports expectation-based verification after actions:

```json
{
  "action": "navigate",
  "url": "https://example.com/login",
  "expect": [
    {
      "template": "url_host_is",
      "params": {"host": "example.com"}
    },
    {
      "template": "field_value_equals",
      "params": {"selector": "#username", "value": ""},
      "required": false
    }
  ]
}
```

**Expectation Templates:**

| Template | Purpose | Required Params |
|----------|---------|-----------------|
| `field_value_equals` | Input field has expected value | `selector`, `value` |
| `focused_element_is` | Specific element has focus | `selector` |
| `checkbox_checked` | Checkbox is checked | `selector` |
| `window_title_contains` | Window title contains text | `substring` |
| `dialog_present` | Alert/confirm dialog visible | - |
| `url_is` | Exact URL match | `url` |
| `url_host_is` | Hostname match | `host` |
| `file_exists` | File exists on disk | `path` |
| `download_completed` | File download finished | `filename` |

**Expectation Result:**

- `required: true` (default): Failed expectation fails the tool call
- `required: false`: Failed expectation marks result as `ambiguous`, not failed

---

## macOS Automation Tool

Tool name: `mac_automation` (macOS only)

Native macOS GUI automation with verification.

### Requirements

- macOS platform
- Accessibility permissions enabled
- Successful `perception_capture` preflight before `click_at`

### GUI Verification

Same expectation templates as browser tool, plus macOS-specific:

| Template | Purpose | Required Params |
|----------|---------|-----------------|
| `element_exists` | UI element exists | `identifier` or `label` |
| `element_enabled` | UI element is enabled | `identifier` |

---

## CLI Discovery

Internal module (not exposed as tool): `cli_discovery`

Scans PATH for known CLI tools. Used internally and exposed via `/api/cli-tools` endpoint.

**Discovered Tools:**

| Tool | Category |
|------|----------|
| `git` | version_control |
| `python`, `python3` | interpreter |
| `node`, `npm` | runtime |
| `pip`, `pip3` | package_manager |
| `docker` | container |
| `cargo` | build |
| `make` | build |
| `kubectl` | orchestration |
| `rustc` | compiler |

---

## Tool Security Policy

All tools respect `[autonomy]` configuration:

| Setting | Affected Tools |
|---------|----------------|
| `workspace_only` | `file_read`, `file_write`, `shell` |
| `forbidden_paths` | `file_read`, `file_write`, `shell` |
| `allowed_commands` | `shell` |
| `allowed_domains` | `browser_open`, `http_request` |

---

## Related Documentation

- [sop-tools-reference.md](sop-tools-reference.md) - SOP-specific tools
- [config-reference.md](config-reference.md) - Tool configuration
- [commands-reference.md](commands-reference.md) - CLI commands
- [gateway-api-reference.md](gateway-api-reference.md) - API endpoints
