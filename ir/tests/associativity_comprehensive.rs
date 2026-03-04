//! Comprehensive tests for Associativity, Precedence, PrecedenceKind,
//! and ConflictResolution types in the IR crate.

use adze_ir::*;
use std::collections::HashSet;

// ===========================================================================
// Associativity — variant construction and equality
// ===========================================================================

#[test]
fn associativity_left_variant() {
    let a = Associativity::Left;
    assert_eq!(a, Associativity::Left);
    assert_ne!(a, Associativity::Right);
    assert_ne!(a, Associativity::None);
}

#[test]
fn associativity_right_variant() {
    let a = Associativity::Right;
    assert_eq!(a, Associativity::Right);
    assert_ne!(a, Associativity::Left);
    assert_ne!(a, Associativity::None);
}

#[test]
fn associativity_none_variant() {
    let a = Associativity::None;
    assert_eq!(a, Associativity::None);
    assert_ne!(a, Associativity::Left);
    assert_ne!(a, Associativity::Right);
}

// ===========================================================================
// Associativity — Clone and Copy
// ===========================================================================

#[test]
fn associativity_clone() {
    let a = Associativity::Left;
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn associativity_copy() {
    let a = Associativity::Right;
    let b = a; // Copy
    assert_eq!(a, b); // original still usable — proves Copy
}

// ===========================================================================
// Associativity — Debug formatting
// ===========================================================================

#[test]
fn associativity_debug_left() {
    assert_eq!(format!("{:?}", Associativity::Left), "Left");
}

#[test]
fn associativity_debug_right() {
    assert_eq!(format!("{:?}", Associativity::Right), "Right");
}

#[test]
fn associativity_debug_none() {
    assert_eq!(format!("{:?}", Associativity::None), "None");
}

// ===========================================================================
// Associativity — Serde roundtrip
// ===========================================================================

#[test]
fn associativity_serde_roundtrip_all_variants() {
    for variant in [
        Associativity::Left,
        Associativity::Right,
        Associativity::None,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: Associativity = serde_json::from_str(&json).unwrap();
        assert_eq!(variant, back);
    }
}

#[test]
fn associativity_serde_json_shape() {
    // Enum variants serialize as strings by default with serde
    let json = serde_json::to_string(&Associativity::Left).unwrap();
    assert!(json.contains("Left"), "unexpected json: {json}");
}

// ===========================================================================
// PrecedenceKind — construction and equality
// ===========================================================================

#[test]
fn precedence_kind_static() {
    let p = PrecedenceKind::Static(5);
    assert_eq!(p, PrecedenceKind::Static(5));
    assert_ne!(p, PrecedenceKind::Static(6));
    assert_ne!(p, PrecedenceKind::Dynamic(5));
}

#[test]
fn precedence_kind_dynamic() {
    let p = PrecedenceKind::Dynamic(10);
    assert_eq!(p, PrecedenceKind::Dynamic(10));
    assert_ne!(p, PrecedenceKind::Dynamic(11));
    assert_ne!(p, PrecedenceKind::Static(10));
}

#[test]
fn precedence_kind_zero() {
    let s = PrecedenceKind::Static(0);
    let d = PrecedenceKind::Dynamic(0);
    assert_eq!(s, PrecedenceKind::Static(0));
    assert_eq!(d, PrecedenceKind::Dynamic(0));
    assert_ne!(s, d);
}

#[test]
fn precedence_kind_negative() {
    let s = PrecedenceKind::Static(-1);
    let d = PrecedenceKind::Dynamic(-100);
    assert_eq!(s, PrecedenceKind::Static(-1));
    assert_eq!(d, PrecedenceKind::Dynamic(-100));
}

#[test]
fn precedence_kind_i16_extremes() {
    let min_s = PrecedenceKind::Static(i16::MIN);
    let max_s = PrecedenceKind::Static(i16::MAX);
    let min_d = PrecedenceKind::Dynamic(i16::MIN);
    let max_d = PrecedenceKind::Dynamic(i16::MAX);

    assert_ne!(min_s, max_s);
    assert_ne!(min_d, max_d);
    assert_ne!(min_s, min_d);
    assert_ne!(max_s, max_d);
}

// ===========================================================================
// PrecedenceKind — Clone and Copy
// ===========================================================================

#[test]
fn precedence_kind_clone() {
    let p = PrecedenceKind::Static(3);
    let q = p;
    assert_eq!(p, q);
}

#[test]
fn precedence_kind_copy() {
    let p = PrecedenceKind::Dynamic(7);
    let q = p; // Copy
    assert_eq!(p, q); // original still usable
}

// ===========================================================================
// PrecedenceKind — Debug formatting
// ===========================================================================

#[test]
fn precedence_kind_debug() {
    assert_eq!(format!("{:?}", PrecedenceKind::Static(5)), "Static(5)");
    assert_eq!(format!("{:?}", PrecedenceKind::Dynamic(-3)), "Dynamic(-3)");
}

// ===========================================================================
// PrecedenceKind — Serde roundtrip
// ===========================================================================

#[test]
fn precedence_kind_serde_roundtrip() {
    let cases = [
        PrecedenceKind::Static(0),
        PrecedenceKind::Static(i16::MAX),
        PrecedenceKind::Static(i16::MIN),
        PrecedenceKind::Dynamic(42),
        PrecedenceKind::Dynamic(-1),
    ];
    for pk in cases {
        let json = serde_json::to_string(&pk).unwrap();
        let back: PrecedenceKind = serde_json::from_str(&json).unwrap();
        assert_eq!(pk, back, "roundtrip failed for {pk:?}");
    }
}

// ===========================================================================
// Precedence struct — construction
// ===========================================================================

#[test]
fn precedence_struct_basic() {
    let p = Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(10), SymbolId(20)],
    };
    assert_eq!(p.level, 1);
    assert_eq!(p.associativity, Associativity::Left);
    assert_eq!(p.symbols.len(), 2);
}

#[test]
fn precedence_struct_empty_symbols() {
    let p = Precedence {
        level: 0,
        associativity: Associativity::None,
        symbols: vec![],
    };
    assert!(p.symbols.is_empty());
}

#[test]
fn precedence_struct_negative_level() {
    let p = Precedence {
        level: -5,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(1)],
    };
    assert_eq!(p.level, -5);
}

#[test]
fn precedence_struct_extreme_levels() {
    let p_min = Precedence {
        level: i16::MIN,
        associativity: Associativity::Left,
        symbols: vec![],
    };
    let p_max = Precedence {
        level: i16::MAX,
        associativity: Associativity::Right,
        symbols: vec![],
    };
    assert_eq!(p_min.level, i16::MIN);
    assert_eq!(p_max.level, i16::MAX);
}

// ===========================================================================
// Precedence struct — Clone and Debug
// ===========================================================================

#[test]
fn precedence_struct_clone() {
    let p = Precedence {
        level: 3,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(1)],
    };
    let q = p.clone();
    assert_eq!(q.level, 3);
    assert_eq!(q.associativity, Associativity::Right);
    assert_eq!(q.symbols, vec![SymbolId(1)]);
}

#[test]
fn precedence_struct_debug() {
    let p = Precedence {
        level: 2,
        associativity: Associativity::None,
        symbols: vec![],
    };
    let dbg = format!("{p:?}");
    assert!(dbg.contains("Precedence"));
    assert!(dbg.contains("2"));
    assert!(dbg.contains("None"));
}

// ===========================================================================
// Precedence struct — Serde roundtrip
// ===========================================================================

#[test]
fn precedence_struct_serde_roundtrip() {
    let p = Precedence {
        level: 7,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(100), SymbolId(200)],
    };
    let json = serde_json::to_string(&p).unwrap();
    let back: Precedence = serde_json::from_str(&json).unwrap();
    assert_eq!(back.level, 7);
    assert_eq!(back.associativity, Associativity::Left);
    assert_eq!(back.symbols, vec![SymbolId(100), SymbolId(200)]);
}

// ===========================================================================
// ConflictResolution — variants and equality
// ===========================================================================

#[test]
fn conflict_resolution_precedence_variant() {
    let cr = ConflictResolution::Precedence(PrecedenceKind::Static(5));
    assert_eq!(
        cr,
        ConflictResolution::Precedence(PrecedenceKind::Static(5))
    );
    assert_ne!(
        cr,
        ConflictResolution::Precedence(PrecedenceKind::Dynamic(5))
    );
}

#[test]
fn conflict_resolution_associativity_variant() {
    let cr = ConflictResolution::Associativity(Associativity::Left);
    assert_eq!(cr, ConflictResolution::Associativity(Associativity::Left));
    assert_ne!(cr, ConflictResolution::Associativity(Associativity::Right));
}

#[test]
fn conflict_resolution_glr_variant() {
    let cr = ConflictResolution::GLR;
    assert_eq!(cr, ConflictResolution::GLR);
    assert_ne!(
        cr,
        ConflictResolution::Precedence(PrecedenceKind::Static(0))
    );
}

#[test]
fn conflict_resolution_all_distinct() {
    let a = ConflictResolution::Precedence(PrecedenceKind::Static(1));
    let b = ConflictResolution::Associativity(Associativity::Left);
    let c = ConflictResolution::GLR;
    assert_ne!(a, b);
    assert_ne!(b, c);
    assert_ne!(a, c);
}

// ===========================================================================
// ConflictResolution — Serde roundtrip
// ===========================================================================

#[test]
fn conflict_resolution_serde_roundtrip() {
    let cases = [
        ConflictResolution::Precedence(PrecedenceKind::Static(0)),
        ConflictResolution::Precedence(PrecedenceKind::Dynamic(-10)),
        ConflictResolution::Associativity(Associativity::Left),
        ConflictResolution::Associativity(Associativity::Right),
        ConflictResolution::Associativity(Associativity::None),
        ConflictResolution::GLR,
    ];
    for cr in &cases {
        let json = serde_json::to_string(cr).unwrap();
        let back: ConflictResolution = serde_json::from_str(&json).unwrap();
        assert_eq!(*cr, back, "roundtrip failed for {cr:?}");
    }
}

// ===========================================================================
// Associativity in HashSet (via manual Hash-like usage)
// ===========================================================================

#[test]
fn associativity_all_variants_distinguishable_in_collection() {
    // Even though Associativity doesn't derive Hash, we can verify
    // distinctness via Debug strings as a proxy.
    let variants = [
        Associativity::Left,
        Associativity::Right,
        Associativity::None,
    ];
    let debug_strings: HashSet<String> = variants.iter().map(|v| format!("{v:?}")).collect();
    assert_eq!(debug_strings.len(), 3);
}

// ===========================================================================
// Rule with precedence and associativity
// ===========================================================================

#[test]
fn rule_with_all_precedence_associativity_combos() {
    let assocs = [
        Associativity::Left,
        Associativity::Right,
        Associativity::None,
    ];
    let precs = [
        PrecedenceKind::Static(0),
        PrecedenceKind::Static(i16::MAX),
        PrecedenceKind::Dynamic(-1),
    ];

    for assoc in &assocs {
        for prec in &precs {
            let rule = Rule {
                lhs: SymbolId(1),
                rhs: vec![Symbol::Terminal(SymbolId(2))],
                precedence: Some(*prec),
                associativity: Some(*assoc),
                fields: Vec::new(),
                production_id: ProductionId(0),
            };
            assert_eq!(rule.precedence.unwrap(), *prec);
            assert_eq!(rule.associativity.unwrap(), *assoc);
        }
    }
}

// ===========================================================================
// Pattern matching exhaustiveness
// ===========================================================================

#[test]
fn precedence_kind_match_exhaustive() {
    let cases = [PrecedenceKind::Static(1), PrecedenceKind::Dynamic(2)];
    for pk in cases {
        match pk {
            PrecedenceKind::Static(v) => assert_eq!(v, 1),
            PrecedenceKind::Dynamic(v) => assert_eq!(v, 2),
        }
    }
}

#[test]
fn associativity_match_exhaustive() {
    let cases = [
        Associativity::Left,
        Associativity::Right,
        Associativity::None,
    ];
    for a in cases {
        match a {
            Associativity::Left => {}
            Associativity::Right => {}
            Associativity::None => {}
        }
    }
}

// ===========================================================================
// Serde deserialization from known JSON
// ===========================================================================

#[test]
fn associativity_deserialize_from_known_json() {
    let left: Associativity = serde_json::from_str(r#""Left""#).unwrap();
    let right: Associativity = serde_json::from_str(r#""Right""#).unwrap();
    let none: Associativity = serde_json::from_str(r#""None""#).unwrap();
    assert_eq!(left, Associativity::Left);
    assert_eq!(right, Associativity::Right);
    assert_eq!(none, Associativity::None);
}

#[test]
fn precedence_kind_deserialize_from_known_json() {
    let s: PrecedenceKind = serde_json::from_str(r#"{"Static":42}"#).unwrap();
    let d: PrecedenceKind = serde_json::from_str(r#"{"Dynamic":-7}"#).unwrap();
    assert_eq!(s, PrecedenceKind::Static(42));
    assert_eq!(d, PrecedenceKind::Dynamic(-7));
}

#[test]
fn associativity_deserialize_invalid_rejects() {
    let result = serde_json::from_str::<Associativity>(r#""Unknown""#);
    assert!(result.is_err());
}
