# Architecture Decision Records

ADRs record durable architecture decisions. Use an ADR when a choice should keep
guiding future work after the immediate PR sequence is complete.

Do not use ADRs for ordinary tasks, status updates, or temporary execution
state. Those belong in implementation plans, handoffs, or active goal manifests.

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

## Naming

The repository currently has both legacy ADR names and newer `ADZE-ADR-*`
names. New ADRs should use:

```text
ADZE-ADR-0001-short-kebab-title.md
```

Example:

```text
ADZE-ADR-0001-adze-document-one-parse-truth.md
ADZE-ADR-0002-no-durable-unpublished-production-crates.md
ADZE-ADR-0003-summary-first-glr-ambiguity.md
ADZE-ADR-0004-schema-versioned-projections.md
```

## Header

Every ADR should start with:

```md
Status: proposed | accepted | superseded
Date:
Owner:
Linked proposal:
Linked specs:
Linked plan:
```

## Template

```md
# ADZE-ADR-0001: Title

Status:
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

## Alternatives Considered

What did we reject?

## Follow-Up Specs And Plans

What must be specified or implemented next?
```
