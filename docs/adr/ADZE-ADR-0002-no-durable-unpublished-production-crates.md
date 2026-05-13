# ADZE-ADR-0002: No durable unpublished production crates

Status: accepted
Date: 2026-05-13
Owner: release/package
Linked proposal: ../proposals/ADZE-PROP-0001-0.9-contract-convergence.md
Linked specs: ../specs/ADZE-SPEC-0001-package-surface-boundary.md
Linked plan: ../../plans/0.9.0/microcrate-collapse.md

## Decision

Adze does not have a durable unpublished production crate category.

Before the next release, every production-used microcrate that is not a public
published surface must transition into an SRP submodule under the crate or
xtask/tooling owner that actually uses it.

The temporary ledger category `owner-module-migration-target` means:

```text
this crate exists only while it is being collapsed into an owner SRP submodule
or removed
```

It is not a release-state category.

## Context

The workspace grew many small support crates for governance, BDD, concurrency,
parser contracts, feature policy, source-location helpers, formatting helpers,
and runtime governance. That split helped parallel development, but it also
created a wide Cargo graph and CI surface that users do not experience as
public product value.

Keeping those crates as unpublished production surfaces would make release
claims, MSRV migration, lint policy, CI economics, and support-tier proof harder
to reason about. If code is production-relevant but not a published API, it
should live near its owner as an SRP module.

## Consequences

- Public crates remain publishable surfaces with release metadata and
  support-tier proof.
- Dev-only crates may remain separate only when they are genuinely test,
  fixture, benchmark, or automation surfaces.
- Migration targets are release blockers until they are removed, inlined, moved
  to SRP owner submodules, or explicitly reclassified by a later accepted ADR.
- Package-collapse PRs should be owner-sized and should update Cargo metadata,
  lockfile, policy ledgers, release lists, docs, and CI routing together.
- `policy/package-boundary.toml` remains the package classification ledger.

## Alternatives Considered

### Keep unpublished production crates indefinitely

Rejected. It hides production surfaces from release policy and keeps CI cost
high without a user-facing contract.

### Publish every support crate

Rejected. Most support crates are not stable public APIs and should not become
permanent crates.io surfaces just because they were useful seams during
development.

### Collapse everything in one PR

Rejected. The graph is too broad for one safe change. Collapse should proceed
by owner group or zero-reverse-dependency facade.

## Follow-Up Specs And Plans

- `../specs/ADZE-SPEC-0001-package-surface-boundary.md`
- `../../plans/0.9.0/microcrate-collapse.md`
- `../../policy/package-boundary.toml`
