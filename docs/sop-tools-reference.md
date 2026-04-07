# SOP Tools Reference

Standard Operating Procedure (SOP) tools for managing and executing structured workflows.

Last verified: **April 8, 2026**.

## Overview

SOPs are predefined workflows that agents can execute to accomplish complex, multi-step tasks consistently. The SOP system provides tools for listing, executing, monitoring, and controlling SOP runs.

## SOP Tools

### `sop_list`

List all loaded Standard Operating Procedures with their triggers, priority, step count, and active runs.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `filter` | string | No | Filter SOPs by name substring or priority (`low`/`normal`/`high`/`critical`) |

**Example:**

```json
{
  "filter": "backup"
}
```

**Returns:**

- SOP name, trigger patterns, priority level
- Step count for each SOP
- Number of active runs
- Filtered view when filter parameter is provided

---

### `sop_execute`

Manually trigger an SOP by name. Returns the run ID and first step instruction.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | Yes | Name of the SOP to execute |
| `payload` | string | No | Optional trigger payload as JSON string |

**Example:**

```json
{
  "name": "daily_backup",
  "payload": "{\"target\": \"/data\"}"
}
```

**Returns:**

- Run ID for tracking
- First step instruction
- Execution status

---

### `sop_status`

Check the current status of an SOP run by its ID.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `run_id` | string | Yes | The run ID returned by `sop_execute` |

**Example:**

```json
{
  "run_id": "sop-run-20260408-abc123"
}
```

**Returns:**

- Current step number and total steps
- Step status (`pending`, `in_progress`, `completed`, `failed`)
- Step instruction and verification criteria
- Run-level status and result

---

### `sop_advance`

Advance an SOP run to the next step after completing the current step.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `run_id` | string | Yes | The run ID to advance |
| `step_result` | string | No | Result of the current step execution (`success`, `failure`, `ambiguous`) |
| `notes` | string | No | Optional notes about step execution |

**Example:**

```json
{
  "run_id": "sop-run-20260408-abc123",
  "step_result": "success",
  "notes": "Backup completed successfully, 1.2GB transferred"
}
```

**Returns:**

- Next step instruction (if more steps exist)
- Completion status (if all steps finished)
- Updated run state

---

### `sop_approve`

Approve a pending SOP step that requires confirmation before proceeding.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `run_id` | string | Yes | The run ID containing the pending step |
| `step_number` | integer | No | Specific step to approve (defaults to current pending step) |
| `approval_token` | string | No | OTP/token if required by policy |

**Example:**

```json
{
  "run_id": "sop-run-20260408-abc123",
  "step_number": 3
}
```

**Returns:**

- Approval confirmation
- Next step instruction
- Updated run state

---

## SOP Priority Levels

| Level | Description |
|-------|-------------|
| `low` | Background tasks, cleanup operations |
| `normal` | Standard workflows, routine operations |
| `high` | Important tasks requiring attention |
| `critical` | Urgent workflows, may interrupt other operations |

## SOP Run States

| State | Description |
|-------|-------------|
| `pending` | Run created, waiting to start |
| `running` | Currently executing a step |
| `awaiting_approval` | Step requires manual approval |
| `paused` | Temporarily suspended |
| `completed` | All steps finished successfully |
| `failed` | Step failure or run aborted |

## Integration with Agent System

SOPs integrate with the agent system through:

- **Natural language triggers**: SOPs can be triggered by message patterns
- **Tool delegation**: Agent can delegate to SOPs for structured execution
- **Memory integration**: SOP results stored in `episode` memory category
- **Audit logging**: All SOP runs logged when audit is configured

## Related Documentation

- [commands-reference.md](commands-reference.md) - CLI commands
- [config-reference.md](config-reference.md) - SOP configuration
- [docs/sop/](sop/) - SOP authoring guides
