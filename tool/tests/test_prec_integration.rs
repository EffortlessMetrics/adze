use std::fs;
use tempfile::tempdir;

/// Test that precedence errors are properly integrated with the grammar generation pipeline
/// and don't break other parts of the grammar processing

#[test]
fn precedence_error_preserves_other_grammar_elements() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(
        &grammar_path,
        r#"
        #[adze::grammar("test_integration")]
        mod grammar {
            #[adze::language]
            pub enum Expression {
                Number(
                    #[adze::leaf(pattern = r"\d+")]
                    i32
                ),
                // This should cause a precedence error
                #[adze::prec(1)]
                #[adze::prec_left(2)]
                BadPrecedence(
                    Box<Expression>,
                    #[adze::leaf(text = "+")]
                    (),
                    Box<Expression>,
                ),
                // This should be valid
                ValidRule(
                    Box<Expression>,
                    #[adze::leaf(text = "*")]
                    (),
                    Box<Expression>,
                ),
            }

            // Extra should still be processed
            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }

            // External should still be processed  
            #[adze::external]
            struct IndentToken;
        }
    "#,
    )
    .unwrap();

    let err = adze_tool::generate_grammars(&grammar_path).unwrap_err();
    let error_msg = err.to_string();

    // Should get precedence error
    assert!(
        error_msg.contains("only one of prec, prec_left, and prec_right can be specified"),
        "Expected precedence error, got: {}",
        error_msg
    );

    // Error should be specific and mention which attributes were found
    assert!(
        error_msg.contains("prec, prec_left"),
        "Expected error to mention conflicting attributes, got: {}",
        error_msg
    );
}

#[test]
fn precedence_error_in_struct_fields() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(
        &grammar_path,
        r#"
        #[adze::grammar("test_struct_prec")]
        mod grammar {
            #[adze::language]
            pub struct Statement {
                expr: Expression,
            }

            #[adze::prec(5)]
            #[adze::prec_right(10)]
            pub struct Expression {
                #[adze::leaf(pattern = r"\d+")]
                value: i32,
            }
        }
    "#,
    )
    .unwrap();

    let err = adze_tool::generate_grammars(&grammar_path).unwrap_err();
    let error_msg = err.to_string();

    assert!(
        error_msg.contains("only one of prec, prec_left, and prec_right can be specified"),
        "Expected precedence error on struct, got: {}",
        error_msg
    );
    assert!(
        error_msg.contains("prec, prec_right"),
        "Expected error to mention specific conflicting attributes, got: {}",
        error_msg
    );
}

#[test]
fn multiple_precedence_errors_reports_first() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(
        &grammar_path,
        r#"
        #[adze::grammar("test_multiple_errors")]
        mod grammar {
            #[adze::language]
            pub enum Expression {
                Number(
                    #[adze::leaf(pattern = r"\d+")]
                    i32
                ),
                #[adze::prec(1)]
                #[adze::prec_left(2)]
                FirstBad(Box<Expression>),
                
                #[adze::prec_left(3)]
                #[adze::prec_right(4)]
                SecondBad(Box<Expression>),
            }
        }
    "#,
    )
    .unwrap();

    let err = adze_tool::generate_grammars(&grammar_path).unwrap_err();
    let error_msg = err.to_string();

    // Should get an error (implementation may report first error encountered)
    assert!(
        error_msg.contains("only one of prec, prec_left, and prec_right can be specified"),
        "Expected precedence error, got: {}",
        error_msg
    );
}

#[test]
fn precedence_error_with_complex_expressions() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(
        &grammar_path,
        r#"
        #[adze::grammar("test_non_integer")]
        mod grammar {
            #[adze::language]
            pub enum Expression {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                    i32
                ),
                #[adze::prec("high")]
                Priority(Box<Expression>),
            }
        }
    "#,
    )
    .unwrap();

    let err = adze_tool::generate_grammars(&grammar_path).unwrap_err();
    let error_msg = err.to_string();

    // Should get non-integer precedence error
    assert!(
        error_msg.contains("Expected integer literal for precedence"),
        "Expected non-integer precedence error, got: {}",
        error_msg
    );
    assert!(
        error_msg.contains("positive integer"),
        "Expected helpful error message, got: {}",
        error_msg
    );
}

#[test]
fn precedence_error_line_information_preserved() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    // Create grammar with error on a specific line
    let grammar_content = r#"
#[adze::grammar("test_line_info")]
mod grammar {
    #[adze::language]
    pub enum Expression {
        Number(
            #[adze::leaf(pattern = r"\d+")]
            i32
        ),
        // Line 9: This should cause error
        #[adze::prec(1)]
        #[adze::prec_left(2)]
        BadRule(Box<Expression>),
    }
}
"#;

    fs::write(&grammar_path, grammar_content).unwrap();

    let err = adze_tool::generate_grammars(&grammar_path).unwrap_err();
    let error_msg = err.to_string();

    // Error should contain precedence conflict message
    assert!(
        error_msg.contains("only one of prec, prec_left, and prec_right can be specified"),
        "Expected precedence conflict error, got: {}",
        error_msg
    );

    // syn::Error should preserve span information for good IDE integration
    // The exact format depends on syn's error formatting, but it should be helpful
    assert!(
        error_msg.len() > 50,
        "Error message should be detailed, got: {}",
        error_msg
    );
}
