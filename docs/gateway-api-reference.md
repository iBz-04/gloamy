# Gateway API Reference

HTTP API endpoints exposed by the Gloamy gateway server.

Last verified: **April 8, 2026**.

## Base URL

Default: `http://127.0.0.1:42617`

Configurable via `[gateway]` config section or `--host`/`--port` CLI flags.

## Authentication

All endpoints require authentication via one of:

1. **Pairing token** (when `gateway.require_pairing = true`):
   - Header: `Authorization: Bearer <pairing_token>`
   - Obtained via initial pairing flow

2. **API key** (for trusted clients):
   - Header: `X-API-Key: <api_key>`

## API Endpoints

### Status & Health

#### `GET /api/status`

Runtime status summary including provider, model, version info.

**Response:**

```json
{
  "version": "0.1.0",
  "provider": "openai",
  "model": "gpt-5-mini",
  "uptime_secs": 3600
}
```

---

#### `GET /api/health`

Component health snapshot for monitoring.

**Response:**

```json
{
  "health": {
    "gateway": "healthy",
    "memory": "healthy",
    "provider": "healthy",
    "channels": "healthy"
  }
}
```

---

### Configuration

#### `GET /api/config`

Retrieve current configuration (secrets masked).

**Response:** `config.toml` as JSON with sensitive values replaced by `***`.

---

#### `POST /api/config`

Update configuration fields.

**Request:** Partial config object with changes.

**Notes:**
- Only specified fields are modified
- Secrets are preserved when masked values submitted
- Runtime validates changes before applying

---

### Tools

#### `GET /api/tools`

List all registered tools with their schemas.

**Response:**

```json
{
  "tools": [
    {
      "name": "shell",
      "description": "Execute shell commands",
      "parameters": { ... }
    }
  ]
}
```

---

#### `GET /api/cli-tools`

Discover CLI tools available on the system.

**Response:**

```json
{
  "cli_tools": [
    {
      "name": "git",
      "path": "/usr/bin/git",
      "version": "2.42.0",
      "category": "version_control"
    },
    {
      "name": "docker",
      "path": "/usr/local/bin/docker",
      "version": "24.0.7",
      "category": "container"
    }
  ]
}
```

**Discovered tools include:** `git`, `python`, `python3`, `node`, `npm`, `pip`, `pip3`, `docker`, `cargo`, `make`, `kubectl`, `rustc`

---

### Integrations

#### `GET /api/integrations`

List configured third-party integrations and their status.

**Response:**

```json
{
  "integrations": [
    {
      "name": "composio",
      "enabled": true,
      "connected": true
    },
    {
      "name": "one",
      "enabled": false
    }
  ]
}
```

---

### Skills

#### `GET /api/skills`

List installed skills and their metadata.

**Response:**

```json
{
  "skills": [
    {
      "name": "docx",
      "version": "1.0.0",
      "tools": ["docx_create", "docx_edit"]
    }
  ]
}
```

---

### Cron / Scheduling

#### `GET /api/cron`

List scheduled tasks.

**Response:** Array of scheduled jobs with expression, command, and status.

---

### Memory

#### `GET /api/memory`

Query memory store.

**Query Parameters:**
- `q`: Search query
- `category`: Filter by category (`core`, `daily`, `episode`, etc.)
- `limit`: Maximum results (default: 10)

---

### Cost Tracking

#### `GET /api/cost`

Current cost tracking summary.

**Response:**

```json
{
  "enabled": true,
  "daily_cost_usd": 2.45,
  "monthly_cost_usd": 42.10,
  "total_tokens": 1500000,
  "request_count": 250,
  "by_model": {
    "gpt-5-mini": { "requests": 200, "cost_usd": 1.50 },
    "claude-sonnet-4": { "requests": 50, "cost_usd": 0.95 }
  }
}
```

---

#### `GET /api/cost/timeline`

Daily token usage trend (up to 3660 days).

**Response:**

```json
{
  "timeline": [
    { "date": "2026-04-01", "tokens": 50000, "cost_usd": 1.20 },
    { "date": "2026-04-02", "tokens": 45000, "cost_usd": 1.08 }
  ]
}
```

---

### Logs

#### `GET /api/logs`

Retrieve daemon logs (requires pairing token).

**Query Parameters:**
- `tail`: Number of lines (default: 100, max: 1000)
- `level`: Filter by level (`error`, `warn`, `info`, `debug`)

**Response:**

```json
{
  "logs": [
    { "timestamp": "2026-04-08T10:30:00Z", "level": "INFO", "message": "Gateway started" },
    { "timestamp": "2026-04-08T10:30:05Z", "level": "WARN", "message": "High memory usage" }
  ]
}
```

**Log sources:** `daemon.stdout.log`, `daemon.stderr.log` from config directory.

---

### Diagnostics

#### `GET /api/doctor`

Run diagnostics and return health report.

**Response:**

```json
{
  "status": "healthy",
  "checks": [
    { "name": "config", "status": "pass" },
    { "name": "provider", "status": "pass" },
    { "name": "memory", "status": "pass" }
  ]
}
```

---

## WebSocket Endpoints

Real-time event streaming is available via WebSocket at:

```
ws://127.0.0.1:42617/ws
```

**Authentication:** Same as HTTP (token in `Authorization` header during handshake).

**Events:**
- `message` - Incoming channel messages
- `tool_call` - Tool execution events
- `cost_update` - Real-time cost tracking updates

---

## Webhook Endpoints

Channel webhooks receive events at:

| Channel | Endpoint |
|---------|----------|
| Telegram | `POST /telegram` |
| Discord | `POST /discord` |
| Slack | `POST /slack` |
| WhatsApp | `POST /whatsapp` |
| Linq | `POST /linq` |
| Nextcloud Talk | `POST /nextcloud-talk` |
| WATI | `POST /wati` |

---

## Error Responses

All endpoints return consistent error format:

```json
{
  "error": "Description of what went wrong",
  "code": "ERROR_CODE"
}
```

**HTTP Status Codes:**

| Code | Meaning |
|------|---------|
| `200` | Success |
| `400` | Bad request (invalid parameters) |
| `401` | Unauthorized (missing/invalid auth) |
| `403` | Forbidden (insufficient permissions) |
| `404` | Endpoint not found |
| `500` | Internal server error |

---

## CORS Support

The gateway supports CORS for browser clients:

- `Access-Control-Allow-Origin: *` (configurable)
- Preflight (`OPTIONS`) requests handled automatically
- Credentials supported when origin explicitly configured

---

## Rate Limiting

Default rate limits per IP:

- API endpoints: 100 requests/minute
- Webhook endpoints: 60 requests/minute per channel

Configurable via `[gateway.rate_limiting]` config section.

---

## Related Documentation

- [commands-reference.md](commands-reference.md) - CLI commands
- [config-reference.md](config-reference.md) - Gateway configuration
- [channels-reference.md](channels-reference.md) - Channel webhook setup
- [network-deployment.md](network-deployment.md) - Production deployment
