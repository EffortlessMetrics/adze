//! Convert an engine forest into the public Tree facade.

use crate::engine::Forest;
use crate::tree::{Tree, TreeNode};
use crate::error::ParseError;

#[cfg(feature = "glr-core")]
use rust_sitter_glr_core::ForestView as CoreForestView;

pub fn forest_to_tree(forest: Forest) -> Result<Tree, ParseError> {
    match forest {
        #[cfg(feature = "glr-core")]
        Forest::Glr(core) => build_from_glr(core),
        _ => Err(ParseError::with_msg("unsupported forest type")),
    }
}

#[cfg(feature = "glr-core")]
fn build_from_glr(core: rust_sitter_glr_core::Forest) -> Result<Tree, ParseError> {
    let view = core.view();
    let roots = view.roots();

    if roots.is_empty() {
        return Err(ParseError::with_msg("forest has no roots"));
    }

    // Take the first root for now (could handle ambiguity later)
    let root_id = roots[0];
    let root_node = build_node(view, root_id);
    Ok(Tree::new(root_node))
}

#[cfg(feature = "glr-core")]
fn build_node(view: &dyn CoreForestView, id: u32) -> TreeNode {
    let span = view.span(id);
    let kind = view.kind(id);
    let kids = view
        .best_children(id)
        .iter()
        .copied()
        .map(|c| build_node(view, c))
        .collect();
    TreeNode::new_with_children(kind, span.start as usize, span.end as usize, kids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_sitter_glr_core::{Action, Driver, GotoIndexing, LexMode, ParseRule, ParseTable};
    use rust_sitter_ir::{Grammar, RuleId, StateId, SymbolId};
    use std::collections::BTreeMap;

    fn simple_table() -> ParseTable {
        let t_sym = SymbolId(1);
        let eof = SymbolId(0);
        let s_sym = SymbolId(3);

        let rules = vec![ParseRule { lhs: s_sym, rhs_len: 1 }];

        let mut actions = vec![vec![vec![]; 4]; 3];
        actions[0][1].push(Action::Shift(StateId(1)));
        actions[1][0].push(Action::Reduce(RuleId(0)));
        actions[2][0].push(Action::Accept);

        let invalid = StateId(65535);
        let mut gotos = vec![vec![invalid; 4]; 3];
        gotos[0][3] = StateId(2);

        let mut symbol_to_index = BTreeMap::new();
        for i in 0..4 {
            symbol_to_index.insert(SymbolId(i as u16), i);
        }

        let mut nonterminal_to_index = BTreeMap::new();
        nonterminal_to_index.insert(s_sym, 3);

        ParseTable {
            action_table: actions,
            goto_table: gotos,
            symbol_metadata: vec![],
            state_count: 3,
            symbol_count: 4,
            symbol_to_index,
            index_to_symbol: (0..4).map(|i| SymbolId(i as u16)).collect(),
            external_scanner_states: vec![],
            rules,
            nonterminal_to_index,
            goto_indexing: GotoIndexing::NonterminalMap,
            eof_symbol: eof,
            start_symbol: s_sym,
            grammar: Grammar::new("test".to_string()),
            initial_state: StateId(0),
            token_count: 1,
            external_token_count: 0,
            lex_modes: vec![
                LexMode {
                    lex_state: 0,
                    external_lex_state: 0,
                };
                3
            ],
            extras: vec![],
            dynamic_prec_by_rule: vec![0; 1],
            rule_assoc_by_rule: vec![0; 1],
            alias_sequences: vec![],
            field_names: vec![],
            field_map: BTreeMap::new(),
        }
    }

    #[test]
    fn valid_forest_produces_tree() {
        let table = simple_table();
        let mut driver = Driver::new(&table);
        let tokens = vec![(1u32, 0u32, 1u32)];
        let forest = driver.parse_tokens(tokens.into_iter()).unwrap();
        let tree = forest_to_tree(Forest::Glr(forest)).unwrap();
        assert_eq!(tree.root_kind(), 3u32);
    }

    #[test]
    fn stub_forest_errors() {
        let result = forest_to_tree(Forest::Stub);
        assert!(result.is_err());
    }
}
