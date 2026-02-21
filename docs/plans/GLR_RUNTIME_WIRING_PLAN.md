# GLR Runtime Wiring Implementation Plan

**Status**: INFRASTRUCTURE COMPLETE ✅ (Steps 1-3 + parser_v4 integration)
**Priority**: HIGH (Testing and enablement remaining)
**Effort**: 8-12 hours (original) + 4-6 hours (parser_v4 integration) - COMPLETED
**Related**: ARCHITECTURE_ISSUE_GLR_PARSER.md, PARSER_V4_EXTRACTION_INTEGRATION.md

---

## 📊 Current Status (2025-11-19 - Updated)

### ✅ Completed
- **Step 1**: Feature flag architecture (`glr` feature added)
- **Step 2**: Parser backend selection API (ParserBackend enum + tests)
- **Step 3**: Parser routing infrastructure (full GLR integration)
- **parser_v4 Integration**: Extraction integration complete ✅
  - Added `parse_tree()` method to parser_v4
  - Implemented ParseNode to ParsedNode conversion layer
  - Full end-to-end GLR parsing pipeline working

### 🚧 Previously Blocked - PARTIALLY RESOLVED
- ~~**Full GLR Integration**: Blocked by parser_v4 extraction incompatibility~~ (✅ RESOLVED)
  - See: [PARSER_V4_EXTRACTION_INTEGRATION.md](./PARSER_V4_EXTRACTION_INTEGRATION.md)
  - ✅ RESOLVED: Added `parse_tree()` method returning `ParseNode`
  - ✅ RESOLVED: Conversion layer implemented
  - ✅ RESOLVED: Full GLR extraction pipeline architecture complete

### ❌ NEW BLOCKER - Table Loading Incompatibility
- **parser_v4 Table Loading**: decoder cannot correctly load/interpret GLR tables
  - See: [PARSER_V4_TABLE_LOADING_BLOCKER.md](./PARSER_V4_TABLE_LOADING_BLOCKER.md)
  - Issue: parser_v4 hits error states instead of successfully parsing
  - Root Cause: Mismatch between tablegen encoding and decoder interpretation
  - Impact: GLR feature compiles but parsing fails at runtime
  - Status: Documented, investigating decoder fix vs. alternative approaches

### 🔄 Current Behavior
- ✅ Routing logic compiles and works
- ✅ Feature flag selection works correctly
- ✅ GLR path uses parser_v4 with full extraction architecture
- ❌ GLR parsing fails due to table loading issue

---

## 🎯 Goal

Wire `parser_v4.rs` (full GLR runtime) as the default parser for macro-generated grammars in pure-Rust mode, replacing the simple LR `pure_parser.rs`.

---

## 📋 Prerequisites (Already Complete ✅)

- ✅ GLR table generation correct (`glr-core/src/lib.rs`)
- ✅ Table compression preserves multi-action cells (`tablegen/src/compress.rs`)
- ✅ Action encoding contract defined and tested (`tablegen/src/schema.rs`)
- ✅ BDD scenarios documented (27 scenarios in `tests/features/glr_runtime_integration.feature`)
- ✅ Action::Error decoding fixed (`runtime/src/pure_parser.rs`)
- ✅ Test policy enforcement (CI prevents regressions)

---

## 🏗️ Architecture Design

### Current State (Broken)
```
User Grammar
    ↓
adze_tool::build_parsers()
    ↓
GLR tables generated (correct) ✅
    ↓
runtime::__private::parse()
    ↓
pure_parser::Parser (LR only) ❌ ← PROBLEM
    ↓
Wrong associativity/precedence
```

### Target State (Fixed)
```
User Grammar
    ↓
adze_tool::build_parsers()
    ↓
GLR tables generated (correct) ✅
    ↓
runtime::__private::parse()
    ↓
Feature flag routing:
├─ tree-sitter → C runtime (default)
├─ pure-rust    → pure_parser (LR, simple grammars)
└─ glr          → parser_v4 (GLR, full support) ✅
    ↓
Correct associativity/precedence
```

---

## 🔧 Implementation Steps (TDD/BDD)

### Step 1: Feature Flag Architecture (2 hours)

**File**: `runtime/Cargo.toml`

```toml
[features]
default = ["tree-sitter-standard"]
tree-sitter-standard = []
tree-sitter-c2rust = []
pure-rust = []
glr = ["pure-rust"]  # GLR requires pure-rust
```

**Test**:
```bash
# Should compile with different feature combinations
cargo build -p adze --features "pure-rust"
cargo build -p adze --features "glr"
cargo build -p adze --features "tree-sitter-standard"
```

**Acceptance Criteria**:
- [ ] All feature combinations compile
- [ ] `glr` feature implies `pure-rust`
- [ ] No feature conflicts

---

### Step 2: Parser Selection API (3 hours)

**File**: `runtime/src/parser_selection.rs` (new)

```rust
/// Parser backend selection based on feature flags and grammar requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserBackend {
    /// Tree-sitter C runtime (default, stable)
    TreeSitter,
    /// Pure Rust LR parser (simple grammars, WASM)
    PureRust,
    /// Pure Rust GLR parser (ambiguous grammars, experimental)
    GLR,
}

impl ParserBackend {
    /// Select parser backend based on compile-time features and grammar metadata
    pub fn select(has_conflicts: bool) -> Self {
        #[cfg(feature = "glr")]
        {
            Self::GLR
        }

        #[cfg(all(feature = "pure-rust", not(feature = "glr")))]
        {
            if has_conflicts {
                panic!(
                    "Grammar has shift/reduce conflicts but GLR feature not enabled. \
                     Enable with: features = [\"glr\"]"
                );
            }
            Self::PureRust
        }

        #[cfg(all(
            not(feature = "pure-rust"),
            not(feature = "glr"),
        ))]
        {
            Self::TreeSitter
        }
    }
}
```

**Tests**: `runtime/tests/test_parser_selection.rs`
```rust
#[test]
#[cfg(feature = "glr")]
fn test_glr_feature_selects_glr_backend() {
    assert_eq!(ParserBackend::select(false), ParserBackend::GLR);
    assert_eq!(ParserBackend::select(true), ParserBackend::GLR);
}

#[test]
#[cfg(all(feature = "pure-rust", not(feature = "glr")))]
fn test_pure_rust_rejects_conflicts() {
    // Should succeed for conflict-free grammars
    assert_eq!(ParserBackend::select(false), ParserBackend::PureRust);

    // Should panic for conflicting grammars
    let result = std::panic::catch_unwind(|| {
        ParserBackend::select(true)
    });
    assert!(result.is_err());
}

#[test]
#[cfg(not(any(feature = "pure-rust", feature = "glr")))]
fn test_default_selects_tree_sitter() {
    assert_eq!(ParserBackend::select(false), ParserBackend::TreeSitter);
    assert_eq!(ParserBackend::select(true), ParserBackend::TreeSitter);
}
```

**Acceptance Criteria**:
- [ ] `ParserBackend::select()` returns correct backend based on features
- [ ] Pure-rust mode rejects conflicting grammars with helpful error
- [ ] GLR mode accepts all grammars
- [ ] Tests pass for all feature combinations

---

### Step 3: Wire parser_v4 in __private::parse() (3 hours)

**File**: `runtime/src/__private.rs`

```rust
pub fn parse<T: Extract>(source: &str) -> Result<T, String> {
    let backend = ParserBackend::select(T::HAS_CONFLICTS);

    match backend {
        ParserBackend::TreeSitter => {
            // Existing tree-sitter C runtime path
            parse_with_tree_sitter(source)
        }

        ParserBackend::PureRust => {
            // Simple LR parser (no conflicts)
            parse_with_pure_parser(source)
        }

        ParserBackend::GLR => {
            // Full GLR parser
            parse_with_glr(source)
        }
    }
}

#[cfg(feature = "glr")]
fn parse_with_glr<T: Extract>(source: &str) -> Result<T, String> {
    // Load grammar from T::GRAMMAR_JSON
    let grammar = serde_json::from_str(T::GRAMMAR_JSON)?;

    // Create parser_v4 instance
    let mut parser = parser_v4::Parser::new(&grammar)?;

    // Parse
    let tree = parser.parse(source)?;

    // Extract typed AST
    T::extract_from_tree(&tree)
}
```

**Acceptance Criteria**:
- [ ] `parse()` routes to correct backend
- [ ] GLR path uses `parser_v4::Parser`
- [ ] Compiles with all feature combinations
- [ ] Type checking enforces `HAS_CONFLICTS` metadata

---

### Step 4: Add Grammar Metadata (2 hours)

**File**: `tool/src/pure_rust_builder.rs`

Update grammar generation to include conflict information:

```rust
impl LanguageGenerator {
    pub fn generate(&self) -> TokenStream {
        let has_conflicts = self.detect_conflicts();

        quote! {
            impl Extract for #type_name {
                const HAS_CONFLICTS: bool = #has_conflicts;
                const GRAMMAR_JSON: &'static str = #grammar_json;
                // ...
            }
        }
    }

    fn detect_conflicts(&self) -> bool {
        // Check if any state has multiple actions for a symbol
        self.parse_table.action_table.iter().any(|row| {
            row.iter().any(|cell| cell.len() > 1)
        })
    }
}
```

**Test**: `tool/tests/test_grammar_metadata.rs`
```rust
#[test]
fn test_conflict_detection() {
    let grammar_with_conflicts = /* arithmetic with left-assoc */;
    assert!(grammar_with_conflicts.has_conflicts());

    let grammar_without_conflicts = /* simple grammar */;
    assert!(!grammar_without_conflicts.has_conflicts());
}
```

**Acceptance Criteria**:
- [ ] `HAS_CONFLICTS` correctly set based on parse table
- [ ] Metadata included in generated code
- [ ] Detection logic tested

---

### Step 5: BDD Scenario Verification (2 hours)

Implement Rust tests that verify the BDD scenarios from `tests/features/glr_runtime_integration.feature`:

**File**: `runtime/tests/glr_bdd_scenarios.rs`

```rust
/// BDD Scenario: Left-associative multiplication
/// Given a grammar with left-associative multiplication at precedence 2
/// When I parse "1 * 2 * 3"
/// Then the result should be ((1 * 2) * 3)
#[test]
#[cfg(feature = "glr")]
fn scenario_left_associative_multiplication() {
    use arithmetic::*;  // Example grammar

    let result = parse("1 * 2 * 3").expect("Parse should succeed");

    // Verify structure is left-associative
    match result {
        Expr::Mul(box Expr::Mul(box Expr::Num(1), _, box Expr::Num(2)), _, box Expr::Num(3)) => {
            // Correct: ((1 * 2) * 3)
        }
        _ => panic!("Expected left-associative tree, got: {:?}", result),
    }
}

/// BDD Scenario: Right-associative exponentiation
#[test]
#[cfg(feature = "glr")]
fn scenario_right_associative_power() {
    // Similar implementation for right-associativity
}

/// BDD Scenario: Mixed precedence
#[test]
#[cfg(feature = "glr")]
fn scenario_mixed_precedence() {
    let result = parse("1 + 2 * 3").expect("Parse should succeed");

    // Verify: 1 + (2 * 3), not (1 + 2) * 3
    match result {
        Expr::Add(box Expr::Num(1), _, box Expr::Mul(..)) => {
            // Correct precedence
        }
        _ => panic!("Expected correct precedence, got: {:?}", result),
    }
}
```

**Acceptance Criteria**:
- [ ] At least 10 BDD scenarios implemented as tests
- [ ] Tests cover left/right associativity
- [ ] Tests cover mixed precedence
- [ ] All tests pass with `--features glr`

---

### Step 6: Re-enable Arithmetic Tests (1 hour)

**Files**:
- `example/src/arithmetic.rs` - Remove `#[ignore]` from tests

**Process**:
1. Remove `#[ignore]` from associativity tests
2. Run with GLR feature: `cargo test -p adze-example --features glr`
3. Verify all tests pass
4. Update test documentation

**Acceptance Criteria**:
- [ ] `example/src/arithmetic.rs`: All tests passing
- [ ] No `#[ignore]` on associativity tests
- [ ] CI runs tests with GLR feature

---

## 📊 Test Strategy

### Unit Tests
- [ ] `ParserBackend::select()` logic
- [ ] Grammar conflict detection
- [ ] Feature flag combinations

### Integration Tests
- [ ] `__private::parse()` routing
- [ ] GLR parser initialization
- [ ] Tree extraction

### BDD Tests
- [ ] 10+ scenarios from feature file
- [ ] Associativity verification
- [ ] Precedence verification

### Regression Tests
- [ ] Python grammar (273 symbols)
- [ ] Arithmetic examples
- [ ] All previously passing tests

---

## 🚀 Rollout Plan

### Phase 1: Infrastructure (Steps 1-2)
- Feature flags
- Parser selection API
- Tests for selection logic

### Phase 2: Integration (Steps 3-4)
- Wire parser_v4
- Add grammar metadata
- Integration tests

### Phase 3: Verification (Steps 5-6)
- BDD scenario tests
- Re-enable arithmetic tests
- Regression testing

---

## ✅ Definition of Done

### Code Complete
- [ ] All 6 steps implemented
- [ ] Code reviewed (or self-reviewed against checklist)
- [ ] Documentation updated

### Tests Passing
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] At least 10 BDD scenarios pass
- [ ] Arithmetic tests re-enabled and passing
- [ ] CI green on all feature combinations

### Documentation Updated
- [ ] README.md - Update "Parser Modes" table with actual feature flags
- [ ] ARCHITECTURE.md - Mark GLR wiring as complete
- [ ] ARCHITECTURE_ISSUE_GLR_PARSER.md - Mark as RESOLVED
- [ ] STATUS_NOW.md - Update blockers list

### Acceptance Criteria Met
- [ ] `cargo test --features glr` passes 100%
- [ ] `1 * 2 * 3` parses as `((1 * 2) * 3)` with left-assoc ✅
- [ ] `2 ^ 3 ^ 4` parses as `(2 ^ (3 ^ 4))` with right-assoc ✅
- [ ] `1 + 2 * 3` parses as `(1 + (2 * 3))` with correct precedence ✅
- [ ] No regressions in existing tests

---

## 🎯 Success Metrics

**Before**:
- Macro path: 100% working
- Pure-Rust GLR path: Broken (wrong associativity)
- Ignored tests: 20

**After**:
- Macro path: 100% working (no change)
- Pure-Rust GLR path: 100% working ✅
- Ignored tests: <15 (at least 5 re-enabled)
- GLR runtime: Default for conflicting grammars ✅

---

## 📅 Timeline

**Total Effort**: 8-12 hours
**Target Completion**: 2-3 days (part-time) or 1-2 days (full-time)

**Day 1** (4-6 hours):
- Steps 1-2: Feature flags + Parser selection

**Day 2** (4-6 hours):
- Steps 3-4: Wire parser_v4 + Grammar metadata

**Day 3** (2-3 hours):
- Steps 5-6: BDD tests + Re-enable arithmetic tests
- Documentation updates

---

## 🔗 Related Documents

- [ARCHITECTURE_ISSUE_GLR_PARSER.md](../../ARCHITECTURE_ISSUE_GLR_PARSER.md) - Problem statement
- [tests/features/glr_runtime_integration.feature](../../tests/features/glr_runtime_integration.feature) - BDD scenarios
- [tablegen/src/schema.rs](../../tablegen/src/schema.rs) - Encoding contract
- [TEST_INVENTORY.md](../../TEST_INVENTORY.md) - Test tracking

---

**Let's build this! 🚀**
