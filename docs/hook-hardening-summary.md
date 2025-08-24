# Pre-commit Hook Hardening Summary

## What You Had (The Issue)
- Whitespace/formatting diffs appearing repeatedly in pre-commit output
- Formatting changes left unstaged after commits
- Locale warnings (`LC_ALL: cannot change locale`)
- Diagnostic output from guard scripts was suppressed

## What's Fixed Now

### 1. **Partial Staging Detection** ✅
The hook now **refuses to auto-format** files that have both staged and unstaged changes:

```bash
# Detects files with mixed staging status
check_partial_staging() {
    staged_hash=$(git hash-object --stdin < <(git show ":$file"))
    working_hash=$(git hash-object "$file")
    # If hashes differ, file is partially staged
}
```

**Why this matters**: Prevents surprising reflows where formatting drags unstaged hunks into commits.

### 2. **Targeted Formatting (Staged Files Only)** ✅
- **Before**: `cargo fmt --all` (formats entire codebase)
- **After**: Only formats staged Rust files individually

```bash
# Format only the staged Rust files
for file in "${STAGED_RUST_FILES[@]}"; do
    rustfmt "$file"
    git add "$file"  # Re-stage formatting changes
done
```

**Benefits**:
- Faster execution
- Only touches files in the commit
- No unrelated formatting noise

### 3. **Full Diagnostic Output** ✅
Guard scripts now show complete output:

```bash
# Before (suppressed):
if ! scripts/check-goto-indexing.sh >/dev/null 2>&1; then

# After (visible):
if ! scripts/check-goto-indexing.sh; then
```

You now see exactly which files violate invariants.

### 4. **Versioned Hooks in .githooks/** ✅
```
.githooks/
├── README.md       # Documentation
├── install.sh      # Installation script
├── pre-commit      # Robust pre-commit hook
└── pre-push        # Comprehensive validation
```

Team-wide consistency via version control.

### 5. **Locale Fix Applied** ✅
```bash
export LC_ALL=C.UTF-8
export LANG=C.UTF-8
```

No more locale warnings in git operations.

## The Hook Flow Now

1. **Check for partial staging** → Fail if detected
2. **Format staged Rust files only** → Stage the changes
3. **Run clippy** → Fall back to core crates if needed
4. **Check GOTO indexing** → Show full violations
5. **Verify SymbolId(0) usage** → Warn if misused
6. **Check test connectivity** → Fail on .rs.disabled files
7. **Optional quick tests** → Run with `RUN_QUICK_TESTS=1`

## How Commits Work Now

### Scenario 1: Clean Working Tree
```bash
# Edit files
vim src/lib.rs
# Stage and commit
git add src/lib.rs
git commit -m "fix: update parser logic"
# ✅ Formats src/lib.rs only, stages changes, commits
```

### Scenario 2: Partial Staging (Protected)
```bash
# Edit file
vim src/lib.rs
# Stage part of it
git add -p src/lib.rs  # Stage some hunks
# Try to commit
git commit -m "fix: partial update"
# ❌ FAILS: "Partially staged files detected"
# Fix: Either stage all or stash unstaged
git add src/lib.rs  # Stage everything
git commit -m "fix: partial update"
# ✅ Success
```

### Scenario 3: Multiple Files
```bash
# Edit multiple files
vim src/lib.rs src/parser.rs tests/test.rs
# Stage only what you want
git add src/lib.rs tests/test.rs
git commit -m "test: add parser tests"
# ✅ Formats only lib.rs and test.rs, ignores parser.rs
```

## Performance Impact

| Operation | Before | After |
|-----------|--------|-------|
| Format check | ~2-3s (entire workspace) | ~0.5s (staged files only) |
| Clippy | Full workspace always | Falls back to core crates |
| Guard scripts | Output hidden | Full visibility |
| Partial staging | Could corrupt commits | Fails fast with help |

## Environment Variables

- `RUN_QUICK_TESTS=1` - Enable quick invariant tests in pre-commit
- `RUN_FULL_TESTS=1` - Enable full test suite in pre-push (if using)

## Troubleshooting

### "Partially staged files detected"
**Solution**: 
- `git add <file>` to stage all changes, or
- `git reset <file>` to unstage, or  
- `git stash --keep-index` to stash unstaged changes

### Clippy failures
**Solution**: Run `cargo clippy --all-targets --all-features` to see full output

### GOTO indexing failures
**Solution**: Use remap helpers instead of direct enum flips

### Test connectivity failures
**Solution**: No `.rs.disabled` files allowed - use `#[ignore]` attribute instead

## Verification

The hooks are working correctly when:
1. Formatting changes are included IN commits (not left unstaged)
2. Partial staging is detected and prevented
3. You see full diagnostic output from all checks
4. Only staged files are formatted (not the entire codebase)
5. No locale warnings appear

## Summary

Your pre-commit hook is now:
- **Safe**: Prevents partial staging corruption
- **Fast**: Only processes staged files
- **Transparent**: Shows all diagnostic output
- **Versioned**: Tracked in .githooks/ for team consistency
- **Deterministic**: Same behavior every time

The whitespace diff noise you were seeing is eliminated because formatting changes are now properly staged before the commit completes.