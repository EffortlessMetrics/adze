//! Comprehensive tests for adze_ir Symbol enum and related types.

use adze_ir::{Symbol, SymbolId};

// ── Symbol::Terminal ──

#[test]
fn symbol_terminal() {
    let s = Symbol::Terminal(SymbolId(1));
    assert!(matches!(s, Symbol::Terminal(_)));
}

#[test]
fn symbol_terminal_id() {
    let s = Symbol::Terminal(SymbolId(42));
    if let Symbol::Terminal(id) = s {
        assert_eq!(id, SymbolId(42));
    } else {
        panic!("expected Terminal");
    }
}

// ── Symbol::NonTerminal ──

#[test]
fn symbol_nonterminal() {
    let s = Symbol::NonTerminal(SymbolId(1));
    assert!(matches!(s, Symbol::NonTerminal(_)));
}

#[test]
fn symbol_nonterminal_id() {
    let s = Symbol::NonTerminal(SymbolId(99));
    if let Symbol::NonTerminal(id) = s {
        assert_eq!(id, SymbolId(99));
    } else {
        panic!("expected NonTerminal");
    }
}

// ── Symbol Clone ──

#[test]
fn symbol_clone_terminal() {
    let s = Symbol::Terminal(SymbolId(5));
    let c = s.clone();
    assert_eq!(format!("{:?}", s), format!("{:?}", c));
}

#[test]
fn symbol_clone_nonterminal() {
    let s = Symbol::NonTerminal(SymbolId(10));
    let c = s.clone();
    assert_eq!(format!("{:?}", s), format!("{:?}", c));
}

// ── Symbol Debug ──

#[test]
fn symbol_debug_terminal() {
    let d = format!("{:?}", Symbol::Terminal(SymbolId(1)));
    assert!(!d.is_empty());
}

#[test]
fn symbol_debug_nonterminal() {
    let d = format!("{:?}", Symbol::NonTerminal(SymbolId(1)));
    assert!(!d.is_empty());
}

// ── Symbol PartialEq ──

#[test]
fn symbol_eq_terminal() {
    assert_eq!(Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(1)));
}

#[test]
fn symbol_ne_terminal_different_id() {
    assert_ne!(Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2)));
}

#[test]
fn symbol_ne_terminal_vs_nonterminal() {
    assert_ne!(
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(1))
    );
}

// ── Symbol Serialize ──

#[test]
fn symbol_serialize_terminal() {
    let s = Symbol::Terminal(SymbolId(42));
    let json = serde_json::to_string(&s).unwrap();
    assert!(!json.is_empty());
}

#[test]
fn symbol_serialize_nonterminal() {
    let s = Symbol::NonTerminal(SymbolId(99));
    let json = serde_json::to_string(&s).unwrap();
    assert!(!json.is_empty());
}

#[test]
fn symbol_roundtrip_terminal() {
    let s = Symbol::Terminal(SymbolId(42));
    let json = serde_json::to_string(&s).unwrap();
    let s2: Symbol = serde_json::from_str(&json).unwrap();
    assert_eq!(format!("{:?}", s), format!("{:?}", s2));
}

#[test]
fn symbol_roundtrip_nonterminal() {
    let s = Symbol::NonTerminal(SymbolId(99));
    let json = serde_json::to_string(&s).unwrap();
    let s2: Symbol = serde_json::from_str(&json).unwrap();
    assert_eq!(format!("{:?}", s), format!("{:?}", s2));
}

// ── Symbol in collections ──

#[test]
fn symbol_in_vec() {
    let v = vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(2)),
        Symbol::Terminal(SymbolId(3)),
    ];
    assert_eq!(v.len(), 3);
}

#[test]
fn symbol_vec_contains() {
    let v = vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(2)),
    ];
    assert!(v.contains(&Symbol::Terminal(SymbolId(1))));
    assert!(!v.contains(&Symbol::Terminal(SymbolId(99))));
}

// ── Pattern matching ──

#[test]
fn symbol_match_terminal() {
    let s = Symbol::Terminal(SymbolId(1));
    match s {
        Symbol::Terminal(_) => (),
        _ => panic!("expected Terminal"),
    }
}

#[test]
fn symbol_match_nonterminal() {
    let s = Symbol::NonTerminal(SymbolId(1));
    match s {
        Symbol::NonTerminal(_) => (),
        _ => panic!("expected NonTerminal"),
    }
}
