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
}
