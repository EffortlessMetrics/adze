# Now / Next / Later

**Last updated:** 2026-03-03

Adze status and rolling execution plan. For paper cuts and pain points, see [`docs/status/FRICTION_LOG.md`](./FRICTION_LOG.md).

---

## Now

### 🔴 Fix Runtime Compile Errors (Stop-the-Line)
- [ ] `adze` (runtime) crate has ~20 compile errors on `main` (lifetime, type, borrow issues).
- [ ] `ci-supported` gate is **red** — this blocks all other work.
- [ ] Fix `runtime/src/pure_parser.rs` parse errors that also block `cargo fmt`.

### ✅ Workspace Polish (Complete)
- [x] Cargo.toml metadata polish across workspace crates.
- [x] Core pure-Rust pipeline compiles cleanly: `adze-ir`, `adze-glr-core`, `adze-tablegen`.
- [x] 47 microcrates in `crates/` with stable structure.
- [x] Benchmarks, fuzzing, golden-tests, and book scaffolding in place.

### 🚀 Documentation Sync (In Progress)
- [x] Rework [`ARCHITECTURE.md`](../explanations/architecture.md) with Mermaid and Governance details.
- [x] Update [`GETTING_STARTED.md`](../tutorials/getting-started.md) and [`GRAMMAR_EXAMPLES.md`](../reference/grammar-examples.md) for 0.8.0.
- [x] Sync [`DEVELOPER_GUIDE.md`](../DEVELOPER_GUIDE.md) with `just` and `xtask` workflows.
- [ ] Close remaining release blockers in doc history/version drift (`FR-001`): version strings and legacy naming in advanced how-to guides.
- [x] Update [`ROADMAP.md`](../../ROADMAP.md) and [`KNOWN_LIMITATIONS.md`](../reference/known-limitations.md).

---

## Next

### 📦 Publishable Baseline
- [ ] Resolve all runtime compile errors so `cargo check --workspace` passes.
- [ ] Perform a clean `cargo package` dry-run for all core crates.
- [ ] Finalize the "Supported Lane" vs "Experimental Lane" crate split.
- [ ] Standardize feature-flag names across the workspace (`glr`, `simd`, etc).
- [ ] Add READMEs to `crates/` microcrates (only 1 of 47 currently has one).

### 🛠️ CLI Refinement
- [ ] Implement `adze check` for static grammar validation.
- [ ] Implement `adze stats` for parse table metrics (states, symbols, conflicts).

---

## Later

### 🌳 Incremental Parsing
- Move from conservative fallback to active forest-splicing for massive performance gains in editors.
- Currently disabled and falls back to fresh parsing (see `glr_incremental.rs`).

### 🔍 Query Completion
- Implement remaining Tree-sitter query predicates (`#any-of?`, etc) and provide a cookbook.

### 🌐 Playground & LSP
- Stabilize the LSP generator so it can be used to generate production-grade language servers.
