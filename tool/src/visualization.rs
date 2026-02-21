// Grammar visualization tools for Adze
// This module provides tools to visualize grammars and parse trees

use adze_ir::{Grammar, Symbol, SymbolId};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

/// Grammar visualizer that generates various output formats
pub struct GrammarVisualizer {
    grammar: Grammar,
}

impl GrammarVisualizer {
    pub fn new(grammar: Grammar) -> Self {
        Self { grammar }
    }

    /// Generate a Graphviz DOT representation of the grammar
    pub fn to_dot(&self) -> String {
        let mut output = String::new();
        writeln!(&mut output, "digraph Grammar {{").unwrap();
        writeln!(&mut output, "  rankdir=LR;").unwrap();
        writeln!(&mut output, "  node [shape=box];").unwrap();

        // Style for different node types
        writeln!(&mut output, "  // Terminals").unwrap();
        for (id, token) in &self.grammar.tokens {
            let label = self.escape_dot(&token.name);
            writeln!(
                &mut output,
                "  t{} [label=\"{}\" shape=ellipse style=filled fillcolor=lightblue];",
                id.0, label
            )
            .unwrap();
        }

        writeln!(&mut output, "\n  // Non-terminals").unwrap();
        for id in self.grammar.rules.keys() {
            let name = self.get_symbol_name(*id);
            writeln!(
                &mut output,
                "  n{} [label=\"{}\" style=filled fillcolor=lightgreen];",
                id.0,
                self.escape_dot(&name)
            )
            .unwrap();
        }

        writeln!(&mut output, "\n  // External tokens").unwrap();
        for external in &self.grammar.externals {
            writeln!(
                &mut output,
                "  e{} [label=\"{}\" shape=diamond style=filled fillcolor=lightcoral];",
                external.symbol_id.0,
                self.escape_dot(&external.name)
            )
            .unwrap();
        }

        writeln!(&mut output, "\n  // Rules").unwrap();
        for (lhs, rules) in &self.grammar.rules {
            for rule in rules {
                for (i, symbol) in rule.rhs.iter().enumerate() {
                    let from = format!("n{}", lhs.0);
                    let to = match symbol {
                        Symbol::Terminal(id) => format!("t{}", id.0),
                        Symbol::NonTerminal(id) => format!("n{}", id.0),
                        Symbol::External(id) => format!("e{}", id.0),
                        Symbol::Optional(_) => format!("opt{}", i),
                        Symbol::Repeat(_) => format!("rep{}", i),
                        Symbol::RepeatOne(_) => format!("rep1{}", i),
                        Symbol::Choice(_) => format!("choice{}", i),
                        Symbol::Sequence(_) => format!("seq{}", i),
                        Symbol::Epsilon => continue, // Skip epsilon transitions in visualization
                    };

                    let label = if rule.rhs.len() > 1 {
                        format!("{}", i + 1)
                    } else {
                        String::new()
                    };

                    writeln!(&mut output, "  {} -> {} [label=\"{}\"];", from, to, label).unwrap();
                }
            }
        }

        writeln!(&mut output, "}}").unwrap();
        output
    }

    /// Generate a railroad diagram in SVG format
    pub fn to_railroad_svg(&self) -> String {
        let mut output = String::new();
        let width = 800;
        let mut y_offset = 50;

        writeln!(
            &mut output,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="600">"#,
            width
        )
        .unwrap();
        writeln!(&mut output, r#"  <style>"#).unwrap();
        writeln!(
            &mut output,
            r#"    .rule-name {{ font-family: monospace; font-weight: bold; }}"#
        )
        .unwrap();
        writeln!(&mut output, r#"    .terminal {{ fill: #4a90e2; }}"#).unwrap();
        writeln!(&mut output, r#"    .non-terminal {{ fill: #7ed321; }}"#).unwrap();
        writeln!(
            &mut output,
            r#"    .line {{ stroke: #333; stroke-width: 2; fill: none; }}"#
        )
        .unwrap();
        writeln!(&mut output, r#"  </style>"#).unwrap();

        // Draw each rule
        for (lhs, rules) in &self.grammar.rules {
            let rule_name = self.get_symbol_name(*lhs);

            for rule in rules {
                // Rule name
                writeln!(
                    &mut output,
                    r#"  <text x="10" y="{}" class="rule-name">{} ::=</text>"#,
                    y_offset,
                    self.escape_xml(&rule_name)
                )
                .unwrap();

                // Rule diagram
                let mut x_offset = 150;
                for symbol in &rule.rhs {
                    let (text, class) = match symbol {
                        Symbol::Terminal(id) => {
                            let token = self
                                .grammar
                                .tokens
                                .get(id)
                                .map(|t| t.name.clone())
                                .unwrap_or_else(|| format!("T{}", id.0));
                            (token, "terminal")
                        }
                        Symbol::NonTerminal(id) => (self.get_symbol_name(*id), "non-terminal"),
                        Symbol::External(id) => (format!("External{}", id.0), "terminal"),
                        Symbol::Optional(inner) => {
                            (format!("{}?", self.format_symbol_simple(inner)), "optional")
                        }
                        Symbol::Repeat(inner) => {
                            (format!("{}*", self.format_symbol_simple(inner)), "repeat")
                        }
                        Symbol::RepeatOne(inner) => {
                            (format!("{}+", self.format_symbol_simple(inner)), "repeat")
                        }
                        Symbol::Choice(choices) => {
                            let choice_text = choices
                                .iter()
                                .map(|s| self.format_symbol_simple(s))
                                .collect::<Vec<_>>()
                                .join(" | ");
                            (format!("({})", choice_text), "choice")
                        }
                        Symbol::Sequence(seq) => {
                            let seq_text = seq
                                .iter()
                                .map(|s| self.format_symbol_simple(s))
                                .collect::<Vec<_>>()
                                .join(" ");
                            (seq_text, "sequence")
                        }
                        Symbol::Epsilon => ("ε".to_string(), "epsilon"),
                    };

                    let text_width = text.len() * 8 + 20;

                    // Draw box
                    writeln!(&mut output, r#"  <rect x="{}" y="{}" width="{}" height="30" rx="5" class="{}" opacity="0.3"/>"#, 
                    x_offset, y_offset - 15, text_width, class).unwrap();

                    // Draw text
                    writeln!(
                        &mut output,
                        r#"  <text x="{}" y="{}" text-anchor="middle">{}</text>"#,
                        x_offset + text_width / 2,
                        y_offset + 5,
                        self.escape_xml(&text)
                    )
                    .unwrap();

                    // Draw connecting line
                    if x_offset > 150 {
                        writeln!(
                            &mut output,
                            r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" class="line"/>"#,
                            x_offset - 10,
                            y_offset,
                            x_offset,
                            y_offset
                        )
                        .unwrap();
                    }

                    x_offset += text_width + 20;
                }

                y_offset += 60;
            }
        }

        writeln!(&mut output, "</svg>").unwrap();
        output
    }

    /// Generate a textual representation of the grammar
    pub fn to_text(&self) -> String {
        let mut output = String::new();

        writeln!(&mut output, "Grammar: {}", self.grammar.name).unwrap();
        writeln!(&mut output, "{}", "=".repeat(50)).unwrap();

        // Tokens
        writeln!(&mut output, "\nTokens:").unwrap();
        for (id, token) in &self.grammar.tokens {
            let pattern = match &token.pattern {
                adze_ir::TokenPattern::String(s) => format!("\"{}\"", s),
                adze_ir::TokenPattern::Regex(r) => format!("/{}/", r),
            };
            writeln!(&mut output, "  {} ({:?}) = {}", token.name, id, pattern).unwrap();
        }

        // External tokens
        if !self.grammar.externals.is_empty() {
            writeln!(&mut output, "\nExternal Tokens:").unwrap();
            for external in &self.grammar.externals {
                writeln!(
                    &mut output,
                    "  {} ({:?})",
                    external.name, external.symbol_id
                )
                .unwrap();
            }
        }

        // Rules
        writeln!(&mut output, "\nRules:").unwrap();
        for (lhs, rules) in &self.grammar.rules {
            let lhs_name = self.get_symbol_name(*lhs);
            for rule in rules {
                write!(&mut output, "  {} ::=", lhs_name).unwrap();

                for symbol in &rule.rhs {
                    match symbol {
                        Symbol::Terminal(id) => {
                            let name = self
                                .grammar
                                .tokens
                                .get(id)
                                .map(|t| t.name.clone())
                                .unwrap_or_else(|| format!("T{}", id.0));
                            write!(&mut output, " '{}'", name).unwrap();
                        }
                        Symbol::NonTerminal(id) => {
                            write!(&mut output, " {}", self.get_symbol_name(*id)).unwrap();
                        }
                        Symbol::External(id) => {
                            write!(&mut output, " ${}", id.0).unwrap();
                        }
                        Symbol::Optional(inner) => {
                            write!(&mut output, " {}?", self.format_symbol_simple(inner)).unwrap();
                        }
                        Symbol::Repeat(inner) => {
                            write!(&mut output, " {}*", self.format_symbol_simple(inner)).unwrap();
                        }
                        Symbol::RepeatOne(inner) => {
                            write!(&mut output, " {}+", self.format_symbol_simple(inner)).unwrap();
                        }
                        Symbol::Choice(choices) => {
                            write!(&mut output, " (").unwrap();
                            for (i, choice) in choices.iter().enumerate() {
                                if i > 0 {
                                    write!(&mut output, " | ").unwrap();
                                }
                                write!(&mut output, "{}", self.format_symbol_simple(choice))
                                    .unwrap();
                            }
                            write!(&mut output, ")").unwrap();
                        }
                        Symbol::Sequence(seq) => {
                            for s in seq {
                                write!(&mut output, " {}", self.format_symbol_simple(s)).unwrap();
                            }
                        }
                        Symbol::Epsilon => {
                            write!(&mut output, " ε").unwrap();
                        }
                    }
                }

                // Add metadata
                if let Some(prec) = &rule.precedence {
                    write!(&mut output, " [precedence: {:?}]", prec).unwrap();
                }
                if let Some(assoc) = &rule.associativity {
                    write!(&mut output, " [associativity: {:?}]", assoc).unwrap();
                }

                writeln!(&mut output).unwrap();
            }
        }

        // Precedences
        if !self.grammar.precedences.is_empty() {
            writeln!(&mut output, "\nPrecedence Declarations:").unwrap();
            for prec in &self.grammar.precedences {
                write!(&mut output, "  Level {}: ", prec.level).unwrap();
                for symbol in &prec.symbols {
                    write!(&mut output, "{:?} ", symbol).unwrap();
                }
                writeln!(&mut output, "({:?})", prec.associativity).unwrap();
            }
        }

        // Conflicts
        if !self.grammar.conflicts.is_empty() {
            writeln!(&mut output, "\nConflict Declarations:").unwrap();
            for conflict in &self.grammar.conflicts {
                write!(&mut output, "  Symbols: ").unwrap();
                for symbol in &conflict.symbols {
                    write!(&mut output, "{:?} ", symbol).unwrap();
                }
                writeln!(&mut output, "Resolution: {:?}", conflict.resolution).unwrap();
            }
        }

        output
    }

    /// Generate dependency graph showing which symbols depend on which
    pub fn dependency_graph(&self) -> String {
        let mut output = String::new();
        let mut dependencies: HashMap<SymbolId, HashSet<SymbolId>> = HashMap::new();

        // Build dependency map
        for (lhs, rules) in &self.grammar.rules {
            let deps = dependencies.entry(*lhs).or_default();
            for rule in rules {
                for symbol in &rule.rhs {
                    if let Symbol::NonTerminal(id) = symbol {
                        deps.insert(*id);
                    }
                }
            }
        }

        writeln!(&mut output, "Symbol Dependencies:").unwrap();
        writeln!(&mut output, "===================").unwrap();

        for (symbol, deps) in dependencies {
            let symbol_name = self.get_symbol_name(symbol);
            write!(&mut output, "{} depends on:", symbol_name).unwrap();

            if deps.is_empty() {
                write!(&mut output, " (none)").unwrap();
            } else {
                for dep in deps {
                    write!(&mut output, " {}", self.get_symbol_name(dep)).unwrap();
                }
            }
            writeln!(&mut output).unwrap();
        }

        output
    }

    fn get_symbol_name(&self, id: SymbolId) -> String {
        // Check tokens
        if let Some(token) = self.grammar.tokens.get(&id) {
            return token.name.clone();
        }

        // Check if it's a rule
        if self.grammar.rules.contains_key(&id) {
            return format!("rule_{}", id.0);
        }

        // Check externals
        for external in &self.grammar.externals {
            if external.symbol_id == id {
                return external.name.clone();
            }
        }

        format!("symbol_{}", id.0)
    }

    fn format_symbol_simple(&self, symbol: &Symbol) -> String {
        match symbol {
            Symbol::Terminal(id) => self
                .grammar
                .tokens
                .get(id)
                .map(|t| t.name.clone())
                .unwrap_or_else(|| format!("T{}", id.0)),
            Symbol::NonTerminal(id) => self.get_symbol_name(*id),
            Symbol::External(id) => format!("External{}", id.0),
            Symbol::Optional(inner) => format!("{}?", self.format_symbol_simple(inner)),
            Symbol::Repeat(inner) => format!("{}*", self.format_symbol_simple(inner)),
            Symbol::RepeatOne(inner) => format!("{}+", self.format_symbol_simple(inner)),
            Symbol::Choice(choices) => {
                let parts: Vec<_> = choices
                    .iter()
                    .map(|s| self.format_symbol_simple(s))
                    .collect();
                format!("({})", parts.join("|"))
            }
            Symbol::Sequence(seq) => {
                let parts: Vec<_> = seq.iter().map(|s| self.format_symbol_simple(s)).collect();
                parts.join(" ")
            }
            Symbol::Epsilon => "ε".to_string(),
        }
    }

    fn escape_dot(&self, s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    }

    fn escape_xml(&self, s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

// Note: TreeVisualizer for parse trees should be implemented in the runtime crate
// where tree_sitter types are available, not in the tool crate

#[cfg(test)]
mod tests {
    use super::*;
    use adze_ir::Rule;
    use adze_ir::{ProductionId, Token, TokenPattern};

    #[test]
    fn test_grammar_to_text() {
        let mut grammar = Grammar::new("test".to_string());

        let id_sym = SymbolId(1);
        grammar.tokens.insert(
            id_sym,
            Token {
                name: "identifier".to_string(),
                pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
                fragile: false,
            },
        );

        let expr_sym = SymbolId(2);
        grammar.rules.insert(
            expr_sym,
            vec![Rule {
                lhs: expr_sym,
                rhs: vec![Symbol::Terminal(id_sym)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            }],
        );

        let visualizer = GrammarVisualizer::new(grammar);
        let text = visualizer.to_text();

        assert!(text.contains("Grammar: test"));
        assert!(text.contains("identifier"));
        assert!(text.contains("rule_2 ::= 'identifier'"));
    }

    #[test]
    fn test_dot_generation() {
        let grammar = Grammar::new("test".to_string());
        let visualizer = GrammarVisualizer::new(grammar);
        let dot = visualizer.to_dot();

        assert!(dot.contains("digraph Grammar"));
        assert!(dot.contains("rankdir=LR"));
    }
}
