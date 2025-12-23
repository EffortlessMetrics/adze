# Rust-Sitter Grammar Examples

This document provides comprehensive examples of how to define grammars using rust-sitter v0.6.0 with GLR parser support (PR #56).

## Table of Contents

1. [Basic Grammar Structure](#basic-grammar-structure)
2. [Leaf Patterns](#leaf-patterns)
3. [Repetition and Optionals](#repetition-and-optionals)
4. [Enums and Variants](#enums-and-variants)
5. [GLR Parser Patterns](#glr-parser-patterns) ✨ **NEW**
6. [Complex Grammars](#complex-grammars)

## Basic Grammar Structure

Every rust-sitter grammar starts with the `#[rust_sitter::grammar]` attribute:

```rust
#[rust_sitter::grammar("my_language")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct Program {
        pub statement: Statement,
    }
    
    #[rust_sitter::language]
    pub struct Statement {
        pub value: Expression,
        #[rust_sitter::leaf(text = ";")]
        _semicolon: (),
    }
    
    #[rust_sitter::language]
    pub struct Expression {
        #[rust_sitter::leaf(pattern = r"\d+")]
        pub number: String,
    }
}
```

## Leaf Patterns

Leaf nodes represent terminal symbols in your grammar:

### Exact Text Match

```rust
#[rust_sitter::language]
pub struct Keywords {
    #[rust_sitter::leaf(text = "if")]
    _if: (),
    
    #[rust_sitter::leaf(text = "else")]
    _else: (),
    
    #[rust_sitter::leaf(text = "return")]
    _return: (),
}
```

### Pattern Matching

```rust
#[rust_sitter::language]
pub struct Tokens {
    // Identifier pattern
    #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
    pub identifier: String,
    
    // Number patterns
    #[rust_sitter::leaf(pattern = r"\d+")]
    pub integer: String,
    
    #[rust_sitter::leaf(pattern = r"\d+\.\d*")]
    pub float: String,
    
    // String patterns
    #[rust_sitter::leaf(pattern = r#""([^"\\]|\\.)*""#)]
    pub string: String,
}
```

### Transformation

```rust
#[rust_sitter::language]
pub struct Numbers {
    // Parse integer values
    #[rust_sitter::leaf(pattern = r"\d+", transform = |s| s.parse().unwrap())]
    pub int_value: i32,
    
    // Parse float values
    #[rust_sitter::leaf(pattern = r"\d+\.\d*", transform = |s| s.parse().unwrap())]
    pub float_value: f64,
}
```

## Repetition and Optionals

### Optional Fields

```rust
#[rust_sitter::language]
pub struct Function {
    #[rust_sitter::leaf(text = "fn")]
    _fn: (),
    pub name: Identifier,
    pub params: Parameters,
    pub return_type: Option<ReturnType>,
    pub body: Block,
}

#[rust_sitter::language]
pub struct ReturnType {
    #[rust_sitter::leaf(text = "->")]
    _arrow: (),
    pub type_name: Type,
}
```

### Repetition (Zero or More)

```rust
#[rust_sitter::language]
pub struct Block {
    #[rust_sitter::leaf(text = "{")]
    _open: (),
    #[rust_sitter::repeat]
    pub statements: Vec<Statement>,
    #[rust_sitter::leaf(text = "}")]
    _close: (),
}
```

### Non-Empty Repetition (One or More)

```rust
#[rust_sitter::language]
pub struct ParameterList {
    #[rust_sitter::leaf(text = "(")]
    _open: (),
    #[rust_sitter::repeat(non_empty = true)]
    pub params: Vec<Parameter>,
    #[rust_sitter::leaf(text = ")")]
    _close: (),
}
```

## Enums and Variants

Enums represent choice points in your grammar:

```rust
#[rust_sitter::language]
pub enum Expression {
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Literal(Literal),
    Identifier(Identifier),
    Call(CallExpr),
}

#[rust_sitter::language]
pub struct BinaryExpr {
    pub left: Box<Expression>,
    pub op: BinaryOp,
    pub right: Box<Expression>,
}

#[rust_sitter::language]
pub enum BinaryOp {
    Add(AddOp),
    Sub(SubOp),
    Mul(MulOp),
    Div(DivOp),
}

#[rust_sitter::language]
pub struct AddOp {
    #[rust_sitter::leaf(text = "+")]
    _op: (),
}
```

## GLR Parser Patterns ✨

With PR #56, rust-sitter includes a production-ready GLR parser that can handle ambiguous grammars. These patterns show how to work with grammars that have unavoidable conflicts.

### Classic Ambiguous Expression Grammar

This grammar creates intentional ambiguity to demonstrate GLR capabilities:

```rust
use rust_sitter_ir::{Grammar, Rule, Symbol, SymbolId, Token, TokenPattern, ProductionId};
use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets, ParseForest};
use rust_sitter::glr_parser_no_error_recovery::GLRParser;

// Create ambiguous grammar: E -> E + E | E * E | num
fn create_ambiguous_expression_grammar() -> Grammar {
    let mut grammar = Grammar::new("ambiguous_expr".to_string());
    
    // Define symbol IDs
    const SYM_NUMBER: SymbolId = SymbolId(1);
    const SYM_PLUS: SymbolId = SymbolId(2);
    const SYM_STAR: SymbolId = SymbolId(3);
    const SYM_EXPR: SymbolId = SymbolId(10);
    
    // Token definitions
    grammar.tokens.insert(SYM_NUMBER, Token {
        name: "number".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(SYM_PLUS, Token {
        name: "plus".to_string(),
        pattern: TokenPattern::String("+".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(SYM_STAR, Token {
        name: "mult".to_string(),
        pattern: TokenPattern::String("*".to_string()),
        fragile: false,
    });
    
    // Grammar rules - NO PRECEDENCE = AMBIGUITY
    let rules = vec![
        // E -> num
        Rule {
            lhs: SYM_EXPR,
            rhs: vec![Symbol::Terminal(SYM_NUMBER)],
            production_id: ProductionId(0),
            precedence: None,        // Intentionally no precedence
            associativity: None,     // Creates shift/reduce conflicts
            fields: vec![],
        },
        // E -> E + E (ambiguous associativity)
        Rule {
            lhs: SYM_EXPR,
            rhs: vec![
                Symbol::NonTerminal(SYM_EXPR),
                Symbol::Terminal(SYM_PLUS),
                Symbol::NonTerminal(SYM_EXPR),
            ],
            production_id: ProductionId(1),
            precedence: None,        // No precedence = ambiguous
            associativity: None,     
            fields: vec![],
        },
        // E -> E * E (creates reduce/reduce conflicts with +)
        Rule {
            lhs: SYM_EXPR,
            rhs: vec![
                Symbol::NonTerminal(SYM_EXPR),
                Symbol::Terminal(SYM_STAR),
                Symbol::NonTerminal(SYM_EXPR),
            ],
            production_id: ProductionId(2),
            precedence: None,        // Ambiguous precedence with +
            associativity: None,
            fields: vec![],
        },
    ];
    
    for rule in rules {
        grammar.rules.entry(SYM_EXPR).or_default().push(rule);
    }
    
    grammar.rule_names.insert(SYM_EXPR, "expression".to_string());
    grammar
}

// Usage example
fn parse_with_glr() -> Result<(), Box<dyn std::error::Error>> {
    let grammar = create_ambiguous_expression_grammar();
    let first_follow = FirstFollowSets::compute(&grammar)?;
    let parse_table = build_lr1_automaton(&grammar, &first_follow)?;
    
    let mut parser = GLRParser::new(parse_table);
    
    // Parse "1+2*3" - has two interpretations:
    // 1. ((1+2)*3) = 9
    // 2. (1+(2*3)) = 7
    let tokens = vec![
        SymbolId(1), // "1"
        SymbolId(2), // "+"
        SymbolId(1), // "2"
        SymbolId(3), // "*"
        SymbolId(1), // "3"
    ];
    
    let forest = parser.parse(&tokens)?;
    
    println!("GLR parsing successful!");
    println!("Parse alternatives: {}", forest.roots.len());
    
    // Analyze alternatives
    for (i, root) in forest.roots.iter().enumerate() {
        println!("Alternative {}: {:?}", i, root);
        println!("  Span: {:?}", root.span);
        println!("  Derivations: {}", root.alternatives.len());
    }
    
    Ok(())
}
```

### Dangling Else Problem

Classic ambiguity in if-else statements:

```rust
// Grammar that creates the dangling else ambiguity
fn create_dangling_else_grammar() -> Grammar {
    let mut grammar = Grammar::new("dangling_else".to_string());
    
    // Tokens
    grammar.tokens.insert(SymbolId(1), Token {
        name: "if".to_string(),
        pattern: TokenPattern::String("if".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(SymbolId(2), Token {
        name: "else".to_string(), 
        pattern: TokenPattern::String("else".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(SymbolId(3), Token {
        name: "condition".to_string(),
        pattern: TokenPattern::String("cond".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(SymbolId(4), Token {
        name: "statement".to_string(),
        pattern: TokenPattern::String("stmt".to_string()),
        fragile: false,
    });
    
    let stmt_id = SymbolId(10);
    
    // Rules that create dangling else ambiguity
    let rules = vec![
        // statement -> if condition statement
        Rule {
            lhs: stmt_id,
            rhs: vec![
                Symbol::Terminal(SymbolId(1)), // if
                Symbol::Terminal(SymbolId(3)), // cond
                Symbol::NonTerminal(stmt_id),  // statement
            ],
            production_id: ProductionId(0),
            precedence: None,  // No precedence = ambiguity
            associativity: None,
            fields: vec![],
        },
        // statement -> if condition statement else statement  
        Rule {
            lhs: stmt_id,
            rhs: vec![
                Symbol::Terminal(SymbolId(1)), // if
                Symbol::Terminal(SymbolId(3)), // cond
                Symbol::NonTerminal(stmt_id),  // statement
                Symbol::Terminal(SymbolId(2)), // else
                Symbol::NonTerminal(stmt_id),  // statement
            ],
            production_id: ProductionId(1),
            precedence: None,  // Creates shift/reduce conflict
            associativity: None,
            fields: vec![],
        },
        // statement -> stmt (base case)
        Rule {
            lhs: stmt_id,
            rhs: vec![Symbol::Terminal(SymbolId(4))],
            production_id: ProductionId(2),
            precedence: None,
            associativity: None,
            fields: vec![],
        },
    ];
    
    for rule in rules {
        grammar.rules.entry(stmt_id).or_default().push(rule);
    }
    
    grammar
}
```

### Forest Analysis Patterns

Tools for analyzing parse forests from ambiguous grammars:

```rust
use std::collections::HashMap;
use rust_sitter_glr_core::{ParseForest, ForestNode};

// Analyze ambiguity in parse forest
pub struct AmbiguityAnalyzer;

impl AmbiguityAnalyzer {
    /// Find all nodes with multiple interpretations
    pub fn find_ambiguous_nodes(forest: &ParseForest) -> Vec<&ForestNode> {
        forest.nodes.values()
            .filter(|node| node.alternatives.len() > 1)
            .collect()
    }
    
    /// Count total number of ambiguous decision points
    pub fn count_ambiguous_decisions(forest: &ParseForest) -> usize {
        forest.nodes.values()
            .map(|node| if node.alternatives.len() > 1 { 1 } else { 0 })
            .sum()
    }
    
    /// Get the most ambiguous node (highest number of alternatives)
    pub fn most_ambiguous_node(forest: &ParseForest) -> Option<&ForestNode> {
        forest.nodes.values()
            .filter(|node| node.alternatives.len() > 1)
            .max_by_key(|node| node.alternatives.len())
    }
    
    /// Extract all complete parse trees from forest
    pub fn extract_all_trees(forest: &ParseForest) -> Vec<String> {
        let mut trees = Vec::new();
        
        for root in &forest.roots {
            let tree_str = Self::format_tree(&forest.nodes, root, 0);
            trees.push(tree_str);
        }
        
        trees
    }
    
    fn format_tree(
        nodes: &HashMap<usize, ForestNode>, 
        node: &ForestNode, 
        depth: usize
    ) -> String {
        let indent = "  ".repeat(depth);
        let mut result = format!("{}{}({:?})", indent, node.symbol.0, node.span);
        
        for alternative in &node.alternatives {
            result.push_str(&format!("\n{}[", indent));
            for &child_id in &alternative.children {
                if let Some(child) = nodes.get(&child_id) {
                    result.push_str(&Self::format_tree(nodes, child, depth + 1));
                }
            }
            result.push(']');
        }
        
        result
    }
}

// Usage example
fn analyze_ambiguous_parse() -> Result<(), Box<dyn std::error::Error>> {
    let grammar = create_ambiguous_expression_grammar();
    let first_follow = FirstFollowSets::compute(&grammar)?;
    let parse_table = build_lr1_automaton(&grammar, &first_follow)?;
    let mut parser = GLRParser::new(parse_table);
    
    // Parse ambiguous input
    let tokens = vec![SymbolId(1), SymbolId(2), SymbolId(1), SymbolId(3), SymbolId(1)];
    let forest = parser.parse(&tokens)?;
    
    // Analyze the forest
    let analyzer = AmbiguityAnalyzer;
    
    let ambiguous_nodes = analyzer.find_ambiguous_nodes(&forest);
    println!("Ambiguous nodes: {}", ambiguous_nodes.len());
    
    let decision_count = analyzer.count_ambiguous_decisions(&forest);
    println!("Ambiguous decisions: {}", decision_count);
    
    if let Some(most_ambiguous) = analyzer.most_ambiguous_node(&forest) {
        println!("Most ambiguous node: {} alternatives at {:?}", 
                 most_ambiguous.alternatives.len(), most_ambiguous.span);
    }
    
    let all_trees = analyzer.extract_all_trees(&forest);
    println!("All parse interpretations:");
    for (i, tree) in all_trees.iter().enumerate() {
        println!("Tree {}: {}", i, tree);
    }
    
    Ok(())
}
```

### GLR Performance Testing

Patterns for testing GLR parser performance on ambiguous grammars:

```rust
use std::time::Instant;

pub struct GLRPerformanceTester {
    parser: GLRParser,
    grammar: Grammar,
}

impl GLRPerformanceTester {
    pub fn new(grammar: Grammar, parse_table: ParseTable) -> Self {
        Self {
            parser: GLRParser::new(parse_table),
            grammar,
        }
    }
    
    /// Test parsing performance on inputs of varying ambiguity
    pub fn benchmark_ambiguity_levels(&mut self) -> Vec<(usize, std::time::Duration, usize)> {
        let mut results = Vec::new();
        
        // Test inputs with different levels of ambiguity
        let test_cases = vec![
            // Simple case: no ambiguity
            vec![SymbolId(1)], // just "1"
            
            // Low ambiguity: one conflict point  
            vec![SymbolId(1), SymbolId(2), SymbolId(1)], // "1+1"
            
            // Medium ambiguity: multiple conflict points
            vec![SymbolId(1), SymbolId(2), SymbolId(1), SymbolId(3), SymbolId(1)], // "1+1*1"
            
            // High ambiguity: nested conflicts
            vec![
                SymbolId(1), SymbolId(2), SymbolId(1), SymbolId(3), 
                SymbolId(1), SymbolId(2), SymbolId(1)
            ], // "1+1*1+1"
        ];
        
        for (i, tokens) in test_cases.into_iter().enumerate() {
            let start = Instant::now();
            let forest = self.parser.parse(&tokens).unwrap();
            let duration = start.elapsed();
            
            let alternative_count = forest.roots.len();
            results.push((i, duration, alternative_count));
            
            println!("Test {}: {}ms, {} alternatives", 
                     i, duration.as_millis(), alternative_count);
        }
        
        results
    }
    
    /// Measure stack forking behavior
    pub fn analyze_forking_behavior(&mut self, tokens: &[SymbolId]) -> ForkingStats {
        // This would require access to internal parser state
        // For now, we analyze the result forest
        let forest = self.parser.parse(tokens).unwrap();
        
        let total_nodes = forest.nodes.len();
        let ambiguous_nodes = forest.nodes.values()
            .filter(|node| node.alternatives.len() > 1)
            .count();
        
        let max_alternatives = forest.nodes.values()
            .map(|node| node.alternatives.len())
            .max()
            .unwrap_or(0);
        
        ForkingStats {
            total_nodes,
            ambiguous_nodes,
            max_alternatives_at_node: max_alternatives,
            parse_alternatives: forest.roots.len(),
        }
    }
}

pub struct ForkingStats {
    pub total_nodes: usize,
    pub ambiguous_nodes: usize,
    pub max_alternatives_at_node: usize,
    pub parse_alternatives: usize,
}
```

## Complex Grammars

### JSON Grammar Example

```rust
#[rust_sitter::grammar("json")]
pub mod json_grammar {
    #[rust_sitter::language]
    pub struct Document {
        pub value: Value,
    }
    
    #[rust_sitter::language]
    pub enum Value {
        Object(Object),
        Array(Array),
        String(StringLit),
        Number(Number),
        Boolean(Boolean),
        Null(Null),
    }
    
    #[rust_sitter::language]
    pub struct Object {
        #[rust_sitter::leaf(text = "{")]
        _open: (),
        #[rust_sitter::repeat]
        pub members: Vec<Member>,
        #[rust_sitter::leaf(text = "}")]
        _close: (),
    }
    
    #[rust_sitter::language]
    pub struct Member {
        pub key: StringLit,
        #[rust_sitter::leaf(text = ":")]
        _colon: (),
        pub value: Value,
        pub comma: Option<Comma>,
    }
    
    #[rust_sitter::language]
    pub struct Comma {
        #[rust_sitter::leaf(text = ",")]
        _comma: (),
    }
    
    #[rust_sitter::language]
    pub struct Array {
        #[rust_sitter::leaf(text = "[")]
        _open: (),
        #[rust_sitter::repeat]
        pub elements: Vec<Element>,
        #[rust_sitter::leaf(text = "]")]
        _close: (),
    }
    
    #[rust_sitter::language]
    pub struct Element {
        pub value: Value,
        pub comma: Option<Comma>,
    }
    
    #[rust_sitter::language]
    pub struct StringLit {
        #[rust_sitter::leaf(pattern = r#""([^"\\]|\\.)*""#)]
        pub value: String,
    }
    
    #[rust_sitter::language]
    pub struct Number {
        #[rust_sitter::leaf(pattern = r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?")]
        pub value: String,
    }
    
    #[rust_sitter::language]
    pub enum Boolean {
        True(True),
        False(False),
    }
    
    #[rust_sitter::language]
    pub struct True {
        #[rust_sitter::leaf(text = "true")]
        _true: (),
    }
    
    #[rust_sitter::language]
    pub struct False {
        #[rust_sitter::leaf(text = "false")]
        _false: (),
    }
    
    #[rust_sitter::language]
    pub struct Null {
        #[rust_sitter::leaf(text = "null")]
        _null: (),
    }
}
```

### Expression Grammar with Operators

```rust
#[rust_sitter::grammar("calc")]
pub mod calc_grammar {
    #[rust_sitter::language]
    pub struct Program {
        pub expression: Expression,
    }
    
    #[rust_sitter::language]
    pub enum Expression {
        Binary(Box<BinaryExpression>),
        Unary(Box<UnaryExpression>),
        Primary(PrimaryExpression),
    }
    
    #[rust_sitter::language]
    pub struct BinaryExpression {
        pub left: Expression,
        pub operator: BinaryOperator,
        pub right: Expression,
    }
    
    #[rust_sitter::language]
    pub enum BinaryOperator {
        Add(AddOp),
        Subtract(SubOp),
        Multiply(MulOp),
        Divide(DivOp),
        Power(PowerOp),
    }
    
    #[rust_sitter::language]
    pub struct AddOp {
        #[rust_sitter::leaf(text = "+")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub struct SubOp {
        #[rust_sitter::leaf(text = "-")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub struct MulOp {
        #[rust_sitter::leaf(text = "*")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub struct DivOp {
        #[rust_sitter::leaf(text = "/")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub struct PowerOp {
        #[rust_sitter::leaf(text = "^")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub struct UnaryExpression {
        pub operator: UnaryOperator,
        pub operand: Expression,
    }
    
    #[rust_sitter::language]
    pub enum UnaryOperator {
        Plus(UnaryPlusOp),
        Minus(UnaryMinusOp),
    }
    
    #[rust_sitter::language]
    pub struct UnaryPlusOp {
        #[rust_sitter::leaf(text = "+")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub struct UnaryMinusOp {
        #[rust_sitter::leaf(text = "-")]
        _op: (),
    }
    
    #[rust_sitter::language]
    pub enum PrimaryExpression {
        Number(Number),
        Identifier(Identifier),
        Parenthesized(Box<ParenthesizedExpression>),
    }
    
    #[rust_sitter::language]
    pub struct Number {
        #[rust_sitter::leaf(pattern = r"\d+(?:\.\d+)?", transform = |s| s.parse::<f64>().unwrap())]
        pub value: f64,
    }
    
    #[rust_sitter::language]
    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
        pub name: String,
    }
    
    #[rust_sitter::language]
    pub struct ParenthesizedExpression {
        #[rust_sitter::leaf(text = "(")]
        _open: (),
        pub expression: Expression,
        #[rust_sitter::leaf(text = ")")]
        _close: (),
    }
}
```

## Best Practices

1. **Use underscores for syntax-only fields**: Fields that represent punctuation or keywords should start with `_` to indicate they're not semantically important.

2. **Box recursive types**: When you have recursive structures (like expressions), use `Box<T>` to avoid infinite-size types.

3. **Prefer enums for alternatives**: Use enums to represent different variants of a language construct.

4. **Use Option for optional syntax**: When a language feature is optional, use `Option<T>`.

5. **Use Vec for repetitions**: The `#[rust_sitter::repeat]` attribute works with `Vec<T>`.

## Current Limitations

The v0.5.0-beta release has some limitations:

- No support for precedence annotations (`#[rust_sitter::prec]`)
- No support for associativity (`#[rust_sitter::prec_left]`, `#[rust_sitter::prec_right]`)
- No support for external scanners (`#[rust_sitter::external]`)
- No support for word tokens (`#[rust_sitter::word]`)
- No support for delimited lists (`#[rust_sitter::delimited]`)

These features are planned for future releases.

## Using Your Grammar

Once you've defined your grammar, add it to your `build.rs`:

```rust
use rust_sitter_tool::build_parsers;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/grammar.rs");
    build_parsers(&PathBuf::from("src/grammar.rs"));
}
```

And use it in your code:

```rust
use my_language::grammar::*;

fn main() {
    // Your parsing code here
}
```