// Grammar analyzer for the rust-sitter playground

use crate::{AnalysisResult, GrammarStats, Conflict, ConflictKind, Ambiguity, Suggestion, SuggestionLevel};
use rust_sitter_ir::{Grammar, Rule, Symbol};
use anyhow::Result;
use std::collections::{HashSet, HashMap};

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
    let mut terminal_count = 0;
    let mut nonterminal_count = 0;
    let mut total_rule_length = 0;
    let mut max_rule_length = 0;
    let mut nullable_rules = 0;
    let mut left_recursive_rules = 0;
    let mut right_recursive_rules = 0;
    
    // Count terminals and non-terminals
    let mut terminals = HashSet::new();
    let mut nonterminals = HashSet::new();
    
    for rule in &grammar.rules {
        nonterminals.insert(&rule.name);
        let rule_length = count_symbols(&rule.body);
        total_rule_length += rule_length;
        max_rule_length = max_rule_length.max(rule_length);
        
        // Check for nullable rules
        if is_nullable(&rule.body) {
            nullable_rules += 1;
        }
        
        // Check for recursion
        if is_left_recursive(&rule.name, &rule.body) {
            left_recursive_rules += 1;
        }
        if is_right_recursive(&rule.name, &rule.body) {
            right_recursive_rules += 1;
        }
        
        // Collect terminals
        collect_terminals(&rule.body, &mut terminals);
    }
    
    terminal_count = terminals.len();
    nonterminal_count = nonterminals.len();
    
    let avg_rule_length = if grammar.rules.is_empty() {
        0.0
    } else {
        total_rule_length as f64 / grammar.rules.len() as f64
    };
    
    GrammarStats {
        rule_count: grammar.rules.len(),
        terminal_count,
        nonterminal_count,
        max_rule_length,
        avg_rule_length,
        nullable_rules,
        left_recursive_rules,
        right_recursive_rules,
    }
}

fn count_symbols(symbol: &Symbol) -> usize {
    match symbol {
        Symbol::Terminal(_) | Symbol::NonTerminal(_) => 1,
        Symbol::Sequence(seq) => seq.iter().map(count_symbols).sum(),
        Symbol::Choice(choices) => choices.iter().map(count_symbols).max().unwrap_or(0),
        Symbol::Repeat(inner) | Symbol::Optional(inner) => count_symbols(inner),
        _ => 0,
    }
}

fn is_nullable(symbol: &Symbol) -> bool {
    match symbol {
        Symbol::Terminal(_) => false,
        Symbol::NonTerminal(_) => false, // Would need to check rule definitions
        Symbol::Optional(_) => true,
        Symbol::Repeat(_) => true,
        Symbol::Sequence(seq) => seq.iter().all(is_nullable),
        Symbol::Choice(choices) => choices.iter().any(is_nullable),
        _ => false,
    }
}

fn is_left_recursive(rule_name: &str, symbol: &Symbol) -> bool {
    match symbol {
        Symbol::NonTerminal(name) => name == rule_name,
        Symbol::Sequence(seq) => {
            if let Some(first) = seq.first() {
                is_left_recursive(rule_name, first)
            } else {
                false
            }
        }
        Symbol::Choice(choices) => choices.iter().any(|s| is_left_recursive(rule_name, s)),
        _ => false,
    }
}

fn is_right_recursive(rule_name: &str, symbol: &Symbol) -> bool {
    match symbol {
        Symbol::NonTerminal(name) => name == rule_name,
        Symbol::Sequence(seq) => {
            if let Some(last) = seq.last() {
                is_right_recursive(rule_name, last)
            } else {
                false
            }
        }
        Symbol::Choice(choices) => choices.iter().any(|s| is_right_recursive(rule_name, s)),
        _ => false,
    }
}

fn collect_terminals(symbol: &Symbol, terminals: &mut HashSet<String>) {
    match symbol {
        Symbol::Terminal(term) => {
            if let Some(value) = &term.value {
                terminals.insert(value.clone());
            }
        }
        Symbol::Sequence(seq) => {
            for s in seq {
                collect_terminals(s, terminals);
            }
        }
        Symbol::Choice(choices) => {
            for s in choices {
                collect_terminals(s, terminals);
            }
        }
        Symbol::Repeat(inner) | Symbol::Optional(inner) => {
            collect_terminals(inner, terminals);
        }
        _ => {}
    }
}

fn detect_conflicts(grammar: &Grammar) -> Result<Vec<Conflict>> {
    // This would integrate with the GLR parser builder to detect actual conflicts
    // For now, return a placeholder
    Ok(vec![])
}

fn detect_ambiguities(grammar: &Grammar) -> Vec<Ambiguity> {
    let mut ambiguities = Vec::new();
    
    // Detect common ambiguity patterns
    for rule in &grammar.rules {
        // Check for ambiguous operator precedence
        if is_potentially_ambiguous(&rule.body) {
            ambiguities.push(Ambiguity {
                rule: rule.name.clone(),
                example: generate_ambiguous_example(&rule.name),
                parse_count: 2,
            });
        }
    }
    
    ambiguities
}

fn is_potentially_ambiguous(symbol: &Symbol) -> bool {
    // Simple heuristic: repeated binary operators without precedence
    match symbol {
        Symbol::Choice(choices) => {
            // Check if multiple choices could match similar patterns
            choices.len() > 1 && choices.iter().any(|s| matches!(s, Symbol::Sequence(_)))
        }
        _ => false,
    }
}

fn generate_ambiguous_example(rule_name: &str) -> String {
    // Generate example based on common patterns
    match rule_name {
        "expression" | "expr" => "1 + 2 * 3".to_string(),
        "statement" | "stmt" => "if (a) if (b) c else d".to_string(),
        _ => "a b c".to_string(),
    }
}

fn generate_suggestions(
    grammar: &Grammar,
    stats: &GrammarStats,
    conflicts: &Vec<Conflict>,
) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();
    
    // Check for high recursion
    if stats.left_recursive_rules > stats.rule_count / 4 {
        suggestions.push(Suggestion {
            level: SuggestionLevel::Warning,
            message: "High amount of left recursion detected. Consider using iterative rules.".to_string(),
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
            message: format!("{} conflicts detected. Consider adding precedence rules.", conflicts.len()),
            rule: None,
        });
    }
    
    // Check for complexity
    if stats.max_rule_length > 10 {
        suggestions.push(Suggestion {
            level: SuggestionLevel::Info,
            message: "Some rules are very long. Consider breaking them into smaller rules.".to_string(),
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