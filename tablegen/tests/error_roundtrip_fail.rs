use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::Grammar;

#[test]
#[ignore]
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
    let err = (|| -> adze_tablegen::Result<()> {
        let _pt = pt?; // GLRError -> TableGenError
        Ok(())
    })()
    .unwrap_err();

    // The error should be converted to TableGenError
    let msg = err.to_string();
    let msg_lower = msg.to_lowercase();
    // Since it's from GLRError, it should be wrapped as TableGenError::TableGeneration
    assert!(
        msg_lower.contains("table generation")
            || msg_lower.contains("grammar")
            || msg_lower.contains("empty"),
        "unexpected error message: {msg}"
    );
}
