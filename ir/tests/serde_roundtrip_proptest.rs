#![allow(clippy::needless_range_loop)]

//! Property-based serde roundtrip tests for all IR types.
//!
//! Each test serializes a value to JSON, deserializes it back, and asserts
//! the result equals the original. For types without `PartialEq` (e.g. `Grammar`),
//! we compare `serde_json::Value` representations.

use adze_ir::{
    AliasSequence, Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, FieldId,
    Grammar, Precedence, PrecedenceKind, ProductionId, Rule, RuleId, StateId, Symbol, SymbolId,
    SymbolMetadata, Token, TokenPattern,
};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (0u16..500).prop_map(SymbolId)
}

fn arb_field_id() -> impl Strategy<Value = FieldId> {
    (0u16..100).prop_map(FieldId)
}

fn arb_production_id() -> impl Strategy<Value = ProductionId> {
    (0u16..100).prop_map(ProductionId)
}

fn arb_associativity() -> impl Strategy<Value = Associativity> {
    prop_oneof![
        Just(Associativity::Left),
        Just(Associativity::Right),
        Just(Associativity::None),
    ]
}

fn arb_precedence_kind() -> impl Strategy<Value = PrecedenceKind> {
    prop_oneof![
        (-100i16..100).prop_map(PrecedenceKind::Static),
        (-100i16..100).prop_map(PrecedenceKind::Dynamic),
    ]
}

fn arb_token_pattern() -> impl Strategy<Value = TokenPattern> {
    prop_oneof![
        "[a-zA-Z0-9_]{1,10}".prop_map(TokenPattern::String),
        "[a-zA-Z0-9_.+*?]{1,10}".prop_map(TokenPattern::Regex),
    ]
}

fn arb_symbol_leaf() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        arb_symbol_id().prop_map(Symbol::Terminal),
        arb_symbol_id().prop_map(Symbol::NonTerminal),
        arb_symbol_id().prop_map(Symbol::External),
        Just(Symbol::Epsilon),
    ]
}

fn arb_symbol() -> impl Strategy<Value = Symbol> {
    arb_symbol_leaf().prop_recursive(3, 16, 4, |inner| {
        prop_oneof![
            inner.clone().prop_map(|s| Symbol::Optional(Box::new(s))),
            inner.clone().prop_map(|s| Symbol::Repeat(Box::new(s))),
            inner.clone().prop_map(|s| Symbol::RepeatOne(Box::new(s))),
            prop::collection::vec(inner.clone(), 1..4).prop_map(Symbol::Choice),
            prop::collection::vec(inner, 1..4).prop_map(Symbol::Sequence),
        ]
    })
}

fn arb_token() -> impl Strategy<Value = Token> {
    (
        "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
        arb_token_pattern(),
        any::<bool>(),
    )
        .prop_map(|(name, pattern, fragile)| Token {
            name,
            pattern,
            fragile,
        })
}

fn arb_rule() -> impl Strategy<Value = Rule> {
    (
        arb_symbol_id(),
        prop::collection::vec(arb_symbol(), 0..5),
        prop::option::of(arb_precedence_kind()),
        prop::option::of(arb_associativity()),
        prop::collection::vec((arb_field_id(), 0usize..10), 0..3),
        arb_production_id(),
    )
        .prop_map(
            |(lhs, rhs, precedence, associativity, fields, production_id)| Rule {
                lhs,
                rhs,
                precedence,
                associativity,
                fields,
                production_id,
            },
        )
}

fn arb_symbol_metadata() -> impl Strategy<Value = SymbolMetadata> {
    (any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>()).prop_map(
        |(visible, named, hidden, terminal)| SymbolMetadata {
            visible,
            named,
            hidden,
            terminal,
        },
    )
}

fn arb_external_token() -> impl Strategy<Value = ExternalToken> {
    ("[a-zA-Z_][a-zA-Z0-9_]{0,10}", arb_symbol_id())
        .prop_map(|(name, symbol_id)| ExternalToken { name, symbol_id })
}

fn arb_precedence() -> impl Strategy<Value = Precedence> {
    (
        -100i16..100,
        arb_associativity(),
        prop::collection::vec(arb_symbol_id(), 0..5),
    )
        .prop_map(|(level, associativity, symbols)| Precedence {
            level,
            associativity,
            symbols,
        })
}

fn arb_conflict_resolution() -> impl Strategy<Value = ConflictResolution> {
    prop_oneof![
        arb_precedence_kind().prop_map(ConflictResolution::Precedence),
        arb_associativity().prop_map(ConflictResolution::Associativity),
        Just(ConflictResolution::GLR),
    ]
}

fn arb_conflict_declaration() -> impl Strategy<Value = ConflictDeclaration> {
    (
        prop::collection::vec(arb_symbol_id(), 1..5),
        arb_conflict_resolution(),
    )
        .prop_map(|(symbols, resolution)| ConflictDeclaration {
            symbols,
            resolution,
        })
}

fn arb_alias_sequence() -> impl Strategy<Value = AliasSequence> {
    prop::collection::vec(
        prop::option::of("[a-zA-Z_]{1,8}".prop_map(String::from)),
        0..6,
    )
    .prop_map(|aliases| AliasSequence { aliases })
}

/// Helper: roundtrip via JSON and compare `serde_json::Value` representations.
fn assert_json_value_roundtrip<T: serde::Serialize + serde::de::DeserializeOwned>(val: &T) -> T {
    let json = serde_json::to_string(val).expect("serialize");
    let back: T = serde_json::from_str(&json).expect("deserialize");
    let v1 = serde_json::to_value(val).expect("to_value original");
    let v2 = serde_json::to_value(&back).expect("to_value roundtrip");
    assert_eq!(v1, v2, "JSON roundtrip mismatch");
    back
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

proptest! {
    // ---- SymbolId ----
    #[test]
    fn roundtrip_symbol_id(id in 0u16..=u16::MAX) {
        let orig = SymbolId(id);
        let back = assert_json_value_roundtrip(&orig);
        prop_assert_eq!(orig, back);
    }

    // ---- RuleId ----
    #[test]
    fn roundtrip_rule_id(id in 0u16..=u16::MAX) {
        let orig = RuleId(id);
        let back = assert_json_value_roundtrip(&orig);
        prop_assert_eq!(orig, back);
    }

    // ---- StateId ----
    #[test]
    fn roundtrip_state_id(id in 0u16..=u16::MAX) {
        let orig = StateId(id);
        let back = assert_json_value_roundtrip(&orig);
        prop_assert_eq!(orig, back);
    }

    // ---- FieldId ----
    #[test]
    fn roundtrip_field_id(id in 0u16..=u16::MAX) {
        let orig = FieldId(id);
        let back = assert_json_value_roundtrip(&orig);
        prop_assert_eq!(orig, back);
    }

    // ---- ProductionId ----
    #[test]
    fn roundtrip_production_id(id in 0u16..=u16::MAX) {
        let orig = ProductionId(id);
        let back = assert_json_value_roundtrip(&orig);
        prop_assert_eq!(orig, back);
    }

    // ---- Associativity ----
    #[test]
    fn roundtrip_associativity(assoc in arb_associativity()) {
        let back = assert_json_value_roundtrip(&assoc);
        prop_assert_eq!(assoc, back);
    }

    // ---- PrecedenceKind ----
    #[test]
    fn roundtrip_precedence_kind(pk in arb_precedence_kind()) {
        let back = assert_json_value_roundtrip(&pk);
        prop_assert_eq!(pk, back);
    }

    // ---- TokenPattern::String ----
    #[test]
    fn roundtrip_token_pattern_string(s in "[a-zA-Z0-9_]{1,20}") {
        let orig = TokenPattern::String(s);
        let back = assert_json_value_roundtrip(&orig);
        prop_assert_eq!(orig, back);
    }

    // ---- TokenPattern::Regex ----
    #[test]
    fn roundtrip_token_pattern_regex(s in "[a-zA-Z0-9_.+*?]{1,20}") {
        let orig = TokenPattern::Regex(s);
        let back = assert_json_value_roundtrip(&orig);
        prop_assert_eq!(orig, back);
    }

    // ---- TokenPattern (mixed) ----
    #[test]
    fn roundtrip_token_pattern(pat in arb_token_pattern()) {
        let back = assert_json_value_roundtrip(&pat);
        prop_assert_eq!(pat, back);
    }

    // ---- Token ----
    #[test]
    fn roundtrip_token(tok in arb_token()) {
        let back = assert_json_value_roundtrip(&tok);
        prop_assert_eq!(tok, back);
    }

    // ---- Symbol leaf variants ----
    #[test]
    fn roundtrip_symbol_leaf(sym in arb_symbol_leaf()) {
        let back = assert_json_value_roundtrip(&sym);
        prop_assert_eq!(sym, back);
    }

    // ---- Symbol (recursive) ----
    #[test]
    fn roundtrip_symbol(sym in arb_symbol()) {
        let back = assert_json_value_roundtrip(&sym);
        prop_assert_eq!(sym, back);
    }

    // ---- Vec<Symbol> ----
    #[test]
    fn roundtrip_symbol_vec(syms in prop::collection::vec(arb_symbol(), 0..8)) {
        let back: Vec<Symbol> = assert_json_value_roundtrip(&syms);
        prop_assert_eq!(syms, back);
    }

    // ---- SymbolMetadata ----
    #[test]
    fn roundtrip_symbol_metadata(meta in arb_symbol_metadata()) {
        let back = assert_json_value_roundtrip(&meta);
        prop_assert_eq!(meta, back);
    }

    // ---- Rule ----
    #[test]
    fn roundtrip_rule(rule in arb_rule()) {
        let back = assert_json_value_roundtrip(&rule);
        prop_assert_eq!(rule, back);
    }

    // ---- ExternalToken ----
    #[test]
    fn roundtrip_external_token(et in arb_external_token()) {
        // ExternalToken doesn't derive PartialEq, compare via JSON Value
        assert_json_value_roundtrip(&et);
    }

    // ---- Precedence ----
    #[test]
    fn roundtrip_precedence(prec in arb_precedence()) {
        assert_json_value_roundtrip(&prec);
    }

    // ---- ConflictResolution ----
    #[test]
    fn roundtrip_conflict_resolution(cr in arb_conflict_resolution()) {
        let back = assert_json_value_roundtrip(&cr);
        prop_assert_eq!(cr, back);
    }

    // ---- ConflictDeclaration ----
    #[test]
    fn roundtrip_conflict_declaration(cd in arb_conflict_declaration()) {
        assert_json_value_roundtrip(&cd);
    }

    // ---- AliasSequence ----
    #[test]
    fn roundtrip_alias_sequence(seq in arb_alias_sequence()) {
        assert_json_value_roundtrip(&seq);
    }

    // ---- Grammar (empty) ----
    #[test]
    fn roundtrip_grammar_empty(name in "[a-zA-Z][a-zA-Z0-9_]{0,15}") {
        let grammar = Grammar::new(name);
        assert_json_value_roundtrip(&grammar);
    }

    // ---- Grammar with tokens ----
    #[test]
    fn roundtrip_grammar_with_tokens(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        toks in prop::collection::vec(
            (arb_symbol_id(), arb_token()), 1..5
        ),
    ) {
        let mut grammar = Grammar::new(name);
        for (id, tok) in toks {
            grammar.tokens.insert(id, tok);
        }
        assert_json_value_roundtrip(&grammar);
    }

    // ---- Grammar with rules ----
    #[test]
    fn roundtrip_grammar_with_rules(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        rules in prop::collection::vec(arb_rule(), 1..5),
    ) {
        let mut grammar = Grammar::new(name);
        for rule in rules {
            grammar.add_rule(rule);
        }
        assert_json_value_roundtrip(&grammar);
    }

    // ---- Grammar with precedences ----
    #[test]
    fn roundtrip_grammar_with_precedences(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        precs in prop::collection::vec(arb_precedence(), 1..4),
    ) {
        let mut grammar = Grammar::new(name);
        grammar.precedences = precs;
        assert_json_value_roundtrip(&grammar);
    }

    // ---- Grammar with conflicts ----
    #[test]
    fn roundtrip_grammar_with_conflicts(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        conflicts in prop::collection::vec(arb_conflict_declaration(), 1..4),
    ) {
        let mut grammar = Grammar::new(name);
        grammar.conflicts = conflicts;
        assert_json_value_roundtrip(&grammar);
    }

    // ---- Grammar with externals ----
    #[test]
    fn roundtrip_grammar_with_externals(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        exts in prop::collection::vec(arb_external_token(), 1..4),
    ) {
        let mut grammar = Grammar::new(name);
        grammar.externals = exts;
        assert_json_value_roundtrip(&grammar);
    }

    // ---- Grammar with alias sequences ----
    #[test]
    fn roundtrip_grammar_with_aliases(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        seqs in prop::collection::vec(
            (arb_production_id(), arb_alias_sequence()), 1..4
        ),
    ) {
        let mut grammar = Grammar::new(name);
        for (pid, seq) in seqs {
            grammar.alias_sequences.insert(pid, seq);
        }
        assert_json_value_roundtrip(&grammar);
    }

    // ---- Grammar with fields (sorted for validity) ----
    #[test]
    fn roundtrip_grammar_with_fields(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        field_names in prop::collection::vec("[a-z]{1,8}", 1..6),
    ) {
        let mut grammar = Grammar::new(name);
        let mut sorted: Vec<String> = field_names;
        sorted.sort();
        sorted.dedup();
        for (i, fname) in sorted.into_iter().enumerate() {
            grammar.fields.insert(FieldId(i as u16), fname);
        }
        assert_json_value_roundtrip(&grammar);
    }

    // ---- Grammar with extras / supertypes / inline_rules ----
    #[test]
    fn roundtrip_grammar_with_symbol_lists(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        extras in prop::collection::vec(arb_symbol_id(), 0..4),
        supertypes in prop::collection::vec(arb_symbol_id(), 0..4),
        inline_rules in prop::collection::vec(arb_symbol_id(), 0..4),
    ) {
        let mut grammar = Grammar::new(name);
        grammar.extras = extras;
        grammar.supertypes = supertypes;
        grammar.inline_rules = inline_rules;
        assert_json_value_roundtrip(&grammar);
    }

    // ---- Grammar full composite ----
    #[test]
    fn roundtrip_grammar_composite(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        rules in prop::collection::vec(arb_rule(), 1..3),
        toks in prop::collection::vec((arb_symbol_id(), arb_token()), 1..3),
        precs in prop::collection::vec(arb_precedence(), 0..2),
        exts in prop::collection::vec(arb_external_token(), 0..2),
        extras in prop::collection::vec(arb_symbol_id(), 0..3),
    ) {
        let mut grammar = Grammar::new(name);
        for rule in rules {
            grammar.add_rule(rule);
        }
        for (id, tok) in toks {
            grammar.tokens.insert(id, tok);
        }
        grammar.precedences = precs;
        grammar.externals = exts;
        grammar.extras = extras;
        assert_json_value_roundtrip(&grammar);
    }

    // ---- Pretty vs compact JSON roundtrip ----
    #[test]
    fn roundtrip_pretty_vs_compact(sym in arb_symbol()) {
        let compact = serde_json::to_string(&sym).unwrap();
        let pretty = serde_json::to_string_pretty(&sym).unwrap();
        let from_compact: Symbol = serde_json::from_str(&compact).unwrap();
        let from_pretty: Symbol = serde_json::from_str(&pretty).unwrap();
        prop_assert_eq!(&from_compact, &from_pretty);
        prop_assert_eq!(&sym, &from_compact);
    }
}
