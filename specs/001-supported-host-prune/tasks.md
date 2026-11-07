description: "Task list for supported host architecture pruning"
---

# Tasks: Supported Host Architecture Prune

**Input**: Design documents from `/specs/001-supported-host-prune/`

**Prerequisites**: `spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/platforms.md`, `quickstart.md`

**Tests**: No new test cases or test execution are included by default. Remove tests that exist only for deleted hardware surfaces; run permitted build/static validation and record skipped test execution.

## Phase 1: Brownfield Inventory

**Purpose**: Preserve existing worktree edits and establish the deletion boundary.

- [x] T001 Record branch, worktree, remote, existing dirty paths, and clean `main` worktree in the task notes.
- [x] T002 [P] Inventory architecture, firmware, board, config, CLI, installer, CI, and docs references using the existing checkout.
- [x] T003 [P] Confirm existing project guidance and quality gates in `AGENTS.md` and CI workflows.
- [x] T004 Review the accepted architecture list and lockfile/reproducibility exceptions in `spec.md` and `plan.md`.

## Phase 2: User Story 1 - Build and install only the supported hosts (Priority: P1)

**Goal**: CI, release packaging, host detection, and Nix development systems agree on the three target triples.

**Independent Test**: Static inspection of target rows and host tuples; build checks are recorded separately.

- [x] T005 [US1] Reduce `.github/workflows/ci.yml` and `.github/workflows/ci-full.yml` matrices to Linux amd64, Linux arm64, and arm64 Darwin.
- [x] T006 [US1] Reduce `.github/workflows/release.yml` and `.github/workflows/promote-release.yml` host artifact jobs to the same three Rust target triples; retain Docker Linux amd64/arm64.
- [x] T007 [US1] Restrict `scripts/bootstrap.sh` host detection to Linux x86_64, Linux aarch64/arm64, and Darwin arm64/aarch64 with an explicit unsupported-host result.
- [x] T008 [US1] Remove unsupported Android and alternate musl linker entries from `.cargo/config.toml` and constrain `flake.nix` dev shells to the three supported host systems.
- [x] T009 [US1] Remove board/firmware wording from `Cargo.toml` and `Dockerfile` package descriptions while preserving unrelated package metadata.

## Phase 3: User Story 2 - Keep the runtime generic and host-only (Priority: P1)

**Goal**: Remove all board, MCU, firmware, and robotics features from code, package contents, CLI, configuration, onboarding, and SOP events.

**Independent Test**: Static search and source review show no active hardware/firmware/robot surfaces in maintained source or package paths.

- [x] T010 [US2] Remove `HardwareCommands` and `PeripheralCommands` from `src/lib.rs`; remove `hardware`/`peripheral` modules, CLI subcommands, flags, and dispatch from `src/main.rs`.
- [x] T011 [US2] Remove hardware/peripheral config exports, fields, structs, defaults, and feature-only config fixtures from `src/config/mod.rs` and `src/config/schema.rs`.
- [x] T012 [US2] Remove the physical hardware onboarding step, imports, config fields, and summary output from `src/onboard/wizard.rs`.
- [x] T013 [US2] Remove peripheral override parameters and tool registration from `src/agent/loop_.rs` and all call sites.
- [x] T014 [US2] Delete `src/peripherals/`, `src/hardware/`, and `src/tools/hardware_board_info.rs`, `src/tools/hardware_memory_map.rs`, and `src/tools/hardware_memory_read.rs`; remove their module and registry exports from `src/tools/mod.rs`.
- [x] T015 [US2] Remove peripheral trigger variants, matching, dispatch helpers, and peripheral-only tests from `src/sop/types.rs`, `src/sop/engine.rs`, `src/sop/dispatch.rs`, and `src/sop/mod.rs`.
- [x] T016 [US2] Remove the `crates/robot` workspace member and crate, hardware-only dependencies/features, and firmware package include paths from `Cargo.toml`; refresh `Cargo.lock` for the remaining workspace.
- [x] T017 [US2] Remove all remaining `firmware/**` files and board/robot package content; preserve `flake.lock` and `fuzz/Cargo.lock` as unrelated reproducibility metadata.
- [x] T017a [US2] Remove direct Windows host operations from service management, browser launching, screenshot capture, secret-file permissions, host integration listing, and Windows-only symlink coverage; retain generic security filters and shell completion support.

## Phase 4: User Story 3 - Keep user and contributor guidance consistent (Priority: P2)

**Goal**: All maintained docs, help, inventories, and workflow notes reflect generic support for exactly the three hosts.

**Independent Test**: Link and target audits show no stale active hardware instructions or unsupported host claims.

- [x] T018 [US3] Delete dedicated board, MCU, firmware, robot, Android, and datasheet docs: `docs/adding-boards-and-tools.md`, `docs/arduino-uno-q-setup.md`, `docs/nucleo-setup.md`, `docs/hardware-peripherals-design.md`, `docs/hardware/README.md`, `docs/android-setup.md`, and `docs/datasheets/{arduino-uno,esp32,nucleo-f401re}.md`.
- [x] T019 [US3] Remove hardware navigation and optional robot content from `README.md`, `AGENTS.md`, `docs/README.md`, `docs/SUMMARY.md`, `docs/docs-inventory.md`, `docs/structure/README.md`, and `docs/feature-overview.md`.
- [x] T020 [US3] Remove hardware/peripheral command and config sections from `docs/commands-reference.md` and `docs/config-reference.md`; update SOP source documentation in `docs/sop/README.md`, `docs/sop/connectivity.md`, and `docs/sop/syntax.md`.
- [x] T021 [US3] Rewrite `docs/network-deployment.md` as generic Linux deployment guidance; remove Raspberry Pi, GPIO, board, and peripheral setup.
- [x] T022 [US3] Scope `docs/agnostic-security.md`, remove the hardware-focused `docs/frictionless-security.md` proposal, and update operations, troubleshooting, LangGraph, Cargo-slicer, and Rust toolchain guidance to the maintained host targets.
- [x] T023 [US3] Update `.github/workflows/README.md`, `docs/ci-map.md`, `docs/release-process.md`, and `.github/workflows/main-branch-flow.md` to describe only the supported host targets.

## Phase 5: Cross-cutting validation and delivery

- [x] T024 [P] Search maintained source, build config, release workflows, and active docs for removed target and hardware surfaces; resolve every non-historical hit.
- [x] T025 [P] Review Cargo and Nix lockfiles, package contents, and config loader behavior to ensure reproducibility and explicit legacy-key handling.
- [x] T026 Check changed markdown links and documentation navigation; preserve locale parity required by `AGENTS.md`.
- [x] T027 Run `git diff --check` and approved Rust formatting/build/static checks; record any test commands not run.
- [x] T028 Load `brownfield.validate` and verify the Spec Kit scaffold/profile against current project structure without running project-provided test commands.
- [x] T029 Review the full worktree diff, including pre-existing user changes, before creating a single squash commit on `main`.
- [ ] T030 Squash-merge the completed worktree into the existing local `main` branch and leave remote push untouched.

## Dependencies and Execution Order

- T001-T004 establish the baseline and block deletion work.
- T005-T009 can proceed alongside T010-T017 when edits do not touch the same files.
- T018-T023 follow the code-surface removal inventory.
- T024-T028 depend on all implementation and documentation edits.
- T029-T030 depend on a reviewed diff and successful permitted checks.

## Notes

- `[P]` tasks use separate files and can be reviewed independently.
- The current branch is two commits behind local `main`; reconcile that topology during final squash merge.
- Existing user worktree changes are retained and reviewed, not reset.
