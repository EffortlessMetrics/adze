//! Runtime-oriented fixture metadata for BDD grammar scenarios.
//!
//! This crate owns lightweight symbol metadata and token pattern fixtures that
//! are consumed by runtime integration tests and fixture facades.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use adze_ir::SymbolId;

const DANGLING_ELSE_IF: SymbolId = SymbolId(1);
const DANGLING_ELSE_THEN: SymbolId = SymbolId(2);
const DANGLING_ELSE_ELSE: SymbolId = SymbolId(3);
const DANGLING_ELSE_EXPR: SymbolId = SymbolId(4);
const DANGLING_ELSE_STMT: SymbolId = SymbolId(5);

const PRECEDENCE_NUM: SymbolId = SymbolId(1);
const PRECEDENCE_PLUS: SymbolId = SymbolId(2);
const PRECEDENCE_STAR: SymbolId = SymbolId(3);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_pattern_kind_clone_eq() {
        let a = TokenPatternKind::Literal("x");
        let b = a;
        assert_eq!(a, b);
        let c = TokenPatternKind::Regex(r"\d+");
        assert_ne!(format!("{c:?}"), format!("{a:?}"));
    }

    #[test]
    fn symbol_metadata_spec_debug() {
        let spec = SymbolMetadataSpec {
            is_terminal: true,
            is_visible: false,
            is_supertype: false,
        };
        let dbg = format!("{spec:?}");
        assert!(dbg.contains("is_terminal"));
    }

    #[test]
    fn fixture_constants_are_non_empty() {
        assert!(!DANGLING_ELSE_SYMBOL_METADATA.is_empty());
        assert!(!DANGLING_ELSE_TOKEN_PATTERNS.is_empty());
        assert!(!PRECEDENCE_ARITHMETIC_SYMBOL_METADATA.is_empty());
        assert!(!PRECEDENCE_ARITHMETIC_TOKEN_PATTERNS.is_empty());
    }
}
