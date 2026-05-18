use super::{GrammarJsConverter, JsRule};
use adze_ir::{Grammar, Symbol, SymbolId};

impl GrammarJsConverter {
    pub(super) fn find_extra_symbol(&self, rule: &JsRule, grammar: &Grammar) -> Option<SymbolId> {
        eprintln!("DEBUG find_extra_symbol: rule = {:?}", rule);
        match rule {
            JsRule::Symbol { name } => {
                eprintln!("  Looking for symbol '{}'", name);

                // First check if it's directly a token
                if let Some(&symbol_id) = self.symbol_names.get(name) {
                    eprintln!("    Found symbol '{}' with id {:?}", name, symbol_id);

                    // Check if this is actually a token in the grammar
                    if grammar.tokens.contains_key(&symbol_id) {
                        eprintln!("    Symbol is a token, returning {:?}", symbol_id);
                        return Some(symbol_id);
                    }

                    // If it's a rule, we need to check if it's a simple wrapper around a token
                    // For extras like Whitespace that wrap a token pattern
                    if let Some(rules) = grammar.rules.get(&symbol_id) {
                        eprintln!("    Symbol is a rule with {} alternatives", rules.len());
                        // If there's exactly one rule and it's a simple sequence with one token
                        if rules.len() == 1
                            && rules[0].rhs.len() == 1
                            && let Symbol::Terminal(token_id) = &rules[0].rhs[0]
                        {
                            eprintln!("    Rule wraps token {:?}, using that for extra", token_id);
                            return Some(*token_id);
                        }
                    }
                }

                // Fallback: return the symbol itself
                let result = self.symbol_names.get(name).copied();
                eprintln!("  Symbol '{}' -> {:?}", name, result);
                result
            }
            JsRule::Pattern { value } => {
                // Look for a token with matching pattern
                eprintln!("  Looking for pattern '{}' in tokens", value);
                // Special handling for whitespace patterns
                if value.contains(r"\s") {
                    // Look for the whitespace token we added
                    if let Some(&id) = self.symbol_names.get("_WHITESPACE") {
                        eprintln!("    Found whitespace token with id {:?}", id);
                        return Some(id);
                    }
                }
                eprintln!("  Pattern '{}' not found in tokens", value);
                None
            }
            _ => {
                eprintln!("  Unhandled rule type");
                None
            }
        }
    }
}
