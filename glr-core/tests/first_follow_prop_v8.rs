#![allow(clippy::needless_range_loop)]
//! Property and invariant tests for `FirstFollowSets` in adze-glr-core (v8).
//!
//! 80+ tests across 20 categories:
//! 1.  Terminal FIRST set contains itself
//! 2.  Non-terminal FIRST set contains first terminal of rule
//! 3.  FOLLOW(start) contains EOF
//! 4.  Nullable detection for epsilon rules
//! 5.  Non-nullable for terminal-only rules
//! 6.  FIRST() returns non-empty for any reachable symbol
//! 7.  FOLLOW() returns non-empty for any referenced non-terminal
//! 8.  FIRST sets don't contain EOF (unless nullable)
//! 9.  FOLLOW sets can contain EOF
//! 10. Single rule grammar FIRST/FOLLOW
//! 11. Grammar with alternatives → union of FIRST sets
//! 12. Chain rule propagation
//! 13. Multiple tokens → each terminal's FIRST is {self}
//! 14. Precedence doesn't affect FIRST/FOLLOW
//! 15. Associativity doesn't affect FIRST/FOLLOW
//! 16. Determinism: same grammar → same sets
//! 17. Clone grammar → same FIRST/FOLLOW
//! 18. Various grammar sizes
//! 19. Grammar with extras → FIRST/FOLLOW still valid
//! 20. Grammar with inline rules → FIRST/FOLLOW still valid

use adze_glr_core::FirstFollowSets;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
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

fn rule(lhs: SymbolId, rhs: Vec<Symbol>, prod: u16) -> Rule {
    Rule {
        lhs,
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(prod),
    }
}

fn rule_prec(
    lhs: SymbolId,
    rhs: Vec<Symbol>,
    prod: u16,
    prec: i16,
    assoc: Option<Associativity>,
) -> Rule {
    Rule {
        lhs,
        rhs,
        precedence: Some(PrecedenceKind::Static(prec)),
        associativity: assoc,
        fields: vec![],
        production_id: ProductionId(prod),
    }
}

/// Assert FIRST(sym) contains exactly the expected symbol ids.
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

/// Assert FIRST(sym) contains all of `expected` (may contain more).
fn assert_first_contains(ff: &FirstFollowSets, sym: SymbolId, expected: &[SymbolId]) {
    let set = ff
        .first(sym)
        .unwrap_or_else(|| panic!("no FIRST set for {sym:?}"));
    for &e in expected {
        assert!(
            set.contains(e.0 as usize),
            "FIRST({sym:?}) should contain {e:?}",
        );
    }
}

/// Assert FOLLOW(sym) contains all of `expected` (may contain more).
fn assert_follow_contains(ff: &FirstFollowSets, sym: SymbolId, expected: &[SymbolId]) {
    let set = ff
        .follow(sym)
        .unwrap_or_else(|| panic!("no FOLLOW set for {sym:?}"));
    for &e in expected {
        assert!(
            set.contains(e.0 as usize),
            "FOLLOW({sym:?}) should contain {e:?}",
        );
    }
}

/// Assert FOLLOW(sym) contains exactly the expected symbol ids.
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

/// Count bits set in the FIRST set for a symbol.
fn first_count(ff: &FirstFollowSets, sym: SymbolId) -> usize {
    ff.first(sym).map(|s| s.count_ones(..)).unwrap_or(0)
}

/// Count bits set in the FOLLOW set for a symbol.
fn follow_count(ff: &FirstFollowSets, sym: SymbolId) -> usize {
    ff.follow(sym).map(|s| s.count_ones(..)).unwrap_or(0)
}

/// Assert FIRST(sym) does NOT contain `absent`.
fn assert_first_not_contains(ff: &FirstFollowSets, sym: SymbolId, absent: SymbolId) {
    if let Some(set) = ff.first(sym) {
        assert!(
            !set.contains(absent.0 as usize),
            "FIRST({sym:?}) should NOT contain {absent:?}",
        );
    }
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
const T_WS: SymbolId = SymbolId(12);
const T_COMMENT: SymbolId = SymbolId(13);
const T_SEMI: SymbolId = SymbolId(14);

const NT_S: SymbolId = SymbolId(30);
const NT_A: SymbolId = SymbolId(31);
const NT_B: SymbolId = SymbolId(32);
const NT_C: SymbolId = SymbolId(33);
const NT_D: SymbolId = SymbolId(34);
const NT_E: SymbolId = SymbolId(35);
const NT_T: SymbolId = SymbolId(36);
const NT_F: SymbolId = SymbolId(37);

// =========================================================================
// Category 1: Terminal in leading position → its ID appears in FIRST of NT
// =========================================================================

/// S → a — FIRST(S) contains the terminal `a`.
#[test]
fn cat01_terminal_appears_in_nt_first() {
    let mut g = Grammar::new("ffp_v8_c01a".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
}

/// S → a b c — only the leading terminal 'a' is in FIRST(S).
#[test]
fn cat01_leading_terminal_only_in_first() {
    let mut g = Grammar::new("ffp_v8_c01b".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![rule(
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
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert!(!ff.is_nullable(NT_S));
}

/// Each alternative's leading terminal appears in FIRST(S).
#[test]
fn cat01_each_alternative_terminal_in_first() {
    let mut g = Grammar::new("ffp_v8_c01c".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            rule(NT_S, vec![Symbol::Terminal(T_B)], 1),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A, T_B]);
}

/// Operator terminals appear correctly in FIRST of containing NT.
#[test]
fn cat01_operator_terminal_in_nt_first() {
    let mut g = Grammar::new("ffp_v8_c01d".into());
    tok(&mut g, T_PLUS, "plus", "+");
    tok(&mut g, T_STAR, "star", "*");
    tok(&mut g, T_NUM, "num", "0");
    g.rule_names.insert(NT_E, "e".into());
    g.rules.insert(
        NT_E,
        vec![
            rule(NT_E, vec![Symbol::Terminal(T_NUM)], 0),
            rule(
                NT_E,
                vec![Symbol::Terminal(T_PLUS), Symbol::NonTerminal(NT_E)],
                1,
            ),
            rule(
                NT_E,
                vec![Symbol::Terminal(T_STAR), Symbol::NonTerminal(NT_E)],
                2,
            ),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, NT_E, &[T_NUM, T_PLUS, T_STAR]);
}

// =========================================================================
// Category 2: Non-terminal FIRST set contains first terminal of rule
// =========================================================================

/// S → a b — FIRST(S) = {a}.
#[test]
fn cat02_nt_first_is_leading_terminal() {
    let mut g = Grammar::new("ffp_v8_c02a".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::Terminal(T_A), Symbol::Terminal(T_B)],
            0,
        )],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
}

/// S → A, A → b — FIRST(S) inherits {b} from A.
#[test]
fn cat02_nt_first_through_nonterminal() {
    let mut g = Grammar::new("ffp_v8_c02b".into());
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0)]);
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_B)], 1)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_B]);
    assert_first_eq(&ff, NT_A, &[T_B]);
}

/// S → A c, A → ε | a — FIRST(S) = {a, c} since A is nullable.
#[test]
fn cat02_nt_first_skips_nullable_prefix() {
    let mut g = Grammar::new("ffp_v8_c02c".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_C)],
            0,
        )],
    );
    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::Epsilon], 1),
            rule(NT_A, vec![Symbol::Terminal(T_A)], 2),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, NT_S, &[T_A, T_C]);
}

/// S → A B c, A → ε, B → ε | b — FIRST(S) = {b, c}.
#[test]
fn cat02_nt_first_skips_two_nullable_prefixes() {
    let mut g = Grammar::new("ffp_v8_c02d".into());
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rule_names.insert(NT_B, "b_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![
                Symbol::NonTerminal(NT_A),
                Symbol::NonTerminal(NT_B),
                Symbol::Terminal(T_C),
            ],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Epsilon], 1)]);
    g.rules.insert(
        NT_B,
        vec![
            rule(NT_B, vec![Symbol::Epsilon], 2),
            rule(NT_B, vec![Symbol::Terminal(T_B)], 3),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, NT_S, &[T_B, T_C]);
}

// =========================================================================
// Category 3: FOLLOW(start) contains EOF
// =========================================================================

/// Minimal grammar: S → a. FOLLOW(S) ⊇ {EOF}.
#[test]
fn cat03_follow_start_contains_eof() {
    let mut g = Grammar::new("ffp_v8_c03a".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_S, &[EOF]);
}

/// Larger grammar: S → A B. FOLLOW(S) still contains EOF.
#[test]
fn cat03_follow_start_eof_larger_grammar() {
    let mut g = Grammar::new("ffp_v8_c03b".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rule_names.insert(NT_B, "b_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_B)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_B)], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_S, &[EOF]);
}

/// Recursive grammar: S → a S | a. FOLLOW(S) contains EOF.
#[test]
fn cat03_follow_start_eof_recursive() {
    let mut g = Grammar::new("ffp_v8_c03c".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(
                NT_S,
                vec![Symbol::Terminal(T_A), Symbol::NonTerminal(NT_S)],
                0,
            ),
            rule(NT_S, vec![Symbol::Terminal(T_A)], 1),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_S, &[EOF]);
}

/// Nullable start symbol: S → ε. FOLLOW(S) contains EOF.
#[test]
fn cat03_follow_start_eof_nullable_start() {
    let mut g = Grammar::new("ffp_v8_c03d".into());
    g.rule_names.insert(NT_S, "s".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Epsilon], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_S, &[EOF]);
}

// =========================================================================
// Category 4: Nullable detection for epsilon rules
// =========================================================================

/// A → ε is nullable.
#[test]
fn cat04_epsilon_rule_is_nullable() {
    let mut g = Grammar::new("ffp_v8_c04a".into());
    g.rule_names.insert(NT_S, "s".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Epsilon], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_S));
}

/// S → A, A → ε — nullable propagates through chain.
#[test]
fn cat04_nullable_propagates_through_chain() {
    let mut g = Grammar::new("ffp_v8_c04b".into());
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0)]);
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Epsilon], 1)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
    assert!(ff.is_nullable(NT_S));
}

/// S → A B, A → ε, B → ε — both nullable makes S nullable.
#[test]
fn cat04_all_nullable_makes_sequence_nullable() {
    let mut g = Grammar::new("ffp_v8_c04c".into());
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rule_names.insert(NT_B, "b_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_B)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Epsilon], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Epsilon], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
    assert!(ff.is_nullable(NT_B));
    assert!(ff.is_nullable(NT_S));
}

/// S → A | ε, A → a — S is nullable via epsilon alternative.
#[test]
fn cat04_nullable_via_epsilon_alternative() {
    let mut g = Grammar::new("ffp_v8_c04d".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0),
            rule(NT_S, vec![Symbol::Epsilon], 1),
        ],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_S));
    assert!(!ff.is_nullable(NT_A));
}

// =========================================================================
// Category 5: Non-nullable for terminal-only rules
// =========================================================================

/// S → a — single terminal is not nullable.
#[test]
fn cat05_single_terminal_not_nullable() {
    let mut g = Grammar::new("ffp_v8_c05a".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(!ff.is_nullable(NT_S));
}

/// S → a b c — multi-terminal not nullable.
#[test]
fn cat05_multi_terminal_not_nullable() {
    let mut g = Grammar::new("ffp_v8_c05b".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![rule(
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
    assert!(!ff.is_nullable(NT_S));
}

/// S → a | b — all-terminal alternatives are not nullable.
#[test]
fn cat05_terminal_alternatives_not_nullable() {
    let mut g = Grammar::new("ffp_v8_c05c".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            rule(NT_S, vec![Symbol::Terminal(T_B)], 1),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(!ff.is_nullable(NT_S));
}

/// Terminals themselves are never nullable.
#[test]
fn cat05_terminal_itself_not_nullable() {
    let mut g = Grammar::new("ffp_v8_c05d".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(!ff.is_nullable(T_A));
}

// =========================================================================
// Category 6: FIRST() returns non-empty for any reachable symbol
// =========================================================================

/// All non-terminals in a connected grammar have non-empty FIRST.
#[test]
fn cat06_all_reachable_nts_nonempty_first() {
    // S → A B, A → a, B → b
    let mut g = Grammar::new("ffp_v8_c06a".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rule_names.insert(NT_B, "b_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_B)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_B)], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    for nt in [NT_S, NT_A, NT_B] {
        assert!(
            first_count(&ff, nt) > 0,
            "FIRST({nt:?}) should be non-empty"
        );
    }
}

/// Deep chain: all non-terminals have non-empty FIRST.
#[test]
fn cat06_deep_chain_nonempty_first() {
    // S → A, A → B, B → C, C → a
    let mut g = Grammar::new("ffp_v8_c06b".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rule_names.insert(NT_B, "b_nt".into());
    g.rule_names.insert(NT_C, "c_nt".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0)]);
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::NonTerminal(NT_B)], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::NonTerminal(NT_C)], 2)]);
    g.rules
        .insert(NT_C, vec![rule(NT_C, vec![Symbol::Terminal(T_A)], 3)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    for nt in [NT_S, NT_A, NT_B, NT_C] {
        assert!(
            first_count(&ff, nt) > 0,
            "FIRST({nt:?}) should be non-empty"
        );
    }
}

/// Terminals registered in the grammar have FIRST sets available.
#[test]
fn cat06_terminals_have_first_sets() {
    let mut g = Grammar::new("ffp_v8_c06c".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    tok(&mut g, T_D, "d", "d");
    g.rule_names.insert(NT_S, "s".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    // All registered terminals should have a FIRST set (even if empty).
    for t in [T_A, T_B, T_C, T_D] {
        assert!(ff.first(t).is_some(), "FIRST({t:?}) should exist");
    }
}

/// Nullable non-terminal still has non-empty FIRST if it also has a terminal alternative.
#[test]
fn cat06_nullable_with_terminal_nonempty_first() {
    let mut g = Grammar::new("ffp_v8_c06d".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::Epsilon], 0),
            rule(NT_S, vec![Symbol::Terminal(T_A)], 1),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(first_count(&ff, NT_S) > 0);
}

// =========================================================================
// Category 7: FOLLOW() returns non-empty for any referenced non-terminal
// =========================================================================

/// A appears in S → A b, so FOLLOW(A) is non-empty.
#[test]
fn cat07_referenced_nt_nonempty_follow() {
    let mut g = Grammar::new("ffp_v8_c07a".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_B)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(follow_count(&ff, NT_A) > 0, "FOLLOW(A) should be non-empty");
}

/// S is a start symbol, so FOLLOW(S) is non-empty (contains EOF).
#[test]
fn cat07_start_symbol_nonempty_follow() {
    let mut g = Grammar::new("ffp_v8_c07b".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(follow_count(&ff, NT_S) > 0, "FOLLOW(S) should be non-empty");
}

/// Multiple referenced non-terminals all have non-empty FOLLOW.
#[test]
fn cat07_all_referenced_nts_nonempty_follow() {
    // S → A B, A → a, B → b
    let mut g = Grammar::new("ffp_v8_c07c".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rule_names.insert(NT_B, "b_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_B)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_B)], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    // A has {b} in FOLLOW (B follows A in S), B has EOF in FOLLOW (end of S)
    assert!(follow_count(&ff, NT_A) > 0);
    assert!(follow_count(&ff, NT_B) > 0);
}

/// Tail position: S → a A. FOLLOW(A) ⊇ FOLLOW(S) ⊇ {EOF}.
#[test]
fn cat07_tail_position_follow_propagation() {
    let mut g = Grammar::new("ffp_v8_c07d".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::Terminal(T_A), Symbol::NonTerminal(NT_A)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_B)], 1)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[EOF]);
}

// =========================================================================
// Category 8: FIRST sets don't contain EOF (unless nullable)
// =========================================================================

/// Non-nullable non-terminal: FIRST should not contain EOF.
#[test]
fn cat08_first_no_eof_for_non_nullable() {
    let mut g = Grammar::new("ffp_v8_c08a".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_not_contains(&ff, NT_S, EOF);
}

/// Non-nullable chain: FIRST should not contain EOF.
#[test]
fn cat08_first_no_eof_chain() {
    let mut g = Grammar::new("ffp_v8_c08b".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0)]);
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_not_contains(&ff, NT_S, EOF);
    assert_first_not_contains(&ff, NT_A, EOF);
}

/// Terminal FIRST set never contains EOF.
#[test]
fn cat08_terminal_first_no_eof() {
    let mut g = Grammar::new("ffp_v8_c08c".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_not_contains(&ff, T_A, EOF);
    assert_first_not_contains(&ff, T_B, EOF);
}

/// Multiple alternatives, all terminal: no EOF in FIRST.
#[test]
fn cat08_alternatives_first_no_eof() {
    let mut g = Grammar::new("ffp_v8_c08d".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            rule(NT_S, vec![Symbol::Terminal(T_B)], 1),
            rule(NT_S, vec![Symbol::Terminal(T_C)], 2),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_not_contains(&ff, NT_S, EOF);
}

// =========================================================================
// Category 9: FOLLOW sets can contain EOF
// =========================================================================

/// Start symbol FOLLOW always contains EOF.
#[test]
fn cat09_follow_can_contain_eof_start() {
    let mut g = Grammar::new("ffp_v8_c09a".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    let set = ff.follow(NT_S).unwrap();
    assert!(set.contains(EOF.0 as usize));
}

/// Tail non-terminal inherits EOF from parent.
#[test]
fn cat09_follow_eof_propagates_to_tail() {
    let mut g = Grammar::new("ffp_v8_c09b".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0)]);
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[EOF]);
}

/// Chain of tail positions: EOF propagates through all.
#[test]
fn cat09_follow_eof_propagates_through_chain() {
    // S → A, A → B, B → c
    let mut g = Grammar::new("ffp_v8_c09c".into());
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rule_names.insert(NT_B, "b_nt".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0)]);
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::NonTerminal(NT_B)], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_C)], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_S, &[EOF]);
    assert_follow_contains(&ff, NT_A, &[EOF]);
    assert_follow_contains(&ff, NT_B, &[EOF]);
}

/// FOLLOW can contain both EOF and terminals.
#[test]
fn cat09_follow_contains_eof_and_terminals() {
    // S → A b | A. FOLLOW(A) = {b, EOF}
    let mut g = Grammar::new("ffp_v8_c09d".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(
                NT_S,
                vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_B)],
                0,
            ),
            rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 1),
        ],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[T_B, EOF]);
}

// =========================================================================
// Category 10: Single rule grammar FIRST/FOLLOW
// =========================================================================

/// Simplest grammar: S → a.
#[test]
fn cat10_single_rule_simple() {
    let mut g = Grammar::new("ffp_v8_c10a".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert_follow_contains(&ff, NT_S, &[EOF]);
    assert!(!ff.is_nullable(NT_S));
}

/// Single rule: S → a b.
#[test]
fn cat10_single_rule_two_terminals() {
    let mut g = Grammar::new("ffp_v8_c10b".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::Terminal(T_A), Symbol::Terminal(T_B)],
            0,
        )],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert_follow_contains(&ff, NT_S, &[EOF]);
}

/// Single epsilon rule: S → ε.
#[test]
fn cat10_single_epsilon_rule() {
    let mut g = Grammar::new("ffp_v8_c10c".into());
    g.rule_names.insert(NT_S, "s".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Epsilon], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_S));
    assert_follow_contains(&ff, NT_S, &[EOF]);
}

/// Single non-terminal rule: S → A, A → a.
#[test]
fn cat10_single_nt_rule() {
    let mut g = Grammar::new("ffp_v8_c10d".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0)]);
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert_first_eq(&ff, NT_A, &[T_A]);
}

// =========================================================================
// Category 11: Grammar with alternatives → union of FIRST sets
// =========================================================================

/// S → a | b — FIRST(S) = {a, b}.
#[test]
fn cat11_alternatives_union_two() {
    let mut g = Grammar::new("ffp_v8_c11a".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            rule(NT_S, vec![Symbol::Terminal(T_B)], 1),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A, T_B]);
}

/// S → a | b | c — FIRST(S) = {a, b, c}.
#[test]
fn cat11_alternatives_union_three() {
    let mut g = Grammar::new("ffp_v8_c11b".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            rule(NT_S, vec![Symbol::Terminal(T_B)], 1),
            rule(NT_S, vec![Symbol::Terminal(T_C)], 2),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A, T_B, T_C]);
}

/// S → A | B, A → a, B → b — FIRST(S) = FIRST(A) ∪ FIRST(B).
#[test]
fn cat11_alternatives_union_through_nts() {
    let mut g = Grammar::new("ffp_v8_c11c".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rule_names.insert(NT_B, "b_nt".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0),
            rule(NT_S, vec![Symbol::NonTerminal(NT_B)], 1),
        ],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 2)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_B)], 3)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A, T_B]);
}

/// S → a | A, A → b | c — FIRST(S) = {a, b, c}.
#[test]
fn cat11_alternatives_mixed_terminal_and_nt() {
    let mut g = Grammar::new("ffp_v8_c11d".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 1),
        ],
    );
    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::Terminal(T_B)], 2),
            rule(NT_A, vec![Symbol::Terminal(T_C)], 3),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A, T_B, T_C]);
}

// =========================================================================
// Category 12: Chain rule propagation: A → B, B → c
// =========================================================================

/// Two-level chain: S → A, A → c.
#[test]
fn cat12_chain_two_level() {
    let mut g = Grammar::new("ffp_v8_c12a".into());
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0)]);
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_C)], 1)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_C]);
    assert_first_eq(&ff, NT_A, &[T_C]);
}

/// Three-level chain: S → A, A → B, B → c.
#[test]
fn cat12_chain_three_level() {
    let mut g = Grammar::new("ffp_v8_c12b".into());
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rule_names.insert(NT_B, "b_nt".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0)]);
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::NonTerminal(NT_B)], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_C)], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    for nt in [NT_S, NT_A, NT_B] {
        assert_first_eq(&ff, nt, &[T_C]);
    }
}

/// Four-level chain: S → A, A → B, B → C, C → d.
#[test]
fn cat12_chain_four_level() {
    let mut g = Grammar::new("ffp_v8_c12c".into());
    tok(&mut g, T_D, "d", "d");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rule_names.insert(NT_B, "b_nt".into());
    g.rule_names.insert(NT_C, "c_nt".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0)]);
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::NonTerminal(NT_B)], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::NonTerminal(NT_C)], 2)]);
    g.rules
        .insert(NT_C, vec![rule(NT_C, vec![Symbol::Terminal(T_D)], 3)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    for nt in [NT_S, NT_A, NT_B, NT_C] {
        assert_first_eq(&ff, nt, &[T_D]);
    }
}

/// Chain with FOLLOW propagation: S → A b, A → B, B → c.
/// FOLLOW(B) ⊇ FOLLOW(A) ⊇ {b}.
#[test]
fn cat12_chain_follow_propagation() {
    let mut g = Grammar::new("ffp_v8_c12d".into());
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rule_names.insert(NT_B, "b_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_B)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::NonTerminal(NT_B)], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_C)], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[T_B]);
    assert_follow_contains(&ff, NT_B, &[T_B]);
}

// =========================================================================
// Category 13: Multiple tokens → each terminal's FIRST is {self}
// =========================================================================

/// Five tokens: each appears in the FIRST of an NT that starts with it.
#[test]
fn cat13_five_tokens_each_in_nt_first() {
    let mut g = Grammar::new("ffp_v8_c13a".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    tok(&mut g, T_D, "d", "d");
    tok(&mut g, T_PLUS, "plus", "+");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            rule(NT_S, vec![Symbol::Terminal(T_B)], 1),
            rule(NT_S, vec![Symbol::Terminal(T_C)], 2),
            rule(NT_S, vec![Symbol::Terminal(T_D)], 3),
            rule(NT_S, vec![Symbol::Terminal(T_PLUS)], 4),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A, T_B, T_C, T_D, T_PLUS]);
}

/// Operator tokens all appear in the FIRST of an NT with those alternatives.
#[test]
fn cat13_operator_tokens_in_nt_first() {
    let mut g = Grammar::new("ffp_v8_c13b".into());
    tok(&mut g, T_PLUS, "plus", "+");
    tok(&mut g, T_STAR, "star", "*");
    tok(&mut g, T_LPAREN, "lparen", "(");
    tok(&mut g, T_RPAREN, "rparen", ")");
    tok(&mut g, T_NUM, "num", "0");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::Terminal(T_NUM)], 0),
            rule(NT_S, vec![Symbol::Terminal(T_PLUS)], 1),
            rule(NT_S, vec![Symbol::Terminal(T_STAR)], 2),
            rule(NT_S, vec![Symbol::Terminal(T_LPAREN)], 3),
            rule(NT_S, vec![Symbol::Terminal(T_RPAREN)], 4),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_PLUS, T_STAR, T_LPAREN, T_RPAREN, T_NUM]);
}

/// Tokens used in different rules are tracked in their respective NTs' FIRST.
#[test]
fn cat13_tokens_tracked_across_rules() {
    let mut g = Grammar::new("ffp_v8_c13c".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::Terminal(T_A), Symbol::NonTerminal(NT_A)],
            0,
        )],
    );
    g.rules.insert(
        NT_A,
        vec![rule(
            NT_A,
            vec![Symbol::Terminal(T_B), Symbol::Terminal(T_C)],
            1,
        )],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert_first_eq(&ff, NT_A, &[T_B]);
}

/// Single token grammar: terminal appears as FIRST of its NT.
#[test]
fn cat13_single_token_in_nt_first() {
    let mut g = Grammar::new("ffp_v8_c13d".into());
    tok(&mut g, T_NUM, "num", "0");
    g.rule_names.insert(NT_S, "s".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_NUM)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_NUM]);
}

// =========================================================================
// Category 14: Precedence doesn't affect FIRST/FOLLOW computation
// =========================================================================

/// Same grammar with and without precedence yields same FIRST.
#[test]
fn cat14_prec_no_effect_on_first() {
    // Without precedence
    let mut g1 = Grammar::new("ffp_v8_c14a_no".into());
    tok(&mut g1, T_NUM, "num", "0");
    tok(&mut g1, T_PLUS, "plus", "+");
    g1.rule_names.insert(NT_E, "e".into());
    g1.rules.insert(
        NT_E,
        vec![
            rule(NT_E, vec![Symbol::Terminal(T_NUM)], 0),
            rule(
                NT_E,
                vec![
                    Symbol::NonTerminal(NT_E),
                    Symbol::Terminal(T_PLUS),
                    Symbol::NonTerminal(NT_E),
                ],
                1,
            ),
        ],
    );

    // With precedence
    let mut g2 = Grammar::new("ffp_v8_c14a_pr".into());
    tok(&mut g2, T_NUM, "num", "0");
    tok(&mut g2, T_PLUS, "plus", "+");
    g2.rule_names.insert(NT_E, "e".into());
    g2.rules.insert(
        NT_E,
        vec![
            rule(NT_E, vec![Symbol::Terminal(T_NUM)], 0),
            rule_prec(
                NT_E,
                vec![
                    Symbol::NonTerminal(NT_E),
                    Symbol::Terminal(T_PLUS),
                    Symbol::NonTerminal(NT_E),
                ],
                1,
                10,
                None,
            ),
        ],
    );

    let ff1 = FirstFollowSets::compute(&g1).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();

    assert_first_eq(&ff1, NT_E, &[T_NUM]);
    assert_first_eq(&ff2, NT_E, &[T_NUM]);
}

/// Precedence on multiple rules does not alter FIRST sets.
#[test]
fn cat14_prec_multi_rule_no_effect() {
    let mut g = Grammar::new("ffp_v8_c14b".into());
    tok(&mut g, T_NUM, "num", "0");
    tok(&mut g, T_PLUS, "plus", "+");
    tok(&mut g, T_STAR, "star", "*");
    g.rule_names.insert(NT_E, "e".into());
    g.rules.insert(
        NT_E,
        vec![
            rule(NT_E, vec![Symbol::Terminal(T_NUM)], 0),
            rule_prec(
                NT_E,
                vec![
                    Symbol::NonTerminal(NT_E),
                    Symbol::Terminal(T_PLUS),
                    Symbol::NonTerminal(NT_E),
                ],
                1,
                1,
                None,
            ),
            rule_prec(
                NT_E,
                vec![
                    Symbol::NonTerminal(NT_E),
                    Symbol::Terminal(T_STAR),
                    Symbol::NonTerminal(NT_E),
                ],
                2,
                2,
                None,
            ),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_E, &[T_NUM]);
}

/// Precedence doesn't change FOLLOW sets either.
#[test]
fn cat14_prec_no_effect_on_follow() {
    let mut g1 = Grammar::new("ffp_v8_c14c_no".into());
    tok(&mut g1, T_A, "a", "a");
    tok(&mut g1, T_B, "b", "b");
    g1.rule_names.insert(NT_S, "s".into());
    g1.rule_names.insert(NT_A, "a_nt".into());
    g1.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_B)],
            0,
        )],
    );
    g1.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);

    let mut g2 = Grammar::new("ffp_v8_c14c_pr".into());
    tok(&mut g2, T_A, "a", "a");
    tok(&mut g2, T_B, "b", "b");
    g2.rule_names.insert(NT_S, "s".into());
    g2.rule_names.insert(NT_A, "a_nt".into());
    g2.rules.insert(
        NT_S,
        vec![rule_prec(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_B)],
            0,
            5,
            None,
        )],
    );
    g2.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);

    let ff1 = FirstFollowSets::compute(&g1).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();

    assert_follow_contains(&ff1, NT_A, &[T_B]);
    assert_follow_contains(&ff2, NT_A, &[T_B]);
}

/// High precedence values have no effect.
#[test]
fn cat14_high_prec_no_effect() {
    let mut g = Grammar::new("ffp_v8_c14d".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![rule_prec(NT_S, vec![Symbol::Terminal(T_A)], 0, 1000, None)],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert_follow_contains(&ff, NT_S, &[EOF]);
}

// =========================================================================
// Category 15: Associativity doesn't affect FIRST/FOLLOW
// =========================================================================

/// Left, right, and no associativity all produce the same FIRST.
#[test]
fn cat15_assoc_left_same_first() {
    let mut g = Grammar::new("ffp_v8_c15a".into());
    tok(&mut g, T_NUM, "num", "0");
    tok(&mut g, T_PLUS, "plus", "+");
    g.rule_names.insert(NT_E, "e".into());
    g.rules.insert(
        NT_E,
        vec![
            rule(NT_E, vec![Symbol::Terminal(T_NUM)], 0),
            rule_prec(
                NT_E,
                vec![
                    Symbol::NonTerminal(NT_E),
                    Symbol::Terminal(T_PLUS),
                    Symbol::NonTerminal(NT_E),
                ],
                1,
                1,
                Some(Associativity::Left),
            ),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_E, &[T_NUM]);
}

/// Right associativity: same FIRST.
#[test]
fn cat15_assoc_right_same_first() {
    let mut g = Grammar::new("ffp_v8_c15b".into());
    tok(&mut g, T_NUM, "num", "0");
    tok(&mut g, T_PLUS, "plus", "+");
    g.rule_names.insert(NT_E, "e".into());
    g.rules.insert(
        NT_E,
        vec![
            rule(NT_E, vec![Symbol::Terminal(T_NUM)], 0),
            rule_prec(
                NT_E,
                vec![
                    Symbol::NonTerminal(NT_E),
                    Symbol::Terminal(T_PLUS),
                    Symbol::NonTerminal(NT_E),
                ],
                1,
                1,
                Some(Associativity::Right),
            ),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_E, &[T_NUM]);
}

/// Comparing left vs right vs none — all produce identical FIRST/FOLLOW.
#[test]
fn cat15_assoc_comparison_all_same() {
    let assocs = [None, Some(Associativity::Left), Some(Associativity::Right)];
    let mut results = Vec::new();

    for (i, assoc) in assocs.iter().enumerate() {
        let name = format!("ffp_v8_c15c_{i}");
        let mut g = Grammar::new(name);
        tok(&mut g, T_NUM, "num", "0");
        tok(&mut g, T_PLUS, "plus", "+");
        g.rule_names.insert(NT_E, "e".into());
        g.rules.insert(
            NT_E,
            vec![
                rule(NT_E, vec![Symbol::Terminal(T_NUM)], 0),
                rule_prec(
                    NT_E,
                    vec![
                        Symbol::NonTerminal(NT_E),
                        Symbol::Terminal(T_PLUS),
                        Symbol::NonTerminal(NT_E),
                    ],
                    1,
                    1,
                    *assoc,
                ),
            ],
        );

        let ff = FirstFollowSets::compute(&g).unwrap();
        let first_bits: Vec<u16> = {
            let s = ff.first(NT_E).unwrap();
            (0..s.len())
                .filter(|&i| s.contains(i))
                .map(|i| i as u16)
                .collect()
        };
        results.push(first_bits);
    }

    assert_eq!(results[0], results[1]);
    assert_eq!(results[1], results[2]);
}

/// Associativity on simple rules has no effect.
#[test]
fn cat15_assoc_simple_rule_no_effect() {
    let mut g = Grammar::new("ffp_v8_c15d".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![rule_prec(
            NT_S,
            vec![Symbol::Terminal(T_A)],
            0,
            1,
            Some(Associativity::Left),
        )],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert_follow_contains(&ff, NT_S, &[EOF]);
}

// =========================================================================
// Category 16: Determinism — same grammar → same sets
// =========================================================================

/// Computing twice yields identical FIRST sets.
#[test]
fn cat16_determinism_first_identical() {
    let mut g = Grammar::new("ffp_v8_c16a".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 1),
        ],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_B)], 2)]);

    let ff1 = FirstFollowSets::compute(&g).unwrap();
    let ff2 = FirstFollowSets::compute(&g).unwrap();

    for sym in [NT_S, NT_A, T_A, T_B] {
        let s1 = ff1.first(sym).unwrap();
        let s2 = ff2.first(sym).unwrap();
        assert_eq!(s1, s2, "FIRST({sym:?}) should be deterministic");
    }
}

/// Computing twice yields identical FOLLOW sets.
#[test]
fn cat16_determinism_follow_identical() {
    let mut g = Grammar::new("ffp_v8_c16b".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_B)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);

    let ff1 = FirstFollowSets::compute(&g).unwrap();
    let ff2 = FirstFollowSets::compute(&g).unwrap();

    for sym in [NT_S, NT_A] {
        let s1 = ff1.follow(sym).unwrap();
        let s2 = ff2.follow(sym).unwrap();
        assert_eq!(s1, s2, "FOLLOW({sym:?}) should be deterministic");
    }
}

/// Computing twice yields identical nullable results.
#[test]
fn cat16_determinism_nullable_identical() {
    let mut g = Grammar::new("ffp_v8_c16c".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0),
            rule(NT_S, vec![Symbol::Epsilon], 1),
        ],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 2)]);

    let ff1 = FirstFollowSets::compute(&g).unwrap();
    let ff2 = FirstFollowSets::compute(&g).unwrap();

    assert_eq!(ff1.is_nullable(NT_S), ff2.is_nullable(NT_S));
    assert_eq!(ff1.is_nullable(NT_A), ff2.is_nullable(NT_A));
}

/// Ten computations all agree.
#[test]
fn cat16_determinism_ten_runs() {
    let mut g = Grammar::new("ffp_v8_c16d".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            rule(NT_S, vec![Symbol::Terminal(T_B)], 1),
            rule(NT_S, vec![Symbol::Terminal(T_C)], 2),
        ],
    );

    let baseline = FirstFollowSets::compute(&g).unwrap();
    let baseline_first = baseline.first(NT_S).unwrap().clone();

    for _ in 0..9 {
        let ff = FirstFollowSets::compute(&g).unwrap();
        assert_eq!(ff.first(NT_S).unwrap(), &baseline_first);
    }
}

// =========================================================================
// Category 17: Clone grammar → same FIRST/FOLLOW
// =========================================================================

/// Cloned grammar produces identical FIRST.
#[test]
fn cat17_clone_same_first() {
    let mut g = Grammar::new("ffp_v8_c17a".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            rule(NT_S, vec![Symbol::Terminal(T_B)], 1),
        ],
    );

    let g2 = g.clone();
    let ff1 = FirstFollowSets::compute(&g).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();

    assert_eq!(ff1.first(NT_S).unwrap(), ff2.first(NT_S).unwrap());
}

/// Cloned grammar produces identical FOLLOW.
#[test]
fn cat17_clone_same_follow() {
    let mut g = Grammar::new("ffp_v8_c17b".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_B)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);

    let g2 = g.clone();
    let ff1 = FirstFollowSets::compute(&g).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();

    assert_eq!(ff1.follow(NT_A).unwrap(), ff2.follow(NT_A).unwrap());
    assert_eq!(ff1.follow(NT_S).unwrap(), ff2.follow(NT_S).unwrap());
}

/// Cloned grammar produces identical nullable.
#[test]
fn cat17_clone_same_nullable() {
    let mut g = Grammar::new("ffp_v8_c17c".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::Epsilon], 0),
            rule(NT_S, vec![Symbol::Terminal(T_A)], 1),
        ],
    );

    let g2 = g.clone();
    let ff1 = FirstFollowSets::compute(&g).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();

    assert_eq!(ff1.is_nullable(NT_S), ff2.is_nullable(NT_S));
}

/// Clone of a complex grammar: entire FIRST/FOLLOW match.
#[test]
fn cat17_clone_complex_grammar() {
    // E → E + T | T, T → num
    let mut g = Grammar::new("ffp_v8_c17d".into());
    tok(&mut g, T_NUM, "num", "0");
    tok(&mut g, T_PLUS, "plus", "+");
    g.rule_names.insert(NT_E, "e".into());
    g.rule_names.insert(NT_T, "t".into());
    g.rules.insert(
        NT_E,
        vec![
            rule(
                NT_E,
                vec![
                    Symbol::NonTerminal(NT_E),
                    Symbol::Terminal(T_PLUS),
                    Symbol::NonTerminal(NT_T),
                ],
                0,
            ),
            rule(NT_E, vec![Symbol::NonTerminal(NT_T)], 1),
        ],
    );
    g.rules
        .insert(NT_T, vec![rule(NT_T, vec![Symbol::Terminal(T_NUM)], 2)]);

    let g2 = g.clone();
    let ff1 = FirstFollowSets::compute(&g).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();

    for sym in [NT_E, NT_T] {
        assert_eq!(ff1.first(sym).unwrap(), ff2.first(sym).unwrap());
        assert_eq!(ff1.follow(sym).unwrap(), ff2.follow(sym).unwrap());
        assert_eq!(ff1.is_nullable(sym), ff2.is_nullable(sym));
    }
}

// =========================================================================
// Category 18: Various grammar sizes
// =========================================================================

/// Minimal: 1 token, 1 rule.
#[test]
fn cat18_size_minimal() {
    let mut g = Grammar::new("ffp_v8_c18a".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
}

/// Small: 3 tokens, 3 non-terminals, 5 rules.
#[test]
fn cat18_size_small() {
    let mut g = Grammar::new("ffp_v8_c18b".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rule_names.insert(NT_B, "b_nt".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0),
            rule(NT_S, vec![Symbol::NonTerminal(NT_B)], 1),
        ],
    );
    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::Terminal(T_A)], 2),
            rule(NT_A, vec![Symbol::Terminal(T_B)], 3),
        ],
    );
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_C)], 4)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A, T_B, T_C]);
    assert_first_eq(&ff, NT_A, &[T_A, T_B]);
    assert_first_eq(&ff, NT_B, &[T_C]);
}

/// Medium: arithmetic expression grammar (E, T, F).
#[test]
fn cat18_size_medium_arithmetic() {
    // E → E + T | T
    // T → T * F | F
    // F → ( E ) | num
    let mut g = Grammar::new("ffp_v8_c18c".into());
    tok(&mut g, T_NUM, "num", "0");
    tok(&mut g, T_PLUS, "plus", "+");
    tok(&mut g, T_STAR, "star", "*");
    tok(&mut g, T_LPAREN, "lparen", "(");
    tok(&mut g, T_RPAREN, "rparen", ")");
    g.rule_names.insert(NT_E, "e".into());
    g.rule_names.insert(NT_T, "t".into());
    g.rule_names.insert(NT_F, "f".into());
    g.rules.insert(
        NT_E,
        vec![
            rule(
                NT_E,
                vec![
                    Symbol::NonTerminal(NT_E),
                    Symbol::Terminal(T_PLUS),
                    Symbol::NonTerminal(NT_T),
                ],
                0,
            ),
            rule(NT_E, vec![Symbol::NonTerminal(NT_T)], 1),
        ],
    );
    g.rules.insert(
        NT_T,
        vec![
            rule(
                NT_T,
                vec![
                    Symbol::NonTerminal(NT_T),
                    Symbol::Terminal(T_STAR),
                    Symbol::NonTerminal(NT_F),
                ],
                2,
            ),
            rule(NT_T, vec![Symbol::NonTerminal(NT_F)], 3),
        ],
    );
    g.rules.insert(
        NT_F,
        vec![
            rule(
                NT_F,
                vec![
                    Symbol::Terminal(T_LPAREN),
                    Symbol::NonTerminal(NT_E),
                    Symbol::Terminal(T_RPAREN),
                ],
                4,
            ),
            rule(NT_F, vec![Symbol::Terminal(T_NUM)], 5),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    // FIRST(E) = FIRST(T) = FIRST(F) = {(, num}
    for nt in [NT_E, NT_T, NT_F] {
        assert_first_contains(&ff, nt, &[T_LPAREN, T_NUM]);
    }
    // FOLLOW(E) ⊇ {), +, EOF}
    assert_follow_contains(&ff, NT_E, &[T_RPAREN, T_PLUS, EOF]);
    // FOLLOW(T) ⊇ {+, *, ), EOF}
    assert_follow_contains(&ff, NT_T, &[T_PLUS, T_STAR]);
    // FOLLOW(F) ⊇ {+, *, ), EOF}
    assert_follow_contains(&ff, NT_F, &[T_PLUS, T_STAR]);
}

/// Larger: 6 non-terminals, wide grammar.
#[test]
fn cat18_size_larger_six_nts() {
    let mut g = Grammar::new("ffp_v8_c18d".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    tok(&mut g, T_D, "d", "d");
    tok(&mut g, T_E, "e_tok", "e");
    tok(&mut g, T_F, "f_tok", "f");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rule_names.insert(NT_B, "b_nt".into());
    g.rule_names.insert(NT_C, "c_nt".into());
    g.rule_names.insert(NT_D, "d_nt".into());
    g.rule_names.insert(NT_E, "e_nt".into());
    // S → A | B | C
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0),
            rule(NT_S, vec![Symbol::NonTerminal(NT_B)], 1),
            rule(NT_S, vec![Symbol::NonTerminal(NT_C)], 2),
        ],
    );
    // A → a | D
    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::Terminal(T_A)], 3),
            rule(NT_A, vec![Symbol::NonTerminal(NT_D)], 4),
        ],
    );
    // B → b
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_B)], 5)]);
    // C → c | E
    g.rules.insert(
        NT_C,
        vec![
            rule(NT_C, vec![Symbol::Terminal(T_C)], 6),
            rule(NT_C, vec![Symbol::NonTerminal(NT_E)], 7),
        ],
    );
    // D → d
    g.rules
        .insert(NT_D, vec![rule(NT_D, vec![Symbol::Terminal(T_D)], 8)]);
    // E → e
    g.rules
        .insert(NT_E, vec![rule(NT_E, vec![Symbol::Terminal(T_E)], 9)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_contains(&ff, NT_S, &[T_A, T_B, T_C, T_D, T_E]);
    assert_first_eq(&ff, NT_D, &[T_D]);
    assert_first_eq(&ff, NT_E, &[T_E]);
}

// =========================================================================
// Category 19: Grammar with extras → FIRST/FOLLOW still valid
// =========================================================================

/// Grammar with whitespace extra token.
#[test]
fn cat19_extras_whitespace() {
    let mut g = Grammar::new("ffp_v8_c19a".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_WS, "ws", " ");
    g.rule_names.insert(NT_S, "s".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);
    g.extras.push(T_WS);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert_follow_contains(&ff, NT_S, &[EOF]);
}

/// Grammar with comment extra token.
#[test]
fn cat19_extras_comment() {
    let mut g = Grammar::new("ffp_v8_c19b".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_COMMENT, "comment", "//");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_B)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    g.extras.push(T_COMMENT);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert_follow_contains(&ff, NT_A, &[T_B]);
}

/// Grammar with multiple extras.
#[test]
fn cat19_extras_multiple() {
    let mut g = Grammar::new("ffp_v8_c19c".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_WS, "ws", " ");
    tok(&mut g, T_COMMENT, "comment", "//");
    g.rule_names.insert(NT_S, "s".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);
    g.extras.push(T_WS);
    g.extras.push(T_COMMENT);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert!(!ff.is_nullable(NT_S));
}

/// Extras don't pollute FIRST sets of non-terminals.
#[test]
fn cat19_extras_dont_pollute_first() {
    let mut g = Grammar::new("ffp_v8_c19d".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_WS, "ws", " ");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            rule(NT_S, vec![Symbol::Terminal(T_B)], 1),
        ],
    );
    g.extras.push(T_WS);

    let ff = FirstFollowSets::compute(&g).unwrap();
    // FIRST(S) should only contain {a, b}, not ws
    assert_first_eq(&ff, NT_S, &[T_A, T_B]);
}

// =========================================================================
// Category 20: Grammar with inline rules → FIRST/FOLLOW still valid
// =========================================================================

/// Inline non-terminal computes FIRST normally.
#[test]
fn cat20_inline_first_normal() {
    let mut g = Grammar::new("ffp_v8_c20a".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_B)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    g.inline_rules.push(NT_A);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert_first_eq(&ff, NT_A, &[T_A]);
}

/// Inline non-terminal FOLLOW still computed.
#[test]
fn cat20_inline_follow_computed() {
    let mut g = Grammar::new("ffp_v8_c20b".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_B)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    g.inline_rules.push(NT_A);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[T_B]);
}

/// Multiple inline non-terminals.
#[test]
fn cat20_inline_multiple() {
    let mut g = Grammar::new("ffp_v8_c20c".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rule_names.insert(NT_B, "b_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![
                Symbol::NonTerminal(NT_A),
                Symbol::NonTerminal(NT_B),
                Symbol::Terminal(T_C),
            ],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_B)], 2)]);
    g.inline_rules.push(NT_A);
    g.inline_rules.push(NT_B);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert_first_eq(&ff, NT_A, &[T_A]);
    assert_first_eq(&ff, NT_B, &[T_B]);
}

/// Inline with alternatives.
#[test]
fn cat20_inline_with_alternatives() {
    let mut g = Grammar::new("ffp_v8_c20d".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0)]);
    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::Terminal(T_A)], 1),
            rule(NT_A, vec![Symbol::Terminal(T_B)], 2),
            rule(NT_A, vec![Symbol::Terminal(T_C)], 3),
        ],
    );
    g.inline_rules.push(NT_A);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A, T_B, T_C]);
}

// =========================================================================
// Additional cross-cutting invariant tests
// =========================================================================

/// Left recursion: S → S a | a — FIRST(S) = {a}.
#[test]
fn cross_left_recursion_first() {
    let mut g = Grammar::new("ffp_v8_xlr".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(
                NT_S,
                vec![Symbol::NonTerminal(NT_S), Symbol::Terminal(T_A)],
                0,
            ),
            rule(NT_S, vec![Symbol::Terminal(T_A)], 1),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert!(!ff.is_nullable(NT_S));
}

/// Right recursion: S → a S | a — FIRST(S) = {a}.
#[test]
fn cross_right_recursion_first() {
    let mut g = Grammar::new("ffp_v8_xrr".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(
                NT_S,
                vec![Symbol::Terminal(T_A), Symbol::NonTerminal(NT_S)],
                0,
            ),
            rule(NT_S, vec![Symbol::Terminal(T_A)], 1),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
}

/// Mutual recursion: A → B a, B → A b | c — FIRST(A) = FIRST(B) = {c}.
#[test]
fn cross_mutual_recursion() {
    let mut g = Grammar::new("ffp_v8_xmr".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rule_names.insert(NT_B, "b_nt".into());
    g.rules.insert(
        NT_A,
        vec![rule(
            NT_A,
            vec![Symbol::NonTerminal(NT_B), Symbol::Terminal(T_A)],
            0,
        )],
    );
    g.rules.insert(
        NT_B,
        vec![
            rule(
                NT_B,
                vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_B)],
                1,
            ),
            rule(NT_B, vec![Symbol::Terminal(T_C)], 2),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_A, &[T_C]);
    assert_first_eq(&ff, NT_B, &[T_C]);
}

/// FOLLOW propagation: S → A B, A → a, B → b. FOLLOW(A) ⊇ {b} (FIRST(B)).
#[test]
fn cross_follow_from_first_of_successor() {
    let mut g = Grammar::new("ffp_v8_xfs".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rule_names.insert(NT_B, "b_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_B)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_B)], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[T_B]);
}

/// FOLLOW through nullable successor: S → A B c, B → ε. FOLLOW(A) ⊇ {c}.
#[test]
fn cross_follow_through_nullable_successor() {
    let mut g = Grammar::new("ffp_v8_xfn".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rule_names.insert(NT_B, "b_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![
                Symbol::NonTerminal(NT_A),
                Symbol::NonTerminal(NT_B),
                Symbol::Terminal(T_C),
            ],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Epsilon], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[T_C]);
}

/// Semicolon-separated statements: S → A ; A | A, A → a.
/// FOLLOW(A) ⊇ {;, EOF}.
#[test]
fn cross_semicolon_separated() {
    let mut g = Grammar::new("ffp_v8_xss".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_SEMI, "semi", ";");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(
                NT_S,
                vec![
                    Symbol::NonTerminal(NT_A),
                    Symbol::Terminal(T_SEMI),
                    Symbol::NonTerminal(NT_A),
                ],
                0,
            ),
            rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 1),
        ],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[T_SEMI, EOF]);
}

/// Parenthesized group: S → ( A ), A → a. FOLLOW(A) ⊇ {)}.
#[test]
fn cross_parenthesized_follow() {
    let mut g = Grammar::new("ffp_v8_xpf".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_LPAREN, "lparen", "(");
    tok(&mut g, T_RPAREN, "rparen", ")");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![
                Symbol::Terminal(T_LPAREN),
                Symbol::NonTerminal(NT_A),
                Symbol::Terminal(T_RPAREN),
            ],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[T_RPAREN]);
    assert_first_eq(&ff, NT_S, &[T_LPAREN]);
}

/// All-nullable sequence: S → A B C, A → ε, B → ε, C → ε. S is nullable.
#[test]
fn cross_all_nullable_sequence() {
    let mut g = Grammar::new("ffp_v8_xan".into());
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rule_names.insert(NT_B, "b_nt".into());
    g.rule_names.insert(NT_C, "c_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
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
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Epsilon], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Epsilon], 2)]);
    g.rules
        .insert(NT_C, vec![rule(NT_C, vec![Symbol::Epsilon], 3)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_S));
    assert!(ff.is_nullable(NT_A));
    assert!(ff.is_nullable(NT_B));
    assert!(ff.is_nullable(NT_C));
}

/// Non-nullable in sequence prevents parent from being nullable.
/// S → A B, A → ε, B → b. S is NOT nullable.
#[test]
fn cross_partial_nullable_not_nullable() {
    let mut g = Grammar::new("ffp_v8_xpn".into());
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "s".into());
    g.rule_names.insert(NT_A, "a_nt".into());
    g.rule_names.insert(NT_B, "b_nt".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_B)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Epsilon], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_B)], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
    assert!(!ff.is_nullable(NT_B));
    assert!(!ff.is_nullable(NT_S));
}

/// FIRST set size invariant: |FIRST(NT)| ≤ total terminal count.
#[test]
fn cross_first_size_bounded_by_terminals() {
    let mut g = Grammar::new("ffp_v8_xfb".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "s".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            rule(NT_S, vec![Symbol::Terminal(T_B)], 1),
            rule(NT_S, vec![Symbol::Terminal(T_C)], 2),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    let terminal_count = g.tokens.len();
    assert!(first_count(&ff, NT_S) <= terminal_count);
}

/// FOLLOW set: start symbol should have exactly EOF when no self-referencing.
#[test]
fn cross_start_follow_only_eof_simple() {
    let mut g = Grammar::new("ffp_v8_xso".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "s".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_eq(&ff, NT_S, &[EOF]);
}
