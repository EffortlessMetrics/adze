//! Comprehensive tests for NodeTypesGenerator advanced patterns.

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::NodeTypesGenerator;

fn make_simple_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

// ── Construction ──

#[test]
fn node_types_gen_new() {
    let g = make_simple_grammar();
    let _gen = NodeTypesGenerator::new(&g);
}

#[test]
fn node_types_gen_generate() {
    let g = make_simple_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let result = generator.generate();
    assert!(result.is_ok());
}

#[test]
fn node_types_gen_output_nonempty() {
    let g = make_simple_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let output = generator.generate().unwrap();
    assert!(!output.is_empty());
}

#[test]
fn node_types_gen_output_json() {
    let g = make_simple_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let output = generator.generate().unwrap();
    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(parsed.is_array());
}

// ── Different grammar sizes ──

#[test]
fn node_types_two_tokens() {
    let g = GrammarBuilder::new("tt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    assert!(!output.is_empty());
}

#[test]
fn node_types_chain() {
    let g = GrammarBuilder::new("ch")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    assert!(!output.is_empty());
}

#[test]
fn node_types_recursive() {
    let g = GrammarBuilder::new("rec")
        .token("x", "x")
        .token("p", "+")
        .rule("e", vec!["x"])
        .rule("e", vec!["e", "p", "x"])
        .start("e")
        .build();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    assert!(!output.is_empty());
}

#[test]
fn node_types_precedence() {
    let g = GrammarBuilder::new("prec")
        .token("n", "n")
        .token("p", "+")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "p", "e"], 1, Associativity::Left)
        .start("e")
        .build();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    assert!(!output.is_empty());
}

#[test]
fn node_types_large() {
    let mut b = GrammarBuilder::new("large");
    for i in 0..15 {
        let n: &str = Box::leak(format!("t{}", i).into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let g = b.start("s").build();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    assert!(!output.is_empty());
}

// ── Determinism ──

#[test]
fn node_types_deterministic() {
    let make = || {
        let g = make_simple_grammar();
        NodeTypesGenerator::new(&g).generate().unwrap()
    };
    assert_eq!(make(), make());
}

#[test]
fn node_types_deterministic_multi_rule() {
    let make = || {
        let g = GrammarBuilder::new("dm")
            .token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a"])
            .rule("s", vec!["b"])
            .start("s")
            .build();
        NodeTypesGenerator::new(&g).generate().unwrap()
    };
    assert_eq!(make(), make());
}

// ── Output format checks ──

#[test]
fn node_types_contains_type_field() {
    let g = make_simple_grammar();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    if let serde_json::Value::Array(arr) = parsed {
        if let Some(first) = arr.first() {
            assert!(first.get("type").is_some());
        }
    }
}

#[test]
fn node_types_contains_named_field() {
    let g = make_simple_grammar();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    if let serde_json::Value::Array(arr) = parsed {
        if let Some(first) = arr.first() {
            assert!(first.get("named").is_some());
        }
    }
}

// ── After normalize ──

#[test]
fn node_types_after_normalize() {
    let mut g = GrammarBuilder::new("norm")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    g.normalize();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    assert!(!output.is_empty());
}

// ── Multiple calls ──

#[test]
fn node_types_multiple_calls_same_gen() {
    let g = make_simple_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let r1 = generator.generate().unwrap();
    let r2 = generator.generate().unwrap();
    assert_eq!(r1, r2);
}

// ── Empty grammar ──

#[test]
fn node_types_empty_grammar() {
    let g = GrammarBuilder::new("empty").build();
    let generator = NodeTypesGenerator::new(&g);
    let result = generator.generate();
    // May be empty array or error - just don't crash
    let _ = result;
}

// ── Tokens only ──

#[test]
fn node_types_tokens_only() {
    let g = GrammarBuilder::new("tok")
        .token("a", "a")
        .token("b", "b")
        .build();
    let generator = NodeTypesGenerator::new(&g);
    let result = generator.generate();
    let _ = result;
}

// ── Unicode grammar name ──

#[test]
fn node_types_unicode_grammar() {
    let g = GrammarBuilder::new("日本語")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    assert!(!output.is_empty());
}

// ── JSON parsing ──

#[test]
fn node_types_valid_json_array() {
    let g = make_simple_grammar();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(v.is_array());
}

#[test]
fn node_types_all_entries_are_objects() {
    let g = make_simple_grammar();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    if let serde_json::Value::Array(arr) = v {
        for item in &arr {
            assert!(item.is_object());
        }
    }
}

#[test]
fn node_types_all_have_type() {
    let g = GrammarBuilder::new("aht")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    if let serde_json::Value::Array(arr) = v {
        for item in &arr {
            assert!(item.get("type").is_some(), "missing 'type' in {:?}", item);
        }
    }
}
