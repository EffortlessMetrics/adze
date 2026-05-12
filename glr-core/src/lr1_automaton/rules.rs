use crate::{Grammar, ParseRule};
use std::collections::BTreeMap;

pub(crate) struct RuleInfo {
    pub(crate) rules: Vec<ParseRule>,
    pub(crate) dynamic_prec_by_rule: Vec<i16>,
    pub(crate) rule_assoc_by_rule: Vec<i8>,
    pub(crate) production_to_rule_id: BTreeMap<u16, u16>,
}

pub(crate) fn collect(grammar: &Grammar) -> RuleInfo {
    let mut rules = Vec::new();
    let mut dynamic_prec_by_rule = Vec::new();
    let mut rule_assoc_by_rule = Vec::new();
    let mut production_to_rule_id = BTreeMap::new();

    for (rule_id, rule) in grammar.all_rules().enumerate() {
        production_to_rule_id.insert(rule.production_id.0, rule_id as u16);
        rules.push(ParseRule {
            lhs: rule.lhs,
            rhs_len: rule.rhs.len() as u16,
        });

        let prec = match rule.precedence {
            Some(adze_ir::PrecedenceKind::Static(p)) => p,
            Some(adze_ir::PrecedenceKind::Dynamic(p)) => p,
            None => 0,
        };
        dynamic_prec_by_rule.push(prec);

        let assoc = match rule.associativity {
            Some(adze_ir::Associativity::Left) => 1,
            Some(adze_ir::Associativity::Right) => -1,
            _ => 0,
        };
        rule_assoc_by_rule.push(assoc);
    }

    RuleInfo {
        rules,
        dynamic_prec_by_rule,
        rule_assoc_by_rule,
        production_to_rule_id,
    }
}
