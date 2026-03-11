//! Shared grammar fixtures and conflict analysis helpers for GLR BDD tests.
//!
//! This crate intentionally owns grammar-level BDD fixtures (fixture grammars,
//! parse-table builders, and table introspection helpers) while re-exporting
//! runtime metadata fixtures from `adze-bdd-runtime-fixtures-core`.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_bdd_grammar_analysis_core::{
    ConflictAnalysis, analyze_conflicts, count_multi_action_cells, resolve_shift_reduce_actions,
};
pub use adze_bdd_runtime_fixtures_core::{
    DANGLING_ELSE_SYMBOL_METADATA, DANGLING_ELSE_TOKEN_PATTERNS,
    PRECEDENCE_ARITHMETIC_SYMBOL_METADATA, PRECEDENCE_ARITHMETIC_TOKEN_PATTERNS,
    SymbolMetadataSpec, TokenPatternKind, TokenPatternSpec,
};
use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::{
    Associativity, Grammar, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern,
};

const DANGLING_ELSE_IF: SymbolId = SymbolId(1);
const DANGLING_ELSE_THEN: SymbolId = SymbolId(2);
const DANGLING_ELSE_ELSE: SymbolId = SymbolId(3);
const DANGLING_ELSE_EXPR: SymbolId = SymbolId(4);
const DANGLING_ELSE_STMT: SymbolId = SymbolId(5);
const DANGLING_ELSE_NON_TERMINAL_S: SymbolId = SymbolId(10);

const PRECEDENCE_NUM: SymbolId = SymbolId(1);
const PRECEDENCE_PLUS: SymbolId = SymbolId(2);
const PRECEDENCE_STAR: SymbolId = SymbolId(3);
const PRECEDENCE_EXPR: SymbolId = SymbolId(10);

/// Create the dangling-else grammar used by GLR conflict fixtures.
pub fn dangling_else_grammar() -> Grammar {
    let mut grammar = Grammar::new("if_then_else".to_string());

    grammar.tokens.insert(
        DANGLING_ELSE_IF,
        Token {
            name: "if".to_string(),
            pattern: TokenPattern::String("if".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        DANGLING_ELSE_THEN,
        Token {
            name: "then".to_string(),
            pattern: TokenPattern::String("then".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        DANGLING_ELSE_ELSE,
        Token {
            name: "else".to_string(),
            pattern: TokenPattern::String("else".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        DANGLING_ELSE_EXPR,
        Token {
            name: "expr".to_string(),
            pattern: TokenPattern::String("expr".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        DANGLING_ELSE_STMT,
        Token {
            name: "stmt".to_string(),
            pattern: TokenPattern::String("stmt".to_string()),
            fragile: false,
        },
    );

    grammar
        .rule_names
        .insert(DANGLING_ELSE_NON_TERMINAL_S, "S".to_string());

    grammar.rules.insert(
        DANGLING_ELSE_NON_TERMINAL_S,
        vec![
            Rule {
                lhs: DANGLING_ELSE_NON_TERMINAL_S,
                rhs: vec![
                    Symbol::Terminal(DANGLING_ELSE_IF),
                    Symbol::Terminal(DANGLING_ELSE_EXPR),
                    Symbol::Terminal(DANGLING_ELSE_THEN),
                    Symbol::NonTerminal(DANGLING_ELSE_NON_TERMINAL_S),
                ],
                precedence: None,
                associativity: None,
                production_id: ProductionId(0),
                fields: vec![],
            },
            Rule {
                lhs: DANGLING_ELSE_NON_TERMINAL_S,
                rhs: vec![
                    Symbol::Terminal(DANGLING_ELSE_IF),
                    Symbol::Terminal(DANGLING_ELSE_EXPR),
                    Symbol::Terminal(DANGLING_ELSE_THEN),
                    Symbol::NonTerminal(DANGLING_ELSE_NON_TERMINAL_S),
                    Symbol::Terminal(DANGLING_ELSE_ELSE),
                    Symbol::NonTerminal(DANGLING_ELSE_NON_TERMINAL_S),
                ],
                precedence: None,
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
            Rule {
                lhs: DANGLING_ELSE_NON_TERMINAL_S,
                rhs: vec![Symbol::Terminal(DANGLING_ELSE_STMT)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(2),
                fields: vec![],
            },
        ],
    );

    let _ = grammar.get_or_build_registry();
    grammar
}

/// Create an arithmetic grammar with configurable precedence/associativity.
pub fn precedence_arithmetic_grammar(plus_assoc: Associativity) -> Grammar {
    let mut grammar = Grammar::new("precedence_expr".to_string());

    grammar.tokens.insert(
        PRECEDENCE_PLUS,
        Token {
            name: "+".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        PRECEDENCE_STAR,
        Token {
            name: "*".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        PRECEDENCE_NUM,
        Token {
            name: "num".to_string(),
            pattern: TokenPattern::String("num".to_string()),
            fragile: false,
        },
    );

    grammar
        .rule_names
        .insert(PRECEDENCE_EXPR, "Expr".to_string());
    grammar.rules.insert(
        PRECEDENCE_EXPR,
        vec![
            Rule {
                lhs: PRECEDENCE_EXPR,
                rhs: vec![
                    Symbol::NonTerminal(PRECEDENCE_EXPR),
                    Symbol::Terminal(PRECEDENCE_PLUS),
                    Symbol::NonTerminal(PRECEDENCE_EXPR),
                ],
                precedence: Some(PrecedenceKind::Static(1)),
                associativity: Some(plus_assoc),
                production_id: ProductionId(0),
                fields: vec![],
            },
            Rule {
                lhs: PRECEDENCE_EXPR,
                rhs: vec![
                    Symbol::NonTerminal(PRECEDENCE_EXPR),
                    Symbol::Terminal(PRECEDENCE_STAR),
                    Symbol::NonTerminal(PRECEDENCE_EXPR),
                ],
                precedence: Some(PrecedenceKind::Static(2)),
                associativity: Some(Associativity::Left),
                production_id: ProductionId(1),
                fields: vec![],
            },
            Rule {
                lhs: PRECEDENCE_EXPR,
                rhs: vec![Symbol::Terminal(PRECEDENCE_NUM)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(2),
                fields: vec![],
            },
        ],
    );

    let _ = grammar.get_or_build_registry();
    grammar
}

/// Create arithmetic grammar without precedence metadata.
pub fn no_precedence_grammar() -> Grammar {
    let mut grammar = Grammar::new("no_precedence_expr".to_string());

    let plus_id = SymbolId(1);
    let num_id = SymbolId(2);
    let expr_id = SymbolId(10);

    grammar.tokens.insert(
        plus_id,
        Token {
            name: "+".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        num_id,
        Token {
            name: "num".to_string(),
            pattern: TokenPattern::String("num".to_string()),
            fragile: false,
        },
    );

    grammar.rule_names.insert(expr_id, "Expr".to_string());
    grammar.rules.insert(
        expr_id,
        vec![
            Rule {
                lhs: expr_id,
                rhs: vec![
                    Symbol::NonTerminal(expr_id),
                    Symbol::Terminal(plus_id),
                    Symbol::NonTerminal(expr_id),
                ],
                precedence: None,
                associativity: None,
                production_id: ProductionId(0),
                fields: vec![],
            },
            Rule {
                lhs: expr_id,
                rhs: vec![Symbol::Terminal(num_id)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
        ],
    );

    let _ = grammar.get_or_build_registry();
    grammar
}

/// Build an LR(1) parse table from the fixture grammar.
pub fn build_lr1_parse_table(grammar: &Grammar) -> Result<ParseTable, String> {
    let first_follow = FirstFollowSets::compute(grammar).map_err(|err| {
        format!(
            "FAILED to compute FIRST/FOLLOW for fixture grammar {}: {}",
            grammar.name, err
        )
    })?;

    build_lr1_automaton(grammar, &first_follow).map_err(|err| {
        format!(
            "FAILED to build LR(1) automaton for fixture grammar {}: {}",
            grammar.name, err
        )
    })
}

/// Build runtime-ready parse table shape used by Runtime2 BDD tests.
pub fn build_runtime_parse_table(grammar: &Grammar) -> Result<ParseTable, String> {
    build_lr1_parse_table(grammar)
        .map(|table| table.normalize_eof_to_zero().with_detected_goto_indexing())
}

/// Build dangling-else parse table in LR(1) form.
pub fn build_dangling_else_parse_table() -> Result<ParseTable, String> {
    build_lr1_parse_table(&dangling_else_grammar())
}

/// Build dangling-else parse table in Runtime2-ready form.
pub fn build_runtime_dangling_else_parse_table() -> Result<ParseTable, String> {
    build_runtime_parse_table(&dangling_else_grammar())
}

/// Build precedence arithmetic parse table in LR(1) form.
pub fn build_precedence_arithmetic_parse_table(
    plus_assoc: Associativity,
) -> Result<ParseTable, String> {
    build_lr1_parse_table(&precedence_arithmetic_grammar(plus_assoc))
}

/// Build precedence arithmetic parse table in Runtime2-ready form.
pub fn build_runtime_precedence_arithmetic_parse_table(
    plus_assoc: Associativity,
) -> Result<ParseTable, String> {
    build_runtime_parse_table(&precedence_arithmetic_grammar(plus_assoc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dangling_else_grammar_has_expected_tokens() {
        let g = dangling_else_grammar();
        assert_eq!(g.name, "if_then_else");
        assert_eq!(g.tokens.len(), 5);
    }

    #[test]
    fn precedence_arithmetic_grammar_has_three_rules() {
        let g = precedence_arithmetic_grammar(Associativity::Left);
        assert_eq!(g.rules.get(&SymbolId(10)).unwrap().len(), 3);
    }

    #[test]
    fn no_precedence_grammar_builds_successfully() {
        let g = no_precedence_grammar();
        assert_eq!(g.name, "no_precedence_expr");
        assert!(!g.rules.is_empty());
    }

    #[test]
    fn fixture_constants_are_non_empty() {
        assert!(!DANGLING_ELSE_SYMBOL_METADATA.is_empty());
        assert!(!DANGLING_ELSE_TOKEN_PATTERNS.is_empty());
        assert!(!PRECEDENCE_ARITHMETIC_SYMBOL_METADATA.is_empty());
        assert!(!PRECEDENCE_ARITHMETIC_TOKEN_PATTERNS.is_empty());
    }
}
