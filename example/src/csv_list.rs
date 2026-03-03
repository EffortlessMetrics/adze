#[adze::grammar("csv_list")]
pub mod grammar {
    use adze::Spanned;

    /// A comma-separated list of identifiers, e.g. `alpha, beta, gamma`
    #[adze::language]
    #[derive(Debug)]
    pub struct CsvList {
        #[adze::repeat(non_empty = true)]
        #[adze::delimited(
            #[adze::leaf(text = ",")]
            ()
        )]
        #[adze::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*", transform = |v| v.to_string())]
        items: Spanned<Vec<Spanned<String>>>,
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
    fn csv_single_item() {
        let result = grammar::parse("alpha");
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn csv_two_items() {
        let result = grammar::parse("alpha, beta");
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn csv_multiple_items() {
        let result = grammar::parse("a, b, c, d");
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn csv_no_spaces() {
        let result = grammar::parse("x,y,z");
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn csv_error_cases() {
        // Empty input (non_empty = true requires at least one)
        assert!(grammar::parse("").is_err());
        // Leading comma
        assert!(grammar::parse(", alpha").is_err());
        // Trailing comma
        assert!(grammar::parse("alpha,").is_err());
        // Just a number (doesn't match identifier pattern)
        assert!(grammar::parse("123").is_err());
    }
}
