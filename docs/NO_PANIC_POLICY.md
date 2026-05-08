# No-Panic Policy

The Adze workspace targets **no unreceipted panic-family behavior in
production or tests**. Panic-family means any call shape that can abort the
process: `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`,
unchecked indexing, byte-boundary string slicing, `Result::unwrap` on
`Result`-returning functions, and similar.

Test assertion macros (`assert!`, `assert_eq!`, `assert_ne!`) are **not**
panic-family for v1. They are oracles. We may revisit this once a fallible
assertion helper exists.

## The ledger

Exceptions are recorded in [`policy/no-panic-allowlist.toml`](../policy/no-panic-allowlist.toml).
Identity is `(path, family, selector)` — line and column drift do not
invalidate an entry.

Each entry must specify:

```toml
[[allow]]
id = "panic-0001"
path = "crates/example/src/parser.rs"
family = "unwrap"
classification = "test_helper"
owner = "parser"
explanation = "Fixture setup; migrate to fallible fixture builder."
expires = "2026-07-01"

[allow.selector]
kind = "method_call"
container = "parses_boundary_fixture"
callee = "unwrap"
receiver_fingerprint = "std::fs::read_to_string(path)"

[allow.last_seen]
line = 42
column = 17
```

`last_seen` is advisory only. The checker updates it via
`cargo xtask no-panic-propose --update-last-seen` (planned).

## Workflow

1. **Inspect debt.** Run:
   ```bash
   cargo xtask check-no-panic-family
   ```
   The checker reports unallowlisted findings, expired entries, and stale
   entries (allowlist entries with no matching code).

2. **Propose receipts.** For new debt, run:
   ```bash
   cargo xtask no-panic-propose
   ```
   This writes `target/policy/no-panic-proposed.toml`. **It never edits
   `policy/no-panic-allowlist.toml` automatically.** Review the proposal,
   set owner/reason/expires, and copy entries in by hand.

3. **Burn down.** Cleanup happens by replacing the panic-family call with a
   `Result`-returning equivalent. Update or remove the allowlist entry once
   the call is gone.

## Classifications

| Classification     | Meaning                                                         |
| ------------------ | --------------------------------------------------------------- |
| `test_helper`      | Setup/teardown plumbing inside a test module.                   |
| `fixture_setup`    | Deterministic fixture construction, e.g. `Path::new`.           |
| `invariant`        | Genuinely unreachable; documented invariant nearby.             |
| `external_contract`| Forced by an external API that does not return `Result`.        |
| `legacy`           | Pre-existing debt; must have an `expires` date.                 |
| `placeholder`      | `todo!` / `unimplemented!` for in-flight work.                  |
| `governance`       | Required by a policy framework or proc macro contract.          |

`legacy` entries must expire within 6 months. `placeholder` entries must
expire within 3 months.

## Baseline status

The initial baseline for the **7 core pipeline crates** has been generated
and committed to `policy/no-panic-allowlist.toml`:

| Crate | Directory |
|-------|-----------|
| `adze` | `runtime/` |
| `adze-macro` | `macro/` |
| `adze-tool` | `tool/` |
| `adze-common` | `common/` |
| `adze-ir` | `ir/` |
| `adze-glr-core` | `glr-core/` |
| `adze-tablegen` | `tablegen/` |

As of 2026-05-08 the baseline contains ~16,931 entries. Breakdown:
- ~16,506 `test_helper` (test/bench code)
- ~418 `invariant` (production code with documented invariants)
- 7 `fixture` (example/demo code)

The remaining workspace crates (governance micro-crates, grammars, CLI, etc.)
are not yet in scope. They will be added in a follow-up once the core
baseline is stabilized.

**Note:** The `no-panic-propose` command requires `--release` mode on Windows
due to a stack overflow in debug builds when parsing the full workspace with
`syn`.

## What the checker does

`cargo xtask check-no-panic-family` walks the workspace, parses Rust source
with `syn`, finds panic-family calls, builds a selector for each, and
matches against the allowlist. It emits Markdown + JSON reports in
`target/policy/`:

- `target/policy/no-panic.md` — human-readable summary.
- `target/policy/no-panic.json` — machine-readable findings.

Today the check runs in **advisory** mode: it reports but does not fail CI.
Once the baseline is committed and burned down, we will flip it to blocking
in `just ci-supported`.

## Exit modes

| Mode                              | Description                                  |
| --------------------------------- | -------------------------------------------- |
| `advisory` (current default)      | Report + write artifacts, never fails.       |
| `blocking-allowlist`              | Fail on unallowlisted findings.              |
| `blocking-strict`                 | Fail on unallowlisted, stale, or expired.    |

The mode is chosen via `--mode` on the command line.
