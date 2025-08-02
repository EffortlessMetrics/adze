use rust_sitter::glr_lexer::GLRLexer;
use rust_sitter::glr_parser::GLRParser;
use rust_sitter::subtree::Subtree;
// Test subtree reuse in incremental parsing
use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

// Import internal modules for testing
