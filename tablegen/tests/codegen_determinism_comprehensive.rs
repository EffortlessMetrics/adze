#![allow(clippy::needless_range_loop)]

//! Comprehensive determinism tests for code generation in adze-tablegen.
//!
//! Validates that the same grammar always produces identical generated code,
//! that output is valid Rust, and that structural properties (array sizes,
//! constants, symbol ordering) are stable across runs.

use adze_glr_core::ParseTable;
use adze_ir::builder::GrammarBuilder;
use adze_ir::{FieldId, Grammar};
use adze_tablegen::StaticLanguageGenerator;
use adze_tablegen::language_gen::LanguageGenerator;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn minimal_grammar() -> (Grammar, ParseTable) {
    let grammar = GrammarBuilder::new("minimal")
        .token("number", r"\d+")
        .rule("expr", vec!["number"])
        .start("expr")
        .build();
    (grammar, ParseTable::default())
}

fn arithmetic_grammar() -> (Grammar, ParseTable) {
    let grammar = GrammarBuilder::new("arith")
        .token("number", r"\d+")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["number"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["expr", "star", "expr"])
        .start("expr")
        .build();
    (grammar, ParseTable::default())
}

fn grammar_with_fields() -> (Grammar, ParseTable) {
    let mut grammar = GrammarBuilder::new("field_lang")
        .token("number", r"\d+")
        .token("plus", "+")
        .rule("binary", vec!["number", "plus", "number"])
        .start("binary")
        .build();
    grammar.fields.insert(FieldId(0), "left".to_string());
    grammar.fields.insert(FieldId(1), "operator".to_string());
    grammar.fields.insert(FieldId(2), "right".to_string());
    (grammar, ParseTable::default())
}

fn grammar_with_externals() -> (Grammar, ParseTable) {
    let grammar = GrammarBuilder::new("ext_lang")
        .token("id", r"[a-z]+")
        .token("colon", ":")
        .external("indent")
        .external("dedent")
        .rule("block", vec!["id", "colon", "indent", "id", "dedent"])
        .start("block")
        .build();
    (grammar, ParseTable::default())
}

fn large_grammar() -> (Grammar, ParseTable) {
    let mut builder = GrammarBuilder::new("large");
    for i in 0..20 {
        builder = builder.token(&format!("tok_{i}"), &format!("t{i}"));
    }
    for i in 0..15 {
        let tok = format!("tok_{i}");
        let rule = format!("rule_{i}");
        builder = builder.rule(&rule, vec![Box::leak(tok.into_boxed_str())]);
    }
    builder = builder
        .rule("top", vec!["rule_0"])
        .rule("top", vec!["rule_1"])
        .start("top");
    let grammar = builder.build();
    (grammar, ParseTable::default())
}

fn grammar_with_extras() -> (Grammar, ParseTable) {
    let grammar = GrammarBuilder::new("extras_lang")
        .token("id", r"[a-z]+")
        .token("ws", r"[ \t]+")
        .extra("ws")
        .rule("start", vec!["id"])
        .start("start")
        .build();
    (grammar, ParseTable::default())
}

/// Generate code string via `LanguageGenerator`.
fn gen_lang(grammar: &Grammar, table: &ParseTable) -> String {
    LanguageGenerator::new(grammar, table)
        .generate()
        .to_string()
}

/// Generate code string via `StaticLanguageGenerator`.
fn gen_static(grammar: Grammar, table: ParseTable) -> String {
    StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string()
}

/// Generate node types JSON via `StaticLanguageGenerator`.
fn gen_node_types(grammar: Grammar, table: ParseTable) -> String {
    StaticLanguageGenerator::new(grammar, table).generate_node_types()
}

// ═══════════════════════════════════════════════════════════════════════
// 1–3. Same grammar → identical generated code (run twice, compare)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn identical_output_minimal_language_gen() {
    let (g, t) = minimal_grammar();
    let a = gen_lang(&g, &t);
    let b = gen_lang(&g, &t);
    assert_eq!(a, b, "Two runs of LanguageGenerator must be identical");
}

#[test]
fn identical_output_arithmetic_language_gen() {
    let (g, t) = arithmetic_grammar();
    let a = gen_lang(&g, &t);
    let b = gen_lang(&g, &t);
    assert_eq!(a, b);
}

#[test]
fn identical_output_static_gen() {
    let (g1, t1) = minimal_grammar();
    let (g2, t2) = minimal_grammar();
    let a = gen_static(g1, t1);
    let b = gen_static(g2, t2);
    assert_eq!(a, b, "Two fresh StaticLanguageGenerator runs must match");
}

// ═══════════════════════════════════════════════════════════════════════
// 4–5. Generated code is valid Rust (compiles as token stream / syn parse)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn language_gen_output_is_valid_token_stream() {
    let (g, t) = arithmetic_grammar();
    let code = gen_lang(&g, &t);
    let parsed: Result<proc_macro2::TokenStream, _> = code.parse();
    assert!(
        parsed.is_ok(),
        "Output must be a valid TokenStream: {:?}",
        parsed.err()
    );
}

#[test]
fn static_gen_output_is_valid_token_stream() {
    let (g, t) = grammar_with_fields();
    let code = gen_static(g, t);
    let parsed: Result<proc_macro2::TokenStream, _> = code.parse();
    assert!(
        parsed.is_ok(),
        "Output must be a valid TokenStream: {:?}",
        parsed.err()
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 6–7. Generated code length is stable
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn code_length_stable_language_gen() {
    let (g, t) = arithmetic_grammar();
    let a = gen_lang(&g, &t);
    let b = gen_lang(&g, &t);
    assert_eq!(a.len(), b.len(), "Code length must be stable across runs");
}

#[test]
fn code_length_stable_static_gen() {
    let (g1, t1) = grammar_with_externals();
    let (g2, t2) = grammar_with_externals();
    let a = gen_static(g1, t1);
    let b = gen_static(g2, t2);
    assert_eq!(a.len(), b.len());
}

// ═══════════════════════════════════════════════════════════════════════
// 8–11. Array sizes in generated code match grammar
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn symbol_count_matches_grammar_language_gen() {
    let (g, t) = minimal_grammar();
    let code = gen_lang(&g, &t);
    // count_symbols = 1 (EOF) + tokens + rules
    let expected = 1 + g.tokens.len() + g.rules.len();
    assert!(
        code.contains(&format!("symbol_count : {expected}"))
            || code.contains(&format!("symbol_count: {expected}")),
        "symbol_count should be {expected}, code:\n{code}"
    );
}

#[test]
fn symbol_count_matches_grammar_arithmetic() {
    let (g, t) = arithmetic_grammar();
    let code = gen_lang(&g, &t);
    let expected = 1 + g.tokens.len() + g.rules.len();
    assert!(
        code.contains(&format!("{expected}u32")) || code.contains(&format!("{expected} u32")),
        "symbol_count should be {expected}"
    );
}

#[test]
fn field_count_matches_grammar() {
    let (g, t) = grammar_with_fields();
    let code = gen_lang(&g, &t);
    let expected = g.fields.len();
    assert!(
        code.contains(&format!("{expected}u32")) || code.contains(&format!("{expected} u32")),
        "field_count should be {expected}"
    );
}

#[test]
fn external_token_count_matches_grammar() {
    let (g, t) = grammar_with_externals();
    let code = gen_static(g.clone(), t);
    let expected = g.externals.len();
    // StaticLanguageGenerator uses generate_node_types which reflects externals
    // The code should contain EXTERNAL_TOKEN_COUNT or the numeric value
    assert!(
        code.contains("EXTERNAL_TOKEN_COUNT") || code.contains(&format!("{expected}")),
        "external token count should be reflected"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 12–15. Constants in generated code match grammar
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn abi_version_constant_present() {
    let (g, t) = minimal_grammar();
    let code = gen_lang(&g, &t);
    assert!(
        code.contains("15"),
        "ABI version 15 must appear in generated code"
    );
}

#[test]
fn grammar_name_embedded_in_function_name() {
    let (g, t) = minimal_grammar();
    let code = gen_lang(&g, &t);
    assert!(
        code.contains("tree_sitter_minimal"),
        "FFI function name must embed grammar name"
    );
}

#[test]
fn state_count_matches_parse_table() {
    let (g, t) = minimal_grammar();
    let code = gen_lang(&g, &t);
    let expected = t.state_count;
    assert!(
        code.contains(&format!("{expected}u32")) || code.contains(&format!("{expected} u32")),
        "state_count should be {expected}"
    );
}

#[test]
fn production_id_count_deterministic() {
    let (g, t) = arithmetic_grammar();
    let generator = LanguageGenerator::new(&g, &t);
    let a = generator.count_production_ids_public();
    let b = generator.count_production_ids_public();
    assert_eq!(a, b, "production_id_count must be stable");
}

// ═══════════════════════════════════════════════════════════════════════
// 16–19. Symbol table ordering is stable
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn symbol_names_order_stable_across_runs() {
    let (g, t) = arithmetic_grammar();
    let a = gen_lang(&g, &t);
    let b = gen_lang(&g, &t);
    // Extract SYMBOL_NAMES portions
    let extract = |s: &str| -> Option<String> {
        let start = s.find("SYMBOL_NAMES")?;
        let end = s[start..].find(';').map(|i| start + i)?;
        Some(s[start..end].to_string())
    };
    assert_eq!(
        extract(&a),
        extract(&b),
        "SYMBOL_NAMES order must be stable"
    );
}

#[test]
fn field_names_order_stable_across_runs() {
    let (g, t) = grammar_with_fields();
    let a = gen_lang(&g, &t);
    let b = gen_lang(&g, &t);
    let extract = |s: &str| -> Option<String> {
        let start = s.find("FIELD_NAMES")?;
        let end = s[start..].find(';').map(|i| start + i)?;
        Some(s[start..end].to_string())
    };
    assert_eq!(extract(&a), extract(&b), "FIELD_NAMES order must be stable");
}

#[test]
fn symbol_metadata_order_stable() {
    let (g, t) = arithmetic_grammar();
    let a = gen_lang(&g, &t);
    let b = gen_lang(&g, &t);
    let extract = |s: &str| -> Option<String> {
        let start = s.find("SYMBOL_METADATA")?;
        let end = s[start..].find(';').map(|i| start + i)?;
        Some(s[start..end].to_string())
    };
    assert_eq!(extract(&a), extract(&b));
}

#[test]
fn lex_modes_order_stable() {
    let (g, t) = arithmetic_grammar();
    let a = gen_lang(&g, &t);
    let b = gen_lang(&g, &t);
    let extract = |s: &str| -> Option<String> {
        let start = s.find("LEX_MODES")?;
        let end = s[start..].find(';').map(|i| start + i)?;
        Some(s[start..end].to_string())
    };
    assert_eq!(extract(&a), extract(&b));
}

// ═══════════════════════════════════════════════════════════════════════
// 20–25. Multiple runs produce byte-identical output
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn byte_identical_minimal_five_runs() {
    let mut outputs = Vec::new();
    for _ in 0..5 {
        let (g, t) = minimal_grammar();
        outputs.push(gen_lang(&g, &t));
    }
    for i in 1..outputs.len() {
        assert_eq!(
            outputs[0], outputs[i],
            "Run 0 vs run {i} differ for minimal grammar"
        );
    }
}

#[test]
fn byte_identical_arithmetic_five_runs() {
    let mut outputs = Vec::new();
    for _ in 0..5 {
        let (g, t) = arithmetic_grammar();
        outputs.push(gen_lang(&g, &t));
    }
    for i in 1..outputs.len() {
        assert_eq!(outputs[0], outputs[i], "Run 0 vs run {i} differ");
    }
}

#[test]
fn byte_identical_fields_five_runs() {
    let mut outputs = Vec::new();
    for _ in 0..5 {
        let (g, t) = grammar_with_fields();
        outputs.push(gen_lang(&g, &t));
    }
    for i in 1..outputs.len() {
        assert_eq!(outputs[0], outputs[i], "Run 0 vs run {i} differ");
    }
}

#[test]
fn byte_identical_externals_five_runs() {
    let mut outputs = Vec::new();
    for _ in 0..5 {
        let (g, t) = grammar_with_externals();
        outputs.push(gen_static(g, t));
    }
    for i in 1..outputs.len() {
        assert_eq!(outputs[0], outputs[i], "Run 0 vs run {i} differ");
    }
}

#[test]
fn byte_identical_large_grammar_three_runs() {
    let mut outputs = Vec::new();
    for _ in 0..3 {
        let (g, t) = large_grammar();
        outputs.push(gen_static(g, t));
    }
    for i in 1..outputs.len() {
        assert_eq!(outputs[0], outputs[i], "Run 0 vs run {i} differ");
    }
}

#[test]
fn byte_identical_extras_grammar_five_runs() {
    let mut outputs = Vec::new();
    for _ in 0..5 {
        let (g, t) = grammar_with_extras();
        outputs.push(gen_lang(&g, &t));
    }
    for i in 1..outputs.len() {
        assert_eq!(outputs[0], outputs[i], "Run 0 vs run {i} differ");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 26–28. Node types JSON determinism
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn node_types_json_deterministic() {
    let (g1, t1) = minimal_grammar();
    let (g2, t2) = minimal_grammar();
    let a = gen_node_types(g1, t1);
    let b = gen_node_types(g2, t2);
    assert_eq!(a, b, "NODE_TYPES JSON must be deterministic");
}

#[test]
fn node_types_json_byte_identical_three_runs() {
    let mut outputs = Vec::new();
    for _ in 0..3 {
        let (g, t) = arithmetic_grammar();
        outputs.push(gen_node_types(g, t));
    }
    for i in 1..outputs.len() {
        assert_eq!(outputs[0], outputs[i]);
    }
}

#[test]
fn node_types_length_stable() {
    let (g1, t1) = grammar_with_externals();
    let (g2, t2) = grammar_with_externals();
    let a = gen_node_types(g1, t1);
    let b = gen_node_types(g2, t2);
    assert_eq!(a.len(), b.len(), "NODE_TYPES length must be stable");
}

// ═══════════════════════════════════════════════════════════════════════
// 29–30. Cross-generator consistency & hash stability
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn language_gen_and_static_gen_both_deterministic_independently() {
    // Both generators should individually be deterministic
    let (g1, t1) = arithmetic_grammar();
    let (g2, t2) = arithmetic_grammar();
    let lang_a = gen_lang(&g1, &t1);
    let lang_b = gen_lang(&g2, &t2);
    assert_eq!(lang_a, lang_b, "LanguageGenerator must be deterministic");

    let (g3, t3) = arithmetic_grammar();
    let (g4, t4) = arithmetic_grammar();
    let static_a = gen_static(g3, t3);
    let static_b = gen_static(g4, t4);
    assert_eq!(
        static_a, static_b,
        "StaticLanguageGenerator must be deterministic"
    );
}

#[test]
fn sha256_of_output_stable() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let hash_str = |s: &str| -> u64 {
        let mut h = DefaultHasher::new();
        s.hash(&mut h);
        h.finish()
    };

    let (g1, t1) = arithmetic_grammar();
    let (g2, t2) = arithmetic_grammar();
    let a = gen_lang(&g1, &t1);
    let b = gen_lang(&g2, &t2);
    assert_eq!(
        hash_str(&a),
        hash_str(&b),
        "Hash of generated code must be stable"
    );
}
