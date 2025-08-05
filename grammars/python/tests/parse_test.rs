use rust_sitter_python::grammar_python::LANGUAGE;
use rust_sitter::parser_v4::Parser;

#[test]
fn test_parse_simple_python() {
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE).expect("Failed to set language");
    
    let source = b"print('hello world')";
    let tree = parser.parse(source, None).expect("Failed to parse");
    
    assert!(!tree.root_node().has_error());
    assert_eq!(tree.root_node().kind(), "module");
    
    println!("Parse tree: {}", tree.root_node().to_sexp());
}

#[test]
fn test_parse_function_with_indentation() {
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE).expect("Failed to set language");
    
    let source = b"def hello(name):
    print(f'Hello, {name}!')
    return True
";
    
    let tree = parser.parse(source, None).expect("Failed to parse");
    
    assert!(!tree.root_node().has_error());
    assert_eq!(tree.root_node().kind(), "module");
    
    // Check that we have a function_definition node
    let function = tree.root_node().child(0).unwrap();
    assert_eq!(function.kind(), "function_definition");
    
    println!("Parse tree: {}", tree.root_node().to_sexp());
}

#[test]
fn test_parse_class_with_methods() {
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE).expect("Failed to set language");
    
    let source = b"class MyClass:
    def __init__(self, value):
        self.value = value
    
    def get_value(self):
        return self.value
";
    
    let tree = parser.parse(source, None).expect("Failed to parse");
    
    assert!(!tree.root_node().has_error());
    assert_eq!(tree.root_node().kind(), "module");
    
    // Check that we have a class_definition node
    let class = tree.root_node().child(0).unwrap();
    assert_eq!(class.kind(), "class_definition");
    
    println!("Parse tree: {}", tree.root_node().to_sexp());
}

#[test] 
fn test_parse_nested_indentation() {
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE).expect("Failed to set language");
    
    let source = b"if True:
    if False:
        pass
    else:
        for i in range(10):
            print(i)
";
    
    let tree = parser.parse(source, None).expect("Failed to parse");
    
    assert!(!tree.root_node().has_error());
    println!("Parse tree: {}", tree.root_node().to_sexp());
}