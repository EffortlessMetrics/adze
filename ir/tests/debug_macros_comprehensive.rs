//! Tests for IR debug macros (emit_ir!).

use adze_ir::builder::GrammarBuilder;
use adze_ir::emit_ir;

#[test]
fn test_emit_ir_single_arg_no_env_does_nothing() {
    let g = GrammarBuilder::new("test")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    // Should not panic even without ADZE_DEBUG_IR
    emit_ir!(g);
}

#[test]
fn test_emit_ir_two_args_no_env_does_nothing() {
    let g = GrammarBuilder::new("test")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    emit_ir!("test label", g);
}

#[test]
fn test_emit_ir_with_empty_grammar() {
    let g = GrammarBuilder::new("empty").build();
    emit_ir!(g);
    emit_ir!("empty grammar", g);
}

#[test]
fn test_emit_ir_with_complex_grammar() {
    let g = GrammarBuilder::new("complex")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("term", vec!["num"])
        .rule("term", vec!["term", "star", "num"])
        .start("expr")
        .build();
    emit_ir!(g);
    emit_ir!("complex arithmetic", g);
}

#[test]
fn test_emit_ir_with_preset_python() {
    let g = GrammarBuilder::python_like();
    emit_ir!("python preset", g);
}

#[test]
fn test_emit_ir_with_preset_javascript() {
    let g = GrammarBuilder::javascript_like();
    emit_ir!("js preset", g);
}
