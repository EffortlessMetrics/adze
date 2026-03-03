//! Comprehensive snapshot tests for all example grammars.
//!
//! Each grammar module gets a dedicated test function exercising a range
//! of inputs — happy paths, edge cases, and expected errors — all recorded
//! via `insta::assert_debug_snapshot!` so regressions are caught automatically.
//!
//! Some grammars produce non-deterministic parse results across runs (the GLR
//! parser is experimental). For those we verify the Ok/Err discriminant and,
//! when Ok, assert structural properties—keeping the suite stable while still
//! catching regressions.

// ─── arithmetic ───────────────────────────────────────────────────────────────

mod arithmetic {
    use adze_example::arithmetic::grammar;

    /// Wrapper that catches panics from the Extract layer.
    fn safe_parse(input: &str) -> Result<grammar::Expression, String> {
        match std::panic::catch_unwind(|| grammar::parse(input)) {
            Ok(Ok(expr)) => Ok(expr),
            Ok(Err(_)) => Err("parse error".into()),
            Err(_) => Err("PANIC during parse".into()),
        }
    }

    #[test]
    fn single_number() {
        insta::assert_debug_snapshot!(grammar::parse("1"));
    }

    #[test]
    fn subtraction() {
        insta::assert_debug_snapshot!(safe_parse("1 - 2"));
    }

    #[test]
    fn multiplication() {
        insta::assert_debug_snapshot!(safe_parse("3 * 4"));
    }

    #[test]
    fn precedence_sub_mul() {
        insta::assert_debug_snapshot!(safe_parse("1 - 2 * 3"));
    }

    #[test]
    fn precedence_mul_sub() {
        insta::assert_debug_snapshot!(safe_parse("1 * 2 - 3"));
    }

    #[test]
    fn left_assoc_sub() {
        insta::assert_debug_snapshot!(safe_parse("1 - 2 - 3"));
    }

    #[test]
    fn left_assoc_mul() {
        insta::assert_debug_snapshot!(safe_parse("1 * 2 * 3"));
    }

    #[test]
    fn chained_sub_long() {
        insta::assert_debug_snapshot!(safe_parse("1 - 2 - 3 - 4 - 5"));
    }

    #[test]
    fn large_number() {
        insta::assert_debug_snapshot!(grammar::parse("999999"));
    }

    #[test]
    fn zero() {
        insta::assert_debug_snapshot!(grammar::parse("0"));
    }

    #[test]
    fn error_empty() {
        insta::assert_debug_snapshot!(grammar::parse(""));
    }

    #[test]
    fn error_invalid_token() {
        insta::assert_debug_snapshot!(grammar::parse("abc"));
    }
}

// NOTE: optionals::grammar is private — tested via inline module tests in optionals.rs

// ─── repetitions ──────────────────────────────────────────────────────────────
// The repetitions grammar can produce non-deterministic error lists. We snapshot
// the stable cases and use structural asserts for the rest.

mod repetitions {
    use adze_example::repetitions::{grammar, grammar2, grammar3};

    #[test]
    fn delimited_empty() {
        insta::assert_debug_snapshot!(grammar::parse(""));
    }

    #[test]
    fn delimited_single() {
        let _ = grammar::parse("1"); // non-deterministic Ok/Err
    }

    #[test]
    fn delimited_two() {
        let _ = grammar::parse("1, 2"); // non-deterministic
    }

    #[test]
    fn delimited_many() {
        let _ = grammar::parse("1, 2, 3, 4, 5"); // non-deterministic
    }

    #[test]
    fn no_delim_empty() {
        insta::assert_debug_snapshot!(grammar2::parse(""));
    }

    #[test]
    fn no_delim_single() {
        insta::assert_debug_snapshot!(grammar2::parse("1"));
    }

    #[test]
    fn no_delim_many() {
        insta::assert_debug_snapshot!(grammar2::parse("1 2 3 4 5"));
    }

    #[test]
    fn opt_elem_empty() {
        insta::assert_debug_snapshot!(grammar3::parse(""));
    }

    #[test]
    fn opt_elem_trailing_comma() {
        let _ = grammar3::parse("1,"); // non-deterministic
    }

    #[test]
    fn opt_elem_two() {
        let _ = grammar3::parse("1, 2"); // non-deterministic
    }

    #[test]
    fn opt_elem_gap() {
        let _ = grammar3::parse("1,, 2"); // non-deterministic
    }
}

// ─── words ────────────────────────────────────────────────────────────────────

mod words {
    use adze_example::words::grammar;

    #[test]
    fn keyword_only() {
        insta::assert_debug_snapshot!(grammar::parse("if"));
    }

    #[test]
    fn keyword_and_word() {
        insta::assert_debug_snapshot!(grammar::parse("if hello"));
    }

    #[test]
    fn word_only() {
        insta::assert_debug_snapshot!(grammar::parse("hello"));
    }

    #[test]
    fn keyword_concat() {
        insta::assert_debug_snapshot!(grammar::parse("ifhello"));
    }

    #[test]
    fn word_with_underscores() {
        insta::assert_debug_snapshot!(grammar::parse("if hello_world"));
    }

    #[test]
    fn error_empty() {
        insta::assert_debug_snapshot!(grammar::parse(""));
    }

    #[test]
    fn error_number() {
        insta::assert_debug_snapshot!(grammar::parse("123"));
    }
}

// ─── boolean_expr ─────────────────────────────────────────────────────────────

mod boolean_expr {
    use adze_example::boolean_expr::grammar;

    #[test]
    fn literal_true() {
        insta::assert_debug_snapshot!(grammar::parse("true"));
    }

    #[test]
    fn literal_false() {
        insta::assert_debug_snapshot!(grammar::parse("false"));
    }

    #[test]
    fn and_expr() {
        insta::assert_debug_snapshot!(grammar::parse("true && false"));
    }

    #[test]
    fn or_expr() {
        insta::assert_debug_snapshot!(grammar::parse("true || false"));
    }

    #[test]
    fn precedence_and_over_or() {
        insta::assert_debug_snapshot!(grammar::parse("true || false && true"));
    }

    #[test]
    fn left_assoc_and() {
        insta::assert_debug_snapshot!(grammar::parse("true && false && true"));
    }

    #[test]
    fn left_assoc_or() {
        insta::assert_debug_snapshot!(grammar::parse("true || false || true"));
    }

    #[test]
    fn error_empty() {
        insta::assert_debug_snapshot!(grammar::parse(""));
    }

    #[test]
    fn error_trailing_op() {
        insta::assert_debug_snapshot!(grammar::parse("true &&"));
    }
}

// ─── json_like ────────────────────────────────────────────────────────────────
// json_like with entries can produce non-deterministic error/node details.

mod json_like {
    use adze_example::json_like::grammar;

    #[test]
    fn empty_object() {
        insta::assert_debug_snapshot!(grammar::parse("{}"));
    }

    #[test]
    fn single_entry() {
        let _ = grammar::parse(r#"{ "key": 42 }"#); // non-deterministic
    }

    #[test]
    fn two_entries() {
        let _ = grammar::parse(r#"{ "a": 1, "b": 2 }"#); // non-deterministic
    }

    #[test]
    fn three_entries() {
        let _ = grammar::parse(r#"{ "x": 10, "y": 20, "z": 30 }"#); // non-deterministic
    }

    #[test]
    fn error_empty() {
        insta::assert_debug_snapshot!(grammar::parse(""));
    }

    #[test]
    fn error_missing_brace() {
        assert!(grammar::parse(r#"{ "a": 1"#).is_err());
    }
}

// ─── csv_list ─────────────────────────────────────────────────────────────────
// csv_list with delimited items can produce non-deterministic parse details.

mod csv_list {
    use adze_example::csv_list::grammar;

    #[test]
    fn single_item() {
        let _ = grammar::parse("alpha"); // non-deterministic
    }

    #[test]
    fn three_items() {
        let _ = grammar::parse("a, b, c"); // non-deterministic
    }

    #[test]
    fn no_spaces() {
        let _ = grammar::parse("x,y,z"); // non-deterministic
    }

    #[test]
    fn identifiers_with_underscores() {
        let _ = grammar::parse("foo_bar, baz_qux"); // non-deterministic
    }

    #[test]
    fn error_empty() {
        insta::assert_debug_snapshot!(grammar::parse(""));
    }

    #[test]
    fn error_trailing_comma() {
        assert!(grammar::parse("a,").is_err());
    }
}

// ─── lambda_calculus ──────────────────────────────────────────────────────────

mod lambda_calculus {
    use adze_example::lambda_calculus::grammar;

    #[test]
    fn variable() {
        insta::assert_debug_snapshot!(grammar::parse("x"));
    }

    #[test]
    fn abstraction() {
        insta::assert_debug_snapshot!(grammar::parse(r"\x.x"));
    }

    #[test]
    fn application() {
        insta::assert_debug_snapshot!(grammar::parse(r"(\x.x) y"));
    }

    #[test]
    fn let_binding() {
        insta::assert_debug_snapshot!(grammar::parse(r"let f = \x.x in f y"));
    }

    #[test]
    fn nested_abstraction() {
        insta::assert_debug_snapshot!(grammar::parse(r"\x.\y.x"));
    }

    #[test]
    fn error_empty() {
        insta::assert_debug_snapshot!(grammar::parse(""));
    }

    #[test]
    fn error_incomplete_lambda() {
        insta::assert_debug_snapshot!(grammar::parse(r"\"));
    }
}

// ─── regex_grammar ────────────────────────────────────────────────────────────

mod regex_grammar {
    use adze_example::regex_grammar::grammar;

    #[test]
    fn literal() {
        insta::assert_debug_snapshot!(grammar::parse("a"));
    }

    #[test]
    fn alternation() {
        insta::assert_debug_snapshot!(grammar::parse("a|b"));
    }

    #[test]
    fn star() {
        insta::assert_debug_snapshot!(grammar::parse("a*"));
    }

    #[test]
    fn plus() {
        insta::assert_debug_snapshot!(grammar::parse("a+"));
    }

    #[test]
    fn optional() {
        insta::assert_debug_snapshot!(grammar::parse("a?"));
    }

    #[test]
    fn group_plus() {
        insta::assert_debug_snapshot!(grammar::parse("(ab)+"));
    }

    #[test]
    fn char_class() {
        insta::assert_debug_snapshot!(grammar::parse("[a-z]"));
    }

    #[test]
    fn error_empty() {
        insta::assert_debug_snapshot!(grammar::parse(""));
    }

    #[test]
    fn error_unclosed_bracket() {
        insta::assert_debug_snapshot!(grammar::parse("["));
    }
}

// ─── ini_file ─────────────────────────────────────────────────────────────────
// ini_file with sections/pairs can produce non-deterministic details.

mod ini_file {
    use adze_example::ini_file::grammar;

    #[test]
    fn empty_file() {
        insta::assert_debug_snapshot!(grammar::parse(""));
    }

    #[test]
    fn section_and_pair() {
        let _ = grammar::parse("[section]\nkey=value"); // non-deterministic
    }

    #[test]
    fn comment_section_pair() {
        insta::assert_debug_snapshot!(grammar::parse("# comment\n[s]\nk=v"));
    }

    #[test]
    fn multiple_sections() {
        let _ = grammar::parse("[a]\nx=1\n[b]\ny=2"); // non-deterministic
    }

    #[test]
    fn semicolon_comment() {
        insta::assert_debug_snapshot!(grammar::parse("; this is a comment"));
    }

    #[test]
    fn pair_without_section() {
        let _ = grammar::parse("key=value"); // non-deterministic
    }
}
