# syntax=docker/dockerfile:1.7

# Gloamy container image.
#
# Stages:
#   builder  — compiles the `gloamy` binary
#   runtime  — shared production runtime layer (binary + data dir + non-root user)
#   dev      — runtime plus an interactive shell and the CLI tools dev workflows expect
#   release  — final/default stage; what gets published to GHCR
#
# Stage names are a contract. These consumers build specific targets:
#   scripts/bootstrap.sh --docker  → --target release, passes `onboard ...` as args
#   dev/docker-compose.yml         → --target dev
#   dev/ci.sh docker-smoke         → --target dev, runs `--version`
#   .github/workflows/release.yml  → default (last) stage, linux/amd64 + linux/arm64

# ══════════════════════════════════════════════════════════════════════════════
# builder
# ══════════════════════════════════════════════════════════════════════════════
# Pinned to bookworm so the runtime stage below can share the same glibc.
FROM rust:1.92-slim-bookworm AS builder

# build-essential + pkg-config: C toolchain for the bundled SQLite and `ring`.
# git: crates resolved from git and build scripts that read repository metadata.
# TLS is rustls throughout, so no OpenSSL development headers are needed.
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        git \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .

# Cache mount ids include the target arch so parallel multi-arch builds do not
# contend over one cargo directory. The binary is copied out inside the same RUN
# because cache mounts are not persisted into the resulting layer.
ARG TARGETARCH
RUN --mount=type=cache,id=gloamy-cargo-registry-${TARGETARCH},target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=gloamy-cargo-git-${TARGETARCH},target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=gloamy-target-${TARGETARCH},target=/build/target,sharing=locked \
    cargo build --release --locked --bin gloamy \
    && cp target/release/gloamy /usr/local/bin/gloamy

# ══════════════════════════════════════════════════════════════════════════════
# runtime
# ══════════════════════════════════════════════════════════════════════════════
FROM debian:bookworm-slim AS runtime

# ca-certificates: outbound TLS to model providers.
# curl, git, jq: present in the default `autonomy.allowed_commands` allowlist, so
#   the shell tool is usable out of the box instead of failing on a missing binary.
# tzdata: cron schedules resolve local timezones.
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        git \
        jq \
        tzdata \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/gloamy /usr/local/bin/gloamy

# Config resolution is `$HOME/.gloamy/config.toml`, so pointing HOME at
# /gloamy-data keeps every mutable path under a single mount point.
ENV HOME=/gloamy-data \
    GLOAMY_WORKSPACE=/gloamy-data/workspace

# The image defaults to uid 1000, but scripts/bootstrap.sh runs it with
# `--user $(id -u):$(id -g)`. HOME and the workspace are world-writable so those
# arbitrary uids can still work; .gloamy is not, because it holds the encrypted
# secret store. Callers running as another uid bind-mount their own .gloamy over
# it, which is exactly what bootstrap does.
RUN useradd --uid 1000 --home-dir /gloamy-data --shell /bin/bash gloamy \
    && mkdir -p /gloamy-data/.gloamy /gloamy-data/workspace \
    && chown -R 1000:1000 /gloamy-data \
    && chmod 0777 /gloamy-data /gloamy-data/workspace \
    && chmod 0700 /gloamy-data/.gloamy

USER 1000
WORKDIR /gloamy-data/workspace

# Gateway default. Publishing this port is left to the caller; docker-compose.yml
# binds it to host loopback only.
EXPOSE 42617

# `gloamy` as entrypoint means `docker run <image> --version` and
# `docker run <image> onboard --api-key ...` both work, which bootstrap and the
# docker-smoke check rely on.
ENTRYPOINT ["gloamy"]
CMD ["--help"]

# ══════════════════════════════════════════════════════════════════════════════
# dev
# ══════════════════════════════════════════════════════════════════════════════
FROM runtime AS dev

# Tooling for `dev/cli.sh agent` (interactive exec) and for agent shell commands
# during development. Deliberately absent from the release stage to keep the
# published image small.
USER root
RUN apt-get update && apt-get install -y --no-install-recommends \
        iputils-ping \
        less \
        nano \
        procps \
        python3 \
        ripgrep \
        unzip \
        vim-tiny \
        zip \
    && rm -rf /var/lib/apt/lists/*
USER 1000

# ══════════════════════════════════════════════════════════════════════════════
# release  (default target)
# ══════════════════════════════════════════════════════════════════════════════
FROM runtime AS release

ARG VERSION=dev
LABEL org.opencontainers.image.title="gloamy" \
      org.opencontainers.image.description="Rust-first autonomous agent runtime for CLI, channels, gateway, and hardware workflows." \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.source="https://github.com/iBz-04/gloamy" \
      org.opencontainers.image.licenses="MIT OR Apache-2.0"
