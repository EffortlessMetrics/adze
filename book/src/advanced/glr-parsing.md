# GLR (Generalized LR) Parsing

GLR parsing is a powerful parsing technique that extends traditional LR parsing to handle ambiguous grammars. With the GLR parser implementation in PR #56, rust-sitter can now parse languages with inherent ambiguities, multiple interpretations, and complex grammar conflicts.

## What is GLR Parsing?

**GLR (Generalized LR) parsing** allows parsers to handle ambiguous grammars by exploring multiple parse paths simultaneously. Unlike traditional LR parsers that fail on conflicts, GLR parsers create **parse forests** containing all valid interpretations.

### Traditional vs GLR Parsing

| Feature | Traditional LR | GLR Parsing |
|---------|---------------|-------------|
| **Grammar Support** | Unambiguous only | Ambiguous grammars ✨ |
| **Conflicts** | Error/failure | Explored simultaneously |
| **Output** | Single parse tree | Parse forest with alternatives |
| **Use Cases** | Simple languages | Complex languages (C++, natural language) |

## ActionCell Architecture

The core innovation in PR #56 is the **ActionCell architecture**, where each parser state/symbol combination can hold multiple conflicting actions:

```rust
// Traditional LR: action_table[state][symbol] = Action
// GLR: action_table[state][symbol] = Vec<Action>

pub fn get_actions(&self, state: StateId, symbol: SymbolId) -> Vec<Action> {
    // Returns multiple actions for conflicts
    self.table.action_table[state_idx][symbol_idx].clone()
}
```

### How ActionCells Work

1. **Single Action**: Normal LR parsing continues with one action
2. **Multiple Actions**: Parser **forks** into multiple parse stacks
3. **Conflict Types**: 
   - **Shift/Reduce**: Parse both ways (shift token vs reduce rule)
   - **Reduce/Reduce**: Multiple reductions possible
   - **Fork Actions**: Nested conflicts handled recursively

```rust
match action {
    Action::Shift(next_state) => {
        // Create new parse stack in shifted state
        let mut new_stack = stack.clone();
        new_stack.states.push(*next_state);
        new_stack.nodes.push(node_id);
        new_stacks.push(new_stack);
    }
    Action::Fork(fork_actions) => {
        // Handle each conflicting action
        for fork_action in fork_actions {
            self.handle_terminal_action(
                stack, fork_action, symbol, position,
                new_stacks, forest, accepted
            );
        }
    }
    // ... other actions
}
```

## Parse Forest Structure

GLR parsers produce **parse forests** instead of single parse trees. A forest efficiently represents multiple parse interpretations:

```rust
pub struct ParseForest {
    pub roots: Vec<ForestNode>,           // All valid complete parses
    pub nodes: HashMap<usize, ForestNode>, // Shared node storage
    pub grammar: Grammar,                  // Grammar used
    pub source: String,                   // Original source text
    pub next_node_id: usize,              // Node ID allocator
}

pub struct ForestNode {
    pub id: usize,                        // Unique identifier
    pub symbol: SymbolId,                 // Symbol this node represents
    pub span: (usize, usize),            // Source text span
    pub alternatives: Vec<ForestAlternative>, // Multiple derivations
    pub error_meta: ErrorMeta,           // Error information
}
```

### Forest vs Tree Benefits

- **Memory Efficiency**: Shared subtrees reduce duplication
- **Complete Information**: All valid parses preserved
- **Conflict Analysis**: See exactly where ambiguities occur
- **Flexible Output**: Convert to single tree or analyze all alternatives

## Creating GLR Grammars

### Ambiguous Expression Grammar

The classic ambiguous grammar that GLR can handle:

```rust
// E -> E + E | E * E | num
// This grammar is ambiguous for "1+2*3"

fn create_ambiguous_grammar() -> Grammar {
    let mut grammar = Grammar::new("expr".to_string());
    
    // Tokens
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
    
    // Rules - notice no precedence declarations!
    let rules = vec![
        Rule {  // E -> num
            lhs: SYM_EXPR,
            rhs: vec![Symbol::Terminal(SYM_NUMBER)],
            production_id: ProductionId(0),
            precedence: None,  // No precedence = ambiguity
            associativity: None,
            fields: vec![],
        },
        Rule {  // E -> E + E
            lhs: SYM_EXPR,
            rhs: vec![
                Symbol::NonTerminal(SYM_EXPR),
                Symbol::Terminal(SYM_PLUS),
                Symbol::NonTerminal(SYM_EXPR),
            ],
            production_id: ProductionId(1),
            precedence: None,  // Creates shift/reduce conflicts
            associativity: None,
            fields: vec![],
        },
        Rule {  // E -> E * E  
            lhs: SYM_EXPR,
            rhs: vec![
                Symbol::NonTerminal(SYM_EXPR),
                Symbol::Terminal(SYM_STAR),
                Symbol::NonTerminal(SYM_EXPR),
            ],
            production_id: ProductionId(2),
            precedence: None,  // Creates more conflicts
            associativity: None,
            fields: vec![],
        },
    ];
    
    for rule in rules {
        grammar.rules.entry(SYM_EXPR).or_default().push(rule);
    }
    
    grammar
}
```

### When to Use GLR

**GLR is beneficial when:**
- Grammar has unavoidable ambiguities
- Multiple valid interpretations exist
- Precedence rules are complex or context-dependent
- Analyzing all possible parses is valuable

**Examples:**
- **C++ templates**: `A<B<C>>` vs `A < B < C > >`
- **Natural language**: "I saw the man with the telescope"
- **Python decorators**: Multiple decorator application orders
- **Regular expressions**: Nested quantifiers

## Using the GLR Parser

### Basic GLR Parsing

```rust
use rust_sitter::glr_parser_no_error_recovery::GLRParser;
use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets};

// Create grammar and parse table
let grammar = create_ambiguous_grammar();
let first_follow = FirstFollowSets::compute(&grammar)?;
let parse_table = build_lr1_automaton(&grammar, &first_follow)?;

// Create GLR parser
let mut parser = GLRParser::new(parse_table);

// Parse ambiguous input
let tokens = vec![SYM_NUMBER, SYM_PLUS, SYM_NUMBER, SYM_STAR, SYM_NUMBER];
let forest = parser.parse(&tokens)?;

// Analyze results
println!("Parse completed successfully!");
println!("Number of parse alternatives: {}", forest.roots.len());
for (i, root) in forest.roots.iter().enumerate() {
    println!("Alternative {}: {:?}", i, root);
}
```

### Handling Parse Forests

```rust
// Extract all parse trees from forest
fn extract_parse_trees(forest: &ParseForest) -> Vec<String> {
    let mut trees = Vec::new();
    
    for root in &forest.roots {
        let tree_repr = format_forest_node(&forest.nodes, root);
        trees.push(tree_repr);
    }
    
    trees
}

// Format a forest node for display
fn format_forest_node(
    nodes: &HashMap<usize, ForestNode>, 
    node: &ForestNode
) -> String {
    let mut result = format!("{}(", node.symbol.0);
    
    for alternative in &node.alternatives {
        result.push('[');
        for &child_id in &alternative.children {
            if let Some(child) = nodes.get(&child_id) {
                result.push_str(&format_forest_node(nodes, child));
                result.push(' ');
            }
        }
        result.push(']');
    }
    
    result.push(')');
    result
}
```

### Forest Analysis Tools

```rust
// Analyze ambiguity in parse forest
pub struct ForestAnalyzer;

impl ForestAnalyzer {
    pub fn count_ambiguities(forest: &ParseForest) -> usize {
        forest.nodes.values()
            .filter(|node| node.alternatives.len() > 1)
            .count()
    }
    
    pub fn find_most_ambiguous_nodes(forest: &ParseForest) -> Vec<&ForestNode> {
        let mut nodes: Vec<_> = forest.nodes.values()
            .filter(|node| node.alternatives.len() > 1)
            .collect();
        
        nodes.sort_by_key(|node| node.alternatives.len());
        nodes.reverse();
        nodes
    }
    
    pub fn extract_disambiguation_points(forest: &ParseForest) -> Vec<(SymbolId, usize, usize)> {
        forest.nodes.values()
            .filter(|node| node.alternatives.len() > 1)
            .map(|node| (node.symbol, node.span.0, node.span.1))
            .collect()
    }
}
```

## Advanced GLR Techniques

### Fork Tracking and Management

```rust
// Track parse stack forking during parsing
pub struct ForkTracker {
    fork_count: usize,
    max_active_stacks: usize,
    stack_history: Vec<usize>,
}

impl ForkTracker {
    pub fn record_fork(&mut self, new_stack_count: usize) {
        self.fork_count += 1;
        self.max_active_stacks = self.max_active_stacks.max(new_stack_count);
        self.stack_history.push(new_stack_count);
    }
    
    pub fn analyze_complexity(&self) -> ForkComplexity {
        ForkComplexity {
            total_forks: self.fork_count,
            peak_parallelism: self.max_active_stacks,
            average_stacks: self.stack_history.iter().sum::<usize>() as f64 / self.stack_history.len() as f64,
        }
    }
}
```

### Custom Forest Processors

```rust
// Process parse forests for specific applications
pub trait ForestProcessor {
    type Output;
    
    fn process_forest(&mut self, forest: &ParseForest) -> Self::Output;
    fn process_node(&mut self, node: &ForestNode) -> Option<Self::Output>;
}

// Example: Extract preferred interpretation based on heuristics
pub struct PreferenceBasedProcessor {
    preferences: Vec<SymbolId>,  // Preferred symbols in conflicts
}

impl ForestProcessor for PreferenceBasedProcessor {
    type Output = ForestNode;
    
    fn process_forest(&mut self, forest: &ParseForest) -> Self::Output {
        // Find root with highest preference score
        forest.roots.iter()
            .max_by_key(|root| self.score_node(&forest.nodes, root))
            .cloned()
            .unwrap_or_else(|| forest.roots[0].clone())
    }
    
    fn process_node(&mut self, node: &ForestNode) -> Option<Self::Output> {
        // Select best alternative based on preferences
        let best_alt = node.alternatives.iter()
            .max_by_key(|alt| self.score_alternative(alt))?;
            
        Some(ForestNode {
            id: node.id,
            symbol: node.symbol,
            span: node.span,
            alternatives: vec![best_alt.clone()],
            error_meta: node.error_meta.clone(),
        })
    }
}
```

## Performance Considerations

### GLR Performance Characteristics

GLR parsing complexity depends on grammar ambiguity:

- **Unambiguous sections**: Linear time (like LR)
- **Local ambiguity**: Polynomial time in ambiguous region size
- **Highly ambiguous**: Exponential in worst case

### Optimization Strategies

```rust
// Monitor GLR performance during parsing
pub struct GLRProfiler {
    start_time: Instant,
    fork_events: Vec<ForkEvent>,
    memory_usage: Vec<usize>,
}

impl GLRProfiler {
    pub fn record_parse_start(&mut self) {
        self.start_time = Instant::now();
    }
    
    pub fn record_fork(&mut self, position: usize, stack_count: usize) {
        self.fork_events.push(ForkEvent {
            position,
            stack_count,
            timestamp: self.start_time.elapsed(),
        });
    }
    
    pub fn analyze_performance(&self) -> GLRPerformanceReport {
        GLRPerformanceReport {
            total_time: self.start_time.elapsed(),
            max_parallelism: self.fork_events.iter().map(|e| e.stack_count).max().unwrap_or(1),
            fork_frequency: self.fork_events.len() as f64 / self.start_time.elapsed().as_secs_f64(),
            memory_peak: self.memory_usage.iter().max().copied().unwrap_or(0),
        }
    }
}
```

## Integration with Runtime2

The GLR parser integrates with the high-level runtime2 API:

```rust
// Using GLR through the main Parser API
use rust_sitter_runtime::Parser;

let mut parser = Parser::new();
parser.set_language(glr_language)?;

// Automatic GLR routing when grammar has conflicts
let tree = parser.parse_utf8("ambiguous input", None)?;

// Tree is automatically resolved from best forest interpretation
let root = tree.root_node();
println!("Resolved parse: {}", root.kind());
```

## Best Practices

### Grammar Design for GLR

1. **Identify Ambiguity Sources**: Know where conflicts will occur
2. **Test Incrementally**: Start with simple ambiguous cases
3. **Monitor Performance**: Profile parsing on representative inputs
4. **Provide Disambiguation**: Use semantic actions or preferences when possible

### Debugging GLR Grammars

```rust
// Debug GLR parsing with detailed forest information
use std::env;

env::set_var("RUST_SITTER_GLR_DEBUG", "true");

let forest = parser.parse(&tokens)?;

// Analyze forest structure
let analyzer = ForestAnalyzer;
println!("Ambiguous nodes: {}", analyzer.count_ambiguities(&forest));

for node in analyzer.find_most_ambiguous_nodes(&forest) {
    println!("High ambiguity at {:?}: {} alternatives", 
             node.span, node.alternatives.len());
}
```

### Testing GLR Parsers

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_glr_ambiguous_expression() {
        let mut parser = create_glr_parser();
        let forest = parser.parse(&[SYM_NUMBER, SYM_PLUS, SYM_NUMBER, SYM_STAR, SYM_NUMBER])?;
        
        // Should have multiple interpretations for "1+2*3"
        assert!(forest.roots.len() >= 2, "Should have multiple parse interpretations");
        
        // Check that both groupings are present:
        // ((1+2)*3) and (1+(2*3))
        let trees = extract_parse_trees(&forest);
        assert!(trees.iter().any(|t| t.contains("(1+2)*3")));
        assert!(trees.iter().any(|t| t.contains("1+(2*3)")));
    }
    
    #[test] 
    fn test_glr_performance() {
        let mut profiler = GLRProfiler::new();
        let mut parser = create_glr_parser();
        
        profiler.record_parse_start();
        let _forest = parser.parse(&long_ambiguous_input())?;
        
        let report = profiler.analyze_performance();
        assert!(report.total_time < Duration::from_secs(1), "GLR parsing too slow");
        assert!(report.max_parallelism < 100, "Too many concurrent stacks");
    }
}
```

## Future Enhancements

The GLR implementation in PR #56 provides the foundation for advanced features:

- **Error Recovery**: Extending GLR with sophisticated error handling
- **Incremental GLR**: Updating parse forests efficiently after edits  
- **Disambiguation**: Automatic conflict resolution based on context
- **Streaming GLR**: Processing large inputs without full materialization

The ActionCell architecture makes these enhancements possible while maintaining the core GLR parsing capabilities demonstrated in the current implementation.