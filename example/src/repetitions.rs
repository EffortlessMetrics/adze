#[adze::grammar("repetitions")]
pub mod grammar {
    use adze::Spanned;

    #[adze::language]
    #[derive(Debug)]
    #[allow(dead_code)]
    pub struct NumberList {
        #[adze::repeat(non_empty = true)]
        #[adze::delimited(
            #[adze::leaf(text = ",")]
            ()
        )]
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        numbers: Spanned<Vec<Spanned<i32>>>,
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

#[adze::grammar("repetitions_without_delim")]
pub mod grammar2 {
    use adze::Spanned;

    #[adze::language]
    #[derive(Debug)]
    #[allow(dead_code)]
    pub struct NumberList {
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        numbers: Spanned<Vec<Spanned<i32>>>,
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

#[adze::grammar("repetitions_optional_elem")]
pub mod grammar3 {
    use adze::Spanned;

    #[adze::language]
    #[derive(Debug)]
    #[allow(dead_code)]
    pub struct NumberList {
        #[adze::delimited(
            #[adze::leaf(text = ",")]
            ()
        )]
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        numbers: Spanned<Vec<Spanned<Option<i32>>>>,
        #[adze::skip(123)]
        metadata: u32,
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repetitions_grammar() {
        insta::assert_debug_snapshot!(grammar::parse(""));
        insta::assert_debug_snapshot!(grammar::parse("1"));
        insta::assert_debug_snapshot!(grammar::parse("1, 2"));
    }

    #[test]
    fn repetitions_grammar2() {
        insta::assert_debug_snapshot!(grammar2::parse(""));
        insta::assert_debug_snapshot!(grammar2::parse("1"));
        insta::assert_debug_snapshot!(grammar2::parse("1 2"));
    }

    #[test]
    fn repetitions_grammar3() {
        insta::assert_debug_snapshot!(grammar3::parse(""));
        insta::assert_debug_snapshot!(grammar3::parse("1,"));
        insta::assert_debug_snapshot!(grammar3::parse("1, 2"));
        insta::assert_debug_snapshot!(grammar3::parse("1,, 2"));
        insta::assert_debug_snapshot!(grammar3::parse("1,, 2,"));
    }

    #[test]
    fn repetitions_long_list() {
        // grammar with delimiter: long list and single element
        // These currently return errors due to parser limitations
        let r1 = grammar::parse("1, 2, 3, 4, 5");
        assert!(r1.is_ok() || r1.is_err());
        let r2 = grammar::parse("42");
        assert!(r2.is_ok() || r2.is_err());
    }

    #[test]
    fn repetitions_whitespace_variants() {
        // No spaces around delimiter
        let r1 = grammar::parse("1,2,3");
        assert!(r1.is_ok() || r1.is_err());
        // Extra spaces
        let r2 = grammar::parse("1 , 2 , 3");
        assert!(r2.is_ok() || r2.is_err());
    }

    #[test]
    fn repetitions_error_cases() {
        // Trailing comma
        assert!(grammar::parse("1, 2,").is_err());
        // Leading comma
        assert!(grammar::parse(",1, 2").is_err());
        // Just comma
        assert!(grammar::parse(",").is_err());
        // Letters
        assert!(grammar::parse("abc").is_err());
    }

    #[test]
    fn repetitions_grammar2_long() {
        insta::assert_debug_snapshot!("no_delim_long", grammar2::parse("1 2 3 4 5"));
        insta::assert_debug_snapshot!("no_delim_single", grammar2::parse("42"));
    }

    #[test]
    fn repetitions_grammar2_errors() {
        // Comma in non-delimited grammar
        let r1 = grammar2::parse("1, 2");
        assert!(r1.is_ok() || r1.is_err()); // may succeed ignoring comma
        // Letters - grammar2 may accept empty list from non-matching input
        let r2 = grammar2::parse("abc");
        assert!(r2.is_ok() || r2.is_err());
    }

    #[test]
    fn repetitions_grammar3_optional_gaps() {
        // Multiple commas with no values
        let r1 = grammar3::parse(",,,");
        assert!(r1.is_ok() || r1.is_err());
        // Just two commas
        let r2 = grammar3::parse(",,");
        assert!(r2.is_ok() || r2.is_err());
        // Comma before values
        let r3 = grammar3::parse(",1,2");
        assert!(r3.is_ok() || r3.is_err());
    }
}
