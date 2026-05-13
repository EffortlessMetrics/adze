# Work Item Template

Use this template when adding a work item to `.adze/goals/active.toml`.

## Shape

```toml
[[work_item]]
id = "kebab-case-id"
status = "ready"
proposal = "docs/proposals/ADZE-PROP-NNNN-..."
spec = "docs/specs/ADZE-SPEC-NNNN-..."
adr = "docs/adr/ADZE-ADR-NNNN-..."
plan = "plans/milestone/name.md"
blocked_by = []
prs = []
commands = [
  "just ci-supported",
]
```

## Status Values

| Status | Meaning |
| --- | --- |
| `ready` | Can be started. No blockers. |
| `active` | Currently being worked on. |
| `blocked` | Cannot start until `blocked_by` items are `complete`. |
| `complete` | Done. Proof commands pass. |
| `superseded` | Replaced by a different approach. |

## Rules

- `id` must be unique within the active manifest.
- `ready` and `active` items should have `commands`.
- `blocked` items must have `blocked_by`.
- `complete` items should have `prs` or proof of completion.
- `proposal`, `spec`, `adr`, and `plan` are optional paths.
- Paths should be relative to the workspace root.
