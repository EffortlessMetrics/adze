# No-panic policy

> **Definition.** *Panic-free* in adze means: no **unreceipted** panic-family
> behavior in production or test code.

## What counts as panic-family

| Family                          | Detected shape                                     |
| ------------------------------- | -------------------------------------------------- |
| `unwrap`                        | `expr.unwrap()`                                    |
| `expect`                        | `expr.expect("...")`                               |
| `panic_macro`                   | `panic!(...)`                                      |
| `todo`                          | `todo!(...)`                                       |
| `unimplemented`                 | `unimplemented!(...)`                              |
| `unreachable`                   | `unreachable!(...)`                                |
| `indexing`                      | `expr[idx]` (non-string-literal target)            |
| `string_slice`                  | `&s[a..b]` against a string-typed receiver         |
| `get_unwrap`                    | `.get(idx).unwrap()`                               |
| `unchecked_time_subtraction`    | `Duration` subtraction without `checked_sub`       |

Test assertion macros (`assert_eq!`, `assert!`, etc.) are **not** in scope
for v1. Migrating tests to fallible-assert helpers is a separate piece of
work and lives in [POLICY_ALLOWLISTS.md](./POLICY_ALLOWLISTS.md#fallible-test-helpers).

## Authority

The semantic checker `cargo xtask check-no-panic-family` is the authoritative
gate. Clippy is a fast local detector but does not own the receipt.

## Receipt format

Every intentional exception is a TOML record in
`policy/no-panic-allowlist.toml`:

```toml
[[allow]]
id = "panic-0001"
path = "glr-core/src/parser/state.rs"
family = "unwrap"
classification = "test_helper"          # production | test_helper | placeholder | fixture
owner = "glr-core"
explanation = "Fixture loader; migrate to fallible builder."
expires = "2026-09-01"

[allow.selector]
kind = "method_call"
container = "load_fixture_table"
callee = "unwrap"
receiver_fingerprint = "std::fs::read_to_string(path)"

[allow.last_seen]
line = 42
column = 17
```

### Identity

```
identity = path + family + selector
```

`last_seen` is a *drift hint* only — it is **not** the matching key.
Moving the call elsewhere in the same function does not invalidate the
receipt; renaming the function or changing the receiver does.

### Selectors

| Kind            | Required keys                                    |
| --------------- | ------------------------------------------------ |
| `method_call`   | `container`, `callee`, `receiver_fingerprint`    |
| `macro_invoke`  | `container`, `name`                              |
| `index_expr`    | `container`, `target_fingerprint`                |

`receiver_fingerprint` is a normalized string of the receiver expression.

## Workflow

```bash
# detect what is currently in the tree
cargo xtask check-no-panic-family

# generate a proposed baseline (does not mutate policy/)
cargo xtask no-panic propose
ls target/policy/reports/no-panic-proposed-allowlist.toml

# review proposed entries, copy the ones you intend to keep into
# policy/no-panic-allowlist.toml, and fill in owner / explanation / expires
```

`no-panic propose` will **never** mutate `policy/no-panic-allowlist.toml`.
That is a deliberate guardrail: receipts are an editorial decision, not
an automation output.

## What the checker fails on

* Unallowlisted findings.
* Stale entries — receipts whose selector no longer matches anything in
  the tree (unless `retired = true`).
* Expired entries — receipts whose `expires` is in the past.
* Drift warnings — `last_seen` line/col diverged from the current call
  site (advisory; promote to error in Stage 3).

## What it does *not* do

* It does not (yet) understand semantic conditions on the call. A
  `unwrap()` that is provably infallible because the value comes from a
  matching `is_some()` check is currently treated the same as any other.
  Use `if let Some(...)` / `match` to make that visible to the reader as
  well as the checker.
* It does not modify source files. Suppressions at the call site
  (`#[expect(..., reason = "policy:no-panic:<id>")]`) remain a manual
  step until Stage 3.

## Crate posture

Adze is a parser/AST grammar toolchain. The high-leverage families for
this codebase are:

* `string_slice` and `char_indices_as_byte_indices` — UTF-8 boundary bugs
  in parser code are silent and brutal.
* `indexing` and `out_of_bounds_indexing` — table lookups in
  `glr-core` and `tablegen` happen on hot paths.
* `unwrap_in_result` — `Extract` and parse adapters return `Result`;
  internal `.unwrap()` collapses error context.

These should burn down first.
