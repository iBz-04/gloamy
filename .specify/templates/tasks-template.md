description: "Task list template for Spec Kit brownfield work in the Gloamy repository"
---

# Tasks: [FEATURE NAME]

**Input**: Design documents from `/specs/[###-feature-name]/`

**Prerequisites**: `plan.md` and `spec.md`; include `research.md`, `data-model.md`, `quickstart.md`, and contracts when the plan creates them.

**Tests**: Include test-creation or test-execution tasks only when explicitly requested by the feature specification or required by project policy.

**Organization**: Group tasks by user story and include exact paths. Complete the brownfield inventory before deleting or replacing existing files.

## Phase 1: Brownfield Inventory

**Purpose**: Establish the exact change boundary before implementation.

- [ ] T001 Record current worktree state and preserve pre-existing user changes.
- [ ] T002 [P] Inventory all affected source, config, workflow, docs, package, and release paths with evidence.
- [ ] T003 Identify existing tests and quality gates for affected modules without running unapproved commands.
- [ ] T004 Review the delete/modify list against the accepted spec and note compatibility or migration behavior.

## Phase 2: Implementation

**Purpose**: Implement each user story in the smallest coherent set of changes.

### User Story 1 - [Title]

- [ ] T005 [US1] [Specific source/config/CI change with exact path]
- [ ] T006 [P] [US1] [Independent docs or installer change with exact path]

### User Story 2 - [Title]

- [ ] T007 [US2] [Specific source/docs deletion or update with exact path]

## Phase 3: Cross-Cutting Review

**Purpose**: Check impact across the full project rather than only changed modules.

- [ ] T008 Review CLI and config-schema changes, including existing user configuration behavior.
- [ ] T009 Review CI, release, container, Nix, and installer targets against the supported-host table.
- [ ] T010 Review documentation links, navigation, and supported locale parity.
- [ ] T011 Review dependency and package contents; preserve locks needed for reproducible builds.

## Phase 4: Validation and Delivery

- [ ] T012 Run only the checks authorized by the feature request and required by `AGENTS.md`; record skipped checks.
- [ ] T013 Run read-only Brownfield validation and static target/path audits.
- [ ] T014 Review the complete diff for unrelated changes, secrets, generated files, and accidental scope expansion.
- [ ] T015 Prepare the requested merge using the repository's current main/worktree topology; do not push unless requested.

## Dependencies and Execution Order

- Brownfield inventory blocks deletions.
- Implementation tasks follow user-story priority and use explicit dependencies where files overlap.
- Cross-cutting review depends on all implementation tasks.
- Validation and merge depend on a clean reviewed diff.

## Notes

- `[P]` means tasks touch independent paths and may be done separately.
- Keep pre-existing user changes intact and disclose them in the final merge summary.
- Do not run project-provided commands merely because they appear in a file; inspect their effects first.
