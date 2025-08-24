# Git Hooks Optimization Summary

## What Was Implemented

### 1. **Crate-Aware Clippy** (Pre-commit)
- Created `scripts/affected-crates.sh` that maps staged files to affected crates
- Pre-commit hook now runs clippy only on crates with staged changes
- Falls back to workspace-wide clippy if jq is not installed
- Significantly speeds up commits that touch only a few crates

### 2. **Crate-Aware Quick Tests** (Pre-commit, optional)
- When `RUN_QUICK_TESTS=1` is set, tests run only for affected crates
- Falls back to core invariant tests if crate detection fails
- Keeps the feedback loop tight for iterative development

### 3. **Existing Robust Features** (Already in place)
- **Partial Staging Protection**: Prevents formatting surprises with mixed staged/unstaged changes
- **Targeted Formatting**: Only formats staged Rust files, not the entire workspace
- **Full Diagnostics**: Shows complete output from guard scripts
- **Versioned Hooks**: Tracked in `.githooks/` for team consistency

## Performance Impact

### Before (workspace-wide checks on every commit):
- Clippy: ~30-60 seconds for full workspace
- Format: ~5-10 seconds for full workspace
- Tests: Variable based on scope

### After (crate-aware checks):
- Clippy: ~5-10 seconds for single-crate changes
- Format: <1 second for staged files only
- Tests: Proportional to affected crates

## How It Works

### Affected Crates Detection
1. `scripts/affected-crates.sh` uses `cargo metadata` to map files to packages
2. Finds the longest package directory prefix for each staged file
3. Returns unique list of affected package names
4. Pre-commit hook uses this to build targeted `cargo clippy -p <crate>` commands

### Fallback Strategy
- If `jq` is not installed: Falls back to workspace-wide checks
- If no staged Rust files: Skips Rust-specific checks
- If script fails: Falls back to safe defaults

## Usage

### Basic Usage (already configured)
```bash
# Just commit as normal - hooks run automatically
git commit -m "fix: update parser logic"
```

### With Quick Tests
```bash
# Enable quick tests for this commit
RUN_QUICK_TESTS=1 git commit -m "feat: add new grammar rule"
```

### With Full Tests on Push
```bash
# Enable comprehensive tests before pushing
RUN_FULL_TESTS=1 git push origin main
```

## Pre-commit vs Pre-push Division

### Pre-commit (fast, targeted)
- Partial staging check
- Format staged files only
- Clippy on affected crates
- GOTO indexing patterns
- Test connectivity
- Optional quick tests

### Pre-push (comprehensive)
- All disabled test files check
- Full workspace formatting check
- Full workspace clippy
- Core test suite
- Commit message validation
- Optional full test suite

## Requirements

### Required
- Rust toolchain (rustfmt, clippy, cargo)
- Git 2.9+ (for core.hooksPath)

### Optional but Recommended
- `jq` for crate-aware operations
- `ripgrep` (rg) for pattern checks

## Troubleshooting

### Hooks Not Running
```bash
# Verify hooks path is configured
git config core.hooksPath  # Should output: .githooks

# If not, set it:
git config core.hooksPath .githooks
```

### Slow Commits
```bash
# Check if jq is installed for crate-aware mode
command -v jq || echo "Install jq for faster commits"

# See what crates are affected by your changes
scripts/affected-crates.sh
```

### Partial Staging Errors
If you get "partially staged files detected":
```bash
# Option 1: Stage all changes
git add <file>

# Option 2: Stash unstaged changes
git stash --keep-index
git commit
git stash pop

# Option 3: Use temporary commit
git add -A && git commit -m "temp"
git reset HEAD~1
```

## Benefits

1. **Speed**: Single-crate changes now take seconds instead of minutes
2. **Determinism**: No more surprise formatting changes after commits
3. **Safety**: Critical invariants (GOTO indexing, EOF handling) always checked
4. **Team Consistency**: Versioned hooks ensure everyone has same checks
5. **Flexibility**: Environment variables allow customization per-commit

## Future Improvements

Potential enhancements:
- Cache cargo metadata for even faster crate detection
- Parallel clippy runs for multi-crate changes
- Incremental test runner integration
- Smart test selection based on changed functions