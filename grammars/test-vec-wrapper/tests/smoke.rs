use test_vec_wrapper::grammar;

#[test]
fn test_smoke_language_object_constructs() {
    let lang = grammar::language();
    assert!(
        lang.symbol_count > 0,
        "generated language should expose symbols"
    );
}
