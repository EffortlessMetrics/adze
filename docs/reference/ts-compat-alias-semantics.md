# Tree-sitter Compatibility Alias Semantics

**Status:** Initial node-identity projection implemented; broader alias parity
remains future work.

This document defines the intended alias-aware `ts_compat` node identity
contract. It complements
[`ts-compat-node-identity.md`](ts-compat-node-identity.md), which documents the
current parsed-node behavior.

Current Adze runtime state:

- alias metadata is preserved at the generated `TSLanguage` ABI boundary,
- runtime decode preserves alias sequences in native parse-table data,
- native `AdzeDocument` nodes expose separate visible and grammar identity
  slots and can mark known production aliases,
- parsed `ts_compat::Node` values project alias-visible `kind()`,
  `kind_id()`, `is_named()`, and `to_sexp()` while keeping
  `grammar_name()`/`grammar_id()` on the raw parsed symbol.

This document is the target contract for the projection layer. The current
runtime implements the node-identity and S-expression portions for known
production alias sequence entries; node-types and query-compatible alias
metadata remain future work.

## Upstream Shape

Tree-sitter distinguishes visible node identity from grammar identity.

In the Rust binding:

| Upstream API | Meaning |
|---|---|
| `Node::kind()` | Visible node type string. |
| `Node::kind_id()` | Visible node type id. |
| `Node::grammar_name()` | Grammar symbol name, ignoring aliases. |
| `Node::grammar_id()` | Grammar symbol id, ignoring aliases. |

The C API has the same split through `ts_node_type`,
`ts_node_symbol`, `ts_node_grammar_type`, and
`ts_node_grammar_symbol`.

Adze's compatibility layer should follow that split.

## Target Adze Node Identity

Alias-aware `ts_compat::Node` should carry both visible and grammar identity:

```rust
pub struct NodeIdentity {
    visible_name: SymbolName,
    visible_id: SymbolId,
    grammar_name: SymbolName,
    grammar_id: SymbolId,
    alias: Option<AliasId>,
    is_named: bool,
}
```

Projected APIs should mean:

| API | Target meaning |
|---|---|
| `Node::kind()` | Alias-visible node name when an alias applies; otherwise grammar symbol name. |
| `Node::kind_id()` | Alias-visible/public symbol id when an alias applies; otherwise grammar symbol id. |
| `Node::grammar_name()` | Original grammar symbol name, ignoring aliases. |
| `Node::grammar_id()` | Original grammar symbol id, ignoring aliases. |
| `Node::is_named()` | Alias-adjusted namedness when alias metadata changes visibility; otherwise grammar symbol namedness. |
| `Node::to_sexp()` | Renders the same visible identity as `kind()` for named nodes. |

The grammar identity APIs must remain usable for parse-state metadata and
lookahead-style APIs that need original grammar symbols.

## Alias Kinds

The target contract must handle at least these cases:

| Case | Example intent | Expected projection |
|---|---|---|
| Named alias over named symbol | grammar `expression` displayed as `binary_expression` | `kind()` is `binary_expression`; `grammar_name()` is `expression`. |
| Named alias over anonymous symbol | token `"+"` displayed as `plus_operator` | visible node may become named if alias metadata says named. |
| Anonymous alias over named symbol | grammar node hidden behind anonymous display | named-child filtering follows visible namedness. |
| No alias | normal grammar symbol | visible identity equals grammar identity. |

If a case cannot be represented by current ABI metadata, the implementation must
extend native `AdzeDocument`/tree identity data before changing `ts_compat`.

## S-Expression Contract

Alias-aware S-expressions should render visible node identity:

```text
(source_file (binary_expression left: (number) right: (number)))
```

For named-node S-expressions:

- visible named nodes are included,
- anonymous children remain omitted unless the S-expression mode explicitly
  asks for anonymous nodes,
- field labels are rendered from edge metadata,
- `grammar_name()` is not used for display unless no alias applies.

This keeps `to_sexp()` aligned with `kind()`.

## Node-Types Metadata

Alias-aware node-types output should describe visible node types because that is
what Tree-sitter tooling and queries inspect.

The output must still retain enough metadata to relate visible aliases back to
grammar symbols for diagnostics, parse-state metadata, and native provenance.

Minimum future node-types canaries:

- an aliased named node appears under its visible alias name,
- the original grammar symbol does not appear as a separate visible node type
  unless it is also visible without the alias,
- field metadata remains attached to parent-child edges,
- named/anonymous status follows visible alias metadata,
- grammar identity remains available through `grammar_name()`/`grammar_id()`.

## Native Data Requirement

The Tree-sitter adapter must not guess alias identity.

Native parse data must preserve:

- grammar symbol id,
- visible symbol id,
- visible name,
- grammar name,
- alias id or alias sequence entry,
- visible namedness,
- grammar namedness,
- edge field metadata.

If `AdzeDocument` does not expose these fields, the adapter should remain on the
current raw-symbol contract rather than inventing alias behavior locally.

## Proof Requirements

Before extending this contract, add canaries that prove the ABI, native tree,
and `ts_compat` projection agree.

Required proof slices:

```bash
cargo test -p adze --features "pure-rust,glr,ts-compat" \
  --test tablegen_abi_decode_roundtrip \
  compressed_tslanguage_decode_preserves_alias_sequences \
  -- --exact --nocapture

cargo test -p adze --features "pure-rust,ts-compat" \
  --test adze_document_alpha \
  parse_document_projects_alias_visible_identity_from_native_node_data \
  -- --exact --nocapture

cargo test -p adze --features "pure-rust,ts-compat" \
  --test ts_compat_node_metadata \
  alias_visible_kind_and_grammar_identity_are_distinct \
  -- --exact --nocapture

cargo test -p adze --features "pure-rust,ts-compat" \
  --test ts_compat_node_metadata \
  anonymous_alias_controls_named_child_filtering \
  -- --exact --nocapture

cargo test -p adze --features "pure-rust,ts-compat" \
  --test ts_compat_to_sexp \
  alias_visible_identity_is_used_in_sexp \
  -- --exact --nocapture
```

The canaries above cover current node identity, visible namedness for
named-child filtering, and S-expression projection. Future node-types and query
alias canaries should be added beside them rather than inferred from these
narrower tests.

## Compatibility Status

Remaining limitations:

- node-types output does not yet claim alias-visible parity,
- query metadata/execution does not yet claim alias-visible parity,
- imported grammar corpus parity remains future work.
