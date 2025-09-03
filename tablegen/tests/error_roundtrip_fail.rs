use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
use rust_sitter_ir::Grammar;

#[test]
fn error_roundtrip_propagates_failure() {
    // Create an invalid grammar that will fail during automaton construction
    // Empty grammar will fail when trying to build automaton
    let g = Grammar::default();

    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff);

    // Ensure this is an Err from glr-core…
    assert!(
        pt.is_err(),
        "expected automaton build to fail with empty grammar"
    );

    // …and that trying to use it in tablegen lifts to TableGenError via `From`.
    let err = (|| -> rust_sitter_tablegen::Result<()> {
        let _pt = pt?; // GLRError -> TableGenError
        Ok(())
    })()
    .unwrap_err();

    // The error should be converted to TableGenError
    let msg = err.to_string();
    // Since it's from GLRError, it should be wrapped as TableGenError::TableGeneration
    assert!(
        msg.contains("table generation") || msg.contains("grammar") || msg.contains("empty"),
        "unexpected error message: {msg}"
    );
}
