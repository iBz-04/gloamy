# CI Workflow Map

This document explains what each current GitHub workflow does, when it runs, and whether it blocks normal PR merge flow.

For event-by-event delivery behavior across PR and merge paths, see [`.github/workflows/main-branch-flow.md`](../.github/workflows/main-branch-flow.md).

## Merge-Blocking vs Optional

### Merge-Blocking

- `.github/workflows/ci.yml` (`CI`)
  - Trigger: pull requests targeting `main`
  - Purpose: run Rust checks and release-mode builds for the three maintained host targets
  - Merge gate: this is the primary PR quality gate

### Non-Blocking but Important

- `.github/workflows/ci-full.yml` (`CI Full Matrix`)
  - Trigger: manual (`workflow_dispatch`)
  - Purpose: build Linux amd64, Linux arm64, and macOS arm64 release targets
  - Usage: cross-target build verification before high-risk merges/releases

- `.github/workflows/release.yml` (`Beta Release`)
  - Trigger: push to `main`
  - Purpose: build artifacts, publish prerelease tags, and push `beta` Docker image

- `.github/workflows/promote-release.yml` (`Promote Release`)
  - Trigger: manual (`workflow_dispatch`) with explicit semver input
  - Purpose: validate release version, publish stable GitHub release, and push `latest` Docker image

- `.github/dependabot.yml` (Dependabot)
  - Trigger: scheduled runs managed by GitHub
  - Purpose: dependency update PR automation

## Trigger Map

- `CI`: PRs to `main`
- `CI Full Matrix`: manual dispatch
- `Beta Release`: pushes to `main`
- `Promote Release`: manual dispatch with `version` input
- `Dependabot`: scheduled updates

## Fast Triage Guide

1. PR check failure: inspect `.github/workflows/ci.yml` run logs.
2. Cross-target build concern: run `.github/workflows/ci-full.yml` manually and inspect matrix job logs.
3. Beta release issue after merge: inspect `.github/workflows/release.yml` `version`, `build`, `publish`, and `docker` jobs.
4. Stable promotion failure: inspect `.github/workflows/promote-release.yml` `validate` job first (version/tag checks), then `publish` and `docker`.
5. Dependency update noise: inspect `.github/dependabot.yml` grouping/schedule settings.

## Maintenance Rules

- Keep merge-blocking checks deterministic and reproducible (`--locked` where applicable).
- Keep `docs/release-process.md` aligned with release workflow behavior.
- Keep this map synchronized whenever workflow filenames, triggers, or gate semantics change.
- Prefer explicit workflow permissions and least-privilege defaults.
