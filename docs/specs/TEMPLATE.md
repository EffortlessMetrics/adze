# Spec Template

Use this template when creating a new spec in `docs/specs/`.

## Naming

```
ADZE-SPEC-NNNN-short-kebab-title.md
```

Use the next available number. Check existing specs for the current high-water mark.

## Template

```markdown
# ADZE-SPEC-NNNN: Title

Status: proposed
Owner:
Created:
Linked proposal:
Linked ADRs:
Linked plan:
Linked issues:
Linked PRs:
Support-tier impact:
Policy impact:

## Problem

What behavior or contract gap exists?

## Behavior

What must be true?

## Non-Goals

What is out of scope?

## Required Evidence

What proof is required?

## Acceptance Examples

Concrete examples of accepted and rejected behavior.

## Test Mapping

Which tests, fixtures, or snapshots cover this contract?

## Implementation Mapping

Which crates, modules, docs, or policy files own the implementation?

## CI Proof

Which commands and CI lanes prove the contract?

## Metrics And Promotion Rule

What moves this from experimental/advisory to stable?
```

## Source Of Truth

Specs own:

- behavior requirements
- non-goals
- acceptance examples
- required evidence
- implementation ownership boundaries
- test and CI proof mapping
- support-tier promotion criteria

Other artifacts own:

- why the work exists: `docs/proposals/`
- durable architecture decisions: `docs/adr/`
- PR-sized sequencing: `plans/<milestone>/`
- active agent/operator state: `.adze/goals/active.toml`
- product claim proof mapping: `docs/status/SUPPORT_TIERS.md`
- exception ledgers: `policy/*.toml`

## Duplication Rule

Specs may reference product claims in `docs/status/SUPPORT_TIERS.md`, but must
not copy the full feature-to-proof table. Specs may reference CI economics and
exceptions in `policy/*.toml`, but must not copy policy ledgers into prose.
