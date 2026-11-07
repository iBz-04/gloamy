# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]

**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

## Summary

[Summarize the user outcome, chosen approach, and major removals or compatibility effects.]

## Brownfield Findings

- **Current behavior and architecture**: [Cite concrete repository paths.]
- **Naming and governance**: [Use observed conventions and `AGENTS.md`.]
- **Files to create/modify/delete**: [Provide file-by-file inventory and evidence before implementation.]
- **Preserve**: [List adjacent behavior and lock files that remain in scope.]

## Technical Context

**Language / version**: Rust 2024; workspace MSRV is recorded in `Cargo.toml`.

**Primary dependencies**: Cargo workspace; optional web client uses Vue, TypeScript, and pnpm.

**Storage**: N/A unless this feature changes a documented persistence contract.

**Testing and quality gates**: Existing Rust unit/integration tests under `src/` and `tests/`; repository CI uses locked Cargo checks, formatting, clippy, and target builds. The web client has its own package scripts.

**Supported host targets**:

| User label | Rust target |
| --- | --- |
| `linux-amd64` | `x86_64-unknown-linux-gnu` |
| `linux-arm64` | `aarch64-unknown-linux-gnu` |
| `aarch64-darwin` | `aarch64-apple-darwin` |

**Project type**: Rust CLI/application workspace with a web client.

**Constraints**: Keep changes aligned with `AGENTS.md`, preserve reproducible locks, avoid unsupported target and hardware claims, and document config/API compatibility.

## Constitution Check

- [ ] Least privilege and explicit errors preserved.
- [ ] Module boundaries and narrow traits preserved.
- [ ] Change is minimal and reversible.
- [ ] Validation follows repository risk tier.
- [ ] Build, release, and docs agree with the three supported targets.

## Design and Implementation

Describe decisions, alternatives, dependency effects, user-visible changes, and any configuration migration. Reference actual project modules and scripts.

### Project Structure

```text
src/                  # Rust application modules
tests/                # Rust integration tests
web/                  # Vue and TypeScript client
docs/                 # User and contributor documentation
scripts/              # Install and CI helpers
.github/workflows/    # CI and release matrices
specs/[feature]/      # This feature's specification and plan
```

**Structure Decision**: Extend the existing module and workflow layout; do not create a parallel implementation tree.

## Complexity Tracking

| Violation | Why needed | Simpler alternative rejected because |
| --- | --- | --- |
| N/A | N/A | No constitution exception is planned. |
