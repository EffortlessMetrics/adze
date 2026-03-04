// Comprehensive tests for IR Symbol enum and its variants
// Tests construction, matching, cloning, and serialization

use adze_ir::{Symbol, SymbolId};

#[test]
fn terminal_construction() {
    let s = Symbol::Terminal(SymbolId(0));
    assert!(matches!(s, Symbol::Terminal(_)));
}

#[test]
fn nonterminal_construction() {
    let s = Symbol::NonTerminal(SymbolId(1));
    assert!(matches!(s, Symbol::NonTerminal(_)));
}

#[test]
fn epsilon_construction() {
    let s = Symbol::Epsilon;
    assert!(matches!(s, Symbol::Epsilon));
}

#[test]
fn optional_construction() {
    let inner = Symbol::Terminal(SymbolId(0));
    let s = Symbol::Optional(Box::new(inner));
    assert!(matches!(s, Symbol::Optional(_)));
}

#[test]
fn repeat_construction() {
    let inner = Symbol::Terminal(SymbolId(0));
    let s = Symbol::Repeat(Box::new(inner));
    assert!(matches!(s, Symbol::Repeat(_)));
}

#[test]
fn choice_construction() {
    let s = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(0)),
        Symbol::Terminal(SymbolId(1)),
    ]);
    assert!(matches!(s, Symbol::Choice(_)));
}

#[test]
fn sequence_construction() {
    let s = Symbol::Sequence(vec![
        Symbol::Terminal(SymbolId(0)),
        Symbol::Terminal(SymbolId(1)),
    ]);
    assert!(matches!(s, Symbol::Sequence(_)));
}

#[test]
fn symbol_clone() {
    let s = Symbol::Terminal(SymbolId(42));
    let s2 = s.clone();
    assert!(matches!(s2, Symbol::Terminal(id) if id == SymbolId(42)));
}

#[test]
fn symbol_debug() {
    let s = Symbol::Terminal(SymbolId(5));
    let dbg = format!("{:?}", s);
    assert!(!dbg.is_empty());
}

#[test]
fn symbol_nested_optional_repeat() {
    let inner = Symbol::Terminal(SymbolId(0));
    let rep = Symbol::Repeat(Box::new(inner));
    let opt = Symbol::Optional(Box::new(rep));
    assert!(matches!(opt, Symbol::Optional(_)));
}

#[test]
fn symbol_choice_empty() {
    let s = Symbol::Choice(vec![]);
    assert!(matches!(s, Symbol::Choice(v) if v.is_empty()));
}

#[test]
fn symbol_sequence_empty() {
    let s = Symbol::Sequence(vec![]);
    assert!(matches!(s, Symbol::Sequence(v) if v.is_empty()));
}

#[test]
fn symbol_serde_roundtrip_terminal() {
    let s = Symbol::Terminal(SymbolId(99));
    let json = serde_json::to_string(&s).unwrap();
    let s2: Symbol = serde_json::from_str(&json).unwrap();
    assert!(matches!(s2, Symbol::Terminal(id) if id == SymbolId(99)));
}

#[test]
fn symbol_serde_roundtrip_epsilon() {
    let s = Symbol::Epsilon;
    let json = serde_json::to_string(&s).unwrap();
    let s2: Symbol = serde_json::from_str(&json).unwrap();
    assert!(matches!(s2, Symbol::Epsilon));
}

#[test]
fn symbol_serde_roundtrip_choice() {
    let s = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(0)),
        Symbol::NonTerminal(SymbolId(1)),
    ]);
    let json = serde_json::to_string(&s).unwrap();
    let s2: Symbol = serde_json::from_str(&json).unwrap();
    assert!(matches!(s2, Symbol::Choice(_)));
}
