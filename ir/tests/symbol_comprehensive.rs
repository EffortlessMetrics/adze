//! Comprehensive tests for Symbol enum variants and operations.
//!
//! Covers: all Symbol variants, nesting, equality, clone, debug, serialization.

use adze_ir::{Associativity, Precedence, Symbol, SymbolId};

#[test]
fn symbol_terminal() {
    let s = Symbol::Terminal(SymbolId(1));
    assert!(matches!(s, Symbol::Terminal(SymbolId(1))));
}

#[test]
fn symbol_nonterminal() {
    let s = Symbol::NonTerminal(SymbolId(2));
    assert!(matches!(s, Symbol::NonTerminal(SymbolId(2))));
}

#[test]
fn symbol_external() {
    let s = Symbol::External(SymbolId(3));
    assert!(matches!(s, Symbol::External(SymbolId(3))));
}

#[test]
fn symbol_optional() {
    let s = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))));
    assert!(matches!(s, Symbol::Optional(_)));
}

#[test]
fn symbol_repeat() {
    let s = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(1))));
    assert!(matches!(s, Symbol::Repeat(_)));
}

#[test]
fn symbol_repeat_one() {
    let s = Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(1))));
    assert!(matches!(s, Symbol::RepeatOne(_)));
}

#[test]
fn symbol_choice() {
    let s = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
    ]);
    if let Symbol::Choice(c) = s {
        assert_eq!(c.len(), 2);
    } else {
        panic!("Expected Choice");
    }
}

#[test]
fn symbol_sequence() {
    let s = Symbol::Sequence(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(2)),
    ]);
    if let Symbol::Sequence(seq) = s {
        assert_eq!(seq.len(), 2);
    } else {
        panic!("Expected Sequence");
    }
}

#[test]
fn symbol_epsilon() {
    let s = Symbol::Epsilon;
    assert!(matches!(s, Symbol::Epsilon));
}

#[test]
fn symbol_nested_optional_repeat() {
    let s = Symbol::Optional(Box::new(Symbol::Repeat(Box::new(Symbol::Terminal(
        SymbolId(1),
    )))));
    assert!(matches!(s, Symbol::Optional(_)));
}

#[test]
fn symbol_choice_empty() {
    let s = Symbol::Choice(vec![]);
    if let Symbol::Choice(c) = s {
        assert!(c.is_empty());
    }
}

#[test]
fn symbol_sequence_single() {
    let s = Symbol::Sequence(vec![Symbol::Epsilon]);
    if let Symbol::Sequence(seq) = s {
        assert_eq!(seq.len(), 1);
    }
}

#[test]
fn symbol_clone() {
    let s = Symbol::Terminal(SymbolId(42));
    let s2 = s.clone();
    assert!(matches!(s2, Symbol::Terminal(SymbolId(42))));
}

#[test]
fn symbol_debug() {
    let s = Symbol::Terminal(SymbolId(1));
    let debug = format!("{:?}", s);
    assert!(debug.contains("Terminal"));
}

#[test]
fn symbol_id_copy() {
    let id = SymbolId(10);
    let id2 = id;
    let id3 = id; // Copy — id is still valid
    assert_eq!(id2.0, id3.0);
}

#[test]
fn symbol_id_equality() {
    assert_eq!(SymbolId(0), SymbolId(0));
    assert_ne!(SymbolId(0), SymbolId(1));
}

#[test]
fn symbol_id_ordering() {
    assert!(SymbolId(0) < SymbolId(1));
    assert!(SymbolId(100) > SymbolId(99));
}

#[test]
fn symbol_id_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(SymbolId(1));
    set.insert(SymbolId(2));
    set.insert(SymbolId(1)); // duplicate
    assert_eq!(set.len(), 2);
}

// --- Associativity tests ---

#[test]
fn associativity_left() {
    let a = Associativity::Left;
    assert!(matches!(a, Associativity::Left));
}

#[test]
fn associativity_right() {
    let a = Associativity::Right;
    assert!(matches!(a, Associativity::Right));
}

#[test]
fn associativity_none() {
    let a = Associativity::None;
    assert!(matches!(a, Associativity::None));
}

#[test]
fn associativity_equality() {
    assert_eq!(Associativity::Left, Associativity::Left);
    assert_ne!(Associativity::Left, Associativity::Right);
}

#[test]
fn associativity_copy() {
    let a = Associativity::Left;
    let b = a;
    let c = a; // Still valid because Copy
    assert_eq!(b, c);
}

// --- Precedence tests ---

#[test]
fn precedence_basic() {
    let p = Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(1), SymbolId(2)],
    };
    assert_eq!(p.level, 1);
    assert_eq!(p.symbols.len(), 2);
}

#[test]
fn precedence_negative_level() {
    let p = Precedence {
        level: -5,
        associativity: Associativity::None,
        symbols: vec![],
    };
    assert_eq!(p.level, -5);
}

#[test]
fn precedence_clone() {
    let p = Precedence {
        level: 10,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(1)],
    };
    let p2 = p.clone();
    assert_eq!(p2.level, 10);
    assert!(matches!(p2.associativity, Associativity::Right));
}

// --- Serialization roundtrip tests ---

#[test]
fn symbol_serde_terminal() {
    let s = Symbol::Terminal(SymbolId(5));
    let json = serde_json::to_string(&s).unwrap();
    let s2: Symbol = serde_json::from_str(&json).unwrap();
    assert!(matches!(s2, Symbol::Terminal(SymbolId(5))));
}

#[test]
fn associativity_serde() {
    for a in [
        Associativity::Left,
        Associativity::Right,
        Associativity::None,
    ] {
        let json = serde_json::to_string(&a).unwrap();
        let a2: Associativity = serde_json::from_str(&json).unwrap();
        assert_eq!(a, a2);
    }
}

#[test]
fn symbol_serde_complex() {
    let s = Symbol::Choice(vec![
        Symbol::Sequence(vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(2)))),
        ]),
        Symbol::Repeat(Box::new(Symbol::NonTerminal(SymbolId(3)))),
    ]);
    let json = serde_json::to_string(&s).unwrap();
    let s2: Symbol = serde_json::from_str(&json).unwrap();
    assert!(matches!(s2, Symbol::Choice(_)));
}
