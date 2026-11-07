# Feature Specification: Supported Host Architectures and Generic PC Scope

**Feature Branch**: `feat/app-server-oauth-models-v2`

**Created**: 2026-11-06

**Status**: Implemented

**Input**: User request to retain only `linux-amd64`, `linux-arm64`, and `aarch64-darwin`, and remove non-PC hardware and firmware support.

## User Scenarios & Testing

### User Story 1 - Install on a supported host (Priority: P1)

As a user on one of the supported host architectures, I want the project to build and publish a matching binary so I can install and run it without selecting an unsupported target.

**Why this priority**: The exact supported host list is the central product contract.

**Independent Test**: Inspect release and installation behavior for each supported host, then confirm unsupported hosts receive an explicit unsupported-platform result.

**Acceptance Scenarios**:

1. **Given** a Linux amd64 host, **When** the user follows the documented install path, **Then** the matching Linux amd64 artifact is selected.
2. **Given** a Linux arm64 host or an arm64 Darwin host, **When** the user follows the documented install path, **Then** the matching artifact for that host is selected.
3. **Given** any other host architecture or operating system, **When** the user requests a prebuilt artifact, **Then** the installer reports that the host is unsupported and does not select a different architecture.

---

### User Story 2 - Maintain a generic PC runtime (Priority: P1)

As a maintainer, I want the project to contain only generic host-runtime support so that board-specific firmware, hardware setup, and device-control surfaces do not remain as advertised or buildable product features.

**Why this priority**: The requested product boundary excludes embedded boards and their operational burden.

**Independent Test**: Review the maintained source tree, package contents, CLI surface, configuration schema, and documentation for board, peripheral, robot, or firmware features.

**Acceptance Scenarios**:

1. **Given** a clean checkout, **When** a maintainer inspects package contents and product commands, **Then** no board firmware, flashing workflow, board setup command, or hardware-specific robot crate is present.
2. **Given** a user reads the supported-platform documentation, **When** they search for hardware setup guidance, **Then** only the three supported generic host architectures are described.

---

### User Story 3 - Keep project guidance consistent (Priority: P2)

As a contributor, I want CI, release, installation, development, and user documentation to agree on the supported architecture list so that unsupported targets are not reintroduced accidentally.

**Why this priority**: Consistent repository guidance prevents platform drift after this cleanup.

**Independent Test**: Compare every maintained build/release matrix and platform guide against the supported architecture list and confirm internal links resolve.

**Acceptance Scenarios**:

1. **Given** CI and release configuration, **When** a contributor inspects target entries, **Then** only Linux amd64, Linux arm64, and arm64 Darwin are built as host artifacts.
2. **Given** a platform guide or installer, **When** a contributor checks target examples and host detection, **Then** the same three targets are accepted and unsupported targets are explicitly rejected or absent.

### Edge Cases

- Linux arm64 may also describe single-board computers; the product target remains generic Linux arm64, while board GPIO, robot, serial flashing, and board-specific setup support is removed.
- Dependency lock data may contain transitive package names for other operating systems; those entries do not advertise product support and must not be removed solely because their names mention another target.
- Existing user configurations may contain the removed peripheral or hardware settings; supported behavior must fail or ignore removed options according to the existing configuration compatibility policy rather than silently reactivating removed tooling.
- The Intel Darwin host is unsupported even though it shares the macOS operating system with arm64 Darwin.

## Requirements

### Functional Requirements

- **FR-001**: The project MUST support exactly `linux-amd64`, `linux-arm64`, and `aarch64-darwin` as generic host architecture labels.
- **FR-002**: The build and release experience MUST provide host artifacts for each supported architecture and MUST NOT publish host artifacts for other operating system and architecture combinations.
- **FR-003**: The installer MUST select an artifact only for one of the three supported host combinations and MUST clearly reject unsupported combinations.
- **FR-004**: The maintained product MUST remove firmware, flashing workflows, board-specific device control, and robot hardware support that is not part of the generic host runtime.
- **FR-005**: User-facing commands, configuration, onboarding, package contents, and documentation MUST NOT retain active board/peripheral integration surfaces after their implementation is removed.
- **FR-006**: User-facing platform documentation and build guidance MUST describe the same three supported architectures.
- **FR-007**: The cleanup MUST preserve unrelated agent runtime capabilities and must not modify pinned upstream or third-party source.
- **FR-008**: Maintainers MUST be able to trace architecture support from the documented list to CI/release artifact names and installer host detection.

### Key Entities

- **Supported host architecture**: One of the three user-visible OS/architecture combinations and its corresponding distributable artifact.
- **Board integration**: Any firmware, board-specific device control, flashing, setup, configuration, or documentation for embedded boards or robotics hardware.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All maintained CI and release target matrices contain exactly the three supported host combinations and no unsupported host-artifact targets.
- **SC-002**: Installer host detection has a defined result for all three supported hosts and rejects every other host combination without choosing a fallback artifact.
- **SC-003**: No board firmware, MCU flashing command, board setup guide, board-specific configuration surface, or robot hardware crate remains in the maintained project tree.
- **SC-004**: All current documentation links affected by the removal resolve, and the supported architecture list is consistent across release, install, and contributor guidance.
- **SC-005**: Existing runtime functionality unrelated to host-architecture and hardware scope remains present in the reviewed change.

## Assumptions

- `linux-amd64` means 64-bit x86 Linux; `linux-arm64` means 64-bit ARM Linux; `aarch64-darwin` means 64-bit ARM macOS.
- Generic PC scope excludes Android/Termux, Windows, Intel macOS, 32-bit ARM, microcontrollers, board-specific Linux GPIO, and robotics hardware.
- Linux arm64 remains a valid generic host target; Raspberry Pi- or board-specific setup and GPIO instructions are not retained as supported product features.
- Lock files remain when they support reproducibility across the supported product; transitive dependency metadata is not itself a supported-platform claim.
