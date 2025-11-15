#[test]
#[ignore = "Macro-based grammar generation needs parser runtime fixes"]
fn test_empty() {
    let module = test_vec_wrapper::grammar::parse("").unwrap();
    assert_eq!(module.statements.len(), 0);
}

#[test]
#[ignore = "Macro-based grammar generation needs parser runtime fixes"]
fn test_single_number() {
    let module = test_vec_wrapper::grammar::parse("42").unwrap();
    assert_eq!(module.statements.len(), 1);
    assert_eq!(module.statements[0].value, "42");
}

#[test]
#[ignore = "Macro-based grammar generation needs parser runtime fixes"]
fn test_multiple_numbers() {
    let module = test_vec_wrapper::grammar::parse("1 2 3").unwrap();
    assert_eq!(module.statements.len(), 3);
    assert_eq!(module.statements[0].value, "1");
    assert_eq!(module.statements[1].value, "2");
    assert_eq!(module.statements[2].value, "3");
}
