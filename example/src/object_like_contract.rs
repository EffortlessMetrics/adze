#[adze::grammar("object_like_contract")]
pub mod grammar {
    /// A small object-like grammar used by generated parse-error contract tests.
    #[adze::language]
    #[derive(Debug)]
    pub struct Object {
        #[adze::leaf(text = "{")]
        _open: (),
        #[adze::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*", transform = |v| v.to_string())]
        key: String,
        #[adze::leaf(text = ":")]
        _colon: (),
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap_or_default())]
        value: i32,
        #[adze::leaf(text = "}")]
        _close: (),
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
    fn object_like_contract_accepts_single_entry() {
        grammar::parse("{ name: 42 }").expect("single-entry object should parse");
    }

    #[test]
    fn object_like_contract_rejects_bad_shapes() {
        assert!(grammar::parse("").is_err());
        assert!(grammar::parse("name: 42 }").is_err());
        assert!(grammar::parse(r#"{ 123: 42 }"#).is_err());
        assert!(grammar::parse("{ name 42 }").is_err());
        assert!(grammar::parse("{ name: nope }").is_err());
    }
}
