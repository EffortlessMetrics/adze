#![allow(clippy::empty_line_after_outer_attr, clippy::unnecessary_cast)]

#[adze::grammar("test_vec_wrapper")]
pub mod grammar {
    #[adze::language]
    pub struct TestModule {
        #[adze::repeat(non_empty = false)]
        pub statements: Vec<TestStatement>,
    }

    pub struct TestStatement {
        #[adze::leaf(pattern = r"\d+", text = true)]
        pub value: String,
    }

    #[adze::extra]
    #[allow(dead_code)]
    struct Whitespace {
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}
