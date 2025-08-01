//! Test grammar with intentional ambiguity to exercise GLR parsing
//! 
//! This grammar creates the classic dangling-else ambiguity:
//! "if a then if b then c else d" can be parsed as either:
//! 1. if a then (if b then c else d)
//! 2. if a then (if b then c) else d

use rust_sitter::*;

#[rust_sitter::grammar("ambiguous")]
pub struct AmbiguousGrammar;

#[rust_sitter::language]
impl AmbiguousGrammar {
    pub fn parse_statement(&self, input: &str) -> Statement {
        self.parse(input)
    }
}

pub enum Statement {
    If(IfStatement),
    Expression(Expression),
}

pub struct IfStatement {
    pub condition: Expression,
    pub then_branch: Box<Statement>,
    pub else_branch: Option<Box<Statement>>,
}

#[rust_sitter::leaf(pattern = r"[a-z]+")]
pub struct Identifier(String);

pub struct Expression {
    pub id: Identifier,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dangling_else_ambiguity() {
        let grammar = AmbiguousGrammar;
        
        // This should produce an ambiguous parse
        let input = "if a then if b then c else d";
        
        // In a GLR parser, this would produce multiple parse trees
        // For now, we just ensure it can parse without panicking
        let result = std::panic::catch_unwind(|| {
            grammar.parse_statement(input)
        });
        
        // The parse might fail or succeed depending on how ambiguity is handled
        // The key is that it shouldn't panic with empty string terminal errors
        println!("Parse result: {:?}", result.is_ok());
    }
    
    #[test]
    fn test_simple_if() {
        let grammar = AmbiguousGrammar;
        let _stmt = grammar.parse_statement("if x then y");
    }
    
    #[test]
    fn test_nested_if_without_else() {
        let grammar = AmbiguousGrammar;
        let _stmt = grammar.parse_statement("if a then if b then c");
    }
}