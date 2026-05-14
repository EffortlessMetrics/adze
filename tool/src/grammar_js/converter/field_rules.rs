use super::{GrammarJsConverter, JsRule};
use adze_ir::{FieldId, Grammar, SymbolId};
use anyhow::Result;

impl GrammarJsConverter {
    pub(super) fn convert_field_rule(
        &mut self,
        grammar: &mut Grammar,
        lhs: SymbolId,
        name: &str,
        content: &JsRule,
    ) -> Result<()> {
        let field_id = self.get_or_create_field(name);
        self.convert_field_symbol_dependency(grammar, content)?;

        eprintln!(
            "Debug: FIELD conversion - lhs: SymbolId({}), field: {}, content: {:?}",
            lhs.0, name, content
        );

        if let JsRule::Choice { members } = content {
            self.convert_field_choice(grammar, lhs, name, field_id, members);
        } else if let Some(symbol) = self.rule_to_symbol(grammar, content) {
            eprintln!("Debug: FIELD resolved to symbol: {:?}", symbol);
            self.add_rule_with_fields(grammar, lhs, vec![symbol], None, None, vec![(field_id, 0)]);
        }
        Ok(())
    }

    fn convert_field_symbol_dependency(
        &mut self,
        grammar: &mut Grammar,
        content: &JsRule,
    ) -> Result<()> {
        if let JsRule::Symbol { name } = content
            && let Some(&content_symbol_id) = self.symbol_names.get(name)
            && let Some(content_rule) = self.grammar_js.rules.get(name).cloned()
        {
            eprintln!("Debug: Converting nested rule {} for field", name);
            self.convert_rule_body(grammar, &content_rule, content_symbol_id)?;
        }
        Ok(())
    }

    fn convert_field_choice(
        &mut self,
        grammar: &mut Grammar,
        lhs: SymbolId,
        name: &str,
        field_id: FieldId,
        members: &[JsRule],
    ) {
        eprintln!("Debug: FIELD contains CHOICE, converting each member with field");
        for (index, member) in members.iter().enumerate() {
            eprintln!(
                "Debug: Converting choice member {} for field {}",
                index, name
            );
            if matches!(member, JsRule::Blank) {
                eprintln!("Debug: Adding empty rule for BLANK with field {}", name);
                self.add_rule(grammar, lhs, vec![], None, None);
            } else if let Some(symbol) = self.rule_to_symbol(grammar, member) {
                eprintln!(
                    "Debug: Adding rule with symbol {:?} and field {}",
                    symbol, name
                );
                self.add_rule_with_fields(
                    grammar,
                    lhs,
                    vec![symbol],
                    None,
                    None,
                    vec![(field_id, 0)],
                );
            }
        }
    }
}
