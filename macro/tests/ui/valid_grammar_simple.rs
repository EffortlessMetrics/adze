//! Test case: Valid simple grammar should compile

#[adze::grammar("arithmetic")]
#[adze::language]
pub struct Arithmetic {
    pub expr: Expr,
}

#[derive(Debug)]
pub enum Expr {
    #[adze::leaf(text = r"[0-9]+")]
    Number(String),

    #[adze::prec_left(1)]
    Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),

    #[adze::prec_left(2)]
    Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
}

fn main() {
    // This should compile successfully
}
