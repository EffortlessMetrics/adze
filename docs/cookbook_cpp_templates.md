# Cookbook: Parsing C++ Templates with GLR

## The Challenge

C++ templates are notoriously ambiguous. Consider:

```cpp
vector<vector<int>> matrix;  // Is '>>' a right-shift or two '>'?
```

Traditional parsers struggle because they must decide immediately whether to:
1. Treat `>>` as a single right-shift operator
2. Parse as two separate `>` tokens closing nested templates

## The GLR Solution

With GLR parsing, we explore **both interpretations simultaneously**:

```
Input: vector<vector<int>>
                        ^^
Fork at '>>':
  Path A: Shift '>>' as right_shift_op
  Path B: Reduce '>' then shift next '>'
```

## Step-by-Step Implementation

### 1. Define Your Grammar

```rust
use adze::language;

#[language]
pub struct CppTemplates {
    #[adze::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
    pub identifier: String,
    
    #[adze::leaf(pattern = r">>")]
    pub right_shift: String,
    
    #[adze::leaf(pattern = r">")]
    pub greater_than: String,
    
    #[adze::leaf(pattern = r"<")]
    pub less_than: String,
}

#[derive(adze::Node)]
pub struct TemplateType {
    name: Identifier,
    _lt: LessThan,
    args: TemplateArgs,
    _gt: GreaterThan,
}

#[derive(adze::Node)]
pub enum TemplateArgs {
    Single(Type),
    Multiple(Vec<Type>),
}
```

### 2. Handle the Ambiguity

The key is letting GLR maintain both parse paths:

```rust
// In your grammar rules
#[derive(adze::Node)]
pub enum Type {
    Simple(Identifier),
    Template(TemplateType),
    #[prec(2)]  // Higher precedence for template interpretation
    NestedTemplate(NestedTemplate),
}

#[derive(adze::Node)]
pub struct NestedTemplate {
    outer: TemplateType,
    #[adze::leaf(pattern = r">")]  // Force separate '>'
    _close1: String,
    #[adze::leaf(pattern = r">")]  // tokens for nested
    _close2: String,
}
```

### 3. Use Precedence to Guide Resolution

```rust
#[derive(adze::Node)]
pub enum Expression {
    #[prec(10)]  // Templates have high precedence
    Template(TemplateType),
    
    #[prec(1)]   // Right-shift has low precedence
    RightShift {
        left: Box<Expression>,
        op: RightShift,
        right: Box<Expression>,
    },
}
```

### 4. Testing Your Parser

```rust
#[test]
fn test_nested_templates() {
    let mut parser = Parser::new();
    parser.set_language(cpp_templates::get_language()).unwrap();
    
    // Test nested vector template
    let source = "vector<vector<int>>";
    let tree = parser.parse(source, None).unwrap();
    assert_eq!(tree.error_count(), 0);
    
    // Test actual right-shift
    let source = "x >> 2";
    let tree = parser.parse(source, None).unwrap();
    assert_eq!(tree.error_count(), 0);
}
```

### 5. Advanced: Context-Sensitive Resolution

Sometimes you need context to resolve ambiguity:

```rust
#[derive(adze::Node)]
pub struct Declaration {
    #[adze::context]  // Mark as template context
    type_spec: TypeSpecifier,
    name: Identifier,
}

impl Declaration {
    fn in_template_context(&self) -> bool {
        // Use context to favor template interpretation
        true
    }
}
```

## Real-World Example

Here's how to parse complex template expressions:

```rust
// Input
let source = r#"
    std::map<std::string, std::vector<std::pair<int, int>>> lookup;
    int shift = value >> 3;
"#;

// Parse with GLR
let mut parser = Parser::new();
parser.set_language(cpp_templates::get_language()).unwrap();
let tree = parser.parse(source, None).unwrap();

// GLR handles both:
// - Nested templates with multiple '>>' sequences
// - Actual right-shift operations
assert_eq!(tree.error_count(), 0);
```

## Debugging Tips

### 1. Visualize Fork Points

```rust
// Enable GLR debugging
std::env::set_var("ADZE_GLR_DEBUG", "1");

// Parse and see fork points
let tree = parser.parse(source, None).unwrap();
// Output: Fork at state 42, symbol '>>'
//         Path A: Shift(67)
//         Path B: Reduce(12)
```

### 2. Check Parse Forest

When ambiguity persists:

```rust
if let Some(forest) = parser.get_parse_forest() {
    println!("Ambiguous parse with {} alternatives", 
             forest.alternatives.len());
}
```

### 3. Use Precedence Annotations

```rust
#[derive(adze::Node)]
pub struct TemplateClose {
    #[prec(100)]  // Very high precedence
    #[adze::pattern(">")]
    close: String,
}
```

## Common Pitfalls and Solutions

### Pitfall 1: Infinite Ambiguity
**Problem**: Grammar allows unbounded ambiguity
**Solution**: Use precedence and associativity

### Pitfall 2: Performance Degradation
**Problem**: Too many forks slow down parsing
**Solution**: Add disambiguation rules for common cases

### Pitfall 3: Wrong Parse Selected
**Problem**: GLR picks unexpected interpretation
**Solution**: Adjust precedence values

## Conclusion

GLR parsing elegantly solves C++ template ambiguity by:
1. Maintaining multiple parse paths
2. Using precedence for conflict resolution
3. Producing parse forests when truly ambiguous

This approach extends to other ambiguous constructs like:
- Rust's turbofish (`::<>`) 
- Python's walrus operator (`:=`)
- JavaScript's arrow functions vs comparisons

The key insight: **defer decisions until you have enough context**, which is exactly what GLR enables!