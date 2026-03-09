# One-Click Bootstrap

This page defines the fastest supported path to install and initialize Gloamy.

Last verified: **February 20, 2026**.

## Option 0: Homebrew (macOS/Linuxbrew)

```bash
brew install gloamy
```

## Option A (Recommended): Clone + local script

```bash
git clone https://github.com/iBz-04/gloamy.git
cd gloamy
./bootstrap.sh
```

What it does by default:

1. `cargo build --release --locked`
2. `cargo install --path . --force --locked`

### Resource preflight and pre-built flow

Source builds typically require at least:

- **2 GB RAM + swap**
- **6 GB free disk**

When resources are constrained, bootstrap now attempts a pre-built binary first.

```bash
./bootstrap.sh --prefer-prebuilt
```

To require binary-only installation and fail if no compatible release asset exists:

```bash
./bootstrap.sh --prebuilt-only
```

To bypass pre-built flow and force source compilation:

```bash
./bootstrap.sh --force-source-build
```

## Dual-mode bootstrap

Default behavior is **app-only** (build/install Gloamy) and expects existing Rust toolchain.

For fresh machines, enable environment bootstrap explicitly:

```bash
./bootstrap.sh --install-system-deps --install-rust
```

Notes:

- `--install-system-deps` installs compiler/build prerequisites (may require `sudo`).
- `--install-rust` installs Rust via `rustup` when missing.
- `--prefer-prebuilt` tries release binary download first, then falls back to source build.
- `--prebuilt-only` disables source fallback.
- `--force-source-build` disables pre-built flow entirely.

## Option B: Remote one-liner

```bash
curl -fsSL https://raw.githubusercontent.com/iBz-04/gloamy/main/scripts/bootstrap.sh | bash
```

For high-security environments, prefer Option A so you can review the script before execution.

Legacy compatibility:

```bash
curl -fsSL https://raw.githubusercontent.com/iBz-04/gloamy/main/scripts/install.sh | bash
```

This legacy endpoint prefers forwarding to `scripts/bootstrap.sh` and falls back to legacy source install if unavailable in that revision.

If you run Option B outside a repository checkout, the bootstrap script automatically clones a temporary workspace, builds, installs, and then cleans it up.

## Optional onboarding modes

### Containerized onboarding (Docker)

```bash
./bootstrap.sh --docker
```

This builds a local Gloamy image and launches onboarding inside a container while
persisting config/workspace to `./.gloamy-docker`.

Container CLI defaults to `docker`. If Docker CLI is unavailable and `podman` exists,
bootstrap auto-falls back to `podman`. You can also set `GLOAMY_CONTAINER_CLI`
explicitly (for example: `GLOAMY_CONTAINER_CLI=podman ./bootstrap.sh --docker`).

For Podman, bootstrap runs with `--userns keep-id` and `:Z` volume labels so
workspace/config mounts remain writable inside the container.

If you add `--skip-build`, bootstrap skips local image build. It first tries the local
Docker tag (`GLOAMY_DOCKER_IMAGE`, default: `gloamy-bootstrap:local`); if missing,
it pulls `ghcr.io/iBz-04/gloamy:latest` and tags it locally before running.

### Quick onboarding (non-interactive)

```bash
./bootstrap.sh --onboard --api-key "sk-..." --provider openrouter
```

Or with environment variables:

```bash
GLOAMY_API_KEY="sk-..." GLOAMY_PROVIDER="openrouter" ./bootstrap.sh --onboard
```

### Interactive onboarding

```bash
./bootstrap.sh --interactive-onboard
```

## Useful flags

- `--install-system-deps`
- `--install-rust`
- `--skip-build` (in `--docker` mode: use local image if present, otherwise pull `ghcr.io/iBz-04/gloamy:latest`)
- `--skip-install`
- `--provider <id>`

See all options:

```bash
./bootstrap.sh --help
```

## Related docs

- [README.md](../README.md)
- [commands-reference.md](commands-reference.md)
- [providers-reference.md](providers-reference.md)
- [channels-reference.md](channels-reference.md)
