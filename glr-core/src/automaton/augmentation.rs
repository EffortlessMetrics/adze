use crate::{GLRError, GrammarError};
use adze_ir::*;

pub(super) struct AugmentedGrammar {
    pub(super) grammar: Grammar,
    pub(super) original_start: SymbolId,
    pub(super) augmented_start: SymbolId,
}

pub(super) fn augment_grammar(
    grammar: &Grammar,
    max_symbol: u16,
) -> Result<AugmentedGrammar, GLRError> {
    let mut augmented_grammar = grammar.clone();

    let original_start =
        grammar
            .start_symbol()
            .ok_or(GLRError::GrammarError(GrammarError::UnresolvedSymbol(
                SymbolId(0),
            )))?;

    let augmented_start_id = max_symbol.checked_add(2).ok_or_else(|| {
        GLRError::StateMachine(
            "Augmented start symbol would overflow u16: grammar has too many symbols".into(),
        )
    })?;
    let augmented_start = SymbolId(augmented_start_id);

    let max_production_id = grammar
        .all_rules()
        .map(|r| r.production_id.0)
        .max()
        .unwrap_or(0);
    let augmented_production_id = max_production_id
        .checked_add(1)
        .ok_or_else(|| GLRError::StateMachine("Production ID overflow".into()))?;

    let augmented_rule = Rule {
        lhs: augmented_start,
        rhs: vec![Symbol::NonTerminal(original_start)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(augmented_production_id),
    };
    augmented_grammar
        .rules
        .insert(augmented_start, vec![augmented_rule]);
    augmented_grammar
        .rule_names
        .insert(augmented_start, "$start".to_string());

    Ok(AugmentedGrammar {
        grammar: augmented_grammar,
        original_start,
        augmented_start,
    })
}
