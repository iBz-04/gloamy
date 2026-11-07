# ADR 0001: Separate Rust MSRV from build toolchains

- **Status:** Accepted
- **Date:** 2026-11-03

## Context

The Rust package `rust-version` is a compatibility promise to downstream users. CI, release, and container builds also need a current, reproducible compiler for security fixes and current dependency resolution. Treating those as one version either raises the public MSRV unnecessarily or leaves project builds on an old compiler.

The project also contains a Node-based web client that should follow an explicit supported Node release.

## Decision

- Require Rust 1.96 for the application and host fuzz package, following the maintainer's explicit compatibility policy.
- Pin CI, release, and container builds to a current stable Rust patch independently of that MSRV.
- Change the MSRV only for an explicit maintainer policy decision or a documented dependency requirement, and validate the selected minimum.
- Pin the web runtime to an explicit Node 24 patch in both the local version file and its container image. Keep Node type definitions on the same major line.
- Pin container images by digest where the repository already uses digest pinning, and refresh the digest with the selected version.

## Consequences

- CI can use a current compiler without silently changing the package compatibility contract.
- Dependency upgrades must be checked against the declared MSRV before they are accepted.
- Rust and Node patch updates require coordinated changes to their CI, container, and developer-facing pins.

## Maintainer policy update

The supported host package requires Rust 1.96. CI and container builds remain pinned to Rust 1.99.1; the web client uses Node 24.22.2. Maintained host targets are Linux amd64, Linux arm64, and macOS arm64. Users compiling host packages with an older compiler must upgrade to Rust 1.96 or newer. Revert the manifest and policy changes together to roll back this requirement.

## Minimum-version validation

The minimum-compiler CI job checks all application features and targets on Rust 1.96.0. The Matrix SDK requirement starts at 0.16.1 to address the 0.16.0 compiler query-depth failure documented in [upstream issue 6254](https://github.com/matrix-org/matrix-rust-sdk/issues/6254).
