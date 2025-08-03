#[rust_sitter::grammar("test_vec_wrapper")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct TestModule {
        #[rust_sitter::repeat(non_empty = false)]
        pub statements: Vec<TestStatement>,
    }

    #[rust_sitter::language]
    pub struct TestStatement {
        #[rust_sitter::leaf(pattern = r"\d+")]
        pub value: u32,
    }
}