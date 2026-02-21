// Integration tests for adze
use adze::Extract;
use adze_tool::build_parsers;
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
        #[adze::grammar("arithmetic")]
        mod grammar {
            #[adze::language]
            pub struct Expr {
                pub left: Box<Expr>,
                #[adze::leaf(text = "+")]
                _plus: (),
                pub right: Box<Expr>,
            }
            
            #[adze::language]
            pub struct Expr {
                #[adze::leaf(pattern = r"\d+", transform = |s| s.parse().unwrap())]
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
    
    let grammar_code = r##"
        #[adze::grammar("strings")]
        mod grammar {
            #[adze::language]
            pub struct StringLit {
                #[adze::leaf(pattern = r#""([^"\\]|\\.)*""#)]
                pub value: String,
            }
        }
    "##;
    
    fs::write(&grammar_file, grammar_code).unwrap();
    build_parsers(&grammar_file);
}

#[test]
fn test_repetition_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let grammar_file = temp_dir.path().join("repetition_grammar.rs");
    
    let grammar_code = r#"
        #[adze::grammar("repetition")]
        mod grammar {
            #[adze::language]
            pub struct List {
                #[adze::repeat(non_empty = true)]
                pub items: Vec<Item>,
            }
            
            #[adze::language]
            pub struct Item {
                #[adze::leaf(pattern = r"\w+")]
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
        #[adze::grammar("optional")]
        mod grammar {
            #[adze::language]
            pub struct Function {
                #[adze::leaf(text = "fn")]
                _fn: (),
                #[adze::leaf(pattern = r"\w+")]
                pub name: String,
                pub params: Option<ParamList>,
                pub body: Block,
            }
            
            #[adze::language]
            pub struct ParamList {
                #[adze::leaf(text = "(")]
                _lparen: (),
                #[adze::leaf(text = ")")]
                _rparen: (),
            }
            
            #[adze::language]
            pub struct Block {
                #[adze::leaf(text = "{")]
                _lbrace: (),
                #[adze::leaf(text = "}")]
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
        #[adze::grammar("precedence")]
        mod grammar {
            #[adze::language]
            #[adze::precedence(1)]
            pub struct Add {
                pub left: Box<Expr>,
                #[adze::leaf(text = "+")]
                _op: (),
                pub right: Box<Expr>,
            }
            
            #[adze::language]
            #[adze::precedence(2)]
            pub struct Mul {
                pub left: Box<Expr>,
                #[adze::leaf(text = "*")]
                _op: (),
                pub right: Box<Expr>,
            }
            
            #[adze::language]
            pub enum Expr {
                Add(Add),
                Mul(Mul),
                Num(Num),
            }
            
            #[adze::language]
            pub struct Num {
                #[adze::leaf(pattern = r"\d+", transform = |s| s.parse().unwrap())]
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
        use adze::external_scanner::{ExternalScanner, ScanResult};
        
        #[adze::grammar("external")]
        mod grammar {
            #[adze::language]
            #[adze::external_scanner(StringScanner)]
            pub struct StringLit {
                #[adze::external]
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
                            symbol: adze_ir::SymbolId(0),
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
        #[adze::grammar("error_recovery")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                #[adze::repeat]
                pub statements: Vec<Statement>,
            }
            
            #[adze::language]
            pub struct Statement {
                pub expr: Expr,
                #[adze::leaf(text = ";")]
                _semi: (),
            }
            
            #[adze::language]
            pub enum Expr {
                Num(Num),
                Error(ErrorNode),
            }
            
            #[adze::language]
            pub struct Num {
                #[adze::leaf(pattern = r"\d+")]
                pub value: String,
            }
            
            #[adze::language]
            #[adze::error]
            pub struct ErrorNode {
                #[adze::leaf(pattern = r".+")]
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