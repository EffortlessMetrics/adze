use std::fs;
use tempfile::tempdir;

#[test]
fn invalid_precedence_combination() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(
        &grammar_path,
        r#"
        #[adze::grammar("test_combination")]
        mod grammar {
            #[adze::language]
            pub enum Expression {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
                    i32
                ),
                #[adze::prec(1)]
                #[adze::prec_left(2)]
                Add(
                    Box<Expression>,
                    #[adze::leaf(text = "+")]
                    (),
                    Box<Expression>,
                ),
            }
        }
    "#,
    )
    .unwrap();

    let err = adze_tool::generate_grammars(&grammar_path).unwrap_err();
    let error_msg = err.to_string();
    assert!(
        error_msg.contains("only one of prec, prec_left, and prec_right can be specified"),
        "Expected precedence conflict error, got: {}",
        error_msg
    );
}

#[test]
fn precedence_with_prec_and_prec_right() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(
        &grammar_path,
        r#"
        #[adze::grammar("test_prec_right_combo")]
        mod grammar {
            #[adze::language]
            pub enum Expression {
                Number(
                    #[adze::leaf(pattern = r"\d+")]
                    i32
                ),
                #[adze::prec(5)]
                #[adze::prec_right(10)]
                Power(
                    Box<Expression>,
                    #[adze::leaf(text = "^")]
                    (),
                    Box<Expression>,
                ),
            }
        }
    "#,
    )
    .unwrap();

    let err = adze_tool::generate_grammars(&grammar_path).unwrap_err();
    let error_msg = err.to_string();
    assert!(
        error_msg.contains("only one of prec, prec_left, and prec_right can be specified"),
        "Expected precedence conflict error, got: {}",
        error_msg
    );
}

#[test]
fn all_three_precedence_attributes() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(
        &grammar_path,
        r#"
        #[adze::grammar("test_all_three")]
        mod grammar {
            #[adze::language]
            pub enum Expression {
                Number(
                    #[adze::leaf(pattern = r"\d+")]
                    i32
                ),
                #[adze::prec(1)]
                #[adze::prec_left(2)]
                #[adze::prec_right(3)]
                Ternary(
                    Box<Expression>,
                    #[adze::leaf(text = "?")]
                    (),
                    Box<Expression>,
                    #[adze::leaf(text = ":")]
                    (),
                    Box<Expression>,
                ),
            }
        }
    "#,
    )
    .unwrap();

    let err = adze_tool::generate_grammars(&grammar_path).unwrap_err();
    let error_msg = err.to_string();
    assert!(
        error_msg.contains("only one of prec, prec_left, and prec_right can be specified"),
        "Expected precedence conflict error, got: {}",
        error_msg
    );
}

#[test]
fn non_integer_precedence_literal() {
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
    assert!(
        error_msg.contains("Expected integer literal for precedence"),
        "Expected non-integer precedence error, got: {}",
        error_msg
    );
}

#[test]
fn non_integer_prec_left_literal() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(
        &grammar_path,
        r#"
        #[adze::grammar("test_non_integer_left")]
        mod grammar {
            #[adze::language]
            pub enum Expression {
                Number(
                    #[adze::leaf(pattern = r"\d+")]
                    i32
                ),
                #[adze::prec_left(3.14)]
                Float(Box<Expression>),
            }
        }
    "#,
    )
    .unwrap();

    let err = adze_tool::generate_grammars(&grammar_path).unwrap_err();
    let error_msg = err.to_string();
    assert!(
        error_msg.contains("Expected integer literal for left-associative precedence"),
        "Expected non-integer precedence error, got: {}",
        error_msg
    );
}

#[test]
fn non_integer_prec_right_literal() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(
        &grammar_path,
        r#"
        #[adze::grammar("test_non_integer_right")]
        mod grammar {
            #[adze::language]
            pub enum Expression {
                Number(
                    #[adze::leaf(pattern = r"\d+")]
                    i32
                ),
                #[adze::prec_right(true)]
                Boolean(Box<Expression>),
            }
        }
    "#,
    )
    .unwrap();

    let err = adze_tool::generate_grammars(&grammar_path).unwrap_err();
    let error_msg = err.to_string();
    assert!(
        error_msg.contains("Expected integer literal for right-associative precedence"),
        "Expected non-integer precedence error, got: {}",
        error_msg
    );
}

#[test]
fn precedence_with_variable_reference() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(
        &grammar_path,
        r#"
        #[adze::grammar("test_variable_ref")]
        mod grammar {
            const HIGH_PREC: u32 = 10;
            
            #[adze::language]
            pub enum Expression {
                Number(
                    #[adze::leaf(pattern = r"\d+")]
                    i32
                ),
                #[adze::prec(HIGH_PREC)]
                HighPriority(Box<Expression>),
            }
        }
    "#,
    )
    .unwrap();

    let err = adze_tool::generate_grammars(&grammar_path).unwrap_err();
    let error_msg = err.to_string();
    assert!(
        error_msg.contains("Expected integer literal for precedence"),
        "Expected non-literal precedence error, got: {}",
        error_msg
    );
}

#[test]
fn precedence_with_negative_number() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(
        &grammar_path,
        r#"
        #[adze::grammar("test_negative")]
        mod grammar {
            #[adze::language]
            pub enum Expression {
                Number(
                    #[adze::leaf(pattern = r"\d+")]
                    i32
                ),
                #[adze::prec(-5)]
                Negative(Box<Expression>),
            }
        }
    "#,
    )
    .unwrap();

    // This should fail at parse time since we expect u32
    let err = adze_tool::generate_grammars(&grammar_path).unwrap_err();
    let error_msg = err.to_string();
    // Either parsing fails or we get an invalid integer error
    assert!(
        error_msg.contains("Invalid integer literal")
            || error_msg.contains("Expected integer literal")
            || error_msg.contains("number too large"),
        "Expected negative precedence error, got: {}",
        error_msg
    );
}

#[test]
fn precedence_with_zero() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(
        &grammar_path,
        r#"
        #[adze::grammar("test_zero_prec")]
        mod grammar {
            #[adze::language]
            pub enum Expression {
                Number(
                    #[adze::leaf(pattern = r"\d+")]
                    i32
                ),
                #[adze::prec(0)]
                Zero(Box<Expression>),
            }
        }
    "#,
    )
    .unwrap();

    // Zero precedence should be valid
    let grammars = adze_tool::generate_grammars(&grammar_path).unwrap();
    assert!(
        !grammars.is_empty(),
        "Should generate grammar with zero precedence"
    );
    let grammar = &grammars[0];

    // Check that the precedence is properly applied
    let zero_rule = grammar["rules"]["Expression_Zero"].as_object().unwrap();
    assert_eq!(zero_rule["type"], "PREC");
    assert_eq!(zero_rule["value"], 0);
}

#[test]
fn precedence_with_very_large_number() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(
        &grammar_path,
        r#"
        #[adze::grammar("test_large_prec")]
        mod grammar {
            #[adze::language]
            pub enum Expression {
                Number(
                    #[adze::leaf(pattern = r"\d+")]
                    i32
                ),
                #[adze::prec(4294967295)]
                MaxPrec(Box<Expression>),
            }
        }
    "#,
    )
    .unwrap();

    // Max u32 should be valid
    let grammars = adze_tool::generate_grammars(&grammar_path).unwrap();
    assert!(
        !grammars.is_empty(),
        "Should generate grammar with max precedence"
    );
    let grammar = &grammars[0];

    // Check that the precedence is properly applied
    let max_rule = grammar["rules"]["Expression_MaxPrec"].as_object().unwrap();
    assert_eq!(max_rule["type"], "PREC");
    assert_eq!(max_rule["value"], 4294967295_u64);
}

#[test]
fn precedence_too_large_for_u32() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(
        &grammar_path,
        r#"
        #[adze::grammar("test_too_large")]
        mod grammar {
            #[adze::language]
            pub enum Expression {
                Number(
                    #[adze::leaf(pattern = r"\d+")]
                    i32
                ),
                #[adze::prec(4294967296)]
                TooBig(Box<Expression>),
            }
        }
    "#,
    )
    .unwrap();

    let err = adze_tool::generate_grammars(&grammar_path).unwrap_err();
    let error_msg = err.to_string();
    assert!(
        error_msg.contains("Invalid integer literal") || error_msg.contains("number too large"),
        "Expected overflow error, got: {}",
        error_msg
    );
}

#[test]
fn valid_precedence_combinations() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(
        &grammar_path,
        r#"
        #[adze::grammar("test_valid_precs")]
        mod grammar {
            #[adze::language]
            pub enum Expression {
                Number(
                    #[adze::leaf(pattern = r"\d+")]
                    i32
                ),
                #[adze::prec(1)]
                Addition(
                    Box<Expression>,
                    #[adze::leaf(text = "+")]
                    (),
                    Box<Expression>,
                ),
                #[adze::prec_left(2)]
                Multiplication(
                    Box<Expression>,
                    #[adze::leaf(text = "*")]
                    (),
                    Box<Expression>,
                ),
                #[adze::prec_right(3)]
                Exponentiation(
                    Box<Expression>,
                    #[adze::leaf(text = "^")]
                    (),
                    Box<Expression>,
                ),
            }
        }
    "#,
    )
    .unwrap();

    // This should succeed
    let grammars = adze_tool::generate_grammars(&grammar_path).unwrap();
    assert!(
        !grammars.is_empty(),
        "Should generate valid grammar with different precedence types"
    );

    let grammar = &grammars[0];
    let rules = grammar["rules"].as_object().unwrap();

    // Verify each precedence type was applied correctly
    let add_rule = &rules["Expression_Addition"];
    assert_eq!(add_rule["type"], "PREC");
    assert_eq!(add_rule["value"], 1);

    let mult_rule = &rules["Expression_Multiplication"];
    assert_eq!(mult_rule["type"], "PREC_LEFT");
    assert_eq!(mult_rule["value"], 2);

    let exp_rule = &rules["Expression_Exponentiation"];
    assert_eq!(exp_rule["type"], "PREC_RIGHT");
    assert_eq!(exp_rule["value"], 3);
}
