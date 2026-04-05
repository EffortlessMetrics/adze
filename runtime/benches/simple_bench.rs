// Simple benchmarks for the pure-Rust Tree-sitter parser
// Focuses on lexer performance which is working correctly

use adze::lexer::{self, GrammarLexer};
use adze_ir::{SymbolId, TokenPattern};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn configured_lexer(
    token_patterns: &[(SymbolId, TokenPattern, i32)],
    skip_symbols: &[SymbolId],
) -> GrammarLexer {
    let mut lexer = GrammarLexer::new(token_patterns);
    lexer.set_skip_symbols(skip_symbols.to_vec());
    lexer
}

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

fn token_count_without_eof(tokens: &[lexer::Token]) -> usize {
    tokens.iter().filter(|token| token.symbol.0 != 0).count()
}

fn assert_eof_terminated(tokens: &[lexer::Token], expected_non_eof: usize) {
    assert!(!tokens.is_empty());
    assert_eq!(
        tokens
            .last()
            .expect("lexer produced an empty stream")
            .symbol
            .0,
        0
    );
    assert_eq!(token_count_without_eof(tokens), expected_non_eof);
}

fn token_floor_from_whitespace(input: &str) -> usize {
    input
        .split_whitespace()
        .filter(|segment| !segment.is_empty())
        .count()
}

fn assert_min_non_eof_tokens(tokens: &[lexer::Token], min_non_eof: usize) {
    assert!(!tokens.is_empty());
    assert_eq!(
        tokens
            .last()
            .expect("lexer produced an empty stream")
            .symbol
            .0,
        0
    );
    assert!(
        token_count_without_eof(tokens) >= min_non_eof,
        "expected at least {min_non_eof} non-eof tokens, got {}",
        token_count_without_eof(tokens)
    );
}

fn benchmark_lexer_simple(c: &mut Criterion) {
    // Create token patterns for a simple grammar
    let token_patterns = vec![
        (SymbolId(1), TokenPattern::String("+".to_string()), 0),
        (SymbolId(2), TokenPattern::String("-".to_string()), 0),
        (SymbolId(3), TokenPattern::String("*".to_string()), 0),
        (SymbolId(4), TokenPattern::String("/".to_string()), 0),
        (SymbolId(5), TokenPattern::Regex(r"\d+".to_string()), 0),
        (
            SymbolId(6),
            TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
            0,
        ),
        (
            SymbolId(7),
            TokenPattern::Regex(r"[ \t\n\r]+".to_string()),
            10,
        ), // whitespace (skip)
    ];

    c.bench_function("lexer_arithmetic_expression", |b| {
        b.iter(|| {
            let mut lexer = configured_lexer(&token_patterns, &[SymbolId(7)]);
            let input = "123 + 456 * 789 - x / y";
            let tokens = lex_all(&mut lexer, black_box(input));
            assert_eof_terminated(&tokens, 9);
        });
    });

    c.bench_function("lexer_long_expression", |b| {
        let expr = "a + b * c - d / e + f * g - h / i + j * k - l / m + n * o - p / q";
        b.iter(|| {
            let mut lexer = configured_lexer(&token_patterns, &[SymbolId(7)]);
            let tokens = lex_all(&mut lexer, black_box(expr));
            assert_eof_terminated(&tokens, 33);
        });
    });

    c.bench_function("lexer_nested_expression", |b| {
        let expr = "((a + b) * (c - d)) / ((e + f) * (g - h))";
        b.iter(|| {
            let mut lexer = configured_lexer(&token_patterns, &[SymbolId(7)]);
            let tokens = lex_all(&mut lexer, black_box(expr));
            assert_eof_terminated(&tokens, 27);
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
        (
            SymbolId(14),
            TokenPattern::String("return".to_string()),
            100,
        ),
        (
            SymbolId(15),
            TokenPattern::String("function".to_string()),
            100,
        ),
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
        (
            SymbolId(50),
            TokenPattern::Regex(r"\d+(\.\d+)?".to_string()),
            10,
        ),
        (
            SymbolId(51),
            TokenPattern::Regex(r#""[^"]*""#.to_string()),
            10,
        ),
        (
            SymbolId(52),
            TokenPattern::Regex(r"'[^']*'".to_string()),
            10,
        ),
        // Identifier (lowest priority)
        (
            SymbolId(60),
            TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
            0,
        ),
        // Whitespace and comments (skip)
        (
            SymbolId(70),
            TokenPattern::Regex(r"[ \t\n\r]+".to_string()),
            1000,
        ),
        (
            SymbolId(71),
            TokenPattern::Regex(r"//[^\n]*".to_string()),
            1000,
        ),
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
            let mut lexer = configured_lexer(&token_patterns, &[SymbolId(70), SymbolId(71)]);
            let tokens = lex_all(&mut lexer, black_box(program));
            assert_min_non_eof_tokens(&tokens, token_floor_from_whitespace(program) * 2);
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
            let mut lexer = configured_lexer(&token_patterns, &[SymbolId(70), SymbolId(71)]);
            let tokens = lex_all(&mut lexer, black_box(program));
            assert_min_non_eof_tokens(&tokens, token_floor_from_whitespace(program) + 10);
        });
    });
}

fn benchmark_lexer_edge_cases(c: &mut Criterion) {
    let token_patterns = vec![
        (
            SymbolId(1),
            TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
            0,
        ),
        (SymbolId(2), TokenPattern::Regex(r"\d+".to_string()), 10),
        (
            SymbolId(3),
            TokenPattern::Regex(r"[ \t\n\r]+".to_string()),
            100,
        ),
    ];

    c.bench_function("lexer_long_identifier", |b| {
        let ident = "a".repeat(100);
        b.iter(|| {
            let mut lexer = configured_lexer(&token_patterns, &[SymbolId(3)]);
            let tokens = lex_all(&mut lexer, black_box(&ident));
            assert_eof_terminated(&tokens, 1);
        });
    });

    c.bench_function("lexer_many_tokens", |b| {
        let input = "a b c d e f g h i j k l m n o p q r s t u v w x y z ".repeat(10);
        b.iter(|| {
            let mut lexer = configured_lexer(&token_patterns, &[SymbolId(3)]);
            let tokens = lex_all(&mut lexer, black_box(&input));
            assert_min_non_eof_tokens(&tokens, token_floor_from_whitespace(&input));
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
