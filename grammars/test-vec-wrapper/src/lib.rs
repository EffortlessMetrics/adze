#[rust_sitter::grammar("test_vec_wrapper")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct TestModule {
        #[rust_sitter::repeat(non_empty = true)]
        pub statements: Vec<TestStatement>,
    }

    #[rust_sitter::language]
    pub struct TestStatement {
        #[rust_sitter::leaf(pattern = r"\d+", transform = |s| s.parse::<u32>().unwrap())]
        pub value: rust_sitter::WithLeaf<u32>,
    }
}