//! Field name propagation tests
//! 
//! Tests that field names from grammar definitions are correctly
//! propagated through the parse tree and accessible via the Node API.

#![cfg(test)]

use rust_sitter::unified_parser::Parser;

/// Test grammar with field names
#[rust_sitter::grammar]
struct FieldTestGrammar {
    /// Function definition with named fields
    /// Pattern: "fn" name:identifier "(" params:param_list? ")" body:block
    #[rust_sitter::rule(pattern = r#"
        "fn" name:identifier "(" params:param_list? ")" body:block
    "#)]
    function: Function,
    
    /// Parameter list
    #[rust_sitter::rule(pattern = r#"
        param ("," param)*
    "#)]
    param_list: ParamList,
    
    /// Single parameter with name and type
    #[rust_sitter::rule(pattern = r#"
        name:identifier ":" type:identifier
    "#)]
    param: Param,
    
    /// Code block
    #[rust_sitter::rule(pattern = r#"
        "{" statement* "}"
    "#)]
    block: Block,
    
    /// Statement (simplified)
    #[rust_sitter::rule(pattern = r#"
        expression ";"
    "#)]
    statement: Statement,
    
    /// Expression (simplified)
    #[rust_sitter::rule(pattern = r#"
        identifier | number | string
    "#)]
    expression: Expression,
    
    /// Identifier
    #[rust_sitter::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
    identifier: Identifier,
    
    /// Number literal
    #[rust_sitter::leaf(pattern = r"\d+")]
    number: Number,
    
    /// String literal
    #[rust_sitter::leaf(pattern = r#""[^"]*""#)]
    string: String,
}

#[test]
fn test_field_name_extraction() {
    let mut parser = Parser::new();
    // TODO: Set language from FieldTestGrammar
    // parser.set_language(FieldTestGrammar::language());
    
    let source = "fn hello(x: int, y: string) { }";
    let tree = parser.parse(source.as_bytes(), None).expect("Failed to parse");
    let root = tree.root_node();
    
    // Navigate to function node
    let function_node = root.child(0).expect("No function node");
    assert_eq!(function_node.kind(), "function");
    
    // Test field_name() method
    let name_node = function_node.child(1).expect("No name node");
    assert_eq!(name_node.field_name(), Some("name"));
    assert_eq!(name_node.kind(), "identifier");
    
    // Test child_by_field_name() method
    let name_by_field = function_node.child_by_field_name("name")
        .expect("No 'name' field");
    assert_eq!(name_by_field.kind(), "identifier");
    assert_eq!(name_by_field.text(source.as_bytes()), "hello");
    
    let body_by_field = function_node.child_by_field_name("body")
        .expect("No 'body' field");
    assert_eq!(body_by_field.kind(), "block");
    
    let params_by_field = function_node.child_by_field_name("params");
    assert!(params_by_field.is_some());
    assert_eq!(params_by_field.unwrap().kind(), "param_list");
}

#[test]
fn test_field_names_in_nested_nodes() {
    let mut parser = Parser::new();
    // TODO: Set language
    
    let source = "fn add(x: int, y: int) { return x; }";
    let tree = parser.parse(source.as_bytes(), None).expect("Failed to parse");
    
    let function = tree.root_node().child(0).expect("No function");
    let params = function.child_by_field_name("params").expect("No params");
    
    // Check first parameter
    let first_param = params.child(0).expect("No first param");
    assert_eq!(first_param.kind(), "param");
    
    let param_name = first_param.child_by_field_name("name").expect("No param name");
    assert_eq!(param_name.text(source.as_bytes()), "x");
    
    let param_type = first_param.child_by_field_name("type").expect("No param type");
    assert_eq!(param_type.text(source.as_bytes()), "int");
}

#[test]
fn test_missing_field_returns_none() {
    let mut parser = Parser::new();
    // TODO: Set language
    
    let source = "fn simple() { }";
    let tree = parser.parse(source.as_bytes(), None).expect("Failed to parse");
    
    let function = tree.root_node().child(0).expect("No function");
    
    // Function has no params, so field should be None
    let params = function.child_by_field_name("params");
    assert!(params.is_none());
    
    // Non-existent field should return None
    let bogus = function.child_by_field_name("bogus_field");
    assert!(bogus.is_none());
}

#[test]
fn test_field_names_in_s_expression() {
    let mut parser = Parser::new();
    // TODO: Set language
    
    let source = "fn greet(name: string) { }";
    let tree = parser.parse(source.as_bytes(), None).expect("Failed to parse");
    
    let s_exp = tree.root_node().to_sexp();
    
    // S-expression should include field annotations
    assert!(s_exp.contains("name:"));
    assert!(s_exp.contains("params:"));
    assert!(s_exp.contains("body:"));
}

#[test]
fn test_field_names_in_json() {
    let mut parser = Parser::new();
    // TODO: Set language
    
    let source = "fn compute(a: int) { }";
    let tree = parser.parse(source.as_bytes(), None).expect("Failed to parse");
    
    // TODO: Implement JSON serialization with fields
    // let json = tree.root_node().to_json();
    // assert!(json.contains(r#""field":"name""#));
    // assert!(json.contains(r#""field":"params""#));
    // assert!(json.contains(r#""field":"body""#));
}

/// Test that field IDs are preserved during incremental parsing
#[test]
#[cfg(feature = "incremental")]
fn test_field_names_after_incremental_edit() {
    let mut parser = Parser::new();
    // TODO: Set language
    
    let source1 = "fn foo() { }";
    let tree1 = parser.parse(source1.as_bytes(), None).expect("Failed to parse");
    
    // Edit: Change function name
    let source2 = "fn bar() { }";
    let edit = Edit {
        start_byte: 3,
        old_end_byte: 6,
        new_end_byte: 6,
        start_position: Point { row: 0, column: 3 },
        old_end_position: Point { row: 0, column: 6 },
        new_end_position: Point { row: 0, column: 6 },
    };
    
    let tree2 = parser.parse_with_old_tree(
        source2.as_bytes(), 
        Some(&tree1), 
        Some(&edit)
    ).expect("Failed to parse incrementally");
    
    let function = tree2.root_node().child(0).expect("No function");
    let name = function.child_by_field_name("name").expect("No name field");
    assert_eq!(name.text(source2.as_bytes()), "bar");
}