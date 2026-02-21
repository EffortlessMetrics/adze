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
}
