#![allow(clippy::needless_range_loop)]
//! FIRST/FOLLOW set computation tests — v4.
//!
//! 55+ tests covering: terminals in FIRST, epsilon propagation, FOLLOW from
//! context, EOF in FOLLOW of start, consistency properties, determinism,
//! nullable chains, expression grammars, recursion, mutual recursion,
//! single-rule / single-token / all-nullable edge cases, and GrammarBuilder
//! integration.

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

/// Extract the set of SymbolIds present in a FixedBitSet.
fn bitset_ids(set: &fixedbitset::FixedBitSet) -> Vec<SymbolId> {
    (0..set.len())
        .filter(|&i| set.contains(i))
        .map(|i| SymbolId(i as u16))
        .collect()
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



// ---------------------------------------------------------------------------
// Symbol ID constants — terminals low, non-terminals high
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
const T_SEMI: SymbolId = SymbolId(10);
const T_IF: SymbolId = SymbolId(11);
const T_ELSE: SymbolId = SymbolId(12);

const NT_S: SymbolId = SymbolId(30);
const NT_A: SymbolId = SymbolId(31);
const NT_B: SymbolId = SymbolId(32);
const NT_C: SymbolId = SymbolId(33);
const NT_D: SymbolId = SymbolId(34);
const NT_E: SymbolId = SymbolId(35);
const NT_T: SymbolId = SymbolId(36);
const NT_F: SymbolId = SymbolId(37);


// =========================================================================
// 1. FIRST — single terminal production
// =========================================================================
#[test]
fn first_single_terminal_production() {
    // S → a
    let mut g = Grammar::new("v4_01".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert!(!ff.is_nullable(NT_S));
}

// =========================================================================
// 2. FIRST — two-terminal production takes only the first
// =========================================================================
#[test]
fn first_two_terminals_takes_first() {
    // S → a b
    let mut g = Grammar::new("v4_02".into());
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
    assert_first_eq(&ff, NT_S, &[T_A]);
}

// =========================================================================
// 3. FIRST — union of alternatives
// =========================================================================
#[test]
fn first_union_of_alternatives() {
    // S → a | b | c
    let mut g = Grammar::new("v4_03".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
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

// =========================================================================
// 4. FIRST — propagation through non-terminal
// =========================================================================
#[test]
fn first_propagation_through_nonterminal() {
    // S → A,  A → a
    let mut g = Grammar::new("v4_04".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 0)]);
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 1)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert_first_eq(&ff, NT_A, &[T_A]);
}

// =========================================================================
// 5. FIRST — epsilon makes non-terminal nullable
// =========================================================================
#[test]
fn first_epsilon_nullable() {
    // A → ε
    let mut g = Grammar::new("v4_05".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Epsilon], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
}

// =========================================================================
// 6. FIRST — epsilon propagation: S → A B, A nullable
// =========================================================================
#[test]
fn first_epsilon_propagation_to_second_symbol() {
    // S → A B,  A → ε | a,  B → b
    let mut g = Grammar::new("v4_06".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::Epsilon], 0),
            rule(NT_A, vec![Symbol::Terminal(T_A)], 1),
        ],
    );
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_B)], 2)]);
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_B)],
            3,
        )],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    // FIRST(S) = FIRST(A) ∪ FIRST(B) since A is nullable
    assert_first_eq(&ff, NT_S, &[T_A, T_B]);
    assert!(!ff.is_nullable(NT_S));
}

// =========================================================================
// 7. FIRST — epsilon propagation through chain of nullables
// =========================================================================
#[test]
fn first_epsilon_chain_propagation() {
    // S → A B C,  A → ε,  B → ε,  C → c
    let mut g = Grammar::new("v4_07".into());
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Epsilon], 0)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Epsilon], 1)]);
    g.rules
        .insert(NT_C, vec![rule(NT_C, vec![Symbol::Terminal(T_C)], 2)]);
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![
                Symbol::NonTerminal(NT_A),
                Symbol::NonTerminal(NT_B),
                Symbol::NonTerminal(NT_C),
            ],
            3,
        )],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_C]);
    assert!(ff.is_nullable(NT_A));
    assert!(ff.is_nullable(NT_B));
    assert!(!ff.is_nullable(NT_C));
    assert!(!ff.is_nullable(NT_S));
}

// =========================================================================
// 8. FIRST — left recursion does not loop
// =========================================================================
#[test]
fn first_left_recursion_terminates() {
    // A → A b | c
    let mut g = Grammar::new("v4_08".into());
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_A, "A".into());
    g.rules.insert(
        NT_A,
        vec![
            rule(
                NT_A,
                vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_B)],
                0,
            ),
            rule(NT_A, vec![Symbol::Terminal(T_C)], 1),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_A, &[T_C]);
    assert!(!ff.is_nullable(NT_A));
}

// =========================================================================
// 9. FIRST — right recursion
// =========================================================================
#[test]
fn first_right_recursion() {
    // A → b A | c
    let mut g = Grammar::new("v4_09".into());
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_A, "A".into());
    g.rules.insert(
        NT_A,
        vec![
            rule(
                NT_A,
                vec![Symbol::Terminal(T_B), Symbol::NonTerminal(NT_A)],
                0,
            ),
            rule(NT_A, vec![Symbol::Terminal(T_C)], 1),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_A, &[T_B, T_C]);
}

// =========================================================================
// 10. FOLLOW — start symbol always has EOF
// =========================================================================
#[test]
fn follow_start_has_eof() {
    // S → a
    let mut g = Grammar::new("v4_10".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_S, &[EOF]);
}

// =========================================================================
// 11. FOLLOW — terminal after non-terminal
// =========================================================================
#[test]
fn follow_terminal_after_nonterminal() {
    // S → A b,  A → a
    let mut g = Grammar::new("v4_11".into());
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

// =========================================================================
// 12. FOLLOW — non-terminal after non-terminal: FIRST of follower
// =========================================================================
#[test]
fn follow_nonterminal_after_nonterminal() {
    // S → A B,  A → a,  B → b
    let mut g = Grammar::new("v4_12".into());
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

// =========================================================================
// 13. FOLLOW — nullable suffix propagates FOLLOW of LHS
// =========================================================================
#[test]
fn follow_nullable_suffix_propagates_lhs() {
    // S → A B,  A → a,  B → ε | b
    // FOLLOW(A) ⊇ FIRST(B) ∪ FOLLOW(S)
    let mut g = Grammar::new("v4_13".into());
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
    assert_follow_contains(&ff, NT_A, &[T_B, EOF]);
}

// =========================================================================
// 14. FOLLOW — at-end non-terminal inherits FOLLOW of LHS
// =========================================================================
#[test]
fn follow_end_nonterminal_inherits_lhs() {
    // S → a A,  A → b
    // FOLLOW(A) ⊇ FOLLOW(S) = {EOF}
    let mut g = Grammar::new("v4_14".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
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
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_B)], 1)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[EOF]);
}

// =========================================================================
// 15. FOLLOW — middle symbol gets FIRST of next
// =========================================================================
#[test]
fn follow_middle_gets_first_of_next() {
    // S → A B C,  A → a,  B → b,  C → c
    let mut g = Grammar::new("v4_15".into());
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

// =========================================================================
// 16. FOLLOW — right recursion propagates EOF
// =========================================================================
#[test]
fn follow_right_recursion_eof() {
    // S → a A,  A → b A | c
    let mut g = Grammar::new("v4_16".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
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
    g.rules.insert(
        NT_A,
        vec![
            rule(
                NT_A,
                vec![Symbol::Terminal(T_B), Symbol::NonTerminal(NT_A)],
                1,
            ),
            rule(NT_A, vec![Symbol::Terminal(T_C)], 2),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[EOF]);
}

// =========================================================================
// 17. FIRST/FOLLOW consistency — terminals not nullable
// =========================================================================
#[test]
fn consistency_terminals_not_nullable() {
    let mut g = Grammar::new("v4_17".into());
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
    assert!(!ff.is_nullable(T_A));
    assert!(!ff.is_nullable(T_B));
}

// =========================================================================
// 18. Consistency — every non-terminal has a FIRST set
// =========================================================================
#[test]
fn consistency_all_nonterminals_have_first() {
    let mut g = Grammar::new("v4_18".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0)]);
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.first(NT_S).is_some());
    assert!(ff.first(NT_A).is_some());
}

// =========================================================================
// 19. Consistency — FIRST sets are non-empty for productive non-terminals
// =========================================================================
#[test]
fn consistency_first_nonempty_productive() {
    let mut g = Grammar::new("v4_19".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    let first_s = ff.first(NT_S).unwrap();
    assert!(first_s.count_ones(..) > 0);
}

// =========================================================================
// 20. Determinism — same grammar gives same FIRST sets
// =========================================================================
#[test]
fn determinism_first_sets() {
    let build = || {
        let mut g = Grammar::new("det".into());
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
        g
    };

    let ff1 = FirstFollowSets::compute(&build()).unwrap();
    let ff2 = FirstFollowSets::compute(&build()).unwrap();

    assert_eq!(bitset_ids(ff1.first(NT_S).unwrap()), bitset_ids(ff2.first(NT_S).unwrap()));
    assert_eq!(bitset_ids(ff1.first(NT_A).unwrap()), bitset_ids(ff2.first(NT_A).unwrap()));
}

// =========================================================================
// 21. Determinism — same grammar gives same FOLLOW sets
// =========================================================================
#[test]
fn determinism_follow_sets() {
    let build = || {
        let mut g = Grammar::new("det2".into());
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
        g
    };

    let ff1 = FirstFollowSets::compute(&build()).unwrap();
    let ff2 = FirstFollowSets::compute(&build()).unwrap();

    assert_eq!(bitset_ids(ff1.follow(NT_S).unwrap()), bitset_ids(ff2.follow(NT_S).unwrap()));
    assert_eq!(bitset_ids(ff1.follow(NT_A).unwrap()), bitset_ids(ff2.follow(NT_A).unwrap()));
}

// =========================================================================
// 22. Determinism — nullable flags are deterministic
// =========================================================================
#[test]
fn determinism_nullable_flags() {
    let build = || {
        let mut g = Grammar::new("det3".into());
        tok(&mut g, T_A, "a", "a");
        g.rule_names.insert(NT_A, "A".into());
        g.rule_names.insert(NT_B, "B".into());
        g.rules.insert(
            NT_A,
            vec![
                rule(NT_A, vec![Symbol::Epsilon], 0),
                rule(NT_A, vec![Symbol::Terminal(T_A)], 1),
            ],
        );
        g.rules
            .insert(NT_B, vec![rule(NT_B, vec![Symbol::NonTerminal(NT_A)], 2)]);
        g
    };

    let ff1 = FirstFollowSets::compute(&build()).unwrap();
    let ff2 = FirstFollowSets::compute(&build()).unwrap();

    assert_eq!(ff1.is_nullable(NT_A), ff2.is_nullable(NT_A));
    assert_eq!(ff1.is_nullable(NT_B), ff2.is_nullable(NT_B));
}

// =========================================================================
// 23. Epsilon — purely epsilon non-terminal
// =========================================================================
#[test]
fn epsilon_only_rule() {
    // A → ε
    let mut g = Grammar::new("v4_23".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Epsilon], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
    // FIRST set should have no terminals
    let first = ff.first(NT_A).unwrap();
    assert_eq!(first.count_ones(..), 0);
}

// =========================================================================
// 24. Epsilon — nullable chain: A → B, B → ε
// =========================================================================
#[test]
fn epsilon_nullable_chain() {
    let mut g = Grammar::new("v4_24".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Epsilon], 0)]);
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::NonTerminal(NT_B)], 1)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
    assert!(ff.is_nullable(NT_B));
}

// =========================================================================
// 25. Epsilon — transitive nullable: A → B, B → C, C → ε
// =========================================================================
#[test]
fn epsilon_transitive_nullable() {
    let mut g = Grammar::new("v4_25".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rules
        .insert(NT_C, vec![rule(NT_C, vec![Symbol::Epsilon], 0)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::NonTerminal(NT_C)], 1)]);
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::NonTerminal(NT_B)], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
    assert!(ff.is_nullable(NT_B));
    assert!(ff.is_nullable(NT_C));
}

// =========================================================================
// 26. Epsilon — all nullable concatenation: S → A B, A → ε, B → ε
// =========================================================================
#[test]
fn epsilon_all_nullable_concatenation() {
    let mut g = Grammar::new("v4_26".into());
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Epsilon], 0)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Epsilon], 1)]);
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_B)],
            2,
        )],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_S));
    assert!(ff.is_nullable(NT_A));
    assert!(ff.is_nullable(NT_B));
}

// =========================================================================
// 27. Epsilon — mixed nullable/non-nullable in sequence
// =========================================================================
#[test]
fn epsilon_mixed_nullable_sequence() {
    // S → A B C,  A → ε,  B → b,  C → ε
    let mut g = Grammar::new("v4_27".into());
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Epsilon], 0)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_B)], 1)]);
    g.rules
        .insert(NT_C, vec![rule(NT_C, vec![Symbol::Epsilon], 2)]);
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![
                Symbol::NonTerminal(NT_A),
                Symbol::NonTerminal(NT_B),
                Symbol::NonTerminal(NT_C),
            ],
            3,
        )],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
    assert!(!ff.is_nullable(NT_B));
    assert!(ff.is_nullable(NT_C));
    // S is NOT nullable: B is not nullable
    assert!(!ff.is_nullable(NT_S));
    // FIRST(S) = {b} since A is nullable but B starts with b
    assert_first_eq(&ff, NT_S, &[T_B]);
}

// =========================================================================
// 28. Complex — arithmetic grammar FIRST sets
// =========================================================================
#[test]
fn complex_arithmetic_first() {
    // E → E + T | T,  T → T * F | F,  F → ( E ) | num
    let mut g = Grammar::new("v4_28".into());
    tok(&mut g, T_PLUS, "+", "+");
    tok(&mut g, T_STAR, "*", "*");
    tok(&mut g, T_LPAREN, "(", "(");
    tok(&mut g, T_RPAREN, ")", ")");
    tok(&mut g, T_NUM, "num", "[0-9]+");
    g.rule_names.insert(NT_E, "E".into());
    g.rule_names.insert(NT_T, "T".into());
    g.rule_names.insert(NT_F, "F".into());
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
    assert_first_eq(&ff, NT_F, &[T_LPAREN, T_NUM]);
    assert_first_eq(&ff, NT_T, &[T_LPAREN, T_NUM]);
    assert_first_eq(&ff, NT_E, &[T_LPAREN, T_NUM]);
    assert!(!ff.is_nullable(NT_E));
    assert!(!ff.is_nullable(NT_T));
    assert!(!ff.is_nullable(NT_F));
}

// =========================================================================
// 29. Complex — arithmetic grammar FOLLOW sets
// =========================================================================
#[test]
fn complex_arithmetic_follow() {
    let mut g = Grammar::new("v4_29".into());
    tok(&mut g, T_PLUS, "+", "+");
    tok(&mut g, T_STAR, "*", "*");
    tok(&mut g, T_LPAREN, "(", "(");
    tok(&mut g, T_RPAREN, ")", ")");
    tok(&mut g, T_NUM, "num", "[0-9]+");
    g.rule_names.insert(NT_E, "E".into());
    g.rule_names.insert(NT_T, "T".into());
    g.rule_names.insert(NT_F, "F".into());
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
    // FOLLOW(E) ⊇ {$, +, )}
    assert_follow_contains(&ff, NT_E, &[EOF, T_PLUS, T_RPAREN]);
    // FOLLOW(T) ⊇ {+, *, ), $}
    assert_follow_contains(&ff, NT_T, &[T_PLUS, T_STAR, T_RPAREN, EOF]);
    // FOLLOW(F) ⊇ {+, *, ), $}
    assert_follow_contains(&ff, NT_F, &[T_PLUS, T_STAR, T_RPAREN, EOF]);
}

// =========================================================================
// 30. Complex — mutual recursion: A → B a, B → A b | c
// =========================================================================
#[test]
fn complex_mutual_recursion() {
    let mut g = Grammar::new("v4_30".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
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
    // Both ultimately start with c
    assert_first_contains(&ff, NT_A, &[T_C]);
    assert_first_contains(&ff, NT_B, &[T_C]);
    assert!(!ff.is_nullable(NT_A));
    assert!(!ff.is_nullable(NT_B));
}

// =========================================================================
// 31. Complex — mutual recursion with nullable
// =========================================================================
#[test]
fn complex_mutual_recursion_nullable() {
    // A → B | a,  B → A | ε
    let mut g = Grammar::new("v4_31".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::NonTerminal(NT_B)], 0),
            rule(NT_A, vec![Symbol::Terminal(T_A)], 1),
        ],
    );
    g.rules.insert(
        NT_B,
        vec![
            rule(NT_B, vec![Symbol::NonTerminal(NT_A)], 2),
            rule(NT_B, vec![Symbol::Epsilon], 3),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_B));
    // A is nullable through B → ε path
    assert!(ff.is_nullable(NT_A));
    assert_first_contains(&ff, NT_A, &[T_A]);
}

// =========================================================================
// 32. Complex — deeply nested: S → A, A → B, B → C, C → a
// =========================================================================
#[test]
fn complex_deeply_nested_chain() {
    let mut g = Grammar::new("v4_32".into());
    tok(&mut g, T_A, "a", "a");
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
        .insert(NT_C, vec![rule(NT_C, vec![Symbol::Terminal(T_A)], 3)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert_first_eq(&ff, NT_A, &[T_A]);
    assert_first_eq(&ff, NT_B, &[T_A]);
    assert_first_eq(&ff, NT_C, &[T_A]);
    // All should have EOF in FOLLOW (chain to start)
    for sym in [NT_S, NT_A, NT_B, NT_C] {
        assert_follow_contains(&ff, sym, &[EOF]);
    }
}

// =========================================================================
// 33. Complex — diamond: S → A B, A → C, B → C, C → c
// =========================================================================
#[test]
fn complex_diamond_grammar() {
    let mut g = Grammar::new("v4_33".into());
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_B)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::NonTerminal(NT_C)], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::NonTerminal(NT_C)], 2)]);
    g.rules
        .insert(NT_C, vec![rule(NT_C, vec![Symbol::Terminal(T_C)], 3)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_C]);
    assert_first_eq(&ff, NT_A, &[T_C]);
    assert_first_eq(&ff, NT_B, &[T_C]);
    // FOLLOW(A) should contain FIRST(B) = {c}
    assert_follow_contains(&ff, NT_A, &[T_C]);
}

// =========================================================================
// 34. Edge case — single rule, single token grammar
// =========================================================================
#[test]
fn edge_single_rule_single_token() {
    let mut g = Grammar::new("v4_34".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert_follow_contains(&ff, NT_S, &[EOF]);
    assert!(!ff.is_nullable(NT_S));
}

// =========================================================================
// 35. Edge case — all productions nullable
// =========================================================================
#[test]
fn edge_all_nullable() {
    // S → A,  A → B,  B → ε
    let mut g = Grammar::new("v4_35".into());
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0)]);
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::NonTerminal(NT_B)], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Epsilon], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_S));
    assert!(ff.is_nullable(NT_A));
    assert!(ff.is_nullable(NT_B));
}

// =========================================================================
// 36. Edge case — multiple alternatives with same terminal
// =========================================================================
#[test]
fn edge_duplicate_terminal_in_alternatives() {
    // S → a | a b
    let mut g = Grammar::new("v4_36".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::Terminal(T_A)], 0),
            rule(
                NT_S,
                vec![Symbol::Terminal(T_A), Symbol::Terminal(T_B)],
                1,
            ),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    // FIRST should still just be {a}
    assert_first_eq(&ff, NT_S, &[T_A]);
}

// =========================================================================
// 37. Edge case — empty RHS treated as epsilon
// =========================================================================
#[test]
fn edge_explicit_epsilon_production() {
    // S → ε | a
    let mut g = Grammar::new("v4_37".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::Epsilon], 0),
            rule(NT_S, vec![Symbol::Terminal(T_A)], 1),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_S));
    assert_first_eq(&ff, NT_S, &[T_A]);
}

// =========================================================================
// 38. Epsilon — all-nullable prefix, FOLLOW propagation through nullables
// =========================================================================
#[test]
fn epsilon_all_nullable_follow_propagation() {
    // S → A B,  A → ε | a,  B → ε | b
    // A and B both nullable; FOLLOW(A) ⊇ FIRST(B) = {b} ∪ FOLLOW(S)
    // FOLLOW(B) ⊇ FOLLOW(S)
    let mut g = Grammar::new("v4_38".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::Epsilon], 0),
            rule(NT_A, vec![Symbol::Terminal(T_A)], 1),
        ],
    );
    g.rules.insert(
        NT_B,
        vec![
            rule(NT_B, vec![Symbol::Epsilon], 2),
            rule(NT_B, vec![Symbol::Terminal(T_B)], 3),
        ],
    );
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_B)],
            4,
        )],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
    assert!(ff.is_nullable(NT_B));
    assert!(ff.is_nullable(NT_S));
    // FOLLOW(A) ⊇ {b} because FIRST(B) = {b}
    assert_follow_contains(&ff, NT_A, &[T_B]);
}

// =========================================================================
// 39. FIRST — multiple non-terminals, first nullable
// =========================================================================
#[test]
fn first_multiple_nonterminals_first_nullable() {
    // S → A B c,  A → ε | a,  B → b
    // FIRST(S) = {a, b} because A nullable → b from B
    let mut g = Grammar::new("v4_39".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::Epsilon], 0),
            rule(NT_A, vec![Symbol::Terminal(T_A)], 1),
        ],
    );
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_B)], 2)]);
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![
                Symbol::NonTerminal(NT_A),
                Symbol::NonTerminal(NT_B),
                Symbol::Terminal(T_C),
            ],
            3,
        )],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A, T_B]);
}

// =========================================================================
// 40. FOLLOW — multiple contexts for same non-terminal
// =========================================================================
#[test]
fn follow_multiple_contexts() {
    // S → A b | c A d,  A → a
    // FOLLOW(A) ⊇ {b, d}
    let mut g = Grammar::new("v4_40".into());
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
// 41. FOLLOW — chain to start: S → A, A → B, B → a
// =========================================================================
#[test]
fn follow_chain_to_start() {
    let mut g = Grammar::new("v4_41".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0)]);
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::NonTerminal(NT_B)], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_A)], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    // EOF propagates through: FOLLOW(S) → FOLLOW(A) → FOLLOW(B)
    assert_follow_contains(&ff, NT_A, &[EOF]);
    assert_follow_contains(&ff, NT_B, &[EOF]);
}

// =========================================================================
// 42. FIRST/FOLLOW — overlap in ambiguous grammar
// =========================================================================
#[test]
fn first_follow_overlap_in_nullable() {
    // S → A B,  A → b | ε,  B → b | c
    // FIRST(B) and FOLLOW(A) both contain b
    let mut g = Grammar::new("v4_42".into());
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
            rule(NT_A, vec![Symbol::Terminal(T_B)], 1),
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

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[T_B, T_C]);
    assert_first_eq(&ff, NT_S, &[T_B, T_C]);
}

// =========================================================================
// 43. Consistency — FOLLOW(A) = FOLLOW(S) when S → A with no other context
// =========================================================================
#[test]
fn consistency_follow_equals_parent_single_chain() {
    // S → A,  A → a | b
    let mut g = Grammar::new("v4_43".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0)]);
    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::Terminal(T_A)], 1),
            rule(NT_A, vec![Symbol::Terminal(T_B)], 2),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    let follow_s = ff.follow(NT_S).unwrap();
    let follow_a = ff.follow(NT_A).unwrap();
    // A is at end of S, so FOLLOW(A) ⊇ FOLLOW(S)
    for i in 0..follow_s.len() {
        if follow_s.contains(i) {
            assert!(
                follow_a.contains(i),
                "FOLLOW(S) element {i} missing from FOLLOW(A)"
            );
        }
    }
}

// =========================================================================
// 44. Complex — statement-like grammar with semicolons
// =========================================================================
#[test]
fn complex_statement_grammar() {
    // S → A ;,  A → B | C,  B → a,  C → b
    let mut g = Grammar::new("v4_44".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_SEMI, ";", ";");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_SEMI)],
            0,
        )],
    );
    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::NonTerminal(NT_B)], 1),
            rule(NT_A, vec![Symbol::NonTerminal(NT_C)], 2),
        ],
    );
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_A)], 3)]);
    g.rules
        .insert(NT_C, vec![rule(NT_C, vec![Symbol::Terminal(T_B)], 4)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_A, &[T_A, T_B]);
    assert_follow_contains(&ff, NT_A, &[T_SEMI]);
    assert_follow_contains(&ff, NT_B, &[T_SEMI]);
    assert_follow_contains(&ff, NT_C, &[T_SEMI]);
}

// =========================================================================
// 45. Complex — if-else-like: S → if A else A | a
// =========================================================================
#[test]
fn complex_if_else_grammar() {
    // S → if A else A | a,  A → b
    let mut g = Grammar::new("v4_45".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_IF, "if", "if");
    tok(&mut g, T_ELSE, "else", "else");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(
                NT_S,
                vec![
                    Symbol::Terminal(T_IF),
                    Symbol::NonTerminal(NT_A),
                    Symbol::Terminal(T_ELSE),
                    Symbol::NonTerminal(NT_A),
                ],
                0,
            ),
            rule(NT_S, vec![Symbol::Terminal(T_A)], 1),
        ],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_B)], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_IF, T_A]);
    // FOLLOW(A) ⊇ {else, $} — first A followed by else, second A at end
    assert_follow_contains(&ff, NT_A, &[T_ELSE, EOF]);
}

// =========================================================================
// 46. GrammarBuilder — simple token grammar
// =========================================================================
#[test]
fn builder_simple_token() {
    use adze_ir::builder::GrammarBuilder;

    let grammar = GrammarBuilder::new("simple")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let start_id = grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "start")
        .map(|(id, _)| *id)
        .unwrap();
    let x_id = *grammar
        .tokens
        .iter()
        .find(|(_, t)| t.name == "x")
        .map(|(id, _)| id)
        .unwrap();

    assert!(ff.first(start_id).unwrap().contains(x_id.0 as usize));
    assert!(!ff.is_nullable(start_id));
}

// =========================================================================
// 47. GrammarBuilder — arithmetic grammar
// =========================================================================
#[test]
fn builder_arithmetic_grammar() {
    use adze_ir::builder::GrammarBuilder;

    let grammar = GrammarBuilder::new("calc")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "NUMBER"])
        .rule("term", vec!["NUMBER"])
        .start("expr")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let expr_id = grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "expr")
        .map(|(id, _)| *id)
        .unwrap();
    let term_id = grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "term")
        .map(|(id, _)| *id)
        .unwrap();
    let number_id = *grammar
        .tokens
        .iter()
        .find(|(_, t)| t.name == "NUMBER")
        .map(|(id, _)| id)
        .unwrap();

    assert!(ff.first(expr_id).unwrap().contains(number_id.0 as usize));
    assert!(ff.first(term_id).unwrap().contains(number_id.0 as usize));
    assert!(!ff.is_nullable(expr_id));
    assert!(!ff.is_nullable(term_id));
}

// =========================================================================
// 48. GrammarBuilder — determinism via builder
// =========================================================================
#[test]
fn builder_determinism() {
    use adze_ir::builder::GrammarBuilder;

    let build = || {
        GrammarBuilder::new("det")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
            .build()
    };

    let g1 = build();
    let g2 = build();
    let ff1 = FirstFollowSets::compute(&g1).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();

    let id1 = g1
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "start")
        .map(|(id, _)| *id)
        .unwrap();
    let id2 = g2
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "start")
        .map(|(id, _)| *id)
        .unwrap();

    assert_eq!(
        bitset_ids(ff1.first(id1).unwrap()),
        bitset_ids(ff2.first(id2).unwrap())
    );
}

// =========================================================================
// 49. Edge case — many alternatives
// =========================================================================
#[test]
fn edge_many_alternatives() {
    // S → a | b | c | d
    let mut g = Grammar::new("v4_49".into());
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

// =========================================================================
// 50. Edge case — long production RHS
// =========================================================================
#[test]
fn edge_long_production() {
    // S → a b c d
    let mut g = Grammar::new("v4_50".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    tok(&mut g, T_D, "d", "d");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![
                Symbol::Terminal(T_A),
                Symbol::Terminal(T_B),
                Symbol::Terminal(T_C),
                Symbol::Terminal(T_D),
            ],
            0,
        )],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert!(!ff.is_nullable(NT_S));
}

// =========================================================================
// 51. Complex — list grammar: L → L ; E | E,  E → a
// =========================================================================
#[test]
fn complex_list_grammar() {
    let mut g = Grammar::new("v4_51".into());
    tok(&mut g, T_SEMI, ";", ";");
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_E, "E".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(
                NT_S,
                vec![
                    Symbol::NonTerminal(NT_S),
                    Symbol::Terminal(T_SEMI),
                    Symbol::NonTerminal(NT_E),
                ],
                0,
            ),
            rule(NT_S, vec![Symbol::NonTerminal(NT_E)], 1),
        ],
    );
    g.rules
        .insert(NT_E, vec![rule(NT_E, vec![Symbol::Terminal(T_A)], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert_first_eq(&ff, NT_E, &[T_A]);
    // FOLLOW(E) ⊇ {;, $}
    assert_follow_contains(&ff, NT_E, &[T_SEMI, EOF]);
}

// =========================================================================
// 52. Consistency — non-nullable symbol has non-empty FIRST
// =========================================================================
#[test]
fn consistency_non_nullable_has_first() {
    let mut g = Grammar::new("v4_52".into());
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
    // For every non-nullable non-terminal, FIRST should be non-empty
    for &sym in &[NT_S, NT_A] {
        assert!(!ff.is_nullable(sym));
        let first = ff.first(sym).unwrap();
        assert!(
            first.count_ones(..) > 0,
            "{sym:?} is non-nullable but has empty FIRST"
        );
    }
}

// =========================================================================
// 53. Complex — parenthesized expression
// =========================================================================
#[test]
fn complex_parenthesized_expr() {
    // S → ( S ) | a
    let mut g = Grammar::new("v4_53".into());
    tok(&mut g, T_LPAREN, "(", "(");
    tok(&mut g, T_RPAREN, ")", ")");
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(
                NT_S,
                vec![
                    Symbol::Terminal(T_LPAREN),
                    Symbol::NonTerminal(NT_S),
                    Symbol::Terminal(T_RPAREN),
                ],
                0,
            ),
            rule(NT_S, vec![Symbol::Terminal(T_A)], 1),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_LPAREN, T_A]);
    assert_follow_contains(&ff, NT_S, &[T_RPAREN, EOF]);
}

// =========================================================================
// 54. Epsilon — nullable with follow from two producers
// =========================================================================
#[test]
fn epsilon_nullable_follow_from_two_producers() {
    // S → A b | c A d,  A → ε | a
    // FOLLOW(A) ⊇ {b, d}
    let mut g = Grammar::new("v4_54".into());
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
    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::Epsilon], 2),
            rule(NT_A, vec![Symbol::Terminal(T_A)], 3),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
    assert_follow_contains(&ff, NT_A, &[T_B, T_D]);
}

// =========================================================================
// 55. Complex — wide fan-out: S → A | B | C | D
// =========================================================================
#[test]
fn complex_wide_fanout() {
    // S → A | B | C | D,  A → a,  B → b,  C → c,  D → d
    let mut g = Grammar::new("v4_55".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    tok(&mut g, T_D, "d", "d");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rule_names.insert(NT_D, "D".into());
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0),
            rule(NT_S, vec![Symbol::NonTerminal(NT_B)], 1),
            rule(NT_S, vec![Symbol::NonTerminal(NT_C)], 2),
            rule(NT_S, vec![Symbol::NonTerminal(NT_D)], 3),
        ],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 4)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_B)], 5)]);
    g.rules
        .insert(NT_C, vec![rule(NT_C, vec![Symbol::Terminal(T_C)], 6)]);
    g.rules
        .insert(NT_D, vec![rule(NT_D, vec![Symbol::Terminal(T_D)], 7)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A, T_B, T_C, T_D]);
    // All children at end of S → FOLLOW includes EOF
    for sym in [NT_A, NT_B, NT_C, NT_D] {
        assert_follow_contains(&ff, sym, &[EOF]);
    }
}

// =========================================================================
// 56. Determinism — FIRST/FOLLOW stable across iterations
// =========================================================================
#[test]
fn determinism_stable_iterations() {
    let mut g = Grammar::new("v4_56".into());
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
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_B)], 3)]);

    // Compute multiple times, verify same results
    let ff1 = FirstFollowSets::compute(&g).unwrap();
    let ff2 = FirstFollowSets::compute(&g).unwrap();
    let ff3 = FirstFollowSets::compute(&g).unwrap();

    for sym in [NT_S, NT_A, NT_B] {
        assert_eq!(bitset_ids(ff1.first(sym).unwrap()), bitset_ids(ff2.first(sym).unwrap()));
        assert_eq!(bitset_ids(ff2.first(sym).unwrap()), bitset_ids(ff3.first(sym).unwrap()));
        assert_eq!(bitset_ids(ff1.follow(sym).unwrap()), bitset_ids(ff2.follow(sym).unwrap()));
        assert_eq!(bitset_ids(ff2.follow(sym).unwrap()), bitset_ids(ff3.follow(sym).unwrap()));
    }
}

// =========================================================================
// 57. FOLLOW — nullable chain at end: S → a A B, A → c, B → ε
// =========================================================================
#[test]
fn follow_nullable_at_end_chain() {
    // FOLLOW(A) ⊇ FIRST(B) ∪ FOLLOW(S)
    // B nullable → FOLLOW(A) ⊇ FOLLOW(S) = {$}
    let mut g = Grammar::new("v4_57".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![
                Symbol::Terminal(T_A),
                Symbol::NonTerminal(NT_A),
                Symbol::NonTerminal(NT_B),
            ],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_C)], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Epsilon], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_B));
    assert_follow_contains(&ff, NT_A, &[EOF]);
}

// =========================================================================
// 58. Edge case — self-recursive nullable: A → A | ε
// =========================================================================
#[test]
fn edge_self_recursive_nullable() {
    let mut g = Grammar::new("v4_58".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::NonTerminal(NT_A)], 0),
            rule(NT_A, vec![Symbol::Epsilon], 1),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
}

// =========================================================================
// 59. Consistency — FOLLOW set of start always non-empty (has EOF)
// =========================================================================
#[test]
fn consistency_start_follow_nonempty() {
    let mut g = Grammar::new("v4_59".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    let follow_s = ff.follow(NT_S).unwrap();
    assert!(
        follow_s.count_ones(..) > 0,
        "FOLLOW(start) should always contain at least EOF"
    );
}

// =========================================================================
// 60. Complex — triple mutual recursion: A → B c, B → C d, C → A | e
// =========================================================================
#[test]
fn complex_triple_mutual_recursion() {
    let mut g = Grammar::new("v4_60".into());
    tok(&mut g, T_C, "c", "c");
    tok(&mut g, T_D, "d", "d");
    let t_e = SymbolId(13);
    tok(&mut g, t_e, "e", "e");
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rules.insert(
        NT_A,
        vec![rule(
            NT_A,
            vec![Symbol::NonTerminal(NT_B), Symbol::Terminal(T_C)],
            0,
        )],
    );
    g.rules.insert(
        NT_B,
        vec![rule(
            NT_B,
            vec![Symbol::NonTerminal(NT_C), Symbol::Terminal(T_D)],
            1,
        )],
    );
    g.rules.insert(
        NT_C,
        vec![
            rule(NT_C, vec![Symbol::NonTerminal(NT_A)], 2),
            rule(NT_C, vec![Symbol::Terminal(t_e)], 3),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    // All ultimately derive from 'e'
    assert_first_contains(&ff, NT_A, &[t_e]);
    assert_first_contains(&ff, NT_B, &[t_e]);
    assert_first_contains(&ff, NT_C, &[t_e]);
    assert!(!ff.is_nullable(NT_A));
    assert!(!ff.is_nullable(NT_B));
    assert!(!ff.is_nullable(NT_C));
}
