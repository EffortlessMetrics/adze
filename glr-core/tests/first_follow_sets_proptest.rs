#![allow(clippy::needless_range_loop)]
//! Property-style tests for `FirstFollowSets` public API in adze-glr-core.
//!
//! Categories:
//! 1. First set of terminal is itself
//! 2. First set of non-terminal includes its productions' first terminals
//! 3. Follow set of start symbol contains EOF
//! 4. First/follow determinism (same grammar → same sets)
//! 5. First set nullable propagation (epsilon rules)
//! 6. Follow set propagation through production rules
//! 7. First set with choice/alternation
//! 8. Empty grammar handling

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
const T_E: SymbolId = SymbolId(5);
const T_F: SymbolId = SymbolId(6);
const T_G: SymbolId = SymbolId(7);

const NT_S: SymbolId = SymbolId(30);
const NT_A: SymbolId = SymbolId(31);
const NT_B: SymbolId = SymbolId(32);
const NT_C: SymbolId = SymbolId(33);
const NT_D: SymbolId = SymbolId(34);
const NT_E: SymbolId = SymbolId(35);

// =========================================================================
// Category 1: First set of terminal is itself
// =========================================================================

/// S → a — FIRST(S) = {a}, the single terminal in its production.
#[test]
fn first_of_lone_terminal_production() {
    let mut g = Grammar::new("cat1_lone".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
}

/// S → a b c — FIRST(S) is only the *first* terminal {a}, not {a,b,c}.
#[test]
fn first_of_multi_terminal_sequence_is_leading() {
    let mut g = Grammar::new("cat1_seq".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
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

/// Two distinct rules S → a and S → b — FIRST(S) = {a, b}.
#[test]
fn first_terminal_from_two_productions() {
    let mut g = Grammar::new("cat1_two".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
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

// =========================================================================
// Category 2: First set of non-terminal includes productions' first terms
// =========================================================================

/// S → A, A → a — FIRST(S) inherits from FIRST(A).
#[test]
fn first_nt_delegates_to_child() {
    let mut g = Grammar::new("cat2_delegate".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0)]);
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert_first_eq(&ff, NT_A, &[T_A]);
}

/// S → A, A → B, B → C, C → d — four-deep chain, FIRST propagates.
#[test]
fn first_nt_deep_four_level_chain() {
    let mut g = Grammar::new("cat2_deep4".into());
    tok(&mut g, T_D, "d", "d");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
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

/// S → A b, A → a | c — FIRST(S) = FIRST(A) = {a, c} (not {b}).
#[test]
fn first_nt_with_two_alt_productions() {
    let mut g = Grammar::new("cat2_alt".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_B)],
            0,
        )],
    );
    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::Terminal(T_A)], 1),
            rule(NT_A, vec![Symbol::Terminal(T_C)], 2),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A, T_C]);
}

// =========================================================================
// Category 3: Follow set of start symbol contains EOF
// =========================================================================

/// Simplest grammar: S → a. FOLLOW(S) ⊇ {$}.
#[test]
fn follow_start_has_eof_simple() {
    let mut g = Grammar::new("cat3_simple".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_S, &[EOF]);
}

/// Start with multiple rules: S → a | b. FOLLOW(S) still has EOF.
#[test]
fn follow_start_has_eof_multi_rule() {
    let mut g = Grammar::new("cat3_multi".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            rule(NT_S, vec![Symbol::Terminal(T_B)], 1),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_S, &[EOF]);
}

/// Recursive start: S → S a | b. FOLLOW(S) ⊇ {$, a}.
#[test]
fn follow_start_has_eof_recursive() {
    let mut g = Grammar::new("cat3_rec".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(
                NT_S,
                vec![Symbol::NonTerminal(NT_S), Symbol::Terminal(T_A)],
                0,
            ),
            rule(NT_S, vec![Symbol::Terminal(T_B)], 1),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_S, &[EOF, T_A]);
}

// =========================================================================
// Category 4: Determinism — same grammar gives identical sets
// =========================================================================

/// Build the same grammar 5 times — every FIRST/FOLLOW/nullable must agree.
#[test]
fn determinism_five_runs() {
    let build = || {
        let mut g = Grammar::new("det5".into());
        tok(&mut g, T_A, "a", "a");
        tok(&mut g, T_B, "b", "b");
        tok(&mut g, T_C, "c", "c");
        g.rule_names.insert(NT_S, "S".into());
        g.rule_names.insert(NT_A, "A".into());
        g.rule_names.insert(NT_B, "B".into());
        g.rules.insert(
            NT_S,
            vec![rule(
                NT_S,
                vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_B)],
                0,
            )],
        );
        g.rules.insert(
            NT_A,
            vec![
                rule(NT_A, vec![Symbol::Terminal(T_A)], 1),
                rule(NT_A, vec![Symbol::Epsilon], 2),
            ],
        );
        g.rules.insert(
            NT_B,
            vec![
                rule(NT_B, vec![Symbol::Terminal(T_B)], 3),
                rule(NT_B, vec![Symbol::Terminal(T_C)], 4),
            ],
        );
        g
    };

    let results: Vec<_> = (0..5)
        .map(|_| FirstFollowSets::compute(&build()).unwrap())
        .collect();

    let bits = |ff: &FirstFollowSets, sym: SymbolId, is_first: bool| -> Vec<usize> {
        let set = if is_first {
            ff.first(sym).unwrap()
        } else {
            ff.follow(sym).unwrap()
        };
        (0..set.len()).filter(|&i| set.contains(i)).collect()
    };

    for sym in [NT_S, NT_A, NT_B] {
        for i in 1..results.len() {
            assert_eq!(
                bits(&results[0], sym, true),
                bits(&results[i], sym, true),
                "FIRST({sym:?}) differs on run {i}",
            );
            assert_eq!(
                bits(&results[0], sym, false),
                bits(&results[i], sym, false),
                "FOLLOW({sym:?}) differs on run {i}",
            );
            assert_eq!(
                results[0].is_nullable(sym),
                results[i].is_nullable(sym),
                "nullable({sym:?}) differs on run {i}",
            );
        }
    }
}

/// Determinism with a nullable-heavy grammar.
#[test]
fn determinism_nullable_heavy() {
    let build = || {
        let mut g = Grammar::new("det_null".into());
        tok(&mut g, T_A, "a", "a");
        g.rule_names.insert(NT_S, "S".into());
        g.rule_names.insert(NT_A, "A".into());
        g.rules.insert(
            NT_S,
            vec![rule(
                NT_S,
                vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_A)],
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
        g
    };
    let ff1 = FirstFollowSets::compute(&build()).unwrap();
    let ff2 = FirstFollowSets::compute(&build()).unwrap();
    for sym in [NT_S, NT_A] {
        assert_eq!(ff1.is_nullable(sym), ff2.is_nullable(sym));
    }
}

// =========================================================================
// Category 5: Nullable propagation (epsilon rules)
// =========================================================================

/// A → ε makes A nullable; FIRST(A) = {} (no terminals).
#[test]
fn nullable_epsilon_only() {
    let mut g = Grammar::new("cat5_eps".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Epsilon], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
    assert_first_eq(&ff, NT_A, &[]);
}

/// Transitive nullable: S → A, A → ε.
#[test]
fn nullable_transitive_through_single_nt() {
    let mut g = Grammar::new("cat5_trans".into());
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0)]);
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Epsilon], 1)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
    assert!(ff.is_nullable(NT_S));
}

/// S → A B, A → ε, B → ε — nullable propagates through sequence.
#[test]
fn nullable_sequence_all_nullable() {
    let mut g = Grammar::new("cat5_seqnull".into());
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
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
    assert!(ff.is_nullable(NT_S));
    assert!(ff.is_nullable(NT_A));
    assert!(ff.is_nullable(NT_B));
}

/// S → A B, A → ε, B → b — S not nullable because B is not.
#[test]
fn nullable_sequence_partial() {
    let mut g = Grammar::new("cat5_partial".into());
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
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
    // FIRST(S) = FIRST(A) ∪ FIRST(B) = {b} because A is nullable
    assert_first_eq(&ff, NT_S, &[T_B]);
}

/// Nullable propagation: S → A B c, A → ε, B → ε | b — FIRST(S) = {b, c}.
#[test]
fn nullable_prefix_skips_to_terminal() {
    let mut g = Grammar::new("cat5_skip".into());
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
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
    assert!(!ff.is_nullable(NT_S));
    assert_first_eq(&ff, NT_S, &[T_B, T_C]);
}

// =========================================================================
// Category 6: Follow set propagation through production rules
// =========================================================================

/// S → A b — FOLLOW(A) ⊇ {b}.
#[test]
fn follow_contains_trailing_terminal() {
    let mut g = Grammar::new("cat6_trail".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
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
    assert_follow_contains(&ff, NT_A, &[T_B]);
}

/// S → A B, B → b — FOLLOW(A) ⊇ FIRST(B) = {b}.
#[test]
fn follow_contains_first_of_next_nt() {
    let mut g = Grammar::new("cat6_nextnt".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
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

/// S → A B, B → ε | b — B nullable so FOLLOW(A) ⊇ FIRST(B) ∪ FOLLOW(S).
#[test]
fn follow_propagates_through_nullable_suffix() {
    let mut g = Grammar::new("cat6_nullsuf".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
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
    g.rules.insert(
        NT_B,
        vec![
            rule(NT_B, vec![Symbol::Epsilon], 2),
            rule(NT_B, vec![Symbol::Terminal(T_B)], 3),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    // FOLLOW(A) ⊇ {b, $}
    assert_follow_contains(&ff, NT_A, &[T_B, EOF]);
}

/// Last NT in production inherits LHS's FOLLOW: S → a A, FOLLOW(A) ⊇ FOLLOW(S).
#[test]
fn follow_last_nt_inherits_lhs_follow() {
    let mut g = Grammar::new("cat6_last".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::Terminal(T_A), Symbol::NonTerminal(NT_A)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[EOF]);
}

/// S → A B C, B → b, C → c — FOLLOW(A) ⊇ FIRST(B) = {b}.
#[test]
fn follow_middle_nt_gets_first_of_successor() {
    let mut g = Grammar::new("cat6_mid".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
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
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_B)], 2)]);
    g.rules
        .insert(NT_C, vec![rule(NT_C, vec![Symbol::Terminal(T_C)], 3)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[T_B]);
    assert_follow_contains(&ff, NT_B, &[T_C]);
    assert_follow_contains(&ff, NT_C, &[EOF]);
}

/// NT used in two places gets union of follow contexts.
/// S → A b, S → c A d — FOLLOW(A) ⊇ {b, d}.
#[test]
fn follow_from_multiple_contexts() {
    let mut g = Grammar::new("cat6_multi".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    tok(&mut g, T_D, "d", "d");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(
                NT_S,
                vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_B)],
                0,
            ),
            rule(
                NT_S,
                vec![
                    Symbol::Terminal(T_C),
                    Symbol::NonTerminal(NT_A),
                    Symbol::Terminal(T_D),
                ],
                1,
            ),
        ],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[T_B, T_D]);
}

// =========================================================================
// Category 7: First set with choice / alternation
// =========================================================================

/// S → a | b | c | d — FIRST is union of all alternatives.
#[test]
fn first_four_way_terminal_choice() {
    let mut g = Grammar::new("cat7_4way".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    tok(&mut g, T_D, "d", "d");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            rule(NT_S, vec![Symbol::Terminal(T_B)], 1),
            rule(NT_S, vec![Symbol::Terminal(T_C)], 2),
            rule(NT_S, vec![Symbol::Terminal(T_D)], 3),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A, T_B, T_C, T_D]);
}

/// S → A | B, A → a, B → b — choice over non-terminals.
#[test]
fn first_choice_over_nonterminals() {
    let mut g = Grammar::new("cat7_ntchoice".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
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

/// S → A | ε, A → a — choice with epsilon makes S nullable, FIRST(S)={a}.
#[test]
fn first_choice_with_epsilon_alternative() {
    let mut g = Grammar::new("cat7_eps".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
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
    assert_first_eq(&ff, NT_S, &[T_A]);
}

/// Overlapping first sets: S → A | B, A → a | b, B → b | c.
/// FIRST(S) = {a, b, c}.
#[test]
fn first_choice_overlapping_sets() {
    let mut g = Grammar::new("cat7_overlap".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
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
    g.rules.insert(
        NT_B,
        vec![
            rule(NT_B, vec![Symbol::Terminal(T_B)], 4),
            rule(NT_B, vec![Symbol::Terminal(T_C)], 5),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A, T_B, T_C]);
}

// =========================================================================
// Category 8: Empty grammar handling
// =========================================================================

/// Grammar with no rules — compute should succeed without panic.
#[test]
fn empty_grammar_computes_without_panic() {
    let g = Grammar::new("empty".into());
    let result = FirstFollowSets::compute(&g);
    // Should succeed (possibly with empty sets) or return an error — must not panic.
    let _ = result;
}

/// Grammar with one non-terminal but no productions — no panic.
#[test]
fn grammar_with_named_nt_but_no_rules() {
    let mut g = Grammar::new("no_rules".into());
    g.rule_names.insert(NT_S, "S".into());
    // No rules inserted — S has no productions.
    let result = FirstFollowSets::compute(&g);
    let _ = result;
}

/// Grammar with tokens registered but no rules using them.
#[test]
fn grammar_with_tokens_no_rules() {
    let mut g = Grammar::new("tok_no_rule".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    let result = FirstFollowSets::compute(&g);
    let _ = result;
}

// =========================================================================
// Additional cross-cutting property tests
// =========================================================================

/// Non-start non-terminal not at end of any production should NOT have EOF in FOLLOW.
#[test]
fn follow_non_start_no_eof_when_not_trailing() {
    // S → A b, A → a
    let mut g = Grammar::new("no_eof_mid".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
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
    let follow_a = ff.follow(NT_A).unwrap();
    assert!(
        !follow_a.contains(EOF.0 as usize),
        "FOLLOW(A) should not contain EOF when A is never at the end",
    );
    assert_follow_eq(&ff, NT_A, &[T_B]);
}

/// is_nullable returns false for non-nullable non-terminal with only terminal RHS.
#[test]
fn is_nullable_false_for_terminal_only() {
    let mut g = Grammar::new("notnull".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_A, "A".into());
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(!ff.is_nullable(NT_A));
}

/// first_of_sequence on empty slice should be empty (no terminals).
#[test]
fn first_of_sequence_empty_slice() {
    let mut g = Grammar::new("seq_empty".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    let seq_first = ff.first_of_sequence(&[]).unwrap();
    assert_eq!(
        seq_first.count_ones(..),
        0,
        "FIRST of empty sequence should be empty"
    );
}

/// first_of_sequence with a leading terminal returns just that terminal.
#[test]
fn first_of_sequence_leading_terminal() {
    let mut g = Grammar::new("seq_lead".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::Terminal(T_A), Symbol::Terminal(T_B)],
            0,
        )],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    let seq_first = ff
        .first_of_sequence(&[Symbol::Terminal(T_A), Symbol::Terminal(T_B)])
        .unwrap();
    assert!(seq_first.contains(T_A.0 as usize));
    assert!(!seq_first.contains(T_B.0 as usize));
}
