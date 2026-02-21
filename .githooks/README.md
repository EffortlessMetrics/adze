# adze Git Hooks

This directory contains version-controlled Git hooks for the adze project. These hooks provide robust validation and formatting to maintain code quality and prevent common issues.

## Installation

To install the hooks, run the installation script:

```bash
.githooks/install.sh
```

This will create symlinks from `.git/hooks/` to the hooks in this directory.

## Available Hooks

### pre-commit

**Purpose**: Validates and formats code before each commit

**Key Features**:
- ✅ **Partial staging detection**: Fails if files have both staged and unstaged changes (prevents formatting conflicts)
- ✅ **Targeted formatting**: Only formats staged Rust files, not the entire codebase
- ✅ **Full diagnostic output**: Shows complete output from all validation scripts
- ✅ **Comprehensive checks**: Runs clippy, GOTO indexing validation, and test connectivity checks
- ✅ **Colored output**: Easy-to-read status indicators
- ✅ **Locale fixes**: Properly configured to avoid locale warnings

**Checks performed**:
1. Partial staging detection (fails if detected)
2. Targeted formatting of staged Rust files only
3. Clippy analysis with full error reporting
4. GOTO indexing pattern validation
5. SymbolId(0) misuse detection
6. Test connectivity verification (no `.rs.disabled` files)
7. Optional quick invariant tests (if `RUN_QUICK_TESTS=1`)

**Example output**:
```
[pre-commit] Running formatting and validation checks...
→ Using robust pre-commit hook from .githooks/
→ Checking for partially staged files...
✓ No partially staged files found
→ Identifying staged Rust files...
→ Found 3 staged Rust files
→ Formatting staged Rust files...
  → Formatting runtime/src/parser.rs
  → Staging formatting changes for runtime/src/parser.rs
✓ Formatting changes staged
→ Running clippy checks...
✓ Clippy checks passed
→ Checking GOTO indexing patterns...
✅ All goto_indexing checks passed!
✓ GOTO indexing patterns verified
→ Checking for SymbolId(0) misuse...
→ Checking test connectivity...
=== Test Connectivity Check ===
✓ No disabled test files found
✓ Test connectivity verified

✅ All pre-commit checks passed
→ Commit proceeding with staged changes
```

### pre-push

**Purpose**: Runs comprehensive validation before pushing to remote

**Key Features**:
- ✅ **Comprehensive validation**: Full formatting, clippy, and test suite
- ✅ **Commit validation**: Checks all commits being pushed for disabled files and message quality
- ✅ **Breaking change detection**: Warns about potential API breaking changes
- ✅ **Full diagnostic output**: Complete error reporting for all failures

**Checks performed**:
1. Disabled test file detection in commits being pushed
2. Complete code formatting verification
3. Comprehensive clippy analysis
4. GOTO indexing pattern validation
5. Test connectivity verification
6. Core test suite execution
7. Optional full test suite (if `RUN_FULL_TESTS=1`)
8. Breaking change detection
9. Commit message validation

**Example output**:
```
[pre-push] Running comprehensive checks before push...
→ Using robust pre-push hook from .githooks/
→ Pushing to: origin (https://github.com/user/adze.git)
→ Found 2 commit(s) to push
→ Checking commits for disabled test files...
✓ No disabled test files in push
→ Verifying code formatting...
✓ Code formatting verified
→ Running comprehensive clippy analysis...
✓ Clippy analysis passed
→ Checking GOTO indexing patterns...
✓ GOTO indexing patterns verified
→ Verifying test connectivity...
✓ Test connectivity verified
→ Running core test suite...
✓ Core tests passed
→ Validating commit messages...
✓ Commit messages validated

✅ All pre-push checks passed
→ Push proceeding to origin
```

## Environment Variables

### `RUN_QUICK_TESTS`
- **Hook**: pre-commit
- **Purpose**: Enable quick invariant tests during pre-commit
- **Usage**: `RUN_QUICK_TESTS=1 git commit -m "message"`

### `RUN_FULL_TESTS`
- **Hook**: pre-push  
- **Purpose**: Require full test suite to pass before push
- **Usage**: `RUN_FULL_TESTS=1 git push`

## Key Improvements Over Standard Hooks

### 1. Partial Staging Detection
The hooks detect and **fail** when files have both staged and unstaged changes. This prevents the common issue where formatting changes unstaged content but only staged content gets committed.

**Problem solved**:
```bash
# This scenario now fails:
echo "fn main() {}" >> src/lib.rs    # Unstaged change
git add src/lib.rs                   # Stage current content  
echo "// comment" >> src/lib.rs      # More unstaged changes
git commit                          # FAILS - partial staging detected
```

**Solution provided**:
```
✖ Partially staged files detected
The following files have staged changes but also unstaged changes:
  src/lib.rs

Please either:
  1. Stage all changes: git add <file>
  2. Unstage partial changes: git reset <file>  
  3. Stash unstaged changes: git stash --keep-index
```

### 2. Targeted Formatting
Instead of running `cargo fmt --all`, the hooks identify staged Rust files and format only those files.

**Benefits**:
- ⚡ Faster execution (only format changed files)
- 🎯 Precise staging (only stage formatting changes for files being committed)
- 🔒 Avoid formatting unrelated files in the repository

### 3. Full Diagnostic Output
All guard scripts now show their complete output instead of being silenced with `>/dev/null 2>&1`.

**Before**: `scripts/check-goto-indexing.sh >/dev/null 2>&1`
**After**: `scripts/check-goto-indexing.sh` (shows full output)

This provides better debugging information when checks fail.

### 4. Version-Controlled Hooks
Hooks are stored in `.githooks/` and version-controlled, allowing:
- ✅ Consistent hook behavior across team members
- ✅ Hook evolution tracked in git history  
- ✅ Easy installation via `install.sh` script
- ✅ No risk of losing custom hooks during git operations

## Troubleshooting

### Hook Not Running
```bash
# Check if hooks are installed
ls -la .git/hooks/

# Reinstall hooks
.githooks/install.sh

# Verify hook is executable
chmod +x .git/hooks/pre-commit
```

### Partial Staging Errors
```bash
# Option 1: Stage all changes
git add path/to/file

# Option 2: Unstage and commit only staged changes  
git reset path/to/file

# Option 3: Stash unstaged changes temporarily
git stash --keep-index
```

### Clippy Failures
```bash
# See detailed clippy output
cargo clippy --all-targets --all-features

# Fix specific issues
cargo clippy --fix --allow-dirty
```

### Disabled Test Files
```bash
# Find all disabled test files
find . -name "*.rs.disabled"

# Re-enable a test file
git mv tests/my_test.rs.disabled tests/my_test.rs

# Use #[ignore] instead of file renaming for temporary disabling
#[ignore = "TODO: fix flaky test"]
#[test]
fn my_test() { ... }
```

### Skipping Hooks Temporarily
```bash
# Skip pre-commit hooks
git commit --no-verify -m "emergency commit"

# Skip pre-push hooks
git push --no-verify
```

## Architecture

The hooks follow a consistent pattern:

1. **Setup**: Color definitions, locale configuration, error handling
2. **Detection**: Identify issues early (partial staging, disabled files)
3. **Targeted Actions**: Only act on relevant files/changes
4. **Full Diagnostics**: Show complete output for transparency
5. **Clear Guidance**: Provide actionable error messages and solutions

This design ensures the hooks are both robust and developer-friendly, preventing common git workflow issues while maintaining fast, targeted validation.