//! Test case: Valid simple grammar should compile

use rust_sitter::rust_sitter;

#[rust_sitter::grammar("arithmetic")]
#[rust_sitter::language]
pub struct Arithmetic {
    pub expr: Expr,
}

#[derive(Debug)]
pub enum Expr {
    #[rust_sitter::leaf(text = r"[0-9]+")]
    Number(String),

    #[rust_sitter::prec_left(1)]
    Add(Box<Expr>, #[rust_sitter::leaf(text = "+")] (), Box<Expr>),

    #[rust_sitter::prec_left(2)]
    Mul(Box<Expr>, #[rust_sitter::leaf(text = "*")] (), Box<Expr>),
}

fn main() {
    // This should compile successfully
}
