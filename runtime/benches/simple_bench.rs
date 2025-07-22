// Simple benchmarks for the pure-Rust Tree-sitter parser
// Focuses on lexer performance which is working correctly

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_sitter::lexer::{self, GrammarLexer};
use rust_sitter_ir::{TokenPattern, SymbolId};

// Helper function to lex an entire input
fn lex_all(lexer: &mut GrammarLexer, input: &str) -> Vec<lexer::Token> {
    let mut tokens = Vec::new();
    let mut position = 0;
    let input_bytes = input.as_bytes();
    
    loop {
        if let Some(token) = lexer.next_token(input_bytes, position) {
            position = token.end;
            let is_eof = token.symbol.0 == 0;
            tokens.push(token);
            if is_eof {
                break;
            }
        } else {
            // Error - skip one byte
            position += 1;
            if position >= input_bytes.len() {
                break;
            }
        }
    }
    
    tokens
}

fn benchmark_lexer_simple(c: &mut Criterion) {
    // Create token patterns for a simple grammar
    let token_patterns = vec![
        (SymbolId(1), TokenPattern::String("+".to_string()), 0),
        (SymbolId(2), TokenPattern::String("-".to_string()), 0),
        (SymbolId(3), TokenPattern::String("*".to_string()), 0),
        (SymbolId(4), TokenPattern::String("/".to_string()), 0),
        (SymbolId(5), TokenPattern::Regex(r"\d+".to_string()), 0),
        (SymbolId(6), TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()), 0),
        (SymbolId(7), TokenPattern::Regex(r"[ \t\n\r]+".to_string()), 10), // whitespace (skip)
    ];
    
    c.bench_function("lexer_arithmetic_expression", |b| {
        b.iter(|| {
            let mut lexer = GrammarLexer::new(&token_patterns);
            let input = "123 + 456 * 789 - x / y";
            let tokens = lex_all(&mut lexer, black_box(input));
            assert_eq!(tokens.len(), 9); // Including EOF
        });
    });
    
    c.bench_function("lexer_long_expression", |b| {
        let expr = "a + b * c - d / e + f * g - h / i + j * k - l / m + n * o - p / q";
        b.iter(|| {
            let mut lexer = GrammarLexer::new(&token_patterns);
            let tokens = lex_all(&mut lexer, black_box(expr));
            assert!(tokens.len() > 20);
        });
    });
    
    c.bench_function("lexer_nested_expression", |b| {
        let expr = "((a + b) * (c - d)) / ((e + f) * (g - h))";
        b.iter(|| {
            let mut lexer = GrammarLexer::new(&token_patterns);
            let tokens = lex_all(&mut lexer, black_box(expr));
            assert!(tokens.len() > 15);
        });
    });
}

fn benchmark_lexer_programming_language(c: &mut Criterion) {
    // More complex token patterns resembling a programming language
    let token_patterns = vec![
        // Keywords
        (SymbolId(10), TokenPattern::String("if".to_string()), 100),
        (SymbolId(11), TokenPattern::String("else".to_string()), 100),
        (SymbolId(12), TokenPattern::String("while".to_string()), 100),
        (SymbolId(13), TokenPattern::String("for".to_string()), 100),
        (SymbolId(14), TokenPattern::String("return".to_string()), 100),
        (SymbolId(15), TokenPattern::String("function".to_string()), 100),
        (SymbolId(16), TokenPattern::String("var".to_string()), 100),
        (SymbolId(17), TokenPattern::String("const".to_string()), 100),
        
        // Operators
        (SymbolId(20), TokenPattern::String("==".to_string()), 50),
        (SymbolId(21), TokenPattern::String("!=".to_string()), 50),
        (SymbolId(22), TokenPattern::String("<=".to_string()), 50),
        (SymbolId(23), TokenPattern::String(">=".to_string()), 50),
        (SymbolId(24), TokenPattern::String("&&".to_string()), 50),
        (SymbolId(25), TokenPattern::String("||".to_string()), 50),
        (SymbolId(26), TokenPattern::String("=".to_string()), 40),
        (SymbolId(27), TokenPattern::String("+".to_string()), 40),
        (SymbolId(28), TokenPattern::String("-".to_string()), 40),
        (SymbolId(29), TokenPattern::String("*".to_string()), 40),
        (SymbolId(30), TokenPattern::String("/".to_string()), 40),
        (SymbolId(31), TokenPattern::String("<".to_string()), 40),
        (SymbolId(32), TokenPattern::String(">".to_string()), 40),
        
        // Delimiters
        (SymbolId(40), TokenPattern::String("(".to_string()), 20),
        (SymbolId(41), TokenPattern::String(")".to_string()), 20),
        (SymbolId(42), TokenPattern::String("{".to_string()), 20),
        (SymbolId(43), TokenPattern::String("}".to_string()), 20),
        (SymbolId(44), TokenPattern::String("[".to_string()), 20),
        (SymbolId(45), TokenPattern::String("]".to_string()), 20),
        (SymbolId(46), TokenPattern::String(";".to_string()), 20),
        (SymbolId(47), TokenPattern::String(",".to_string()), 20),
        
        // Literals
        (SymbolId(50), TokenPattern::Regex(r"\d+(\.\d+)?".to_string()), 10),
        (SymbolId(51), TokenPattern::Regex(r#""[^"]*""#.to_string()), 10),
        (SymbolId(52), TokenPattern::Regex(r"'[^']*'".to_string()), 10),
        
        // Identifier (lowest priority)
        (SymbolId(60), TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()), 0),
        
        // Whitespace and comments (skip)
        (SymbolId(70), TokenPattern::Regex(r"[ \t\n\r]+".to_string()), 1000),
        (SymbolId(71), TokenPattern::Regex(r"//[^\n]*".to_string()), 1000),
    ];
    
    c.bench_function("lexer_simple_program", |b| {
        let program = r#"
            function factorial(n) {
                if (n <= 1) {
                    return 1;
                } else {
                    return n * factorial(n - 1);
                }
            }
            
            const result = factorial(5);
        "#;
        
        b.iter(|| {
            let mut lexer = GrammarLexer::new(&token_patterns);
            let tokens = lex_all(&mut lexer, black_box(program));
            assert!(tokens.len() > 30);
        });
    });
    
    c.bench_function("lexer_complex_program", |b| {
        let program = r#"
            function quickSort(arr, left, right) {
                if (left < right) {
                    const pivotIndex = partition(arr, left, right);
                    quickSort(arr, left, pivotIndex - 1);
                    quickSort(arr, pivotIndex + 1, right);
                }
            }
            
            function partition(arr, left, right) {
                const pivot = arr[right];
                var i = left - 1;
                
                for (var j = left; j < right; j = j + 1) {
                    if (arr[j] <= pivot) {
                        i = i + 1;
                        const temp = arr[i];
                        arr[i] = arr[j];
                        arr[j] = temp;
                    }
                }
                
                const temp = arr[i + 1];
                arr[i + 1] = arr[right];
                arr[right] = temp;
                
                return i + 1;
            }
            
            const numbers = [64, 34, 25, 12, 22, 11, 90];
            quickSort(numbers, 0, 6);
        "#;
        
        b.iter(|| {
            let mut lexer = GrammarLexer::new(&token_patterns);
            let tokens = lex_all(&mut lexer, black_box(program));
            assert!(tokens.len() > 100);
        });
    });
}

fn benchmark_lexer_edge_cases(c: &mut Criterion) {
    let token_patterns = vec![
        (SymbolId(1), TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()), 0),
        (SymbolId(2), TokenPattern::Regex(r"\d+".to_string()), 10),
        (SymbolId(3), TokenPattern::Regex(r"[ \t\n\r]+".to_string()), 100),
    ];
    
    c.bench_function("lexer_long_identifier", |b| {
        let ident = "a".repeat(100);
        b.iter(|| {
            let mut lexer = GrammarLexer::new(&token_patterns);
            let tokens = lex_all(&mut lexer, black_box(&ident));
            assert_eq!(tokens.len(), 2); // identifier + EOF
        });
    });
    
    c.bench_function("lexer_many_tokens", |b| {
        let input = "a b c d e f g h i j k l m n o p q r s t u v w x y z ".repeat(10);
        b.iter(|| {
            let mut lexer = GrammarLexer::new(&token_patterns);
            let tokens = lex_all(&mut lexer, black_box(&input));
            assert!(tokens.len() > 250);
        });
    });
}

criterion_group!(
    benches,
    benchmark_lexer_simple,
    benchmark_lexer_programming_language,
    benchmark_lexer_edge_cases
);
criterion_main!(benches);