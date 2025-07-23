// Query compiler - compiles query strings into Query objects
use super::ast::Query;
use super::parser::QueryParser;
use rust_sitter_ir::Grammar;

/// Compile a query string into a Query object
pub fn compile_query(source: &str, grammar: &Grammar) -> Result<Query, super::QueryError> {
    let parser = QueryParser::new(source, grammar);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_sitter_ir::{Grammar, Token, TokenPattern, SymbolId};
    
    fn create_test_grammar() -> Grammar {
        let mut grammar = Grammar::new("test".to_string());
        
        // Add some test tokens
        grammar.tokens.insert(SymbolId(1), Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex("[a-zA-Z]+".to_string()),
            fragile: false,
        });
        
        grammar.tokens.insert(SymbolId(2), Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex("[0-9]+".to_string()),
            fragile: false,
        });
        
        // Add rule names
        grammar.rule_names.insert(SymbolId(10), "expression".to_string());
        grammar.rule_names.insert(SymbolId(11), "statement".to_string());
        
        grammar
    }
    
    #[test]
    fn test_simple_query() {
        let grammar = create_test_grammar();
        let query_str = r#"(expression)"#;
        
        let query = compile_query(query_str, &grammar).unwrap();
        assert_eq!(query.patterns.len(), 1);
        assert_eq!(query.capture_count(), 0);
    }
    
    #[test]
    fn test_query_with_capture() {
        let grammar = create_test_grammar();
        let query_str = r#"(expression @expr)"#;
        
        let query = compile_query(query_str, &grammar).unwrap();
        assert_eq!(query.patterns.len(), 1);
        assert_eq!(query.capture_count(), 1);
        assert_eq!(query.capture_index("expr"), Some(0));
    }
    
    #[test]
    fn test_query_with_field() {
        let grammar = create_test_grammar();
        let query_str = r#"(statement value: (expression))"#;
        
        let query = compile_query(query_str, &grammar).unwrap();
        assert_eq!(query.patterns.len(), 1);
    }
    
    #[test]
    fn test_query_with_predicate() {
        let grammar = create_test_grammar();
        let query_str = r#"
            (expression @expr)
            (#eq? @expr "test")
        "#;
        
        let query = compile_query(query_str, &grammar).unwrap();
        assert_eq!(query.patterns.len(), 1);
        assert_eq!(query.patterns[0].predicates.len(), 1);
    }
    
    #[test]
    fn test_query_with_quantifiers() {
        let grammar = create_test_grammar();
        let query_str = r#"(expression (identifier)+ (number)?)"#;
        
        let query = compile_query(query_str, &grammar).unwrap();
        assert_eq!(query.patterns.len(), 1);
    }
}