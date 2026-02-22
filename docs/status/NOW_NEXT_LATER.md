# Now / Next / Later

**Last updated:** 2026-02-22

Adze status and rolling execution plan. For paper cuts and pain points, see [`docs/status/FRICTION_LOG.md`](./FRICTION_LOG.md).

---

## Now

### 🚀 Documentation Sync
- [x] Rework `ARCHITECTURE.md` with Mermaid and Governance details.
- [x] Update `GETTING_STARTED.md` and `GRAMMAR_EXAMPLES.md` for 0.8.0.
- [x] Sync `DEVELOPER_GUIDE.md` with `just` and `xtask` workflows.
- [x] Update `ROADMAP.md` and `KNOWN_LIMITATIONS.md`.

### 🟢 Maintain Supported Lane
- [ ] Ensure `just ci-supported` stays under 5 minutes on standard hardware.
- [ ] Keep `crates/` micro-crate boundaries clean as governance evolves.

---

## Next

### 📦 Publishable Baseline
- [ ] Finalize the "Support Lane" vs "Experimental Lane" crate split.
- [ ] Perform a clean `cargo package` dry-run for all core crates.
- [ ] Standardize feature-flag names across the workspace (`glr`, `simd`, etc).

### 🛠️ CLI Refinement
- [ ] Implement `adze check` for static grammar validation.
- [ ] Implement `adze stats` for parse table metrics (states, symbols, conflicts).

---

## Later

### 🌳 Incremental Parsing
- Move from conservative fallback to active forest-splicing for massive performance gains in editors.

### 🔍 Query Completion
- Implement remaining Tree-sitter query predicates (`#any-of?`, etc) and provide a cookbook.

### 🌐 Playground & LSP
- Stabilize the LSP generator so it can be used to generate production-grade language servers.
