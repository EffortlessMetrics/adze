# File policy

Adze is a Rust-first repository. Every committed file whose extension is
not on the implicit-allow list must either:

1. Be matched by a glob entry in `policy/non-rust-allowlist.toml`, with
   an owner, surface, classification, and reason — or
2. Be replaced by Rust-equivalent code (typically inside `xtask/`).

The check `cargo xtask check-file-policy` is authoritative.

## Implicit-allow extensions

These need no entry:

```
.rs   .toml   .lock   .md   .txt   .gitignore   .editorconfig
.snap .proptest-regressions   .sha256   .sha
```

Any other extension or pattern in a tracked file requires a receipt.

## Surfaces

The `surface` field describes *where* the file lives in the system map:

| `surface`     | Examples                                          |
| ------------- | ------------------------------------------------- |
| `ci`          | `.github/workflows/*.yml`                         |
| `github`      | issue templates, FUNDING, repo settings           |
| `tooling`     | `pre-commit`, `justfile`, `Makefile`              |
| `ide`         | `.vscode/*.json`                                  |
| `scripts`     | `scripts/*.sh`, build wrappers                    |
| `fixtures`    | parser inputs, expected trees, golden references  |
| `demo`        | playground, wasm-demo, dashboards                 |
| `ffi`         | C bindings (ts-bridge, ts-c-harness)              |
| `site`        | mdbook generated output                           |
| `baseline`    | criterion baselines                               |
| `profiling`   | dhat heap captures, etc.                          |
| `test-runner` | manifests consumed by test runners               |

## Classifications

The `classification` field describes *how strict* the receipt should be:

| `classification` | Meaning                                                |
| ---------------- | ------------------------------------------------------ |
| `production`     | Ships in builds or is user-facing (demo HTML, etc.)    |
| `test`           | Fixtures, expected outputs, integration corpora        |
| `tooling`        | Local-dev scripts that don't ship                      |
| `config`         | YAML/JSON consumed by external tooling (CI, IDE)       |
| `generated`      | Output of a deterministic generator we own             |

`production` and `test` entries SHOULD declare `covered_by` — the command
that validates the file is correct.

## Required keys

```toml
[[allow]]
glob = "..."            # required (mutually exclusive with `path`)
# path = "..."          # exact match alternative
kind = "..."            # short label, e.g. "shell_script"
owner = "..."           # team or component
surface = "..."         # see table above
classification = "..."  # see table above
reason = "..."          # one sentence
covered_by = ["..."]    # optional but recommended
expires = "YYYY-MM-DD"  # optional; required for `tooling` debt
retired = false         # default; set true to keep stale entries for audit
generated_by = "..."    # optional; the command that produces a `generated` file
```

## What the checker fails on

* Tracked non-Rust file with no implicit-allow extension and no glob
  match in `policy/non-rust-allowlist.toml`.
* Entries whose `expires` is in the past.
* Entries that match nothing in the working tree (unless
  `retired = true`).

## Generated allowlist proposal

If a new fixture or build artifact is added, run:

```bash
cargo xtask check-file-policy
```

The report at `target/policy/reports/file-policy.md` lists unallowlisted
files. Copy them into `policy/non-rust-allowlist.toml` with the
appropriate owner/surface/classification/reason — do **not** broaden a
glob to swallow unrelated files.

## Why we're picky about this

Non-Rust files erode the contract that this is a Rust-first repository.
Each shell script is a place where `set -euo pipefail` is missing. Each
piece of YAML is a place where a typo silently disables a check. Each
demo HTML page is a place where browser bugs become our bugs.

The receipt makes the choice deliberate. The expiry makes "we'll port it
later" visible.
