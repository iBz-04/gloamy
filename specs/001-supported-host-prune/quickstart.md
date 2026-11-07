# Validation Quickstart

This guide records the repository checks for the architecture-prune change. Run only the checks approved for the task; test execution is not assumed.

## Static review

```bash
git diff --check
rg -n 'x86_64-apple-darwin|x86_64-pc-windows|aarch64-linux-android|armv7-unknown-linux|thumbv|riscv64' .github .cargo scripts docs Cargo.toml flake.nix
rg -n -i 'Raspberry Pi|STM32|Arduino|Nucleo|ESP32|Uno Q|firmware|peripheral' src crates docs README.md Cargo.toml
```

The searches should find no active product/build/docs support for removed targets or board features. Exclude dependency lockfiles and the feature's own historical removal notes from the platform audit.

## Workflow artifacts

- [Reusable Brownfield prompt](reusable-prompt.md) gives the repeatable prompt and Spec Kit CLI sequence.
- [Workflow diagram](workflow.md) visualizes the gates from brownfield discovery through local squash merge.
- [Brownfield validation report](brownfield-validation.md) records the read-only scaffold and project-profile review.

The Mermaid source is maintained in [workflow.mermaid.json](workflow.mermaid.json). It was linted against Mermaid 11.16.1; rendering was disabled in this environment.

## Rust build checks

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked -j4
cargo clippy --locked --all-targets -- -D warnings
cargo build --release --locked --target x86_64-unknown-linux-gnu
cargo build --release --locked --target aarch64-unknown-linux-gnu
cargo build --release --locked --target aarch64-apple-darwin
```

Cross-target builds require the relevant Rust target and native linker/toolchain. CI remains the authoritative cross-platform build environment.

## Web and docs checks

```bash
cd web && pnpm build
```

Review all changed documentation links and locale navigation against `AGENTS.md`. Do not treat command presence as evidence that the command was executed.
