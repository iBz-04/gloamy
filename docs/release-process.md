# Gloamy Release Process

This runbook defines the maintainers' standard release flow for the workflows currently present in this repository.

Last verified: **March 27, 2026**.

## Release Goals

- Keep releases predictable and repeatable.
- Publish only from code already in `master`.
- Verify multi-target artifacts before publish.
- Keep release cadence regular even with high PR volume.

## Workflow Contract

Release automation currently lives in:

- `.github/workflows/release.yml` (`Beta Release`)
- `.github/workflows/promote-release.yml` (`Promote Release`)

### Beta release path

- Trigger: every push to `master`.
- Output:
  - prerelease GitHub tag/version in form `v<cargo-version>-beta.<run-number>`
  - build artifacts (`tar.gz`/`zip`) and `SHA256SUMS`
  - Docker images tagged with beta tag and `beta`

### Stable release path

- Trigger: manual `workflow_dispatch` of `Promote Release` with input `version` (`X.Y.Z`).
- Guardrails enforced by workflow:
  - input must be semver `X.Y.Z`
  - `Cargo.toml` version must exactly match input version
  - stable tag `vX.Y.Z` must not already exist on origin
- Output:
  - stable GitHub release (`vX.Y.Z`) with build artifacts and `SHA256SUMS`
  - Docker images tagged with `vX.Y.Z` and `latest`

## Maintainer Procedure

### 1) Preflight on `master`

1. Ensure CI is green for the target commit.
2. Confirm no known high-priority regressions are unresolved.
3. Confirm release matrix targets are still expected (Linux x86_64/aarch64, macOS arm64, Windows x86_64).

### 2) Beta verification (automatic)

1. Merge release-ready changes to `master`.
2. Monitor `Beta Release` workflow run.
3. Verify prerelease assets and Docker `beta` tag update.

### 3) Promote stable release (manual)

1. Ensure `Cargo.toml` has the intended stable version.
2. Run `Promote Release` workflow manually with `version=<Cargo.toml version>`.
3. Monitor `validate`, `build`, `publish`, and `docker` jobs.

### 4) Post-release validation

1. Verify GitHub Release assets are downloadable.
2. Verify checksum file is present and non-empty.
3. Verify GHCR tags (`vX.Y.Z`, `latest`) are published.
4. Smoke-test one installation path that consumes release assets.

## Emergency / Recovery Path

If stable promotion fails:

1. Fix the issue on `master`.
2. Re-run `Promote Release` with the same target version if no tag was created.
3. If tag was already created but publish steps failed, resolve the workflow issue and re-run the failed job path.

## Operational Notes

- Keep release changes small and reversible.
- Prefer one release checklist/issue per version so handoff is clear.
- Avoid ad-hoc release operations outside the documented workflows.
