# Phase 3.2: Tokenization & Forest-to-Tree Conversion

**Status**: In Progress
**Dependencies**: Phase 3.1 Complete ✅
**Objective**: Implement lexical scanner and forest-to-tree conversion for production-ready GLR parsing
**Timeline**: 4-6 days (estimated)
**Contract Version**: 1.0.0
**Date**: 2025-01-19

---

## Executive Summary

Phase 3.1 delivered a working GLR engine with Parser API integration, but uses stub implementations for:
1. ❌ **Tokenization**: Stub tokenizer only produces EOF token
2. ❌ **Forest Conversion**: Returns `Tree::new_stub()` without actual tree construction

**Phase 3.2 Goal**: Complete the GLR parsing pipeline with production-ready tokenization and forest-to-tree conversion, enabling parsing of real grammars with real inputs.

---

## Current State Analysis

### Phase 3.1 Accomplishments ✅
- GLR engine structure implemented (`GLREngine`, `ParserStack`, `ParseForest`)
- Fork/merge logic working (borrow checker issues resolved)
- Parser API routing to GLR engine when in GLR mode
- 10/10 tests passing (6 API + 4 integration)
- Contract specifications complete

### Phase 3.1 Limitations (TODOs)
```rust
// runtime2/src/parser.rs:parse_glr() - Line 193
// TODO: Tokenize input (Phase 3.2)
let tokens = vec![crate::Token {
    kind: 0, // EOF
    start: input.len() as u32,
    end: input.len() as u32,
}];

// runtime2/src/parser.rs:parse_glr() - Line 207
// TODO: Convert forest to Tree (Phase 3.3)
let tree = Tree::new_stub();
```

### Phase 3.2 Requirements

**Must Have**:
1. ✅ Lexical scanner that tokenizes input according to grammar
2. ✅ Forest-to-tree converter that builds valid Tree from ParseForest
3. ✅ Disambiguation strategy (prefer-shift as default)
4. ✅ Integration with existing Token and Tree APIs
5. ✅ Comprehensive test coverage (unit + integration)

**Should Have**:
- Multiple disambiguation strategies (configurable)
- Performance optimization (lazy evaluation)
- Error recovery for invalid tokens

**Could Have**:
- External scanner support for context-sensitive tokens
- Incremental tokenization
- Token caching

---

## Architecture Overview

### Current GLR Pipeline (Phase 3.1)
```
User Input (bytes)
    ↓
[STUB] Tokenizer → [EOF token only]
    ↓
GLREngine::parse()
    ↓
ParseForest (roots: Vec<ForestNodeId>)
    ↓
[STUB] Tree::new_stub() → [Empty tree]
    ↓
User Code
```

### Proposed GLR Pipeline (Phase 3.2)
```
User Input (bytes)
    ↓
Tokenizer::scan(input, grammar) ← NEW
    ↓
Vec<Token> {kind, start, end}
    ↓
GLREngine::parse(&tokens)
    ↓
ParseForest {
    nodes: Vec<ForestNode>,
    roots: Vec<ForestNodeId>
}
    ↓
ForestConverter::to_tree(forest, strategy) ← NEW
    ↓
Tree {
    root_node: Node,
    source: Vec<u8>,
    language: Language
}
    ↓
User Code
```

---

## Component 1: Lexical Scanner (Tokenizer)

### 1.1 Contract Specification

#### Data Structures

```rust
/// Tokenizer scans input and produces tokens according to grammar
///
/// Contract:
/// - Thread-safe (Send + Sync)
/// - Deterministic (same input → same tokens)
/// - Complete coverage (no input bytes skipped)
/// - Position tracking (byte offsets and line/column)
///
pub struct Tokenizer {
    /// Token patterns from grammar (regex or literal)
    patterns: Vec<TokenPattern>,
    /// Token precedence (for disambiguation)
    precedence: Vec<usize>,
    /// Whitespace handling mode
    whitespace_mode: WhitespaceMode,
}

/// Token pattern from grammar
#[derive(Debug, Clone)]
pub struct TokenPattern {
    /// Symbol ID from grammar
    symbol_id: SymbolId,
    /// Pattern matcher (regex or literal string)
    matcher: Matcher,
    /// Is this a keyword or identifier?
    is_keyword: bool,
}

/// Pattern matching strategy
#[derive(Debug, Clone)]
pub enum Matcher {
    /// Literal string match (exact)
    Literal(String),
    /// Regex pattern match
    Regex(regex::Regex),
    /// External scanner (context-sensitive)
    External(ExternalScannerId),
}

/// Whitespace handling strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhitespaceMode {
    /// Skip whitespace (most common)
    Skip,
    /// Preserve whitespace as tokens
    Preserve,
    /// Whitespace is significant in context
    Significant,
}
```

#### API Contract

```rust
impl Tokenizer {
    /// Create tokenizer from grammar
    ///
    /// # Contract
    ///
    /// ## Preconditions
    /// - `grammar` must have at least 1 token defined
    /// - All regex patterns must be valid
    /// - Token symbol IDs must be unique
    ///
    /// ## Postconditions
    /// - Tokenizer ready to scan input
    /// - Patterns sorted by precedence (longest match first)
    /// - Whitespace mode set according to grammar
    ///
    /// ## Errors
    /// - `TokenizerError::NoTokens`: Grammar has no tokens
    /// - `TokenizerError::InvalidRegex`: Regex compilation failed
    /// - `TokenizerError::DuplicateSymbol`: Symbol ID collision
    ///
    pub fn from_grammar(grammar: &Grammar) -> Result<Self, TokenizerError>;

    /// Scan input and produce tokens
    ///
    /// # Contract
    ///
    /// ## Preconditions
    /// - `input` is valid UTF-8 or raw bytes
    /// - Tokenizer initialized from grammar
    ///
    /// ## Postconditions
    /// - All input bytes covered (no gaps)
    /// - Tokens in order (sorted by start position)
    /// - Last token is EOF with position at input.len()
    ///
    /// ## Invariants
    /// - For all tokens: token[i].end == token[i+1].start (no gaps/overlaps)
    /// - EOF token always present: tokens.last().kind == 0
    ///
    /// ## Errors
    /// - `TokenizerError::InvalidToken`: Unrecognized character sequence
    /// - `TokenizerError::InvalidUtf8`: Input is not valid UTF-8 (if required)
    ///
    /// ## Performance
    /// - Time: O(n * m) where n = input.len(), m = patterns.len()
    /// - Space: O(t) where t = token count (typically n/5)
    ///
    pub fn scan(&self, input: &[u8]) -> Result<Vec<Token>, TokenizerError>;

    /// Scan incrementally (for large inputs)
    ///
    /// # Contract
    ///
    /// ## Preconditions
    /// - `input` chunk is valid
    /// - `offset` is the byte position of this chunk in full input
    ///
    /// ## Postconditions
    /// - Tokens relative to `offset`
    /// - May request more input if pattern incomplete at end
    ///
    pub fn scan_incremental(
        &self,
        input: &[u8],
        offset: usize,
    ) -> Result<ScanResult, TokenizerError>;
}

/// Result of incremental scanning
#[derive(Debug)]
pub struct ScanResult {
    /// Tokens scanned so far
    pub tokens: Vec<Token>,
    /// Number of bytes consumed
    pub consumed: usize,
    /// More input needed?
    pub needs_more: bool,
}
```

### 1.2 Algorithm Specification

#### Maximal Munch (Longest Match) Algorithm

```
Algorithm: Tokenizer::scan(input)

Input: input: &[u8] - byte slice to tokenize
Output: Vec<Token> - sequence of tokens

State:
  position: usize = 0
  tokens: Vec<Token> = []

Loop while position < input.len():
  best_match: Option<(SymbolId, usize)> = None  // (symbol, length)

  // Try all patterns at current position
  For each pattern in self.patterns:
    match_len = pattern.match_at(input, position)

    If match_len > 0:
      // Prefer longer matches (maximal munch)
      If best_match.is_none() OR match_len > best_match.1:
        best_match = Some((pattern.symbol_id, match_len))

      // If same length, use precedence
      Else If match_len == best_match.1:
        If pattern.precedence > current_precedence:
          best_match = Some((pattern.symbol_id, match_len))

  // Apply best match or error
  If let Some((symbol_id, length)) = best_match:
    // Skip whitespace if configured
    If symbol_id.is_whitespace() AND self.whitespace_mode == Skip:
      position += length
      continue

    // Create token
    token = Token {
      kind: symbol_id.0 as u32,
      start: position as u32,
      end: (position + length) as u32,
    }
    tokens.push(token)
    position += length
  Else:
    // No pattern matched - error
    return Err(TokenizerError::InvalidToken { position })

// Append EOF token
tokens.push(Token {
  kind: 0,  // EOF
  start: input.len() as u32,
  end: input.len() as u32,
})

return Ok(tokens)
```

#### Pattern Matching Priority

1. **Longest Match**: Prefer patterns that match more characters
2. **Precedence**: If tied, use pattern precedence from grammar
3. **Order**: If still tied, use pattern definition order

**Example**:
```
Input: "ifx"
Patterns:
  1. "if" (keyword)
  2. [a-z]+ (identifier)

Result: "ifx" matches as identifier (longer match wins)

Input: "if "
Patterns:
  1. "if" (keyword, precedence=10)
  2. [a-z]+ (identifier, precedence=5)

Result: "if" matches as keyword (same length, higher precedence)
```

### 1.3 Error Handling

```rust
/// Tokenizer errors
#[derive(Debug, thiserror::Error)]
pub enum TokenizerError {
    #[error("No tokens defined in grammar")]
    NoTokens,

    #[error("Invalid regex pattern '{pattern}': {error}")]
    InvalidRegex {
        pattern: String,
        error: regex::Error,
    },

    #[error("Duplicate symbol ID: {symbol_id}")]
    DuplicateSymbol { symbol_id: u16 },

    #[error("Invalid token at position {position}: '{snippet}'")]
    InvalidToken {
        position: usize,
        snippet: String,  // Up to 20 chars of context
    },

    #[error("Invalid UTF-8 at position {position}")]
    InvalidUtf8 { position: usize },
}
```

### 1.4 Testing Strategy

#### Unit Tests

```rust
#[cfg(test)]
mod tokenizer_tests {
    #[test]
    fn test_literal_tokens() {
        // Test: "1 + 2" → [NUM, PLUS, NUM, EOF]
    }

    #[test]
    fn test_regex_tokens() {
        // Test: Identifiers, numbers, strings
    }

    #[test]
    fn test_maximal_munch() {
        // Test: "ifx" matches as identifier, not "if" + "x"
    }

    #[test]
    fn test_keyword_vs_identifier() {
        // Test: "if" is keyword, "ifx" is identifier
    }

    #[test]
    fn test_whitespace_handling() {
        // Test: Whitespace skipped correctly
    }

    #[test]
    fn test_error_invalid_token() {
        // Test: "@#$" produces InvalidToken error
    }

    #[test]
    fn test_eof_token() {
        // Test: EOF always present at end
    }

    #[test]
    fn test_token_positions() {
        // Test: No gaps or overlaps in token positions
    }
}
```

#### Integration Tests

```rust
#[cfg(test)]
mod tokenizer_integration {
    #[test]
    fn test_arithmetic_expression() {
        // Grammar: expr → NUMBER | expr + expr
        // Input: "1 + 2 + 3"
        // Expected: [NUM(1), PLUS, NUM(2), PLUS, NUM(3), EOF]
    }

    #[test]
    fn test_nested_parens() {
        // Grammar: expr → ( expr ) | NUMBER
        // Input: "((1))"
        // Expected: [LPAREN, LPAREN, NUM(1), RPAREN, RPAREN, EOF]
    }

    #[test]
    fn test_unicode_input() {
        // Grammar: ident → [α-ω]+
        // Input: "αβγ"
        // Expected: [IDENT(αβγ), EOF]
    }
}
```

---

## Component 2: Forest-to-Tree Conversion

### 2.1 Contract Specification

#### Data Structures

```rust
/// Converts ParseForest to single Tree
///
/// Contract:
/// - Selects one parse tree from potentially multiple valid parses
/// - Applies disambiguation strategy
/// - Preserves all node metadata (symbols, ranges, visibility)
///
pub struct ForestConverter {
    /// Disambiguation strategy
    strategy: DisambiguationStrategy,
    /// Symbol metadata from grammar
    symbol_metadata: Vec<SymbolMetadata>,
}

/// Disambiguation strategies for ambiguous parses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisambiguationStrategy {
    /// Prefer shift over reduce (Tree-sitter default)
    PreferShift,
    /// Prefer reduce over shift
    PreferReduce,
    /// Use precedence declarations from grammar
    Precedence,
    /// Take first valid parse (fast but arbitrary)
    First,
    /// Return error on ambiguity
    RejectAmbiguity,
}

/// Forest node representation
#[derive(Debug, Clone)]
pub enum ForestNode {
    /// Terminal node (token from input)
    Terminal {
        symbol: SymbolId,
        range: Range<usize>,
    },
    /// Nonterminal node (reduction result)
    Nonterminal {
        symbol: SymbolId,
        children: Vec<ForestNodeId>,
        rule_id: RuleId,
    },
    /// Packed node (multiple derivations for same span)
    Packed {
        alternatives: Vec<ForestNodeId>,
    },
}
```

#### API Contract

```rust
impl ForestConverter {
    /// Create converter with strategy
    ///
    /// # Contract
    ///
    /// ## Preconditions
    /// - `symbol_metadata` matches grammar symbols
    /// - Strategy is valid
    ///
    /// ## Postconditions
    /// - Converter ready to convert forests
    ///
    pub fn new(
        strategy: DisambiguationStrategy,
        symbol_metadata: Vec<SymbolMetadata>,
    ) -> Self;

    /// Convert ParseForest to Tree
    ///
    /// # Contract
    ///
    /// ## Preconditions
    /// - `forest.roots` is non-empty (at least one valid parse)
    /// - Forest nodes form valid tree structure (no cycles)
    /// - All ForestNodeIds reference valid nodes
    ///
    /// ## Postconditions
    /// - Tree has single root node
    /// - All visible nodes included in tree
    /// - Node ranges are consistent (parent contains all children)
    /// - Tree structure matches grammar rules
    ///
    /// ## Invariants
    /// - Root node covers entire input (range: 0..input.len())
    /// - For all nodes: node.start() <= child.start() < child.end() <= node.end()
    /// - Symbol visibility preserved (invisible nodes hidden)
    ///
    /// ## Errors
    /// - `ConversionError::NoRoots`: Forest has no root nodes
    /// - `ConversionError::AmbiguousForest`: Multiple valid parses (if strategy = RejectAmbiguity)
    /// - `ConversionError::InvalidForest`: Forest structure is malformed
    ///
    /// ## Performance
    /// - Time: O(n) where n = forest node count (single traversal)
    /// - Space: O(d) where d = tree depth (stack for DFS)
    ///
    pub fn to_tree(
        &self,
        forest: &ParseForest,
        input: &[u8],
    ) -> Result<Tree, ConversionError>;

    /// Detect ambiguity in forest
    ///
    /// # Contract
    ///
    /// ## Returns
    /// - `None`: Forest is unambiguous (single parse)
    /// - `Some(count)`: Forest has `count` alternative parses
    ///
    pub fn detect_ambiguity(&self, forest: &ParseForest) -> Option<usize>;

    /// Extract all parse trees (for debugging/analysis)
    ///
    /// # Contract
    ///
    /// ## Returns
    /// - Vector of all valid parse trees
    /// - Empty if forest is empty
    ///
    pub fn all_trees(&self, forest: &ParseForest) -> Vec<Tree>;
}
```

### 2.2 Algorithm Specification

#### Forest Traversal Algorithm

```
Algorithm: ForestConverter::to_tree(forest, input)

Input:
  forest: &ParseForest - parse forest with multiple roots
  input: &[u8] - original input bytes

Output: Tree - single parse tree

Phase 1: Disambiguation (Select Root)
  If forest.roots.is_empty():
    return Err(ConversionError::NoRoots)

  If forest.roots.len() == 1:
    selected_root = forest.roots[0]
  Else:
    // Multiple roots - apply disambiguation
    selected_root = self.disambiguate_roots(&forest.roots, forest)

Phase 2: Tree Construction (DFS Traversal)
  root_node = self.build_node(selected_root, forest, input)

  tree = Tree {
    root: root_node,
    source: input.to_vec(),
    language: None,  // Set by caller
  }

  return Ok(tree)

Helper: build_node(node_id, forest, input) -> Node
  forest_node = &forest.nodes[node_id.0]

  match forest_node:
    ForestNode::Terminal { symbol, range }:
      return Node::new_terminal(
        symbol,
        input[range.clone()],
        range.start,
      )

    ForestNode::Nonterminal { symbol, children, rule_id }:
      // Recursively build child nodes
      child_nodes = children.map(|child_id| {
        self.build_node(child_id, forest, input)
      })

      // Filter invisible nodes (like parentheses)
      visible_children = child_nodes.filter(|node| {
        self.symbol_metadata[node.symbol()].is_visible
      })

      return Node::new_nonterminal(
        symbol,
        visible_children,
        rule_id,
      )

    ForestNode::Packed { alternatives }:
      // Ambiguity - select one alternative
      selected = self.disambiguate_alternatives(alternatives, forest)
      return self.build_node(selected, forest, input)

Helper: disambiguate_alternatives(alternatives, forest) -> ForestNodeId
  match self.strategy:
    PreferShift:
      // Prefer alternatives from shift actions
      return alternatives.iter()
        .find(|alt| forest.nodes[alt.0].is_shift_derived())
        .unwrap_or(alternatives[0])

    Precedence:
      // Use grammar precedence declarations
      return alternatives.iter()
        .max_by_key(|alt| forest.nodes[alt.0].precedence())
        .unwrap()

    First:
      return alternatives[0]

    RejectAmbiguity:
      return Err(ConversionError::AmbiguousForest {
        count: alternatives.len()
      })
```

### 2.3 Disambiguation Strategies

#### 1. Prefer-Shift (Default)

**Rationale**: Tree-sitter's default strategy. Shift actions tend to create right-associative trees.

**Example**:
```
Input: "1 + 2 + 3"
Grammar: expr → expr + expr (ambiguous)

Parses:
  A: ((1 + 2) + 3)  ← Left-associative (reduce first)
  B: (1 + (2 + 3))  ← Right-associative (shift first)

Strategy: PreferShift selects B
```

#### 2. Precedence-Based

**Rationale**: Use explicit precedence declarations from grammar.

**Example**:
```
Grammar:
  expr → expr + expr (precedence 10, left)
  expr → expr * expr (precedence 20, left)

Input: "1 + 2 * 3"

Without precedence: Ambiguous
With precedence: (1 + (2 * 3))  ← * binds tighter
```

#### 3. Reject Ambiguity

**Rationale**: Useful for grammar debugging - force user to resolve ambiguity.

**Example**:
```
Input: "if a then if b then c else d"
Grammar: (dangling else)

Strategy: RejectAmbiguity
Result: Error - "Ambiguous parse: 2 alternatives"
```

### 2.4 Error Handling

```rust
/// Forest conversion errors
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Forest has no root nodes")]
    NoRoots,

    #[error("Ambiguous forest: {count} valid parses (use disambiguation strategy)")]
    AmbiguousForest { count: usize },

    #[error("Invalid forest structure: {reason}")]
    InvalidForest { reason: String },

    #[error("Invalid node reference: {node_id:?}")]
    InvalidNodeId { node_id: ForestNodeId },

    #[error("Cycle detected in forest at node {node_id:?}")]
    CycleDetected { node_id: ForestNodeId },
}
```

### 2.5 Testing Strategy

#### Unit Tests

```rust
#[cfg(test)]
mod forest_converter_tests {
    #[test]
    fn test_unambiguous_forest() {
        // Single root → single tree
    }

    #[test]
    fn test_prefer_shift_strategy() {
        // "1 + 2 + 3" → right-associative tree
    }

    #[test]
    fn test_prefer_reduce_strategy() {
        // "1 + 2 + 3" → left-associative tree
    }

    #[test]
    fn test_precedence_strategy() {
        // "1 + 2 * 3" → (1 + (2 * 3))
    }

    #[test]
    fn test_reject_ambiguity() {
        // Ambiguous forest → error
    }

    #[test]
    fn test_invisible_nodes() {
        // Parentheses hidden in tree
    }

    #[test]
    fn test_detect_ambiguity() {
        // Count alternative parses
    }

    #[test]
    fn test_all_trees_extraction() {
        // Extract all valid parse trees
    }
}
```

#### Integration Tests

```rust
#[cfg(test)]
mod forest_integration {
    #[test]
    fn test_end_to_end_arithmetic() {
        // Tokenize → Parse → Convert → Verify tree structure
        let input = b"1 + 2 * 3";
        let tokens = tokenizer.scan(input)?;
        let forest = glr_engine.parse(&tokens)?;
        let tree = converter.to_tree(&forest, input)?;

        assert_eq!(tree.root_node().kind(), "expr");
        assert_eq!(tree.root_node().child_count(), 3);
    }

    #[test]
    fn test_ambiguous_grammar_with_disambiguation() {
        // Ambiguous grammar + disambiguation strategy → single tree
    }

    #[test]
    fn test_nested_expressions() {
        // "((1 + 2) * (3 + 4))" → correct nesting
    }
}
```

---

## Integration Plan

### Phase 3.2.1: Tokenizer Implementation (2 days)

**Day 1: Core Implementation**
- [ ] Create `runtime2/src/tokenizer.rs`
- [ ] Implement `Tokenizer` struct
- [ ] Implement `scan()` method with maximal munch
- [ ] Add pattern matching (Literal and Regex)
- [ ] Add whitespace handling

**Day 2: Testing & Integration**
- [ ] Create `runtime2/tests/tokenizer_test.rs`
- [ ] Implement unit tests (8 tests)
- [ ] Implement integration tests (3 tests)
- [ ] Integrate with `Parser::parse_glr()`
- [ ] Remove stub tokenizer

### Phase 3.2.2: Forest Converter Implementation (2 days)

**Day 3: Core Implementation**
- [ ] Create `runtime2/src/forest_converter.rs`
- [ ] Implement `ForestConverter` struct
- [ ] Implement `to_tree()` with DFS traversal
- [ ] Implement `PreferShift` strategy
- [ ] Add node visibility filtering

**Day 4: Disambiguation & Testing**
- [ ] Implement additional disambiguation strategies
- [ ] Create `runtime2/tests/forest_converter_test.rs`
- [ ] Implement unit tests (8 tests)
- [ ] Implement integration tests (3 tests)
- [ ] Integrate with `Parser::parse_glr()`
- [ ] Remove stub tree

### Phase 3.2.3: End-to-End Testing (1-2 days)

**Day 5: Integration Testing**
- [ ] Update `runtime2/tests/glr_integration_test.rs`
- [ ] Add test: parse arithmetic expressions
- [ ] Add test: parse ambiguous grammars
- [ ] Add test: verify disambiguation
- [ ] All tests passing (Phase 3.1 + Phase 3.2)

**Day 6: Documentation & Polish**
- [ ] Update `docs/specs/GLR_PARSER_API_CONTRACT.md`
- [ ] Create session summary document
- [ ] Update `CLAUDE.md` with Phase 3.2 notes
- [ ] Commit and push all changes

---

## Success Criteria

### Phase 3.2 Complete When:

- [x] Tokenizer implemented and tested (8 unit + 3 integration tests)
- [x] Forest converter implemented and tested (8 unit + 3 integration tests)
- [x] At least 2 disambiguation strategies working
- [x] End-to-end test: parse "1 + 2 * 3" with ambiguous grammar
- [x] No stub implementations remaining in `Parser::parse_glr()`
- [x] All Phase 3.1 tests still passing
- [x] Documentation updated with Phase 3.2 contracts
- [x] Session summary created

---

## Performance Targets

| Metric | Target | Rationale |
|--------|--------|-----------|
| Tokenization Speed | ≥ 1 MB/s | Fast enough for typical source files |
| Forest Conversion | O(n) time | Single traversal of forest |
| Memory Overhead | ≤ 2x input size | Tokens + Forest ≈ 2x input bytes |
| Test Coverage | ≥ 85% | Comprehensive testing |

---

## Risk Mitigation

### Risk: Tokenization Complexity
- **Mitigation**: Start with simple literal/regex patterns, defer external scanners to Phase 3.3
- **Fallback**: Use Tree-sitter's tokenizer via FFI if pure-Rust is too complex

### Risk: Forest Conversion Performance
- **Mitigation**: Profile and optimize hot paths, use arena allocation if needed
- **Fallback**: Implement lazy tree construction (compute on demand)

### Risk: Disambiguation Strategy Correctness
- **Mitigation**: Test against known-good Tree-sitter outputs for same grammars
- **Fallback**: Default to `RejectAmbiguity` and require explicit disambiguation

---

## Dependencies

### Crates Required
- `regex` (already in workspace) - for pattern matching
- `unicode-segmentation` (optional) - for Unicode token boundaries

### Internal Dependencies
- `adze-ir` - Grammar representation
- `adze-glr-core` - ParseTable and Action types
- `runtime2` - Tree, Node, Token APIs

---

## Future Enhancements (Phase 3.3+)

- External scanner support for context-sensitive tokens
- Incremental tokenization (update tokens for edited regions)
- SPPF (Shared Packed Parse Forest) for efficient ambiguity representation
- User-defined disambiguation callbacks
- Token caching and memoization

---

## References

### Related Specifications
- [Phase 3 Master Plan](./PHASE_3_PURE_RUST_GLR_RUNTIME.md)
- [GLR Engine Contract](./GLR_ENGINE_CONTRACT.md)
- [GLR Parser API Contract](./GLR_PARSER_API_CONTRACT.md)

### Academic Papers
- "Efficient Computation of LALR(1) Look-Ahead Sets" - DeRemer & Pennello (1982)
- "Practical Arbitrary Lookahead LR Parsing" - Visser (1997)
- "Scannerless Generalized-LR Parsing" - Visser (1997)

### Implementation References
- Tree-sitter Scanner Implementation
- Rust `regex` crate documentation
- GLL Parsing (Scott & Johnstone)

---

**Document Status**: Complete
**Last Updated**: 2025-01-19
**Next Review**: After Phase 3.2 Implementation
