use rust_sitter::glr_lexer::GLRLexer;
use rust_sitter::glr_parser::GLRParser;
use rust_sitter::subtree::Subtree;
// Test incremental parsing functionality
use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
use rust_sitter_ir::{
    Associativity, Grammar, PrecedenceEntry, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId,
    Token, TokenPattern,
};
use std::sync::Arc;

// NOTE: These tests use internal modules not exported by the public API
// In a real application, you would use the public API through rust_sitter
