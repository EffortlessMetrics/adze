//! Test case: Grammar without a start symbol should fail

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
