# Implementation Plan: Supported Host Architecture Prune

**Branch**: `feat/app-server-oauth-models-v2` | **Date**: 2026-11-06 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/001-supported-host-prune/spec.md`

## Summary

Reduce published and documented host support to Linux amd64, Linux arm64, and arm64 Darwin. Remove the embedded-board and robotics subsystem end to end, including firmware, configuration, CLI commands, onboarding, tool registration, board-trigger surfaces, and dedicated setup documentation. Keep the Linux container targets for amd64/arm64 and preserve unrelated lockfiles and runtime capabilities.

## Brownfield Findings

- **Current architecture**: Rust CLI/runtime in `src/`, an optional Vue/TypeScript web client in `web/`, and a workspace member `crates/robot`.
- **Current host targets**: Release workflows already build GNU Linux amd64/arm64 and arm64 Darwin, with Windows x86_64; the full CI matrix includes Intel Darwin and Windows. `scripts/bootstrap.sh` also maps ARMv7 and Intel Darwin.
- **Current device scope**: `src/peripherals/`, `src/hardware/`, three `src/tools/hardware_*` modules, `[hardware]`/`[peripherals]` config, `gloamy hardware`/`gloamy peripheral`, onboarding steps, SOP peripheral triggers, `firmware/`, and `crates/robot` expose non-generic hardware features.
- **Repository governance**: `AGENTS.md` requires narrow changes, secure defaults, risk-based validation, and documentation/index parity across the supported locales.
- **Existing worktree changes**: preserve existing edits and deletions. Review them with the final diff; do not reset the working tree.

### Initial worktree baseline

- Worktree: `/home/avery/build/other-peoples-software/wt/app-server-oauth-v2`, branch `feat/app-server-oauth-models-v2`, initially two commits behind local `main`.
- Main worktree: `/home/avery/build/other-peoples-software/wt/main-amalgamation`, initially clean. The fork remote was present; no push was requested.
- Pre-existing edits retained: dependency/toolchain changes in Cargo files, Rust source updates, web package/configuration updates, memory/migration changes, docs/workflow updates including the main-branch flow rename, and firmware deletions.
- `flake.lock` and `fuzz/Cargo.lock` had been deleted in the initial dirty state; they were restored from the feature branch because the user request did not call for removing reproducibility metadata.

### File-by-file change inventory

| Area | Action | Evidence / reason |
| --- | --- | --- |
| `firmware/**` | Delete (firmware deletions already present) | MCU/Arduino/ESP32/Nucleo/Uno Q firmware is outside generic host scope. |
| `crates/robot/**` and workspace membership | Delete/remove | The crate is a Raspberry Pi/robot hardware product surface, not a generic host runtime. |
| `src/peripherals/**`, `src/hardware/**`, `src/tools/hardware_board_info.rs`, `src/tools/hardware_memory_map.rs`, `src/tools/hardware_memory_read.rs` | Delete | Board enumeration, GPIO, serial, probes, flashing, and memory-map tools only support device integration. |
| `src/lib.rs`, `src/main.rs`, `src/agent/loop_.rs` | Modify | Remove hardware/peripheral modules, CLI subcommands, flags, registry injection, and call parameters. |
| `src/config/mod.rs`, `src/config/schema.rs`, `src/onboard/wizard.rs` | Modify | Remove hardware/peripheral configuration types and onboarding questions; keep unrelated config and setup. |
| `src/service/mod.rs`, `src/tools/browser_open.rs`, `src/tools/browser_services.rs`, `src/tools/cli_discovery.rs`, `src/grounding/experts.rs`, `src/security/secrets.rs`, `src/integrations/registry.rs`, and Windows-only symlink coverage | Modify | Remove direct Windows host operations and platform listings so maintained host-specific behavior matches the three supported targets; retain generic shell support and security filters for untrusted Windows-formatted input. |
| `src/sop/types.rs`, `src/sop/engine.rs`, `src/sop/dispatch.rs`, `src/sop/mod.rs` | Modify | Remove the peripheral trigger source, matching, dispatch helper, and associated tests. |
| `Cargo.toml`, `.cargo/config.toml`, `Cargo.lock` | Modify | Remove the robot workspace member, hardware-only dependencies/features, firmware package include paths, and unsupported target configurations; update the lock for the remaining workspace. |
| `.github/workflows/ci.yml`, `ci-full.yml`, `release.yml`, `promote-release.yml` | Modify | Align host artifact matrices to the three target triples; retain Docker `linux/amd64,linux/arm64`. |
| `scripts/bootstrap.sh`, `flake.nix`, `Dockerfile` | Modify | Accept only supported host tuples, set Nix dev-shell systems to the three host tuples, and remove hardware claims from container metadata. |
| Dedicated Android/hardware docs and board datasheets | Delete | The guides are exclusively for unsupported Android, embedded boards, firmware, or robotics. |
| `README.md`, `docs/README.md`, `docs/SUMMARY.md`, `docs/docs-inventory.md`, `docs/structure/README.md`, `docs/feature-overview.md` | Modify | Remove hardware navigation and document only the host runtime. |
| `docs/commands-reference.md`, `docs/config-reference.md`, `docs/sop/**`, security/operations/troubleshooting/CI/release docs | Modify | Remove deleted command/config/platform claims and update cross-references. |
| `docs/network-deployment.md` | Rewrite in place | Keep generic Linux gateway deployment guidance; remove Raspberry Pi and peripherals. |
| `flake.lock`, `fuzz/Cargo.lock` | Restore/preserve | These are reproducibility metadata for still-relevant Nix and fuzz tooling, not unsupported platform features. |
| `.specify/**`, `specs/001-supported-host-prune/**` | Create/update | Spec Kit scaffold, Brownfield-tailored constitution/templates, feature specification, plan, tasks, and workflow guidance. |
| `AGENTS.md` | Modify | Remove the peripheral trait and hardware-doc references; align the repository map and target policy with the new product boundary. |

## Technical Context

**Language / version**: Rust 2024; workspace MSRV is `1.96` in `Cargo.toml`. The web UI is Vue/TypeScript.

**Primary dependencies**: Cargo workspace and optional pnpm web package; GitHub Actions for CI/release; Nix for development shells.

**Storage**: No storage schema change. Removed hardware config is no longer part of the product schema; unknown legacy keys follow the existing config loader's warning behavior.

**Testing and quality gates**: Existing unit tests are colocated in `src/`; integration tests are under `tests/`. CI uses locked checks and release builds. Formatting and clippy are provided by `scripts/ci/rust_quality_gate.sh`.

**Supported host targets**:

| User label | Rust target triple |
| --- | --- |
| `linux-amd64` | `x86_64-unknown-linux-gnu` |
| `linux-arm64` | `aarch64-unknown-linux-gnu` |
| `aarch64-darwin` | `aarch64-apple-darwin` |

**Project type**: Rust CLI/application workspace with an optional web client.

**Constraints**: Preserve unrelated agent runtime features and pre-existing edits; remove only product support outside the accepted host and generic-PC boundary; keep lockfiles reproducible.

## Constitution Check

- [x] Capability and security boundaries remain explicit; removing device execution reduces peripheral attack surface.
- [x] Existing provider/channel/tool/runtime module boundaries remain; only board-specific surfaces are removed.
- [x] The change is bounded to target and hardware scope; no unrelated runtime redesign is planned.
- [x] CI, installer, Nix, package metadata, and documentation will use the same target table.
- [x] Validation and locale follow-through are specified in the task list.

## Design and Implementation

1. Remove the embedded/robot subsystem at its public edges first: CLI commands and arguments, configuration types, onboarding prompts, agent tool injection, and SOP peripheral events.
2. Delete board-only modules, tool implementations, firmware, robot crate, and dedicated board/Android documentation.
3. Remove dependencies and Cargo features used exclusively by those surfaces. Preserve dependencies used by unrelated features, even where transitive packages list other operating systems.
4. Reduce build/release targets to three Rust target triples. Keep Docker's two Linux architectures because Darwin is not a container target.
5. Narrow install detection to Linux x86_64, Linux aarch64, and Darwin arm64/aarch64; reject ARMv7, Intel Darwin, Windows, Android, and unknown host tuples.
6. Keep generic network deployment guidance by rewriting it for ordinary Linux hosts. Update documentation hubs, references, SOP docs, security tables, CI/release descriptions, and repository inventory.
7. Run the permitted validation and static audits; review the complete diff, including pre-existing worktree changes, before squash merging into the existing `main` worktree. Do not push.

### Project Structure

```text
src/                  # Rust application, config, providers, channels, tools, runtime
tests/                # Integration tests
web/                  # Vue and TypeScript client
docs/                 # User, operations, reference, and contributor documentation
scripts/              # Install and CI helpers
.github/workflows/    # CI and release automation
.specify/             # Spec Kit templates, scripts, and Brownfield setup
specs/001-supported-host-prune/  # Specification and implementation artifacts
```

**Structure Decision**: Keep the existing application structure and remove hardware-only branches rather than replacing the runtime architecture.

## Complexity Tracking

| Violation | Why needed | Simpler alternative rejected because |
| --- | --- | --- |
| N/A | N/A | No constitution exception is planned. |
