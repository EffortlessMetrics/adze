// For pure-rust: Include and re-export the generated parser symbols
#[cfg(feature = "pure-rust")]
pub mod generated {
    include!(concat!(
        env!("OUT_DIR"),
        "/grammar_typed_ast_contract/parser_typed_ast_contract.rs"
    ));
}

#[cfg(feature = "pure-rust")]
pub use generated::{LANGUAGE, SMALL_PARSE_TABLE, SMALL_PARSE_TABLE_MAP};

#[adze::grammar("typed_ast_contract")]
pub mod grammar {
    #[adze::language]
    #[derive(Debug, PartialEq, Eq)]
    pub enum Expr {
        Number(#[adze::leaf(pattern = r"\d+", transform = |s| s.parse().unwrap())] i32),

        #[adze::prec_left(1)]
        Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _ws: (),
    }
}
