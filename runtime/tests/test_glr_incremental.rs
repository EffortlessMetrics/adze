use rust_sitter::glr_lexer::GLRLexer;
use rust_sitter::glr_parser::GLRParser;
use rust_sitter::subtree::Subtree;
// Test incremental parsing functionality
use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets};
use rust_sitter_ir::{
    Grammar, Rule, Symbol, Token, TokenPattern, SymbolId, ProductionId,
    PrecedenceEntry, PrecedenceKind, Associativity,
};
use std::sync::Arc;

// NOTE: These tests use internal modules not exported by the public API
// In a real application, you would use the public API through rust_sitter
