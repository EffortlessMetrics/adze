# ADZE-ADR-0003: GLR ambiguity is summary-first

Status: accepted
Date: 2026-05-13
Owner: runtime/glr
Linked proposal: ../proposals/ADZE-PROP-0002-api-foundation.md
Linked specs: ../specs/ADZE-SPEC-0007-glr-ambiguity-summary.md

## Decision

Adze exposes a selected tree and ambiguity summaries by default. Full GLR
forest data is opt-in and experimental until separately specified and proven.

## Context

Tree-sitter-compatible APIs expose one selected tree. Adze can expose more
because GLR ambiguity is meaningful parser information, but raw forest data is
expensive, complex, and hard to stabilize as a first product API.

Users usually need:

- where ambiguity occurred;
- what alternatives existed at a summary level;
- which alternative was selected;
- why selection happened.

They do not usually need raw forest internals on the common path.

## Consequences

- `doc.tree()` returns the selected tree.
- `doc.ambiguities()` exposes native ambiguity summaries.
- `doc.forest()` remains absent, opt-in, feature-gated, or experimental until a
  separate forest contract exists.
- typed AST lowering uses the selected tree by default.
- Tree-sitter compatibility sees only the selected tree.

## Alternatives Considered

### Selected tree only

Rejected. It hides useful parser truth and weakens Adze's GLR-native
differentiator.

### Full forest by default

Rejected. It is too expensive and too unstable for ordinary parse results.

### Summary forever, no forest

Rejected. Advanced tooling may eventually need forest access, but that should be
separately specified.

## Follow-Up Specs / Plans

- `../specs/ADZE-SPEC-0007-glr-ambiguity-summary.md`
- `../../plans/0.9.0/api-foundation.md`
