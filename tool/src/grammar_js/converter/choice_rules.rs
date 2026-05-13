use super::{GrammarJsConverter, JsRule};
use adze_ir::{Associativity, Grammar, PrecedenceKind, SymbolId};
use anyhow::Result;

impl GrammarJsConverter {
    pub(super) fn convert_choice_rule(
        &mut self,
        grammar: &mut Grammar,
        lhs: SymbolId,
        members: &[JsRule],
    ) -> Result<()> {
        eprintln!(
            "Debug: Converting CHOICE for {} with {} members",
            lhs.0,
            members.len()
        );

        for (index, member) in members.iter().enumerate() {
            eprintln!("Debug: Converting choice member {} for {}", index, lhs.0);
            self.convert_choice_member(grammar, lhs, member, index)?;
        }
        Ok(())
    }

    fn convert_choice_member(
        &mut self,
        grammar: &mut Grammar,
        lhs: SymbolId,
        member: &JsRule,
        index: usize,
    ) -> Result<()> {
        match member {
            JsRule::Prec { value, content } => self.convert_choice_member_with_precedence(
                grammar,
                lhs,
                content,
                *value,
                None,
                "precedence",
            ),
            JsRule::PrecLeft { value, content } => self.convert_choice_member_with_precedence(
                grammar,
                lhs,
                content,
                *value,
                Some(Associativity::Left),
                "left precedence",
            ),
            JsRule::PrecRight { value, content } => self.convert_choice_member_with_precedence(
                grammar,
                lhs,
                content,
                *value,
                Some(Associativity::Right),
                "right precedence",
            ),
            JsRule::Seq { members } => {
                eprintln!(
                    "Debug: CHOICE member {} is SEQ with {} members for {}",
                    index,
                    members.len(),
                    lhs.0
                );
                let (rhs, fields) = self.seq_to_rhs_and_fields(grammar, members);
                if !rhs.is_empty() {
                    eprintln!(
                        "Debug: Adding rule {} -> {:?} (from inlined SEQ)",
                        lhs.0, rhs
                    );
                    self.add_rule_with_fields(grammar, lhs, rhs, None, None, fields);
                }
                Ok(())
            }
            _ => {
                if let Some(symbol) = self.rule_to_symbol(grammar, member) {
                    eprintln!("Debug: Adding rule {} -> {:?}", lhs.0, symbol);
                    self.add_rule(grammar, lhs, vec![symbol], None, None);
                } else {
                    eprintln!(
                        "Debug: Failed to convert choice member {} for {}",
                        index, lhs.0
                    );
                }
                Ok(())
            }
        }
    }

    fn convert_choice_member_with_precedence(
        &mut self,
        grammar: &mut Grammar,
        lhs: SymbolId,
        content: &JsRule,
        value: i32,
        associativity: Option<Associativity>,
        label: &str,
    ) -> Result<()> {
        let precedence = Some(PrecedenceKind::Static(value as i16));
        if let JsRule::Seq { members } = content {
            let (rhs, fields) = self.seq_to_rhs_and_fields(grammar, members);
            if !rhs.is_empty() {
                eprintln!(
                    "Debug: Adding rule {} -> {:?} with {} {}",
                    lhs.0, rhs, label, value
                );
                self.add_rule_with_fields(grammar, lhs, rhs, precedence, associativity, fields);
                return Ok(());
            }
        }

        if let Some(symbol) = self.rule_to_symbol(grammar, content) {
            eprintln!(
                "Debug: Adding rule {} -> {:?} with {} {}",
                lhs.0, symbol, label, value
            );
            self.add_rule(grammar, lhs, vec![symbol], precedence, associativity);
        }
        Ok(())
    }
}
