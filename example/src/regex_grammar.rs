/// Simplified regex grammar demonstrating alternation, repetition
/// quantifiers, character classes, and grouping.
#[adze::grammar("regex_grammar")]
pub mod grammar {
    /// Top-level regex: alternation of sequences
    #[adze::language]
    #[derive(PartialEq, Eq, Debug, Clone)]
    pub enum Regex {
        /// Alternation: a|b (lowest precedence)
        #[adze::prec_left(1)]
        Alt(Box<Regex>, #[adze::leaf(text = "|")] (), Box<Regex>),

        /// Concatenation of atoms (implicit sequencing)
        #[adze::prec_left(2)]
        Concat(Box<Regex>, Box<Regex>),

        /// Kleene star: a*
        #[adze::prec_left(3)]
        Star(Box<Regex>, #[adze::leaf(text = "*")] ()),

        /// One-or-more: a+
        #[adze::prec_left(3)]
        Plus(Box<Regex>, #[adze::leaf(text = "+")] ()),

        /// Optional: a?
        #[adze::prec_left(3)]
        Opt(Box<Regex>, #[adze::leaf(text = "?")] ()),

        /// Character class: [a-z]
        CharClass(
            #[adze::leaf(text = "[")] (),
            #[adze::leaf(pattern = r"[a-zA-Z0-9]")] String,
            #[adze::leaf(text = "-")] (),
            #[adze::leaf(pattern = r"[a-zA-Z0-9]")] String,
            #[adze::leaf(text = "]")] (),
        ),

        /// Grouped expression: (ab)
        Group(
            #[adze::leaf(text = "(")] (),
            Box<Regex>,
            #[adze::leaf(text = ")")] (),
        ),

        /// Literal character
        Lit(#[adze::leaf(pattern = r"[a-zA-Z0-9]")] String),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_literal() {
        insta::assert_debug_snapshot!(grammar::parse("a"));
    }

    #[test]
    fn regex_alternation() {
        insta::assert_debug_snapshot!(grammar::parse("a|b"));
    }

    #[test]
    fn regex_star() {
        insta::assert_debug_snapshot!(grammar::parse("a*"));
    }

    #[test]
    fn regex_group_plus() {
        insta::assert_debug_snapshot!(grammar::parse("(ab)+"));
    }

    #[test]
    fn regex_char_class() {
        insta::assert_debug_snapshot!(grammar::parse("[a-z]"));
    }

    #[test]
    fn regex_error_cases() {
        assert!(grammar::parse("").is_err());
        assert!(grammar::parse("[").is_err());
        assert!(grammar::parse("(").is_err());
    }
}
