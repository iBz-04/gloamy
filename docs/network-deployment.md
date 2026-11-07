# Network Deployment on Supported Linux Hosts

This guide covers operating Gloamy on generic Linux amd64 or Linux arm64 hosts. The maintained host targets are listed in [README.md](../README.md#supported-hosts). Board, firmware, and embedded-device setup is outside this guide.

## 1. Network Modes

| Mode | Inbound port needed? | Use case |
|---|---|---|
| Telegram polling | No | Gloamy polls the Telegram API from behind NAT. |
| Matrix sync, including E2EE | No | Gloamy syncs through the Matrix client API. |
| Discord and Slack | No | Outbound channel connections. |
| Nostr | No | Outbound WebSocket connections to relays. |
| Gateway webhooks | Yes | Webhook providers and external clients need a reachable URL. |
| Gateway pairing | Yes | Clients connect through the gateway. |

Telegram, Matrix, Discord, Slack, and Nostr use outbound connections by default. They do not require port forwarding or a public IP.

## 2. Install and Configure a Linux Host

Use the release artifact for the host target, or build from source with the matching Rust target. For a source build:

```bash
# Linux amd64
cargo build --release --locked --target x86_64-unknown-linux-gnu

# Linux arm64
cargo build --release --locked --target aarch64-unknown-linux-gnu
```

See [one-click-bootstrap.md](one-click-bootstrap.md) for installer options. Cross-compiling Linux arm64 on an amd64 host also requires an AArch64 GNU linker.

Configure only the channels and gateway features you intend to run in `~/.gloamy/config.toml`. For example:

```toml
[channels_config.telegram]
bot_token = "YOUR_BOT_TOKEN"
allowed_users = []

[gateway]
host = "127.0.0.1"
port = 42617
allow_public_bind = false
```

Run the daemon locally:

```bash
gloamy daemon --host 127.0.0.1 --port 42617
```

The gateway remains reachable only from the host. Telegram polling can still connect outbound.

## 3. Binding to a Local Network

To let other devices on a trusted LAN reach the gateway, explicitly opt in:

```toml
[gateway]
host = "0.0.0.0"
port = 42617
allow_public_bind = true
```

```bash
gloamy daemon --host 0.0.0.0 --port 42617
```

Binding to `0.0.0.0` exposes the gateway to the network. Use pairing and firewall rules, and expose it only on a network you trust.

### Public webhook URLs

For webhooks or clients outside the LAN, keep the gateway on localhost and use a tunnel:

```toml
[tunnel]
provider = "tailscale" # or "ngrok" or "cloudflare"
```

See the tunnel configuration and [operations runbook](operations-runbook.md) for access controls and lifecycle management. Do not bind a public interface solely to make a webhook work.

## 4. Telegram Polling

Telegram uses long polling, so no inbound port is needed:

```toml
[channels_config.telegram]
bot_token = "YOUR_BOT_TOKEN"
allowed_users = [] # deny by default; bind identities explicitly
```

Run `gloamy daemon` to start the configured channel. Keep one runtime instance per bot token; the Telegram Bot API supports only one active `getUpdates` poller. If you see `Conflict: terminated by other getUpdates request`, stop the duplicate process and restart the intended daemon.

Approve an account with:

```bash
gloamy channel bind-telegram <IDENTITY>
```

`<IDENTITY>` can be a numeric user ID or a username without `@`.

## 5. Webhook Channels

Webhook channels need a public URL so the provider can send events. Tailscale Funnel, ngrok, and Cloudflare Tunnel can forward to `127.0.0.1:42617`. Keep gateway pairing enabled and configure each provider's webhook secret where available.

## 6. Deployment Checklist

- [ ] Choose the supported Linux amd64 or Linux arm64 release artifact.
- [ ] Configure only the required channels and gateway routes.
- [ ] Keep the gateway bound to localhost unless LAN access is required.
- [ ] Use pairing, firewall rules, and a tunnel for remote webhook access.
- [ ] Run one daemon per Telegram bot token.
- [ ] Install and start the user service if Gloamy should run after login or reboot.
- [ ] Confirm logs and health through the [operations runbook](operations-runbook.md).

## References

- [Commands reference](commands-reference.md)
- [Channels reference](channels-reference.md)
- [Configuration reference](config-reference.md)
- [Docker deployment](docker-deployment.md)
- [Operations runbook](operations-runbook.md)
- [Troubleshooting](troubleshooting.md)
