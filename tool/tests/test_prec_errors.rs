use std::fs;
use tempfile::tempdir;

#[test]
fn invalid_precedence_combination() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(&grammar_path, r#"
        #[rust_sitter::grammar("test_combination")]
        mod grammar {
            #[rust_sitter::language]
            pub enum Expression {
                Number(
                    #[rust_sitter::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                    i32
                ),
                #[rust_sitter::prec(1)]
                #[rust_sitter::prec_left(2)]
                Add(
                    Box<Expression>,
                    #[rust_sitter::leaf(text = "+")]
                    (),
                    Box<Expression>,
                ),
            }
        }
    "#).unwrap();

    let err = rust_sitter_tool::generate_grammars(&grammar_path).unwrap_err();
    assert!(err
        .to_string()
        .contains("only one of prec, prec_left, and prec_right can be specified"));
}

#[test]
fn non_integer_precedence_literal() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(&grammar_path, r#"
        #[rust_sitter::grammar("test_non_integer")]
        mod grammar {
            #[rust_sitter::language]
            pub enum Expression {
                Number(
                    #[rust_sitter::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                    i32
                ),
                #[rust_sitter::prec("high")]
                Priority(Box<Expression>),
            }
        }
    "#).unwrap();

    let err = rust_sitter_tool::generate_grammars(&grammar_path).unwrap_err();
    assert!(err
        .to_string()
        .contains("Expected integer literal for precedence"));
}
