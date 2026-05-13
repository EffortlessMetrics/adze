# ADR Template

Use this template when creating a new ADR in `docs/adr/`.

## Naming

```
ADZE-ADR-NNNN-short-kebab-title.md
```

Use the next available number. Check existing ADRs for the current high-water mark.

## Template

```markdown
# ADZE-ADR-NNNN: Title

Status: proposed
Date:
Owner:
Linked proposal:
Linked specs:
Linked plan:

## Decision

State the durable architecture decision.

## Context

Why this decision exists.

## Consequences

What this enables and constrains.

### Enabled

What new possibilities does this create?

### Constrained

What does this prevent or limit?

### Costs

What ongoing maintenance or complexity does this introduce?

## Alternatives Considered

What did we reject? Include a brief reason for each.

## Follow-Up Specs And Plans

What must be specified or implemented next?
```

## Source Of Truth

ADRs own:

- the durable decision
- context that made the decision necessary
- consequences and constraints
- rejected alternatives
- follow-up specs and implementation plans

Other artifacts own:

- product or repo motivation: `docs/proposals/`
- behavior contracts: `docs/specs/`
- implementation sequencing: `plans/<milestone>/`
- current agent/operator state: `.adze/goals/active.toml`
- product claim proof mapping: `docs/status/SUPPORT_TIERS.md`
