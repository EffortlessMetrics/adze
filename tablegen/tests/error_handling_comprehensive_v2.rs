//! Comprehensive tests for tablegen error handling and generator API.

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::{StaticLanguageGenerator, TableCompressor, TableGenError};

fn make_gen(name: &str) -> StaticLanguageGenerator {
    let g = GrammarBuilder::new(name)
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    StaticLanguageGenerator::new(g, pt)
}

// ── Error Construction ──

#[test]
fn error_from_str() {
    let err: TableGenError = "test error".into();
    let msg = format!("{}", err);
    assert!(msg.contains("test error"));
}

#[test]
fn error_from_string() {
    let err: TableGenError = String::from("owned").into();
    assert!(!format!("{}", err).is_empty());
}

#[test]
fn error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
    let err: TableGenError = io_err.into();
    assert!(!format!("{}", err).is_empty());
}

#[test]
fn error_debug() {
    let err: TableGenError = "dbg".into();
    assert!(!format!("{:?}", err).is_empty());
}

#[test]
fn error_display() {
    let err: TableGenError = "disp".into();
    assert!(!format!("{}", err).is_empty());
}

#[test]
fn error_is_std_error() {
    fn check<T: std::error::Error>() {}
    check::<TableGenError>();
}

#[test]
fn error_is_send() {
    fn check<T: Send>() {}
    check::<TableGenError>();
}

#[test]
fn error_is_sync() {
    fn check<T: Sync>() {}
    check::<TableGenError>();
}

#[test]
fn error_empty_message() {
    let err: TableGenError = "".into();
    let _ = format!("{}", err);
}

#[test]
fn error_long_message() {
    let msg = "x".repeat(10000);
    let err: TableGenError = msg.into();
    let _ = format!("{}", err);
}

#[test]
fn error_unicode() {
    let err: TableGenError = "错误 🚀".into();
    let d = format!("{}", err);
    assert!(d.contains("🚀"));
}

#[test]
fn io_error_kinds() {
    for kind in [
        std::io::ErrorKind::NotFound,
        std::io::ErrorKind::PermissionDenied,
        std::io::ErrorKind::AlreadyExists,
        std::io::ErrorKind::InvalidInput,
        std::io::ErrorKind::Other,
    ] {
        let e: TableGenError = std::io::Error::new(kind, "t").into();
        let _ = format!("{}", e);
    }
}

// ── StaticLanguageGenerator ──

#[test]
fn gen_new() {
    let _ = make_gen("gn");
}

#[test]
fn gen_debug() {
    let g = make_gen("gd");
    assert!(!g.generate_node_types().is_empty());
}

#[test]
fn gen_language_code() {
    let g = make_gen("gc");
    let code = g.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn gen_node_types() {
    let g = make_gen("gnt");
    let json = g.generate_node_types();
    assert!(!json.is_empty());
}

#[test]
fn gen_code_deterministic() {
    let a = make_gen("det").generate_language_code().to_string();
    let b = make_gen("det").generate_language_code().to_string();
    assert_eq!(a, b);
}

#[test]
fn gen_node_types_deterministic() {
    let a = make_gen("nd").generate_node_types();
    let b = make_gen("nd").generate_node_types();
    assert_eq!(a, b);
}

#[test]
fn gen_node_types_valid_json() {
    let json = make_gen("vj").generate_node_types();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.is_array());
}

#[test]
fn gen_node_types_entries_have_type() {
    let json = make_gen("eht").generate_node_types();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(entry.get("type").is_some());
    }
}

#[test]
fn gen_node_types_entries_have_named() {
    let json = make_gen("ehn").generate_node_types();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(entry.get("named").is_some());
    }
}

// ── TableCompressor ──

#[test]
fn compressor_new() {
    let _ = TableCompressor::new();
}

#[test]
fn compressor_default() {
    let _ = TableCompressor::default();
}

// ── Multi-token grammars ──

#[test]
fn gen_multi_token() {
    let g = GrammarBuilder::new("mt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.generate_language_code().is_empty());
}

#[test]
fn gen_multi_alternative() {
    let g = GrammarBuilder::new("ma")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    let slg = StaticLanguageGenerator::new(g, pt);
    let json = slg.generate_node_types();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.is_array());
}

// ── Precedence grammars ──

#[test]
fn gen_precedence() {
    use adze_ir::Associativity;
    let g = GrammarBuilder::new("prec")
        .token("n", "n")
        .token("p", "+")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "p", "e"], 1, Associativity::Left)
        .start("e")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    let slg = StaticLanguageGenerator::new(g, pt);
    let code = slg.generate_language_code().to_string();
    assert!(code.len() > 100);
}

// ── Language code size ──

#[test]
fn gen_code_size_increases() {
    let small = make_gen("s1").generate_language_code().to_string().len();

    let g = GrammarBuilder::new("bigger")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    let big = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string()
        .len();

    assert!(big >= small);
}
