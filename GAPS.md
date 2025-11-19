# Rust-Sitter: Known Gaps and Contribution Opportunities

**Last Updated**: November 15, 2025
**Version**: v0.6.1-beta

This document provides a comprehensive, structured view of what needs to be completed in rust-sitter. Each section is designed to make it easy for contributors to pick up and complete specific tasks.

---

## 📊 Quick Overview

| Category | Total Tasks | Estimated Effort | Priority |
|----------|-------------|------------------|----------|
| [Ignored Tests](#ignored-tests-20-total) | 20 tests | 2-3 weeks | High |
| [Incremental Parsing](#incremental-parsing) | 3 major tasks | 2-3 weeks | High |
| [Query System](#query-system) | 2 major tasks | 1-2 weeks | High |
| [Performance](#performance-benchmarking) | 3 major tasks | 1-2 weeks | Critical |
| [CLI Features](#cli-functionality) | 2 major tasks | 1 week | Medium |
| [Documentation](#documentation-gaps) | 4 tasks | 1 week | Medium |

**Total Estimated Effort to Feature Complete**: 8-13 weeks

---

## 🧪 Ignored Tests (20 Total)

These tests are currently ignored but should be re-enabled. Each test has clear acceptance criteria.

### Error Recovery Tests (7 tests)

**Location**: `glr-core/tests/test_recovery.rs`
**Estimated Effort**: 1-2 weeks
**Priority**: High
**Dependencies**: None - error recovery infrastructure exists

| Test Name | What It Tests | Why Ignored | Estimated Fix Time |
|-----------|---------------|-------------|-------------------|
| `test_empty_object_with_recovery` | Parsing `{}` with error recovery | Needs error node validation | 2 hours |
| `test_incomplete_object_recovery` | Parsing `{` with recovery | Needs EOF error handling | 2 hours |
| `test_missing_value_recovery` | Parsing `{"key":}` with recovery | Needs value error nodes | 3 hours |
| `test_valid_json_no_errors` | Valid JSON produces no errors | Error count assertion needs fix | 1 hour |
| `test_gentle_errors_bounded_recovery` | Limited error propagation | Error boundary logic needs implementation | 4 hours |
| `test_cell_parity_after_lbrace` | Action cell verification after `{` | GLR action cell verification needed | 3 hours |
| `test_zero_width_progress_guard` | Prevents infinite error loops | Zero-width token detection needed | 4 hours |

**How to Contribute**:
1. Pick a test from the table above
2. Read the test code to understand what it's checking
3. Run the test with `cargo test <test_name> -- --ignored --nocapture`
4. Fix the underlying issue (usually in `glr-core/src/lib.rs` or `glr-core/src/error_recovery.rs`)
5. Remove the `#[ignore]` attribute
6. Verify test passes consistently
7. Submit PR with "fix: enable <test_name>" commit message

**Acceptance Criteria**:
- All 7 tests pass without `#[ignore]`
- No new clippy warnings
- Error messages are helpful and accurate

---

### Parser V3 Tests (3 tests)

**Location**: `runtime/tests/parser_v3_test.rs`
**Estimated Effort**: 3-4 days
**Priority**: Medium
**Dependencies**: Parser v3 API needs completion

| Test Name | What It Tests | Why Ignored | Estimated Fix Time |
|-----------|---------------|-------------|-------------------|
| `test_parse_number` | Basic number parsing | Parser v3 API incomplete | 4 hours |
| `test_parse_addition` | Addition expression parsing | Parser v3 API incomplete | 4 hours |
| `test_parse_with_whitespace` | Whitespace handling | Parser v3 API incomplete | 4 hours |

**How to Contribute**:
1. Complete the Parser v3 API in `runtime/src/parser_v3.rs`
2. Implement missing methods: `parse()`, `set_language()`, `reset()`
3. Enable tests one by one
4. Verify all parser_v3 tests pass

**Acceptance Criteria**:
- Parser v3 API is complete and documented
- All 3 tests pass
- Parser v3 has feature parity with Parser (runtime2)

---

### External Scanner Tests (1 test)

**Location**: `runtime/tests/external_scanner_blackbox.rs`
**Estimated Effort**: 4-6 hours
**Priority**: Low
**Dependencies**: External scanner API stability

| Test Name | What It Tests | Why Ignored | Estimated Fix Time |
|-----------|---------------|-------------|-------------------|
| `test_adapter_position_tracking` | Position tracking in external scanners | API needs stabilization | 4 hours |

**How to Contribute**:
1. Review external scanner position tracking implementation
2. Update test to match current API
3. Verify position tracking works correctly
4. Enable test

**Acceptance Criteria**:
- Position tracking is accurate for multi-line input
- Column and line numbers are correct
- UTF-8 characters are handled properly

---

### Helper Function Tests (4 tests)

**Location**: `tool/tests/test_helper_functions.rs`
**Estimated Effort**: 1 day
**Priority**: Low
**Dependencies**: Grammar helper functions need to be implemented

| Test Name | What It Tests | Why Ignored | Estimated Fix Time |
|-----------|---------------|-------------|-------------------|
| `test_comma_sep_helper` | `comma_sep()` helper | Helper not implemented | 2 hours |
| `test_comma_sep1_helper` | `comma_sep1()` helper (non-empty) | Helper not implemented | 2 hours |
| `test_parens_helper` | `parens()` helper | Helper not implemented | 2 hours |
| `test_multiple_helpers` | Multiple helpers together | Depends on above | 2 hours |

**How to Contribute**:
1. Implement helper functions in `tool/src/helpers.rs` (create if needed)
2. Add helpers: `comma_sep<T>()`, `comma_sep1<T>()`, `parens<T>()`
3. Enable tests
4. Add documentation with examples

**Acceptance Criteria**:
- Helper functions work like tree-sitter equivalents
- Clear documentation with usage examples
- All 4 tests pass

---

### Pure Rust E2E Test (1 test)

**Location**: `tool/tests/pure_rust_e2e_test.rs`
**Estimated Effort**: 1 day
**Priority**: Medium
**Dependencies**: None

| Test Name | What It Tests | Why Ignored | Estimated Fix Time |
|-----------|---------------|-------------|-------------------|
| `test_json_grammar_generation` | End-to-end JSON grammar generation | Table format verification needed | 6 hours |

**How to Contribute**:
1. Run test and analyze failure
2. Fix table generation issues
3. Verify generated parser works correctly
4. Enable test

**Acceptance Criteria**:
- JSON grammar generates correct parse tables
- Generated parser can parse valid JSON
- Table format matches tree-sitter

---

### Benchmark Tests (2 tests)

**Location**: `example/tests/integration.rs`
**Estimated Effort**: 4 hours
**Priority**: Low (run with `--ignored` flag)
**Dependencies**: None

| Test Name | What It Tests | Why Ignored | Notes |
|-----------|---------------|-------------|-------|
| `bench_deep_subtraction_tree` | Deep nested expressions | Performance benchmark | Run manually with `cargo test --ignored` |
| `bench_complex_precedence` | Complex precedence expressions | Performance benchmark | Run manually with `cargo test --ignored` |

**How to Contribute**:
- These are intentionally ignored - they're benchmarks, not regular tests
- Run with `cargo test -- --ignored` to verify performance
- Track results over time to detect regressions

---

## 🔄 Incremental Parsing

**Current Status**: Infrastructure exists but `parse_with_old_tree` is not implemented
**Estimated Effort**: 2-3 weeks
**Priority**: High

### Tasks

#### 1. Implement `parse_with_old_tree` Functionality

**File**: `runtime2/src/parser.rs`
**Estimated Time**: 1 week
**Dependencies**: Tree edit API (already exists)

**What Needs to Be Done**:
- [ ] Implement subtree reuse logic
- [ ] Add edit range validation
- [ ] Implement tree diffing algorithm
- [ ] Add performance instrumentation
- [ ] Write comprehensive tests

**How to Contribute**:
```rust
// In runtime2/src/parser.rs
impl Parser {
    /// Parse with an old tree for incremental parsing
    pub fn parse_with_old_tree(
        &mut self,
        input: &str,
        old_tree: Option<&Tree>,
        edits: &[InputEdit],
    ) -> Result<Tree, ParseError> {
        // TODO: Implement subtree reuse
        // 1. Validate edits are within tree bounds
        // 2. Identify subtrees that can be reused
        // 3. Mark affected nodes as "dirty"
        // 4. Reparse only dirty regions
        // 5. Splice reused subtrees into new tree
    }
}
```

**Acceptance Criteria**:
- Incremental parsing is faster than full reparse for small edits
- All tree structure invariants maintained
- Works with GLR grammars
- 7 incremental tests enabled and passing

**Resources**:
- Tree-sitter incremental parsing: https://tree-sitter.github.io/tree-sitter/using-parsers#editing
- Current edit API: `runtime2/src/tree.rs`

---

#### 2. Enable Incremental Tests

**File**: Various test files
**Estimated Time**: 3 days
**Dependencies**: Task #1 above

**Tests to Enable** (7 total):
- Find all incremental parsing tests currently ignored
- Enable after implementing `parse_with_old_tree`
- Verify performance improvements

**How to Contribute**:
```bash
# Find incremental tests
grep -rn "incremental" --include="*.rs" */tests/ | grep "#\[ignore\]"

# Enable each test
# Remove #[ignore] attribute
# Verify test passes
# Measure performance improvement
```

---

#### 3. Document Incremental Parsing Strategies

**File**: `docs/INCREMENTAL_PARSING.md`
**Estimated Time**: 2 days
**Dependencies**: Task #1, #2

**What to Document**:
- [ ] How subtree reuse works
- [ ] When to use incremental vs full parse
- [ ] Performance characteristics
- [ ] API usage examples
- [ ] Troubleshooting guide

**Acceptance Criteria**:
- Clear examples showing 10x+ speedup
- API documentation complete
- Troubleshooting section for common issues

---

## 🔍 Query System

**Current Status**: Basic infrastructure exists, predicates incomplete
**Estimated Effort**: 1-2 weeks
**Priority**: High

### Tasks

#### 1. Finish Predicate Implementation

**File**: `runtime/src/query.rs` or similar
**Estimated Time**: 1 week
**Dependencies**: None

**Missing Predicates**:
- [ ] `#eq?` - Equality check
- [ ] `#match?` - Regex match
- [ ] `#any-of?` - Set membership
- [ ] `#is?` - Node type check
- [ ] `#is-not?` - Negated type check

**How to Contribute**:
```rust
// In runtime/src/query/predicates.rs
pub enum Predicate {
    Eq { left: Capture, right: String },
    Match { capture: Capture, pattern: Regex },
    AnyOf { capture: Capture, values: Vec<String> },
    Is { capture: Capture, node_type: String },
    IsNot { capture: Capture, node_type: String },
}

impl Predicate {
    pub fn evaluate(&self, captures: &[Capture], source: &str) -> bool {
        // TODO: Implement evaluation logic
    }
}
```

**Acceptance Criteria**:
- All tree-sitter query predicates supported
- 5 ignored query tests enabled and passing
- Documentation with examples
- Query cookbook with common patterns

**Resources**:
- Tree-sitter query syntax: https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries
- Current query implementation: Search for "query" in `runtime/`

---

#### 2. Add Query Cookbook

**File**: `docs/QUERY_COOKBOOK.md`
**Estimated Time**: 3 days
**Dependencies**: Task #1

**What to Include**:
- [ ] Finding all function definitions
- [ ] Extracting documentation comments
- [ ] Finding TODOs and FIXMEs
- [ ] Detecting code smells
- [ ] Refactoring patterns

**Template**:
```markdown
## Finding All Functions

```scheme
(function_definition
  name: (identifier) @function.name
  parameters: (parameters) @function.params
  body: (block) @function.body)
```

**Usage**:
```rust
let query = compile_query(QUERY_STRING)?;
for match_ in query.matches(tree.root_node(), source.as_bytes()) {
    // Process matches
}
```
```

---

## 📈 Performance Benchmarking

**Current Status**: No benchmarks being run, performance unknown
**Estimated Effort**: 1-2 weeks
**Priority**: Critical

### Tasks

#### 1. Run Existing Benchmarks

**File**: Various benchmark files
**Estimated Time**: 2 days
**Dependencies**: None

**What to Do**:
```bash
# Find all benchmarks
find . -name "bench*.rs" -o -name "*bench.rs"

# Run benchmarks
cargo bench

# Document results in PERFORMANCE_BASELINE.md
```

**Deliverable**: `docs/PERFORMANCE_BASELINE.md` with:
- Current parse speed (tokens/second)
- Memory usage
- Comparison to tree-sitter-c
- Performance characteristics by grammar complexity

---

#### 2. Add Performance Regression Tests to CI

**File**: `.github/workflows/performance.yml`
**Estimated Time**: 3 days
**Dependencies**: Task #1

**What to Create**:
- [ ] New CI workflow for performance testing
- [ ] Baseline performance metrics
- [ ] Automatic regression detection
- [ ] Performance reporting in PRs

**Template**:
```yaml
name: Performance

on: [pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run benchmarks
        run: cargo bench --bench parser_bench
      - name: Compare with baseline
        run: ./scripts/compare-performance.sh
      - name: Comment PR
        uses: actions/github-script@v6
        with:
          script: |
            // Post performance results as PR comment
```

---

#### 3. Identify Optimization Opportunities

**File**: `docs/PERFORMANCE_OPTIMIZATION.md`
**Estimated Time**: 1 week
**Dependencies**: Task #1, #2

**What to Document**:
- [ ] Profiling results (flamegraphs)
- [ ] Hot paths identified
- [ ] Optimization opportunities ranked by impact
- [ ] Implementation roadmap

**Tools to Use**:
- `cargo flamegraph` for profiling
- `heaptrack` for memory profiling
- `perf` for CPU profiling

---

## 🖥️ CLI Functionality

**Current Status**: CLI exists but missing key features
**Estimated Effort**: 1 week
**Priority**: Medium

### Tasks

#### 1. Implement Dynamic Parser Loading

**File**: `cli/src/commands/parse.rs`
**Estimated Time**: 3 days
**Dependencies**: None

**What to Implement**:
- [ ] Load compiled grammar from shared library
- [ ] Parse input file
- [ ] Display parse tree
- [ ] Error handling and reporting

**How to Contribute**:
```rust
// In cli/src/commands/parse.rs
pub fn parse_command(grammar_path: &Path, input_path: &Path) -> Result<()> {
    // 1. Load shared library
    let lib = unsafe { Library::new(grammar_path)? };

    // 2. Get language function
    let get_language: Symbol<fn() -> Language> =
        unsafe { lib.get(b"tree_sitter_language")? };

    // 3. Create parser and parse
    let mut parser = Parser::new();
    parser.set_language(get_language())?;
    let tree = parser.parse_file(input_path)?;

    // 4. Display results
    println!("{}", tree.root_node().to_sexp());
    Ok(())
}
```

**Acceptance Criteria**:
- `rust-sitter parse grammar.so input.txt` works
- Error messages are clear
- Supports all grammar formats

---

#### 2. Complete Corpus Testing

**File**: `cli/src/commands/test.rs`
**Estimated Time**: 2 days
**Dependencies**: Task #1

**What to Implement**:
- [ ] Actually run parsing tests (currently just validates format)
- [ ] Compare output to expected results
- [ ] Report pass/fail statistics
- [ ] Generate test reports

**Acceptance Criteria**:
- `rust-sitter test` runs all corpus tests
- Reports which tests passed/failed
- Exit code reflects test success

---

## 📚 Documentation Gaps

**Current Status**: Good core documentation, missing some guides
**Estimated Effort**: 1 week
**Priority**: Medium

### Tasks

#### 1. Video Tutorial Series

**Deliverable**: 5-10 short videos
**Estimated Time**: 1 week
**Dependencies**: None

**Videos to Create**:
- [ ] Getting Started (10 min)
- [ ] Writing Your First Grammar (15 min)
- [ ] Handling Operator Precedence (10 min)
- [ ] Query System Basics (15 min)
- [ ] Debugging Parse Errors (10 min)

**Platform**: YouTube, embedded in docs

---

#### 2. Grammar Author's Cookbook

**File**: `docs/GRAMMAR_COOKBOOK.md`
**Estimated Time**: 2 days
**Dependencies**: None

**Recipes to Include**:
- [ ] Whitespace handling patterns
- [ ] Keyword vs identifier disambiguation
- [ ] Handling comments
- [ ] Expression precedence
- [ ] Error recovery strategies
- [ ] External scanner patterns

---

#### 3. Performance Tuning Guide

**File**: `docs/PERFORMANCE_TUNING.md`
**Estimated Time**: 2 days
**Dependencies**: Performance benchmarking tasks

**What to Cover**:
- [ ] Grammar optimization techniques
- [ ] Memory usage reduction
- [ ] Parse speed improvements
- [ ] Profiling tools and techniques
- [ ] Common performance pitfalls

---

#### 4. Troubleshooting Guide

**File**: `docs/TROUBLESHOOTING.md`
**Estimated Time**: 1 day
**Dependencies**: None

**Sections**:
- [ ] Common build errors
- [ ] Parse errors and how to fix them
- [ ] Conflict resolution
- [ ] External scanner issues
- [ ] Performance problems

---

## 🎯 How to Pick a Task

### By Skill Level

**Beginner** (Good first issues):
- Re-enable simple ignored tests
- Add documentation examples
- Write troubleshooting guides
- Implement helper functions

**Intermediate**:
- Implement query predicates
- Complete Parser v3 API
- Add performance benchmarks
- Implement CLI commands

**Advanced**:
- Implement incremental parsing
- Optimize GLR performance
- Complex error recovery
- External scanner improvements

### By Time Available

**A Few Hours**:
- Fix a single ignored test
- Add a query cookbook recipe
- Write a troubleshooting section

**A Day**:
- Implement helper functions
- Complete external scanner test
- Add a video tutorial

**A Week**:
- Implement query predicates
- Complete incremental parsing
- Set up performance CI

**Multiple Weeks**:
- Full incremental parsing system
- Complete performance optimization
- Video tutorial series

### By Interest

**Like Testing**:
- Re-enable ignored tests
- Add performance benchmarks
- Create test frameworks

**Like Performance**:
- Run benchmarks
- Identify optimizations
- Implement performance CI

**Like Documentation**:
- Write cookbooks
- Create video tutorials
- Improve troubleshooting guides

**Like Features**:
- Implement query predicates
- Complete CLI functionality
- Implement incremental parsing

---

## 🤝 Contribution Process

### 1. Pick a Task
- Choose from the tables above
- Check if anyone else is working on it (GitHub issues)
- Comment on the issue or create one

### 2. Set Up Development Environment
```bash
git clone https://github.com/EffortlessMetrics/rust-sitter
cd rust-sitter
cargo build
cargo test
```

### 3. Make Changes
- Create a branch: `git checkout -b fix/enable-error-recovery-tests`
- Make your changes
- Write/update tests
- Run tests: `cargo test`
- Run clippy: `cargo clippy --all -- -D warnings`

### 4. Submit PR
- Push your branch
- Create PR with clear description
- Reference this document: "Fixes #X (from GAPS.md)"
- Respond to review feedback

### 5. Get Merged!
- Once approved, maintainers will merge
- Your contribution will be in the next release

---

## 📞 Getting Help

**Questions?**
- GitHub Discussions: Ask questions about any task
- GitHub Issues: Report problems or blockers
- Discord: (coming with v1.0)

**Stuck?**
- Check existing PRs for similar work
- Read the implementation files mentioned
- Ask for help in the issue or discussion

**Want to Pair?**
- Some tasks benefit from pairing
- Mention in the issue if you'd like help
- Maintainers can schedule pairing sessions

---

## 🏆 Recognition

Contributors who complete tasks will be:
- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Thanked in the changelog
- Given credit in documentation

**Major Contributions** (completing a full category):
- Co-author on related documentation
- Acknowledged in README
- Invited to planning discussions

---

**Last Updated**: November 15, 2025
**Maintained By**: rust-sitter core team
**Next Review**: Monthly

---

## Quick Stats

- **Total Open Tasks**: 43
- **Total Estimated Effort**: 8-13 weeks
- **Good First Issues**: 12
- **High Priority**: 28
- **Beginner Friendly**: 15

**Pick a task and start contributing today!** 🚀
