# ADZE-ADR-0004: Serialized projections are schema-versioned

Status: accepted
Date: 2026-05-13
Owner: cli/wasm/schema
Linked proposal: ../proposals/ADZE-PROP-0002-api-foundation.md
Linked specs: ../specs/ADZE-SPEC-0008-json-cli-wasm-projections.md

## Decision

All JSON, CLI, and WASM serialized document projections carry explicit schema
family and version.

## Context

Serialized outputs are consumed outside Rust semver. A CLI JSON consumer, WASM
consumer, or agent pipeline needs a stable envelope and migration path. Rust
crate semver alone is not enough to describe serialized data compatibility.

Adze also has several distinct version concepts:

- Rust API semver;
- document schema version;
- grammar fingerprint;
- Tree-sitter ABI compatibility.

These must not be collapsed into one implicit version.

## Consequences

- Document JSON uses schema names such as `adze.document.v1`.
- Tree JSON, diagnostics JSON, typed CST JSON, ambiguity JSON, forest JSON, and
  node-types JSON each have explicit schema families when exposed.
- CLI and WASM serialize the same schema families.
- Stable serialized claims require schema snapshots and proof commands.

## Alternatives Considered

### Unversioned JSON

Rejected. It is easy to produce but brittle for downstream tools.

### Tie JSON compatibility only to crate semver

Rejected. Rust API compatibility and serialized output compatibility change at
different rates.

## Follow-Up Specs / Plans

- `../specs/ADZE-SPEC-0008-json-cli-wasm-projections.md`
- `../../plans/0.9.0/api-foundation.md`
