use super::{GrammarJsConverter, JsRule};
use adze_ir::{Grammar, Symbol};

impl GrammarJsConverter {
    pub(super) fn rule_to_symbol(
        &mut self,
        grammar: &mut Grammar,
        rule: &JsRule,
    ) -> Option<Symbol> {
        match rule {
            JsRule::Symbol { name } => {
                eprintln!("Debug: rule_to_symbol for Symbol '{}'", name);
                if let Some(&id) = self.symbol_names.get(name) {
                    eprintln!("Debug:   Found symbol ID {}", id.0);
                    // Check if this symbol is actually a token-backed wrapper.
                    if let Some(token_id) = self.token_for_wrapped_rule(grammar, id, name) {
                        eprintln!(
                            "Debug:   Symbol {} is token-backed, returning Terminal({})",
                            id.0, token_id.0
                        );
                        Some(Symbol::Terminal(token_id))
                    } else {
                        eprintln!(
                            "Debug:   Symbol {} is not a pattern, returning NonTerminal",
                            id.0
                        );
                        Some(Symbol::NonTerminal(id))
                    }
                } else {
                    eprintln!("Debug:   Symbol '{}' not found in symbol_names", name);
                    None
                }
            }
            JsRule::String { value } => {
                // Create inline token
                Some(Symbol::Terminal(
                    self.get_or_create_string_token(grammar, value),
                ))
            }
            JsRule::Pattern { value } => {
                // Create pattern token
                Some(Symbol::Terminal(
                    self.get_or_create_pattern_token(grammar, value),
                ))
            }
            JsRule::Field { content, .. } => {
                // For fields, return the symbol of the content
                self.rule_to_symbol(grammar, content)
            }
            JsRule::Prec { content, .. }
            | JsRule::PrecLeft { content, .. }
            | JsRule::PrecRight { content, .. } => {
                // For precedence rules, return the symbol of the content
                eprintln!("Debug: rule_to_symbol for precedence rule, unwrapping content");
                self.rule_to_symbol(grammar, content)
            }
            _ => None, // Other types not yet handled
        }
    }
}
