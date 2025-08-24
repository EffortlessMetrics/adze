# Pure-Rust Backend Completion Roadmap

## Current Status
✅ **Precedence bug fixed** - Enum variants now correctly propagate precedence annotations
❌ **Pure-rust backend failing** - Token ID mismatches prevent any parsing

## Critical Path to Green Tests

### Phase 1: Fix Lexer Token Generation (Priority: CRITICAL)
The lexer is generating tokens but with wrong symbol IDs. The parser expects non-terminals in state 0, but that's impossible.

**Root Cause**: The parse table generation is including non-terminals in positions where only terminals should be.

#### 1.1 Fix Symbol Classification
- [ ] Ensure only terminals appear in lexer generation
- [ ] Properly separate terminals from non-terminals in symbol_to_index map
- [ ] Fix the parse table generation to use correct symbol IDs

#### 1.2 Fix Token Ordering
- [ ] Check keywords BEFORE generic patterns (identifier, number)
- [ ] Implement proper keyword reservation system
- [ ] Order tokens by specificity (longest match first, then most specific)

#### 1.3 Handle Whitespace as Extra
- [ ] Detect #[rust_sitter::extra] annotations in grammar
- [ ] Mark whitespace tokens with proper metadata
- [ ] Skip extra tokens in parser main loop

### Phase 2: Fix Symbol Table Synchronization (Priority: HIGH)

#### 2.1 Unified Symbol Numbering
- [ ] Create single source of truth for symbol->index mapping
- [ ] Ensure lexer and parser use same symbol IDs
- [ ] Add validation that all referenced symbols exist

#### 2.2 Debug Infrastructure
- [ ] Add symbol table dump functionality
- [ ] Create lexer/parser symbol mismatch detector
- [ ] Add comprehensive logging for symbol resolution

### Phase 3: Complete Parser Features (Priority: MEDIUM)

#### 3.1 Query Predicates (#14)
- [ ] Parse query syntax with predicates
- [ ] Implement predicate evaluation engine
- [ ] Support eq?, not-eq?, match?, not-match?
- [ ] Add capture name validation

#### 3.2 External Scanner Support (#15)
- [ ] Add column tracking to lexer state
- [ ] Implement external scanner FFI interface
- [ ] Support scanner state serialization
- [ ] Handle scanner-produced tokens correctly

#### 3.3 Alias Support (#16)
- [ ] Parse alias annotations in grammar
- [ ] Generate alias mappings in tables
- [ ] Update node types generation
- [ ] Handle aliased nodes in queries

### Phase 4: Advanced Features (Priority: LOW)

#### 4.1 Incremental Parsing (#17)
- [ ] Implement tree edit operations
- [ ] Add subtree reuse logic
- [ ] Support byte range updates
- [ ] Optimize for minimal reparsing

## Implementation Order

### Week 1: Get Basic Parsing Working
1. **Day 1-2**: Fix symbol classification
   - Separate terminals/non-terminals properly
   - Fix parse table generation
   
2. **Day 3-4**: Fix lexer ordering
   - Implement keyword detection
   - Fix token precedence
   
3. **Day 5-6**: Fix whitespace handling
   - Detect extra tokens
   - Update parser to skip extras
   
4. **Day 7**: Test and validate
   - Run python-simple tests
   - Fix any remaining issues

### Week 2: Parser Completeness
1. **Day 1-3**: Query predicates
2. **Day 4-5**: External scanner column tracking  
3. **Day 6-7**: Alias support

### Week 3: Production Ready
1. **Day 1-5**: Incremental parsing
2. **Day 6-7**: Performance optimization and testing

## Test Strategy

### Level 1: Basic Parsing
```rust
// These MUST pass first
test_parse_number()      // "42"
test_parse_identifier()  // "foo"
test_parse_expression()  // "a + b"
```

### Level 2: Precedence Tests
```rust
// Validate our precedence fix
test_unary_precedence()     // "-a.b" -> -(a.b)
test_binary_precedence()    // "a + b * c" -> a + (b * c)
test_call_precedence()      // "a + b(c)" -> a + (b(c))
```

### Level 3: Complex Features
```rust
// Advanced functionality
test_error_recovery()
test_query_evaluation()
test_external_scanner()
test_incremental_parse()
```

## Success Metrics

1. **All python-simple tests pass** with pure-rust backend
2. **Performance parity** with C backend (±10%)
3. **100% grammar compatibility** with Tree-sitter
4. **Zero memory leaks** in parsing

## Known Challenges

1. **Symbol Table Complexity**: Tree-sitter's symbol numbering is complex
2. **Keyword Handling**: Must match Tree-sitter's keyword detection exactly
3. **Extra Token Behavior**: Whitespace/comment handling is subtle
4. **State Machine Compatibility**: Parse tables must match Tree-sitter format

## Debugging Tools Needed

1. **Symbol Table Viewer**: Show all symbols and their IDs
2. **Lexer Trace**: Log every token produced with symbol ID
3. **Parser Trace**: Show state transitions and symbol expectations
4. **Table Dumper**: Export parse tables in readable format

## Next Immediate Steps

1. Add debug logging to understand symbol table state
2. Fix terminal/non-terminal classification 
3. Update lexer generation to only include terminals
4. Test with simplest possible input ("42")