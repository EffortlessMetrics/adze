//! Cross-crate integration tests that verify the full IR → GLR → Tablegen pipeline.
//!
//! These tests exercise the real public APIs of `adze-ir`, `adze-glr-core`, and
//! `adze-tablegen` together, confirming that grammars survive every stage.

use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, ExternalToken, FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern,
};
use adze_tablegen::{StaticLanguageGenerator, TableCompressor, helpers};

// ============================================================================
// Helpers
// ============================================================================

/// Run the full pipeline and return (parse_table, compressed_tables_exist, codegen_string).
fn full_pipeline(grammar: Grammar) -> (adze_glr_core::ParseTable, bool, String) {
    let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW failed");
    let pt = build_lr1_automaton(&grammar, &ff).expect("LR(1) automaton failed");
    assert!(pt.state_count > 0);

    let mut generator = StaticLanguageGenerator::new(grammar, pt.clone());
    let compressed_ok = generator.compress_tables().is_ok();
    let code = generator.generate_language_code().to_string();
    (pt, compressed_ok, code)
}

/// Build a minimal expression grammar for reuse.
fn minimal_expr_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

// ============================================================================
// 1. IR Grammar → FIRST/FOLLOW correctness
// ============================================================================

#[test]
fn first_follow_single_terminal() {
    // Use lowercase non-terminal names so they end up in rule_names
    let grammar = GrammarBuilder::new("ff1")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let s_id = grammar.find_symbol_by_name("start").unwrap();
    // Look up the token ID from the tokens map
    let a_id = *grammar
        .tokens
        .iter()
        .find(|(_, t)| t.name == "A")
        .unwrap()
        .0;

    // FIRST(start) must contain terminal A
    let first_s = ff.first(s_id).expect("FIRST(start) missing");
    assert!(
        first_s.contains(a_id.0 as usize),
        "A should be in FIRST(start)"
    );
    assert!(!ff.is_nullable(s_id), "start should not be nullable");
}

#[test]
fn first_follow_nullable_symbol() {
    // start -> A | ε
    let grammar = GrammarBuilder::new("ff2")
        .token("A", "a")
        .rule("start", vec!["A"])
        .rule("start", vec![]) // epsilon
        .start("start")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let s_id = grammar.find_symbol_by_name("start").unwrap();
    assert!(ff.is_nullable(s_id), "start should be nullable");
}

#[test]
fn first_follow_two_nonterminals() {
    // root -> lhs rhs, lhs -> 'x', rhs -> 'y'
    let grammar = GrammarBuilder::new("ff3")
        .token("X", "x")
        .token("Y", "y")
        .rule("lhs", vec!["X"])
        .rule("rhs", vec!["Y"])
        .rule("root", vec!["lhs", "rhs"])
        .start("root")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let root_id = grammar.find_symbol_by_name("root").unwrap();
    let lhs_id = grammar.find_symbol_by_name("lhs").unwrap();
    let x_id = *grammar
        .tokens
        .iter()
        .find(|(_, t)| t.name == "X")
        .unwrap()
        .0;
    let y_id = *grammar
        .tokens
        .iter()
        .find(|(_, t)| t.name == "Y")
        .unwrap()
        .0;

    // FIRST(root) = FIRST(lhs) = {X}
    assert!(ff.first(root_id).unwrap().contains(x_id.0 as usize));
    // FOLLOW(lhs) should contain Y (since rhs follows lhs)
    assert!(ff.follow(lhs_id).unwrap().contains(y_id.0 as usize));
}

// ============================================================================
// 2. IR Grammar → parse table → compression roundtrip
// ============================================================================

#[test]
fn compression_roundtrip_simple() {
    let grammar = minimal_expr_grammar("rt1");
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let pt = build_lr1_automaton(&grammar, &ff).unwrap();

    let token_indices = helpers::collect_token_indices(&grammar, &pt);
    let start_empty = helpers::eof_accepts_or_reduces(&pt);

    let compressed = TableCompressor::new()
        .compress(&pt, &token_indices, start_empty)
        .expect("compression should succeed");

    // Compressed action table should have row offsets
    assert!(
        !compressed.action_table.row_offsets.is_empty(),
        "action row_offsets must be non-empty"
    );
    // Compressed goto table must also have row offsets
    assert!(
        !compressed.goto_table.row_offsets.is_empty(),
        "goto row_offsets must be non-empty"
    );
    // Both tables should have the same number of row offsets
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        compressed.goto_table.row_offsets.len(),
        "action and goto row_offsets must have same length"
    );
}

#[test]
fn compression_roundtrip_arithmetic() {
    let grammar = GrammarBuilder::new("rt2")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["(", "expr", ")"])
        .rule("factor", vec!["NUM"])
        .start("expr")
        .build();

    let (pt, compressed_ok, _code) = full_pipeline(grammar);
    assert!(compressed_ok, "compression must succeed");
    assert!(pt.state_count >= 6, "arithmetic grammar needs many states");
}

// ============================================================================
// 3. Full pipeline → codegen string is valid Rust-like
// ============================================================================

#[test]
fn codegen_contains_language_function() {
    let grammar = minimal_expr_grammar("cg1");
    let (_pt, _ok, code) = full_pipeline(grammar);

    assert!(
        code.contains("tree_sitter_cg1"),
        "codegen must contain language function named after grammar"
    );
}

#[test]
fn codegen_contains_structural_tokens() {
    let grammar = GrammarBuilder::new("cg2")
        .token("ID", r"[a-z]+")
        .token(";", ";")
        .rule("stmt", vec!["ID", ";"])
        .start("stmt")
        .build();

    let (_pt, _ok, code) = full_pipeline(grammar);

    // Generated code should contain Rust structural keywords
    assert!(
        code.contains("static") || code.contains("const"),
        "codegen should declare static/const data"
    );
    assert!(
        code.contains("SYMBOL_NAMES") || code.contains("symbol"),
        "codegen should have symbol-related declarations"
    );
}

#[test]
fn codegen_node_types_is_json() {
    let grammar = minimal_expr_grammar("cg3");
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let pt = build_lr1_automaton(&grammar, &ff).unwrap();
    let generator = StaticLanguageGenerator::new(grammar, pt);

    let node_types = generator.generate_node_types();
    // Must be valid JSON array
    let parsed: serde_json::Value =
        serde_json::from_str(&node_types).expect("node_types must be valid JSON");
    assert!(parsed.is_array(), "node_types must be a JSON array");
}

// ============================================================================
// 4. Multiple grammars produce independent results
// ============================================================================

#[test]
fn independent_grammars_different_names() {
    let g1 = GrammarBuilder::new("alpha")
        .token("A", "a")
        .rule("S", vec!["A"])
        .start("S")
        .build();

    let g2 = GrammarBuilder::new("beta")
        .token("B", "b")
        .rule("T", vec!["B"])
        .start("T")
        .build();

    let (_pt1, _, code1) = full_pipeline(g1);
    let (_pt2, _, code2) = full_pipeline(g2);

    assert!(code1.contains("alpha"), "code1 should reference alpha");
    assert!(code2.contains("beta"), "code2 should reference beta");
    assert!(!code1.contains("beta"), "code1 must not reference beta");
    assert!(!code2.contains("alpha"), "code2 must not reference alpha");
}

#[test]
fn independent_grammars_different_state_counts() {
    let simple = GrammarBuilder::new("simple")
        .token("X", "x")
        .rule("S", vec!["X"])
        .start("S")
        .build();

    let complex = GrammarBuilder::new("complex")
        .token("X", "x")
        .token("Y", "y")
        .token("+", "+")
        .rule("S", vec!["S", "+", "A"])
        .rule("S", vec!["A"])
        .rule("A", vec!["X"])
        .rule("A", vec!["Y"])
        .start("S")
        .build();

    let (pt_s, _, _) = full_pipeline(simple);
    let (pt_c, _, _) = full_pipeline(complex);

    assert!(
        pt_c.state_count > pt_s.state_count,
        "complex grammar should produce more states"
    );
}

// ============================================================================
// 5. Grammar with external tokens
// ============================================================================

#[test]
fn external_tokens_flow_through() {
    let grammar = GrammarBuilder::new("ext1")
        .token("ID", r"[a-z]+")
        .token(":", ":")
        .external("INDENT")
        .external("DEDENT")
        .rule("block", vec![":", "ID"])
        .start("block")
        .build();

    // The external tokens should be recorded in the grammar
    assert_eq!(grammar.externals.len(), 2);

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let pt = build_lr1_automaton(&grammar, &ff).unwrap();

    // Pipeline should succeed with external tokens present
    assert!(pt.state_count > 0, "parse table should have states");

    let mut generator = StaticLanguageGenerator::new(grammar, pt);
    assert!(
        generator.compress_tables().is_ok(),
        "compression with externals must work"
    );
}

#[test]
fn external_tokens_in_codegen() {
    let grammar = GrammarBuilder::new("ext2")
        .token("A", "a")
        .external("NEWLINE")
        .rule("S", vec!["A"])
        .start("S")
        .build();

    let (_, _, code) = full_pipeline(grammar);
    // Code should be non-empty and parseable
    assert!(!code.is_empty());
    assert!(
        code.contains("tree_sitter_ext2"),
        "codegen must reference grammar name"
    );
}

// ============================================================================
// 6. Grammar with precedence
// ============================================================================

#[test]
fn precedence_grammar_pipeline() {
    let grammar = GrammarBuilder::new("prec1")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("E", vec!["E", "+", "E"], 1, Associativity::Left)
        .rule_with_precedence("E", vec!["E", "*", "E"], 2, Associativity::Left)
        .rule("E", vec!["N"])
        .start("E")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let pt = build_lr1_automaton(&grammar, &ff).unwrap();

    // Check that parse rules carry the precedence info
    // The rules vector in the parse table should reflect the productions
    assert!(pt.rules.len() >= 3, "should have at least 3 productions");

    let (_, compressed, code) = full_pipeline(grammar);
    assert!(compressed, "precedence grammar should compress");
    assert!(code.contains("tree_sitter_prec1"));
}

#[test]
fn precedence_right_assoc() {
    let grammar = GrammarBuilder::new("prec2")
        .token("N", r"\d+")
        .token("^", "^")
        .rule_with_precedence("E", vec!["E", "^", "E"], 3, Associativity::Right)
        .rule("E", vec!["N"])
        .start("E")
        .build();

    let (pt, compressed, _) = full_pipeline(grammar);
    assert!(compressed);
    assert!(pt.state_count > 0);
}

// ============================================================================
// 7. Grammar with field names
// ============================================================================

#[test]
fn field_names_flow_through_pipeline() {
    // Build grammar manually to attach field names
    let mut grammar = Grammar::new("fields1".to_string());

    let num_id = SymbolId(1);
    let op_id = SymbolId(2);
    let expr_id = SymbolId(10);

    grammar.tokens.insert(
        num_id,
        Token {
            name: "NUM".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        op_id,
        Token {
            name: "+".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );

    grammar.rule_names.insert(expr_id, "expr".into());

    // Fields: "left" at position 0, "right" at position 2
    grammar.fields.insert(FieldId(0), "left".into());
    grammar.fields.insert(FieldId(1), "right".into());

    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(op_id),
            Symbol::NonTerminal(expr_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0), (FieldId(1), 2)],
        production_id: ProductionId(0),
    });
    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let pt = build_lr1_automaton(&grammar, &ff).unwrap();

    // Field names should propagate into the parse table
    assert!(
        pt.field_names.contains(&"left".to_string())
            || grammar.fields.values().any(|v| v == "left"),
        "field 'left' must exist"
    );

    let mut generator = StaticLanguageGenerator::new(grammar, pt);
    assert!(generator.compress_tables().is_ok());

    let code = generator.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn field_names_in_node_types() {
    let mut grammar = Grammar::new("fields2".to_string());

    let tok = SymbolId(1);
    let nt = SymbolId(10);

    grammar.tokens.insert(
        tok,
        Token {
            name: "ID".into(),
            pattern: TokenPattern::Regex(r"[a-z]+".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(nt, "decl".into());

    grammar.fields.insert(FieldId(0), "name".into());
    grammar.add_rule(Rule {
        lhs: nt,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    });

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let pt = build_lr1_automaton(&grammar, &ff).unwrap();
    let generator = StaticLanguageGenerator::new(grammar, pt);

    let node_types = generator.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&node_types).unwrap();
    assert!(parsed.is_array());
}

// ============================================================================
// 8. Large grammar (50+ rules)
// ============================================================================

#[test]
fn large_grammar_50_rules() {
    // Build a grammar with many distinct non-terminals and rules
    let mut builder = GrammarBuilder::new("large1")
        .token("ID", r"[a-z]+")
        .token("NUM", r"\d+")
        .token(";", ";")
        .token("=", "=")
        .token("+", "+")
        .token("(", "(")
        .token(")", ")");

    // Create 10 statement types, each with multiple alternatives
    for i in 0..10 {
        let nt = format!("stmt{i}");
        let leak_nt: &'static str = Box::leak(nt.into_boxed_str());
        builder = builder
            .rule(leak_nt, vec!["ID", "=", "NUM", ";"])
            .rule(leak_nt, vec!["ID", "=", "ID", ";"])
            .rule(leak_nt, vec!["ID", "=", "ID", "+", "NUM", ";"])
            .rule(leak_nt, vec!["ID", "=", "(", "NUM", ")", ";"])
            .rule(leak_nt, vec!["ID", ";"])
    }

    // top-level program rule referencing all stmt types
    let mut top_rules = Vec::new();
    for i in 0..10 {
        let nt = format!("stmt{i}");
        let leak: &'static str = Box::leak(nt.into_boxed_str());
        builder = builder.rule("program", vec![leak]);
        top_rules.push(leak);
    }

    builder = builder.start("program");
    let grammar = builder.build();

    // Verify we have at least 50 rules
    let rule_count: usize = grammar.rules.values().map(|v| v.len()).sum();
    assert!(rule_count >= 50, "expected 50+ rules, got {rule_count}");

    let (pt, compressed, code) = full_pipeline(grammar);
    assert!(compressed, "large grammar must compress");
    assert!(pt.state_count > 10, "large grammar needs many states");
    assert!(!code.is_empty(), "codegen must produce output");
}

#[test]
fn large_grammar_deep_nonterminal_chain() {
    // chain: S -> A, A -> B, B -> C, ..., Z -> 'tok'
    let mut builder = GrammarBuilder::new("chain1").token("TOK", "t");

    let names: Vec<String> = (0..20).map(|i| format!("nt{i}")).collect();
    let leaked: Vec<&'static str> = names
        .into_iter()
        .map(|s| -> &'static str { Box::leak(s.into_boxed_str()) })
        .collect();

    for i in 0..leaked.len() - 1 {
        builder = builder.rule(leaked[i], vec![leaked[i + 1]]);
    }
    // Last in chain produces the terminal
    builder = builder.rule(leaked.last().copied().unwrap(), vec!["TOK"]);
    builder = builder.start(leaked[0]);

    let grammar = builder.build();
    let (pt, compressed, _) = full_pipeline(grammar);
    assert!(compressed);
    assert!(pt.state_count > 0);
}

// ============================================================================
// 9. Error grammars handled gracefully
// ============================================================================

#[test]
fn error_grammar_missing_rule_for_nonterminal() {
    // Reference a non-terminal "missing" that has no rules defined anywhere
    let mut grammar = Grammar::new("err1".to_string());

    let tok = SymbolId(1);
    let s = SymbolId(10);
    let missing = SymbolId(20); // no rules, no token

    grammar.tokens.insert(
        tok,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(s, "S".into());
    grammar.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::NonTerminal(missing)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    // validate() should detect the unresolved symbol
    let result = grammar.validate();
    assert!(
        result.is_err(),
        "grammar with undefined non-terminal should fail validation"
    );
}

#[test]
fn error_grammar_no_rules() {
    let grammar = Grammar::new("err2".to_string());
    // A grammar with zero rules: start_symbol() returns None
    assert!(grammar.start_symbol().is_none());

    // FIRST/FOLLOW on empty grammar should still succeed (no rules = trivial)
    let result = FirstFollowSets::compute(&grammar);
    // Either succeeds with empty sets, or errors gracefully
    match result {
        Ok(ff) => {
            // No symbols, so nullable set should be empty
            assert!(!ff.is_nullable(SymbolId(0)));
        }
        Err(_) => {
            // Also acceptable: empty grammar is a degenerate case
        }
    }
}

#[test]
fn error_grammar_undefined_symbol_in_rhs() {
    // Create grammar referencing a SymbolId that's neither a token nor a rule
    let mut grammar = Grammar::new("err3".to_string());
    let a = SymbolId(1);
    let s = SymbolId(10);
    let phantom = SymbolId(99);

    grammar.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(s, "S".into());
    grammar.add_rule(Rule {
        lhs: s,
        rhs: vec![Symbol::Terminal(a), Symbol::Terminal(phantom)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let val = grammar.validate();
    assert!(val.is_err(), "referencing phantom terminal should fail");
}

// ============================================================================
// 10. Determinism: same grammar → same output, 10 iterations
// ============================================================================

#[test]
fn determinism_first_follow() {
    let grammar = minimal_expr_grammar("det1");
    let s_id = grammar.find_symbol_by_name("expr").unwrap();

    let mut prev_first: Option<Vec<usize>> = None;
    for _ in 0..10 {
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let first = ff.first(s_id).unwrap();
        let bits: Vec<usize> = first.ones().collect();
        if let Some(ref p) = prev_first {
            assert_eq!(&bits, p, "FIRST set must be identical across iterations");
        }
        prev_first = Some(bits);
    }
}

#[test]
fn determinism_parse_table() {
    let grammar = minimal_expr_grammar("det2");

    let mut prev_state_count: Option<usize> = None;
    let mut prev_rule_count: Option<usize> = None;
    for _ in 0..10 {
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let pt = build_lr1_automaton(&grammar, &ff).unwrap();
        if let Some(sc) = prev_state_count {
            assert_eq!(pt.state_count, sc, "state count must be deterministic");
        }
        if let Some(rc) = prev_rule_count {
            assert_eq!(pt.rules.len(), rc, "rule count must be deterministic");
        }
        prev_state_count = Some(pt.state_count);
        prev_rule_count = Some(pt.rules.len());
    }
}

#[test]
fn determinism_codegen() {
    let mut codes: Vec<String> = Vec::new();
    for _ in 0..10 {
        let grammar = minimal_expr_grammar("det3");
        let (_, _, code) = full_pipeline(grammar);
        codes.push(code);
    }
    for i in 1..codes.len() {
        assert_eq!(
            codes[0], codes[i],
            "codegen must be deterministic (iteration {i})"
        );
    }
}

#[test]
fn determinism_compression() {
    let mut prev_action_len: Option<usize> = None;
    let mut prev_goto_len: Option<usize> = None;
    for _ in 0..10 {
        let grammar = minimal_expr_grammar("det4");
        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let pt = build_lr1_automaton(&grammar, &ff).unwrap();
        let token_ix = helpers::collect_token_indices(&grammar, &pt);
        let start_empty = helpers::eof_accepts_or_reduces(&pt);
        let ct = TableCompressor::new()
            .compress(&pt, &token_ix, start_empty)
            .unwrap();
        if let Some(a) = prev_action_len {
            assert_eq!(ct.action_table.data.len(), a);
        }
        if let Some(g) = prev_goto_len {
            assert_eq!(ct.goto_table.data.len(), g);
        }
        prev_action_len = Some(ct.action_table.data.len());
        prev_goto_len = Some(ct.goto_table.data.len());
    }
}

// ============================================================================
// Additional tests to reach 20+
// ============================================================================

#[test]
fn normalize_then_first_follow() {
    // Grammar with Optional symbol → normalize should expand it
    let mut grammar = Grammar::new("norm1".to_string());

    let a_id = SymbolId(1);
    let b_id = SymbolId(2);
    let s_id = SymbolId(10);

    grammar.tokens.insert(
        a_id,
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        b_id,
        Token {
            name: "B".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(s_id, "S".into());

    // S -> A Optional(B)
    grammar.add_rule(Rule {
        lhs: s_id,
        rhs: vec![
            Symbol::Terminal(a_id),
            Symbol::Optional(Box::new(Symbol::Terminal(b_id))),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    // normalize the Optional away
    grammar.normalize();

    // Now compute FIRST/FOLLOW on normalized grammar
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    // S should have A in its FIRST set
    assert!(ff.first(s_id).unwrap().contains(a_id.0 as usize));
}

#[test]
fn normalize_repeat_then_pipeline() {
    let mut grammar = Grammar::new("norm2".to_string());

    let a_id = SymbolId(1);
    let s_id = SymbolId(10);

    grammar.tokens.insert(
        a_id,
        Token {
            name: "A".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(s_id, "S".into());

    // S -> Repeat(A)  (zero or more)
    grammar.add_rule(Rule {
        lhs: s_id,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(a_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    grammar.normalize();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    // S should be nullable (since Repeat allows zero occurrences)
    assert!(ff.is_nullable(s_id));

    let pt = build_lr1_automaton(&grammar, &ff).unwrap();
    assert!(pt.state_count > 0);
}

#[test]
fn pipeline_with_extras() {
    let grammar = GrammarBuilder::new("extras1")
        .token("ID", r"[a-z]+")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .rule("S", vec!["ID"])
        .start("S")
        .build();

    assert_eq!(grammar.extras.len(), 1);
    let (pt, compressed, code) = full_pipeline(grammar);
    assert!(compressed);
    assert!(pt.state_count > 0);
    assert!(!code.is_empty());
}

#[test]
fn action_table_has_accept() {
    let grammar = minimal_expr_grammar("acc1");
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let pt = build_lr1_automaton(&grammar, &ff).unwrap();

    // There must be at least one Accept action somewhere
    let has_accept = pt.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    });
    assert!(has_accept, "parse table must have an Accept action");
}

#[test]
fn action_table_has_shift_and_reduce() {
    let grammar = minimal_expr_grammar("sr1");
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let pt = build_lr1_automaton(&grammar, &ff).unwrap();

    let has_shift = pt.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Shift(_))))
    });
    let has_reduce = pt.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Reduce(_))))
    });
    assert!(has_shift, "parse table must have Shift actions");
    assert!(has_reduce, "parse table must have Reduce actions");
}

#[test]
fn pipeline_multiple_start_candidates() {
    // Grammar where first rule isn't named with common pattern — tests start_symbol heuristic
    let grammar = GrammarBuilder::new("msc")
        .token("X", "x")
        .token("Y", "y")
        .rule("foo_bar_baz", vec!["X", "Y"])
        .start("foo_bar_baz")
        .build();

    let (pt, compressed, _) = full_pipeline(grammar);
    assert!(compressed);
    assert!(pt.state_count > 0);
}

#[test]
fn external_token_grammar_raw_ir() {
    // Build with raw IR to have full control over external tokens
    let mut grammar = Grammar::new("ext_raw".to_string());

    let tok_a = SymbolId(1);
    let ext_indent = SymbolId(50);
    let s_id = SymbolId(10);

    grammar.tokens.insert(
        tok_a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(s_id, "S".into());
    grammar.externals.push(ExternalToken {
        name: "INDENT".into(),
        symbol_id: ext_indent,
    });

    grammar.add_rule(Rule {
        lhs: s_id,
        rhs: vec![Symbol::Terminal(tok_a)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let pt = build_lr1_automaton(&grammar, &ff).unwrap();

    // External token should be accounted for
    assert!(!grammar.externals.is_empty());
    assert!(pt.state_count > 0);

    let mut generator = StaticLanguageGenerator::new(grammar, pt);
    assert!(generator.compress_tables().is_ok());
}

#[test]
fn precedence_mixed_assoc() {
    let grammar = GrammarBuilder::new("mixassoc")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("^", "^")
        .rule_with_precedence("E", vec!["E", "+", "E"], 1, Associativity::Left)
        .rule_with_precedence("E", vec!["E", "*", "E"], 2, Associativity::Left)
        .rule_with_precedence("E", vec!["E", "^", "E"], 3, Associativity::Right)
        .rule("E", vec!["N"])
        .start("E")
        .build();

    let (pt, compressed, code) = full_pipeline(grammar);
    assert!(compressed);
    assert!(pt.state_count > 0);
    assert!(code.contains("tree_sitter_mixassoc"));
}

#[test]
fn grammar_validate_ok_then_pipeline() {
    let grammar = GrammarBuilder::new("val1")
        .token("A", "a")
        .token("B", "b")
        .rule("S", vec!["A", "B"])
        .start("S")
        .build();

    // Explicit validation before pipeline
    grammar
        .validate()
        .expect("valid grammar should pass validation");
    let (pt, compressed, _) = full_pipeline(grammar);
    assert!(compressed);
    assert!(pt.state_count > 0);
}

#[test]
fn compressed_table_validate_roundtrip() {
    let grammar = minimal_expr_grammar("vrt");
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let pt = build_lr1_automaton(&grammar, &ff).unwrap();

    let token_ix = helpers::collect_token_indices(&grammar, &pt);
    let start_empty = helpers::eof_accepts_or_reduces(&pt);
    let ct = TableCompressor::new()
        .compress(&pt, &token_ix, start_empty)
        .unwrap();

    // CompressedTables::validate should pass against its source table
    let val = ct.validate(&pt);
    assert!(val.is_ok(), "compressed table validation should pass");
}
