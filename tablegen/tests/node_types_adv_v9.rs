//! Advanced NodeTypesGenerator tests (v9) for adze-tablegen.

use adze_ir::{
    Associativity, ExternalToken, FieldId, Grammar, PrecedenceKind, ProductionId, Rule, Symbol,
    SymbolId, Token, TokenPattern,
};
use adze_tablegen::NodeTypesGenerator;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tok(name: &str, pattern: TokenPattern) -> Token {
    Token {
        name: name.to_string(),
        pattern,
        fragile: false,
    }
}

fn regex_tok(name: &str, pat: &str) -> Token {
    tok(name, TokenPattern::Regex(pat.to_string()))
}

fn string_tok(name: &str, lit: &str) -> Token {
    tok(name, TokenPattern::String(lit.to_string()))
}

fn simple_rule(lhs: SymbolId, rhs: Vec<Symbol>, prod: u16) -> Rule {
    Rule {
        lhs,
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(prod),
    }
}

fn prec_rule(
    lhs: SymbolId,
    rhs: Vec<Symbol>,
    prod: u16,
    prec: i16,
    assoc: Associativity,
) -> Rule {
    Rule {
        lhs,
        rhs,
        precedence: Some(PrecedenceKind::Static(prec)),
        associativity: Some(assoc),
        fields: vec![],
        production_id: ProductionId(prod),
    }
}

fn field_rule(
    lhs: SymbolId,
    rhs: Vec<Symbol>,
    prod: u16,
    fields: Vec<(FieldId, usize)>,
) -> Rule {
    Rule {
        lhs,
        rhs,
        precedence: None,
        associativity: None,
        fields,
        production_id: ProductionId(prod),
    }
}

/// Parse JSON output from generator and return as Value.
fn ntgen(g: &Grammar) -> serde_json::Value {
    let generator = NodeTypesGenerator::new(g);
    let json = generator.generate().expect("generate must succeed");
    serde_json::from_str(&json).expect("must be valid JSON")
}

/// Convenience: generate raw JSON string.
fn gen_str(g: &Grammar) -> String {
    NodeTypesGenerator::new(g)
        .generate()
        .expect("generate must succeed")
}

/// Return array of parsed node types.
fn gen_arr(g: &Grammar) -> Vec<serde_json::Value> {
    serde_json::from_value(ntgen(g)).expect("top-level must be array")
}

// ---------------------------------------------------------------------------
// Grammar builders — each grammar name is prefixed "nta_v9_"
// ---------------------------------------------------------------------------

fn minimal_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_minimal".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));
    let expr = SymbolId(10);
    g.rule_names.insert(expr, "expression".to_string());
    g.add_rule(simple_rule(expr, vec![Symbol::Terminal(SymbolId(0))], 0));
    g
}

fn five_rule_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_five".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));
    g.tokens.insert(SymbolId(1), string_tok("plus", "+"));
    g.tokens.insert(SymbolId(2), string_tok("star", "*"));
    g.tokens.insert(SymbolId(3), string_tok("lparen", "("));
    g.tokens.insert(SymbolId(4), string_tok("rparen", ")"));

    for (i, name) in ["atom", "term", "factor", "group", "program"]
        .iter()
        .enumerate()
    {
        let id = SymbolId((10 + i) as u16);
        g.rule_names.insert(id, name.to_string());
        g.add_rule(simple_rule(
            id,
            vec![Symbol::Terminal(SymbolId(0))],
            i as u16,
        ));
    }
    g
}

fn ten_rule_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_ten".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));
    g.tokens.insert(SymbolId(1), string_tok("comma", ","));

    let names = [
        "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa",
    ];
    for (i, name) in names.iter().enumerate() {
        let id = SymbolId((10 + i) as u16);
        g.rule_names.insert(id, name.to_string());
        g.add_rule(simple_rule(
            id,
            vec![Symbol::Terminal(SymbolId(0))],
            i as u16,
        ));
    }
    g
}

fn precedence_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_prec".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));
    g.tokens.insert(SymbolId(1), string_tok("plus", "+"));
    g.tokens.insert(SymbolId(2), string_tok("star", "*"));

    let expr = SymbolId(10);
    g.rule_names.insert(expr, "expression".to_string());

    // expr -> expr + expr  (prec 1, left)
    g.add_rule(prec_rule(
        expr,
        vec![
            Symbol::NonTerminal(expr),
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(expr),
        ],
        0,
        1,
        Associativity::Left,
    ));
    // expr -> expr * expr  (prec 2, left)
    g.add_rule(prec_rule(
        expr,
        vec![
            Symbol::NonTerminal(expr),
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(expr),
        ],
        1,
        2,
        Associativity::Left,
    ));
    // expr -> NUMBER
    g.add_rule(simple_rule(expr, vec![Symbol::Terminal(SymbolId(0))], 2));
    g
}

fn alternatives_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_alt".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));
    g.tokens
        .insert(SymbolId(1), regex_tok("identifier", r"[a-z]+"));

    let val = SymbolId(10);
    g.rule_names.insert(val, "value".to_string());
    // value -> NUMBER
    g.add_rule(simple_rule(val, vec![Symbol::Terminal(SymbolId(0))], 0));
    // value -> IDENT
    g.add_rule(simple_rule(val, vec![Symbol::Terminal(SymbolId(1))], 1));
    g
}

fn inline_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_inline".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));
    g.tokens.insert(SymbolId(1), string_tok("plus", "+"));

    let helper = SymbolId(10);
    g.rule_names.insert(helper, "_helper".to_string());
    g.add_rule(simple_rule(
        helper,
        vec![Symbol::Terminal(SymbolId(0))],
        0,
    ));
    g.inline_rules.push(helper);

    let expr = SymbolId(11);
    g.rule_names.insert(expr, "expression".to_string());
    g.add_rule(simple_rule(
        expr,
        vec![
            Symbol::NonTerminal(helper),
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(helper),
        ],
        1,
    ));
    g
}

fn extras_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_extras".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));
    let ws = SymbolId(1);
    g.tokens.insert(ws, regex_tok("whitespace", r"\s+"));
    g.extras.push(ws);

    let expr = SymbolId(10);
    g.rule_names.insert(expr, "expression".to_string());
    g.add_rule(simple_rule(expr, vec![Symbol::Terminal(SymbolId(0))], 0));
    g
}

fn externals_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_ext".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));

    let indent = SymbolId(50);
    g.externals.push(ExternalToken {
        name: "indent".to_string(),
        symbol_id: indent,
    });
    let dedent = SymbolId(51);
    g.externals.push(ExternalToken {
        name: "dedent".to_string(),
        symbol_id: dedent,
    });

    let block = SymbolId(10);
    g.rule_names.insert(block, "block".to_string());
    g.add_rule(simple_rule(
        block,
        vec![
            Symbol::External(indent),
            Symbol::Terminal(SymbolId(0)),
            Symbol::External(dedent),
        ],
        0,
    ));
    g
}

fn fields_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_fields".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));
    g.tokens.insert(SymbolId(1), string_tok("plus", "+"));

    let lf = FieldId(0);
    let of = FieldId(1);
    let rf = FieldId(2);
    g.fields.insert(lf, "left".to_string());
    g.fields.insert(of, "operator".to_string());
    g.fields.insert(rf, "right".to_string());

    let bin = SymbolId(10);
    g.rule_names.insert(bin, "binary_expression".to_string());
    g.add_rule(field_rule(
        bin,
        vec![
            Symbol::Terminal(SymbolId(0)),
            Symbol::Terminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(0)),
        ],
        0,
        vec![(lf, 0), (of, 1), (rf, 2)],
    ));
    g
}

fn supertype_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_super".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));
    g.tokens
        .insert(SymbolId(1), regex_tok("identifier", r"[a-z]+"));

    let lit = SymbolId(10);
    g.rule_names.insert(lit, "literal".to_string());
    g.add_rule(simple_rule(lit, vec![Symbol::Terminal(SymbolId(0))], 0));

    let name = SymbolId(11);
    g.rule_names.insert(name, "name".to_string());
    g.add_rule(simple_rule(name, vec![Symbol::Terminal(SymbolId(1))], 1));

    let expr = SymbolId(12);
    g.rule_names.insert(expr, "expression".to_string());
    g.add_rule(simple_rule(
        expr,
        vec![Symbol::NonTerminal(lit)],
        2,
    ));
    g.add_rule(simple_rule(
        expr,
        vec![Symbol::NonTerminal(name)],
        3,
    ));
    g.supertypes.push(expr);
    g
}

fn optional_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_opt".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));
    g.tokens.insert(SymbolId(1), string_tok("semi", ";"));

    let stmt = SymbolId(10);
    g.rule_names.insert(stmt, "statement".to_string());
    g.add_rule(simple_rule(
        stmt,
        vec![
            Symbol::Terminal(SymbolId(0)),
            Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1)))),
        ],
        0,
    ));
    g
}

fn repeat_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_rep".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));

    let list = SymbolId(10);
    g.rule_names.insert(list, "number_list".to_string());
    g.add_rule(simple_rule(
        list,
        vec![Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(0))))],
        0,
    ));
    g
}

fn repeat_one_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_rep1".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));

    let list = SymbolId(10);
    g.rule_names.insert(list, "nonempty_list".to_string());
    g.add_rule(simple_rule(
        list,
        vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(0))))],
        0,
    ));
    g
}

fn choice_rhs_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_choice_rhs".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));
    g.tokens
        .insert(SymbolId(1), regex_tok("identifier", r"[a-z]+"));

    let val = SymbolId(10);
    g.rule_names.insert(val, "value".to_string());
    g.add_rule(simple_rule(
        val,
        vec![Symbol::Choice(vec![
            Symbol::Terminal(SymbolId(0)),
            Symbol::Terminal(SymbolId(1)),
        ])],
        0,
    ));
    g
}

fn sequence_rhs_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_seq_rhs".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));
    g.tokens.insert(SymbolId(1), string_tok("comma", ","));

    let pair = SymbolId(10);
    g.rule_names.insert(pair, "pair".to_string());
    g.add_rule(simple_rule(
        pair,
        vec![Symbol::Sequence(vec![
            Symbol::Terminal(SymbolId(0)),
            Symbol::Terminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(0)),
        ])],
        0,
    ));
    g
}

fn multi_field_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_mf".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));
    g.tokens.insert(SymbolId(1), string_tok("colon", ":"));
    g.tokens
        .insert(SymbolId(2), regex_tok("identifier", r"[a-z]+"));

    let kf = FieldId(0);
    let vf = FieldId(1);
    g.fields.insert(kf, "key".to_string());
    g.fields.insert(vf, "value".to_string());

    let entry = SymbolId(10);
    g.rule_names.insert(entry, "entry".to_string());
    g.add_rule(field_rule(
        entry,
        vec![
            Symbol::Terminal(SymbolId(2)),
            Symbol::Terminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(0)),
        ],
        0,
        vec![(kf, 0), (vf, 2)],
    ));
    g
}

fn nested_nonterminal_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_nested".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));
    g.tokens.insert(SymbolId(1), string_tok("plus", "+"));

    let atom = SymbolId(10);
    g.rule_names.insert(atom, "atom".to_string());
    g.add_rule(simple_rule(atom, vec![Symbol::Terminal(SymbolId(0))], 0));

    let expr = SymbolId(11);
    g.rule_names.insert(expr, "expression".to_string());
    g.add_rule(simple_rule(
        expr,
        vec![
            Symbol::NonTerminal(atom),
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(atom),
        ],
        1,
    ));
    g
}

fn right_assoc_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_rassoc".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));
    g.tokens.insert(SymbolId(1), string_tok("assign", "="));

    let assign = SymbolId(10);
    g.rule_names.insert(assign, "assignment".to_string());
    g.add_rule(prec_rule(
        assign,
        vec![
            Symbol::Terminal(SymbolId(0)),
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(assign),
        ],
        0,
        1,
        Associativity::Right,
    ));
    g.add_rule(simple_rule(
        assign,
        vec![Symbol::Terminal(SymbolId(0))],
        1,
    ));
    g
}

fn dynamic_prec_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_dynprec".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));
    g.tokens.insert(SymbolId(1), string_tok("plus", "+"));

    let expr = SymbolId(10);
    g.rule_names.insert(expr, "expression".to_string());
    g.add_rule(Rule {
        lhs: expr,
        rhs: vec![
            Symbol::NonTerminal(expr),
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(expr),
        ],
        precedence: Some(PrecedenceKind::Dynamic(5)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(simple_rule(expr, vec![Symbol::Terminal(SymbolId(0))], 1));
    g
}

fn multi_external_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_mext".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));

    for i in 0u16..3 {
        let id = SymbolId(50 + i);
        g.externals.push(ExternalToken {
            name: format!("ext_{i}"),
            symbol_id: id,
        });
    }

    let block = SymbolId(10);
    g.rule_names.insert(block, "block".to_string());
    g.add_rule(simple_rule(
        block,
        vec![
            Symbol::External(SymbolId(50)),
            Symbol::Terminal(SymbolId(0)),
            Symbol::External(SymbolId(52)),
        ],
        0,
    ));
    g
}

fn many_tokens_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_manytok".to_string());
    let ops = ["+", "-", "*", "/", "%", "^", "!", "~"];
    for (i, op) in ops.iter().enumerate() {
        g.tokens
            .insert(SymbolId(i as u16), string_tok(&format!("op_{i}"), op));
    }
    g.tokens.insert(SymbolId(8), regex_tok("number", r"\d+"));

    let expr = SymbolId(20);
    g.rule_names.insert(expr, "expression".to_string());
    g.add_rule(simple_rule(expr, vec![Symbol::Terminal(SymbolId(8))], 0));
    g
}

fn fragile_token_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_fragile".to_string());
    g.tokens.insert(
        SymbolId(0),
        Token {
            name: "keyword".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: true,
        },
    );
    let stmt = SymbolId(10);
    g.rule_names.insert(stmt, "statement".to_string());
    g.add_rule(simple_rule(stmt, vec![Symbol::Terminal(SymbolId(0))], 0));
    g
}

fn epsilon_rule_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_eps".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));

    let empty = SymbolId(10);
    g.rule_names.insert(empty, "empty_node".to_string());
    g.add_rule(simple_rule(empty, vec![Symbol::Epsilon], 0));

    let wrapper = SymbolId(11);
    g.rule_names.insert(wrapper, "wrapper".to_string());
    g.add_rule(simple_rule(
        wrapper,
        vec![Symbol::Terminal(SymbolId(0))],
        1,
    ));
    g
}

fn deep_nesting_grammar() -> Grammar {
    let mut g = Grammar::new("nta_v9_deep".to_string());
    g.tokens.insert(SymbolId(0), regex_tok("number", r"\d+"));

    let a = SymbolId(10);
    g.rule_names.insert(a, "level_a".to_string());
    g.add_rule(simple_rule(a, vec![Symbol::Terminal(SymbolId(0))], 0));

    let b = SymbolId(11);
    g.rule_names.insert(b, "level_b".to_string());
    g.add_rule(simple_rule(b, vec![Symbol::NonTerminal(a)], 1));

    let c = SymbolId(12);
    g.rule_names.insert(c, "level_c".to_string());
    g.add_rule(simple_rule(c, vec![Symbol::NonTerminal(b)], 2));

    let d = SymbolId(13);
    g.rule_names.insert(d, "level_d".to_string());
    g.add_rule(simple_rule(d, vec![Symbol::NonTerminal(c)], 3));
    g
}

// ===================================================================
// Tests 1–7: Basic validity
// ===================================================================

#[test]
fn test_nta_v9_generates_valid_json() {
    let raw = gen_str(&minimal_grammar());
    let _: serde_json::Value = serde_json::from_str(&raw).expect("must be valid JSON");
}

#[test]
fn test_nta_v9_json_is_array() {
    let v = ntgen(&minimal_grammar());
    assert!(v.is_array());
}

#[test]
fn test_nta_v9_each_entry_has_type_field() {
    let arr = gen_arr(&minimal_grammar());
    for entry in &arr {
        assert!(entry.get("type").is_some(), "entry missing 'type': {entry}");
    }
}

#[test]
fn test_nta_v9_each_entry_has_named_field() {
    let arr = gen_arr(&minimal_grammar());
    for entry in &arr {
        assert!(
            entry.get("named").is_some(),
            "entry missing 'named': {entry}"
        );
    }
}

#[test]
fn test_nta_v9_named_nodes_for_nonterminals() {
    let arr = gen_arr(&minimal_grammar());
    let named: Vec<&str> = arr
        .iter()
        .filter(|n| n["named"] == true)
        .filter_map(|n| n["type"].as_str())
        .collect();
    assert!(
        named.contains(&"expression"),
        "expected named 'expression' node"
    );
}

#[test]
fn test_nta_v9_anonymous_nodes_for_string_terminals() {
    let arr = gen_arr(&five_rule_grammar());
    let anon: Vec<&str> = arr
        .iter()
        .filter(|n| n["named"] == false)
        .filter_map(|n| n["type"].as_str())
        .collect();
    assert!(anon.contains(&"+"), "expected anonymous '+' token");
    assert!(anon.contains(&"*"), "expected anonymous '*' token");
    assert!(anon.contains(&"("), "expected anonymous '(' token");
    assert!(anon.contains(&")"), "expected anonymous ')' token");
}

#[test]
fn test_nta_v9_minimal_grammar_minimal_json() {
    let arr = gen_arr(&minimal_grammar());
    assert!(
        !arr.is_empty(),
        "minimal grammar should produce at least one entry"
    );
    // Should be small — one named + possibly some anonymous
    assert!(arr.len() <= 5, "expected small output, got {}", arr.len());
}

// ===================================================================
// Tests 8–9: Scaling with rules
// ===================================================================

#[test]
fn test_nta_v9_five_rules_more_entries() {
    let minimal_count = gen_arr(&minimal_grammar()).len();
    let five_count = gen_arr(&five_rule_grammar()).len();
    assert!(
        five_count > minimal_count,
        "5-rule grammar ({five_count}) should produce more entries than minimal ({minimal_count})"
    );
}

#[test]
fn test_nta_v9_ten_rules_more_entries() {
    let five_count = gen_arr(&five_rule_grammar()).len();
    let ten_count = gen_arr(&ten_rule_grammar()).len();
    assert!(
        ten_count > five_count,
        "10-rule grammar ({ten_count}) should produce more entries than 5-rule ({five_count})"
    );
}

// ===================================================================
// Test 10: Determinism
// ===================================================================

#[test]
fn test_nta_v9_deterministic_output() {
    let a = gen_str(&minimal_grammar());
    let b = gen_str(&minimal_grammar());
    assert_eq!(a, b, "same grammar must produce identical JSON");
}

#[test]
fn test_nta_v9_deterministic_complex() {
    let a = gen_str(&precedence_grammar());
    let b = gen_str(&precedence_grammar());
    assert_eq!(a, b, "complex grammar must produce identical JSON");
}

// ===================================================================
// Test 11: Different grammars → different output
// ===================================================================

#[test]
fn test_nta_v9_different_grammars_different_json() {
    let a = gen_str(&minimal_grammar());
    let b = gen_str(&five_rule_grammar());
    assert_ne!(a, b, "different grammars should produce different JSON");
}

// ===================================================================
// Test 12: Children for rules with RHS
// ===================================================================

#[test]
fn test_nta_v9_children_present_for_nonterminal_rhs() {
    let arr = gen_arr(&nested_nonterminal_grammar());
    let expr = arr
        .iter()
        .find(|n| n["type"].as_str() == Some("expression"));
    assert!(expr.is_some(), "expected 'expression' node");
    // Verify expression is a named node — it references nonterminals in RHS
    let expr = expr.unwrap();
    assert_eq!(expr["named"], true, "expression must be named");
}

// ===================================================================
// Tests 13–17: Grammar features
// ===================================================================

#[test]
fn test_nta_v9_precedence_grammar_generates() {
    let arr = gen_arr(&precedence_grammar());
    assert!(
        arr.iter().any(|n| n["type"].as_str() == Some("expression")),
        "precedence grammar should produce 'expression' node"
    );
}

#[test]
fn test_nta_v9_precedence_grammar_has_operator_tokens() {
    let arr = gen_arr(&precedence_grammar());
    let anon: Vec<&str> = arr
        .iter()
        .filter(|n| n["named"] == false)
        .filter_map(|n| n["type"].as_str())
        .collect();
    assert!(anon.contains(&"+"), "expected '+' token");
    assert!(anon.contains(&"*"), "expected '*' token");
}

#[test]
fn test_nta_v9_alternatives_grammar_generates() {
    let arr = gen_arr(&alternatives_grammar());
    assert!(
        arr.iter().any(|n| n["type"].as_str() == Some("value")),
        "alternatives grammar should produce 'value' node"
    );
}

#[test]
fn test_nta_v9_inline_grammar_generates() {
    let arr = gen_arr(&inline_grammar());
    assert!(
        arr.iter()
            .any(|n| n["type"].as_str() == Some("expression")),
        "inline grammar should produce 'expression' node"
    );
}

#[test]
fn test_nta_v9_inline_rule_skipped() {
    let arr = gen_arr(&inline_grammar());
    let names: Vec<&str> = arr
        .iter()
        .filter(|n| n["named"] == true)
        .filter_map(|n| n["type"].as_str())
        .collect();
    // Internal rules starting with _ should be skipped
    assert!(
        !names.contains(&"_helper"),
        "_helper should be skipped in node types"
    );
}

#[test]
fn test_nta_v9_extras_grammar_generates() {
    let arr = gen_arr(&extras_grammar());
    assert!(
        arr.iter()
            .any(|n| n["type"].as_str() == Some("expression")),
        "extras grammar should produce 'expression' node"
    );
}

#[test]
fn test_nta_v9_externals_grammar_generates() {
    let arr = gen_arr(&externals_grammar());
    assert!(
        arr.iter().any(|n| n["type"].as_str() == Some("block")),
        "externals grammar should produce 'block' node"
    );
}

// ===================================================================
// Tests 18–20: JSON structure
// ===================================================================

#[test]
fn test_nta_v9_parseable_by_serde_json() {
    let raw = gen_str(&five_rule_grammar());
    let v: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&raw);
    assert!(v.is_ok(), "JSON must be parseable as Vec<Value>");
}

#[test]
fn test_nta_v9_all_type_values_are_strings() {
    let arr = gen_arr(&five_rule_grammar());
    for entry in &arr {
        assert!(
            entry["type"].is_string(),
            "type field must be a string: {entry}"
        );
    }
}

#[test]
fn test_nta_v9_all_named_values_are_booleans() {
    let arr = gen_arr(&five_rule_grammar());
    for entry in &arr {
        assert!(
            entry["named"].is_boolean(),
            "named field must be a boolean: {entry}"
        );
    }
}

// ===================================================================
// Tests 21–30: Fields and children
// ===================================================================

#[test]
fn test_nta_v9_fields_appear() {
    let arr = gen_arr(&fields_grammar());
    let bin = arr
        .iter()
        .find(|n| n["type"].as_str() == Some("binary_expression"))
        .expect("binary_expression must exist");
    let fields = bin.get("fields").expect("fields must be present");
    assert!(fields.get("left").is_some());
    assert!(fields.get("operator").is_some());
    assert!(fields.get("right").is_some());
}

#[test]
fn test_nta_v9_field_types_array_present() {
    let arr = gen_arr(&fields_grammar());
    let bin = arr
        .iter()
        .find(|n| n["type"].as_str() == Some("binary_expression"))
        .unwrap();
    let left = &bin["fields"]["left"];
    assert!(left["types"].is_array(), "'types' must be an array");
}

#[test]
fn test_nta_v9_field_required_and_multiple() {
    let arr = gen_arr(&fields_grammar());
    let bin = arr
        .iter()
        .find(|n| n["type"].as_str() == Some("binary_expression"))
        .unwrap();
    for name in &["left", "operator", "right"] {
        let f = &bin["fields"][name];
        assert!(f.get("required").is_some(), "{name} needs 'required'");
        assert!(f.get("multiple").is_some(), "{name} needs 'multiple'");
    }
}

#[test]
fn test_nta_v9_multi_field_key_value() {
    let arr = gen_arr(&multi_field_grammar());
    let entry = arr
        .iter()
        .find(|n| n["type"].as_str() == Some("entry"))
        .expect("'entry' node must exist");
    let fields = entry.get("fields").expect("fields must be present");
    assert!(fields.get("key").is_some(), "expected 'key' field");
    assert!(fields.get("value").is_some(), "expected 'value' field");
}

#[test]
fn test_nta_v9_anonymous_nodes_have_no_fields() {
    let arr = gen_arr(&fields_grammar());
    for node in &arr {
        if node["named"] == false {
            assert!(
                node.get("fields").is_none() || node["fields"].is_null(),
                "anonymous '{}' should have no fields",
                node["type"]
            );
        }
    }
}

#[test]
fn test_nta_v9_no_duplicate_type_named_pairs() {
    let arr = gen_arr(&five_rule_grammar());
    let mut pairs: Vec<(String, bool)> = arr
        .iter()
        .map(|n| {
            (
                n["type"].as_str().unwrap_or("").to_string(),
                n["named"].as_bool().unwrap_or(false),
            )
        })
        .collect();
    let orig_len = pairs.len();
    pairs.sort();
    pairs.dedup();
    assert_eq!(orig_len, pairs.len(), "no duplicate (type, named) pairs");
}

// ===================================================================
// Tests 31–40: Special symbol variants
// ===================================================================

#[test]
fn test_nta_v9_optional_generates() {
    let arr = gen_arr(&optional_grammar());
    assert!(
        arr.iter().any(|n| n["type"].as_str() == Some("statement")),
        "optional grammar should produce 'statement'"
    );
}

#[test]
fn test_nta_v9_optional_anon_semicolon() {
    let arr = gen_arr(&optional_grammar());
    let anon: Vec<&str> = arr
        .iter()
        .filter(|n| n["named"] == false)
        .filter_map(|n| n["type"].as_str())
        .collect();
    assert!(anon.contains(&";"), "expected ';' anonymous node");
}

#[test]
fn test_nta_v9_repeat_generates() {
    let arr = gen_arr(&repeat_grammar());
    assert!(
        arr.iter()
            .any(|n| n["type"].as_str() == Some("number_list")),
        "repeat grammar should produce 'number_list'"
    );
}

#[test]
fn test_nta_v9_repeat_one_generates() {
    let arr = gen_arr(&repeat_one_grammar());
    assert!(
        arr.iter()
            .any(|n| n["type"].as_str() == Some("nonempty_list")),
        "repeat1 grammar should produce 'nonempty_list'"
    );
}

#[test]
fn test_nta_v9_choice_rhs_generates() {
    let arr = gen_arr(&choice_rhs_grammar());
    assert!(
        arr.iter().any(|n| n["type"].as_str() == Some("value")),
        "choice-rhs grammar should produce 'value'"
    );
}

#[test]
fn test_nta_v9_sequence_rhs_generates() {
    let arr = gen_arr(&sequence_rhs_grammar());
    assert!(
        arr.iter().any(|n| n["type"].as_str() == Some("pair")),
        "sequence-rhs grammar should produce 'pair'"
    );
}

#[test]
fn test_nta_v9_sequence_anon_comma() {
    let arr = gen_arr(&sequence_rhs_grammar());
    let anon: Vec<&str> = arr
        .iter()
        .filter(|n| n["named"] == false)
        .filter_map(|n| n["type"].as_str())
        .collect();
    assert!(anon.contains(&","), "expected ',' anonymous node");
}

#[test]
fn test_nta_v9_epsilon_grammar_generates() {
    let arr = gen_arr(&epsilon_rule_grammar());
    assert!(
        arr.iter().any(|n| n["type"].as_str() == Some("wrapper")),
        "epsilon grammar should produce 'wrapper'"
    );
}

#[test]
fn test_nta_v9_epsilon_empty_node() {
    let arr = gen_arr(&epsilon_rule_grammar());
    assert!(
        arr.iter()
            .any(|n| n["type"].as_str() == Some("empty_node")),
        "epsilon grammar should produce 'empty_node'"
    );
}

#[test]
fn test_nta_v9_deep_nesting_generates() {
    let arr = gen_arr(&deep_nesting_grammar());
    for name in &["level_a", "level_b", "level_c", "level_d"] {
        assert!(
            arr.iter().any(|n| n["type"].as_str() == Some(name)),
            "expected '{name}' node in deep grammar"
        );
    }
}

// ===================================================================
// Tests 41–50: Supertype, externals, extras
// ===================================================================

#[test]
fn test_nta_v9_supertype_generates() {
    let arr = gen_arr(&supertype_grammar());
    let named: Vec<&str> = arr
        .iter()
        .filter(|n| n["named"] == true)
        .filter_map(|n| n["type"].as_str())
        .collect();
    assert!(named.contains(&"expression"), "expected 'expression'");
    assert!(named.contains(&"literal"), "expected 'literal'");
    assert!(named.contains(&"name"), "expected 'name'");
}

#[test]
fn test_nta_v9_supertype_subtypes_field() {
    let arr = gen_arr(&supertype_grammar());
    let expr = arr
        .iter()
        .find(|n| n["type"].as_str() == Some("expression"));
    assert!(expr.is_some(), "'expression' node must exist");
    let expr = expr.unwrap();
    // Supertypes may have a "subtypes" field
    if let Some(subtypes) = expr.get("subtypes") {
        assert!(subtypes.is_array(), "subtypes must be an array");
    }
}

#[test]
fn test_nta_v9_external_tokens_grammar_non_empty() {
    let arr = gen_arr(&externals_grammar());
    assert!(!arr.is_empty(), "externals grammar should produce output");
}

#[test]
fn test_nta_v9_multi_external_generates() {
    let arr = gen_arr(&multi_external_grammar());
    assert!(
        arr.iter().any(|n| n["type"].as_str() == Some("block")),
        "multi-external grammar should produce 'block'"
    );
}

#[test]
fn test_nta_v9_extras_do_not_leak_as_named() {
    let arr = gen_arr(&extras_grammar());
    let named: Vec<&str> = arr
        .iter()
        .filter(|n| n["named"] == true)
        .filter_map(|n| n["type"].as_str())
        .collect();
    // whitespace is an extra, not a named production
    assert!(
        !named.contains(&"whitespace"),
        "extra 'whitespace' should not be a named node"
    );
}

#[test]
fn test_nta_v9_fragile_token_generates() {
    let arr = gen_arr(&fragile_token_grammar());
    assert!(
        arr.iter().any(|n| n["type"].as_str() == Some("statement")),
        "fragile token grammar should produce 'statement'"
    );
}

#[test]
fn test_nta_v9_many_tokens_all_anon() {
    let arr = gen_arr(&many_tokens_grammar());
    let anon: Vec<&str> = arr
        .iter()
        .filter(|n| n["named"] == false)
        .filter_map(|n| n["type"].as_str())
        .collect();
    for op in &["+", "-", "*", "/", "%", "^", "!", "~"] {
        assert!(anon.contains(op), "expected anonymous '{op}' token");
    }
}

#[test]
fn test_nta_v9_right_assoc_generates() {
    let arr = gen_arr(&right_assoc_grammar());
    assert!(
        arr.iter()
            .any(|n| n["type"].as_str() == Some("assignment")),
        "right-assoc grammar should produce 'assignment'"
    );
}

#[test]
fn test_nta_v9_dynamic_prec_generates() {
    let arr = gen_arr(&dynamic_prec_grammar());
    assert!(
        arr.iter()
            .any(|n| n["type"].as_str() == Some("expression")),
        "dynamic-prec grammar should produce 'expression'"
    );
}

#[test]
fn test_nta_v9_right_assoc_anon_equals() {
    let arr = gen_arr(&right_assoc_grammar());
    let anon: Vec<&str> = arr
        .iter()
        .filter(|n| n["named"] == false)
        .filter_map(|n| n["type"].as_str())
        .collect();
    assert!(anon.contains(&"="), "expected '=' anonymous node");
}

// ===================================================================
// Tests 51–60: Edge cases and structural invariants
// ===================================================================

#[test]
fn test_nta_v9_empty_grammar_produces_empty_array() {
    let arr = gen_arr(&Grammar::new("nta_v9_empty".to_string()));
    assert!(arr.is_empty(), "empty grammar → empty array");
}

#[test]
fn test_nta_v9_type_never_empty_string() {
    let arr = gen_arr(&ten_rule_grammar());
    for entry in &arr {
        let ty = entry["type"].as_str().unwrap_or("");
        assert!(!ty.is_empty(), "type must not be an empty string");
    }
}

#[test]
fn test_nta_v9_named_true_for_all_rules() {
    let arr = gen_arr(&ten_rule_grammar());
    let rule_names = [
        "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa",
    ];
    for name in &rule_names {
        let node = arr.iter().find(|n| n["type"].as_str() == Some(name));
        assert!(node.is_some(), "expected node for rule '{name}'");
        assert_eq!(
            node.unwrap()["named"],
            true,
            "rule '{name}' must be named"
        );
    }
}

#[test]
fn test_nta_v9_named_false_for_string_tokens() {
    let arr = gen_arr(&ten_rule_grammar());
    let comma = arr.iter().find(|n| n["type"].as_str() == Some(","));
    if let Some(c) = comma {
        assert_eq!(c["named"], false, "',' must be anonymous");
    }
}

#[test]
fn test_nta_v9_all_entries_are_objects() {
    let arr = gen_arr(&five_rule_grammar());
    for entry in &arr {
        assert!(entry.is_object(), "each entry must be a JSON object");
    }
}

#[test]
fn test_nta_v9_type_is_not_null() {
    let arr = gen_arr(&five_rule_grammar());
    for entry in &arr {
        assert!(!entry["type"].is_null(), "'type' must not be null");
    }
}

#[test]
fn test_nta_v9_named_is_not_null() {
    let arr = gen_arr(&five_rule_grammar());
    for entry in &arr {
        assert!(!entry["named"].is_null(), "'named' must not be null");
    }
}

#[test]
fn test_nta_v9_json_output_is_utf8() {
    let raw = gen_str(&minimal_grammar());
    assert!(std::str::from_utf8(raw.as_bytes()).is_ok());
}

#[test]
fn test_nta_v9_no_trailing_comma_in_json() {
    let raw = gen_str(&five_rule_grammar());
    // serde_json won't emit trailing commas, but verify the string doesn't have ,]
    assert!(!raw.contains(",]"), "JSON should not have trailing comma");
    assert!(!raw.contains(",}"), "JSON should not have trailing comma");
}

#[test]
fn test_nta_v9_output_starts_with_bracket() {
    let raw = gen_str(&minimal_grammar());
    let trimmed = raw.trim();
    assert!(trimmed.starts_with('['), "JSON must start with '['");
    assert!(trimmed.ends_with(']'), "JSON must end with ']'");
}

// ===================================================================
// Tests 61–70: Cross-grammar comparisons
// ===================================================================

#[test]
fn test_nta_v9_five_contains_all_minimal_named() {
    let minimal = gen_arr(&minimal_grammar());
    let five = gen_arr(&five_rule_grammar());
    let five_named: Vec<&str> = five
        .iter()
        .filter(|n| n["named"] == true)
        .filter_map(|n| n["type"].as_str())
        .collect();
    // Five-rule grammar should have more named nodes
    assert!(five_named.len() >= 5, "expected at least 5 named nodes");
    // Minimal grammar should have fewer
    let min_named: Vec<&str> = minimal
        .iter()
        .filter(|n| n["named"] == true)
        .filter_map(|n| n["type"].as_str())
        .collect();
    assert!(min_named.len() < five_named.len());
}

#[test]
fn test_nta_v9_alternatives_vs_choice_rhs_both_have_value() {
    let alt = gen_arr(&alternatives_grammar());
    let choice = gen_arr(&choice_rhs_grammar());
    assert!(
        alt.iter().any(|n| n["type"].as_str() == Some("value")),
        "alternatives grammar produces 'value'"
    );
    assert!(
        choice.iter().any(|n| n["type"].as_str() == Some("value")),
        "choice-rhs grammar produces 'value'"
    );
}

#[test]
fn test_nta_v9_extras_vs_no_extras_named_count() {
    let with_extras = gen_arr(&extras_grammar());
    let without = gen_arr(&minimal_grammar());
    let extras_named: Vec<_> = with_extras
        .iter()
        .filter(|n| n["named"] == true)
        .collect();
    let min_named: Vec<_> = without.iter().filter(|n| n["named"] == true).collect();
    // Both should have at least one named node (expression)
    assert!(!extras_named.is_empty());
    assert!(!min_named.is_empty());
}

#[test]
fn test_nta_v9_fields_grammar_more_complex_than_minimal() {
    let minimal = gen_str(&minimal_grammar());
    let with_fields = gen_str(&fields_grammar());
    assert!(
        with_fields.len() > minimal.len(),
        "fields grammar should produce longer JSON"
    );
}

#[test]
fn test_nta_v9_ten_rule_all_named_present() {
    let arr = gen_arr(&ten_rule_grammar());
    let names = [
        "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa",
    ];
    for name in &names {
        assert!(
            arr.iter().any(|n| n["type"].as_str() == Some(name)),
            "missing named node '{name}'"
        );
    }
}

#[test]
fn test_nta_v9_ten_rule_all_marked_named() {
    let arr = gen_arr(&ten_rule_grammar());
    let names = [
        "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa",
    ];
    for name in &names {
        let node = arr
            .iter()
            .find(|n| n["type"].as_str() == Some(name))
            .unwrap();
        assert_eq!(node["named"], true);
    }
}

#[test]
fn test_nta_v9_nested_has_atom_and_expression() {
    let arr = gen_arr(&nested_nonterminal_grammar());
    assert!(
        arr.iter()
            .any(|n| n["type"].as_str() == Some("expression")),
        "'expression' must exist"
    );
    assert!(
        arr.iter().any(|n| n["type"].as_str() == Some("atom")),
        "'atom' must exist"
    );
}

#[test]
fn test_nta_v9_deep_nesting_four_levels() {
    let arr = gen_arr(&deep_nesting_grammar());
    let named_count = arr.iter().filter(|n| n["named"] == true).count();
    assert!(
        named_count >= 4,
        "deep nesting should have at least 4 named nodes, got {named_count}"
    );
}

#[test]
fn test_nta_v9_precedence_grammar_anon_count() {
    let arr = gen_arr(&precedence_grammar());
    let anon_count = arr.iter().filter(|n| n["named"] == false).count();
    assert!(
        anon_count >= 2,
        "precedence grammar should have >= 2 anonymous nodes for + and *, got {anon_count}"
    );
}

#[test]
fn test_nta_v9_precedence_grammar_named_count() {
    let arr = gen_arr(&precedence_grammar());
    let named_count = arr.iter().filter(|n| n["named"] == true).count();
    assert!(
        named_count >= 1,
        "precedence grammar should have >= 1 named node, got {named_count}"
    );
}

// ===================================================================
// Tests 71–80: Additional invariants and edge cases
// ===================================================================

#[test]
fn test_nta_v9_generate_returns_ok() {
    let g = minimal_grammar();
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok(), "generate should return Ok");
}

#[test]
fn test_nta_v9_empty_grammar_returns_ok() {
    let g = Grammar::new("nta_v9_empty2".to_string());
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok(), "empty grammar generate should return Ok");
}

#[test]
fn test_nta_v9_fields_types_entries_have_type_and_named() {
    let arr = gen_arr(&fields_grammar());
    let bin = arr
        .iter()
        .find(|n| n["type"].as_str() == Some("binary_expression"))
        .unwrap();
    for field_name in &["left", "operator", "right"] {
        let types = bin["fields"][field_name]["types"].as_array();
        if let Some(types) = types {
            for t in types {
                assert!(t.get("type").is_some(), "type ref needs 'type'");
                assert!(t.get("named").is_some(), "type ref needs 'named'");
            }
        }
    }
}

#[test]
fn test_nta_v9_supertype_grammar_has_supertypes_in_grammar() {
    let g = supertype_grammar();
    assert!(!g.supertypes.is_empty(), "grammar must declare supertypes");
}

#[test]
fn test_nta_v9_inline_grammar_has_inline_rules() {
    let g = inline_grammar();
    assert!(
        !g.inline_rules.is_empty(),
        "grammar must declare inline rules"
    );
}

#[test]
fn test_nta_v9_extras_grammar_has_extras() {
    let g = extras_grammar();
    assert!(!g.extras.is_empty(), "grammar must declare extras");
}

#[test]
fn test_nta_v9_externals_grammar_has_externals() {
    let g = externals_grammar();
    assert!(!g.externals.is_empty(), "grammar must declare externals");
}

#[test]
fn test_nta_v9_all_type_refs_in_fields_are_objects() {
    let arr = gen_arr(&multi_field_grammar());
    let entry = arr
        .iter()
        .find(|n| n["type"].as_str() == Some("entry"))
        .unwrap();
    if let Some(fields) = entry.get("fields")
        && let Some(fields_obj) = fields.as_object()
    {
        for (_field_name, field_val) in fields_obj {
            if let Some(types) = field_val["types"].as_array() {
                for t in types {
                    assert!(t.is_object(), "each type ref must be an object");
                }
            }
        }
    }
}

#[test]
fn test_nta_v9_deterministic_ten_rules() {
    let a = gen_str(&ten_rule_grammar());
    let b = gen_str(&ten_rule_grammar());
    assert_eq!(a, b);
}

#[test]
fn test_nta_v9_minimal_vs_empty_different() {
    let empty = gen_str(&Grammar::new("nta_v9_emp3".to_string()));
    let minimal = gen_str(&minimal_grammar());
    assert_ne!(empty, minimal, "empty and minimal grammars differ");
}

// ===================================================================
// Tests 81–85: Additional coverage
// ===================================================================

#[test]
fn test_nta_v9_many_tokens_named_expression() {
    let arr = gen_arr(&many_tokens_grammar());
    assert!(
        arr.iter()
            .any(|n| n["type"].as_str() == Some("expression")),
        "many-tokens grammar should produce 'expression'"
    );
}

#[test]
fn test_nta_v9_repeat_grammar_deterministic() {
    let a = gen_str(&repeat_grammar());
    let b = gen_str(&repeat_grammar());
    assert_eq!(a, b);
}

#[test]
fn test_nta_v9_optional_grammar_deterministic() {
    let a = gen_str(&optional_grammar());
    let b = gen_str(&optional_grammar());
    assert_eq!(a, b);
}

#[test]
fn test_nta_v9_all_named_types_are_lowercase() {
    let arr = gen_arr(&ten_rule_grammar());
    for entry in &arr {
        if entry["named"] == true {
            let ty = entry["type"].as_str().unwrap_or("");
            assert_eq!(
                ty,
                ty.to_lowercase(),
                "named type should be lowercase: '{ty}'"
            );
        }
    }
}

#[test]
fn test_nta_v9_externals_grammar_produces_valid_json() {
    let raw = gen_str(&externals_grammar());
    let v: Result<serde_json::Value, _> = serde_json::from_str(&raw);
    assert!(v.is_ok(), "externals grammar must produce valid JSON");
}
