//! Shared grammar fixtures and conflict analysis helpers for GLR BDD tests.
//!
//! This crate intentionally owns grammar-level BDD fixtures (fixture grammars,
//! parse-table builders, token metadata, and table introspection helpers) so
//! downstream crates can compose behavior without monolithic fixtures.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use adze_glr_core::build_lr1_automaton;
use adze_glr_core::{
    Action, Conflict, ConflictResolver, ConflictType, FirstFollowSets, ParseTable, StateId,
};
use adze_ir::{
    Associativity, Grammar, PrecedenceKind, ProductionId, Rule, RuleId, Symbol, SymbolId, Token,
    TokenPattern,
};

/// Summary of conflict information from a parse table.
#[derive(Debug, Clone)]
pub struct ConflictAnalysis {
    /// Number of table cells with more than one parser action.
    pub total_conflicts: usize,
    /// Number of shift/reduce conflicts.
    pub shift_reduce_conflicts: usize,
    /// Number of reduce/reduce conflicts.
    pub reduce_reduce_conflicts: usize,
    /// Per-cell conflict detail for targeted assertions and debug logs.
    pub conflict_details: Vec<(usize, usize, Vec<Action>)>,
}

/// Token pattern selector for reusable runtime token fixtures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenPatternKind {
    /// Regular-expression matcher.
    Regex(&'static str),
    /// Exact literal matcher.
    Literal(&'static str),
}

/// Lightweight token pattern description used by fixture helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenPatternSpec {
    /// Symbol id from the fixture grammar.
    pub symbol_id: SymbolId,
    /// Matching strategy for the token.
    pub matcher: TokenPatternKind,
    /// Whether the token acts as a keyword.
    pub is_keyword: bool,
}

/// Lightweight symbol metadata description used by fixture helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolMetadataSpec {
    /// Terminal vs non-terminal.
    pub is_terminal: bool,
    /// Visibility flag in parse tree.
    pub is_visible: bool,
    /// Supertype flag in tree metadata.
    pub is_supertype: bool,
}

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

/// Symbol metadata fixture for the dangling-else runtime test grammar.
pub const DANGLING_ELSE_SYMBOL_METADATA: &[SymbolMetadataSpec] = &[
    SymbolMetadataSpec {
        is_terminal: true,
        is_visible: false,
        is_supertype: false,
    },
    SymbolMetadataSpec {
        is_terminal: true,
        is_visible: true,
        is_supertype: false,
    },
    SymbolMetadataSpec {
        is_terminal: true,
        is_visible: true,
        is_supertype: false,
    },
    SymbolMetadataSpec {
        is_terminal: true,
        is_visible: true,
        is_supertype: false,
    },
    SymbolMetadataSpec {
        is_terminal: true,
        is_visible: true,
        is_supertype: false,
    },
    SymbolMetadataSpec {
        is_terminal: false,
        is_visible: true,
        is_supertype: false,
    },
];

/// Token pattern fixture for the dangling-else runtime test grammar.
pub const DANGLING_ELSE_TOKEN_PATTERNS: &[TokenPatternSpec] = &[
    TokenPatternSpec {
        symbol_id: SymbolId(255),
        matcher: TokenPatternKind::Regex(r"\s+"),
        is_keyword: false,
    },
    TokenPatternSpec {
        symbol_id: DANGLING_ELSE_IF,
        matcher: TokenPatternKind::Literal("if"),
        is_keyword: true,
    },
    TokenPatternSpec {
        symbol_id: DANGLING_ELSE_THEN,
        matcher: TokenPatternKind::Literal("then"),
        is_keyword: true,
    },
    TokenPatternSpec {
        symbol_id: DANGLING_ELSE_ELSE,
        matcher: TokenPatternKind::Literal("else"),
        is_keyword: true,
    },
    TokenPatternSpec {
        symbol_id: DANGLING_ELSE_EXPR,
        matcher: TokenPatternKind::Literal("expr"),
        is_keyword: false,
    },
    TokenPatternSpec {
        symbol_id: DANGLING_ELSE_STMT,
        matcher: TokenPatternKind::Literal("stmt"),
        is_keyword: false,
    },
];

/// Symbol metadata fixture for precedence arithmetic runtime tests.
pub const PRECEDENCE_ARITHMETIC_SYMBOL_METADATA: &[SymbolMetadataSpec] = &[
    SymbolMetadataSpec {
        is_terminal: true,
        is_visible: false,
        is_supertype: false,
    },
    SymbolMetadataSpec {
        is_terminal: true,
        is_visible: true,
        is_supertype: false,
    },
    SymbolMetadataSpec {
        is_terminal: true,
        is_visible: true,
        is_supertype: false,
    },
    SymbolMetadataSpec {
        is_terminal: true,
        is_visible: true,
        is_supertype: false,
    },
    SymbolMetadataSpec {
        is_terminal: false,
        is_visible: true,
        is_supertype: false,
    },
];

/// Token pattern fixture for precedence arithmetic runtime tests.
pub const PRECEDENCE_ARITHMETIC_TOKEN_PATTERNS: &[TokenPatternSpec] = &[
    TokenPatternSpec {
        symbol_id: SymbolId(255),
        matcher: TokenPatternKind::Regex(r"\s+"),
        is_keyword: false,
    },
    TokenPatternSpec {
        symbol_id: PRECEDENCE_NUM,
        matcher: TokenPatternKind::Regex(r"\d+"),
        is_keyword: false,
    },
    TokenPatternSpec {
        symbol_id: PRECEDENCE_PLUS,
        matcher: TokenPatternKind::Literal("+"),
        is_keyword: false,
    },
    TokenPatternSpec {
        symbol_id: PRECEDENCE_STAR,
        matcher: TokenPatternKind::Literal("*"),
        is_keyword: false,
    },
];

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

/// Analyze conflict cells in a parse table.
pub fn analyze_conflicts(parse_table: &ParseTable) -> ConflictAnalysis {
    let mut analysis = ConflictAnalysis {
        total_conflicts: 0,
        shift_reduce_conflicts: 0,
        reduce_reduce_conflicts: 0,
        conflict_details: vec![],
    };

    for state in 0..parse_table.state_count {
        for sym in 0..parse_table.symbol_count {
            let actions = &parse_table.action_table[state][sym];
            if actions.len() > 1 {
                analysis.total_conflicts += 1;

                let has_shift = actions.iter().any(|a| matches!(a, Action::Shift(_)));
                let has_reduce = actions.iter().any(|a| matches!(a, Action::Reduce(_)));

                if has_shift && has_reduce {
                    analysis.shift_reduce_conflicts += 1;
                } else if !has_shift && has_reduce {
                    analysis.reduce_reduce_conflicts += 1;
                }

                analysis
                    .conflict_details
                    .push((state, sym, actions.clone()));
            }
        }
    }

    analysis
}

/// Count parse table cells with more than one action.
pub fn count_multi_action_cells(parse_table: &ParseTable) -> usize {
    parse_table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .filter(|cell| cell.len() > 1)
        .count()
}

/// Resolve a synthetic shift/reduce conflict against a given grammar and symbol.
pub fn resolve_shift_reduce_actions(
    grammar: &Grammar,
    symbol: SymbolId,
    reduce_rule: RuleId,
) -> Vec<Action> {
    let mut resolver = ConflictResolver {
        conflicts: vec![Conflict {
            state: StateId(42),
            symbol,
            actions: vec![Action::Shift(StateId(7)), Action::Reduce(reduce_rule)],
            conflict_type: ConflictType::ShiftReduce,
        }],
    };

    resolver.resolve_conflicts(grammar);
    resolver
        .conflicts
        .first()
        .expect("expected one conflict")
        .actions
        .clone()
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
