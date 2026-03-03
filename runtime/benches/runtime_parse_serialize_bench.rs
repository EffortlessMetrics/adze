// Benchmarks for runtime parsing, serialization, and visitor traversal.
//
// Covers:
//   1. Parse small input (< 100 chars)
//   2. Parse medium input (~1 KB)
//   3. Parse large input (~10 KB)
//   4. Tree serialization to JSON (via SerializedNode)
//   5. Tree serialization to S-expression (via GLRNode)
//   6. Visitor traversal speed (depth-first over GLRTree)
//   7. Tree construction from parsed output (GLRTree from Subtree)

use std::sync::Arc;

use adze::glr_lexer::GLRLexer;
use adze::glr_parser::GLRParser;
use adze::glr_tree_bridge::{GLRTree, subtree_to_tree};
use adze::subtree::Subtree;
use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

// ---------------------------------------------------------------------------
// Grammar helpers
// ---------------------------------------------------------------------------

/// Ambiguous arithmetic grammar: E -> E+E | E*E | (E) | number
fn create_arithmetic_grammar() -> Grammar {
    let mut grammar = Grammar::new("expression".to_string());

    let expr_id = SymbolId(0);
    let number_id = SymbolId(1);
    let plus_id = SymbolId(2);
    let mult_id = SymbolId(3);
    let lparen_id = SymbolId(4);
    let rparen_id = SymbolId(5);

    grammar.tokens.insert(
        number_id,
        Token {
            name: "number".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        plus_id,
        Token {
            name: "plus".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        mult_id,
        Token {
            name: "multiply".into(),
            pattern: TokenPattern::String("*".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        lparen_id,
        Token {
            name: "lparen".into(),
            pattern: TokenPattern::String("(".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        rparen_id,
        Token {
            name: "rparen".into(),
            pattern: TokenPattern::String(")".into()),
            fragile: false,
        },
    );

    let rules = vec![
        Rule {
            lhs: expr_id,
            rhs: vec![
                Symbol::NonTerminal(expr_id),
                Symbol::Terminal(plus_id),
                Symbol::NonTerminal(expr_id),
            ],
            production_id: ProductionId(0),
            precedence: None,
            associativity: None,
            fields: vec![],
        },
        Rule {
            lhs: expr_id,
            rhs: vec![
                Symbol::NonTerminal(expr_id),
                Symbol::Terminal(mult_id),
                Symbol::NonTerminal(expr_id),
            ],
            production_id: ProductionId(1),
            precedence: None,
            associativity: None,
            fields: vec![],
        },
        Rule {
            lhs: expr_id,
            rhs: vec![
                Symbol::Terminal(lparen_id),
                Symbol::NonTerminal(expr_id),
                Symbol::Terminal(rparen_id),
            ],
            production_id: ProductionId(2),
            precedence: None,
            associativity: None,
            fields: vec![],
        },
        Rule {
            lhs: expr_id,
            rhs: vec![Symbol::Terminal(number_id)],
            production_id: ProductionId(3),
            precedence: None,
            associativity: None,
            fields: vec![],
        },
    ];

    grammar.rules.insert(expr_id, rules);
    grammar.rule_names.insert(expr_id, "expression".into());
    grammar
}

// ---------------------------------------------------------------------------
// Input generators
// ---------------------------------------------------------------------------

fn small_input() -> String {
    "1 + 2 * 3 + 4".to_string()
}

fn medium_input() -> String {
    let parts: Vec<String> = (1..=250).map(|i| i.to_string()).collect();
    parts.join(" + ")
}

fn large_input() -> String {
    let parts: Vec<String> = (1..=2500).map(|i| i.to_string()).collect();
    parts.join(" + ")
}

// ---------------------------------------------------------------------------
// Parse helper — lex then GLR parse, returning the Subtree root
// ---------------------------------------------------------------------------

fn glr_parse(
    grammar: &Grammar,
    parse_table: &adze_glr_core::ParseTable,
    input: &str,
) -> Result<Arc<Subtree>, String> {
    let mut lexer = GLRLexer::new(grammar, input.to_string()).map_err(|e| format!("{e:?}"))?;
    let mut parser = GLRParser::new(parse_table.clone(), grammar.clone());

    let mut tokens = Vec::new();
    while let Some(token) = lexer.next_token() {
        tokens.push(token);
    }
    for token in &tokens {
        parser.process_token(token.symbol_id, &token.text, token.byte_offset);
    }
    let total_bytes = tokens
        .last()
        .map(|t| t.byte_offset + t.text.len())
        .unwrap_or(0);
    parser.process_eof(total_bytes);
    parser.finish()
}

// ---------------------------------------------------------------------------
// Recursive JSON-like serialization of a Subtree (mirrors TreeSerializer)
// ---------------------------------------------------------------------------

fn serialize_subtree_json(subtree: &Subtree, source: &[u8]) -> serde_json::Value {
    let start = subtree.node.byte_range.start;
    let end = subtree.node.byte_range.end;
    let text = if subtree.children.is_empty() {
        std::str::from_utf8(&source[start..end.min(source.len())])
            .ok()
            .map(|s| serde_json::Value::String(s.to_string()))
    } else {
        None
    };
    let children: Vec<serde_json::Value> = subtree
        .children
        .iter()
        .map(|e| serialize_subtree_json(&e.subtree, source))
        .collect();
    serde_json::json!({
        "symbol": subtree.node.symbol_id.0,
        "start_byte": start,
        "end_byte": end,
        "is_error": subtree.node.is_error,
        "text": text,
        "children": children,
    })
}

// ---------------------------------------------------------------------------
// Recursive S-expression serialization of a Subtree
// ---------------------------------------------------------------------------

fn serialize_subtree_sexpr(subtree: &Subtree, source: &[u8]) -> String {
    let start = subtree.node.byte_range.start;
    let end = subtree.node.byte_range.end;
    if subtree.children.is_empty() {
        let text = std::str::from_utf8(&source[start..end.min(source.len())]).unwrap_or("");
        format!("\"{}\"", text.replace('"', "\\\""))
    } else {
        let children: Vec<String> = subtree
            .children
            .iter()
            .map(|e| serialize_subtree_sexpr(&e.subtree, source))
            .collect();
        format!(
            "(symbol_{} {})",
            subtree.node.symbol_id.0,
            children.join(" ")
        )
    }
}

// ---------------------------------------------------------------------------
// Depth-first visitor count over a Subtree
// ---------------------------------------------------------------------------

fn count_nodes(subtree: &Subtree) -> (usize, usize) {
    let mut nodes = 1usize;
    let mut leaves = if subtree.children.is_empty() {
        1usize
    } else {
        0
    };
    for edge in &subtree.children {
        let (n, l) = count_nodes(&edge.subtree);
        nodes += n;
        leaves += l;
    }
    (nodes, leaves)
}

// ---------------------------------------------------------------------------
// 1-3. Parse benchmarks (small / medium / large)
// ---------------------------------------------------------------------------

fn bench_parse_small(c: &mut Criterion) {
    let grammar = create_arithmetic_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    let input = small_input();

    c.bench_function("rt_parse_small_input", |b| {
        b.iter(|| {
            let _ = glr_parse(&grammar, &table, black_box(&input));
        });
    });
}

fn bench_parse_medium(c: &mut Criterion) {
    let grammar = create_arithmetic_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    let input = medium_input();

    c.bench_function("rt_parse_medium_input", |b| {
        b.iter(|| {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                glr_parse(&grammar, &table, black_box(&input))
            }));
        });
    });
}

fn bench_parse_large(c: &mut Criterion) {
    let grammar = create_arithmetic_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    let input = large_input();

    c.bench_function("rt_parse_large_input", |b| {
        b.iter(|| {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                glr_parse(&grammar, &table, black_box(&input))
            }));
        });
    });
}

// ---------------------------------------------------------------------------
// 4. Serialization to JSON
// ---------------------------------------------------------------------------

fn bench_serialize_json(c: &mut Criterion) {
    let grammar = create_arithmetic_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    let input = small_input();
    let source = input.as_bytes();

    let subtree = glr_parse(&grammar, &table, &input).expect("parse should succeed");

    c.bench_function("rt_serialize_json", |b| {
        b.iter(|| {
            let json = serialize_subtree_json(black_box(&subtree), source);
            black_box(json);
        });
    });
}

// ---------------------------------------------------------------------------
// 5. Serialization to S-expression
// ---------------------------------------------------------------------------

fn bench_serialize_sexpr(c: &mut Criterion) {
    let grammar = create_arithmetic_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    let input = small_input();
    let source = input.as_bytes();

    let subtree = glr_parse(&grammar, &table, &input).expect("parse should succeed");

    c.bench_function("rt_serialize_sexpr", |b| {
        b.iter(|| {
            let sexpr = serialize_subtree_sexpr(black_box(&subtree), source);
            black_box(sexpr);
        });
    });
}

// ---------------------------------------------------------------------------
// 6. Visitor traversal speed
// ---------------------------------------------------------------------------

fn bench_visitor_traversal(c: &mut Criterion) {
    let grammar = create_arithmetic_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    let input = small_input();

    let subtree = glr_parse(&grammar, &table, &input).expect("parse should succeed");

    c.bench_function("rt_visitor_traversal", |b| {
        b.iter(|| {
            let (nodes, leaves) = count_nodes(black_box(&subtree));
            black_box((nodes, leaves));
        });
    });
}

// ---------------------------------------------------------------------------
// 7. Tree construction from parsed output (GLRTree from Subtree)
// ---------------------------------------------------------------------------

fn bench_tree_construction(c: &mut Criterion) {
    let grammar = create_arithmetic_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    let input = small_input();

    let subtree = glr_parse(&grammar, &table, &input).expect("parse should succeed");

    c.bench_function("rt_tree_construction", |b| {
        b.iter(|| {
            let tree: GLRTree = subtree_to_tree(
                Arc::clone(black_box(&subtree)),
                input.as_bytes().to_vec(),
                grammar.clone(),
            );
            black_box(&tree);
        });
    });
}

// ---------------------------------------------------------------------------
// Criterion groups
// ---------------------------------------------------------------------------

criterion_group!(
    parsing,
    bench_parse_small,
    bench_parse_medium,
    bench_parse_large,
);

criterion_group!(serialization, bench_serialize_json, bench_serialize_sexpr,);

criterion_group!(traversal, bench_visitor_traversal, bench_tree_construction,);

criterion_main!(parsing, serialization, traversal);
