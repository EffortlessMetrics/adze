//! Comprehensive FIRST/FOLLOW set computation tests for adze-glr-core.
//!
//! Covers: single terminals, non-terminal propagation, nullable rules, choice,
//! chain propagation, left/right recursion, EOF in FOLLOW, nullable suffixes,
//! arithmetic grammars, precedence grammars, determinism, optional symbols,
//! consistency properties, and edge cases.

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

/// Helper: check that `set` contains exactly `expected` symbol ids.
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
            "FOLLOW({sym:?}) should contain {e:?}"
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
// Constants – terminal IDs in low range, non-terminals in high range
// ---------------------------------------------------------------------------
const EOF: SymbolId = SymbolId(0);

const T_A: SymbolId = SymbolId(1);
const T_B: SymbolId = SymbolId(2);
const T_C: SymbolId = SymbolId(3);
const T_PLUS: SymbolId = SymbolId(5);
const T_STAR: SymbolId = SymbolId(6);
const T_LPAREN: SymbolId = SymbolId(7);
const T_RPAREN: SymbolId = SymbolId(8);
const T_NUM: SymbolId = SymbolId(9);

const NT_S: SymbolId = SymbolId(20);
const NT_A: SymbolId = SymbolId(21);
const NT_B: SymbolId = SymbolId(22);
const NT_C: SymbolId = SymbolId(23);
const NT_D: SymbolId = SymbolId(24);
const NT_E: SymbolId = SymbolId(25);
const NT_T: SymbolId = SymbolId(26);
const NT_F: SymbolId = SymbolId(27);

// =========================================================================
// 1. FIRST set of single terminal symbol
// =========================================================================
#[test]
fn first_single_terminal() {
    // S → a
    let mut g = Grammar::new("t1".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert!(!ff.is_nullable(NT_S));
}

// =========================================================================
// 2. FIRST set of non-terminal that starts with terminal
// =========================================================================
#[test]
fn first_nonterminal_starts_with_terminal() {
    // A → a b
    let mut g = Grammar::new("t2".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_A, "A".into());
    g.rules.insert(
        NT_A,
        vec![rule(
            NT_A,
            vec![Symbol::Terminal(T_A), Symbol::Terminal(T_B)],
            0,
        )],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_A, &[T_A]);
}

// =========================================================================
// 3. FIRST set includes epsilon for nullable rule
// =========================================================================
#[test]
fn first_nullable_rule() {
    // A → ε | a
    let mut g = Grammar::new("t3".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_A, "A".into());
    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::Epsilon], 0),
            rule(NT_A, vec![Symbol::Terminal(T_A)], 1),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
    assert_first_eq(&ff, NT_A, &[T_A]);
}

// =========================================================================
// 4. FIRST set of choice: A → B | C  ⟹  FIRST(A) = FIRST(B) ∪ FIRST(C)
// =========================================================================
#[test]
fn first_choice_union() {
    // B → b,  C → c,  A → B | C
    let mut g = Grammar::new("t4".into());
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_B)], 0)]);
    g.rules
        .insert(NT_C, vec![rule(NT_C, vec![Symbol::Terminal(T_C)], 1)]);
    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::NonTerminal(NT_B)], 2),
            rule(NT_A, vec![Symbol::NonTerminal(NT_C)], 3),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_A, &[T_B, T_C]);
}

// =========================================================================
// 5. FIRST set propagation through chain: A → B, B → c
// =========================================================================
#[test]
fn first_chain_propagation() {
    let mut g = Grammar::new("t5".into());
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_C)], 0)]);
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::NonTerminal(NT_B)], 1)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_A, &[T_C]);
    assert_first_eq(&ff, NT_B, &[T_C]);
}

// =========================================================================
// 6. FIRST set with left recursion: A → A b | c
// =========================================================================
#[test]
fn first_left_recursion() {
    let mut g = Grammar::new("t6".into());
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
    // Only 'c' should be in FIRST(A); 'b' is never the start of an A-derivation.
    assert_first_eq(&ff, NT_A, &[T_C]);
    assert!(!ff.is_nullable(NT_A));
}

// =========================================================================
// 7. FOLLOW set of start symbol includes EOF
// =========================================================================
#[test]
fn follow_start_includes_eof() {
    // S → a  (S is the start symbol)
    let mut g = Grammar::new("t7".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_S, &[EOF]);
}

// =========================================================================
// 8. FOLLOW set propagation from production context
// =========================================================================
#[test]
fn follow_propagation_from_context() {
    // S → A b,  A → a
    // FOLLOW(A) should contain 'b' since 'b' follows A in S → A b
    let mut g = Grammar::new("t8".into());
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
// 9. FOLLOW set with nullable suffix: S → A C, C → ε | c
// =========================================================================
#[test]
fn follow_nullable_suffix() {
    // S → A C,  A → a,  C → ε | c
    // Since C is nullable, FOLLOW(A) ⊇ FOLLOW(S) ∪ FIRST(C)
    let mut g = Grammar::new("t9".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_C)],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    g.rules.insert(
        NT_C,
        vec![
            rule(NT_C, vec![Symbol::Epsilon], 2),
            rule(NT_C, vec![Symbol::Terminal(T_C)], 3),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_C));
    // FOLLOW(A) should contain FIRST(C) = {c} and FOLLOW(S) = {EOF}
    assert_follow_contains(&ff, NT_A, &[T_C, EOF]);
}

// =========================================================================
// 10. FOLLOW set includes FIRST of following symbol
// =========================================================================
#[test]
fn follow_includes_first_of_following() {
    // S → A B,  A → a,  B → b
    // FOLLOW(A) should contain FIRST(B) = {b}
    let mut g = Grammar::new("t10".into());
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
// 11. FOLLOW set with right recursion
// =========================================================================
#[test]
fn follow_right_recursion() {
    // S → a A,  A → b A | c
    // FOLLOW(A) ⊇ FOLLOW(A) (from A → b A), plus FOLLOW(S) = {EOF} (from S → a A)
    let mut g = Grammar::new("t11".into());
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
    // FOLLOW(A) = {$} since A is always at the end of S or recursively at the end of A
    assert_follow_contains(&ff, NT_A, &[EOF]);
}

// =========================================================================
// 12. FIRST/FOLLOW interaction: A → B C, FIRST(C) overlaps FOLLOW(B)
// =========================================================================
#[test]
fn first_follow_interaction_overlap() {
    // S → A,  A → B C,  B → b | ε,  C → b | c
    // FIRST(C) = {b, c} and FOLLOW(B) ⊇ FIRST(C) = {b, c}
    let mut g = Grammar::new("t12".into());
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 0)]);
    g.rules.insert(
        NT_A,
        vec![rule(
            NT_A,
            vec![Symbol::NonTerminal(NT_B), Symbol::NonTerminal(NT_C)],
            1,
        )],
    );
    g.rules.insert(
        NT_B,
        vec![
            rule(NT_B, vec![Symbol::Terminal(T_B)], 2),
            rule(NT_B, vec![Symbol::Epsilon], 3),
        ],
    );
    g.rules.insert(
        NT_C,
        vec![
            rule(NT_C, vec![Symbol::Terminal(T_B)], 4),
            rule(NT_C, vec![Symbol::Terminal(T_C)], 5),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_B));
    assert!(!ff.is_nullable(NT_C));
    // FIRST(A) = FIRST(B) ∪ FIRST(C) = {b, c}  (B is nullable so C's FIRST flows in)
    assert_first_eq(&ff, NT_A, &[T_B, T_C]);
    // FOLLOW(B) ⊇ FIRST(C) = {b, c}
    assert_follow_contains(&ff, NT_B, &[T_B, T_C]);
}

// =========================================================================
// 13. Complete sets for arithmetic grammar
// =========================================================================
#[test]
fn arithmetic_grammar_first_sets() {
    // E → E + T | T,  T → T * F | F,  F → ( E ) | num
    let mut g = Grammar::new("arith".into());
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

    // FIRST(F) = { (, num }
    assert_first_eq(&ff, NT_F, &[T_LPAREN, T_NUM]);
    // FIRST(T) = FIRST(F) = { (, num }
    assert_first_eq(&ff, NT_T, &[T_LPAREN, T_NUM]);
    // FIRST(E) = FIRST(T) = { (, num }
    assert_first_eq(&ff, NT_E, &[T_LPAREN, T_NUM]);

    // None are nullable
    assert!(!ff.is_nullable(NT_E));
    assert!(!ff.is_nullable(NT_T));
    assert!(!ff.is_nullable(NT_F));
}

#[test]
fn arithmetic_grammar_follow_sets() {
    // Same grammar as above
    let mut g = Grammar::new("arith".into());
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

    // FOLLOW(E) ⊇ {$, ), +}
    assert_follow_contains(&ff, NT_E, &[EOF, T_RPAREN, T_PLUS]);
    // FOLLOW(T) ⊇ {+, ), $, *}
    assert_follow_contains(&ff, NT_T, &[T_PLUS, T_RPAREN, EOF, T_STAR]);
    // FOLLOW(F) ⊇ {+, *, ), $}
    assert_follow_contains(&ff, NT_F, &[T_PLUS, T_STAR, T_RPAREN, EOF]);
}

// =========================================================================
// 14. Sets for expression grammar with precedence
// =========================================================================
#[test]
fn precedence_grammar_first_sets() {
    // Same structure as arith, but with precedence annotations.
    // FIRST/FOLLOW should be identical regardless of precedence metadata.
    let mut g = Grammar::new("prec".into());
    tok(&mut g, T_PLUS, "+", "+");
    tok(&mut g, T_STAR, "*", "*");
    tok(&mut g, T_NUM, "num", "[0-9]+");
    tok(&mut g, T_LPAREN, "(", "(");
    tok(&mut g, T_RPAREN, ")", ")");
    g.rule_names.insert(NT_E, "E".into());
    g.rule_names.insert(NT_T, "T".into());
    g.rule_names.insert(NT_F, "F".into());
    g.rules.insert(
        NT_E,
        vec![
            rule_prec(
                NT_E,
                vec![
                    Symbol::NonTerminal(NT_E),
                    Symbol::Terminal(T_PLUS),
                    Symbol::NonTerminal(NT_T),
                ],
                0,
                1,
                Some(Associativity::Left),
            ),
            rule(NT_E, vec![Symbol::NonTerminal(NT_T)], 1),
        ],
    );
    g.rules.insert(
        NT_T,
        vec![
            rule_prec(
                NT_T,
                vec![
                    Symbol::NonTerminal(NT_T),
                    Symbol::Terminal(T_STAR),
                    Symbol::NonTerminal(NT_F),
                ],
                2,
                2,
                Some(Associativity::Left),
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
    assert_first_eq(&ff, NT_E, &[T_LPAREN, T_NUM]);
    assert_first_eq(&ff, NT_T, &[T_LPAREN, T_NUM]);
    assert_first_eq(&ff, NT_F, &[T_LPAREN, T_NUM]);
}

#[test]
fn precedence_grammar_follow_sets() {
    let mut g = Grammar::new("prec".into());
    tok(&mut g, T_PLUS, "+", "+");
    tok(&mut g, T_STAR, "*", "*");
    tok(&mut g, T_NUM, "num", "[0-9]+");
    tok(&mut g, T_LPAREN, "(", "(");
    tok(&mut g, T_RPAREN, ")", ")");
    g.rule_names.insert(NT_E, "E".into());
    g.rule_names.insert(NT_T, "T".into());
    g.rule_names.insert(NT_F, "F".into());
    g.rules.insert(
        NT_E,
        vec![
            rule_prec(
                NT_E,
                vec![
                    Symbol::NonTerminal(NT_E),
                    Symbol::Terminal(T_PLUS),
                    Symbol::NonTerminal(NT_T),
                ],
                0,
                1,
                Some(Associativity::Left),
            ),
            rule(NT_E, vec![Symbol::NonTerminal(NT_T)], 1),
        ],
    );
    g.rules.insert(
        NT_T,
        vec![
            rule_prec(
                NT_T,
                vec![
                    Symbol::NonTerminal(NT_T),
                    Symbol::Terminal(T_STAR),
                    Symbol::NonTerminal(NT_F),
                ],
                2,
                2,
                Some(Associativity::Left),
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
    assert_follow_contains(&ff, NT_E, &[EOF, T_RPAREN, T_PLUS]);
    assert_follow_contains(&ff, NT_T, &[T_PLUS, T_RPAREN, EOF, T_STAR]);
    assert_follow_contains(&ff, NT_F, &[T_PLUS, T_STAR, T_RPAREN, EOF]);
}

// =========================================================================
// 15. Determinism – sets identical across multiple computations
// =========================================================================
#[test]
fn determinism_multiple_computations() {
    let build = || {
        let mut g = Grammar::new("det".into());
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
        g
    };

    let ff1 = FirstFollowSets::compute(&build()).unwrap();
    let ff2 = FirstFollowSets::compute(&build()).unwrap();
    let ff3 = FirstFollowSets::compute(&build()).unwrap();

    let bits_first = |ff: &FirstFollowSets, sym: SymbolId| -> Vec<usize> {
        let set = ff.first(sym).unwrap();
        (0..set.len()).filter(|&i| set.contains(i)).collect()
    };
    let bits_follow = |ff: &FirstFollowSets, sym: SymbolId| -> Vec<usize> {
        let set = ff.follow(sym).unwrap();
        (0..set.len()).filter(|&i| set.contains(i)).collect()
    };

    for sym in [NT_S, NT_A, NT_B] {
        assert_eq!(
            bits_first(&ff1, sym),
            bits_first(&ff2, sym),
            "FIRST({sym:?}) differs between run 1 and 2"
        );
        assert_eq!(
            bits_first(&ff2, sym),
            bits_first(&ff3, sym),
            "FIRST({sym:?}) differs between run 2 and 3"
        );
        assert_eq!(
            bits_follow(&ff1, sym),
            bits_follow(&ff2, sym),
            "FOLLOW({sym:?}) differs between run 1 and 2"
        );
        assert_eq!(
            bits_follow(&ff2, sym),
            bits_follow(&ff3, sym),
            "FOLLOW({sym:?}) differs between run 2 and 3"
        );
        assert_eq!(
            ff1.is_nullable(sym),
            ff2.is_nullable(sym),
            "nullable({sym:?}) differs"
        );
    }
}

// =========================================================================
// 16. Multiple terminals in choice
// =========================================================================
#[test]
fn first_multiple_terminal_alternatives() {
    // A → a | b | c
    let mut g = Grammar::new("t16".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_A, "A".into());
    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::Terminal(T_A)], 0),
            rule(NT_A, vec![Symbol::Terminal(T_B)], 1),
            rule(NT_A, vec![Symbol::Terminal(T_C)], 2),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_A, &[T_A, T_B, T_C]);
}

// =========================================================================
// 17. Deeply chained non-terminals
// =========================================================================
#[test]
fn first_deep_chain() {
    // S → A, A → B, B → C, C → a
    let mut g = Grammar::new("t17".into());
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
    for sym in [NT_S, NT_A, NT_B, NT_C] {
        assert_first_eq(&ff, sym, &[T_A]);
    }
}

// =========================================================================
// 18. Nullable prefix propagation
// =========================================================================
#[test]
fn first_nullable_prefix() {
    // S → A B c,  A → ε,  B → ε | b
    // FIRST(S) should include FIRST(A) ∪ FIRST(B) ∪ {c} since both A and B nullable
    let mut g = Grammar::new("t18".into());
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
    assert!(ff.is_nullable(NT_A));
    assert!(ff.is_nullable(NT_B));
    assert!(!ff.is_nullable(NT_S)); // S is not nullable since terminal c is required
    assert_first_eq(&ff, NT_S, &[T_B, T_C]);
}

// =========================================================================
// 19. Epsilon-only rule
// =========================================================================
#[test]
fn nullable_only_rule() {
    // A → ε
    let mut g = Grammar::new("t19".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Epsilon], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
    // FIRST(A) should be empty (no terminals)
    let first = ff.first(NT_A).unwrap();
    assert_eq!(
        first.count_ones(..),
        0,
        "FIRST of epsilon-only rule should be empty"
    );
}

// =========================================================================
// 20. FOLLOW propagation through nullable tail
// =========================================================================
#[test]
fn follow_through_nullable_tail() {
    // S → A B C,  A → a,  B → b,  C → ε
    // FOLLOW(B) ⊇ FIRST(C) ∪ FOLLOW(S) since C is nullable
    let mut g = Grammar::new("t20".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
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
        .insert(NT_C, vec![rule(NT_C, vec![Symbol::Epsilon], 3)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    // FOLLOW(B) should include EOF (from FOLLOW(S) propagated through nullable C)
    assert_follow_contains(&ff, NT_B, &[EOF]);
}

// =========================================================================
// 21. FOLLOW propagation across multiple nullable
// =========================================================================
#[test]
fn follow_multiple_nullable_suffix() {
    // S → A B C D,  A → a,  B → b,  C → ε,  D → ε
    // FOLLOW(B) ⊇ FOLLOW(S) since both C and D are nullable
    let mut g = Grammar::new("t21".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rule_names.insert(NT_D, "D".into());
    g.rules.insert(
        NT_S,
        vec![rule(
            NT_S,
            vec![
                Symbol::NonTerminal(NT_A),
                Symbol::NonTerminal(NT_B),
                Symbol::NonTerminal(NT_C),
                Symbol::NonTerminal(NT_D),
            ],
            0,
        )],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_B)], 2)]);
    g.rules
        .insert(NT_C, vec![rule(NT_C, vec![Symbol::Epsilon], 3)]);
    g.rules
        .insert(NT_D, vec![rule(NT_D, vec![Symbol::Epsilon], 4)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_B, &[EOF]);
    assert_follow_contains(&ff, NT_A, &[T_B]);
}

// =========================================================================
// 22. first_of_sequence API
// =========================================================================
#[test]
fn first_of_sequence_api() {
    let mut g = Grammar::new("t22".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::Terminal(T_A)], 0),
            rule(NT_A, vec![Symbol::Epsilon], 1),
        ],
    );
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_B)], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();

    // FIRST of sequence [A, B] = {a, b}  (A is nullable so B's FIRST contributes)
    let seq = vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_B)];
    let first_seq = ff.first_of_sequence(&seq).unwrap();
    assert!(first_seq.contains(T_A.0 as usize));
    assert!(first_seq.contains(T_B.0 as usize));
}

// =========================================================================
// 23. first_of_sequence with terminal prefix
// =========================================================================
#[test]
fn first_of_sequence_terminal_prefix() {
    let mut g = Grammar::new("t23".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_B, "B".into());
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_B)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();

    // FIRST of [a, B] = {a}
    let seq = vec![Symbol::Terminal(T_A), Symbol::NonTerminal(NT_B)];
    let first_seq = ff.first_of_sequence(&seq).unwrap();
    assert!(first_seq.contains(T_A.0 as usize));
    assert!(!first_seq.contains(T_B.0 as usize));
}

// =========================================================================
// 24. Mutual recursion: A → B a, B → A b | c
// =========================================================================
#[test]
fn first_mutual_recursion() {
    let mut g = Grammar::new("t24".into());
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
    // Both A and B should have FIRST = {c}  (only way to start a derivation)
    assert_first_eq(&ff, NT_A, &[T_C]);
    assert_first_eq(&ff, NT_B, &[T_C]);
}

// =========================================================================
// 25. FOLLOW set of terminal at end of production
// =========================================================================
#[test]
fn follow_last_nonterminal_gets_lhs_follow() {
    // S → a A,  A → b
    // A is last in S's production, so FOLLOW(A) ⊇ FOLLOW(S) = {$}
    let mut g = Grammar::new("t25".into());
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
// 26. Entirely nullable grammar
// =========================================================================
#[test]
fn entirely_nullable_grammar() {
    // S → A B,  A → ε,  B → ε
    let mut g = Grammar::new("t26".into());
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
    // FIRST sets should all be empty
    assert_eq!(ff.first(NT_S).unwrap().count_ones(..), 0);
    assert_eq!(ff.first(NT_A).unwrap().count_ones(..), 0);
    assert_eq!(ff.first(NT_B).unwrap().count_ones(..), 0);
}

// =========================================================================
// 27. compute_normalized works with Choice symbol
// =========================================================================
#[test]
fn compute_normalized_with_choice() {
    // S → (a | b)  using Choice symbol, needs normalization
    let mut g = Grammar::new("t27".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![Rule {
            lhs: NT_S,
            rhs: vec![Symbol::Choice(vec![
                Symbol::Terminal(T_A),
                Symbol::Terminal(T_B),
            ])],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let first_s = ff.first(NT_S).unwrap();
    assert!(first_s.contains(T_A.0 as usize));
    assert!(first_s.contains(T_B.0 as usize));
}

// =========================================================================
// 28. FOLLOW with multiple productions referencing the same non-terminal
// =========================================================================
#[test]
fn follow_from_multiple_productions() {
    // S → A b | A c,  A → a
    // FOLLOW(A) = {b, c}
    let mut g = Grammar::new("t28".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
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
                vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_C)],
                1,
            ),
        ],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 2)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_follow_contains(&ff, NT_A, &[T_B, T_C]);
}

// =========================================================================
// 29. Non-start symbol does NOT automatically get EOF in FOLLOW
// =========================================================================
#[test]
fn follow_non_start_no_eof() {
    // S → A b,  A → a
    // FOLLOW(A) = {b}, no EOF since A is not the start and 'b' always follows.
    let mut g = Grammar::new("t29".into());
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
    assert_follow_eq(&ff, NT_A, &[T_B]);
}

// =========================================================================
// 30. Left-recursive FOLLOW propagation: S → S a | b
// =========================================================================
#[test]
fn follow_left_recursive_start() {
    // S → S a | b
    // FOLLOW(S) = {$, a}  (EOF for start, 'a' from S → S a)
    let mut g = Grammar::new("t30".into());
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
    assert_first_eq(&ff, NT_S, &[T_B]);
    assert_follow_contains(&ff, NT_S, &[EOF, T_A]);
}

// =========================================================================
// 31. Single-rule grammar: one non-terminal, one terminal
// =========================================================================
#[test]
fn single_rule_grammar() {
    // S → a
    let mut g = Grammar::new("single".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert!(!ff.is_nullable(NT_S));
    assert_follow_contains(&ff, NT_S, &[EOF]);
}

// =========================================================================
// 32. Empty grammar: no rules at all
// =========================================================================
#[test]
fn empty_grammar_no_rules() {
    let g = Grammar::new("empty".into());
    let ff = FirstFollowSets::compute(&g).unwrap();
    // No symbols to query; just ensure it doesn't panic
    assert!(ff.first(NT_S).is_none());
    assert!(ff.follow(NT_S).is_none());
    assert!(!ff.is_nullable(NT_S));
}

// =========================================================================
// 33. Optional symbol via compute_normalized (Repeat)
// =========================================================================
#[test]
fn compute_normalized_with_repeat() {
    // S → a*  (zero or more 'a')
    let mut g = Grammar::new("t33".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![Rule {
            lhs: NT_S,
            rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(T_A)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    // S should have 'a' in FIRST (from the repeat)
    let first_s = ff.first(NT_S).unwrap();
    assert!(first_s.contains(T_A.0 as usize));
    // S should be nullable (Repeat allows zero occurrences)
    assert!(ff.is_nullable(NT_S));
}

// =========================================================================
// 34. Optional symbol via compute_normalized (Optional)
// =========================================================================
#[test]
fn compute_normalized_with_optional() {
    // S → a?  (optional 'a')
    let mut g = Grammar::new("t34".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules.insert(
        NT_S,
        vec![Rule {
            lhs: NT_S,
            rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(T_A)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let first_s = ff.first(NT_S).unwrap();
    assert!(first_s.contains(T_A.0 as usize));
    assert!(ff.is_nullable(NT_S));
}

// =========================================================================
// 35. Consistency: FIRST(terminal) contains only itself
// =========================================================================
#[test]
fn terminal_first_set_is_itself() {
    let mut g = Grammar::new("t35".into());
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
    // Terminal FIRST sets are only populated for non-terminals; terminals
    // implicitly have FIRST = {themselves}. Verify via first_of_sequence.
    let seq = vec![Symbol::Terminal(T_A)];
    let fs = ff.first_of_sequence(&seq).unwrap();
    assert!(fs.contains(T_A.0 as usize));
    assert!(!fs.contains(T_B.0 as usize));
}

// =========================================================================
// 36. Consistency: non-nullable non-terminal never has empty FIRST
// =========================================================================
#[test]
fn non_nullable_has_nonempty_first() {
    // S → A B,  A → a,  B → b | c
    let mut g = Grammar::new("t36".into());
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
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 1)]);
    g.rules.insert(
        NT_B,
        vec![
            rule(NT_B, vec![Symbol::Terminal(T_B)], 2),
            rule(NT_B, vec![Symbol::Terminal(T_C)], 3),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    for sym in [NT_S, NT_A, NT_B] {
        assert!(!ff.is_nullable(sym));
        assert!(
            ff.first(sym).unwrap().count_ones(..) > 0,
            "non-nullable {sym:?} must have non-empty FIRST"
        );
    }
}

// =========================================================================
// 37. Consistency: FIRST(A) ⊆ FIRST(S) when S → A (chain rule)
// =========================================================================
#[test]
fn first_subset_through_chain() {
    // S → A,  A → a | b
    let mut g = Grammar::new("t37".into());
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
    let first_a = ff.first(NT_A).unwrap();
    let first_s = ff.first(NT_S).unwrap();
    // Every element in FIRST(A) must be in FIRST(S)
    for i in 0..first_a.len() {
        if first_a.contains(i) {
            assert!(
                first_s.contains(i),
                "FIRST(A) element {i} not found in FIRST(S)"
            );
        }
    }
}

// =========================================================================
// 38. Nullable chain: S → A, A → B, B → ε
// =========================================================================
#[test]
fn nullable_chain_propagation() {
    // S → A,  A → B,  B → ε
    // All should be nullable
    let mut g = Grammar::new("t38".into());
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
// 39. FOLLOW propagation through deep chain
// =========================================================================
#[test]
fn follow_deep_chain_propagation() {
    // S → A,  A → B,  B → C,  C → a
    // FOLLOW(C) ⊇ FOLLOW(B) ⊇ FOLLOW(A) ⊇ FOLLOW(S) = {$}
    let mut g = Grammar::new("t39".into());
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
    for sym in [NT_S, NT_A, NT_B, NT_C] {
        assert_follow_contains(&ff, sym, &[EOF]);
    }
}

// =========================================================================
// 40. first_of_sequence with empty sequence
// =========================================================================
#[test]
fn first_of_sequence_empty() {
    let mut g = Grammar::new("t40".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_A, "A".into());
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    let empty_seq: Vec<Symbol> = vec![];
    let first_seq = ff.first_of_sequence(&empty_seq).unwrap();
    assert_eq!(
        first_seq.count_ones(..),
        0,
        "FIRST of empty sequence should be empty"
    );
}

// =========================================================================
// 41. GrammarBuilder-based test: arithmetic via builder
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

    // Find symbol IDs via rule_names
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

    // FIRST(expr) and FIRST(term) should both contain NUMBER
    assert!(ff.first(expr_id).unwrap().contains(number_id.0 as usize));
    assert!(ff.first(term_id).unwrap().contains(number_id.0 as usize));
    assert!(!ff.is_nullable(expr_id));
    assert!(!ff.is_nullable(term_id));
}

// =========================================================================
// 42. GrammarBuilder: nullable start (python-like)
// =========================================================================
#[test]
fn builder_python_like_nullable_start() {
    use adze_ir::builder::GrammarBuilder;

    let grammar = GrammarBuilder::python_like();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let module_id = grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "module")
        .map(|(id, _)| *id)
        .unwrap();

    // module is nullable (has ε production)
    assert!(ff.is_nullable(module_id));
    // FOLLOW(module) should contain EOF since it's the start symbol
    assert_follow_contains(&ff, module_id, &[EOF]);
}

// =========================================================================
// 43. Consistency: FOLLOW(A) ⊆ FOLLOW(S) when S → A and no other context
// =========================================================================
#[test]
fn follow_subset_single_chain() {
    // S → A,  A → a | b
    // Since A only appears in S → A, FOLLOW(A) = FOLLOW(S)
    let mut g = Grammar::new("t43".into());
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
    // FOLLOW(A) should be a superset of FOLLOW(S) (they should be equal here)
    for i in 0..follow_s.len() {
        if follow_s.contains(i) {
            assert!(
                follow_a.contains(i),
                "FOLLOW(S) element {i} not in FOLLOW(A)"
            );
        }
    }
}

// =========================================================================
// 44. Middle nonterminal FOLLOW: S → A B C
// =========================================================================
#[test]
fn follow_middle_nonterminal() {
    // S → A B C,  A → a,  B → b,  C → c
    // FOLLOW(B) should contain FIRST(C) = {c}
    let mut g = Grammar::new("t44".into());
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
    assert_follow_contains(&ff, NT_B, &[T_C]);
    // B is not at end, so should not get EOF unless C is nullable
    assert!(!ff.is_nullable(NT_C));
}

// =========================================================================
// 45. Nullable with terminal alternative: A → ε | a, not both
// =========================================================================
#[test]
fn nullable_with_terminal_alternative() {
    // S → A B,  A → ε | a,  B → b
    // S is not nullable (B not nullable)
    // FIRST(S) = {a, b} (A nullable → B's first contributes)
    let mut g = Grammar::new("t45".into());
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
    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::Epsilon], 1),
            rule(NT_A, vec![Symbol::Terminal(T_A)], 2),
        ],
    );
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Terminal(T_B)], 3)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
    assert!(!ff.is_nullable(NT_B));
    assert!(!ff.is_nullable(NT_S));
    assert_first_eq(&ff, NT_S, &[T_A, T_B]);
}
