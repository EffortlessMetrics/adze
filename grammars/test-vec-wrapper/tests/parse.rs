use test_vec_wrapper::grammar::{TestModule, TestStatement};
use rust_sitter::Extract;

fn parse(input: &str) -> Result<rust_sitter::Tree, rust_sitter::ParseError> {
    let mut parser = rust_sitter::Parser::new()?;
    parser.set_language(test_vec_wrapper::LANGUAGE)?;
    parser.parse(input, None).ok_or(rust_sitter::ParseError::Unknown)
}

#[test]
fn test_empty() {
    let tree = parse("").unwrap();
    let module = TestModule::extract(&tree.root()).unwrap();
    assert_eq!(module.statements.len(), 0);
}

#[test]
fn test_single_number() {
    let tree = parse("42").unwrap();
    let module = TestModule::extract(&tree.root()).unwrap();
    assert_eq!(module.statements.len(), 1);
    assert_eq!(module.statements[0].value, 42);
}