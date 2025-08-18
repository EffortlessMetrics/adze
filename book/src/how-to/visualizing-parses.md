# GLR Parser Visualization Guide

This guide explains how to use the GLR visualization tools to debug ambiguous grammars and understand parser behavior.

## Overview

The GLR (Generalized LR) parser can handle ambiguous grammars by maintaining multiple parse stacks simultaneously. When the parser encounters ambiguity (shift/reduce or reduce/reduce conflicts), it "forks" into multiple paths and explores them in parallel. The visualization tools help you see this forking and merging behavior.

## Using the Visualization API

### Basic Example

```rust
use rust_sitter::glr_visualization::{GLRVisualizer, VisualizationOptions};
use rust_sitter::glr_parser::GLRParser;
use rust_sitter::glr_lexer::GLRLexer;

// Create parser and lexer
let grammar = create_your_grammar();
let parse_table = build_lr1_automaton(&grammar, &first_follow)?;
let mut parser = GLRParser::new(parse_table, grammar.clone());
let mut lexer = GLRLexer::new(&grammar, input)?;

// Enable visualization
let mut visualizer = GLRVisualizer::new(VisualizationOptions {
    show_lookahead: true,
    show_items: true,
    show_parse_trees: true,
    compact_mode: false,
});

// Parse with visualization tracking
let mut tokens = Vec::new();
while let Some(token) = lexer.next_token() {
    tokens.push(token);
}

for token in &tokens {
    visualizer.record_state(&parser);
    parser.process_token(token.symbol_id, &token.text, token.byte_offset);
}

parser.process_eof();
let result = parser.finish();

// Generate visualization
let dot_graph = visualizer.to_dot();
std::fs::write("parser_trace.dot", dot_graph)?;
```

### Visualization Options

- **show_lookahead**: Display lookahead symbols at each state
- **show_items**: Show LR(1) items in each state
- **show_parse_trees**: Include partial parse trees at each step
- **compact_mode**: Reduce detail for large traces

## Understanding the Output

### DOT Graph Format

The generated DOT file can be rendered with Graphviz:

```bash
dot -Tpng parser_trace.dot -o parser_trace.png
```

### Graph Elements

1. **States** (rectangles): Parser states with their state ID
2. **Forks** (diamonds): Points where the parser splits into multiple paths
3. **Merges** (inverted triangles): Points where paths converge
4. **Actions** (edges): Shift, reduce, or accept actions

### Color Coding

- 🟦 **Blue**: Normal parsing path
- 🟥 **Red**: Fork point (ambiguity detected)
- 🟩 **Green**: Successful merge
- 🟨 **Yellow**: Pruned path (dead end)

## Common Ambiguity Patterns

### 1. Expression Precedence Ambiguity

```
Input: 1 + 2 * 3
Fork at: After parsing "1 + 2"
- Path 1: Reduce to E (treats as (1 + 2) * 3)
- Path 2: Shift * (treats as 1 + (2 * 3))
```

### 2. Dangling Else

```
Input: if (a) if (b) x else y
Fork at: After parsing "else"
- Path 1: Else belongs to inner if
- Path 2: Else belongs to outer if
```

### 3. Shift/Reduce Conflicts

The visualizer shows:
- The conflicting state
- The lookahead causing conflict
- Both possible actions

## Debugging Tips

### 1. Identify Fork Points

Look for red diamond nodes in the graph. These indicate where your grammar is ambiguous.

### 2. Trace Parse Paths

Follow the edges from a fork to see how different interpretations proceed.

### 3. Find Unnecessary Ambiguity

If paths merge immediately after forking with the same result, the grammar might have redundant ambiguity.

### 4. Performance Analysis

Count forks to estimate parsing complexity. Exponential forking indicates problematic grammar design.

## Example: Arithmetic Grammar

Here's a complete example analyzing an ambiguous arithmetic grammar:

```rust
// Grammar: E -> E + E | E * E | num
// This grammar is ambiguous for precedence and associativity

let input = "1 + 2 * 3 + 4";
let mut visualizer = GLRVisualizer::new(VisualizationOptions::default());

// ... parsing code ...

// The visualization will show:
// 1. Fork after "1 + 2" (shift * vs reduce E+E)
// 2. Fork after "3" (multiple ways to group)
// 3. Multiple parse trees in the final result
```

## Text-Based Trace

For simpler debugging, use the text trace:

```rust
let trace = visualizer.to_text_trace();
println!("{}", trace);
```

Output format:
```
Step 1: State 0, Token 'num' (1)
  Action: Shift to state 3
  Stack: [0, 3]

Step 2: State 3, Token '+' 
  Action: Reduce E -> num
  Stack: [0, 1]

Step 3: State 1, Token '+'
  Action: Shift to state 4
  Stack: [0, 1, 4]
...
```

## Integration with Testing

Add visualization to failing tests:

```rust
#[test]
fn test_ambiguous_grammar() {
    let result = parse_with_visualization("ambiguous input");
    if result.is_ambiguous() {
        let viz = result.get_visualization();
        eprintln!("Parse visualization:\n{}", viz.to_text_trace());
        // Save DOT file for CI artifacts
        std::fs::write("test_parse.dot", viz.to_dot()).ok();
    }
}
```

## Performance Considerations

- Visualization adds overhead; disable in production
- Use `compact_mode` for long inputs
- Consider sampling (record every Nth step) for very long parses

## Troubleshooting

### "Too many forks" error
Your grammar is extremely ambiguous. Consider:
- Adding precedence declarations
- Refactoring to remove ambiguity
- Using GLR parsing limits

### Visualization too large
- Enable `compact_mode`
- Focus on specific input regions
- Use text trace instead of DOT

### Can't see the ambiguity
- Ensure `show_items` is enabled
- Check if paths merge before the end
- Look for subtle lookahead differences