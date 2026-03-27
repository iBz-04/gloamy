# Gloamy Docs Structure Map

This page defines the documentation structure across three axes:

1. Language
2. Part (category)
3. Function (document intent)

Last refreshed: **February 22, 2026**.

## 1) By Language

| Language | Entry point | Canonical tree | Notes |
|---|---|---|---|
| English | `docs/README.md` | `docs/` | Source-of-truth runtime behavior docs are authored in English first. |
| Localization index | `docs/i18n/README.md` | `docs/i18n/` | This repository snapshot currently ships English-first docs plus compatibility stubs (`*.vi.md`) that point to English guides. |

## 2) By Part (Category)

These directories are the primary navigation modules by product area.

- `docs/getting-started/` for initial setup and first-run flows
- `docs/reference/` for command/config/provider/channel reference indexes
- `docs/operations/` for day-2 operations, deployment, and troubleshooting entry points
- `docs/security/` for security guidance and security-oriented navigation
- `docs/hardware/` for board/peripheral implementation and hardware workflows
- `docs/` root policy/process docs (`pr-workflow.md`, `reviewer-playbook.md`, `ci-map.md`) for contribution and CI/review processes
- `docs/project/` for project snapshots, planning context, and status-oriented docs

## 3) By Function (Document Intent)

Use this grouping to decide where new docs belong.

### Runtime Contract (current behavior)

- `docs/commands-reference.md`
- `docs/providers-reference.md`
- `docs/channels-reference.md`
- `docs/config-reference.md`
- `docs/operations-runbook.md`
- `docs/troubleshooting.md`
- `docs/one-click-bootstrap.md`

### Setup / Integration Guides

- `docs/custom-providers.md`
- `docs/zai-glm-setup.md`
- `docs/langgraph-integration.md`
- `docs/network-deployment.md`
- `docs/matrix-e2ee-guide.md`
- `docs/mattermost-setup.md`
- `docs/nextcloud-talk-setup.md`

### Policy / Process

- `docs/pr-workflow.md`
- `docs/reviewer-playbook.md`
- `docs/ci-map.md`

### Proposals / Roadmaps

- `docs/sandboxing.md`
- `docs/resource-limits.md`
- `docs/audit-logging.md`
- `docs/agnostic-security.md`
- `docs/frictionless-security.md`
- `docs/security-roadmap.md`

### Snapshots / Time-Bound Reports

- `docs/project-triage-snapshot-2026-02-18.md`

### Assets / Templates

- `docs/datasheets/`
- `docs/doc-template.md`

## Placement Rules (Quick)

- New runtime behavior docs must be linked from the appropriate category index and `docs/SUMMARY.md`.
- Navigation changes must keep the active entry points (`README.md`, `docs/README.md`, `docs/SUMMARY.md`) aligned with the current file tree.
- If new locale trees are added later, update this structure map and `docs/SUMMARY.md` in the same change.
