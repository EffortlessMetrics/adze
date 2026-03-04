//! Tests for error type formatting (Display impls).

use adze::error_reporting::ErrorReporter;

#[test]
fn error_reporter_empty_source() {
    let _reporter = ErrorReporter::new(String::new());
    // Verify construction doesn't panic
}

#[test]
fn error_reporter_with_input() {
    let _reporter = ErrorReporter::new("let x = 1".to_string());
    // ErrorReporter doesn't implement Debug, just verify construction works
}

#[test]
fn span_error_display() {
    let err = adze::SpanError {
        span: (5, 0),
        source_len: 10,
        reason: adze::SpanErrorReason::StartGreaterThanEnd,
    };
    let s = format!("{err}");
    assert!(!s.is_empty());
}

#[test]
fn span_error_reason_variants() {
    let variants = [
        adze::SpanErrorReason::StartGreaterThanEnd,
        adze::SpanErrorReason::StartOutOfBounds,
        adze::SpanErrorReason::EndOutOfBounds,
    ];
    for v in &variants {
        let _ = format!("{v:?}");
    }
}

#[test]
fn span_error_start_out_of_bounds() {
    let err = adze::SpanError {
        span: (100, 200),
        source_len: 10,
        reason: adze::SpanErrorReason::StartOutOfBounds,
    };
    let s = format!("{err}");
    assert!(s.contains("10") || !s.is_empty());
}

#[test]
fn span_error_end_out_of_bounds() {
    let err = adze::SpanError {
        span: (0, 200),
        source_len: 10,
        reason: adze::SpanErrorReason::EndOutOfBounds,
    };
    let s = format!("{err}");
    assert!(!s.is_empty());
}
