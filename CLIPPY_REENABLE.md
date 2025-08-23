# Clippy Re-enable Guide

When Tree-sitter dependency edges are unified, use this guide to re-enable full workspace clippy.

## Step 1: Check Current Dependency State

```bash
cargo tree -p rust-sitter -i tree-sitter -e features
cargo tree -p rust-sitter | rg "tree-sitter"
```

## Step 2: (Optional) Temporary Pin

If you need to pin versions before full unification:

```toml
# Cargo.toml (workspace root)
[patch.crates-io]
tree-sitter = { version = "0.25.8" }
# tree-sitter-language = { version = "0.1.5" }  # if required
```

## Step 3: Re-enable Full Workspace Clippy

Replace the current degraded clippy logic in `xtask/src/lint.rs`:

```rust
// xtask/src/lint.rs (inside non-fast path)
println!("Running clippy on full workspace...");
let mut clippy_cmd = vec!["clippy", "--workspace", "--all-features", "--", "-D", "warnings"];
let extra: Vec<&str> = clippy_args.iter().map(|s| s.as_str()).collect();
clippy_cmd.extend(extra);
run("cargo", &clippy_cmd).context("cargo clippy failed")?;
```

## Step 4: Test

```bash
cargo xtask lint
cargo xtask lint --fast
```

## Notes

The current degraded mode runs clippy per-crate to avoid cross-crate dependency conflicts.
Once Tree-sitter versions are aligned, full workspace clippy will work again.