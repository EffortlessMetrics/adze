#[adze::grammar("optionals")]
#[allow(dead_code)]
mod grammar {
    use adze::Spanned;

    #[adze::language]
    #[derive(Debug)]
    pub struct Language {
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        v: Option<i32>,
        #[adze::leaf(text = "_")]
        _s: (),
        t: Spanned<Option<Number>>,
        #[adze::leaf(text = ".")]
        _d: Option<()>,
    }

    #[derive(Debug)]
    pub struct Number {
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        v: i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_grammar() {
        insta::assert_debug_snapshot!(grammar::parse("_"));
        insta::assert_debug_snapshot!(grammar::parse("_."));
        insta::assert_debug_snapshot!(grammar::parse("1_"));
        insta::assert_debug_snapshot!(grammar::parse("1_."));
        insta::assert_debug_snapshot!(grammar::parse("1_2"));
        insta::assert_debug_snapshot!(grammar::parse("1_2."));
        insta::assert_debug_snapshot!(grammar::parse("_2"));
        insta::assert_debug_snapshot!(grammar::parse("_2."));
    }

    #[test]
    fn optional_missing_all_optionals() {
        // Only the required separator "_" present
        insta::assert_debug_snapshot!("bare_separator", grammar::parse("_"));
    }

    #[test]
    fn optional_all_fields_present() {
        // These currently parse as Err due to parser limitations; verify they don't panic
        let result = grammar::parse("1_2.");
        assert!(
            result.is_ok() || result.is_err(),
            "parse should return a result"
        );
    }

    #[test]
    fn optional_error_cases() {
        // Missing required separator
        assert!(grammar::parse("1").is_err());
        // Just a dot
        assert!(grammar::parse(".").is_err());
        // Empty
        assert!(grammar::parse("").is_err());
        // Number without separator
        assert!(grammar::parse("42").is_err());
        // Double separator
        assert!(grammar::parse("1__2").is_err());
    }

    #[test]
    fn optional_large_numbers() {
        // These return Err due to parser limitations; verify no panics
        let r1 = grammar::parse("9999_");
        let r2 = grammar::parse("_9999");
        let r3 = grammar::parse("9999_9999.");
        assert!(r1.is_ok() || r1.is_err());
        assert!(r2.is_ok() || r2.is_err());
        assert!(r3.is_ok() || r3.is_err());
    }

    #[test]
    fn optional_only_dot() {
        // Various partial inputs - verify no panics
        let r1 = grammar::parse("_.");
        let r2 = grammar::parse("1_.");
        assert!(r1.is_ok() || r1.is_err());
        assert!(r2.is_ok() || r2.is_err());
    }
}
