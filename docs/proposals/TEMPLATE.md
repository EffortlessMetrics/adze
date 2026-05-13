# Proposal Template

Use this template when creating a new proposal in `docs/proposals/`.

## Naming

```
ADZE-PROP-NNNN-short-kebab-title.md
```

Use the next available number. Check existing proposals for the current high-water mark.

## Template

```markdown
# ADZE-PROP-NNNN: Title

Status: proposed
Owner:
Created:
Target milestone:
Linked specs:
Linked ADRs:
Linked plan:
Linked issues:
Linked PRs:
Support-tier impact:
Policy impact:

## Problem

What risk, user pain, or product gap exists?

## Users And Surfaces

Who benefits and which repo/product surfaces are affected?

## Success Criteria

What must be true when this proposal is done?

## Proposed Shape

What direction are we taking?

## Alternatives Considered

What did we reject and why?

## Specs To Create Or Update

Which `ADZE-SPEC-*` documents define the behavior?

## Architecture Decisions Needed

Which `ADZE-ADR-*` records are required?

## Implementation Campaign Shape

What are the major PR-sized phases?

## Evidence Plan

Which proof commands, fixtures, support-tier updates, or policy receipts will
show the proposal worked?

## Risks

What can go wrong?

## Non-Goals

What is explicitly out of scope?

## Exit Criteria

When is this proposal complete?
```

## Source Of Truth

Proposals own:

- the problem or opportunity
- affected users and repo surfaces
- success criteria
- alternatives considered
- risks and non-goals
- the evidence plan at a product level

Other artifacts own:

- behavior contracts: `docs/specs/`
- durable architecture decisions: `docs/adr/`
- PR sequencing and proof commands: `plans/<milestone>/`
- current agent/operator state: `.adze/goals/active.toml`
- product claim proof mapping: `docs/status/SUPPORT_TIERS.md`
- policy exceptions and CI ledgers: `policy/*.toml`
