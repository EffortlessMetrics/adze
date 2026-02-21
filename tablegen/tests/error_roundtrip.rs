use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::{
    Result, TableCompressor,
    helpers::{collect_token_indices, eof_accepts_or_reduces},
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
