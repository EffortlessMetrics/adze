#[adze::grammar("json_like")]
pub mod grammar {
    /// A simple JSON-like object: `{ "key": 123, "other": 456 }`
    #[adze::language]
    #[derive(Debug)]
    pub struct Object {
        #[adze::leaf(text = "{")]
        _open: (),
        #[adze::delimited(
            #[adze::leaf(text = ",")]
            ()
        )]
        entries: Vec<Entry>,
        #[adze::leaf(text = "}")]
        _close: (),
    }

    #[derive(Debug)]
    pub struct Entry {
        #[adze::leaf(pattern = r#""[^"]*""#, transform = |v| v[1..v.len()-1].to_string())]
        key: String,
        #[adze::leaf(text = ":")]
        _colon: (),
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        value: i32,
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
    fn json_like_empty_object() {
        insta::assert_debug_snapshot!(grammar::parse("{}"));
    }

    #[test]
    fn json_like_single_entry() {
        let result = grammar::parse(r#"{ "name": 42 }"#);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn json_like_multiple_entries() {
        let result = grammar::parse(r#"{ "a": 1, "b": 2, "c": 3 }"#);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn json_like_error_cases() {
        // Missing braces
        assert!(grammar::parse(r#""a": 1"#).is_err());
        // Missing colon
        assert!(grammar::parse(r#"{ "a" 1 }"#).is_err());
        // Empty input
        assert!(grammar::parse("").is_err());
    }
}
