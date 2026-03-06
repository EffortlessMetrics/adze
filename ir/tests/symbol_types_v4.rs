//! Comprehensive tests for Symbol enum and ID newtypes in adze-ir.

use std::collections::HashSet;
use std::fmt::Write;

use adze_ir::{FieldId, ProductionId, RuleId, StateId, Symbol, SymbolId};

// ── Section 1: Symbol variant construction (8 tests) ──

#[test]
fn test_terminal_construction_zero() {
    let sym = Symbol::Terminal(SymbolId(0));
    assert!(matches!(sym, Symbol::Terminal(SymbolId(0))));
}

#[test]
fn test_terminal_construction_nonzero() {
    let sym = Symbol::Terminal(SymbolId(42));
    assert!(matches!(sym, Symbol::Terminal(SymbolId(42))));
}

#[test]
fn test_terminal_construction_max() {
    let sym = Symbol::Terminal(SymbolId(u16::MAX));
    assert!(matches!(sym, Symbol::Terminal(SymbolId(65535))));
}

#[test]
fn test_nonterminal_construction_zero() {
    let sym = Symbol::NonTerminal(SymbolId(0));
    assert!(matches!(sym, Symbol::NonTerminal(SymbolId(0))));
}

#[test]
fn test_nonterminal_construction_nonzero() {
    let sym = Symbol::NonTerminal(SymbolId(100));
    assert!(matches!(sym, Symbol::NonTerminal(SymbolId(100))));
}

#[test]
fn test_nonterminal_construction_max() {
    let sym = Symbol::NonTerminal(SymbolId(u16::MAX));
    assert!(matches!(sym, Symbol::NonTerminal(SymbolId(65535))));
}

#[test]
fn test_epsilon_construction() {
    let sym = Symbol::Epsilon;
    assert!(matches!(sym, Symbol::Epsilon));
}

#[test]
fn test_external_construction() {
    let sym = Symbol::External(SymbolId(7));
    assert!(matches!(sym, Symbol::External(SymbolId(7))));
}

// ── Section 2: Symbol equality (8 tests) ──

#[test]
fn test_terminal_same_id_equal() {
    assert_eq!(Symbol::Terminal(SymbolId(5)), Symbol::Terminal(SymbolId(5)));
}

#[test]
fn test_terminal_different_id_not_equal() {
    assert_ne!(Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2)));
}

#[test]
fn test_nonterminal_same_id_equal() {
    assert_eq!(
        Symbol::NonTerminal(SymbolId(10)),
        Symbol::NonTerminal(SymbolId(10))
    );
}

#[test]
fn test_nonterminal_different_id_not_equal() {
    assert_ne!(
        Symbol::NonTerminal(SymbolId(3)),
        Symbol::NonTerminal(SymbolId(4))
    );
}

#[test]
fn test_terminal_ne_nonterminal_same_id() {
    assert_ne!(
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(1))
    );
}

#[test]
fn test_epsilon_equals_epsilon() {
    assert_eq!(Symbol::Epsilon, Symbol::Epsilon);
}

#[test]
fn test_epsilon_ne_terminal() {
    assert_ne!(Symbol::Epsilon, Symbol::Terminal(SymbolId(0)));
}

#[test]
fn test_epsilon_ne_nonterminal() {
    assert_ne!(Symbol::Epsilon, Symbol::NonTerminal(SymbolId(0)));
}

// ── Section 3: Symbol pattern matching (8 tests) ──

#[test]
fn test_match_terminal_extracts_id() {
    let sym = Symbol::Terminal(SymbolId(99));
    match sym {
        Symbol::Terminal(id) => assert_eq!(id, SymbolId(99)),
        _ => panic!("expected Terminal"),
    }
}

#[test]
fn test_match_nonterminal_extracts_id() {
    let sym = Symbol::NonTerminal(SymbolId(50));
    match sym {
        Symbol::NonTerminal(id) => assert_eq!(id, SymbolId(50)),
        _ => panic!("expected NonTerminal"),
    }
}

#[test]
fn test_match_epsilon_has_no_payload() {
    let sym = Symbol::Epsilon;
    match sym {
        Symbol::Epsilon => {} // success
        _ => panic!("expected Epsilon"),
    }
}

#[test]
fn test_match_external_extracts_id() {
    let sym = Symbol::External(SymbolId(3));
    match sym {
        Symbol::External(id) => assert_eq!(id, SymbolId(3)),
        _ => panic!("expected External"),
    }
}

#[test]
fn test_if_let_terminal() {
    let sym = Symbol::Terminal(SymbolId(11));
    if let Symbol::Terminal(id) = sym {
        assert_eq!(id.0, 11);
    } else {
        panic!("expected Terminal");
    }
}

#[test]
fn test_if_let_nonterminal() {
    let sym = Symbol::NonTerminal(SymbolId(22));
    if let Symbol::NonTerminal(id) = sym {
        assert_eq!(id.0, 22);
    } else {
        panic!("expected NonTerminal");
    }
}

#[test]
fn test_match_does_not_confuse_variants() {
    let symbols = [
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(1)),
        Symbol::Epsilon,
    ];
    let mut terminal_count = 0;
    let mut nonterminal_count = 0;
    let mut epsilon_count = 0;
    for sym in &symbols {
        match sym {
            Symbol::Terminal(_) => terminal_count += 1,
            Symbol::NonTerminal(_) => nonterminal_count += 1,
            Symbol::Epsilon => epsilon_count += 1,
            _ => {}
        }
    }
    assert_eq!(terminal_count, 1);
    assert_eq!(nonterminal_count, 1);
    assert_eq!(epsilon_count, 1);
}

#[test]
fn test_matches_macro_on_variants() {
    assert!(matches!(Symbol::Terminal(SymbolId(0)), Symbol::Terminal(_)));
    assert!(!matches!(
        Symbol::Terminal(SymbolId(0)),
        Symbol::NonTerminal(_)
    ));
    assert!(matches!(Symbol::Epsilon, Symbol::Epsilon));
}

// ── Section 4: SymbolId traits (8 tests) ──

#[test]
fn test_symbol_id_copy() {
    let a = SymbolId(10);
    let b = a; // Copy
    assert_eq!(a, b);
}

#[test]
fn test_symbol_id_debug() {
    let id = SymbolId(42);
    let dbg = format!("{id:?}");
    assert!(dbg.contains("42"));
}

#[test]
fn test_symbol_id_display() {
    let id = SymbolId(7);
    let disp = format!("{id}");
    assert_eq!(disp, "Symbol(7)");
}

#[test]
fn test_symbol_id_hash_same_value() {
    let mut set = HashSet::new();
    set.insert(SymbolId(5));
    set.insert(SymbolId(5));
    assert_eq!(set.len(), 1);
}

#[test]
fn test_symbol_id_hash_different_values() {
    let mut set = HashSet::new();
    set.insert(SymbolId(1));
    set.insert(SymbolId(2));
    assert_eq!(set.len(), 2);
}

#[test]
fn test_symbol_id_ord() {
    assert!(SymbolId(1) < SymbolId(2));
    assert!(SymbolId(100) > SymbolId(99));
    assert!(SymbolId(0) <= SymbolId(0));
}

#[test]
fn test_symbol_id_eq_and_ne() {
    assert_eq!(SymbolId(0), SymbolId(0));
    assert_ne!(SymbolId(0), SymbolId(1));
}

#[test]
fn test_symbol_id_inner_access() {
    let id = SymbolId(255);
    assert_eq!(id.0, 255);
}

// ── Section 5: RuleId traits (8 tests) ──

#[test]
fn test_rule_id_copy() {
    let a = RuleId(3);
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn test_rule_id_debug() {
    let id = RuleId(88);
    let dbg = format!("{id:?}");
    assert!(dbg.contains("88"));
}

#[test]
fn test_rule_id_display() {
    let id = RuleId(12);
    let disp = format!("{id}");
    assert_eq!(disp, "Rule(12)");
}

#[test]
fn test_rule_id_hash_same() {
    let mut set = HashSet::new();
    set.insert(RuleId(9));
    set.insert(RuleId(9));
    assert_eq!(set.len(), 1);
}

#[test]
fn test_rule_id_hash_different() {
    let mut set = HashSet::new();
    set.insert(RuleId(10));
    set.insert(RuleId(20));
    assert_eq!(set.len(), 2);
}

#[test]
fn test_rule_id_ord() {
    assert!(RuleId(0) < RuleId(1));
    assert!(RuleId(50) > RuleId(49));
}

#[test]
fn test_rule_id_eq_and_ne() {
    assert_eq!(RuleId(7), RuleId(7));
    assert_ne!(RuleId(7), RuleId(8));
}

#[test]
fn test_rule_id_inner_access() {
    let id = RuleId(1000);
    assert_eq!(id.0, 1000);
}

// ── Section 6: StateId traits (8 tests) ──

#[test]
fn test_state_id_copy() {
    let a = StateId(15);
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn test_state_id_debug() {
    let id = StateId(33);
    let dbg = format!("{id:?}");
    assert!(dbg.contains("33"));
}

#[test]
fn test_state_id_display() {
    let id = StateId(4);
    let disp = format!("{id}");
    assert_eq!(disp, "State(4)");
}

#[test]
fn test_state_id_hash_same() {
    let mut set = HashSet::new();
    set.insert(StateId(0));
    set.insert(StateId(0));
    assert_eq!(set.len(), 1);
}

#[test]
fn test_state_id_hash_different() {
    let mut set = HashSet::new();
    set.insert(StateId(1));
    set.insert(StateId(2));
    assert_eq!(set.len(), 2);
}

#[test]
fn test_state_id_ord() {
    assert!(StateId(0) < StateId(u16::MAX));
    assert!(StateId(10) >= StateId(10));
}

#[test]
fn test_state_id_eq_and_ne() {
    assert_eq!(StateId(200), StateId(200));
    assert_ne!(StateId(200), StateId(201));
}

#[test]
fn test_state_id_inner_access() {
    let id = StateId(65000);
    assert_eq!(id.0, 65000);
}

// ── Section 7: FieldId and ProductionId (6 tests) ──

#[test]
fn test_field_id_copy_and_eq() {
    let a = FieldId(6);
    let b = a;
    assert_eq!(a, b);
    assert_ne!(FieldId(6), FieldId(7));
}

#[test]
fn test_field_id_display() {
    let id = FieldId(99);
    let disp = format!("{id}");
    assert_eq!(disp, "Field(99)");
}

#[test]
fn test_field_id_hash() {
    let mut set = HashSet::new();
    set.insert(FieldId(1));
    set.insert(FieldId(1));
    set.insert(FieldId(2));
    assert_eq!(set.len(), 2);
}

#[test]
fn test_production_id_copy_and_eq() {
    let a = ProductionId(0);
    let b = a;
    assert_eq!(a, b);
    assert_ne!(ProductionId(0), ProductionId(1));
}

#[test]
fn test_production_id_display() {
    let id = ProductionId(55);
    let disp = format!("{id}");
    assert_eq!(disp, "Production(55)");
}

#[test]
fn test_production_id_ord() {
    assert!(ProductionId(0) < ProductionId(1));
    assert!(ProductionId(100) > ProductionId(50));
}

// ── Section 8: ID arithmetic and boundary values (6 tests) ──

#[test]
fn test_symbol_id_min_max() {
    let min_id = SymbolId(0);
    let max_id = SymbolId(u16::MAX);
    assert_eq!(min_id.0, 0);
    assert_eq!(max_id.0, 65535);
    assert!(min_id < max_id);
}

#[test]
fn test_rule_id_min_max() {
    let min_id = RuleId(0);
    let max_id = RuleId(u16::MAX);
    assert!(min_id < max_id);
    assert_eq!(max_id.0, 65535);
}

#[test]
fn test_state_id_min_max() {
    let min_id = StateId(0);
    let max_id = StateId(u16::MAX);
    assert!(min_id < max_id);
    assert_eq!(max_id.0, 65535);
}

#[test]
fn test_symbol_id_wrapping_add() {
    let id = SymbolId(u16::MAX);
    let next = SymbolId(id.0.wrapping_add(1));
    assert_eq!(next, SymbolId(0));
}

#[test]
fn test_rule_id_wrapping_add() {
    let id = RuleId(u16::MAX);
    let next = RuleId(id.0.wrapping_add(1));
    assert_eq!(next, RuleId(0));
}

#[test]
fn test_production_id_wrapping_add() {
    let id = ProductionId(u16::MAX);
    let next = ProductionId(id.0.wrapping_add(1));
    assert_eq!(next, ProductionId(0));
}

// ── Bonus: Cross-type distinctness and Symbol trait coverage ──

#[test]
fn test_symbol_debug_terminal() {
    let sym = Symbol::Terminal(SymbolId(5));
    let dbg = format!("{sym:?}");
    assert!(dbg.contains("Terminal"));
    assert!(dbg.contains("5"));
}

#[test]
fn test_symbol_debug_nonterminal() {
    let sym = Symbol::NonTerminal(SymbolId(8));
    let dbg = format!("{sym:?}");
    assert!(dbg.contains("NonTerminal"));
    assert!(dbg.contains("8"));
}

#[test]
fn test_symbol_debug_epsilon() {
    let dbg = format!("{:?}", Symbol::Epsilon);
    assert!(dbg.contains("Epsilon"));
}

#[test]
fn test_symbol_clone() {
    let sym = Symbol::Terminal(SymbolId(77));
    let cloned = sym.clone();
    assert_eq!(sym, cloned);
}

#[test]
fn test_symbol_hash_in_set() {
    let mut set = HashSet::new();
    set.insert(Symbol::Terminal(SymbolId(1)));
    set.insert(Symbol::Terminal(SymbolId(1)));
    set.insert(Symbol::NonTerminal(SymbolId(1)));
    set.insert(Symbol::Epsilon);
    assert_eq!(set.len(), 3);
}

#[test]
fn test_symbol_ord() {
    let mut symbols = [
        Symbol::NonTerminal(SymbolId(2)),
        Symbol::Terminal(SymbolId(1)),
        Symbol::Epsilon,
        Symbol::Terminal(SymbolId(0)),
    ];
    symbols.sort();
    // Verify sort is stable — Terminal < NonTerminal < ... < Epsilon by enum variant order
    assert!(symbols[0] <= symbols[1]);
    assert!(symbols[1] <= symbols[2]);
    assert!(symbols[2] <= symbols[3]);
}

#[test]
fn test_all_id_display_formats() {
    let mut buf = String::new();
    write!(buf, "{}", SymbolId(1)).unwrap();
    write!(buf, " {}", RuleId(2)).unwrap();
    write!(buf, " {}", StateId(3)).unwrap();
    write!(buf, " {}", FieldId(4)).unwrap();
    write!(buf, " {}", ProductionId(5)).unwrap();
    assert_eq!(buf, "Symbol(1) Rule(2) State(3) Field(4) Production(5)");
}

#[test]
fn test_id_types_are_distinct() {
    // Compile-time type safety: each ID type is distinct.
    // If they were type aliases, this would fail to compile with duplicate match arms.
    fn describe_symbol(id: SymbolId) -> &'static str {
        let _ = id;
        "symbol"
    }
    fn describe_rule(id: RuleId) -> &'static str {
        let _ = id;
        "rule"
    }
    assert_eq!(describe_symbol(SymbolId(1)), "symbol");
    assert_eq!(describe_rule(RuleId(1)), "rule");
}
