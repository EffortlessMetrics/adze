use super::GrammarJsConverter;
use adze_ir::{
    Associativity, FieldId, Grammar, PrecedenceKind, ProductionId, Rule, RuleId, Symbol, SymbolId,
};

impl GrammarJsConverter {
    pub(super) fn add_rule(
        &mut self,
        grammar: &mut Grammar,
        lhs: SymbolId,
        rhs: Vec<Symbol>,
        precedence: Option<PrecedenceKind>,
        associativity: Option<Associativity>,
    ) {
        self.add_rule_with_fields(grammar, lhs, rhs, precedence, associativity, Vec::new());
    }

    pub(super) fn add_rule_with_fields(
        &mut self,
        grammar: &mut Grammar,
        lhs: SymbolId,
        rhs: Vec<Symbol>,
        precedence: Option<PrecedenceKind>,
        associativity: Option<Associativity>,
        fields: Vec<(FieldId, usize)>,
    ) {
        eprintln!("Debug: Adding rule for SymbolId({}) -> {:?}", lhs.0, rhs);

        // Check if an identical rule already exists
        let duplicate_exists = grammar.rules.get(&lhs).is_some_and(|existing_rules| {
            existing_rules.iter().any(|r| {
                r.rhs == rhs
                    && r.precedence == precedence
                    && r.associativity == associativity
                    && r.fields == fields
            })
        });

        if duplicate_exists {
            eprintln!(
                "Debug: Skipping duplicate rule for SymbolId({}) -> {:?}",
                lhs.0, rhs
            );
            return;
        }

        let rule = Rule {
            lhs,
            rhs,
            precedence,
            associativity,
            fields,
            production_id: ProductionId(self.next_production_id.try_into().unwrap()),
        };
        self.next_production_id += 1;

        // Calculate rule_id before modifying grammar.rules
        let total_rules = grammar
            .rules
            .values()
            .map(|rules| rules.len())
            .sum::<usize>();
        let rule_id = RuleId(total_rules.try_into().unwrap());
        grammar.production_ids.insert(rule_id, rule.production_id);

        // Now add the rule
        grammar.rules.entry(lhs).or_default().push(rule);
    }
}
