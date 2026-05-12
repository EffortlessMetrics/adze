# Active Goals

This directory is for machine-readable execution state used by Droid, Codex, and
other automation. It answers "what is being worked now?" rather than "why does
this work exist?" or "what behavior is required?"

Use:

```text
active.toml
archive/
```

Do not put long prose plans here. Link to proposals, specs, ADRs, plans, issues,
and PRs.

## Source Of Truth

Goal manifests own:

- current campaign ID and title
- active/paused/complete status
- current owner or execution lane
- end-state checklist
- work item state
- commands automation should run
- links to specs, plans, issues, and PRs

Other artifacts own:

- why the campaign exists: `../../docs/proposals/`
- behavior contracts: `../../docs/specs/`
- durable architecture decisions: `../../docs/adr/`
- PR sequencing and proof rationale: `../../plans/<milestone>/`
- product claim proof mapping: `../../docs/status/SUPPORT_TIERS.md`
- policy ledgers: `../../policy/*.toml`

## `active.toml` Shape

```toml
id = "adze-0-9-contract-convergence"
title = "Adze 0.9.0 contract convergence"
status = "active"
owner = "droid-factory"
created = "2026-05-11"

objective = """
Collapse the workspace surface, update Rust/MSRV/lints, recalibrate CI
economics, and promote product claims only where support-tier proof exists.
"""

end_state = [
  "No production workspace crate is unclassified.",
  "No unpublished production crate exists.",
  "CI lane whitelist reflects the post-collapse workspace.",
  "Support tiers map every README stable claim to proof.",
]

[[work_item]]
id = "package-boundary-audit"
status = "ready"
spec = "ADZE-SPEC-0001-package-surface-boundary"
plan = "plans/0.9.0/microcrate-collapse.md"
commands = [
  "cargo metadata --format-version 1 --no-deps",
  "cargo run -q -p xtask -- check-package-boundary",
  "just ci-supported",
]
```

## Status Values

Use a small status vocabulary:

```text
ready
active
blocked
complete
superseded
```

Archive completed manifests under `archive/` with a dated filename.
