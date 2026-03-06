#![cfg(feature = "test-api")]
//! Comprehensive v3 test suite for FirstFollowSets.
//!
//! 55+ tests covering: FIRST sets for terminals and non-terminals, FOLLOW sets,
//! nullable detection, consistency properties, complex grammars, and edge cases.

use adze_glr_core::FirstFollowSets;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Grammar construction helpers
// ---------------------------------------------------------------------------

fn tok(g: &mut Grammar, id: SymbolId, name: &str, pat: &str) {
    g.tokens.insert(
        id,
        Token {
            name: name.into(),
            pattern: TokenPattern::String(pat.into()),
            fragile: false,
        },
    );
}

fn make_rule(lhs: SymbolId, rhs: Vec<Symbol>, prod: u16) -> Rule {
    Rule {
        lhs,
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(prod),
    }
}

/// Check that FIRST(sym) contains exactly `expected` symbol ids.
fn assert_first_eq(ff: &FirstFollowSets, sym: SymbolId, expected: &[SymbolId]) {
    let set = ff
        .first(sym)
        .unwrap_or_else(|| panic!("no FIRST set for {sym:?}"));
    let actual: Vec<u16> = (0..set.len())
        .filter(|&i| set.contains(i))
        .map(|i| i as u16)
        .collect();
    let mut exp: Vec<u16> = expected.iter().map(|s| s.0).collect();
    exp.sort();
    assert_eq!(actual, exp, "FIRST({sym:?}) mismatch");
}

/// Check that FIRST(sym) contains all of `expected` (subset check).
fn assert_first_contains(ff: &FirstFollowSets, sym: SymbolId, expected: &[SymbolId]) {
    let set = ff
        .first(sym)
        .unwrap_or_else(|| panic!("no FIRST set for {sym:?}"));
    for &e in expected {
        assert!(
            set.contains(e.0 as usize),
            "FIRST({sym:?}) should contain {e:?}"
        );
    }
}

/// Check that FOLLOW(sym) contains all of `expected`.
fn assert_follow_contains(ff: &FirstFollowSets, sym: SymbolId, expected: &[SymbolId]) {
    let set = ff
        .follow(sym)
        .unwrap_or_else(|| panic!("no FOLLOW set for {sym:?}"));
    for &e in expected {
        assert!(
            set.contains(e.0 as usize),
            "FOLLOW({sym:?}) should contain {e:?}"
        );
    }
}

/// Check that FOLLOW(sym) contains exactly `expected`.
fn assert_follow_eq(ff: &FirstFollowSets, sym: SymbolId, expected: &[SymbolId]) {
    let set = ff
        .follow(sym)
        .unwrap_or_else(|| panic!("no FOLLOW set for {sym:?}"));
    let actual: Vec<u16> = (0..set.len())
        .filter(|&i| set.contains(i))
        .map(|i| i as u16)
        .collect();
    let mut exp: Vec<u16> = expected.iter().map(|s| s.0).collect();
    exp.sort();
    assert_eq!(actual, exp, "FOLLOW({sym:?}) mismatch");
}

// ---------------------------------------------------------------------------
// Symbol ID constants
// ---------------------------------------------------------------------------
const EOF: SymbolId = SymbolId(0);

const T_A: SymbolId = SymbolId(1);
const T_B: SymbolId = SymbolId(2);
const T_C: SymbolId = SymbolId(3);
const T_D: SymbolId = SymbolId(4);
const T_PLUS: SymbolId = SymbolId(5);
const T_STAR: SymbolId = SymbolId(6);
const T_LPAREN: SymbolId = SymbolId(7);
const T_RPAREN: SymbolId = SymbolId(8);
const T_NUM: SymbolId = SymbolId(9);
const T_E: SymbolId = SymbolId(10);
const T_F: SymbolId = SymbolId(11);

const NT_S: SymbolId = SymbolId(20);
const NT_A: SymbolId = SymbolId(21);
const NT_B: SymbolId = SymbolId(22);
const NT_C: SymbolId = SymbolId(23);
const NT_D: SymbolId = SymbolId(24);
const NT_EXPR: SymbolId = SymbolId(25);
const NT_TERM: SymbolId = SymbolId(26);
const NT_FACTOR: SymbolId = SymbolId(27);

// =========================================================================
// 1. FIRST sets for terminals (8 tests)
// =========================================================================

/// 1.1 FIRST set of a terminal used in a rule is clear (terminals don't self-populate).
#[test]
fn first_terminal_is_clear() {
    let mut g = Grammar::new("t_clear".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![make_rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);
    let ff = FirstFollowSets::compute(&g).unwrap();
    let first_a = ff.first(T_A).unwrap();
    assert!(first_a.is_clear(), "terminal FIRST set should be clear");
}

/// 1.2 Two distinct terminals both have clear FIRST sets.
#[test]
fn first_two_terminals_are_clear() {
    let mut g = Grammar::new("t_two".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![make_rule(
            NT_S,
            vec![Symbol::Terminal(T_A), Symbol::Terminal(T_B)],
            0,
        )],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.first(T_A).unwrap().is_clear());
    assert!(ff.first(T_B).unwrap().is_clear());
}

/// 1.3 Terminal has a FIRST set entry (Some, not None).
#[test]
fn first_terminal_exists() {
    let mut g = Grammar::new("t_exists".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![make_rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.first(T_A).is_some());
}

/// 1.4 Unused terminal still gets a FIRST entry.
#[test]
fn first_unused_terminal_has_entry() {
    let mut g = Grammar::new("t_unused".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![make_rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.first(T_B).is_some());
}

/// 1.5 Three terminals used in sequence — all clear.
#[test]
fn first_three_terminals_in_sequence_clear() {
    let mut g = Grammar::new("t_three_seq".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![make_rule(
            NT_S,
            vec![
                Symbol::Terminal(T_A),
                Symbol::Terminal(T_B),
                Symbol::Terminal(T_C),
            ],
            0,
        )],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.first(T_A).unwrap().is_clear());
    assert!(ff.first(T_B).unwrap().is_clear());
    assert!(ff.first(T_C).unwrap().is_clear());
}

/// 1.6 Terminal in alternative rules — still clear.
#[test]
fn first_terminal_in_alternatives_clear() {
    let mut g = Grammar::new("t_alt_clear".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![
            make_rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            make_rule(NT_S, vec![Symbol::Terminal(T_B)], 1),
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.first(T_A).unwrap().is_clear());
    assert!(ff.first(T_B).unwrap().is_clear());
}

/// 1.7 Terminal not in grammar has no FIRST set (None).
#[test]
fn first_unknown_terminal_is_none() {
    let mut g = Grammar::new("t_unknown".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![make_rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.first(SymbolId(9999)).is_none());
}

/// 1.8 Terminal is not nullable.
#[test]
fn terminal_is_not_nullable() {
    let mut g = Grammar::new("t_not_null".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![make_rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(!ff.is_nullable(T_A));
}

// =========================================================================
// 2. FIRST sets for non-terminals (10 tests)
// =========================================================================

/// 2.1 S → a  ⟹  FIRST(S) = {a}
#[test]
fn first_nt_single_terminal() {
    let mut g = Grammar::new("nt1".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![make_rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
}

/// 2.2 S → a b  ⟹  FIRST(S) = {a}
#[test]
fn first_nt_sequence_takes_first_terminal() {
    let mut g = Grammar::new("nt2".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![make_rule(
            NT_S,
            vec![Symbol::Terminal(T_A), Symbol::Terminal(T_B)],
            0,
        )],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
}

/// 2.3 S → a | b  ⟹  FIRST(S) = {a, b}
#[test]
fn first_nt_two_alternatives() {
    let mut g = Grammar::new("nt3".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![
            make_rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            make_rule(NT_S, vec![Symbol::Terminal(T_B)], 1),
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A, T_B]);
}

/// 2.4 S → a | b | c  ⟹  FIRST(S) = {a, b, c}
#[test]
fn first_nt_three_alternatives() {
    let mut g = Grammar::new("nt4".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![
            make_rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            make_rule(NT_S, vec![Symbol::Terminal(T_B)], 1),
            make_rule(NT_S, vec![Symbol::Terminal(T_C)], 2),
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A, T_B, T_C]);
}

/// 2.5 Chain propagation: S → A, A → a  ⟹  FIRST(S) = {a}
#[test]
fn first_nt_chain_propagation() {
    let mut g = Grammar::new("nt5".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules
        .insert(NT_A, vec![make_rule(NT_A, vec![Symbol::Terminal(T_A)], 0)]);
    g.rules.insert(
        NT_S,
        vec![make_rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 1)],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert_first_eq(&ff, NT_A, &[T_A]);
}

/// 2.6 Long chain: S → A → B → C → tok  ⟹  FIRST propagates through all.
#[test]
fn first_nt_long_chain() {
    let mut g = Grammar::new("nt6".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rules
        .insert(NT_C, vec![make_rule(NT_C, vec![Symbol::Terminal(T_A)], 0)]);
    g.rules.insert(
        NT_B,
        vec![make_rule(NT_B, vec![Symbol::NonTerminal(NT_C)], 1)],
    );
    g.rules.insert(
        NT_A,
        vec![make_rule(NT_A, vec![Symbol::NonTerminal(NT_B)], 2)],
    );
    g.rules.insert(
        NT_S,
        vec![make_rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 3)],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert_first_eq(&ff, NT_A, &[T_A]);
    assert_first_eq(&ff, NT_B, &[T_A]);
    assert_first_eq(&ff, NT_C, &[T_A]);
}

/// 2.7 Choice union: A → B | C, B → b, C → c  ⟹  FIRST(A) = {b, c}
#[test]
fn first_nt_choice_union() {
    let mut g = Grammar::new("nt7".into());
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rules
        .insert(NT_B, vec![make_rule(NT_B, vec![Symbol::Terminal(T_B)], 0)]);
    g.rules
        .insert(NT_C, vec![make_rule(NT_C, vec![Symbol::Terminal(T_C)], 1)]);
    g.rules.insert(
        NT_A,
        vec![
            make_rule(NT_A, vec![Symbol::NonTerminal(NT_B)], 2),
            make_rule(NT_A, vec![Symbol::NonTerminal(NT_C)], 3),
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_A, &[T_B, T_C]);
}

/// 2.8 Left recursion: A → A b | c  ⟹  FIRST(A) = {c}
#[test]
fn first_nt_left_recursion() {
    let mut g = Grammar::new("nt8".into());
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_A, "A".into());
    g.rules.insert(
        NT_A,
        vec![
            make_rule(
                NT_A,
                vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_B)],
                0,
            ),
            make_rule(NT_A, vec![Symbol::Terminal(T_C)], 1),
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_A, &[T_C]);
}

/// 2.9 Right recursion: A → a A | b  ⟹  FIRST(A) = {a, b}
#[test]
fn first_nt_right_recursion() {
    let mut g = Grammar::new("nt9".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_A, "A".into());
    g.rules.insert(
        NT_A,
        vec![
            make_rule(
                NT_A,
                vec![Symbol::Terminal(T_A), Symbol::NonTerminal(NT_A)],
                0,
            ),
            make_rule(NT_A, vec![Symbol::Terminal(T_B)], 1),
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_A, &[T_A, T_B]);
}

/// 2.10 Nullable prefix: S → A B, A → ε | a, B → b  ⟹  FIRST(S) = {a, b}
#[test]
fn first_nt_nullable_prefix() {
    let mut g = Grammar::new("nt10".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rules.insert(
        NT_A,
        vec![
            make_rule(NT_A, vec![Symbol::Epsilon], 0),
            make_rule(NT_A, vec![Symbol::Terminal(T_A)], 1),
        ],
    );
    g.rules
        .insert(NT_B, vec![make_rule(NT_B, vec![Symbol::Terminal(T_B)], 2)]);
    g.rules.insert(
        NT_S,
        vec![make_rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_B)],
            3,
        )],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A, T_B]);
}

// =========================================================================
// 3. FOLLOW sets for non-terminals (10 tests)
// =========================================================================

/// 3.1 FOLLOW(start) contains EOF.
#[test]
fn follow_start_contains_eof() {
    let mut g = Grammar::new("f1".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![make_rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_S, &[EOF]);
}

/// 3.2 S → A b  ⟹  FOLLOW(A) ⊇ {b}
#[test]
fn follow_terminal_after_nt() {
    let mut g = Grammar::new("f2".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules.insert(
        NT_S,
        vec![make_rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_B)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![make_rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[T_B]);
}

/// 3.3 S → A B, B → b  ⟹  FOLLOW(A) ⊇ FIRST(B) = {b}
#[test]
fn follow_first_of_following_nt() {
    let mut g = Grammar::new("f3".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rules.insert(
        NT_S,
        vec![make_rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_B)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![make_rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    g.rules
        .insert(NT_B, vec![make_rule(NT_B, vec![Symbol::Terminal(T_B)], 2)]);
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[T_B]);
}

/// 3.4 Last NT inherits FOLLOW of LHS: S → A B  ⟹  FOLLOW(B) ⊇ FOLLOW(S)
#[test]
fn follow_last_nt_inherits_lhs() {
    let mut g = Grammar::new("f4".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rules.insert(
        NT_S,
        vec![make_rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_B)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![make_rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    g.rules
        .insert(NT_B, vec![make_rule(NT_B, vec![Symbol::Terminal(T_B)], 2)]);
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_B, &[EOF]);
}

/// 3.5 Nullable suffix: S → A C, C → ε | c  ⟹  FOLLOW(A) ⊇ {c, EOF}
#[test]
fn follow_nullable_suffix() {
    let mut g = Grammar::new("f5".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rules.insert(
        NT_S,
        vec![make_rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_C)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![make_rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    g.rules.insert(
        NT_C,
        vec![
            make_rule(NT_C, vec![Symbol::Epsilon], 2),
            make_rule(NT_C, vec![Symbol::Terminal(T_C)], 3),
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[T_C, EOF]);
}

/// 3.6 Right recursion: S → a A, A → b A | c  ⟹  FOLLOW(A) ⊇ {EOF}
#[test]
fn follow_right_recursion() {
    let mut g = Grammar::new("f6".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules.insert(
        NT_S,
        vec![make_rule(
            NT_S,
            vec![Symbol::Terminal(T_A), Symbol::NonTerminal(NT_A)],
            0,
        )],
    );
    g.rules.insert(
        NT_A,
        vec![
            make_rule(
                NT_A,
                vec![Symbol::Terminal(T_B), Symbol::NonTerminal(NT_A)],
                1,
            ),
            make_rule(NT_A, vec![Symbol::Terminal(T_C)], 2),
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[EOF]);
}

/// 3.7 Left recursion: S → S a | b  ⟹  FOLLOW(S) ⊇ {a, EOF}
#[test]
fn follow_left_recursion() {
    let mut g = Grammar::new("f7".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![
            make_rule(
                NT_S,
                vec![Symbol::NonTerminal(NT_S), Symbol::Terminal(T_A)],
                0,
            ),
            make_rule(NT_S, vec![Symbol::Terminal(T_B)], 1),
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_S, &[T_A, EOF]);
}

/// 3.8 Middle NT: S → A B C  ⟹  FOLLOW(B) ⊇ FIRST(C)
#[test]
fn follow_middle_nt() {
    let mut g = Grammar::new("f8".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rules.insert(
        NT_S,
        vec![make_rule(
            NT_S,
            vec![
                Symbol::NonTerminal(NT_A),
                Symbol::NonTerminal(NT_B),
                Symbol::NonTerminal(NT_C),
            ],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![make_rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    g.rules
        .insert(NT_B, vec![make_rule(NT_B, vec![Symbol::Terminal(T_B)], 2)]);
    g.rules
        .insert(NT_C, vec![make_rule(NT_C, vec![Symbol::Terminal(T_C)], 3)]);
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_B, &[T_C]);
}

/// 3.9 FOLLOW(A) contains terminal after NT: S → A b c  ⟹  FOLLOW(A) ⊇ {b}
#[test]
fn follow_nt_then_terminal_sequence() {
    let mut g = Grammar::new("f9".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules.insert(
        NT_S,
        vec![make_rule(
            NT_S,
            vec![
                Symbol::NonTerminal(NT_A),
                Symbol::Terminal(T_B),
                Symbol::Terminal(T_C),
            ],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![make_rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[T_B]);
}

/// 3.10 FOLLOW propagation through two nullable: S → A B C, B → ε, C → ε  ⟹  FOLLOW(A) ⊇ FOLLOW(S)
#[test]
fn follow_through_two_nullable() {
    let mut g = Grammar::new("f10".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rules.insert(
        NT_S,
        vec![make_rule(
            NT_S,
            vec![
                Symbol::NonTerminal(NT_A),
                Symbol::NonTerminal(NT_B),
                Symbol::NonTerminal(NT_C),
            ],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![make_rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    g.rules
        .insert(NT_B, vec![make_rule(NT_B, vec![Symbol::Epsilon], 2)]);
    g.rules
        .insert(NT_C, vec![make_rule(NT_C, vec![Symbol::Epsilon], 3)]);
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[EOF]);
}

// =========================================================================
// 4. Nullable detection (8 tests)
// =========================================================================

/// 4.1 Epsilon rule: A → ε  ⟹  nullable(A)
#[test]
fn nullable_epsilon_rule() {
    let mut g = Grammar::new("n1".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_A, vec![make_rule(NT_A, vec![Symbol::Epsilon], 0)]);
    g.rules.insert(
        NT_S,
        vec![make_rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 1)],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
}

/// 4.2 Non-nullable: A → a  ⟹  ¬nullable(A)
#[test]
fn nullable_terminal_only_not_nullable() {
    let mut g = Grammar::new("n2".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_A, "A".into());
    g.rules
        .insert(NT_A, vec![make_rule(NT_A, vec![Symbol::Terminal(T_A)], 0)]);
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(!ff.is_nullable(NT_A));
}

/// 4.3 Propagation: A → B, B → ε  ⟹  nullable(A)
#[test]
fn nullable_propagation_through_chain() {
    let mut g = Grammar::new("n3".into());
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rules
        .insert(NT_B, vec![make_rule(NT_B, vec![Symbol::Epsilon], 0)]);
    g.rules.insert(
        NT_A,
        vec![make_rule(NT_A, vec![Symbol::NonTerminal(NT_B)], 1)],
    );
    g.rules.insert(
        NT_S,
        vec![make_rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 2)],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_B));
    assert!(ff.is_nullable(NT_A));
    assert!(ff.is_nullable(NT_S));
}

/// 4.4 Sequence of nullable: A → B C, B → ε, C → ε  ⟹  nullable(A)
#[test]
fn nullable_sequence_of_nullable() {
    let mut g = Grammar::new("n4".into());
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rules
        .insert(NT_B, vec![make_rule(NT_B, vec![Symbol::Epsilon], 0)]);
    g.rules
        .insert(NT_C, vec![make_rule(NT_C, vec![Symbol::Epsilon], 1)]);
    g.rules.insert(
        NT_A,
        vec![make_rule(
            NT_A,
            vec![Symbol::NonTerminal(NT_B), Symbol::NonTerminal(NT_C)],
            2,
        )],
    );
    g.rules.insert(
        NT_S,
        vec![make_rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 3)],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
}

/// 4.5 Partial nullable: A → B C, B → ε, C → c  ⟹  ¬nullable(A)
#[test]
fn nullable_partial_sequence_not_nullable() {
    let mut g = Grammar::new("n5".into());
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rules
        .insert(NT_B, vec![make_rule(NT_B, vec![Symbol::Epsilon], 0)]);
    g.rules
        .insert(NT_C, vec![make_rule(NT_C, vec![Symbol::Terminal(T_C)], 1)]);
    g.rules.insert(
        NT_A,
        vec![make_rule(
            NT_A,
            vec![Symbol::NonTerminal(NT_B), Symbol::NonTerminal(NT_C)],
            2,
        )],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_B));
    assert!(!ff.is_nullable(NT_C));
    assert!(!ff.is_nullable(NT_A));
}

/// 4.6 One alternative nullable: A → a | ε  ⟹  nullable(A)
#[test]
fn nullable_one_of_two_alternatives() {
    let mut g = Grammar::new("n6".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_A, "A".into());
    g.rules.insert(
        NT_A,
        vec![
            make_rule(NT_A, vec![Symbol::Terminal(T_A)], 0),
            make_rule(NT_A, vec![Symbol::Epsilon], 1),
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
}

/// 4.7 Unknown symbol is not nullable.
#[test]
fn nullable_unknown_symbol_is_false() {
    let mut g = Grammar::new("n7".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![make_rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(!ff.is_nullable(SymbolId(9999)));
}

/// 4.8 Terminal is never nullable.
#[test]
fn nullable_terminal_is_never_nullable() {
    let mut g = Grammar::new("n8".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![make_rule(
            NT_S,
            vec![Symbol::Terminal(T_A), Symbol::Terminal(T_B)],
            0,
        )],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(!ff.is_nullable(T_A));
    assert!(!ff.is_nullable(T_B));
}

// =========================================================================
// 5. FIRST/FOLLOW consistency properties (5 tests)
// =========================================================================

/// 5.1 Idempotent: compute twice on same grammar yields same results.
#[test]
fn consistency_idempotent() {
    let mut g = Grammar::new("c1".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules
        .insert(NT_A, vec![make_rule(NT_A, vec![Symbol::Terminal(T_A)], 0)]);
    g.rules.insert(
        NT_S,
        vec![make_rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_B)],
            1,
        )],
    );
    let ff1 = FirstFollowSets::compute(&g).unwrap();
    let ff2 = FirstFollowSets::compute(&g).unwrap();
    // Compare FIRST sets
    let f1 = ff1.first(NT_S).unwrap();
    let f2 = ff2.first(NT_S).unwrap();
    assert_eq!(f1.count_ones(..), f2.count_ones(..));
    // Compare FOLLOW sets
    let fo1 = ff1.follow(NT_S).unwrap();
    let fo2 = ff2.follow(NT_S).unwrap();
    assert_eq!(fo1.count_ones(..), fo2.count_ones(..));
}

/// 5.2 FIRST(NT) ⊆ terminals: no NT ID appears in a FIRST set.
#[test]
fn consistency_first_only_contains_terminals() {
    let mut g = Grammar::new("c2".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules.insert(
        NT_A,
        vec![
            make_rule(NT_A, vec![Symbol::Terminal(T_A)], 0),
            make_rule(NT_A, vec![Symbol::Terminal(T_B)], 1),
        ],
    );
    g.rules.insert(
        NT_S,
        vec![make_rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 2)],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    let first_s = ff.first(NT_S).unwrap();
    // NT IDs start at 20; only T_A(1) and T_B(2) should be in FIRST
    assert!(!first_s.contains(NT_S.0 as usize));
    assert!(!first_s.contains(NT_A.0 as usize));
}

/// 5.3 FOLLOW(start) always contains EOF.
#[test]
fn consistency_follow_start_always_has_eof() {
    // Multi-rule grammar — insert S rules first so it becomes the start symbol.
    let mut g = Grammar::new("c3".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules.insert(
        NT_S,
        vec![
            make_rule(
                NT_S,
                vec![
                    Symbol::NonTerminal(NT_A),
                    Symbol::Terminal(T_B),
                    Symbol::Terminal(T_C),
                ],
                1,
            ),
            make_rule(NT_S, vec![Symbol::Terminal(T_A)], 2),
        ],
    );
    g.rules
        .insert(NT_A, vec![make_rule(NT_A, vec![Symbol::Terminal(T_A)], 0)]);
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_S, &[EOF]);
}

/// 5.4 If A appears at end of production for S, then FOLLOW(S) ⊆ FOLLOW(A).
#[test]
fn consistency_follow_inheritance_at_end() {
    // S → a A  ⟹  FOLLOW(S) ⊆ FOLLOW(A)
    let mut g = Grammar::new("c4".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules.insert(
        NT_S,
        vec![make_rule(
            NT_S,
            vec![Symbol::Terminal(T_A), Symbol::NonTerminal(NT_A)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![make_rule(NT_A, vec![Symbol::Terminal(T_B)], 1)]);
    let ff = FirstFollowSets::compute(&g).unwrap();
    let follow_s = ff.follow(NT_S).unwrap();
    let follow_a = ff.follow(NT_A).unwrap();
    // Every element of FOLLOW(S) should be in FOLLOW(A)
    for i in 0..follow_s.len() {
        if follow_s.contains(i) {
            assert!(
                follow_a.contains(i),
                "FOLLOW(A) should contain everything in FOLLOW(S), missing bit {i}"
            );
        }
    }
}

/// 5.5 Non-nullable NT: FIRST(A) is non-empty.
#[test]
fn consistency_non_nullable_has_nonempty_first() {
    let mut g = Grammar::new("c5".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![
            make_rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            make_rule(NT_S, vec![Symbol::Terminal(T_B)], 1),
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(!ff.is_nullable(NT_S));
    assert!(ff.first(NT_S).unwrap().count_ones(..) > 0);
}

// =========================================================================
// 6. Complex grammars (8 tests)
// =========================================================================

/// 6.1 Arithmetic grammar: E → E + T | T, T → T * F | F, F → ( E ) | num
#[test]
fn complex_arithmetic_first_sets() {
    let mut g = Grammar::new("arith".into());
    tok(&mut g, T_PLUS, "+", "+");
    tok(&mut g, T_STAR, "*", "*");
    tok(&mut g, T_LPAREN, "(", "(");
    tok(&mut g, T_RPAREN, ")", ")");
    tok(&mut g, T_NUM, "num", "[0-9]+");
    g.rule_names.insert(NT_EXPR, "E".into());
    g.rule_names.insert(NT_TERM, "T".into());
    g.rule_names.insert(NT_FACTOR, "F".into());
    g.rules.insert(
        NT_EXPR,
        vec![
            make_rule(
                NT_EXPR,
                vec![
                    Symbol::NonTerminal(NT_EXPR),
                    Symbol::Terminal(T_PLUS),
                    Symbol::NonTerminal(NT_TERM),
                ],
                0,
            ),
            make_rule(NT_EXPR, vec![Symbol::NonTerminal(NT_TERM)], 1),
        ],
    );
    g.rules.insert(
        NT_TERM,
        vec![
            make_rule(
                NT_TERM,
                vec![
                    Symbol::NonTerminal(NT_TERM),
                    Symbol::Terminal(T_STAR),
                    Symbol::NonTerminal(NT_FACTOR),
                ],
                2,
            ),
            make_rule(NT_TERM, vec![Symbol::NonTerminal(NT_FACTOR)], 3),
        ],
    );
    g.rules.insert(
        NT_FACTOR,
        vec![
            make_rule(
                NT_FACTOR,
                vec![
                    Symbol::Terminal(T_LPAREN),
                    Symbol::NonTerminal(NT_EXPR),
                    Symbol::Terminal(T_RPAREN),
                ],
                4,
            ),
            make_rule(NT_FACTOR, vec![Symbol::Terminal(T_NUM)], 5),
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_FACTOR, &[T_LPAREN, T_NUM]);
    assert_first_eq(&ff, NT_TERM, &[T_LPAREN, T_NUM]);
    assert_first_eq(&ff, NT_EXPR, &[T_LPAREN, T_NUM]);
}

/// 6.2 Arithmetic grammar FOLLOW sets.
#[test]
fn complex_arithmetic_follow_sets() {
    let mut g = Grammar::new("arith2".into());
    tok(&mut g, T_PLUS, "+", "+");
    tok(&mut g, T_STAR, "*", "*");
    tok(&mut g, T_LPAREN, "(", "(");
    tok(&mut g, T_RPAREN, ")", ")");
    tok(&mut g, T_NUM, "num", "[0-9]+");
    g.rule_names.insert(NT_EXPR, "E".into());
    g.rule_names.insert(NT_TERM, "T".into());
    g.rule_names.insert(NT_FACTOR, "F".into());
    g.rules.insert(
        NT_EXPR,
        vec![
            make_rule(
                NT_EXPR,
                vec![
                    Symbol::NonTerminal(NT_EXPR),
                    Symbol::Terminal(T_PLUS),
                    Symbol::NonTerminal(NT_TERM),
                ],
                0,
            ),
            make_rule(NT_EXPR, vec![Symbol::NonTerminal(NT_TERM)], 1),
        ],
    );
    g.rules.insert(
        NT_TERM,
        vec![
            make_rule(
                NT_TERM,
                vec![
                    Symbol::NonTerminal(NT_TERM),
                    Symbol::Terminal(T_STAR),
                    Symbol::NonTerminal(NT_FACTOR),
                ],
                2,
            ),
            make_rule(NT_TERM, vec![Symbol::NonTerminal(NT_FACTOR)], 3),
        ],
    );
    g.rules.insert(
        NT_FACTOR,
        vec![
            make_rule(
                NT_FACTOR,
                vec![
                    Symbol::Terminal(T_LPAREN),
                    Symbol::NonTerminal(NT_EXPR),
                    Symbol::Terminal(T_RPAREN),
                ],
                4,
            ),
            make_rule(NT_FACTOR, vec![Symbol::Terminal(T_NUM)], 5),
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_EXPR, &[EOF, T_RPAREN, T_PLUS]);
    assert_follow_contains(&ff, NT_TERM, &[T_PLUS, T_RPAREN, EOF, T_STAR]);
    assert_follow_contains(&ff, NT_FACTOR, &[T_PLUS, T_STAR, T_RPAREN, EOF]);
}

/// 6.3 Arithmetic grammar: nothing is nullable.
#[test]
fn complex_arithmetic_nullable() {
    let mut g = Grammar::new("arith3".into());
    tok(&mut g, T_PLUS, "+", "+");
    tok(&mut g, T_STAR, "*", "*");
    tok(&mut g, T_LPAREN, "(", "(");
    tok(&mut g, T_RPAREN, ")", ")");
    tok(&mut g, T_NUM, "num", "[0-9]+");
    g.rule_names.insert(NT_EXPR, "E".into());
    g.rule_names.insert(NT_TERM, "T".into());
    g.rule_names.insert(NT_FACTOR, "F".into());
    g.rules.insert(
        NT_EXPR,
        vec![
            make_rule(
                NT_EXPR,
                vec![
                    Symbol::NonTerminal(NT_EXPR),
                    Symbol::Terminal(T_PLUS),
                    Symbol::NonTerminal(NT_TERM),
                ],
                0,
            ),
            make_rule(NT_EXPR, vec![Symbol::NonTerminal(NT_TERM)], 1),
        ],
    );
    g.rules.insert(
        NT_TERM,
        vec![
            make_rule(
                NT_TERM,
                vec![
                    Symbol::NonTerminal(NT_TERM),
                    Symbol::Terminal(T_STAR),
                    Symbol::NonTerminal(NT_FACTOR),
                ],
                2,
            ),
            make_rule(NT_TERM, vec![Symbol::NonTerminal(NT_FACTOR)], 3),
        ],
    );
    g.rules.insert(
        NT_FACTOR,
        vec![
            make_rule(
                NT_FACTOR,
                vec![
                    Symbol::Terminal(T_LPAREN),
                    Symbol::NonTerminal(NT_EXPR),
                    Symbol::Terminal(T_RPAREN),
                ],
                4,
            ),
            make_rule(NT_FACTOR, vec![Symbol::Terminal(T_NUM)], 5),
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(!ff.is_nullable(NT_EXPR));
    assert!(!ff.is_nullable(NT_TERM));
    assert!(!ff.is_nullable(NT_FACTOR));
}

/// 6.4 Mutual recursion: A → B a, B → A b | c
#[test]
fn complex_mutual_recursion() {
    let mut g = Grammar::new("mutual".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rules.insert(
        NT_A,
        vec![make_rule(
            NT_A,
            vec![Symbol::NonTerminal(NT_B), Symbol::Terminal(T_A)],
            0,
        )],
    );
    g.rules.insert(
        NT_B,
        vec![
            make_rule(
                NT_B,
                vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_B)],
                1,
            ),
            make_rule(NT_B, vec![Symbol::Terminal(T_C)], 2),
        ],
    );
    g.rules.insert(
        NT_S,
        vec![make_rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 3)],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    // FIRST(B) = {c}, FIRST(A) = FIRST(B) = {c}
    assert_first_contains(&ff, NT_B, &[T_C]);
    assert_first_contains(&ff, NT_A, &[T_C]);
}

/// 6.5 Diamond pattern: S → A | B, A → C d, B → C e, C → c
#[test]
fn complex_diamond() {
    let mut g = Grammar::new("diamond".into());
    tok(&mut g, T_C, "c", "c");
    tok(&mut g, T_D, "d", "d");
    tok(&mut g, T_E, "e", "e");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rules
        .insert(NT_C, vec![make_rule(NT_C, vec![Symbol::Terminal(T_C)], 0)]);
    g.rules.insert(
        NT_A,
        vec![make_rule(
            NT_A,
            vec![Symbol::NonTerminal(NT_C), Symbol::Terminal(T_D)],
            1,
        )],
    );
    g.rules.insert(
        NT_B,
        vec![make_rule(
            NT_B,
            vec![Symbol::NonTerminal(NT_C), Symbol::Terminal(T_E)],
            2,
        )],
    );
    g.rules.insert(
        NT_S,
        vec![
            make_rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 3),
            make_rule(NT_S, vec![Symbol::NonTerminal(NT_B)], 4),
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_C, &[T_C]);
    assert_first_eq(&ff, NT_A, &[T_C]);
    assert_first_eq(&ff, NT_B, &[T_C]);
    assert_first_eq(&ff, NT_S, &[T_C]);
}

/// 6.6 Nullable interaction: S → A B, A → ε | a, B → b | ε
#[test]
fn complex_nullable_interaction() {
    let mut g = Grammar::new("null_interact".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rules.insert(
        NT_A,
        vec![
            make_rule(NT_A, vec![Symbol::Epsilon], 0),
            make_rule(NT_A, vec![Symbol::Terminal(T_A)], 1),
        ],
    );
    g.rules.insert(
        NT_B,
        vec![
            make_rule(NT_B, vec![Symbol::Terminal(T_B)], 2),
            make_rule(NT_B, vec![Symbol::Epsilon], 3),
        ],
    );
    g.rules.insert(
        NT_S,
        vec![make_rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_B)],
            4,
        )],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
    assert!(ff.is_nullable(NT_B));
    assert!(ff.is_nullable(NT_S));
    // FIRST(S) = {a, b} (A nullable so b flows in)
    assert_first_eq(&ff, NT_S, &[T_A, T_B]);
}

/// 6.7 FIRST/FOLLOW overlap: S → A, A → B C, B → b | ε, C → b | c
#[test]
fn complex_first_follow_overlap() {
    let mut g = Grammar::new("overlap".into());
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rules.insert(
        NT_S,
        vec![make_rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0)],
    );
    g.rules.insert(
        NT_A,
        vec![make_rule(
            NT_A,
            vec![Symbol::NonTerminal(NT_B), Symbol::NonTerminal(NT_C)],
            1,
        )],
    );
    g.rules.insert(
        NT_B,
        vec![
            make_rule(NT_B, vec![Symbol::Terminal(T_B)], 2),
            make_rule(NT_B, vec![Symbol::Epsilon], 3),
        ],
    );
    g.rules.insert(
        NT_C,
        vec![
            make_rule(NT_C, vec![Symbol::Terminal(T_B)], 4),
            make_rule(NT_C, vec![Symbol::Terminal(T_C)], 5),
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_B));
    assert!(!ff.is_nullable(NT_C));
    assert_first_eq(&ff, NT_A, &[T_B, T_C]);
    assert_follow_contains(&ff, NT_B, &[T_B, T_C]);
}

/// 6.8 Many alternatives: S → a | b | c | d  ⟹  FIRST(S) = {a,b,c,d}
#[test]
fn complex_many_alternatives() {
    let mut g = Grammar::new("many_alt".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    tok(&mut g, T_D, "d", "d");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![
            make_rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            make_rule(NT_S, vec![Symbol::Terminal(T_B)], 1),
            make_rule(NT_S, vec![Symbol::Terminal(T_C)], 2),
            make_rule(NT_S, vec![Symbol::Terminal(T_D)], 3),
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A, T_B, T_C, T_D]);
}

// =========================================================================
// 7. Edge cases (6 tests)
// =========================================================================

/// 7.1 Empty grammar computes without error.
#[test]
fn edge_empty_grammar() {
    let g = Grammar::new("empty".into());
    assert!(FirstFollowSets::compute(&g).is_ok());
}

/// 7.2 Single epsilon-only rule.
#[test]
fn edge_single_epsilon_rule() {
    let mut g = Grammar::new("eps_only".into());
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![make_rule(NT_S, vec![Symbol::Epsilon], 0)]);
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_S));
    // FIRST(S) should be empty (epsilon is not a terminal)
    let first_s = ff.first(NT_S).unwrap();
    assert!(first_s.is_clear());
}

/// 7.3 Deep chain: S → A → B → C → D → tok  ⟹  all have FIRST = {tok}
#[test]
fn edge_deep_chain() {
    let mut g = Grammar::new("deep".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rule_names.insert(NT_D, "D".into());
    g.rules
        .insert(NT_D, vec![make_rule(NT_D, vec![Symbol::Terminal(T_A)], 0)]);
    g.rules.insert(
        NT_C,
        vec![make_rule(NT_C, vec![Symbol::NonTerminal(NT_D)], 1)],
    );
    g.rules.insert(
        NT_B,
        vec![make_rule(NT_B, vec![Symbol::NonTerminal(NT_C)], 2)],
    );
    g.rules.insert(
        NT_A,
        vec![make_rule(NT_A, vec![Symbol::NonTerminal(NT_B)], 3)],
    );
    g.rules.insert(
        NT_S,
        vec![make_rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 4)],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    for nt in [NT_S, NT_A, NT_B, NT_C, NT_D] {
        assert_first_eq(&ff, nt, &[T_A]);
    }
}

/// 7.4 Self-referencing epsilon: S → S | ε  ⟹  nullable(S), FIRST(S) = {}
#[test]
fn edge_self_referencing_epsilon() {
    let mut g = Grammar::new("self_eps".into());
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![
            make_rule(NT_S, vec![Symbol::NonTerminal(NT_S)], 0),
            make_rule(NT_S, vec![Symbol::Epsilon], 1),
        ],
    );
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_S));
}

/// 7.5 Many tokens defined, few used: only used tokens appear in FIRST.
#[test]
fn edge_many_tokens_few_used() {
    let mut g = Grammar::new("sparse".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    tok(&mut g, T_D, "d", "d");
    tok(&mut g, T_E, "e", "e");
    tok(&mut g, T_F, "f", "f");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![make_rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
}

/// 7.6 GrammarBuilder-based grammar works equivalently.
#[test]
fn edge_grammar_builder_equivalence() {
    use adze_ir::builder::GrammarBuilder;
    let mut g = GrammarBuilder::new("builder_test")
        .token("a", "a")
        .token("b", "b")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["b"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();

    let start_id = g.start_symbol().unwrap();
    let first_start = ff.first(start_id).unwrap();
    assert!(
        first_start.count_ones(..) >= 2,
        "FIRST(start) should have a and b"
    );
    let follow_start = ff.follow(start_id).unwrap();
    assert!(follow_start.contains(0), "FOLLOW(start) should contain EOF");
}
