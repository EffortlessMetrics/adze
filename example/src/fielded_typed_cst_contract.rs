#[adze::grammar("fielded_typed_cst_contract")]
pub mod grammar {
    #[adze::language]
    #[derive(Debug, PartialEq, Eq)]
    pub struct Pair {
        #[adze::field("left")]
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        pub left: i32,

        #[adze::field("right")]
        #[adze::leaf(text = "+")]
        pub right: (),
    }
}
