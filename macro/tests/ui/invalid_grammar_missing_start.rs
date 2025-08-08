//! Test case: Grammar without a start symbol should fail

use rust_sitter::rust_sitter;

#[rust_sitter::grammar("test")]
pub struct Grammar {
    // Missing #[rust_sitter::language] attribute
    pub expr: Expr,
}

pub enum Expr {
    Number(String),
    Add(Box<Expr>, Box<Expr>),
}

fn main() {}