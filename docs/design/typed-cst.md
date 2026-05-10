# Typed CST Design Contract

**Status:** Draft design contract; not an implemented stable API.

Typed CST is the planned Rust-native syntax view over `AdzeDocument`. It sits
between the generic native CST and the semantic typed AST:

```text
AdzeDocument
  -> generic CST: dynamic, lossless tree
  -> typed CST: generated, lossless Rust syntax handles
  -> typed AST: semantic Rust values
```

Typed CST should make exact syntax ergonomic without creating a second parse
truth.

## Core Rule

Typed CST is a generated view over `AdzeDocument`, not a separate tree.

```rust
let doc = grammar::parse_document(source)?;
let syntax: syntax::SourceFile = doc.syntax()?;
```

Every typed CST wrapper must identify a node in the document:

```rust
pub struct Expression<'doc> {
    doc: &'doc AdzeDocument,
    id: NodeId,
}
```

The wrapper may provide typed methods, but it must not own child nodes or copied
syntax data.

## Why Typed CST Exists

Typed AST is the right surface for semantic users:

```rust
let ast: ast::Module = grammar::parse(source)?;
```

But many tools need concrete syntax:

- formatters,
- refactoring tools,
- codemods,
- LSP navigation,
- source-preserving transforms,
- syntax-aware agents,
- diagnostics and recovery inspection.

Those tools need spans, punctuation, comments, anonymous tokens, missing nodes,
error nodes, and field relationships. A semantic AST may intentionally drop or
normalize those details.

Typed CST keeps the syntax and makes it Rust-typed.

## Relationship To Other Views

| View | Purpose | Lossless? | Typed? |
|---|---|---:|---:|
| Generic CST | Dynamic tree inspection and serialization | Yes | No |
| Tree-sitter compatibility | Upstream-shaped tooling adapter | Mostly, per compatibility contract | No |
| Typed CST | Rust-native syntax tooling | Yes | Yes |
| Typed AST | Semantic application/compiler model | Usually no | Yes |

Typed CST must share node IDs, spans, field edges, flags, and source text with
the generic CST.

## Generated Wrapper Shape

The first typed CST implementation exposes a small shared handle trait and
should generate wrappers that implement it:

```rust
pub trait SyntaxNode<'doc>: Copy {
    fn document(&self) -> &'doc AdzeDocument;
    fn node_id(&self) -> NodeId;

    fn node(&self) -> Option<AdzeNode<'doc>>;
    fn byte_range(&self) -> Option<Range<usize>>;
    fn text(&self) -> Option<&'doc str>;
}
```

Generated node wrappers should implement the trait:

```rust
pub struct FunctionDecl<'doc> {
    doc: &'doc AdzeDocument,
    id: NodeId,
}

impl<'doc> SyntaxNode<'doc> for FunctionDecl<'doc> {
    fn document(&self) -> &'doc AdzeDocument { self.doc }
    fn node_id(&self) -> NodeId { self.id }
}
```

The helpers stay fallible because dynamic traversals, recovered syntax, and
stale handles must not pretend an invalid typed view is normal syntax.

## Field Accessors

Fields are parent-child edge metadata in `AdzeDocument`.

Typed CST accessors project those edges:

```rust
impl<'doc> BinaryExpression<'doc> {
    pub fn left(&self) -> Option<Expression<'doc>>;
    pub fn operator(&self) -> Option<OperatorToken<'doc>>;
    pub fn right(&self) -> Option<Expression<'doc>>;
}
```

The accessor must resolve through `Edge.field_id` or `Edge.field_name`, not by
guessing child positions from generated wrapper code.

Repeated fields should expose iterators:

```rust
impl<'doc> ParameterList<'doc> {
    pub fn parameters(&self) -> impl Iterator<Item = Parameter<'doc>> + 'doc;
}
```

Optional fields should return `Option<T>`. Required fields may return `T` only
after the grammar contract proves they cannot be absent in valid, unrecovered
trees. Recovered or missing syntax still needs an honest failure path.

## Token Wrappers

Typed CST should type tokens as well as nodes when tokens are part of the public
syntax API:

```rust
pub struct IdentifierToken<'doc> {
    doc: &'doc AdzeDocument,
    id: NodeId,
}

pub struct PlusToken<'doc> {
    doc: &'doc AdzeDocument,
    id: NodeId,
}
```

Typed tokens are important for formatters and codemods because punctuation and
identifiers often carry the exact source spans to edit.

## Error And Missing Syntax

Typed CST must not pretend recovered syntax is normal syntax.

Wrappers should expose structural flags:

```rust
node.is_error();
node.is_missing();
node.has_error();
```

Field accessors over recovered syntax should choose one of these shapes:

```rust
pub fn name(&self) -> Option<IdentifierToken<'doc>>;
pub fn required_name(&self) -> Result<IdentifierToken<'doc>, MissingSyntax>;
```

The first implementation should prefer `Option<T>` until the missing/error
contract is proven across generated parser entry points.

## Typed AST Provenance

Typed AST extraction should eventually record provenance from typed CST, but the
relationship is not always one-to-one.

```rust
pub enum Provenance {
    Node(NodeId),
    Span(ByteRange),
    Nodes(Vec<NodeId>),
    Synthetic {
        span: ByteRange,
        reason: SyntheticReason,
    },
}
```

This avoids pretending every semantic AST value maps to exactly one CST node.

## Generation Scope

The alpha generator should produce:

- node wrapper types,
- token wrapper types for public syntax tokens,
- `SyntaxNode` implementations,
- typed field accessors,
- `span()` and `text()` helpers,
- simple node kind validation for wrapper construction.

The alpha generator should not produce:

- visitors,
- rewriters,
- typed query APIs,
- trivia classification APIs,
- typed CST JSON,
- edit builders,
- formatting helpers.

Those are useful generated surfaces, but each needs a separate contract and
proof lane.

## Construction

Typed CST wrappers should be constructed from validated node IDs:

```rust
impl<'doc> Expression<'doc> {
    pub fn cast(doc: &'doc AdzeDocument, id: NodeId) -> Option<Self> {
        doc.tree().node(id).kind().is_expression().then_some(Self { doc, id })
    }
}
```

Generated code should avoid panicking on kind mismatches. A failed cast is a
normal result when a dynamic traversal reaches a different syntax kind.

## Serialization

Typed CST JSON is a future surface, not part of the alpha contract.

When it exists, it must be schema-versioned:

```json
{ "schema": "adze.typed-cst.v1" }
```

Typed CST JSON should serialize a typed projection of the same `AdzeDocument`
node IDs, spans, fields, and flags. It must not re-run parsing or invent a
separate tree.

## Proof Requirements

The first typed CST proof should use one small grammar, such as arithmetic.

Minimum canary shape:

```rust
let doc = grammar::parse_document("1 + 2")?;
let syntax: syntax::SourceFile = doc.syntax()?;
let expr = syntax.expression().unwrap();

assert_eq!(expr.left().unwrap().text(), "1");
assert_eq!(expr.operator().unwrap().text(), "+");
assert_eq!(expr.right().unwrap().text(), "2");
assert_eq!(expr.node_id(), doc.tree().root().child_by_field("expression").unwrap());
```

The proof must show:

- typed wrappers reference `AdzeDocument` node IDs,
- field accessors read native edge metadata,
- spans and text come from the document source,
- optional/repeated accessors behave deterministically,
- missing/error syntax has an honest representation.

## Current Alpha Proof

The current alpha proof is intentionally test-local:

```bash
cargo test -p adze --features "pure-rust,ts-compat" \
  --test typed_cst_arithmetic_spike -- --nocapture
```

It proves arithmetic typed CST handles can reference `AdzeDocument` node IDs,
implement the runtime `SyntaxNode` handle contract, resolve field accessors
through native edge metadata, read spans and text from the document source, and
surface recovery/error flags without panicking.

This is not yet a generated public typed CST wrapper API.

## Support Status

This document does not promote typed CST to a supported surface. Typed CST
remains future work until implementation canaries and support-tier entries land.

Expected sequence:

1. `AdzeDocument` minimal alpha with document-local `NodeId` and edge lookup,
2. test-local typed CST arithmetic spike,
3. runtime `SyntaxNode` handle helpers,
4. generated wrappers and field accessors,
5. typed CST and generic CST parity canaries,
6. typed AST extraction through typed CST,
7. typed CST JSON, if needed.
