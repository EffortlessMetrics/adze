//! Wrappers for common `insta` snapshot-testing patterns.
//!
//! These helpers standardise snapshot naming, redaction, and grammar
//! serialization so every crate uses consistent conventions.

use adze_ir::Grammar;

/// Serialise a grammar to a deterministic, snapshot-friendly string.
///
/// The output includes the grammar name, token list, and rule list in a
/// human-readable format that produces stable diffs.
pub fn grammar_snapshot(grammar: &Grammar) -> String {
    let mut out = String::new();

    out.push_str(&format!("Grammar: {}\n", grammar.name));
    out.push_str(&format!("Tokens ({})\n", grammar.tokens.len()));
    for token in grammar.tokens.values() {
        out.push_str(&format!("  {} = {:?}\n", token.name, token.pattern));
    }

    out.push_str(&format!("Rules ({})\n", grammar.rules.len()));
    for (sym_id, rules) in &grammar.rules {
        let name = grammar
            .rule_names
            .get(sym_id)
            .map(String::as_str)
            .unwrap_or("<anon>");
        out.push_str(&format!("  {name} ({} productions)\n", rules.len()));
        for rule in rules {
            let rhs: Vec<String> = rule.rhs.iter().map(|s| format!("{s:?}")).collect();
            out.push_str(&format!("    -> {}\n", rhs.join(" ")));
        }
    }

    out
}

/// Serialise a grammar to pretty-printed JSON for snapshot comparison.
pub fn grammar_json_snapshot(grammar: &Grammar) -> String {
    serde_json::to_string_pretty(grammar).expect("grammar should be JSON-serialisable")
}

/// Format parse table summary as a snapshot-friendly string.
///
/// Includes state count, symbol count, and dimensions – but NOT the
/// full tables (which are too large for readable snapshots).
pub fn parse_table_summary_snapshot(table: &adze_glr_core::ParseTable) -> String {
    let mut out = String::new();
    out.push_str(&format!("States: {}\n", table.state_count));
    out.push_str(&format!("Symbols: {}\n", table.symbol_count));
    out.push_str(&format!("Tokens: {}\n", table.token_count));
    out.push_str(&format!(
        "External tokens: {}\n",
        table.external_token_count
    ));
    out.push_str(&format!("Rules: {}\n", table.rules.len()));
    out.push_str(&format!("EOF symbol: {:?}\n", table.eof_symbol));
    out.push_str(&format!("Start symbol: {:?}\n", table.start_symbol));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar_helpers::{arithmetic_grammar, build_parse_table, trivial_grammar};

    #[test]
    fn grammar_snapshot_is_deterministic() {
        let g = trivial_grammar();
        let s1 = grammar_snapshot(&g);
        let s2 = grammar_snapshot(&g);
        assert_eq!(s1, s2);
    }

    #[test]
    fn grammar_snapshot_contains_name() {
        let g = arithmetic_grammar();
        let snap = grammar_snapshot(&g);
        assert!(snap.contains("Grammar: arithmetic"));
        assert!(snap.contains("NUMBER"));
    }

    #[test]
    fn grammar_json_snapshot_roundtrips() {
        let g = trivial_grammar();
        let json = grammar_json_snapshot(&g);
        let parsed: Grammar = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, g.name);
    }

    #[test]
    fn parse_table_summary_contains_counts() {
        let g = trivial_grammar();
        let table = build_parse_table(&g).unwrap();
        let summary = parse_table_summary_snapshot(&table);
        assert!(summary.contains("States:"));
        assert!(summary.contains("Symbols:"));
    }
}
