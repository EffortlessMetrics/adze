# Tree-sitter Compatibility Node Identity

This document defines the current `adze::ts_compat` node identity contract.
It is intentionally narrower than full Tree-sitter alias and query parity. The
current runtime preserves alias metadata at the generated `TSLanguage` ABI
boundary, carries production alias entries into native document nodes, and
projects alias-visible identity through parsed `ts_compat::Node` values.

## Current Contract

For parsed `ts_compat` trees today:

| API | Current meaning |
|---|---|
| `Node::kind()` | Alias-visible node name when a production alias applies; otherwise the parsed grammar symbol name. |
| `Node::kind_id()` | Alias-visible symbol id when a production alias applies; otherwise the parsed grammar symbol id. |
| `Node::grammar_name()` | Parsed grammar symbol name, ignoring aliases. |
| `Node::grammar_id()` | Parsed grammar symbol id, ignoring aliases. |
| `Node::to_sexp()` | Tree-sitter-style named-node S-expression rendered from alias-visible `kind()` plus field labels on named children. |

This means grammar identity remains the raw parsed-symbol contract, while
visible identity follows known alias sequence entries.

## Alias Boundary

Alias data is already part of the generated ABI and runtime decode path:

- generated `TSLanguage` values can expose `alias_map` and `alias_sequences`,
- runtime decode preserves alias sequences in `ParseTable`,
- native `AdzeDocument` nodes expose separate visible and grammar identity
  slots and mark `has_alias()` when production alias metadata changes visible
  identity,
- tablegen/runtime canaries prove the alias ABI data survives individual and
  combined metadata roundtrips.

That preservation now feeds the parsed tree projection for known alias sequence
entries. Broader node-types and query compatibility still require additional
canaries.

## Remaining Alias Work

Before Adze claims broader Tree-sitter alias parity, follow-up canaries must
still cover:

- how aliases appear in node-types metadata,
- how anonymous aliases affect named-child filtering.

The draft target contract is
[`ts-compat-alias-semantics.md`](ts-compat-alias-semantics.md).

Until that work lands, code that needs alias-aware Tree-sitter display behavior
should treat the `ts_compat` alias surface as advisory.

## Proof Surface

Current canaries that guard this contract include:

```bash
cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_node_metadata -- --nocapture
cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_node_metadata alias_visible_kind_and_grammar_identity_are_distinct -- --exact --nocapture
cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_to_sexp -- --nocapture
cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_to_sexp alias_visible_identity_is_used_in_sexp -- --exact --nocapture
cargo test -p adze --features "pure-rust,ts-compat" --test adze_document_alpha parse_document_projects_alias_visible_identity_from_native_node_data -- --exact --nocapture
cargo test -p adze --features "pure-rust,glr,ts-compat" --test tablegen_abi_decode_roundtrip compressed_tslanguage_decode_preserves_alias_sequences -- --exact --nocapture
cargo test -p adze --features "pure-rust,glr,ts-compat" --test tablegen_abi_decode_roundtrip combined_tslanguage_decode_preserves_metadata_fields_aliases_externals_and_lex_modes -- --exact --nocapture
```
