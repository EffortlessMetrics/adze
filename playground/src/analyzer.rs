// Grammar analyzer for the rust-sitter playground

use crate::{Ambiguity, AnalysisResult, Conflict, GrammarStats, Suggestion, SuggestionLevel};
use anyhow::Result;
use rust_sitter_ir::{Grammar, Symbol, SymbolId};
use std::collections::HashSet;

/// Analyze a grammar and return insights
pub fn analyze_grammar(grammar: &Grammar) -> Result<AnalysisResult> {
    let stats = compute_stats(grammar);
    let conflicts = detect_conflicts(grammar)?;
    let ambiguities = detect_ambiguities(grammar);
    let suggestions = generate_suggestions(grammar, &stats, &conflicts);

    Ok(AnalysisResult {
        grammar_stats: stats,
        conflicts,
        ambiguities,
        suggestions,
    })
}

fn compute_stats(grammar: &Grammar) -> GrammarStats {
    let mut total_rule_length = 0;
    let mut max_rule_length = 0;
    let mut nullable_rules = 0;
    let mut left_recursive_rules = 0;
    let mut right_recursive_rules = 0;

    // Count terminals and non-terminals
    let mut terminals = HashSet::new();
    let mut nonterminals = HashSet::new();

    let mut rule_count = 0;

    for (symbol_id, rules) in &grammar.rules {
        nonterminals.insert(symbol_id.0.to_string());
        for rule in rules {
            let rule_length = rule.rhs.len();
            total_rule_length += rule_length;
            max_rule_length = max_rule_length.max(rule_length);
            rule_count += 1;

            // Check for nullable rules
            if rule.rhs.is_empty() {
                nullable_rules += 1;
            }

            // Check for left recursion
            if let Some(first) = rule.rhs.first() {
                if is_left_recursive(symbol_id, first) {
                    left_recursive_rules += 1;
                }
            }

            // Check for right recursion
            if let Some(last) = rule.rhs.last() {
                if is_right_recursive(symbol_id, last) {
                    right_recursive_rules += 1;
                }
            }

            // Collect terminals
            for symbol in &rule.rhs {
                collect_terminals(symbol, &mut terminals);
            }
        }
    }

    let terminal_count = terminals.len();
    let nonterminal_count = nonterminals.len();

    let avg_rule_length = if rule_count == 0 {
        0.0
    } else {
        total_rule_length as f64 / rule_count as f64
    };

    GrammarStats {
        rule_count,
        terminal_count,
        nonterminal_count,
        max_rule_length,
        avg_rule_length,
        nullable_rules,
        left_recursive_rules,
        right_recursive_rules,
    }
}

#[allow(dead_code)]
fn count_symbols(symbol: &Symbol) -> usize {
    match symbol {
        Symbol::Terminal(_) | Symbol::NonTerminal(_) | Symbol::External(_) | Symbol::Epsilon => 1,
        Symbol::Sequence(seq) | Symbol::Choice(seq) => seq.iter().map(count_symbols).sum(),
        Symbol::Optional(inner) | Symbol::Repeat(inner) | Symbol::RepeatOne(inner) => {
            count_symbols(inner)
        }
    }
}

#[allow(dead_code)]
fn is_nullable(symbol: &Symbol) -> bool {
    match symbol {
        Symbol::Terminal(_) | Symbol::NonTerminal(_) | Symbol::External(_) => false,
        Symbol::Epsilon | Symbol::Optional(_) | Symbol::Repeat(_) => true,
        Symbol::RepeatOne(_) => false,
        Symbol::Sequence(seq) => seq.iter().all(is_nullable),
        Symbol::Choice(choices) => choices.iter().any(is_nullable),
    }
}

fn is_left_recursive(rule_symbol: &SymbolId, symbol: &Symbol) -> bool {
    match symbol {
        Symbol::NonTerminal(name) => name == rule_symbol,
        Symbol::Sequence(seq) => seq
            .first()
            .is_some_and(|s| is_left_recursive(rule_symbol, s)),
        Symbol::Choice(choices) => choices.iter().any(|s| is_left_recursive(rule_symbol, s)),
        Symbol::Optional(inner) | Symbol::Repeat(inner) | Symbol::RepeatOne(inner) => {
            is_left_recursive(rule_symbol, inner)
        }
        _ => false,
    }
}

fn is_right_recursive(rule_symbol: &SymbolId, symbol: &Symbol) -> bool {
    match symbol {
        Symbol::NonTerminal(name) => name == rule_symbol,
        Symbol::Sequence(seq) => seq
            .last()
            .is_some_and(|s| is_right_recursive(rule_symbol, s)),
        Symbol::Choice(choices) => choices.iter().any(|s| is_right_recursive(rule_symbol, s)),
        Symbol::Optional(inner) | Symbol::Repeat(inner) | Symbol::RepeatOne(inner) => {
            is_right_recursive(rule_symbol, inner)
        }
        _ => false,
    }
}

fn collect_terminals(symbol: &Symbol, terminals: &mut HashSet<String>) {
    match symbol {
        Symbol::Terminal(term_id) => {
            terminals.insert(term_id.0.to_string());
        }
        Symbol::Sequence(seq) | Symbol::Choice(seq) => {
            for s in seq {
                collect_terminals(s, terminals);
            }
        }
        Symbol::Optional(inner) | Symbol::Repeat(inner) | Symbol::RepeatOne(inner) => {
            collect_terminals(inner, terminals);
        }
        _ => {}
    }
}

fn detect_conflicts(_grammar: &Grammar) -> Result<Vec<Conflict>> {
    // This would integrate with the GLR parser builder to detect actual conflicts
    // For now, return a placeholder
    Ok(vec![])
}

fn detect_ambiguities(_grammar: &Grammar) -> Vec<Ambiguity> {
    let ambiguities = Vec::new();

    // Detect common ambiguity patterns
    for (_symbol_id, rules) in &_grammar.rules {
        // Check for ambiguous operator precedence
        for _rule in rules {
            // For now, skip ambiguity detection
            // In the future, this would analyze rule patterns
        }
    }

    ambiguities
}

#[allow(dead_code)]
fn is_potentially_ambiguous(_symbol: &Symbol) -> bool {
    // For now, just return false
    // In the future, this would analyze expression patterns
    false
}

#[allow(dead_code)]
fn generate_ambiguous_example(rule_name: &str) -> String {
    // Generate example based on common patterns
    match rule_name {
        "expression" | "expr" => "1 + 2 * 3".to_string(),
        "statement" | "stmt" => "if (a) if (b) c else d".to_string(),
        _ => "a b c".to_string(),
    }
}

fn generate_suggestions(
    _grammar: &Grammar,
    stats: &GrammarStats,
    conflicts: &[Conflict],
) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();

    // Check for high recursion
    if stats.left_recursive_rules > stats.rule_count / 4 {
        suggestions.push(Suggestion {
            level: SuggestionLevel::Warning,
            message: "High amount of left recursion detected. Consider using iterative rules."
                .to_string(),
            rule: None,
        });
    }

    // Check for missing terminals
    if stats.terminal_count == 0 {
        suggestions.push(Suggestion {
            level: SuggestionLevel::Error,
            message: "No terminal symbols found in grammar.".to_string(),
            rule: None,
        });
    }

    // Check for conflicts
    if !conflicts.is_empty() {
        suggestions.push(Suggestion {
            level: SuggestionLevel::Warning,
            message: format!(
                "{} conflicts detected. Consider adding precedence rules.",
                conflicts.len()
            ),
            rule: None,
        });
    }

    // Check for complexity
    if stats.max_rule_length > 10 {
        suggestions.push(Suggestion {
            level: SuggestionLevel::Info,
            message: "Some rules are very long. Consider breaking them into smaller rules."
                .to_string(),
            rule: None,
        });
    }

    // Check for potential performance issues
    if stats.rule_count > 1000 {
        suggestions.push(Suggestion {
            level: SuggestionLevel::Info,
            message: "Large grammar detected. Consider enabling optimizations.".to_string(),
            rule: None,
        });
    }

    suggestions
}
