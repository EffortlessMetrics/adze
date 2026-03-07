#![allow(clippy::needless_range_loop)]

//! Property-based serde roundtrip tests for all IR types.
//!
//! Each test serializes a value to both JSON and postcard, deserializes back,
//! and asserts the result equals the original.

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

// ---------------------------------------------------------------------------
// Roundtrip helpers
// ---------------------------------------------------------------------------

/// Roundtrip via JSON and compare `serde_json::Value` representations.
fn assert_json_roundtrip<T: serde::Serialize + serde::de::DeserializeOwned>(val: &T) -> T {
    let json = serde_json::to_string(val).expect("json serialize");
    let back: T = serde_json::from_str(&json).expect("json deserialize");
    let v1 = serde_json::to_value(val).expect("to_value original");
    let v2 = serde_json::to_value(&back).expect("to_value roundtrip");
    assert_eq!(v1, v2, "JSON roundtrip mismatch");
    back
}

/// Roundtrip via postcard and assert equality.
fn assert_binary_roundtrip<
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
>(
    val: &T,
) -> T {
    let bytes = postcard::to_stdvec(val).expect("postcard serialize");
    let back: T = postcard::from_bytes(&bytes).expect("postcard deserialize");
    assert_eq!(val, &back, "postcard roundtrip mismatch");
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
        let j = assert_json_roundtrip(&orig);
        let b = assert_binary_roundtrip(&orig);
        prop_assert_eq!(orig, j);
        prop_assert_eq!(orig, b);
    }

    // ---- RuleId ----
    #[test]
    fn roundtrip_rule_id(id in 0u16..=u16::MAX) {
        let orig = RuleId(id);
        let j = assert_json_roundtrip(&orig);
        let b = assert_binary_roundtrip(&orig);
        prop_assert_eq!(orig, j);
        prop_assert_eq!(orig, b);
    }

    // ---- StateId ----
    #[test]
    fn roundtrip_state_id(id in 0u16..=u16::MAX) {
        let orig = StateId(id);
        let j = assert_json_roundtrip(&orig);
        let b = assert_binary_roundtrip(&orig);
        prop_assert_eq!(orig, j);
        prop_assert_eq!(orig, b);
    }

    // ---- FieldId ----
    #[test]
    fn roundtrip_field_id(id in 0u16..=u16::MAX) {
        let orig = FieldId(id);
        let j = assert_json_roundtrip(&orig);
        let b = assert_binary_roundtrip(&orig);
        prop_assert_eq!(orig, j);
        prop_assert_eq!(orig, b);
    }

    // ---- ProductionId ----
    #[test]
    fn roundtrip_production_id(id in 0u16..=u16::MAX) {
        let orig = ProductionId(id);
        let j = assert_json_roundtrip(&orig);
        let b = assert_binary_roundtrip(&orig);
        prop_assert_eq!(orig, j);
        prop_assert_eq!(orig, b);
    }

    // ---- Associativity ----
    #[test]
    fn roundtrip_associativity(assoc in arb_associativity()) {
        let j = assert_json_roundtrip(&assoc);
        let b = assert_binary_roundtrip(&assoc);
        prop_assert_eq!(assoc, j);
        prop_assert_eq!(assoc, b);
    }

    // ---- PrecedenceKind ----
    #[test]
    fn roundtrip_precedence_kind(pk in arb_precedence_kind()) {
        let j = assert_json_roundtrip(&pk);
        let b = assert_binary_roundtrip(&pk);
        prop_assert_eq!(pk, j);
        prop_assert_eq!(pk, b);
    }

    // ---- TokenPattern ----
    #[test]
    fn roundtrip_token_pattern(pat in arb_token_pattern()) {
        let j = assert_json_roundtrip(&pat);
        let b = assert_binary_roundtrip(&pat);
        prop_assert_eq!(&pat, &j);
        prop_assert_eq!(&pat, &b);
    }

    // ---- Token ----
    #[test]
    fn roundtrip_token(tok in arb_token()) {
        let j = assert_json_roundtrip(&tok);
        let b = assert_binary_roundtrip(&tok);
        prop_assert_eq!(&tok, &j);
        prop_assert_eq!(&tok, &b);
    }

    // ---- Symbol leaf variants ----
    #[test]
    fn roundtrip_symbol_leaf(sym in arb_symbol_leaf()) {
        let j = assert_json_roundtrip(&sym);
        let b = assert_binary_roundtrip(&sym);
        prop_assert_eq!(&sym, &j);
        prop_assert_eq!(&sym, &b);
    }

    // ---- Symbol (recursive) ----
    #[test]
    fn roundtrip_symbol(sym in arb_symbol()) {
        let j = assert_json_roundtrip(&sym);
        let b = assert_binary_roundtrip(&sym);
        prop_assert_eq!(&sym, &j);
        prop_assert_eq!(&sym, &b);
    }

    // ---- Vec<Symbol> ----
    #[test]
    fn roundtrip_symbol_vec(syms in prop::collection::vec(arb_symbol(), 0..8)) {
        let j: Vec<Symbol> = assert_json_roundtrip(&syms);
        let b: Vec<Symbol> = assert_binary_roundtrip(&syms);
        prop_assert_eq!(&syms, &j);
        prop_assert_eq!(&syms, &b);
    }

    // ---- SymbolMetadata ----
    #[test]
    fn roundtrip_symbol_metadata(meta in arb_symbol_metadata()) {
        let j = assert_json_roundtrip(&meta);
        let b = assert_binary_roundtrip(&meta);
        prop_assert_eq!(meta, j);
        prop_assert_eq!(meta, b);
    }

    // ---- Rule ----
    #[test]
    fn roundtrip_rule(rule in arb_rule()) {
        let j = assert_json_roundtrip(&rule);
        let b = assert_binary_roundtrip(&rule);
        prop_assert_eq!(&rule, &j);
        prop_assert_eq!(&rule, &b);
    }

    // ---- ExternalToken ----
    #[test]
    fn roundtrip_external_token(et in arb_external_token()) {
        let j = assert_json_roundtrip(&et);
        let b = assert_binary_roundtrip(&et);
        prop_assert_eq!(&et, &j);
        prop_assert_eq!(&et, &b);
    }

    // ---- Precedence ----
    #[test]
    fn roundtrip_precedence(prec in arb_precedence()) {
        let j = assert_json_roundtrip(&prec);
        let b = assert_binary_roundtrip(&prec);
        prop_assert_eq!(&prec, &j);
        prop_assert_eq!(&prec, &b);
    }

    // ---- ConflictResolution ----
    #[test]
    fn roundtrip_conflict_resolution(cr in arb_conflict_resolution()) {
        let j = assert_json_roundtrip(&cr);
        let b = assert_binary_roundtrip(&cr);
        prop_assert_eq!(&cr, &j);
        prop_assert_eq!(&cr, &b);
    }

    // ---- ConflictDeclaration ----
    #[test]
    fn roundtrip_conflict_declaration(cd in arb_conflict_declaration()) {
        let j = assert_json_roundtrip(&cd);
        let b = assert_binary_roundtrip(&cd);
        prop_assert_eq!(&cd, &j);
        prop_assert_eq!(&cd, &b);
    }

    // ---- AliasSequence ----
    #[test]
    fn roundtrip_alias_sequence(seq in arb_alias_sequence()) {
        let j = assert_json_roundtrip(&seq);
        let b = assert_binary_roundtrip(&seq);
        prop_assert_eq!(&seq, &j);
        prop_assert_eq!(&seq, &b);
    }

    // ---- Grammar (empty) ----
    #[test]
    fn roundtrip_grammar_empty(name in "[a-zA-Z][a-zA-Z0-9_]{0,15}") {
        let grammar = Grammar::new(name);
        let j = assert_json_roundtrip(&grammar);
        let b = assert_binary_roundtrip(&grammar);
        prop_assert_eq!(&grammar, &j);
        prop_assert_eq!(&grammar, &b);
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
        let j = assert_json_roundtrip(&grammar);
        let b = assert_binary_roundtrip(&grammar);
        prop_assert_eq!(&grammar, &j);
        prop_assert_eq!(&grammar, &b);
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
        let j = assert_json_roundtrip(&grammar);
        let b = assert_binary_roundtrip(&grammar);
        prop_assert_eq!(&grammar, &j);
        prop_assert_eq!(&grammar, &b);
    }

    // ---- Grammar with precedences ----
    #[test]
    fn roundtrip_grammar_with_precedences(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        precs in prop::collection::vec(arb_precedence(), 1..4),
    ) {
        let mut grammar = Grammar::new(name);
        grammar.precedences = precs;
        let j = assert_json_roundtrip(&grammar);
        let b = assert_binary_roundtrip(&grammar);
        prop_assert_eq!(&grammar, &j);
        prop_assert_eq!(&grammar, &b);
    }

    // ---- Grammar with conflicts ----
    #[test]
    fn roundtrip_grammar_with_conflicts(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        conflicts in prop::collection::vec(arb_conflict_declaration(), 1..4),
    ) {
        let mut grammar = Grammar::new(name);
        grammar.conflicts = conflicts;
        let j = assert_json_roundtrip(&grammar);
        let b = assert_binary_roundtrip(&grammar);
        prop_assert_eq!(&grammar, &j);
        prop_assert_eq!(&grammar, &b);
    }

    // ---- Grammar with externals ----
    #[test]
    fn roundtrip_grammar_with_externals(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        exts in prop::collection::vec(arb_external_token(), 1..4),
    ) {
        let mut grammar = Grammar::new(name);
        grammar.externals = exts;
        let j = assert_json_roundtrip(&grammar);
        let b = assert_binary_roundtrip(&grammar);
        prop_assert_eq!(&grammar, &j);
        prop_assert_eq!(&grammar, &b);
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
        let j = assert_json_roundtrip(&grammar);
        let b = assert_binary_roundtrip(&grammar);
        prop_assert_eq!(&grammar, &j);
        prop_assert_eq!(&grammar, &b);
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
        let j = assert_json_roundtrip(&grammar);
        let b = assert_binary_roundtrip(&grammar);
        prop_assert_eq!(&grammar, &j);
        prop_assert_eq!(&grammar, &b);
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
        let j = assert_json_roundtrip(&grammar);
        let b = assert_binary_roundtrip(&grammar);
        prop_assert_eq!(&grammar, &j);
        prop_assert_eq!(&grammar, &b);
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
        let j = assert_json_roundtrip(&grammar);
        let b = assert_binary_roundtrip(&grammar);
        prop_assert_eq!(&grammar, &j);
        prop_assert_eq!(&grammar, &b);
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

    // ---- Bincode determinism: same input produces same bytes ----
    #[test]
    fn binary_deterministic_symbol(sym in arb_symbol()) {
        let bytes1 = postcard::to_stdvec(&sym).unwrap();
        let bytes2 = postcard::to_stdvec(&sym).unwrap();
        prop_assert_eq!(bytes1, bytes2);
    }

    // ---- Bincode determinism for Grammar ----
    #[test]
    fn binary_deterministic_grammar(
        name in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        rules in prop::collection::vec(arb_rule(), 0..3),
    ) {
        let mut grammar = Grammar::new(name);
        for rule in rules {
            grammar.add_rule(rule);
        }
        let bytes1 = postcard::to_stdvec(&grammar).unwrap();
        let bytes2 = postcard::to_stdvec(&grammar).unwrap();
        prop_assert_eq!(bytes1, bytes2);
    }

    // ---- Cross-format: JSON and postcard produce equal values ----
    #[test]
    fn cross_format_rule(rule in arb_rule()) {
        let from_json: Rule = serde_json::from_str(
            &serde_json::to_string(&rule).unwrap()
        ).unwrap();
        let from_postcard: Rule = postcard::from_bytes(
            &postcard::to_stdvec(&rule).unwrap()
        ).unwrap();
        prop_assert_eq!(&from_json, &from_postcard);
    }
}
