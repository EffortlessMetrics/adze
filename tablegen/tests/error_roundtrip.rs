use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets};
use rust_sitter_ir::builder::GrammarBuilder;
use rust_sitter_tablegen::{
    helpers::{collect_token_indices, eof_accepts_or_reduces},
    Result, TableCompressor,
};

#[test]
fn error_roundtrip_compiles_and_runs() -> Result<()> {
    // perfectly valid grammar — test is about type plumbing, not failure
    let g = GrammarBuilder::new("demo")
        .token("IDENT", r"[a-zA-Z_][a-zA-Z0-9_]*")
        .rule("module", vec![])
        .rule("module", vec!["IDENT"])
        .start("module")
        .build();

    // These return Result<_, GLRError>; using `?` here exercises your `From<GLRError> for TableGenError`
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff)?;

    // And this must still be fine
    let token_ix = collect_token_indices(&g, &pt);
    let _compressed =
        TableCompressor::new().compress(&pt, &token_ix, eof_accepts_or_reduces(&pt))?;
    Ok(())
}
