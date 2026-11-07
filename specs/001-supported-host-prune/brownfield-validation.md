# Brownfield Validation Report

**Feature**: [Supported Host Architectures and Generic PC Scope](spec.md)
**Checked**: 2026-11-06
**Workflow**: Spec Kit Brownfield `validate` v1.0.0, read-only

## Plugin and Project State

| Check | Result | Evidence |
| --- | --- | --- |
| Existing Git project | Pass | `git` worktree at `/home/avery/build/other-peoples-software/wt/app-server-oauth-v2` |
| Spec Kit scaffold | Pass | `.specify/`, `.specify/memory/constitution.md`, `.specify/templates/`, and `specs/` exist |
| Codex skills | Pass | `.agents/skills/speckit-*` exists |
| Plugin compatibility | Pass | Spec Kit CLI 1.3.2; Brownfield compatibility reported true; required workflow files present |
| Exact upstream submodule revision check | Unknown | Plugin reports revision inspection unavailable in its cached source; no mismatch was reported |

## Constitution

| Rule | Result | Evidence |
| --- | --- | --- |
| Primary languages and frameworks | Pass | Rust workspace in `Cargo.toml`; Vue/TypeScript client in `web/package.json` |
| Referenced project directories | Pass | `src/`, `web/`, `docs/`, `scripts/`, `.github/workflows/`, `tests/` exist |
| Test locations | Pass | Unit tests are colocated under `src/`; integration tests are under `tests/` |
| Naming conventions | Pass | Rust modules use `snake_case` (for example `src/config/schema.rs`); docs use lowercase hyphenated names (for example `docs/config-reference.md`) |
| Branch convention | Pass | Existing branches include `feat/`, `fix/`, and `chore/` prefixes; current worktree uses `feat/` |
| Architecture boundary | Pass | Constitution, README, CI/release workflows, bootstrap detection, and Nix systems list the three supported targets |

## Templates and Agent Guidance

| Artifact | Result | Evidence |
| --- | --- | --- |
| Spec template | Pass | Brownfield impact, compatibility, preservation, and observable acceptance sections match this Rust CLI/web repository |
| Plan template | Pass | References actual Rust, web, docs, scripts, tests, Cargo, pnpm, CI, and release paths |
| Tasks template | Pass | Requires inventory before deletion and separates implementation, cross-cutting review, validation, and delivery |
| Checklist template | Pass | Covers compatibility, deletion boundaries, release targets, docs navigation, and validation |
| `AGENTS.md` | Pass | Root guidance applies repository-wide and its directory references exist; no conflicting per-directory ownership files were found |

## Configuration and Scope Drift

- `Config::load_or_init` uses `serde_ignored` and emits `Unknown config key ignored` warnings for removed legacy keys, so hardware settings do not silently reactivate tools.
- The Cargo workspace no longer includes `crates/robot`; `Cargo.lock` no longer contains `gloamy-robot`, `probe-rs`, `nusb`, `rppal`, or `tokio-serial` packages. `flake.lock` and `fuzz/Cargo.lock` remain as reproducibility metadata.
- Remaining Windows strings in source are limited to security input/command filtering or generic shell/browser data; direct Windows service, browser-launch, screenshot, and secret-permission implementations were removed. Transitive Windows crates in `Cargo.lock` are dependency metadata, not supported product targets.
- Read-only Markdown link scan checked 55 changed or new Markdown files; all local path targets resolved.

## Validation Limits

- Brownfield validation was performed from the pinned read-only workflow checklist; the plugin exposes source retrieval and environment/status checks but no separate validator execution tool.
- `git diff --check` passed. Rustfmt passed on the edited Rust modules. `cargo fmt --all -- --check` reports broad formatting drift across unchanged and pre-existing modified workspace files with the installed Rust 1.99 formatter; no workspace-wide reformat was applied.
- `cargo check --workspace --all-features --locked -j4` reached the crate but failed on existing unsafe `std::env::set_var` / `remove_var` calls in `src/config/schema.rs` and `src/onboard/wizard.rs`; the architecture change did not modify those calls.
- Project tests were not run.
