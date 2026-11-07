# Brownfield Review Checklist: [FEATURE NAME]

**Purpose**: Review a change against the existing repository and its user-facing contracts.
**Created**: [DATE]
**Feature**: [Link to `spec.md`]

## Requirements and Existing Behavior

- [ ] Accepted requirements map to evidence from the current project.
- [ ] Existing CLI and configuration behavior affected by the change is identified.
- [ ] Compatibility and migration behavior is stated.
- [ ] Unrelated runtime behavior is named and preserved.

## Scope and Deletions

- [ ] Every deleted path has a reason tied to the specification.
- [ ] No user data, secrets, generated output, or unrelated files are included.
- [ ] Public source, command, configuration, and package surfaces agree.

## Architecture and Release

- [ ] Supported targets match the project constitution and release artifact names.
- [ ] CI, installer, Docker, Nix, and package metadata are aligned.
- [ ] Unsupported targets fail clearly or are absent from supported workflows.

## Documentation

- [ ] README, docs hubs, indexes, and internal links are updated.
- [ ] Supported locale navigation and equivalents follow `AGENTS.md`.
- [ ] No removed hardware, firmware, or platform instructions remain in active docs.

## Validation and Review

- [ ] Required quality gates and checks are recorded with results.
- [ ] Skipped checks are explicit.
- [ ] The complete diff is reviewed for scope, regressions, and reproducibility.
