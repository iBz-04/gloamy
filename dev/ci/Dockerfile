# syntax=docker/dockerfile:1.7

FROM rust:1.99.1-slim-trixie@sha256:ea0999ff67355d958b8799b6751c879a6d770076d26047dca66f6e51e77930d3

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    git \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN rustup toolchain install 1.99.1 --profile minimal --component rustfmt --component clippy

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo install --locked cargo-audit --version 0.22.1 && \
    cargo install --locked cargo-deny --version 0.18.5

WORKDIR /workspace

CMD ["bash"]
