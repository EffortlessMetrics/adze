use adze_ir::error::IrError;
use adze_tablegen::error::TableGenError;

#[test]
fn ir_error_preserves_source_chain() {
    let ir_err = IrError::InvalidSymbol("oops".into());
    let tablegen_err: TableGenError = ir_err.into();

    // Check that we can access the error through the transparent wrapper
    match tablegen_err {
        TableGenError::Ir(e) => {
            let error_msg = e.to_string().to_lowercase();
            assert!(
                error_msg.contains("invalid") && error_msg.contains("symbol"),
                "Expected error message to contain 'invalid' and 'symbol', got: {}",
                error_msg
            );
        }
        _ => panic!("Expected Ir variant"),
    }
}

#[test]
fn ir_error_source_can_be_traversed() {
    let ir_err = IrError::InvalidSymbol("test_symbol".into());
    let tablegen_err: TableGenError = ir_err.into();

    // Verify that the error chain can be traversed using std::error::Error trait
    let error_chain: Vec<String> =
        std::iter::successors(Some(&tablegen_err as &dyn std::error::Error), |e| {
            e.source()
        })
        .map(|e| e.to_string())
        .collect();

    // Should have at least one error in the chain
    assert!(!error_chain.is_empty());

    // First error should be the TableGenError display
    let first_error = &error_chain[0].to_lowercase();
    assert!(first_error.contains("invalid") || first_error.contains("symbol"));
}
