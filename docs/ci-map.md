# CI Workflow Map

This document explains what each current GitHub workflow does, when it runs, and whether it blocks normal PR merge flow.

For event-by-event delivery behavior across PR and merge paths, see [`.github/workflows/master-branch-flow.md`](../.github/workflows/master-branch-flow.md).

## Merge-Blocking vs Optional

### Merge-Blocking

- `.github/workflows/ci.yml` (`CI`)
  - Trigger: pull requests targeting `master`
  - Purpose: run tests (`cargo nextest`) and release-mode builds for primary targets
  - Merge gate: this is the primary PR quality gate

### Non-Blocking but Important

- `.github/workflows/ci-full.yml` (`CI Full Matrix`)
  - Trigger: manual (`workflow_dispatch`)
  - Purpose: run expanded cross-platform release builds (Linux ARM64, macOS Intel, Windows)
  - Usage: deeper compatibility verification before high-risk merges/releases

- `.github/workflows/release.yml` (`Beta Release`)
  - Trigger: push to `master`
  - Purpose: build artifacts, publish prerelease tags, and push `beta` Docker image

- `.github/workflows/promote-release.yml` (`Promote Release`)
  - Trigger: manual (`workflow_dispatch`) with explicit semver input
  - Purpose: validate release version, publish stable GitHub release, and push `latest` Docker image

- `.github/dependabot.yml` (Dependabot)
  - Trigger: scheduled runs managed by GitHub
  - Purpose: dependency update PR automation

## Trigger Map

- `CI`: PRs to `master`
- `CI Full Matrix`: manual dispatch
- `Beta Release`: pushes to `master`
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
