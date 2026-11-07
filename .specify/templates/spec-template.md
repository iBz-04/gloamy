# Feature Specification: [FEATURE NAME]

**Feature Branch**: `[###-feature-name]`

**Created**: [DATE]

**Status**: Draft

**Input**: User description: "$ARGUMENTS"

## User Scenarios & Testing

Write prioritized user journeys in plain language. Each journey MUST be independently understandable and have acceptance scenarios.

### User Story 1 - [Brief Title] (Priority: P1)

[Describe the user journey and why it is valuable.]

**Independent Test**: [Describe an observable outcome without prescribing implementation.]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [observable result]

---

### Edge Cases

- [Boundary, failure, or compatibility case]

## Requirements

### Functional Requirements

- **FR-001**: [A testable capability]

### Existing Project Impact

- **User-facing surfaces**: Identify affected CLI, configuration, provider/channel, runtime, web, and release contracts; mark unaffected surfaces N/A.
- **Brownfield paths**: Name existing modules, docs, scripts, or workflow files that change. For removals, list exact paths and the evidence for each.
- **Compatibility and migration**: State behavior for existing configuration, files, data, and users. If no migration is needed, say why.
- **Preservation boundary**: State important adjacent behavior that must remain unchanged.

### Key Entities *(include only when the feature changes domain data)*

- **[Entity]**: [Meaning and important relationships]

## Success Criteria

- **SC-001**: [A measurable, user-observable outcome]
- **SC-002**: [A measurable outcome for compatibility, release, or operations]

## Assumptions

- [Evidence-based assumption]
- [Dependency or out-of-scope boundary]
