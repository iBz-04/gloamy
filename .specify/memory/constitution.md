# Gloamy Constitution

## Project Identity

Gloamy is a Rust-first autonomous agent runtime for CLI use, model providers, communication channels, tools, memory, gateways, and service workflows. The repository also contains a Vue and TypeScript web client under `web/`.

## Core Principles

### I. Secure and Explicit Boundaries

Capabilities that touch files, networks, processes, secrets, or external services MUST remain explicit and least-privileged. Errors MUST be visible; the runtime MUST NOT silently broaden permissions or capabilities. Never log credentials or sensitive payloads.

### II. Preserve Module and Trait Boundaries

Extend the provider, channel, tool, memory, observer, and runtime surfaces through their existing narrow traits and factories. Keep policy, transport, persistence, and presentation responsibilities separate. Avoid cross-cutting rewrites when a local change is sufficient.

### III. Keep Changes Small and Reversible

Prefer the smallest complete change that meets an accepted requirement. Do not add speculative abstractions, feature flags, configuration, or dependencies. For higher-risk changes, record the rollback path and keep the change reviewable.

### IV. Maintain Reproducible Quality Gates

Rust changes MUST preserve formatting, lint, locked dependency, and CI expectations. Use existing unit and integration coverage according to the risk of the changed path. Web changes MUST preserve the `web/` package's type-check and build requirements. Do not weaken CI to make a change pass.

### V. Support Only the Defined Host Targets

The supported generic host architectures are exactly `linux-amd64`, `linux-arm64`, and `aarch64-darwin`. Their Rust target triples are `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, and `aarch64-apple-darwin`. Embedded firmware, microcontrollers, board-specific GPIO, and robotics hardware are outside the product boundary.

## Code Boundaries

- `src/` contains the Rust application, configuration, providers, channels, tools, gateway, memory, security, runtime, and CLI modules.
- `web/` contains the Vue and TypeScript web client.
- `docs/` contains user, operations, architecture, and contributor documentation.
- `scripts/` contains repository maintenance, install, and CI helper scripts.
- `.github/workflows/` contains CI and release automation; its supported target matrix MUST match this constitution.

## Naming and Repository Conventions

- Rust module and source file names use `snake_case`; documentation paths use the repository's existing lowercase, hyphenated naming style.
- Existing branch names use prefixes such as `feat/`, `fix/`, and `chore/`.
- Commit messages follow the observed Conventional Commit style, such as `feat:`, `fix:`, `chore:`, and `docs:`.
- `AGENTS.md` remains the detailed repository engineering protocol. This constitution supplements it and MUST NOT weaken its requirements.

## Dependencies and Compatibility

- Add a dependency only for a concrete requirement; prefer existing project dependencies and keep optional integrations feature-gated when that matches the current design.
- Treat CLI commands, configuration, provider/channel contracts, and published artifact names as user-facing interfaces. Document breaking changes and migration behavior.
- Keep build, release, installer, and documentation target lists aligned to the three supported architectures.
- Preserve lock files used for reproducible dependency resolution; transitive dependency metadata is not a product support declaration.

## Development Workflow and Quality Gates

- Read repository guidance and adjacent implementation before editing; inspect existing tests before changing behavior.
- For Rust changes, follow the risk-based validation requirements in `AGENTS.md` and the repository scripts: `cargo fmt --all -- --check`, relevant `cargo check`/`cargo clippy`, and applicable unit or integration checks.
- For release-target changes, verify each supported Rust target in CI/release configuration and update the installer mapping and user documentation together.
- For documentation navigation or runtime-contract changes, update supported locale entry points and equivalent localized references as required by `AGENTS.md`.
- Do not execute project-discovered commands until their purpose and effects are understood.

## Governance

Changes to this constitution require a reviewed change, a semantic version increment, and an explanation in the change summary. Feature specifications and plans MUST identify constitution checks and any justified exception before implementation. Reviewers MUST check that behavior, tests, docs, CI, and release surfaces agree. When this constitution and `AGENTS.md` appear inconsistent, stop and resolve the conflict rather than silently weakening either policy.

**Version**: 1.0.0 | **Ratified**: 2026-11-06 | **Last Amended**: 2026-11-06
