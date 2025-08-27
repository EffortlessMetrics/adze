#![allow(clippy::empty_line_after_outer_attr, clippy::unnecessary_cast)]

#[rust_sitter::grammar("mini")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct Program {
        #[rust_sitter::leaf(pattern = r"\d+", text = true)]
        pub number: String,
    }
}

#[cfg(test)]
mod tests {
    use crate::grammar;

    #[test]
    fn test_number() {
        let result = grammar::parse("42");
        assert!(result.is_ok());
        let program: grammar::Program = result.unwrap();
        assert_eq!(program.number, "42");
    }

    #[test]
    fn test_multi_digit_number() {
        let result = grammar::parse("12345");
        assert!(result.is_ok());
        let program: grammar::Program = result.unwrap();
        assert_eq!(program.number, "12345");
    }

    #[test]
    fn test_single_digit() {
        let result = grammar::parse("0");
        assert!(result.is_ok());
        let program: grammar::Program = result.unwrap();
        assert_eq!(program.number, "0");
    }

    #[test]
    fn test_empty_input() {
        let result = grammar::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_non_number() {
        let result = grammar::parse("abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_number_with_trailing_text() {
        // The parser successfully parses "42" and ignores the trailing "abc"
        // This is expected behavior - the parser consumes what it can
        let result = grammar::parse("42abc");
        assert!(result.is_ok());
        let program: grammar::Program = result.unwrap();
        assert_eq!(program.number, "42");
    }
}
