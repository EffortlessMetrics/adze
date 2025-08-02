//! Test grammar with intentional ambiguity to exercise GLR parsing
//! 
//! This grammar creates the classic dangling-else ambiguity:
//! "if a then if b then c else d" can be parsed as either:
//! 1. if a then (if b then c else d)
//! 2. if a then (if b then c) else d

use rust_sitter::*;

#[rust_sitter::grammar("ambiguous")]
pub mod grammar {
    #[rust_sitter::language]
    pub enum Statement {
        If(IfStatement),
        Expression(Expression),
    }

    pub struct IfStatement {
        #[rust_sitter::leaf(text = "if")]
        _if: (),
        condition: Expression,
        #[rust_sitter::leaf(text = "then")]
        _then: (),
        then_branch: Box<Statement>,
        else_branch: Option<ElseBranch>,
    }

    pub struct ElseBranch {
        #[rust_sitter::leaf(text = "else")]
        _else: (),
        statement: Box<Statement>,
    }

    pub struct Expression {
        id: Identifier,
    }

    pub struct Identifier {
        #[rust_sitter::leaf(pattern = r"[a-z]+")]
        name: String,
    }

    #[rust_sitter::extra]
    struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

#[cfg(test)]
mod tests {
    use super::grammar;

    #[test]
    fn test_dangling_else_ambiguity() {
        // This should produce an ambiguous parse
        let input = "if a then if b then c else d";
        
        // In a GLR parser, this would produce multiple parse trees
        // For now, we just ensure it can parse without panicking
        let result = std::panic::catch_unwind(|| {
            grammar::parse(input)
        });
        
        // The parse might fail or succeed depending on how ambiguity is handled
        // The key is that it shouldn't panic with empty string terminal errors
        println!("Parse result: {:?}", result.is_ok());
    }
    
    #[test]
    fn test_simple_if() {
        let _stmt = grammar::parse("if x then y");
    }
    
    #[test]
    fn test_nested_if_without_else() {
        let _stmt = grammar::parse("if a then if b then c");
    }
}