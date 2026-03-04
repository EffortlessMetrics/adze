//! Comprehensive tests for Symbol and SymbolId patterns.

use adze_ir::{Symbol, SymbolId};

#[test]
fn terminal_variant() {
    let s = Symbol::Terminal(SymbolId(1));
    assert!(matches!(s, Symbol::Terminal(_)));
}

#[test]
fn nonterminal_variant() {
    let s = Symbol::NonTerminal(SymbolId(2));
    assert!(matches!(s, Symbol::NonTerminal(_)));
}

#[test]
fn terminal_id_access() {
    let s = Symbol::Terminal(SymbolId(42));
    if let Symbol::Terminal(id) = s {
        assert_eq!(id.0, 42);
    }
}

#[test]
fn nonterminal_id_access() {
    let s = Symbol::NonTerminal(SymbolId(99));
    if let Symbol::NonTerminal(id) = s {
        assert_eq!(id.0, 99);
    }
}

#[test]
fn symbol_clone_terminal() {
    let s = Symbol::Terminal(SymbolId(5));
    let c = s.clone();
    assert_eq!(s, c);
}

#[test]
fn symbol_clone_nonterminal() {
    let s = Symbol::NonTerminal(SymbolId(7));
    let c = s.clone();
    assert_eq!(s, c);
}

#[test]
fn symbol_debug_terminal() {
    let d = format!("{:?}", Symbol::Terminal(SymbolId(1)));
    assert!(!d.is_empty());
}

#[test]
fn symbol_debug_nonterminal() {
    let d = format!("{:?}", Symbol::NonTerminal(SymbolId(2)));
    assert!(!d.is_empty());
}

#[test]
fn symbol_eq_same_variant() {
    assert_eq!(Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(1)));
}

#[test]
fn symbol_ne_diff_variant() {
    assert_ne!(
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(1))
    );
}

#[test]
fn symbol_ne_diff_id() {
    assert_ne!(Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2)));
}

#[test]
fn symbol_id_zero() {
    assert_eq!(SymbolId(0).0, 0);
}

#[test]
fn symbol_id_max() {
    assert_eq!(SymbolId(u16::MAX).0, u16::MAX);
}

#[test]
fn symbol_id_copy() {
    let id = SymbolId(10);
    let id2 = id;
    assert_eq!(id, id2);
}

#[test]
fn symbol_id_hash() {
    use std::collections::HashSet;
    let mut s = HashSet::new();
    s.insert(SymbolId(1));
    s.insert(SymbolId(2));
    s.insert(SymbolId(1));
    assert_eq!(s.len(), 2);
}

#[test]
fn symbol_vec_collect() {
    let v: Vec<Symbol> = (0..10).map(|i| Symbol::Terminal(SymbolId(i))).collect();
    assert_eq!(v.len(), 10);
}

#[test]
fn symbol_serde_json() {
    let s = Symbol::Terminal(SymbolId(42));
    let json = serde_json::to_string(&s).unwrap();
    let s2: Symbol = serde_json::from_str(&json).unwrap();
    assert_eq!(s, s2);
}

#[test]
fn symbol_id_serde_json() {
    let id = SymbolId(123);
    let json = serde_json::to_string(&id).unwrap();
    let id2: SymbolId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, id2);
}
