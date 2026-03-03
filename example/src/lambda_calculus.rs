/// Lambda calculus grammar demonstrating recursive enums with
/// keyword literals, backslash-dot abstraction syntax, and let bindings.
#[adze::grammar("lambda_calculus")]
pub mod grammar {
    /// Lambda calculus expressions
    #[adze::language]
    #[derive(PartialEq, Eq, Debug, Clone)]
    pub enum Expr {
        /// Variable reference: an identifier
        Var(#[adze::leaf(pattern = r"[a-z][a-z0-9]*")] String),

        /// Lambda abstraction: \x.body
        Abs(
            #[adze::leaf(text = r"\")] (),
            #[adze::leaf(pattern = r"[a-z][a-z0-9]*")] String,
            #[adze::leaf(text = ".")] (),
            Box<Expr>,
        ),

        /// Application: f x (left associative)
        #[adze::prec_left(1)]
        App(Box<Expr>, Box<Expr>),

        /// Let binding: let x = e1 in e2
        Let(
            #[adze::leaf(text = "let")] (),
            #[adze::word]
            #[adze::leaf(pattern = r"[a-z][a-z0-9]*")]
            String,
            #[adze::leaf(text = "=")] (),
            Box<Expr>,
            #[adze::leaf(text = "in")] (),
            #[adze::word] Box<Expr>,
        ),

        /// Parenthesized expression
        Paren(
            #[adze::leaf(text = "(")] (),
            Box<Expr>,
            #[adze::leaf(text = ")")] (),
        ),
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
    fn lambda_variable() {
        insta::assert_debug_snapshot!(grammar::parse("x"));
    }

    #[test]
    fn lambda_abstraction() {
        insta::assert_debug_snapshot!(grammar::parse(r"\x.x"));
    }

    #[test]
    fn lambda_application() {
        insta::assert_debug_snapshot!(grammar::parse(r"(\x.x) y"));
    }

    #[test]
    fn lambda_let_binding() {
        insta::assert_debug_snapshot!(grammar::parse(r"let f = \x.x in f y"));
    }

    #[test]
    fn lambda_error_cases() {
        assert!(grammar::parse("").is_err());
        assert!(grammar::parse(r"\").is_err());
        assert!(grammar::parse("let").is_err());
    }
}
