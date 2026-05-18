use super::{GrammarJsConverter, JsRule};
use adze_ir::{Grammar, SymbolId, Token, TokenPattern};
use anyhow::Result;

impl GrammarJsConverter {
    pub(super) fn add_terminal_token(
        &mut self,
        grammar: &mut Grammar,
        name: &str,
        pattern: &str,
    ) -> Result<()> {
        let symbol_id = SymbolId(self.next_symbol_id.try_into().unwrap());
        self.symbol_names.insert(name.to_string(), symbol_id);

        grammar.tokens.insert(
            symbol_id,
            Token {
                name: name.to_string(),
                pattern: TokenPattern::Regex(pattern.to_string()),
                fragile: false,
            },
        );

        self.next_symbol_id += 1;
        Ok(())
    }
    pub(super) fn get_or_create_string_token(
        &mut self,
        grammar: &mut Grammar,
        value: &str,
    ) -> SymbolId {
        // Check if we already have this token
        for (id, token) in &grammar.tokens {
            if let TokenPattern::String(s) = &token.pattern
                && s == value
            {
                return *id;
            }
        }

        // Create new token
        let id = SymbolId(self.next_symbol_id.try_into().unwrap());
        self.next_symbol_id += 1;
        let token = Token {
            name: format!("\"{}\"", value),
            pattern: TokenPattern::String(value.to_string()),
            fragile: false,
        };
        grammar.tokens.insert(id, token);
        id
    }

    pub(super) fn get_or_create_pattern_token(
        &mut self,
        grammar: &mut Grammar,
        pattern: &str,
    ) -> SymbolId {
        // Check if we already have this token
        for (id, token) in &grammar.tokens {
            if let TokenPattern::Regex(p) = &token.pattern
                && p == pattern
            {
                return *id;
            }
        }

        // Create new token
        let id = SymbolId(self.next_symbol_id.try_into().unwrap());
        self.next_symbol_id += 1;
        let token = Token {
            name: format!("/{}/", pattern),
            pattern: TokenPattern::Regex(pattern.to_string()),
            fragile: false,
        };
        grammar.tokens.insert(id, token);
        id
    }
    pub(super) fn token_for_wrapped_rule(
        &mut self,
        grammar: &mut Grammar,
        id: SymbolId,
        name: &str,
    ) -> Option<SymbolId> {
        if let Some(&token_id) = self.token_symbols.get(&id) {
            return Some(token_id);
        }

        let rule = self.grammar_js.rules.get(name)?.clone();
        let token_id = match rule {
            JsRule::String { value } => {
                self.get_or_create_token(grammar, &value, TokenPattern::String(value.clone()))
            }
            JsRule::Pattern { value } => {
                let token_name = hidden_pattern_token_name(&value);
                self.get_or_create_token(grammar, &token_name, TokenPattern::Regex(value.clone()))
            }
            _ => return None,
        };
        self.token_symbols.insert(id, token_id);
        Some(token_id)
    }
    pub(super) fn get_or_create_token(
        &mut self,
        grammar: &mut Grammar,
        name: &str,
        pattern: TokenPattern,
    ) -> SymbolId {
        // Check if token already exists
        if let Some(&symbol_id) = self.symbol_names.get(name) {
            return symbol_id;
        }

        // Create new token
        let symbol_id = SymbolId(self.next_symbol_id.try_into().unwrap());
        self.symbol_names.insert(name.to_string(), symbol_id);
        self.next_symbol_id += 1;

        let token = Token {
            name: name.to_string(),
            pattern,
            fragile: false,
        };
        grammar.tokens.insert(symbol_id, token);

        symbol_id
    }
}

pub(super) fn hidden_pattern_token_name(pattern: &str) -> String {
    format!("_/{pattern}/")
}
