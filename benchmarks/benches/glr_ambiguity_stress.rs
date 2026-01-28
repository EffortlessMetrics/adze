use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_sitter::glr_lexer::GLRLexer;
use rust_sitter::glr_parser::GLRParser;
use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets};
use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

/// Create a grammar with extreme ambiguity - every operator can be parsed multiple ways
fn create_extremely_ambiguous_grammar() -> Grammar {
    let mut grammar = Grammar::new("extreme_ambiguous".to_string());

    // Tokens
    let num_id = SymbolId(1);
    let op_id = SymbolId(2); // Single operator that can mean different things

    // Non-terminals
    let start_id = SymbolId(10); // Start symbol
    let expr_id = SymbolId(11);
    let expr2_id = SymbolId(12);
    let expr3_id = SymbolId(13);

    // Define tokens
    grammar.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex("[0-9]+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        op_id,
        Token {
            name: "op".to_string(),
            pattern: TokenPattern::String("@".to_string()),
            fragile: false,
        },
    );

    let mut rule_id = 0;

    // Multiple ways to parse expr @ expr
    // expr -> expr @ expr (version 1)
    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(op_id),
            Symbol::NonTerminal(expr_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(rule_id),
    });
    rule_id += 1;

    // expr -> expr2
    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::NonTerminal(expr2_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(rule_id),
    });
    rule_id += 1;

    // expr2 -> expr2 @ expr3 (version 2)
    grammar.rules.entry(expr2_id).or_default().push(Rule {
        lhs: expr2_id,
        rhs: vec![
            Symbol::NonTerminal(expr2_id),
            Symbol::Terminal(op_id),
            Symbol::NonTerminal(expr3_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(rule_id),
    });
    rule_id += 1;

    // expr2 -> expr3
    grammar.rules.entry(expr2_id).or_default().push(Rule {
        lhs: expr2_id,
        rhs: vec![Symbol::NonTerminal(expr3_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(rule_id),
    });
    rule_id += 1;

    // expr3 -> expr @ expr3 (version 3, right associative)
    grammar.rules.entry(expr3_id).or_default().push(Rule {
        lhs: expr3_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(op_id),
            Symbol::NonTerminal(expr3_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(rule_id),
    });
    rule_id += 1;

    // expr3 -> num
    grammar.rules.entry(expr3_id).or_default().push(Rule {
        lhs: expr3_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(rule_id),
    });

    // All three expression types can also reduce to num directly
    rule_id += 1;
    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(rule_id),
    });

    rule_id += 1;
    grammar.rules.entry(expr2_id).or_default().push(Rule {
        lhs: expr2_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(rule_id),
    });

    // Add a start rule: start -> expr
    rule_id += 1;
    grammar.rules.entry(start_id).or_default().push(Rule {
        lhs: start_id,
        rhs: vec![Symbol::NonTerminal(expr_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(rule_id),
    });

    // Set up rule names with proper start symbol
    grammar
        .rule_names
        .insert(start_id, "source_file".to_string());
    grammar.rule_names.insert(expr_id, "expr".to_string());
    grammar.rule_names.insert(expr2_id, "expr2".to_string());
    grammar.rule_names.insert(expr3_id, "expr3".to_string());

    grammar
}

fn benchmark_allocation(c: &mut Criterion) {
    let grammar = create_extremely_ambiguous_grammar();
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    let input = "1 @ 2 @ 3 @ 4 @ 5 @ 6"; // Long enough to cause many forks
    let mut lexer = GLRLexer::new(&grammar, input.to_string()).unwrap();
    let tokens: Vec<_> = lexer.tokenize_all();

    c.bench_function("glr_ambiguous_parse", |b| {
        b.iter(|| {
            let mut parser = GLRParser::new(parse_table.clone(), grammar.clone());
            for token in &tokens {
                parser.process_token(token.symbol_id, &token.text, token.byte_offset);
            }
            // We don't finish() because we want to measure process_token performance mostly
            black_box(parser.stack_count());
        });
    });
}

criterion_group!(benches, benchmark_allocation);
criterion_main!(benches);
