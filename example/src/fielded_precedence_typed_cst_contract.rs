#[adze::grammar("fielded_precedence_typed_cst_contract")]
pub mod grammar {
    #[adze::language]
    #[derive(Debug, PartialEq, Eq)]
    pub enum Expr {
        Number(#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),

        #[adze::prec_left(1)]
        Add {
            #[adze::field("left")]
            left: Box<Expr>,
            #[adze::field("operator")]
            #[adze::leaf(text = "+")]
            operator: (),
            #[adze::field("right")]
            right: Box<Expr>,
        },

        #[adze::prec_left(2)]
        Mul {
            #[adze::field("left")]
            left: Box<Expr>,
            #[adze::field("operator")]
            #[adze::leaf(text = "*")]
            operator: (),
            #[adze::field("right")]
            right: Box<Expr>,
        },
    }

    #[adze::extra]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}
