# Codex/Droid Implementation Contract

This document defines the rules that Codex, Droid, and other autonomous agents
must follow when working in this repository.

## Read Order

Before starting work, read these files in order:

1. `CLAUDE.md` or `AGENTS.md` — coding conventions, commands, architecture.
2. `.adze/goals/active.toml` — current work item state and next steps.
3. `policy/doc-artifacts.toml` — document artifact ledger.
4. `docs/status/SUPPORT_TIERS.md` — product claim proof mapping.
5. The proposal, spec, and ADR linked to the current work item.

## PR Rules

1. **One semantic change per PR.** Do not bundle unrelated changes.
2. **Docs/spec/ADR PRs must not change runtime behavior.** Pure documentation.
3. **Required CI must be green before merge.** Do not merge failing CI.
4. **Advisory surfaces must be documented as advisory.** Do not promote
   experimental surfaces without proof commands.
5. **No new proof slices after freeze** unless release docs expose a missing
   claim.

## PR Body Requirements

Every PR body must include:

1. **Production delta** — what changes and why.
2. **Non-goals** — what is explicitly not changing.
3. **Proof commands** — exact commands that verify the change.
4. **Support-tier impact** — which SUPPORT_TIERS.md rows change.
5. **Policy impact** — which policy/*.toml files change.
6. **Rollback** — how to revert if something goes wrong.

## Support-Tier Rules

1. **Stable** surfaces must have passing proof commands in `ci-supported`.
2. **Experimental** surfaces must have passing proof commands in advisory lanes.
3. **Advisory** surfaces must be documented with known gaps.
4. Do not promote a surface without updating `SUPPORT_TIERS.md`.

## Policy-Ledger Rules

1. Every exception must appear in the appropriate `policy/*.toml`.
2. New exceptions must be proposed and reviewed before merging.
3. Identity for panic-family entries is `(path, family, selector)`.
4. Line/column drift never invalidates an entry.

## Rollback Requirements

1. Every PR must describe how to revert.
2. If a PR changes generated code, include the exact regeneration command.
3. If a PR changes policy TOML, include the previous state in the body.
4. If a PR changes `active.toml`, include the previous state in the body.

## Source-of-Truth Rail

```text
ROADMAP
  -> proposal (why)
  -> spec (what)
  -> ADR (durable decision)
  -> plan (how, PR-sized)
  -> active.toml (what now)
  -> PR (implementation)
  -> proof command (verification)
  -> support-tier / policy receipt (receipt)
  -> release closeout (done)
```

Follow this rail. Do not skip layers. Do not implement without a spec. Do not
merge without proof.
