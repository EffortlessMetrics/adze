---
name: "Field Names - Wire them end-to-end"
about: "Implement field name propagation throughout the parse tree"
title: "[FEATURE] Field Names - Wire them end-to-end"
labels: "enhancement, priority-1, unblocks-queries"
assignees: ""
---

## Overview
Field-aware queries require node children to expose grammar field names. Currently `extract_field_name` always returns `None`.

## Implementation Checklist

### Data Model Changes
- [ ] Add field_id to internal child edge structure
```rust
// runtime/src/subtree.rs (or equivalent)
struct ChildEdge {
  node: SubtreeId,
  field_id: u16,  // 0xFFFF = NONE
}
```

### Reducer Hook
- [ ] Wire field map data during reduction in `runtime/src/parser_v4.rs`
```rust
// During reduction with prod_id: ProductionId
// 1) find slice: lang.field_map_slices[prod_id] -> (start, len)
// 2) for each entry in lang.field_map_entries[start..start+len]:
//      entries: { field_id: u16, child_index: u16 }
for e in entries { 
  children[e.child_index as usize].field_id = e.field_id; 
}
```

### Public API
- [ ] Update `runtime/src/node.rs` with field methods
```rust
impl Node {
  pub fn field_name(&self) -> Option<&'static str> { 
    // from language.field_names[self.field_id] 
  }
  pub fn child_by_field_name(&self, name: &str) -> Option<Node> { 
    // scan children once 
  }
}
```

### Language Metadata
- [ ] Expose field data in `runtime/src/language.rs`
  - field_map_entries
  - field_map_slices  
  - field_names

### Fix Stub
- [ ] Replace stub in `runtime/src/pure_parser.rs:1197`
```rust
fn extract_field_name(_subtree: &Subtree, _language: Option<*const TSLanguage>) -> Option<String> {
    // TODO: Implement using field_id from subtree
}
```

## Tests

### Smoke Test
- [ ] Create `runtime/tests/fields_smoke.rs`
```rust
// Grammar: Fn -> "fn" name:IDENT params:ParamList body:Block
#[test]
fn test_field_names() {
    let fn_node = parse("fn foo() {}");
    assert_eq!(
        fn_node.child_by_field_name("name").unwrap().kind(), 
        "identifier"
    );
}
```

### Golden Tests
- [ ] JSON output shows `"field":"name"`
- [ ] S-expression output shows field annotations

### Query Integration (after completion)
- [ ] Pattern `(function_definition name:(identifier) @cap)` captures exactly the identifier

## Acceptance Criteria
- [x] Field-aware queries pass
- [x] Node API stable  
- [x] No performance regression on child iteration
- [x] All field-related tests pass

## Files to Modify
- `runtime/src/subtree.rs` - Add field_id to child edges
- `runtime/src/parser_v4.rs` - Wire field map during reduction
- `runtime/src/node.rs` - Add field_name API methods
- `runtime/src/language.rs` - Expose field metadata
- `runtime/src/pure_parser.rs:1197` - Fix extract_field_name stub
- `runtime/tests/fields_smoke.rs` - New test file

## Risk Notes
Changes memory layout of edges → keep as `u16` to avoid bloat; gate with feature until stable.