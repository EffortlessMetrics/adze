# Tree-sitter Compatibility Node Identity

This document defines the current `adze::ts_compat` node identity contract.
It is intentionally narrower than full Tree-sitter alias parity. The current
runtime preserves alias metadata at the generated `TSLanguage` ABI boundary,
but parsed `ts_compat::Node` values do not yet carry alias-specific identity.

## Current Contract

For parsed `ts_compat` trees today:

| API | Current meaning |
|---|---|
| `Node::kind()` | Name of the parsed symbol stored on the node. |
| `Node::kind_id()` | Raw parsed symbol id stored on the node. |
| `Node::grammar_name()` | Same as `kind()` because nodes do not yet carry alias identity. |
| `Node::grammar_id()` | Same as `kind_id()` because nodes do not yet carry alias identity. |
| `Node::to_sexp()` | Tree-sitter-style named-node S-expression rendered from `kind()` plus field labels on named children. |

This means the current grammar identity APIs are stable as raw parsed-symbol
metadata, not as alias-visible Tree-sitter display metadata.

## Alias Boundary

Alias data is already part of the generated ABI and runtime decode path:

- generated `TSLanguage` values can expose `alias_map` and `alias_sequences`,
- runtime decode preserves alias sequences in `ParseTable`,
- tablegen/runtime canaries prove the alias ABI data survives individual and
  combined metadata roundtrips.

That preservation is necessary for future alias-aware tree projection, but it
does not by itself change `Node::kind()`, `Node::kind_id()`,
`Node::grammar_name()`, `Node::grammar_id()`, or `Node::to_sexp()`.

## Future Alias-Aware Contract

Before Adze claims broader Tree-sitter alias parity, a follow-up design and
canary set must define all of these explicitly:

- whether `kind()` returns the alias-visible name or the grammar symbol name,
- whether `kind_id()` returns an alias/public id or the grammar symbol id,
- whether `grammar_name()` always returns the original grammar symbol name,
- whether `grammar_id()` always returns the original grammar symbol id,
- how aliases appear in `to_sexp()`,
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
cargo test -p adze --features "pure-rust,ts-compat" --test ts_compat_to_sexp -- --nocapture
cargo test -p adze --features "pure-rust,glr,ts-compat" --test tablegen_abi_decode_roundtrip compressed_tslanguage_decode_preserves_alias_sequences -- --exact --nocapture
cargo test -p adze --features "pure-rust,glr,ts-compat" --test tablegen_abi_decode_roundtrip combined_tslanguage_decode_preserves_metadata_fields_aliases_externals_and_lex_modes -- --exact --nocapture
```
