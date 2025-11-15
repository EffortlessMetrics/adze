use test_vec_wrapper::grammar::parse;

#[test]
#[ignore = "Macro-based grammar generation needs parser runtime fixes"]
fn test_empty_vec_no_fork() {
    // Empty input should produce empty Vec without forking
    let result = parse("");
    assert!(result.is_ok());
    let module = result.unwrap();
    assert_eq!(module.statements.len(), 0);
}

#[test]
#[ignore = "Macro-based grammar generation needs parser runtime fixes"]
fn test_whitespace_only_no_fork() {
    // Whitespace only should produce empty Vec without forking
    let result = parse("  \n\t  ");
    assert!(result.is_ok());
    let module = result.unwrap();
    assert_eq!(module.statements.len(), 0);
}

#[test]
#[ignore = "Macro-based grammar generation needs parser runtime fixes"]
fn test_multiple_statements_no_fork() {
    // Multiple numbers should parse without forking
    let result = parse("12 34 56");
    assert!(result.is_ok());
    let module = result.unwrap();
    assert_eq!(module.statements.len(), 3);
    assert_eq!(module.statements[0].value, "12");
    assert_eq!(module.statements[1].value, "34");
    assert_eq!(module.statements[2].value, "56");
}

#[test]
#[ignore = "Macro-based grammar generation needs parser runtime fixes"]
fn test_mixed_whitespace_and_numbers() {
    // Mix of numbers and whitespace
    let result = parse("  42  \n  99  ");
    assert!(result.is_ok());
    let module = result.unwrap();
    assert_eq!(module.statements.len(), 2);
    assert_eq!(module.statements[0].value, "42");
    assert_eq!(module.statements[1].value, "99");
}
