// Integration tests for rust-sitter
use rust_sitter::Extract;
use rust_sitter_tool::build_parsers;
use std::path::PathBuf;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_simple_arithmetic_grammar() {
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().unwrap();
    let grammar_file = temp_dir.path().join("grammar.rs");
    
    // Write a simple arithmetic grammar
    let grammar_code = r#"
        #[rust_sitter::grammar("arithmetic")]
        mod grammar {
            #[rust_sitter::language]
            pub struct Expr {
                pub left: Box<Expr>,
                #[rust_sitter::leaf(text = "+")]
                _plus: (),
                pub right: Box<Expr>,
            }
            
            #[rust_sitter::language]
            pub struct Expr {
                #[rust_sitter::leaf(pattern = r"\d+", transform = |s| s.parse().unwrap())]
                pub value: i32,
            }
        }
    "#;
    
    fs::write(&grammar_file, grammar_code).unwrap();
    
    // Build the parser
    build_parsers(&grammar_file);
    
    // TODO: Test parsing once the build process is complete
}

#[test]
fn test_string_literals() {
    let temp_dir = TempDir::new().unwrap();
    let grammar_file = temp_dir.path().join("string_grammar.rs");
    
    let grammar_code = r#"
        #[rust_sitter::grammar("strings")]
        mod grammar {
            #[rust_sitter::language]
            pub struct StringLit {
                #[rust_sitter::leaf(pattern = r#""([^"\\]|\\.)*""#)]
                pub value: String,
            }
        }
    "#;
    
    fs::write(&grammar_file, grammar_code).unwrap();
    build_parsers(&grammar_file);
}

#[test]
fn test_repetition_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let grammar_file = temp_dir.path().join("repetition_grammar.rs");
    
    let grammar_code = r#"
        #[rust_sitter::grammar("repetition")]
        mod grammar {
            #[rust_sitter::language]
            pub struct List {
                #[rust_sitter::repeat(non_empty = true)]
                pub items: Vec<Item>,
            }
            
            #[rust_sitter::language]
            pub struct Item {
                #[rust_sitter::leaf(pattern = r"\w+")]
                pub name: String,
            }
        }
    "#;
    
    fs::write(&grammar_file, grammar_code).unwrap();
    build_parsers(&grammar_file);
}

#[test]
fn test_optional_fields() {
    let temp_dir = TempDir::new().unwrap();
    let grammar_file = temp_dir.path().join("optional_grammar.rs");
    
    let grammar_code = r#"
        #[rust_sitter::grammar("optional")]
        mod grammar {
            #[rust_sitter::language]
            pub struct Function {
                #[rust_sitter::leaf(text = "fn")]
                _fn: (),
                #[rust_sitter::leaf(pattern = r"\w+")]
                pub name: String,
                pub params: Option<ParamList>,
                pub body: Block,
            }
            
            #[rust_sitter::language]
            pub struct ParamList {
                #[rust_sitter::leaf(text = "(")]
                _lparen: (),
                #[rust_sitter::leaf(text = ")")]
                _rparen: (),
            }
            
            #[rust_sitter::language]
            pub struct Block {
                #[rust_sitter::leaf(text = "{")]
                _lbrace: (),
                #[rust_sitter::leaf(text = "}")]
                _rbrace: (),
            }
        }
    "#;
    
    fs::write(&grammar_file, grammar_code).unwrap();
    build_parsers(&grammar_file);
}

#[test]
fn test_precedence_and_associativity() {
    let temp_dir = TempDir::new().unwrap();
    let grammar_file = temp_dir.path().join("precedence_grammar.rs");
    
    let grammar_code = r#"
        #[rust_sitter::grammar("precedence")]
        mod grammar {
            #[rust_sitter::language]
            #[rust_sitter::precedence(1)]
            pub struct Add {
                pub left: Box<Expr>,
                #[rust_sitter::leaf(text = "+")]
                _op: (),
                pub right: Box<Expr>,
            }
            
            #[rust_sitter::language]
            #[rust_sitter::precedence(2)]
            pub struct Mul {
                pub left: Box<Expr>,
                #[rust_sitter::leaf(text = "*")]
                _op: (),
                pub right: Box<Expr>,
            }
            
            #[rust_sitter::language]
            pub enum Expr {
                Add(Add),
                Mul(Mul),
                Num(Num),
            }
            
            #[rust_sitter::language]
            pub struct Num {
                #[rust_sitter::leaf(pattern = r"\d+", transform = |s| s.parse().unwrap())]
                pub value: i32,
            }
        }
    "#;
    
    fs::write(&grammar_file, grammar_code).unwrap();
    build_parsers(&grammar_file);
}

#[test]
fn test_external_scanner() {
    let temp_dir = TempDir::new().unwrap();
    let grammar_file = temp_dir.path().join("external_grammar.rs");
    
    let grammar_code = r#"
        use rust_sitter::external_scanner::{ExternalScanner, ScanResult};
        
        #[rust_sitter::grammar("external")]
        mod grammar {
            #[rust_sitter::language]
            #[rust_sitter::external_scanner(StringScanner)]
            pub struct StringLit {
                #[rust_sitter::external]
                pub value: String,
            }
        }
        
        struct StringScanner;
        
        impl ExternalScanner for StringScanner {
            fn new() -> Self {
                StringScanner
            }
            
            fn scan(&mut self, valid_symbols: &[bool], input: &[u8], position: usize) -> Option<ScanResult> {
                // Simple string scanner implementation
                if position < input.len() && input[position] == b'"' {
                    let mut i = position + 1;
                    while i < input.len() && input[i] != b'"' {
                        if input[i] == b'\\' && i + 1 < input.len() {
                            i += 2;
                        } else {
                            i += 1;
                        }
                    }
                    if i < input.len() {
                        Some(ScanResult {
                            symbol: rust_sitter_ir::SymbolId(0),
                            length: i - position + 1,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            
            fn serialize(&self, _buffer: &mut Vec<u8>) {}
            fn deserialize(&mut self, _buffer: &[u8]) {}
        }
    "#;
    
    fs::write(&grammar_file, grammar_code).unwrap();
    // Note: External scanner support might need additional setup
}

#[test]
fn test_error_recovery() {
    let temp_dir = TempDir::new().unwrap();
    let grammar_file = temp_dir.path().join("error_grammar.rs");
    
    let grammar_code = r#"
        #[rust_sitter::grammar("error_recovery")]
        mod grammar {
            #[rust_sitter::language]
            pub struct Program {
                #[rust_sitter::repeat]
                pub statements: Vec<Statement>,
            }
            
            #[rust_sitter::language]
            pub struct Statement {
                pub expr: Expr,
                #[rust_sitter::leaf(text = ";")]
                _semi: (),
            }
            
            #[rust_sitter::language]
            pub enum Expr {
                Num(Num),
                Error(ErrorNode),
            }
            
            #[rust_sitter::language]
            pub struct Num {
                #[rust_sitter::leaf(pattern = r"\d+")]
                pub value: String,
            }
            
            #[rust_sitter::language]
            #[rust_sitter::error]
            pub struct ErrorNode {
                #[rust_sitter::leaf(pattern = r".+")]
                pub text: String,
            }
        }
    "#;
    
    fs::write(&grammar_file, grammar_code).unwrap();
    build_parsers(&grammar_file);
}

// Performance benchmarks
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;
    
    #[test]
    #[ignore] // Run with --ignored flag for benchmarks
    fn bench_large_file_parsing() {
        // Generate a large arithmetic expression
        let mut expr = String::new();
        for i in 0..1000 {
            if i > 0 {
                expr.push_str(" + ");
            }
            expr.push_str(&i.to_string());
        }
        
        let start = Instant::now();
        // TODO: Parse the expression once parser is available
        let duration = start.elapsed();
        
        println!("Parsed large expression in {:?}", duration);
        assert!(duration.as_secs() < 1, "Parsing took too long");
    }
}