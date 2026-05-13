# Plan Template

Use this template when creating an implementation plan in `plans/<milestone>/`.

## Naming

```
plans/<milestone>/<descriptive-name>.md
```

Example:

```
plans/0.9.0/microcrate-collapse.md
```

## Template

```markdown
# Plan: <descriptive name>

Milestone:
Owner:
Created:
Linked proposal:
Linked specs:
Linked ADRs:
Status: draft | active | complete | superseded

## Objective

What this plan achieves in one sentence.

## Prerequisites

What must be true before starting.

## Steps

### Step 1: <title>

- Scope:
- Proof:
- Rollback:

### Step 2: <title>

- Scope:
- Proof:
- Rollback:

## Proof Commands

```bash
# All steps must pass these
just ci-supported
```

## Support-Tier Impact

Which SUPPORT_TIERS.md rows change.

## Policy Impact

Which policy/*.toml files change.

## Rollback

How to revert if something goes wrong.
```

## Source Of Truth

Plans own:

- PR-sized sequencing
- per-step scope, proof, and rollback
- dependency ordering

Other artifacts own:

- why the work exists: `docs/proposals/`
- behavior contracts: `docs/specs/`
- durable architecture decisions: `docs/adr/`
- current execution state: `.adze/goals/active.toml`
