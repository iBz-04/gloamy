# Running Gloamy in Docker

This guide covers the container images in this repository and the Compose stack
that runs the agent daemon together with the web UI.

Last verified: **August 24, 2026**.

## What Ships Where

| File | Purpose |
|---|---|
| `Dockerfile` | The `gloamy` binary. Stages: `builder`, `runtime`, `dev`, `release` (default) |
| `docker-compose.yml` | Production-shaped stack: daemon + web UI |
| `web/Dockerfile` | Builds the Vue app, serves it from nginx with a same-origin API proxy |
| `web/nginx.conf.template` | nginx server block; `${GLOAMY_DAEMON_URL}` is substituted at container start |
| `web/nginx-proxy.conf` | Shared `proxy_*` settings for the daemon-bound locations |
| `dev/docker-compose.yml` | Sandboxed development stack (agent + Ubuntu sandbox) |
| `dev/docker-compose.ci.yml` | Local CI runner (`dev/ci.sh`) |

## Quick Start

Requires Docker Compose v2.24 or newer.

```bash
cp .env.example .env
```

Set at least one provider API key in `.env`, then:

```bash
docker compose up -d --build
```

The first build compiles the Rust binary with `lto = "fat"` and
`codegen-units = 1`, so expect a long initial build. Later builds reuse the
cargo cache mounts and are much faster.

Read the one-time pairing code the gateway prints on first start:

```bash
docker compose logs gloamy
```

Then open <http://127.0.0.1:8080> and enter that code. The UI exchanges it for a
bearer token over `POST /pair` and stores the token in the browser.

## Ports and Exposure

| Service | Container | Host | Notes |
|---|---|---|---|
| `web` | 80 | `127.0.0.1:8080` | Static assets + API proxy |
| `gloamy` | 42617 | `127.0.0.1:42617` | Gateway REST/SSE API |

Both host mappings are bound to loopback deliberately. Gloamy can execute shell
commands, so publishing either port on `0.0.0.0` exposes that capability to your
whole network. To reach it from elsewhere, put it behind a reverse proxy with
TLS and authentication, or use a tunnel (`[tunnel]` in config).

Inside the container the gateway binds `0.0.0.0` so the `web` container can
reach it across the Compose network. That trips the gateway's public-bind guard,
which is why the Compose file sets `GLOAMY_ALLOW_PUBLIC_BIND=true`. The guard is
about accidental internet exposure; the loopback-only `ports` mapping is what
actually contains it here.

## Configuration

Compose passes `.env` through to the daemon container, so every variable in
[.env.example](../.env.example) works, including provider-specific keys such as
`OPENROUTER_API_KEY`. The environment overrides that matter for containers:

| Variable | Effect |
|---|---|
| `GLOAMY_API_KEY` | Provider credential (generic fallback) |
| `GLOAMY_PROVIDER` / `GLOAMY_MODEL` | Default provider and model |
| `GLOAMY_GATEWAY_HOST` / `GLOAMY_GATEWAY_PORT` | Gateway bind address |
| `GLOAMY_ALLOW_PUBLIC_BIND` | Opt out of the non-loopback bind guard |
| `GLOAMY_WORKSPACE` | Workspace root (`/gloamy-data/workspace` in the image) |

Everything mutable lives under `$HOME`, which the image sets to `/gloamy-data`
and Compose backs with the `gloamy-data` named volume:

- `/gloamy-data/.gloamy/` — `config.toml`, memory, secret store
- `/gloamy-data/workspace/` — agent working files

To edit the config file directly:

```bash
docker compose exec gloamy sh -c 'cat /gloamy-data/.gloamy/config.toml'
```

For interactive onboarding inside the container instead of hand-editing:

```bash
docker compose run --rm gloamy onboard --interactive
```

## Common Operations

```bash
docker compose ps                      # service + health status
docker compose logs -f gloamy          # follow daemon logs
docker compose exec gloamy gloamy doctor   # diagnostics inside the container
docker compose restart gloamy          # restart after a config change
docker compose down                    # stop, keep the data volume
docker compose down -v                 # stop and delete all agent state
```

Any `gloamy` subcommand works as a one-shot, because `gloamy` is the image
entrypoint:

```bash
docker compose run --rm gloamy status
docker run --rm gloamy:local --version
```

## Image Targets

```bash
docker build --target release -t gloamy:local .   # production runtime (default)
docker build --target dev     -t gloamy:dev   .   # adds shell tooling
```

The `release` stage carries `ca-certificates`, `curl`, `git`, `jq`, and `tzdata`
so the shell tool's default command allowlist is usable out of the box. The
`dev` stage adds `python3`, `ripgrep`, editors, and process tools for when you
exec into a container.

Both stages run as uid 1000 and are non-root. `scripts/bootstrap.sh --docker`
runs the `release` target with `--user $(id -u):$(id -g)` and bind-mounts host
directories over `/gloamy-data/.gloamy` and `/gloamy-data/workspace`, which is
why those paths tolerate arbitrary uids.

## How the Web Proxy Works

The web app calls same-origin paths (`/api/...`, `/pair`, `/health`) so the
browser never deals with CORS or mixed content. nginx forwards exactly those
prefixes to the daemon, mirroring `DAEMON_PATHS` in `web/vite.config.ts` so dev
and Docker behave identically.

Two details are load-bearing:

- `proxy_buffering off` and `gzip off` — `GET /api/events` is a Server-Sent
  Events stream, and buffering it would withhold events until the connection
  closed.
- `proxy_read_timeout 360s` on `/api/` — `POST /api/chat` runs the full agent
  loop and is capped at 300s upstream, so nginx must not cut it first.

Point the UI at a daemon somewhere else by overriding `GLOAMY_DAEMON_URL` on the
`web` service. If the daemon lives on a different origin entirely, rebuild the
web image with `--build-arg VITE_GLOAMY_API_BASE=https://daemon.example.com`;
note that this reintroduces CORS.

## Development Stack

`dev/docker-compose.yml` is a separate stack that runs the agent alongside an
Ubuntu sandbox container for testing shell commands safely:

```bash
./dev/cli.sh up       # start agent + sandbox
./dev/cli.sh agent    # shell into the agent container
./dev/cli.sh shell    # shell into the Ubuntu sandbox
./dev/cli.sh down
```

`./dev/cli.sh up` creates `target/.gloamy/config.toml` from
`dev/config.template.toml`, a `.env` stub, and `playground/` if any are missing.
Local CI runs in containers too — see `./dev/ci.sh` and
[ci-map.md](ci-map.md).

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `web` never starts | It waits for the daemon healthcheck | `docker compose logs gloamy` |
| UI loads, API calls 502 | Daemon unhealthy or still initializing | `docker compose ps`, check daemon logs |
| UI shows no live events | An extra proxy in front is buffering SSE | Disable buffering for `/api/events` there too |
| `401` on every API call | Not paired, or the token was cleared | Re-pair with the code from the daemon logs |
| No pairing code in logs | A token already exists in the volume | Use the existing token, or `docker compose down -v` to reset |
| `Refusing to bind to 0.0.0.0` | `GLOAMY_ALLOW_PUBLIC_BIND` not set | Keep the Compose default, or bind loopback |
| Provider auth errors | `.env` missing or not picked up | Confirm the key, then `docker compose up -d` to recreate |

## Related Docs

- [one-click-bootstrap.md](one-click-bootstrap.md) — `bootstrap.sh --docker`
- [operations-runbook.md](operations-runbook.md) — day-2 operations
- [network-deployment.md](network-deployment.md) — safe gateway exposure
- [config-reference.md](config-reference.md) — full config surface
- [gateway-api-reference.md](gateway-api-reference.md) — endpoints the UI calls
- [troubleshooting.md](troubleshooting.md) — general troubleshooting matrix
