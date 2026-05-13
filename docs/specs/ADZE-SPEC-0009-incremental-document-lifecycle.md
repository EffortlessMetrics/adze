# ADZE-SPEC-0009: Incremental document lifecycle

Status: proposed
Owner: Adze maintainers
Created: 2026-05-13
Linked proposal: ../proposals/ADZE-PROP-0002-api-foundation.md
Linked ADRs: ADZE-ADR-0001 AdzeDocument one parse truth
Linked plan:
Linked issues:
Linked PRs:
Support-tier impact: experimental (intentionally deferred)
Policy impact: none

## Problem

Incremental parsing reuses subtrees from a previous parse to avoid reparsing
the entire input. This is critical for editor responsiveness. The incremental
lifecycle must be defined in terms of `AdzeDocument` — not as a separate
engine — so that reused subtrees produce the same document facts as fresh
parses.

## Behavior

### Proposed, not accepted

This spec describes the intended lifecycle. It remains `proposed` until
implementation proof exists. No 0.9 claim depends on it.

### Document edit model

An edit describes a change to the source text:

```rust
pub struct InputEdit {
    pub old_byte_range: ByteRange,
    pub new_byte_range: ByteRange,
    pub old_point_range: PointRange,
    pub new_point_range: PointRange,
}
```

### Tree reuse

When a new parse is requested with an existing document and an edit, the parser
should reuse unchanged subtrees from the previous document. Reused subtrees
must produce identical document facts (node identity, fields, flags, ranges).

### Fresh parse fallback

If incremental reuse is not possible or not correct, the parser must fall back
to a fresh parse. The resulting document must be indistinguishable from one
produced by `parse_document()` on the new source.

### Document identity across edits

Node IDs are document-local and do not persist across parses. A new document
from an incremental parse has fresh node IDs. Reuse is an optimization, not a
contract about ID stability.

### GLR fork-aware reuse

GLR incremental parsing must handle fork-aware subtree reuse. Reused subtrees
in one fork branch may not be valid in another.

## Non-Goals

- Stable incremental parsing API.
- Node ID stability across edits.
- Partial document update (always produce a complete document).
- Integration with external editor APIs.
- Performance guarantees for incremental parses.

## Required Evidence

None required for 0.9. This spec is proposed only.

When implementation begins:

- Incremental parse produces the same document as fresh parse for identical
  input.
- Reused subtrees have identical identity, fields, flags, and ranges.
- Edit model handles insertions, deletions, and replacements.

## Acceptance Examples

### Incremental parse matches fresh parse

```rust
let doc1 = grammar::parse_document("1 + 2")?;
let edit = InputEdit { /* change "2" to "3" */ };
let doc2 = grammar::parse_document_incremental(&doc1, "1 + 3", &edit)?;
let doc_fresh = grammar::parse_document("1 + 3")?;
// Document facts should be identical (different node IDs allowed)
```

## Test Mapping

- `cargo test --workspace --features incremental_glr`

## Implementation Mapping

| Component | Owner |
| --- | --- |
| Incremental parsing | `runtime/src/pure_incremental.rs` |
| GLR incremental | `runtime/src/glr_incremental.rs` |
| `InputEdit` | `runtime/src/document.rs` |

## CI Proof

```bash
cargo test --workspace --features incremental_glr -- --nocapture
```

## Metrics And Promotion Rule

This spec has no promotion timeline. It remains proposed until:

- Incremental parsing no longer falls back to fresh parsing by default.
- GLR fork-aware reuse is proven.
- Performance benchmarks show measurable improvement over fresh parses.
