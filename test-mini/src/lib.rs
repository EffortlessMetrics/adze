#[rust_sitter::grammar("mini")]
pub mod grammar {
    #[rust_sitter::language]
    pub struct S {
        pub e: E,
    }
    
    #[rust_sitter::language]
    pub enum E {
        Num(#[rust_sitter::leaf(pattern = r"\d+")] String),
    }
}

#[cfg(test)]
mod tests {
    use crate::grammar;
    
    #[test]
    fn test_number() {
        let result = grammar::parse("42");
        assert!(result.is_ok());
    }
}