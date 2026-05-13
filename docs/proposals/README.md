# Proposals

Proposals are the product and repo-governance "why" layer. Use them when work
needs a problem statement, user or maintainer value, alternatives, success
criteria, and an evidence loop before implementation starts.

Proposals are not behavior specs and are not PR queues. A proposal can link to
many specs, ADRs, and implementation plans, but it should not duplicate their
contracts or task lists.

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

## Naming

Use stable IDs:

```text
ADZE-PROP-0001-short-kebab-title.md
```

Example:

```text
ADZE-PROP-0001-0.9-contract-convergence.md
ADZE-PROP-0002-api-foundation.md
```

## Header

Every proposal should start with:

```md
Status: proposed | accepted | implemented | superseded
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
```

## Template

```md
# ADZE-PROP-0001: Title

Status:
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
