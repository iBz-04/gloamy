# Research: Supported Host Architecture Prune

## Decision 1: Canonical host target mapping

- **Decision**: Map `linux-amd64` to `x86_64-unknown-linux-gnu`, `linux-arm64` to `aarch64-unknown-linux-gnu`, and `aarch64-darwin` to `aarch64-apple-darwin`.
- **Evidence**: Those Rust triples are already used by `.github/workflows/ci.yml`, `release.yml`, `promote-release.yml`, and the current installer target detector.
- **Rationale**: Reuse the repository's existing GNU Linux and arm64 Darwin release artifact conventions and make the user labels explicit in docs.
- **Alternatives considered**: Retain Windows, Intel Darwin, Android, ARMv7, or musl variants. Rejected because the requested support list is exactly three user-visible targets.

## Decision 2: Linux arm64 remains generic host support

- **Decision**: Keep generic Linux arm64 builds and remove Raspberry Pi GPIO, robot, serial-board, MCU, and firmware integrations.
- **Evidence**: Raspberry Pi and other boards are represented by `src/peripherals/rpi.rs`, `crates/robot`, firmware, and board-specific docs, while ordinary Linux arm64 is already built by release automation.
- **Rationale**: A generic Linux arm64 artifact is in scope; board-specific peripherals are a different product surface.
- **Alternative considered**: Keep the peripheral and robot subsystems because some run on Linux arm64. Rejected because the request narrows the product to generic PC-type systems.

## Decision 3: Preserve lockfiles and unrelated tooling

- **Decision**: Preserve `flake.lock` and `fuzz/Cargo.lock`; update `Cargo.lock` for the final Cargo workspace.
- **Rationale**: Lockfiles pin dependencies and do not, on their own, advertise or add host support. Deleting them would reduce reproducibility without reducing the supported architecture set.
- **Alternative considered**: Delete all lockfiles alongside firmware. Rejected as unrelated and harmful to deterministic dependency resolution.

## Decision 4: Keep generic Linux deployment guidance

- **Decision**: Rewrite `docs/network-deployment.md` for generic Linux hosts rather than deleting it.
- **Rationale**: Its gateway, tunnel, and channel guidance remains relevant to Linux amd64/arm64 after Raspberry Pi and peripheral instructions are removed.

## No unresolved research items

Target triples, relevant product surfaces, and repository quality gates are evidenced by the current checkout. No external system or service integration is introduced.
