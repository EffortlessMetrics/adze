# Quick Reference - Adze Development

## 🚀 Green-Light Checklist

### Local Sanity (Fast)
```bash
cargo lint --fast                      # Using new alias!
cargo lint --fast --changed-only
cargo lint --fast --since origin/main
```

### Full Sweep (CI-ish)
```bash
cargo lint                              # Full workspace lint
```

### One-liner Release Check
```bash
cargo xtask lint --since origin/main && \
git tag -a v0.6.1-beta -m "Algorithmically correct GLR parser" && \
git push origin v0.6.1-beta
# Then publish crates in order per PUBLISH_CHECKLIST.md
```

### Make Commands (Alternative)
```bash
make lint-fast   # Quick lint on changed files
make lint        # Full lint suite
```

## 🔧 Clippy Re-enable (When TS Edges Unified)

### 1. Inspect Edges
```bash
cargo tree -p adze -i tree-sitter -e features
cargo tree -p adze | rg "tree-sitter"
```

### 2. Temporarily Pin in Workspace
```toml
# Cargo.toml (workspace root)
[patch.crates-io]
tree-sitter = { version = "0.25.8" }
# (and/or) tree-sitter-language = { version = "0.1.5" }
```

### 3. Flip xtask Back to Full Clippy
```bash
cargo xtask lint          # or fast mode for per-crate clippy
```

## ✨ Quality-of-Life Features

Already implemented:
- `docs/dev-workflow.md` quick guide ✅
- pre-commit → `cargo xtask lint --fast --changed-only` ✅
- validator: staged index + PR-diff + auto-fix, ignores doc comments, GH annotations ✅
- `--fast` hint when missing `--since/--changed-only` ✅

## 📝 Optional Next Steps

### PR Template
✅ Already created at `.github/PULL_REQUEST_TEMPLATE.md`

### CI Improvements
Keep parity + criterion smoke as non-blocking until thresholds are tuned.

### Cargo Alias
Already added to `.cargo/config.toml`:
```toml
[alias]
xtask = "run -p xtask --"
lint = "xtask lint"  # Now you can run: cargo lint --fast
```

### Configurable Clippy Cores
Already supported via ENV:
```bash
XTASK_CLIPPY_CORES=adze,glr-core cargo xtask lint --fast
```

---

Built with 💚 - Professional-grade lint gate with great DX and auto-repair