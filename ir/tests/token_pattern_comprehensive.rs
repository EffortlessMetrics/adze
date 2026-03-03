//! Comprehensive tests for TokenPattern, Token, ExternalToken, and newtype ID wrappers.

use adze_ir::{
    ExternalToken, FieldId, ProductionId, RuleId, StateId, SymbolId, Token, TokenPattern,
};

// ---------------------------------------------------------------------------
// TokenPattern variants
// ---------------------------------------------------------------------------

#[test]
fn token_pattern_string_literal() {
    let pat = TokenPattern::String("hello".into());
    match &pat {
        TokenPattern::String(s) => assert_eq!(s, "hello"),
        _ => panic!("expected String variant"),
    }
}

#[test]
fn token_pattern_regex() {
    let pat = TokenPattern::Regex(r"\d+".into());
    match &pat {
        TokenPattern::Regex(r) => assert_eq!(r, r"\d+"),
        _ => panic!("expected Regex variant"),
    }
}

#[test]
fn token_pattern_empty_string() {
    let pat = TokenPattern::String(String::new());
    assert_eq!(pat, TokenPattern::String(String::new()));
}

#[test]
fn token_pattern_empty_regex() {
    let pat = TokenPattern::Regex(String::new());
    assert_eq!(pat, TokenPattern::Regex(String::new()));
}

#[test]
fn token_pattern_string_with_special_chars() {
    let pat = TokenPattern::String("foo\nbar\t\"baz\"".into());
    if let TokenPattern::String(s) = &pat {
        assert!(s.contains('\n'));
        assert!(s.contains('"'));
    } else {
        panic!("expected String");
    }
}

#[test]
fn token_pattern_regex_complex() {
    let pat = TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".into());
    if let TokenPattern::Regex(r) = &pat {
        assert!(r.starts_with('['));
    } else {
        panic!("expected Regex");
    }
}

#[test]
fn token_pattern_equality() {
    let a = TokenPattern::String("x".into());
    let b = TokenPattern::String("x".into());
    let c = TokenPattern::String("y".into());
    let d = TokenPattern::Regex("x".into());

    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_ne!(a, d); // same inner string but different variant
}

#[test]
fn token_pattern_clone() {
    let original = TokenPattern::Regex(r"\w+".into());
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn token_pattern_debug() {
    let pat = TokenPattern::String("hi".into());
    let dbg = format!("{pat:?}");
    assert!(dbg.contains("String"));
    assert!(dbg.contains("hi"));
}

// ---------------------------------------------------------------------------
// Token struct
// ---------------------------------------------------------------------------

#[test]
fn token_construction_string() {
    let tok = Token {
        name: "PLUS".into(),
        pattern: TokenPattern::String("+".into()),
        fragile: false,
    };
    assert_eq!(tok.name, "PLUS");
    assert!(!tok.fragile);
}

#[test]
fn token_construction_regex() {
    let tok = Token {
        name: "NUMBER".into(),
        pattern: TokenPattern::Regex(r"\d+".into()),
        fragile: true,
    };
    assert!(tok.fragile);
    assert!(matches!(tok.pattern, TokenPattern::Regex(_)));
}

#[test]
fn token_equality() {
    let a = Token {
        name: "A".into(),
        pattern: TokenPattern::String("a".into()),
        fragile: false,
    };
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn token_inequality_name() {
    let a = Token {
        name: "A".into(),
        pattern: TokenPattern::String("x".into()),
        fragile: false,
    };
    let b = Token {
        name: "B".into(),
        pattern: TokenPattern::String("x".into()),
        fragile: false,
    };
    assert_ne!(a, b);
}

#[test]
fn token_inequality_fragile() {
    let a = Token {
        name: "A".into(),
        pattern: TokenPattern::String("x".into()),
        fragile: false,
    };
    let b = Token {
        name: "A".into(),
        pattern: TokenPattern::String("x".into()),
        fragile: true,
    };
    assert_ne!(a, b);
}

#[test]
fn token_debug_format() {
    let tok = Token {
        name: "ID".into(),
        pattern: TokenPattern::Regex(r"\w+".into()),
        fragile: false,
    };
    let dbg = format!("{tok:?}");
    assert!(dbg.contains("ID"));
    assert!(dbg.contains("Regex"));
}

// ---------------------------------------------------------------------------
// ExternalToken
// ---------------------------------------------------------------------------

#[test]
fn external_token_construction() {
    let et = ExternalToken {
        name: "INDENT".into(),
        symbol_id: SymbolId(42),
    };
    assert_eq!(et.name, "INDENT");
    assert_eq!(et.symbol_id, SymbolId(42));
}

#[test]
fn external_token_clone_debug() {
    let et = ExternalToken {
        name: "DEDENT".into(),
        symbol_id: SymbolId(7),
    };
    let cloned = et.clone();
    assert_eq!(format!("{et:?}"), format!("{cloned:?}"));
}

// ---------------------------------------------------------------------------
// Newtype ID wrappers
// ---------------------------------------------------------------------------

#[test]
fn symbol_id_basic() {
    let id = SymbolId(0);
    assert_eq!(id.0, 0);
    assert_eq!(id, SymbolId(0));
    assert_ne!(id, SymbolId(1));
}

#[test]
fn rule_id_basic() {
    let id = RuleId(10);
    assert_eq!(id.0, 10);
    assert_eq!(id, RuleId(10));
}

#[test]
fn state_id_basic() {
    let id = StateId(255);
    assert_eq!(id.0, 255);
}

#[test]
fn field_id_basic() {
    let id = FieldId(3);
    assert_eq!(id.0, 3);
    assert_eq!(id, FieldId(3));
}

#[test]
fn production_id_basic() {
    let id = ProductionId(99);
    assert_eq!(id.0, 99);
}

#[test]
fn id_max_values() {
    assert_eq!(SymbolId(u16::MAX).0, u16::MAX);
    assert_eq!(RuleId(u16::MAX).0, u16::MAX);
    assert_eq!(StateId(u16::MAX).0, u16::MAX);
    assert_eq!(FieldId(u16::MAX).0, u16::MAX);
    assert_eq!(ProductionId(u16::MAX).0, u16::MAX);
}

#[test]
fn id_ordering() {
    assert!(SymbolId(1) < SymbolId(2));
    assert!(RuleId(0) < RuleId(1));
    assert!(StateId(5) > StateId(3));
    assert!(ProductionId(10) >= ProductionId(10));
}

#[test]
fn id_hash_set() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(SymbolId(1));
    set.insert(SymbolId(2));
    set.insert(SymbolId(1)); // duplicate
    assert_eq!(set.len(), 2);
}

#[test]
fn id_display() {
    assert_eq!(format!("{}", SymbolId(5)), "Symbol(5)");
    assert_eq!(format!("{}", RuleId(3)), "Rule(3)");
    assert_eq!(format!("{}", StateId(7)), "State(7)");
    assert_eq!(format!("{}", FieldId(1)), "Field(1)");
    assert_eq!(format!("{}", ProductionId(0)), "Production(0)");
}

#[test]
fn id_debug() {
    let dbg = format!("{:?}", SymbolId(42));
    assert!(dbg.contains("SymbolId"));
    assert!(dbg.contains("42"));
}

// ---------------------------------------------------------------------------
// Serde roundtrip
// ---------------------------------------------------------------------------

#[test]
fn serde_token_pattern_string_roundtrip() {
    let pat = TokenPattern::String("if".into());
    let json = serde_json::to_string(&pat).unwrap();
    let back: TokenPattern = serde_json::from_str(&json).unwrap();
    assert_eq!(pat, back);
}

#[test]
fn serde_token_pattern_regex_roundtrip() {
    let pat = TokenPattern::Regex(r"[0-9]+(\.[0-9]+)?".into());
    let json = serde_json::to_string(&pat).unwrap();
    let back: TokenPattern = serde_json::from_str(&json).unwrap();
    assert_eq!(pat, back);
}

#[test]
fn serde_token_roundtrip() {
    let tok = Token {
        name: "SEMICOLON".into(),
        pattern: TokenPattern::String(";".into()),
        fragile: true,
    };
    let json = serde_json::to_string(&tok).unwrap();
    let back: Token = serde_json::from_str(&json).unwrap();
    assert_eq!(tok, back);
}

#[test]
fn serde_external_token_roundtrip() {
    let et = ExternalToken {
        name: "NEWLINE".into(),
        symbol_id: SymbolId(100),
    };
    let json = serde_json::to_string(&et).unwrap();
    let back: ExternalToken = serde_json::from_str(&json).unwrap();
    assert_eq!(format!("{:?}", et), format!("{:?}", back));
}

#[test]
fn serde_id_roundtrip() {
    let ids = (
        SymbolId(1),
        RuleId(2),
        StateId(3),
        FieldId(4),
        ProductionId(5),
    );
    let json = serde_json::to_string(&ids).unwrap();
    let back: (SymbolId, RuleId, StateId, FieldId, ProductionId) =
        serde_json::from_str(&json).unwrap();
    assert_eq!(ids, back);
}

#[test]
fn serde_token_pattern_json_shape() {
    let pat = TokenPattern::String("return".into());
    let json = serde_json::to_string(&pat).unwrap();
    // Ensure the JSON representation distinguishes variants
    let val: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(
        val.is_object() || val.is_string(),
        "unexpected JSON shape: {val}"
    );
}
