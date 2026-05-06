#[adze::grammar("mylang")]
pub mod grammar {
    #[adze::language]
    pub struct Program {
        #[adze::leaf(pattern = r"\d+", text = true)]
        pub number: String,
    }
}

#[cfg(test)]
mod tests {
    use super::grammar;

    #[test]
    fn test_can_load_grammar() {
        let language = grammar::language();
        assert!(language.symbol_count > 0);
    }
}
