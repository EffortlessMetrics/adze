# Dev Workflow (Quick Linting)

This guide covers the essential linting commands for adze development.

## Fast local checks (3-5s)
```bash
cargo xtask lint --fast
```
Runs fmt check, no-mangle validator, and clippy on core crates only. Perfect for the tight edit-compile-test cycle.

## Pre-commit mirror (staged files)
```bash
cargo xtask lint --fast --changed-only
```
Checks only staged .rs files. Mirrors what the pre-commit hook runs.

## PR mirror (diff vs main)
```bash
git fetch origin main
cargo xtask lint --since origin/main --fast
```
Checks files changed since main branch. Mirrors what CI runs on PRs.

## Full CI-equivalent
```bash
cargo xtask lint
```
Runs the complete lint pipeline including all self-tests and full workspace clippy.

## Auto-fix
```bash
cargo xtask lint --fix
```
Automatically fixes formatting issues and debug block problems (adds missing `// );` markers).

## Debug Block Checker

The codebase uses a custom debug block checker to prevent accidentally committed debug code:

### Valid debug blocks
```rust
// eprintln!("debug: {}", value    // );  ← properly closed
```

### Auto-fixable issues
```rust
// eprintln!("debug: {}", value    ← missing // );
```

### Check specific files
```bash
python3 tools/check_debug_blocks.py path/to/file.rs
```

### Fix specific files
```bash
python3 tools/check_debug_blocks.py --fix path/to/file.rs
```

## Pre-commit Setup

To enable automatic checks before commits:
```bash
# Install the pre-commit hook
cp .git/hooks/pre-commit.sample .git/hooks/pre-commit  # if needed
echo '#!/bin/bash' > .git/hooks/pre-commit
echo 'cargo xtask lint --fast --changed-only' >> .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

## Common Workflows

### Before pushing a PR
```bash
# Quick check of your changes
cargo xtask lint --since origin/main --fast

# Full check (recommended before push)
cargo xtask lint --since origin/main
```

### During development
```bash
# Fast feedback loop
cargo xtask lint --fast

# Auto-fix issues
cargo xtask lint --fix
```

### Troubleshooting

**Clippy fails with "multiple times with different names"**
- Use `--fast` mode which runs clippy on individual crates
- This is a known issue with tree-sitter dependencies

**Debug block checker fails**
- Run with `--fix` to auto-fix missing `// );` markers
- Check that doc comments don't have unclosed debug patterns

**Want to skip certain checks**
- Use `--fast` to skip self-tests and limit clippy scope
- Use `--changed-only` to only check staged files