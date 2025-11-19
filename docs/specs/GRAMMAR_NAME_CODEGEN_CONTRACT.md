# GRAMMAR_NAME Code Generation Contract

**Status**: SPECIFICATION (Not Yet Implemented)
**Date**: 2025-11-19
**Related**: [PRODUCTION_READINESS_ROADMAP.md](../PRODUCTION_READINESS_ROADMAP.md) Phase 1
**Requirement**: External scanner support in GLR runtime

---

## Overview

This specification defines the contract for emitting `GRAMMAR_NAME` in generated `Extract` trait implementations. The grammar name is used to look up external scanners in the scanner registry during GLR parsing.

### Purpose

- Enable external scanner registration by grammar name
- Support language-specific scanners (e.g., Python indentation, Ruby heredocs)
- Maintain consistency between macro-annotated name and runtime lookup

### Scope

- **In Scope**: Emitting `GRAMMAR_NAME` const in generated code
- **In Scope**: Extracting name from `#[rust_sitter::grammar("name")]` attribute
- **In Scope**: Validation that name is a valid string literal
- **Out of Scope**: External scanner implementation itself
- **Out of Scope**: Scanner registration mechanism

---

## Contract Definition

### 1. Input Contract

#### 1.1 Grammar Attribute Format

User code MUST use the `#[rust_sitter::grammar("name")]` attribute:

```rust
#[rust_sitter::grammar("python")]
mod python_grammar {
    // grammar definition
}
```

**Constraints**:
- Attribute MUST contain exactly one string literal argument
- Name MUST be a valid UTF-8 string
- Name SHOULD be lowercase alphanumeric with underscores (convention)
- Name MUST NOT be empty

**Invalid Examples**:
```rust
#[rust_sitter::grammar()]           // Error: missing name
#[rust_sitter::grammar(python)]     // Error: not a string literal
#[rust_sitter::grammar("")]         // Error: empty name
#[rust_sitter::grammar("a", "b")]   // Error: multiple arguments
```

---

### 2. Output Contract

#### 2.1 Generated Code Format

For a grammar annotated with `#[rust_sitter::grammar("my_grammar")]`, the generated code MUST include:

```rust
impl ::rust_sitter::Extract<Self> for MyGrammarRoot {
    type LeafFn = /* ... */;

    const HAS_CONFLICTS: bool = /* computed */;

    #[cfg(feature = "pure-rust")]
    const GRAMMAR_NAME: &'static str = "my_grammar";

    #[cfg(feature = "pure-rust")]
    const GRAMMAR_JSON: &'static str = /* generated */;

    // ... other trait items
}
```

**Requirements**:
- `GRAMMAR_NAME` MUST be a `&'static str` const
- Value MUST match the string literal from the attribute exactly
- MUST be gated with `#[cfg(feature = "pure-rust")]` to match trait definition
- MUST appear before `GRAMMAR_JSON` (for consistency)

---

#### 2.2 Multiple Grammar Modules

If multiple grammar modules exist in the same crate, each MUST have its own name:

```rust
#[rust_sitter::grammar("python")]
mod python { /* ... */ }

#[rust_sitter::grammar("javascript")]
mod javascript { /* ... */ }
```

Generated code:
```rust
// In python module
const GRAMMAR_NAME: &'static str = "python";

// In javascript module
const GRAMMAR_NAME: &'static str = "javascript";
```

No name collision checking required (Rust's module system handles this).

---

### 3. Error Handling Contract

#### 3.1 Missing Attribute

If `#[rust_sitter::grammar(...)]` is missing:

```rust
// Error: "Each grammar module must have a #[rust_sitter::grammar(\"name\")] attribute"
mod some_grammar { /* ... */ }
```

**Action**: Emit compile-time error via `proc_macro::Diagnostic` or `panic!`.

---

#### 3.2 Invalid Attribute Format

If attribute argument is not a string literal:

```rust
#[rust_sitter::grammar(python)]  // Missing quotes
```

**Action**: Emit error: "Expected string literal for grammar name, got: <type>"

---

#### 3.3 Empty Name

If string literal is empty:

```rust
#[rust_sitter::grammar("")]
```

**Action**: Emit error: "Grammar name cannot be empty"

---

### 4. Integration Points

#### 4.1 Macro Path Integration

**File**: `macro/src/expansion.rs`

**Current Code** (lines 255-273):
```rust
let grammar_name = input
    .attrs
    .iter()
    .find_map(|a| {
        if a.path() == &syn::parse_quote!(rust_sitter::grammar) {
            let grammar_name_expr = a.parse_args_with(Expr::parse).ok();
            if let Some(Expr::Lit(ExprLit {
                attrs: _,
                lit: Lit::Str(s),
            })) = grammar_name_expr
            {
                Some(Ok(s.value()))
            } else {
                Some(Err(syn::Error::new(
                    Span::call_site(),
                    "Expected string literal for grammar name",
                )))
            }
        } else {
            None
        }
    })
    .transpose()?
    .expect("Each grammar must have a name");
```

**Required Change**: Emit `GRAMMAR_NAME` const in generated output.

**Implementation Location**: After line 550 where `language()` function is generated.

---

#### 4.2 Tool Path Integration

**File**: `tool/src/pure_rust_builder.rs`

**Current Code** (lines 260-266):
```rust
// Extract grammar name from JSON
let grammar_name = grammar_value
    .get("name")
    .and_then(|v| v.as_str())
    .unwrap_or("unknown");
```

**Required Change**: Emit `GRAMMAR_NAME` in generated parser file.

**Implementation Location**: In generated parser module around line 600.

---

### 5. Test Contracts

#### 5.1 Unit Tests

**Test**: Grammar name extraction from attribute

```rust
#[test]
fn test_grammar_name_extraction() {
    let input = quote! {
        #[rust_sitter::grammar("test_lang")]
        mod grammar {}
    };

    let name = extract_grammar_name(&input);
    assert_eq!(name, "test_lang");
}
```

---

#### 5.2 Integration Tests

**Test**: Generated code includes GRAMMAR_NAME

```rust
#[test]
fn test_grammar_name_in_generated_code() {
    #[rust_sitter::grammar("integration_test")]
    mod test_grammar {
        #[rust_sitter::language]
        struct Root;
    }

    // Verify const exists and has correct value
    assert_eq!(test_grammar::Root::GRAMMAR_NAME, "integration_test");
}
```

---

#### 5.3 End-to-End Tests

**Test**: External scanner lookup uses GRAMMAR_NAME

```rust
#[test]
#[cfg(feature = "glr")]
fn test_external_scanner_lookup_by_name() {
    use rust_sitter::scanner_registry::get_global_registry;

    #[rust_sitter::grammar("scanner_test")]
    mod grammar {
        // Grammar with external scanner
    }

    // Register scanner
    let registry = get_global_registry();
    registry.lock().unwrap()
        .register("scanner_test", create_scanner);

    // Parse should find scanner by GRAMMAR_NAME
    let result = grammar::parse("test input");
    assert!(result.is_ok());
}
```

---

### 6. Backward Compatibility

#### 6.1 Existing Grammars Without GRAMMAR_NAME

**Scenario**: Grammars compiled before GRAMMAR_NAME was added.

**Behavior**:
- Compilation will use default value from trait: `"unknown"`
- External scanners will NOT be found
- No compilation errors

**Migration Path**:
1. User updates rust-sitter dependency
2. Re-compiles grammar (code generation runs)
3. GRAMMAR_NAME automatically emitted
4. External scanners work

**Impact**: Zero breaking changes for users without external scanners.

---

#### 6.2 Tree-sitter Backend

**Scenario**: User compiles with default (tree-sitter) backend.

**Behavior**:
- `GRAMMAR_NAME` is gated with `#[cfg(feature = "pure-rust")]`
- Tree-sitter path doesn't use it
- No code size impact

**Validation**: Ensure `#[cfg(...)]` gates match between trait and impl.

---

### 7. Implementation Checklist

#### Phase 1: Specification ✅
- [x] Define input contract
- [x] Define output contract
- [x] Define error handling
- [x] Identify integration points
- [x] Write test contracts

#### Phase 2: Implementation (Pending)
- [ ] Add GRAMMAR_NAME emission to macro/src/expansion.rs
- [ ] Add GRAMMAR_NAME emission to tool/src/pure_rust_builder.rs (if needed)
- [ ] Implement error validation
- [ ] Add unit tests

#### Phase 3: Validation (Pending)
- [ ] Write integration tests
- [ ] Write end-to-end tests
- [ ] Test with example grammars
- [ ] Verify external scanner lookup works

#### Phase 4: Documentation (Pending)
- [ ] Update Extract trait rustdoc with example
- [ ] Add GRAMMAR_NAME to grammar definition tutorial
- [ ] Document in migration guide (if breaking)

---

### 8. Success Criteria

**Acceptance Criteria**:
1. Generated code includes `const GRAMMAR_NAME: &'static str = "..."`
2. Value matches `#[rust_sitter::grammar("...")]` attribute
3. GLR parser can look up external scanners by name
4. All tests pass
5. No regressions in existing grammars

**Validation**:
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Example grammars compile
- [ ] Python grammar (with external scanner) works in GLR mode
- [ ] CI green

---

### 9. Alternative Designs Considered

#### Alt 1: Runtime Grammar Name Resolution

**Idea**: Look up name from TSLanguage struct at runtime.

**Rejected**: TSLanguage doesn't expose name in a standard way; would require parsing embedded strings.

---

#### Alt 2: Procedural Macro Attribute on Root Type

**Idea**: `#[grammar_name = "python"]` on language root struct.

**Rejected**: Duplicates information already in module-level attribute; inconsistent with current design.

---

#### Alt 3: Default to Module Name

**Idea**: If no name specified, use module name.

**Rejected**: Module names can be arbitrary; explicit naming is clearer for registry lookup.

---

### 10. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Name collision in registry | Low | Medium | Document that names must be globally unique |
| Codegen breaks existing grammars | Low | High | Default value in trait prevents breakage |
| Macro/tool inconsistency | Medium | High | Integration tests verify both paths |
| Missing name validation | Low | Medium | Compile-time error on invalid format |

---

## Appendix A: Example Generated Code

### Input

```rust
#[rust_sitter::grammar("python")]
mod python {
    #[rust_sitter::language]
    enum Stmt {
        // ...
    }
}
```

### Output (Macro Path)

```rust
#[cfg(feature = "pure-rust")]
mod python {
    use ::rust_sitter::Extract;

    // ... generated types ...

    impl Extract<Self> for Stmt {
        type LeafFn = /* ... */;

        const HAS_CONFLICTS: bool = true;

        #[cfg(feature = "pure-rust")]
        const GRAMMAR_NAME: &'static str = "python";

        #[cfg(feature = "pure-rust")]
        const GRAMMAR_JSON: &'static str = r#"{ ... }"#;

        // ... trait methods ...
    }

    // ... language() function ...
}
```

### Output (Tool Path)

```rust
// File: target/debug/build/.../out/grammar_python/parser_python.rs

pub static GRAMMAR_NAME: &str = "python";

pub fn language() -> &'static ::rust_sitter::pure_parser::TSLanguage {
    // ... existing language struct generation ...
}
```

---

## Appendix B: Related Contracts

- [Extract Trait Contract](../../runtime/src/lib.rs:215-283)
- [Scanner Registry Contract](../../runtime/src/scanner_registry.rs)
- [GLR Runtime Wiring Plan](../plans/GLR_RUNTIME_WIRING_PLAN.md)
- [Phase 1 Completion Report](../status/PHASE_1_COMPLETION.md)

---

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-11-19 | 1.0 | Initial specification | Claude |

---

**Status**: Ready for Implementation
**Next Steps**: Implement in macro/src/expansion.rs following this contract
