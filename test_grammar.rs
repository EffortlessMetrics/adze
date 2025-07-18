
#[rust_sitter::grammar("test")]
mod grammar {
    #[rust_sitter::language]
    pub enum Expression {
        Number(
            #[rust_sitter::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
            i32
        ),
    }
}
