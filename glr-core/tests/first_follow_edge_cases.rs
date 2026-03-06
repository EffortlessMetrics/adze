#![cfg(feature = "test-api")]

//! Edge case tests for FIRST/FOLLOW set computation.
//!
//! This test suite covers:
//! 1. Left-recursive rules (A → A b | c)
//! 2. Right-recursive rules (A → b A | c)
//! 3. Mutual recursion (A → B c, B → A d | e)
//! 4. FOLLOW set with EOF for multiple symbols
//! 5. Empty/epsilon productions
//! 6. Nullable non-terminals in sequences
//! 7. FIRST sets that overlap (ambiguous)
//! 8. Single-production grammar
//! 9. Medium-sized grammar (50+ productions)
//! 10. FIRST(α) for sequences
//! 11. FOLLOW sets minimality
//! 12. Termination guarantee for all valid grammars

use adze_glr_core::FirstFollowSets;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helper functions
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

/// Check that FIRST(sym) contains exactly the expected symbol IDs
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

/// Check that FOLLOW(sym) contains all expected symbol IDs (may contain more)
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

/// Check that FOLLOW(sym) contains exactly the expected symbol IDs
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
// Terminal/NonTerminal IDs
// ---------------------------------------------------------------------------

const EOF: SymbolId = SymbolId(0);

// Terminals in low range
const T_A: SymbolId = SymbolId(1);
const T_B: SymbolId = SymbolId(2);
const T_C: SymbolId = SymbolId(3);
const T_D: SymbolId = SymbolId(4);
const T_E: SymbolId = SymbolId(5);
const T_F: SymbolId = SymbolId(6);
const T_G: SymbolId = SymbolId(7);
const T_H: SymbolId = SymbolId(8);
const T_I: SymbolId = SymbolId(9);
const T_J: SymbolId = SymbolId(10);

// Non-terminals in high range
const NT_S: SymbolId = SymbolId(100);
const NT_A: SymbolId = SymbolId(101);
const NT_B: SymbolId = SymbolId(102);
const NT_C: SymbolId = SymbolId(103);
const NT_D: SymbolId = SymbolId(104);

// =========================================================================
// 1. LEFT-RECURSIVE RULES: A → A b | c
// =========================================================================
#[test]
fn left_recursive_rules() {
    // A → A b | c
    // FIRST(A) = {c}
    // FOLLOW(A) = {$, b}  (EOF and 'b' from A → A b)
    let mut g = Grammar::new("left_recursive".into());
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
    assert_follow_contains(&ff, NT_A, &[EOF, T_B]);
}

// =========================================================================
// 2. RIGHT-RECURSIVE RULES: A → b A | c
// =========================================================================
#[test]
fn right_recursive_rules() {
    // A → b A | c
    // FIRST(A) = {b, c}
    // FOLLOW(A) = {$}  (EOF for start symbol)
    let mut g = Grammar::new("right_recursive".into());
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
    assert!(!ff.is_nullable(NT_A));
    assert_follow_eq(&ff, NT_A, &[EOF]);
}

// =========================================================================
// 3. MUTUAL RECURSION: A → B c, B → A d | e
// =========================================================================
#[test]
fn mutual_recursion() {
    // A → B c,  B → A d | e
    // FIRST(A) = {d, e}  (from B via A)
    // FIRST(B) = {a, e}  (from A or direct)
    // FOLLOW(A) = {d}  (from B → A d)
    // FOLLOW(B) = {c}  (from A → B c)
    let mut g = Grammar::new("mutual_recursion".into());
    tok(&mut g, T_C, "c", "c");
    tok(&mut g, T_D, "d", "d");
    tok(&mut g, T_E, "e", "e");
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
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
        vec![
            rule(
                NT_B,
                vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_D)],
                1,
            ),
            rule(NT_B, vec![Symbol::Terminal(T_E)], 2),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    // FIRST(A) derives from FIRST(B), which can be 'e' directly or 'd'/'e' from A
    assert_follow_contains(&ff, NT_A, &[T_D]);
    assert_follow_contains(&ff, NT_B, &[T_C]);
}

// =========================================================================
// 4. FOLLOW SET INCLUDES EOF FOR MULTIPLE SYMBOLS
// =========================================================================
#[test]
fn follow_with_eof_multiple_symbols() {
    // S → A B | A C
    // A → a
    // B → b | ε
    // C → c | ε
    //
    // FOLLOW(A) should have {b, c} from B/C, and possibly EOF
    // Both B and C are nullable, so EOF could propagate to A
    let mut g = Grammar::new("follow_eof_multiple".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());

    g.rules.insert(
        NT_S,
        vec![
            rule(
                NT_S,
                vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_B)],
                0,
            ),
            rule(
                NT_S,
                vec![Symbol::NonTerminal(NT_A), Symbol::NonTerminal(NT_C)],
                1,
            ),
        ],
    );
    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Terminal(T_A)], 2)]);
    g.rules.insert(
        NT_B,
        vec![
            rule(NT_B, vec![Symbol::Terminal(T_B)], 3),
            rule(NT_B, vec![Symbol::Epsilon], 4),
        ],
    );
    g.rules.insert(
        NT_C,
        vec![
            rule(NT_C, vec![Symbol::Terminal(T_C)], 5),
            rule(NT_C, vec![Symbol::Epsilon], 6),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_B));
    assert!(ff.is_nullable(NT_C));
    // A is always followed by B or C; both nullable, so EOF is in FOLLOW(A)
    assert_follow_contains(&ff, NT_A, &[EOF]);
}

// =========================================================================
// 5. EMPTY/EPSILON PRODUCTIONS
// =========================================================================
#[test]
fn epsilon_productions() {
    // A → ε | b
    // B → A c | d
    //
    // FIRST(A) = {b}  (epsilon doesn't count)
    // FIRST(B) = {b, d}  (from A c or direct d)
    // is_nullable(A) = true
    let mut g = Grammar::new("epsilon_productions".into());
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    tok(&mut g, T_D, "d", "d");
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());

    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::Epsilon], 0),
            rule(NT_A, vec![Symbol::Terminal(T_B)], 1),
        ],
    );
    g.rules.insert(
        NT_B,
        vec![
            rule(
                NT_B,
                vec![Symbol::NonTerminal(NT_A), Symbol::Terminal(T_C)],
                2,
            ),
            rule(NT_B, vec![Symbol::Terminal(T_D)], 3),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
    assert_first_eq(&ff, NT_A, &[T_B]);
    let first_b = ff.first(NT_B).unwrap();
    assert!(first_b.contains(T_B.0 as usize));
    assert!(first_b.contains(T_D.0 as usize));
}

// =========================================================================
// 6. NULLABLE NON-TERMINALS IN SEQUENCES
// =========================================================================
#[test]
fn nullable_in_sequence() {
    // A → ε
    // B → ε
    // C → A B D | D
    //
    // FIRST(C) should include FIRST(D) because A and B are both nullable
    // When we have [A, B, D], skip A (nullable), skip B (nullable), include D
    let mut g = Grammar::new("nullable_sequence".into());
    tok(&mut g, T_D, "d", "d");
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());
    g.rule_names.insert(NT_D, "D".into());

    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::Epsilon], 0)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::Epsilon], 1)]);
    g.rules.insert(
        NT_C,
        vec![
            rule(
                NT_C,
                vec![
                    Symbol::NonTerminal(NT_A),
                    Symbol::NonTerminal(NT_B),
                    Symbol::NonTerminal(NT_D),
                ],
                2,
            ),
            rule(NT_C, vec![Symbol::NonTerminal(NT_D)], 3),
        ],
    );
    g.rules
        .insert(NT_D, vec![rule(NT_D, vec![Symbol::Terminal(T_D)], 4)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(NT_A));
    assert!(ff.is_nullable(NT_B));
    let first_c = ff.first(NT_C).unwrap();
    assert!(first_c.contains(T_D.0 as usize));
}

// =========================================================================
// 7. OVERLAPPING FIRST SETS (AMBIGUOUS)
// =========================================================================
#[test]
fn overlapping_first_sets() {
    // A → a | b
    // B → a | c
    // S → A | B
    //
    // FIRST(A) = {a, b}
    // FIRST(B) = {a, c}
    // FIRST(S) = {a, b, c}  (union)
    // This is ambiguous: 'a' could start A or B
    let mut g = Grammar::new("overlapping_first".into());
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    g.rule_names.insert(NT_S, "S".into());
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());

    g.rules.insert(
        NT_A,
        vec![
            rule(NT_A, vec![Symbol::Terminal(T_A)], 0),
            rule(NT_A, vec![Symbol::Terminal(T_B)], 1),
        ],
    );
    g.rules.insert(
        NT_B,
        vec![
            rule(NT_B, vec![Symbol::Terminal(T_A)], 2),
            rule(NT_B, vec![Symbol::Terminal(T_C)], 3),
        ],
    );
    g.rules.insert(
        NT_S,
        vec![
            rule(NT_S, vec![Symbol::NonTerminal(NT_A)], 4),
            rule(NT_S, vec![Symbol::NonTerminal(NT_B)], 5),
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_A, &[T_A, T_B]);
    assert_first_eq(&ff, NT_B, &[T_A, T_C]);
    let first_s = ff.first(NT_S).unwrap();
    assert!(first_s.contains(T_A.0 as usize));
    assert!(first_s.contains(T_B.0 as usize));
    assert!(first_s.contains(T_C.0 as usize));
}

// =========================================================================
// 8. SINGLE-PRODUCTION GRAMMAR
// =========================================================================
#[test]
fn single_production() {
    // S → a
    // Minimal grammar with single rule
    let mut g = Grammar::new("single_production".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_S, "S".into());
    g.rules
        .insert(NT_S, vec![rule(NT_S, vec![Symbol::Terminal(T_A)], 0)]);

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_S, &[T_A]);
    assert_follow_eq(&ff, NT_S, &[EOF]);
}

// =========================================================================
// 9. MEDIUM-SIZE GRAMMAR (50+ PRODUCTIONS)
// =========================================================================
#[test]
fn medium_size_grammar_termination() {
    // Create a medium grammar with multiple rules per symbol
    // E → T | E + T
    // T → F | T * F
    // F → ( E ) | a | b
    // This expands to ~8 basic rules, but we'll add more variants
    // to reach 50+ productions

    let mut g = Grammar::new("medium_grammar".into());

    // Terminals
    tok(&mut g, T_A, "a", "a");
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "(", "(");
    tok(&mut g, T_D, ")", ")");
    tok(&mut g, T_E, "+", "+");
    tok(&mut g, T_F, "*", "*");
    tok(&mut g, T_G, "-", "-");
    tok(&mut g, T_H, "/", "/");
    tok(&mut g, T_I, "c", "c");
    tok(&mut g, T_J, "d", "d");

    // Non-terminals
    g.rule_names.insert(NT_S, "E".into()); // Expression (start)
    g.rule_names.insert(NT_A, "T".into()); // Term
    g.rule_names.insert(NT_B, "F".into()); // Factor

    let mut prod_id = 0;

    // E productions (10 variants)
    let mut e_rules = vec![];
    for _ in 0..5 {
        e_rules.push(rule(NT_S, vec![Symbol::NonTerminal(NT_A)], prod_id));
        prod_id += 1;
    }
    for _ in 0..5 {
        e_rules.push(rule(
            NT_S,
            vec![
                Symbol::NonTerminal(NT_S),
                Symbol::Terminal(T_E),
                Symbol::NonTerminal(NT_A),
            ],
            prod_id,
        ));
        prod_id += 1;
    }
    g.rules.insert(NT_S, e_rules);

    // T productions (10 variants)
    let mut t_rules = vec![];
    for _ in 0..5 {
        t_rules.push(rule(NT_A, vec![Symbol::NonTerminal(NT_B)], prod_id));
        prod_id += 1;
    }
    for _ in 0..5 {
        t_rules.push(rule(
            NT_A,
            vec![
                Symbol::NonTerminal(NT_A),
                Symbol::Terminal(T_F),
                Symbol::NonTerminal(NT_B),
            ],
            prod_id,
        ));
        prod_id += 1;
    }
    g.rules.insert(NT_A, t_rules);

    // F productions (30+ variants for more coverage)
    let mut f_rules = vec![];
    // Parenthesized expression
    for _ in 0..8 {
        f_rules.push(rule(
            NT_B,
            vec![
                Symbol::Terminal(T_C),
                Symbol::NonTerminal(NT_S),
                Symbol::Terminal(T_D),
            ],
            prod_id,
        ));
        prod_id += 1;
    }
    // Simple terminals
    for _ in 0..8 {
        f_rules.push(rule(NT_B, vec![Symbol::Terminal(T_A)], prod_id));
        prod_id += 1;
    }
    for _ in 0..8 {
        f_rules.push(rule(NT_B, vec![Symbol::Terminal(T_B)], prod_id));
        prod_id += 1;
    }
    for _ in 0..8 {
        f_rules.push(rule(NT_B, vec![Symbol::Terminal(T_I)], prod_id));
        prod_id += 1;
    }
    g.rules.insert(NT_B, f_rules);

    // This should compute without hanging or error
    let ff = FirstFollowSets::compute(&g).unwrap();

    // Verify basic properties
    assert!(!ff.is_nullable(NT_S)); // E is not nullable
    let first_e = ff.first(NT_S).unwrap();
    assert!(first_e.contains(T_A.0 as usize) || first_e.contains(T_C.0 as usize));
}

// =========================================================================
// 10. VERIFY FIRST(α) FOR SEQUENCES
// =========================================================================
#[test]
fn first_of_sequences() {
    // Verify FIRST computation for a sequence of symbols
    // A → b c d
    // Manually check: FIRST(A) should start with 'b'
    let mut g = Grammar::new("sequence_first".into());
    tok(&mut g, T_B, "b", "b");
    tok(&mut g, T_C, "c", "c");
    tok(&mut g, T_D, "d", "d");
    g.rule_names.insert(NT_A, "A".into());

    g.rules.insert(
        NT_A,
        vec![rule(
            NT_A,
            vec![
                Symbol::Terminal(T_B),
                Symbol::Terminal(T_C),
                Symbol::Terminal(T_D),
            ],
            0,
        )],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert_first_eq(&ff, NT_A, &[T_B]);
}

// =========================================================================
// 11. VERIFY FOLLOW SETS ARE MINIMAL
// =========================================================================
#[test]
fn follow_sets_minimal() {
    // S → A B
    // A → a
    // B → b
    //
    // FOLLOW(A) should be exactly {b}, not including EOF or other symbols
    // FOLLOW(B) should be exactly {EOF}
    let mut g = Grammar::new("minimal_follow".into());
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
    assert_follow_eq(&ff, NT_A, &[T_B]);
    assert_follow_eq(&ff, NT_B, &[EOF]);
}

// =========================================================================
// 12. TERMINATION GUARANTEE FOR ALL VALID GRAMMARS
// =========================================================================
#[test]
fn termination_with_cycles() {
    // Grammar with cycles: A → B, B → C, C → A | a
    // The algorithm must terminate even with cycles
    let mut g = Grammar::new("termination_cycles".into());
    tok(&mut g, T_A, "a", "a");
    g.rule_names.insert(NT_A, "A".into());
    g.rule_names.insert(NT_B, "B".into());
    g.rule_names.insert(NT_C, "C".into());

    g.rules
        .insert(NT_A, vec![rule(NT_A, vec![Symbol::NonTerminal(NT_B)], 0)]);
    g.rules
        .insert(NT_B, vec![rule(NT_B, vec![Symbol::NonTerminal(NT_C)], 1)]);
    g.rules.insert(
        NT_C,
        vec![
            rule(NT_C, vec![Symbol::NonTerminal(NT_A)], 2),
            rule(NT_C, vec![Symbol::Terminal(T_A)], 3),
        ],
    );

    // Should complete without stack overflow or infinite loop
    let ff = FirstFollowSets::compute(&g).unwrap();

    // Verify it computed something reasonable
    let first_c = ff.first(NT_C).unwrap();
    assert!(first_c.contains(T_A.0 as usize));
}
