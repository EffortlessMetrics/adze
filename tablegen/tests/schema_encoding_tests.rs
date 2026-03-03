//! Comprehensive tests for schema module action encoding/decoding.
//!
//! These tests cover edge cases, boundary values, and invalid encodings
//! that complement the unit tests in `schema.rs`.

use adze_glr_core::{Action, StateId};
use adze_ir::RuleId;
use adze_tablegen::schema::{SchemaError, validate_action_decoding, validate_action_encoding};

// ── Shift edge cases ──────────────────────────────────────────────────

#[test]
fn shift_state_1_encodes_to_0x0001() {
    let encoded = validate_action_encoding(&Action::Shift(StateId(1))).unwrap();
    assert_eq!(encoded, 0x0001);
    validate_action_decoding(encoded, &Action::Shift(StateId(1))).unwrap();
}

#[test]
fn shift_max_valid_state_encodes_to_0x7fff() {
    let encoded = validate_action_encoding(&Action::Shift(StateId(0x7FFF))).unwrap();
    assert_eq!(encoded, 0x7FFF);
    validate_action_decoding(encoded, &Action::Shift(StateId(0x7FFF))).unwrap();
}

#[test]
fn shift_0x8000_rejected_conflicts_with_reduce_range() {
    let result = validate_action_encoding(&Action::Shift(StateId(0x8000)));
    assert!(result.is_err());
    match result.unwrap_err() {
        SchemaError::InvalidActionEncoding { action, reason, .. } => {
            assert_eq!(action, Action::Shift(StateId(0x8000)));
            assert!(
                reason.contains("high bit"),
                "reason should mention high bit: {reason}"
            );
        }
        other => panic!("expected InvalidActionEncoding, got: {other:?}"),
    }
}

#[test]
fn shift_0xffff_rejected() {
    // 0xFFFF is reserved for Accept; Shift(0xFFFF) must be rejected.
    let result = validate_action_encoding(&Action::Shift(StateId(0xFFFF)));
    assert!(result.is_err());
}

#[test]
fn shift_boundary_just_below_reduce_range() {
    // 0x7FFE is one below the max valid shift state – must succeed.
    let encoded = validate_action_encoding(&Action::Shift(StateId(0x7FFE))).unwrap();
    assert_eq!(encoded, 0x7FFE);
    validate_action_decoding(encoded, &Action::Shift(StateId(0x7FFE))).unwrap();
}

// ── Reduce edge cases ────────────────────────────────────────────────

#[test]
fn reduce_rule_0_encodes_to_0x8000() {
    let encoded = validate_action_encoding(&Action::Reduce(RuleId(0))).unwrap();
    assert_eq!(encoded, 0x8000);
    validate_action_decoding(encoded, &Action::Reduce(RuleId(0))).unwrap();
}

#[test]
fn reduce_max_valid_rule_encodes_to_0xfffe() {
    // 0x7FFE is the largest valid rule ID (0x7FFF would collide with Accept).
    let encoded = validate_action_encoding(&Action::Reduce(RuleId(0x7FFE))).unwrap();
    assert_eq!(encoded, 0xFFFE);
    validate_action_decoding(encoded, &Action::Reduce(RuleId(0x7FFE))).unwrap();
}

#[test]
fn reduce_0x7fff_rejected_collides_with_accept() {
    let result = validate_action_encoding(&Action::Reduce(RuleId(0x7FFF)));
    assert!(result.is_err());
    match result.unwrap_err() {
        SchemaError::InvalidActionEncoding { action, reason, .. } => {
            assert_eq!(action, Action::Reduce(RuleId(0x7FFF)));
            assert!(
                reason.contains("Accept"),
                "reason should mention Accept: {reason}"
            );
        }
        other => panic!("expected InvalidActionEncoding, got: {other:?}"),
    }
}

#[test]
fn reduce_large_rule_id_just_below_overflow() {
    // Rule ID 0x7FFD – two below the overflow boundary.
    let encoded = validate_action_encoding(&Action::Reduce(RuleId(0x7FFD))).unwrap();
    assert_eq!(encoded, 0x8000 | 0x7FFD);
    validate_action_decoding(encoded, &Action::Reduce(RuleId(0x7FFD))).unwrap();
}

// ── Error and Accept fundamentals ────────────────────────────────────

#[test]
fn error_and_accept_are_distinct_encodings() {
    let error_enc = validate_action_encoding(&Action::Error).unwrap();
    let accept_enc = validate_action_encoding(&Action::Accept).unwrap();
    assert_ne!(error_enc, accept_enc);
    assert_eq!(error_enc, 0x0000);
    assert_eq!(accept_enc, 0xFFFF);
}

// ── Roundtrip exhaustive over all action types ───────────────────────

#[test]
fn roundtrip_all_encodable_action_types() {
    let cases: Vec<Action> = vec![
        Action::Error,
        Action::Accept,
        Action::Shift(StateId(1)),
        Action::Shift(StateId(42)),
        Action::Shift(StateId(0x7FFF)),
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(1)),
        Action::Reduce(RuleId(500)),
        Action::Reduce(RuleId(0x7FFE)),
    ];
    for action in &cases {
        let encoded = validate_action_encoding(action)
            .unwrap_or_else(|e| panic!("encoding failed for {action:?}: {e}"));
        validate_action_decoding(encoded, action)
            .unwrap_or_else(|e| panic!("roundtrip failed for {action:?} (0x{encoded:04X}): {e}"));
    }
}

// ── Recover and Fork are runtime-only (not encodable) ────────────────

#[test]
fn recover_action_rejected() {
    let result = validate_action_encoding(&Action::Recover);
    assert!(result.is_err());
    match result.unwrap_err() {
        SchemaError::InvalidActionEncoding { reason, .. } => {
            assert!(
                reason.contains("runtime"),
                "reason should mention runtime: {reason}"
            );
        }
        other => panic!("expected InvalidActionEncoding, got: {other:?}"),
    }
}

#[test]
fn fork_action_rejected() {
    let inner = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
    let result = validate_action_encoding(&Action::Fork(inner));
    assert!(result.is_err());
    match result.unwrap_err() {
        SchemaError::InvalidActionEncoding { reason, .. } => {
            assert!(
                reason.contains("runtime"),
                "reason should mention runtime: {reason}"
            );
        }
        other => panic!("expected InvalidActionEncoding, got: {other:?}"),
    }
}

// ── Decoding corrupt/invalid raw u16 values ──────────────────────────

#[test]
fn decoding_wrong_expected_action_is_error() {
    // 0x0001 decodes to Shift(1), not Reduce(1).
    let result = validate_action_decoding(0x0001, &Action::Reduce(RuleId(1)));
    assert!(result.is_err());
}

#[test]
fn decoding_accept_encoding_as_reduce_is_error() {
    // 0xFFFF is Accept, not Reduce(0x7FFF).
    let result = validate_action_decoding(0xFFFF, &Action::Reduce(RuleId(0x7FFF)));
    assert!(result.is_err());
}

#[test]
fn decoding_error_encoding_as_shift_is_error() {
    // 0x0000 is Error, not Shift(0).
    let result = validate_action_decoding(0x0000, &Action::Shift(StateId(0)));
    assert!(result.is_err());
}

// ── Shift and Reduce ranges never overlap ────────────────────────────

#[test]
fn shift_and_reduce_ranges_are_disjoint() {
    // Every valid shift encoding is in [0x0001, 0x7FFF].
    // Every valid reduce encoding is in [0x8000, 0xFFFE].
    // Verify boundary values don't collide.
    let shift_max = validate_action_encoding(&Action::Shift(StateId(0x7FFF))).unwrap();
    let reduce_min = validate_action_encoding(&Action::Reduce(RuleId(0))).unwrap();
    assert!(
        shift_max < reduce_min,
        "shift max 0x{shift_max:04X} should be < reduce min 0x{reduce_min:04X}"
    );
    assert_eq!(reduce_min - shift_max, 1, "ranges should be contiguous");
}

// ── SchemaError Display ──────────────────────────────────────────────

#[test]
fn schema_error_display_includes_hex_encoding() {
    let err = SchemaError::InvalidActionEncoding {
        action: Action::Shift(StateId(0x8000)),
        encoded_value: 0x8000,
        reason: "test reason".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("0x8000"), "display should contain hex: {msg}");
    assert!(
        msg.contains("test reason"),
        "display should contain reason: {msg}"
    );
}

// ── Multiple sequential roundtrips ───────────────────────────────────

#[test]
fn multiple_sequential_roundtrips_are_stable() {
    // Encode → decode-validate → re-encode → decode-validate for several actions.
    let actions = [
        Action::Shift(StateId(2)),
        Action::Reduce(RuleId(10)),
        Action::Accept,
        Action::Error,
    ];
    for action in &actions {
        let enc1 = validate_action_encoding(action).unwrap();
        validate_action_decoding(enc1, action).unwrap();
        // Re-encode: validate_action_encoding on the same action should be deterministic.
        let enc2 = validate_action_encoding(action).unwrap();
        assert_eq!(
            enc1, enc2,
            "encoding should be deterministic for {action:?}"
        );
    }
}
