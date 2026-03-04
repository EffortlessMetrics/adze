// Wave 131: Comprehensive tests for tablegen StaticLanguageGenerator
use adze_glr_core::*;
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;
use adze_tablegen::*;

// =====================================================================
// Helper: build a minimal grammar + parse table
// =====================================================================

fn build_simple_grammar_and_table() -> (Grammar, ParseTable) {
    let mut g = GrammarBuilder::new("simple")
        .token("x", r"x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("compute ff");
    let table = build_lr1_automaton(&g, &ff).expect("build automaton");
    (g, table)
}

fn build_two_alt_grammar_and_table() -> (Grammar, ParseTable) {
    let mut g = GrammarBuilder::new("two_alt")
        .token("a", r"a")
        .token("b", r"b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("compute ff");
    let table = build_lr1_automaton(&g, &ff).expect("build automaton");
    (g, table)
}

// =====================================================================
// StaticLanguageGenerator construction
// =====================================================================

#[test]
fn generator_new() {
    let (g, t) = build_simple_grammar_and_table();
    let _ = StaticLanguageGenerator::new(g, t);
}

#[test]
fn generator_generate_language_code() {
    let (g, t) = build_simple_grammar_and_table();
    let generator = StaticLanguageGenerator::new(g, t);
    let code = generator.generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn generator_generate_node_types() {
    let (g, t) = build_simple_grammar_and_table();
    let generator = StaticLanguageGenerator::new(g, t);
    let node_types = generator.generate_node_types();
    assert!(!node_types.is_empty());
}

#[test]
fn generator_two_alt() {
    let (g, t) = build_two_alt_grammar_and_table();
    let generator = StaticLanguageGenerator::new(g, t);
    let code = generator.generate_language_code();
    assert!(!code.to_string().is_empty());
}

// =====================================================================
// NodeTypesGenerator tests
// =====================================================================

#[test]
fn node_types_simple_grammar() {
    let (g, _) = build_simple_grammar_and_table();
    let generator = NodeTypesGenerator::new(&g);
    let result = generator.generate();
    assert!(result.is_ok());
}

#[test]
fn node_types_two_alt_grammar() {
    let (g, _) = build_two_alt_grammar_and_table();
    let generator = NodeTypesGenerator::new(&g);
    let result = generator.generate();
    assert!(result.is_ok());
}

#[test]
fn node_types_is_valid_json() {
    let (g, _) = build_simple_grammar_and_table();
    let generator = NodeTypesGenerator::new(&g);
    let json_str = generator.generate().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Valid JSON");
    assert!(parsed.is_array());
}

#[test]
fn node_types_empty_grammar() {
    let g = Grammar::new("empty".to_string());
    let generator = NodeTypesGenerator::new(&g);
    let result = generator.generate();
    assert!(result.is_ok());
}

// =====================================================================
// AbiLanguageBuilder tests
// =====================================================================

#[test]
fn abi_builder_new() {
    let (g, t) = build_simple_grammar_and_table();
    let _ = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn abi_builder_generate() {
    let (g, t) = build_simple_grammar_and_table();
    let builder = AbiLanguageBuilder::new(&g, &t);
    let code = builder.generate();
    assert!(!code.to_string().is_empty());
}

#[test]
fn abi_builder_two_alt() {
    let (g, t) = build_two_alt_grammar_and_table();
    let builder = AbiLanguageBuilder::new(&g, &t);
    let code = builder.generate();
    assert!(!code.to_string().is_empty());
}

// =====================================================================
// Compression tests
// =====================================================================

#[test]
fn compress_simple_table() {
    let (g, t) = build_simple_grammar_and_table();
    let token_indices = collect_token_indices(&g, &t);
    let compressor = TableCompressor::new();
    let result = compressor.compress(&t, &token_indices, false);
    assert!(result.is_ok(), "Compress failed: {:?}", result.err());
}

#[test]
fn compress_two_alt_table() {
    let (g, t) = build_two_alt_grammar_and_table();
    let token_indices = collect_token_indices(&g, &t);
    let compressor = TableCompressor::new();
    let result = compressor.compress(&t, &token_indices, false);
    assert!(result.is_ok(), "Compress failed: {:?}", result.err());
}

#[test]
fn collect_token_indices_includes_eof() {
    let (g, t) = build_simple_grammar_and_table();
    let indices = collect_token_indices(&g, &t);
    if let Some(&idx) = t.symbol_to_index.get(&t.eof_symbol) {
        assert!(indices.contains(&idx), "Token indices should include EOF");
    }
}

// =====================================================================
// Determinism
// =====================================================================

#[test]
fn generation_is_deterministic() {
    let (g1, t1) = build_simple_grammar_and_table();
    let (g2, t2) = build_simple_grammar_and_table();
    let gen1 = StaticLanguageGenerator::new(g1, t1);
    let gen2 = StaticLanguageGenerator::new(g2, t2);
    let code1 = gen1.generate_language_code().to_string();
    let code2 = gen2.generate_language_code().to_string();
    assert_eq!(code1, code2);
}

#[test]
fn node_types_generation_deterministic() {
    let (g1, _) = build_simple_grammar_and_table();
    let (g2, _) = build_simple_grammar_and_table();
    let gen1 = NodeTypesGenerator::new(&g1);
    let gen2 = NodeTypesGenerator::new(&g2);
    assert_eq!(gen1.generate().unwrap(), gen2.generate().unwrap());
}

// =====================================================================
// Preset grammar tests
// =====================================================================

#[test]
fn python_like_node_types() {
    let g = GrammarBuilder::python_like();
    let generator = NodeTypesGenerator::new(&g);
    assert!(generator.generate().is_ok());
}

#[test]
fn javascript_like_node_types() {
    let g = GrammarBuilder::javascript_like();
    let generator = NodeTypesGenerator::new(&g);
    assert!(generator.generate().is_ok());
}
