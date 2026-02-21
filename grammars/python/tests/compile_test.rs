use adze_python::grammar_python::LANGUAGE;

#[test]
fn test_language_struct_compiles() {
    // Just verify that the LANGUAGE struct compiles and has expected fields
    let _lang = &LANGUAGE;

    // Check that some basic fields exist
    assert!(LANGUAGE.version > 0);
    assert!(LANGUAGE.symbol_count > 0);
    // field_count can be 0 for grammars without fields

    println!("Language version: {}", LANGUAGE.version);
    println!("Symbol count: {}", LANGUAGE.symbol_count);
    println!("Field count: {}", LANGUAGE.field_count);
}
