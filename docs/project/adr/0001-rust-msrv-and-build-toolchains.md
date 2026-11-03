# ADR 0001: Separate Rust MSRV from build toolchains

- **Status:** Accepted
- **Date:** 2026-11-03

## Context

The Rust package `rust-version` is a compatibility promise to downstream users. CI, release, and container builds also need a current, reproducible compiler for security fixes and current dependency resolution. Treating those as one version either raises the public MSRV unnecessarily or leaves project builds on an old compiler.

The project also contains firmware crates with their own target-specific toolchains, and a Node-based web client that should follow an explicit supported Node release.

## Decision

- Keep each Rust package's `rust-version` at the lowest version the checked-in dependency set supports.
- Pin CI, release, and container builds to a current stable Rust patch independently of that MSRV.
- Raise the MSRV only when an updated dependency requires it; document the minimum required version and validate it before changing the public promise.
- Pin the web runtime to an explicit Node 24 patch in both the local version file and its container image. Keep Node type definitions on the same major line.
- Preserve firmware-specific toolchain pins unless firmware compatibility is explicitly part of a dependency update.
- Pin container images by digest where the repository already uses digest pinning, and refresh the digest with the selected version.

## Consequences

- CI can use a current compiler without silently changing the package compatibility contract.
- Dependency upgrades must be checked against the declared MSRV before they are accepted.
- Rust and Node patch updates require coordinated changes to their CI, container, and developer-facing pins.
- Firmware toolchains remain independently managed and must be checked when shared dependencies change.
