#![allow(clippy::empty_line_after_outer_attr, clippy::unnecessary_cast)]

#[rust_sitter::grammar("test_vec_wrapper")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct TestModule {
        #[rust_sitter::repeat(non_empty = false)]
        pub statements: Vec<TestStatement>,
    }

    pub struct TestStatement {
        #[rust_sitter::leaf(pattern = r"\d+")]
        pub value: String,
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}
