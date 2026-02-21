//! Test case: Grammar without a start symbol should fail

#[adze::grammar("test")]
pub struct Grammar {
    // Missing #[adze::language] attribute
    pub expr: Expr,
}

pub enum Expr {
    Number(String),
    Add(Box<Expr>, Box<Expr>),
}

fn main() {}
