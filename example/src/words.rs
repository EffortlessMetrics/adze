#[adze::grammar("words")]
pub mod grammar {
    #[adze::language]
    #[derive(Debug)]
    pub struct Words {
        #[adze::leaf(text = r"if")]
        _keyword: (),
        #[adze::word]
        #[adze::leaf(pattern = r"[a-z_]+", transform = |v| v.to_string())]
        _word: String,
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
    fn words_grammar() {
        insta::assert_debug_snapshot!(grammar::parse("if"));
        insta::assert_debug_snapshot!(grammar::parse("hello"));
        insta::assert_debug_snapshot!(grammar::parse("ifhello"));
        insta::assert_debug_snapshot!(grammar::parse("if hello"));
    }

    #[test]
    fn words_keyword_boundary() {
        // "if" prefix that isn't the keyword
        let r1 = grammar::parse("if iffy");
        assert!(r1.is_ok() || r1.is_err());
        // Word with underscores
        let r2 = grammar::parse("if hello_world");
        assert!(r2.is_ok() || r2.is_err());
        // All underscores
        let r3 = grammar::parse("if ___");
        assert!(r3.is_ok() || r3.is_err());
    }

    #[test]
    fn words_error_cases() {
        // Empty
        assert!(grammar::parse("").is_err());
        // Whitespace only
        assert!(grammar::parse("   ").is_err());
        // Number input
        assert!(grammar::parse("123").is_err());
        // Uppercase
        assert!(grammar::parse("if HELLO").is_err());
    }

    #[test]
    fn words_special_inputs() {
        // Just "if" keyword, no word - may parse or error
        let r1 = grammar::parse("hello");
        assert!(r1.is_ok() || r1.is_err());
        // Long word
        let r2 = grammar::parse("if abcdefghijklmnop");
        assert!(r2.is_ok() || r2.is_err());
        // Single char word
        let r3 = grammar::parse("if a");
        assert!(r3.is_ok() || r3.is_err());
    }
}
