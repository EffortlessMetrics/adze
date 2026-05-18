use super::{GrammarJsConverter, JsRule};
use adze_ir::{FieldId, Grammar, Symbol};

impl GrammarJsConverter {
    pub(super) fn seq_to_rhs_and_fields(
        &mut self,
        grammar: &mut Grammar,
        members: &[JsRule],
    ) -> (Vec<Symbol>, Vec<(FieldId, usize)>) {
        let mut rhs = Vec::new();
        let mut fields = Vec::new();

        for member in members {
            match member {
                JsRule::Field { name, content } => {
                    let field_id = self.get_or_create_field(name);
                    if let Some(symbol) = self.rule_to_symbol(grammar, content) {
                        let position = rhs.len();
                        rhs.push(symbol);
                        if !is_generated_tuple_field_name(name) {
                            fields.push((field_id, position));
                        }
                    } else {
                        eprintln!("Debug: Failed to convert FIELD member {name}");
                    }
                }
                _ => {
                    if let Some(symbol) = self.rule_to_symbol(grammar, member) {
                        rhs.push(symbol);
                    } else {
                        eprintln!("Debug: Failed to convert SEQ member");
                    }
                }
            }
        }

        (rhs, fields)
    }
}

pub(super) fn is_generated_tuple_field_name(name: &str) -> bool {
    let Some((prefix, suffix)) = name.rsplit_once('_') else {
        return false;
    };

    !prefix.is_empty()
        && prefix
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
        && suffix.chars().all(|ch| ch.is_ascii_digit())
}
