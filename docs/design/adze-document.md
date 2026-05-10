# AdzeDocument Design Contract

**Status:** Draft design contract; not an implemented stable API.

`AdzeDocument` is the planned native parse-product boundary for Adze. It is
the source of truth that future Tree-sitter-compatible output, typed CST,
typed AST extraction, diagnostics, and GLR ambiguity views should project from.

The design goal is one parse truth with multiple views, not parallel trees that
can drift.

```text
source
  -> Adze parser/runtime
  -> AdzeDocument
       -> generic CST
       -> typed CST
       -> typed AST
       -> Tree-sitter-compatible Tree/Node/Cursor
       -> diagnostics
       -> GLR ambiguity summaries / forest data
```

## Core Rule

`AdzeDocument` must be monomorphic.

```rust
pub struct AdzeDocument {
    source: SourceText,
    tree: AdzeTree,
    diagnostics: Vec<ParseDiagnostic>,
    ambiguities: AmbiguitySet,
    metadata: ParseMetadata,
}
```

Typed ASTs and typed CSTs are projections:

```rust
let doc = grammar::parse_document(source)?;

let tree = doc.tree();
let syntax: syntax::SourceFile = doc.syntax()?;
let ast: ast::Module = doc.ast()?;
let ts_tree = doc.as_tree_sitter();
```

They are not fields on the document and must not define separate parse truths.

## Document Versus Failure Semantics

`parse_document` should distinguish parse facts from infrastructure failures.

Syntax errors, recovery, and ambiguity should generally produce an
`AdzeDocument` with diagnostics, node flags, and metadata:

```rust
let doc = grammar::parse_document("1 +")?;
assert!(!doc.diagnostics().is_empty());
```

Hard failures are reserved for cases where no trustworthy document can be
produced:

```rust
pub enum ParseFailure {
    NoLanguage,
    Cancelled,
    InternalInvariant,
}
```

This keeps native parsing useful for editors, LSPs, formatters, and agents that
must inspect incomplete source text. Tree-sitter-compatible projections can
render the same state as `is_error()`, `has_error()`, and `is_missing()`, while
native APIs expose the structured diagnostics that explain what happened.

## Native Tree Model

The generic native CST should be lossless enough to support formatting,
refactoring, diagnostics, Tree-sitter-compatible projection, and typed CST
wrappers.

```rust
pub struct AdzeTree {
    root: NodeId,
    nodes: NodeArena,
    language: LanguageMetadata,
}

pub struct AdzeNode {
    id: NodeId,
    kind: NodeKind,
    span: ByteRange,
    point_range: PointRange,
    parent: Option<NodeId>,
    children: Vec<Edge>,
    production_id: Option<ProductionId>,
    rule_id: Option<RuleId>,
    flags: NodeFlags,
}
```

Fields are edge metadata:

```rust
pub struct Edge {
    child: NodeId,
    field_id: Option<FieldId>,
    field_name: Option<Arc<str>>,
}
```

A child is the `left` child of a particular parent; it is not globally `left`.
This is required for both Tree-sitter-compatible field APIs and generated typed
CST accessors.

## Projections

### Simple Typed AST

The existing simple API remains the front door for users who only want typed
semantic values:

```rust
let ast: Module = grammar::parse(source)?;
```

Long term, this should be equivalent to:

```rust
let ast: Module = grammar::parse_document(source)?.ast()?;
```

### Typed CST

Typed CST is a future generated view over `AdzeTree`, not a second tree.

Typed CST wrappers should be cheap handles:

```rust
pub struct FunctionDecl<'doc> {
    doc: &'doc AdzeDocument,
    id: NodeId,
}
```

The first implementation should stay narrow: generated node wrappers, field
accessors, token wrappers where needed, span access, and text access. Visitors,
rewriters, typed queries, trivia classification, and JSON output are later
surfaces that require separate proof.

### Tree-sitter Compatibility

Tree-sitter compatibility is a conformance adapter over the native document.

```rust
let ts_tree = doc.as_tree_sitter();
let root = ts_tree.root_node();
```

The adapter must not invent missing semantics locally. If a Tree-sitter method
cannot be implemented from `AdzeDocument` data, the native document model is
missing required information.

Examples:

| Tree-sitter-compatible API | Native invariant |
|---|---|
| `Node::child(i)` | Stable child edges exist. |
| `Node::field_name_for_child(i)` | Field names live on edges. |
| `Node::child_by_field_id(id)` | Public field IDs translate from edge metadata. |
| `Node::kind()` | Visible node identity exists. |
| `Node::grammar_name()` | Original grammar identity exists. |
| `Node::is_error()` | Node-local error flags exist. |
| `Node::has_error()` | Diagnostics or recovery state propagate through the tree. |
| `Node::is_missing()` | Recovery can represent zero-width inserted structure. |
| `Node::to_sexp()` | Tree shape and field labels are serializable. |

### Diagnostics

Tree-sitter-compatible output exposes structural flags. Native Adze output must
also expose diagnostic data.

```rust
pub struct ParseDiagnostic {
    span: ByteRange,
    point_range: PointRange,
    expected: Vec<ExpectedSymbol>,
    found: Option<FoundSymbol>,
    recovery: Option<RecoveryAction>,
    related_nodes: Vec<NodeId>,
}
```

Text rendering is a view over this data, not the canonical representation.

### GLR Ambiguity

Tree-sitter-compatible output should expose one selected tree. Native Adze
output should expose ambiguity summaries first and raw forest internals only
after the summary contract is proven.

```rust
pub struct Ambiguity {
    span: ByteRange,
    alternatives: Vec<AlternativeSummary>,
    selected: Option<AlternativeId>,
    selection_reason: SelectionReason,
}
```

Default parsing should not eagerly collect expensive forest or trace data unless
the user opts in through explicit parse options.

## Parse Options

`parse_document` should leave room for staged cost:

```rust
pub struct ParseOptions {
    recover: bool,
    collect_diagnostics: bool,
    collect_ambiguities: bool,
    collect_forest: bool,
    collect_trace: bool,
}
```

The common path should be cheap: parse source, retain the generic CST, expose
diagnostics and metadata, and compute richer projections lazily.

## Serialized Outputs

Any future native JSON output must be schema-versioned.

Examples:

```json
{ "schema": "adze.document.v1" }
```

Planned schema families include:

- `adze.document.v1`
- `adze.tree.v1`
- `adze.diagnostics.v1`
- `adze.typed-cst.v1`
- `adze.forest.v1`

No JSON schema should be treated as stable until it has a fixture, snapshot, and
support-tier entry.

## Non-Goals For The First Implementation

The first implementation must not attempt all projections at once.

Out of scope for the alpha document:

- full typed CST generation,
- typed CST visitors or rewriters,
- full Tree-sitter query execution,
- raw GLR forest export,
- typed extraction from ambiguity alternatives,
- stable `adze-json`,
- WASM document bindings,
- support-tier promotion.

The first useful slice is:

```text
AdzeDocument
  -> tree()
  -> source_slice()
  -> NodeId lookup
  -> edge and parent lookup
  -> language()
  -> diagnostics()
  -> metadata()
  -> as_tree_sitter()
```

## Proof Requirements

Before any part of this surface is promoted beyond draft/advisory, it needs a
small contract test and a product proof command.

Minimum proof map:

| Surface | Required proof |
|---|---|
| Generic CST | Root, child edges, fields, spans, and flags are populated from one parse. |
| Tree-sitter projection | `ts_compat` methods read native data, not local guesses. |
| Typed CST | Generated wrappers access the same node IDs and edge fields as the generic CST. |
| Typed AST | Extraction walks the same document and records honest provenance. |
| Diagnostics | Structured diagnostics map to source spans and related nodes. |
| Ambiguity | Selected-tree summaries record alternatives and selection reasons. |
| JSON output | Schema snapshots include explicit version strings. |

Passing tests are not enough by themselves; each test must cover the stated
contract rather than a proxy.

## Support Status

This document does not change Adze support tiers. It records the intended native
API direction so implementation PRs can stay small and reviewable:

1. contract first,
2. minimal document alpha,
3. Tree-sitter projection over the document,
4. typed CST spike,
5. typed AST provenance,
6. GLR ambiguity summaries,
7. schema-versioned CLI/WASM outputs.
