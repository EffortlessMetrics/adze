# Parser_v4 Extraction Integration Design

**Status**: DESIGN
**Priority**: HIGH (Blocker for full GLR runtime wiring)
**Effort**: 4-6 hours
**Related**: GLR_RUNTIME_WIRING_PLAN.md, runtime/src/__private.rs:322

---

## 🎯 Problem Statement

GLR routing infrastructure (Steps 1-3) is complete, but full GLR integration is blocked by a type incompatibility in the extraction layer.

### Current State
```rust
// runtime/src/__private.rs
fn parse_with_glr<T: Extract<T>>(
    input: &str,
    language: impl Fn() -> &'static TSLanguage,
) -> Result<T, Vec<ParseError>> {
    // ✅ Can load parser: parser_v4::from_language(language())
    // ✅ Can parse: parser.parse(input)
    // ❌ Can't extract: parser_v4 returns Tree, need ParseNode

    // Fallback to pure_parser for now
    parse_with_pure_parser(input, language)
}
```

### The Issue

**parser_v4::parse() signature**:
```rust
pub fn parse(&mut self, input: &str) -> Result<Tree>

pub struct Tree {
    pub root_kind: u16,
    pub error_count: usize,
    pub source: String,
}
```

**Extract trait expects**:
```rust
pub trait Extract<Output> {
    fn extract(
        node: Option<&pure_parser::ParsedNode>,  // ← Need actual parse tree
        source: &[u8],
        last_idx: usize,
        leaf_fn: Option<&Self::LeafFn>,
    ) -> Output;
}
```

**parser_v4 DOES build a parse tree internally**:
```rust
// Inside parser_v4::parse()
let mut node_stack: Vec<ParseNode> = vec![];  // ← This is what we need!
// ... parsing logic builds tree ...
// But only returns minimal Tree struct
return Ok(Tree { root_kind, error_count, source });  // ← Lost the nodes!
```

---

## 🔧 Solution Options

### Option A: Modify parser_v4::parse() Return Type ⭐ RECOMMENDED

**Change**:
```rust
pub fn parse(&mut self, input: &str) -> Result<ParseNode>
```

**Pros**:
- Clean API - parse() returns what you'd expect
- Matches tree-sitter C API semantics
- Simple for users

**Cons**:
- Breaking change to parser_v4 API
- May affect other code using parser_v4 (low risk - limited usage)

**Implementation**:
1. Change return type in runtime/src/parser_v4.rs:578
2. Return root node from node_stack instead of Tree
3. Update any callers (likely none in production)

---

### Option B: Add parser_v4::parse_tree() Method

**Add**:
```rust
pub fn parse_tree(&mut self, input: &str) -> Result<ParseNode>
pub fn parse(&mut self, input: &str) -> Result<Tree>  // Keep existing
```

**Pros**:
- Non-breaking change
- Explicit API - users choose what they need

**Cons**:
- Two nearly identical methods
- May confuse users about which to use
- Code duplication in implementation

**Implementation**:
1. Add new parse_tree() method
2. Factor shared parsing logic into private helper
3. Keep parse() for backward compatibility

---

### Option C: Create ParseNode Conversion

**Add**:
```rust
impl From<parser_v4::ParseNode> for pure_parser::ParsedNode {
    fn from(node: parser_v4::ParseNode) -> Self {
        // Convert fields...
    }
}
```

**Pros**:
- No changes to parser_v4
- Clean type conversion

**Cons**:
- Requires parser_v4 to expose ParseNode (currently private in parse())
- Still need parser_v4::parse() to return the tree
- Just defers the problem

---

### Option D: Dual Extract Implementations

**Add**:
```rust
pub trait ExtractV4<Output> {
    fn extract_v4(
        node: Option<&parser_v4::ParseNode>,
        source: &[u8],
    ) -> Output;
}
```

**Pros**:
- Complete separation of concerns
- Each parser has its own extraction path

**Cons**:
- Duplicates extraction logic in generated code
- Macro complexity increases
- Maintenance burden

---

## 📋 Recommended Approach: Option A

Modify `parser_v4::parse()` to return `ParseNode`:

```rust
// File: runtime/src/parser_v4.rs

pub fn parse(&mut self, input: &str) -> Result<ParseNode> {
    // ... existing parsing logic ...

    // Instead of:
    // return Ok(Tree { root_kind, error_count, source });

    // Return the root node from node_stack:
    let root = node_stack
        .pop()
        .ok_or_else(|| anyhow!("Parse completed but node stack is empty"))?;

    Ok(root)
}
```

Then in `__private::parse_with_glr()`:
```rust
#[cfg(feature = "glr")]
fn parse_with_glr<T: Extract<T>>(
    input: &str,
    language: impl Fn() -> &'static TSLanguage,
) -> Result<T, Vec<ParseError>> {
    use crate::parser_v4::Parser;

    // Create parser from TSLanguage
    let mut parser = Parser::from_language(language(), "grammar".to_string());

    // Parse to get root ParseNode
    let root_node = parser.parse(input)
        .map_err(|e| vec![ParseError {
            reason: ParseErrorReason::UnexpectedToken(e.to_string()),
            start: 0,
            end: 0,
        }])?;

    // Convert parser_v4::ParseNode to pure_parser::ParsedNode
    let parsed_node = convert_parse_node(&root_node);

    // Extract typed AST
    Ok(<T as Extract<_>>::extract(
        Some(&parsed_node),
        input.as_bytes(),
        0,
        None,
    ))
}

fn convert_parse_node(node: &parser_v4::ParseNode) -> pure_parser::ParsedNode {
    pure_parser::ParsedNode {
        symbol: node.symbol.0,
        kind: "".to_string(),  // TODO: get from symbol table
        start_byte: node.start_byte,
        end_byte: node.end_byte,
        children: node.children.iter().map(convert_parse_node).collect(),
        field_name: node.field_name.clone(),
    }
}
```

---

## ✅ Acceptance Criteria

- [ ] parser_v4::parse() returns ParseNode (root of parse tree)
- [ ] ParseNode contains all necessary information for extraction
- [ ] Conversion from parser_v4::ParseNode to pure_parser::ParsedNode works
- [ ] parse_with_glr() successfully extracts typed AST
- [ ] All existing parser_v4 tests still pass
- [ ] Arithmetic grammar with GLR feature parses correctly

---

## 🧪 Test Strategy

### Unit Tests
```rust
#[test]
#[cfg(feature = "glr")]
fn test_parser_v4_returns_parse_node() {
    let lang = arithmetic::language();
    let mut parser = Parser::from_language(lang, "arithmetic".to_string());
    let root = parser.parse("1 + 2").unwrap();

    assert_eq!(root.symbol, /* expected symbol */);
    assert!(!root.children.is_empty());
}
```

### Integration Tests
```rust
#[test]
#[cfg(feature = "glr")]
fn test_glr_extraction() {
    use arithmetic::Expression;
    let result = arithmetic::parse("1 * 2 * 3").unwrap();

    // Should be left-associative: ((1 * 2) * 3)
    match result {
        Expression::Mul(box Expression::Mul(..), _, box Expression::Number(3)) => (),
        _ => panic!("Expected left-associative tree"),
    }
}
```

---

## 📅 Implementation Timeline

**Phase 1: Core Changes (2 hours)**
- Modify parser_v4::parse() return type
- Update parser_v4 internal logic
- Add basic conversion function

**Phase 2: Integration (2 hours)**
- Implement parse_with_glr() body
- Add ParseNode → ParsedNode conversion
- Wire up extraction

**Phase 3: Testing (2 hours)**
- Add unit tests for parser_v4 changes
- Add integration tests for GLR path
- Verify arithmetic grammar works

---

## 🔗 Related Files

- `runtime/src/parser_v4.rs:578` - parse() method to modify
- `runtime/src/__private.rs:322` - parse_with_glr() to implement
- `runtime/src/pure_parser.rs` - ParsedNode definition
- `runtime/src/lib.rs` - Extract trait definition

---

## 📝 Notes

- This blocker was discovered during Step 3 implementation
- GLR routing infrastructure (Steps 1-3) is complete
- Parse table generation is correct (glr-core works)
- This is purely a runtime integration issue, not an algorithm issue
- Fallback to pure_parser maintains current behavior while this is resolved

---

**Next Steps**: Implement Option A to unblock full GLR integration
