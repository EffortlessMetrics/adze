# Rust core migration candidates

**Last updated:** 2026-05-16
**Purpose:** inventory non-Rust or non-core surfaces that are plausible candidates
for conversion into Rust-owned Adze components, then sequence them against the
current core design.

This is a planning inventory, not a support-tier promotion. A candidate becomes
supported only when its replacement has a named proof command in
[`SUPPORT_TIERS.md`](./SUPPORT_TIERS.md) and no longer appears as an intentional
exclusion in [`KNOWN_RED.md`](./KNOWN_RED.md).

## Core-design target

Use these constraints when deciding whether a surface should be moved:

- **Rust is the implementation default.** The non-Rust file policy already says
  "Rust + xtask is the default implementation surface" and requires every other
  language/config surface to be justified in
  [`policy/non-rust-allowlist.toml`](../../policy/non-rust-allowlist.toml).
- **One parse truth.** `AdzeDocument` is the canonical native parse product;
  Tree-sitter compatibility, typed CST/AST extraction, diagnostics, metadata,
  WASM, and CLI output should project from the same document instead of from
  parallel parse products.
- **Published support crates stay narrow.** Durable support crates remain useful
  only when they own cross-crate contracts; temporary owner-module migration
  targets must be collapsed or reclassified before release gates.
- **`ci-supported` stays bounded.** Moving a surface into the core design does
  not automatically mean moving it into the required PR gate; promote proof only
  after the replacement is small, deterministic, and behavior-focused.

## Inventory snapshot

Local inventory commands on 2026-05-16 found these non-Rust implementation
surfaces outside ordinary docs/config/snapshots:

| Surface | Current footprint | Why it matters |
|---|---:|---|
| C / headers | 5 `.c` files and 13 `.h` files | Tree-sitter ABI extraction, an excluded C harness, and a tiny WASM sysroot are the highest-risk non-Rust surfaces. |
| Shell scripts | 48 `.sh` files | Many are orchestration around Cargo/CI/release checks and overlap with the existing Rust `xtask` package. |
| JavaScript | 26 `.js` files | Mostly fixtures/demos, but some grammar and dashboard/demo behavior can drift from Rust parser facts. |
| Python | 16 `.py` files | Mostly fixture inputs plus a small amount of CI/RIPR helper logic. |

Commands used for the snapshot:

```bash
rg --files -g '!target' -g '!**/.git/**' \
  | awk 'function ext(path){n=split(path,a,"/"); f=a[n]; if (f !~ /\\./) return "[none]"; sub(/^.*\\./,"",f); return "." f} {count[ext($0)]++} END{for (e in count) print count[e], e}' \
  | sort -nr

rg --files -g '!target' -g '!**/.git/**' | rg '\\.(c|h)$'

find scripts -maxdepth 2 -type f | sort
```

## Highest-value migration candidates

| Priority | Candidate | Current owner / files | Convert to | Acceptance signal |
|---:|---|---|---|---|
| 1 | Tree-sitter table extraction bridge | `tools/ts-bridge/ffi/shim.c`, `tools/ts-bridge/ffi/shim.h`, vendored Tree-sitter C headers/sources, `tools/ts-bridge/src/*` | A Rust-owned import path in `adze-tablegen` plus `adze-parsetable-metadata` that converts upstream grammar artifacts into Adze IR/table metadata once, then projects through `AdzeDocument`/runtime compatibility APIs. | `tools/ts-bridge` no longer needs a C shim or vendored internal Tree-sitter ABI for normal parity extraction; `smoke-link.sh ts-bridge` is replaced by a Rust cargo test or xtask proof. |
| 2 | C compatibility harness | `crates/ts-c-harness/tests/ts_c_shim.c`, excluded `crates/ts-c-harness` package | Rust compatibility tests that assert the same lookup/next-state behavior through `adze`/`adze-tablegen` public or test-only APIs. | `crates/ts-c-harness` can be deleted from the workspace exclusion list, or retained only as an explicitly external interop fixture. |
| 3 | Shell-based CI/release orchestration | `scripts/affected-crates.sh`, `scripts/release-surface.sh`, `scripts/validate-release-surface.sh`, `scripts/check-*.sh`, `scripts/ci-product*.sh`, `scripts/test-*.sh` | `xtask` subcommands with typed arguments, structured errors, shared Cargo metadata parsing, and thin `just` wrappers. | A `cargo run -q -p xtask -- ...` command exists for each release/policy/test lane still used by CI; shell wrappers become compatibility shims or are removed. |
| 4 | Golden/reference generation scripts | `golden-tests/**/*.sh`, `scripts/regenerate_golden_tests.sh`, tree-sitter reference invocation in scripts | `xtask` golden/reference subcommands that record inputs, tool versions, and output hashes in one Rust implementation. | Regenerating and checking golden artifacts uses `cargo xtask` consistently; ad-hoc shell does not own parity semantics. |
| 5 | WASM/demo parser behavior | `runtime/wasm-demo/**/*.html`, `wasm-demo/**`, `playground/**` web assets | Rust/WASM bindings and serialization schemas that expose `AdzeDocument` projections; leave HTML/CSS as presentation only. | Browser/demo output consumes the same native/compat schema as CLI tests, and behavior canaries assert serialized diagnostics/tree facts without DOM scraping. |
| 6 | Grammar fixture and local grammar JS drift | `xtask/fixtures/test-json/grammar.js`, `test-cli/tree-sitter-mylang/**`, language fixture `.js`/`.py` files | Rust annotated grammar examples or schema fixtures consumed by `adze-tool`/`adze-common`; keep language samples only as parser inputs. | Grammar-definition fixtures no longer require JavaScript execution to define parser semantics; parser-input samples remain classified as fixtures. |
| 7 | Python helper scripts | `scripts/ci/*.py`, `scripts/ripr-annotations.py` | Rust `xtask` modules, especially where helpers parse CI plans, repo exposure reports, or policy files already modeled in Rust. | CI/RIPR helper outputs are produced by the same Rust policy stack that owns package/file/lane policy checks. |

## Suggested sequence

### Phase 1 — migrate orchestration into `xtask`

Start with shell/Python scripts because the repo already has Rust policy and
lint infrastructure in `xtask`.

1. Add `xtask affected-crates` and route pre-commit/CI users away from
   `scripts/affected-crates.sh`.
2. Add `xtask release-surface` / `xtask validate-release-surface` using Cargo
   metadata and `policy/package-boundary.toml` directly.
3. Move product-lane orchestration into `xtask ci-product` and keep `just`
   recipes as the user-facing entrypoints.

This phase reduces dependency on `bash`, `jq`, `sed`, and Python without
changing parser semantics.

### Phase 2 — retire C ABI extraction from the normal proof path

Treat `tools/ts-bridge` as a compatibility importer, not as a second table
source of truth.

1. Define the Rust import contract in `adze-parsetable-metadata`: symbols,
   fields, aliases, parse actions, goto edges, version metadata, and source
   provenance.
2. Teach `adze-tablegen` to consume that contract and emit the same runtime
   metadata that generated parsers use.
3. Move parity checks to Rust tests over imported metadata and selected
   `AdzeDocument` projections.
4. Keep direct C/Tree-sitter ABI probes only as optional external interop tests
   until they can be deleted.

### Phase 3 — project demos/tooling from `AdzeDocument`

Once the native document schema is sufficiently implemented, CLI, WASM, and
playground outputs should serialize projections from the same parse document.
At that point JavaScript should be presentation glue, not parser/business logic.

### Phase 4 — clean up fixture semantics

Keep `.js`, `.py`, `.go`, `.json`, and snapshot files when they are language
inputs or golden outputs. Convert files only when they define Adze behavior,
parser tables, release decisions, or proof policy.

## Non-candidates

Do **not** spend migration effort on these unless their role changes:

- Markdown documentation and ADRs.
- Cargo/deny/rustfmt/CI/editor configuration that is platform-required.
- Language sample files used as parser inputs.
- `insta` snapshots and golden expected outputs.
- License files and generated badge endpoint JSON.

These are justified repository artifacts, not implementation surfaces that need
to become Rust code.
