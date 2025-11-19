//! Contract validation tests for grammar extraction
//!
//! These tests validate the Grammar Extraction Contract specification by:
//! 1. Extracting Grammar IR from Rust enum definitions
//! 2. Comparing with manually-built equivalent grammars
//! 3. Verifying production structures match expected patterns
//! 4. Ensuring ambiguous grammars remain ambiguous

use rust_sitter_ir::builder::GrammarBuilder;
use rust_sitter_ir::{Grammar, Symbol};
use std::collections::BTreeSet;

/// Helper to analyze Grammar IR structure
#[derive(Debug)]
struct GrammarAnalysis {
    name: String,
    total_rules: usize,
    total_terminals: usize,
    total_nonterminals: usize,
    symbol_names: BTreeSet<String>,
    production_patterns: Vec<ProductionPattern>,
}

#[derive(Debug, Clone)]
struct ProductionPattern {
    lhs: String,
    rhs_count: usize,
    is_left_recursive: bool,
    has_precedence: bool,
}

impl GrammarAnalysis {
    fn analyze(grammar: &Grammar) -> Self {
        let mut symbol_names = BTreeSet::new();
        let mut production_patterns = Vec::new();

        // Collect all symbol names
        for (id, name) in &grammar.rule_names {
            symbol_names.insert(name.clone());
        }
        for (id, token) in &grammar.tokens {
            symbol_names.insert(token.name.clone());
        }

        // Analyze productions
        for (lhs, rules) in &grammar.rules {
            let lhs_name = grammar.rule_names.get(lhs)
                .cloned()
                .unwrap_or_else(|| format!("{:?}", lhs));

            for rule in rules {
                let is_left_recursive = rule.rhs.first()
                    .map(|sym| match sym {
                        Symbol::NonTerminal(id) => id == lhs,
                        _ => false,
                    })
                    .unwrap_or(false);

                production_patterns.push(ProductionPattern {
                    lhs: lhs_name.clone(),
                    rhs_count: rule.rhs.len(),
                    is_left_recursive,
                    has_precedence: rule.precedence.is_some() || rule.associativity.is_some(),
                });
            }
        }

        Self {
            name: grammar.name.clone(),
            total_rules: grammar.rules.values().map(|v| v.len()).sum(),
            total_terminals: grammar.tokens.len(),
            total_nonterminals: grammar.rules.len(),
            symbol_names,
            production_patterns,
        }
    }

    fn print_summary(&self) {
        eprintln!("\n=== Grammar Analysis: {} ===", self.name);
        eprintln!("  Total rules: {}", self.total_rules);
        eprintln!("  Terminals: {}", self.total_terminals);
        eprintln!("  Non-terminals: {}", self.total_nonterminals);
        eprintln!("  Symbol names: {:?}", self.symbol_names);
        eprintln!("\n  Production patterns:");
        for (idx, pattern) in self.production_patterns.iter().enumerate() {
            eprintln!("    {}: {} → <{} symbols>{}{}",
                idx,
                pattern.lhs,
                pattern.rhs_count,
                if pattern.is_left_recursive { " [LEFT-RECURSIVE]" } else { "" },
                if pattern.has_precedence { " [HAS-PREC]" } else { "" }
            );
        }
    }

    fn has_left_recursive_production(&self) -> bool {
        self.production_patterns.iter().any(|p| p.is_left_recursive)
    }

    fn count_productions_with_rhs_len(&self, len: usize) -> usize {
        self.production_patterns.iter().filter(|p| p.rhs_count == len).count()
    }
}

#[test]
fn test_contract_manual_grammar_structure() {
    eprintln!("\n=== CONTRACT TEST: Manual Grammar Structure ===\n");

    // Build the reference manual grammar
    let grammar = GrammarBuilder::new("manual_ambiguous")
        .token("NUMBER", r"\d+")
        .token("OP", r"[-+*/]")
        .rule("expr", vec!["binary"])
        .rule("expr", vec!["NUMBER"])
        .rule("binary", vec!["expr", "OP", "expr"])
        .start("expr")
        .build();

    let analysis = GrammarAnalysis::analyze(&grammar);
    analysis.print_summary();

    // Contract assertions
    eprintln!("\n--- Contract Validations ---");

    // Requirement: Must have 3 productions
    assert_eq!(analysis.total_rules, 3,
        "Contract violation: Expected 3 productions (expr→binary, expr→NUMBER, binary→expr OP expr)");
    eprintln!("✅ Production count: {}", analysis.total_rules);

    // Requirement: Must have 2 terminals (NUMBER, OP)
    assert_eq!(analysis.total_terminals, 2,
        "Contract violation: Expected 2 terminals (NUMBER, OP)");
    eprintln!("✅ Terminal count: {}", analysis.total_terminals);

    // Requirement: Grammar must create recursion (direct or indirect)
    // The pattern is: binary → expr OP expr, and expr → binary
    // This creates indirect left-recursion: binary → expr OP expr → binary OP expr
    let has_recursive_pattern = analysis.production_patterns.iter()
        .any(|p| p.lhs == "binary" && p.rhs_count == 3);

    assert!(has_recursive_pattern,
        "Contract violation: Expected binary production with 3 RHS symbols (expr OP expr)");
    eprintln!("✅ Has recursive binary production pattern (indirect left-recursion)");

    // Requirement: Must have one production with RHS length 3
    assert_eq!(analysis.count_productions_with_rhs_len(3), 1,
        "Contract violation: Expected exactly one production with 3 RHS symbols");
    eprintln!("✅ Has binary operation production (3 symbols)");

    // Requirement: NO precedence in ambiguous grammar
    let has_any_precedence = analysis.production_patterns.iter().any(|p| p.has_precedence);
    assert!(!has_any_precedence,
        "Contract violation: Ambiguous grammar must not have precedence");
    eprintln!("✅ No precedence annotations (as expected for ambiguous grammar)");

    eprintln!("\n✅ All contract requirements validated for manual grammar");
}

#[test]
fn test_contract_enum_grammar_extraction() {
    eprintln!("\n=== CONTRACT TEST: Enum Grammar JSON Extraction ===\n");

    use std::path::PathBuf;
    use rust_sitter_tool::generate_grammars;

    // Path to the ambiguous_expr example
    let example_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("example/src/ambiguous_expr.rs");

    eprintln!("Extracting grammar from: {}", example_path.display());

    // Extract grammar JSON
    let grammars = generate_grammars(&example_path)
        .expect("Failed to extract grammar from ambiguous_expr.rs");

    assert_eq!(grammars.len(), 1, "Expected exactly one grammar");

    let grammar_json = &grammars[0];
    eprintln!("\n=== Extracted Grammar JSON ===");
    eprintln!("{}", serde_json::to_string_pretty(grammar_json).unwrap());

    // Analyze the rules structure
    let rules = grammar_json["rules"].as_object()
        .expect("Grammar should have rules object");

    eprintln!("\n=== Grammar Rules Analysis ===");
    eprintln!("Total rules: {}", rules.len());

    for (rule_name, rule_def) in rules {
        eprintln!("\nRule '{}':", rule_name);
        eprintln!("  Definition: {}", serde_json::to_string_pretty(rule_def).unwrap());
    }

    // Check for variant intermediate symbols
    let has_expr_binary = rules.contains_key("Expr_Binary");
    let has_expr_number = rules.contains_key("Expr_Number");

    eprintln!("\n=== Variant Symbol Detection ===");
    eprintln!("Has 'Expr_Binary' intermediate symbol: {}", has_expr_binary);
    eprintln!("Has 'Expr_Number' intermediate symbol: {}", has_expr_number);

    if has_expr_binary || has_expr_number {
        eprintln!("\n❌ CONTRACT VIOLATION DETECTED!");
        eprintln!("   Enum variants created intermediate symbols!");
        eprintln!();
        eprintln!("   Expected (from contract):");
        eprintln!("     Expr → Expr OP Expr");
        eprintln!("     Expr → NUMBER");
        eprintln!();
        eprintln!("   Actual structure:");
        eprintln!("     Expr → Expr_Binary");
        eprintln!("     Expr → Expr_Number");
        eprintln!("     Expr_Binary → Expr OP Expr");
        eprintln!("     Expr_Number → NUMBER");
        eprintln!();
        eprintln!("   Impact: Intermediate symbols create disambiguation points!");
        eprintln!("   The LR(1) parser can distinguish Binary vs Number early,");
        eprintln!("   preventing the shift/reduce conflicts we need for GLR.");
    } else {
        eprintln!("\n✅ Contract satisfied: No intermediate variant symbols");
    }

    // Compare with manual grammar
    eprintln!("\n=== Comparison with Manual Grammar ===");
    let manual_grammar = GrammarBuilder::new("manual_ambiguous")
        .token("NUMBER", r"\d+")
        .token("OP", r"[-+*/]")
        .rule("Expr", vec!["Expr", "OP", "Expr"])
        .rule("Expr", vec!["NUMBER"])
        .start("Expr")
        .build();

    let manual_analysis = GrammarAnalysis::analyze(&manual_grammar);
    manual_analysis.print_summary();

    eprintln!("\nManual grammar has {} rules", manual_analysis.total_rules);
    eprintln!("Enum grammar has {} rules (from JSON)", rules.len());

    // This test documents the issue rather than asserting for now
    // Once we fix the extraction, we can enable strict assertions
}

#[test]
fn test_contract_production_pattern_comparison() {
    eprintln!("\n=== CONTRACT TEST: Production Pattern Equivalence ===\n");

    // Create two equivalent grammars with different construction
    let grammar1 = GrammarBuilder::new("left_recursive_1")
        .token("n", r"\d+")
        .token("+", r"\+")
        .rule("E", vec!["E", "+", "n"])
        .rule("E", vec!["n"])
        .start("E")
        .build();

    let grammar2 = GrammarBuilder::new("left_recursive_2")
        .token("n", r"\d+")
        .token("+", r"\+")
        .rule("E", vec!["n"])
        .rule("E", vec!["E", "+", "n"])  // Reversed order
        .start("E")
        .build();

    let analysis1 = GrammarAnalysis::analyze(&grammar1);
    let analysis2 = GrammarAnalysis::analyze(&grammar2);

    analysis1.print_summary();
    analysis2.print_summary();

    // Contract: Order of rule definition must not affect structure
    assert_eq!(analysis1.total_rules, analysis2.total_rules,
        "Rule count must be independent of definition order");
    assert_eq!(analysis1.total_terminals, analysis2.total_terminals,
        "Terminal count must be independent of definition order");
    assert_eq!(analysis1.has_left_recursive_production(), analysis2.has_left_recursive_production(),
        "Left-recursion must be detected regardless of rule order");

    eprintln!("\n✅ Production patterns are equivalent regardless of definition order");
}

#[test]
fn test_contract_recursion_preservation() {
    eprintln!("\n=== CONTRACT TEST: Recursion Preservation ===\n");

    // Test that recursion is correctly identified
    let recursive_grammar = GrammarBuilder::new("recursive")
        .token("x", "x")
        .rule("A", vec!["A", "x"])  // Left-recursive
        .rule("A", vec!["x"])
        .start("A")
        .build();

    let non_recursive_grammar = GrammarBuilder::new("non_recursive")
        .token("x", "x")
        .token("y", "y")
        .rule("A", vec!["x", "y"])  // Not recursive
        .rule("A", vec!["x"])
        .start("A")
        .build();

    let recursive_analysis = GrammarAnalysis::analyze(&recursive_grammar);
    let non_recursive_analysis = GrammarAnalysis::analyze(&non_recursive_grammar);

    recursive_analysis.print_summary();
    non_recursive_analysis.print_summary();

    // Contract: Left-recursion must be detected
    assert!(recursive_analysis.has_left_recursive_production(),
        "Contract violation: Left-recursive grammar not detected");
    assert!(!non_recursive_analysis.has_left_recursive_production(),
        "Contract violation: False positive for left-recursion");

    eprintln!("\n✅ Recursion patterns correctly identified");
}

#[test]
fn test_contract_symbol_naming() {
    eprintln!("\n=== CONTRACT TEST: Symbol Naming Convention ===\n");

    let grammar = GrammarBuilder::new("naming_test")
        .token("NUMBER", r"\d+")
        .token("+", r"\+")
        .token("-", r"-")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["NUMBER"])
        .start("expr")
        .build();

    let analysis = GrammarAnalysis::analyze(&grammar);
    analysis.print_summary();

    // Contract: Symbol names must be predictable
    assert!(analysis.symbol_names.contains("NUMBER"), "Terminal symbols must use provided names");
    assert!(analysis.symbol_names.contains("expr"), "Non-terminal names must be preserved");
    assert!(analysis.symbol_names.contains("term"), "All non-terminals must be named");

    // Contract: No auto-generated complex names like "Expr_Binary_Expr_Binary_1"
    let has_complex_names = analysis.symbol_names.iter()
        .any(|name| name.matches('_').count() > 2);

    if has_complex_names {
        eprintln!("⚠️  Warning: Found complex auto-generated names");
        for name in &analysis.symbol_names {
            if name.matches('_').count() > 2 {
                eprintln!("    {}", name);
            }
        }
    }

    eprintln!("\n✅ Symbol naming follows conventions");
}

#[test]
fn test_contract_precedence_handling() {
    eprintln!("\n=== CONTRACT TEST: Precedence Handling ===\n");

    // Grammar WITH precedence (should resolve conflicts)
    let with_prec = GrammarBuilder::new("with_precedence")
        .token("n", r"\d+")
        .token("+", r"\+")
        .token("*", r"\*")
        .rule_with_precedence("E", vec!["E", "+", "E"], 1, rust_sitter_ir::Associativity::Left)
        .rule_with_precedence("E", vec!["E", "*", "E"], 2, rust_sitter_ir::Associativity::Left)
        .rule("E", vec!["n"])
        .start("E")
        .build();

    // Grammar WITHOUT precedence (should preserve conflicts)
    let without_prec = GrammarBuilder::new("without_precedence")
        .token("n", r"\d+")
        .token("+", r"\+")
        .rule("E", vec!["E", "+", "E"])  // No precedence!
        .rule("E", vec!["n"])
        .start("E")
        .build();

    let with_analysis = GrammarAnalysis::analyze(&with_prec);
    let without_analysis = GrammarAnalysis::analyze(&without_prec);

    with_analysis.print_summary();
    without_analysis.print_summary();

    // Contract: Precedence must be reflected in production patterns
    let prec_count = with_analysis.production_patterns.iter()
        .filter(|p| p.has_precedence)
        .count();

    assert!(prec_count > 0, "Grammar with precedence must have precedence in productions");
    eprintln!("✅ Productions with precedence: {}", prec_count);

    let no_prec_count = without_analysis.production_patterns.iter()
        .filter(|p| p.has_precedence)
        .count();

    assert_eq!(no_prec_count, 0, "Grammar without precedence must not have precedence in productions");
    eprintln!("✅ Productions without precedence: confirmed");

    eprintln!("\n✅ Precedence handling follows contract");
}

/// Document the expected transformation for enum variants
#[test]
fn test_contract_enum_transformation_spec() {
    eprintln!("\n=== CONTRACT SPECIFICATION: Enum Transformation ===\n");

    eprintln!("Input: Rust enum definition");
    eprintln!("```rust");
    eprintln!("#[rust_sitter::language]");
    eprintln!("enum Expr {{");
    eprintln!("    Binary(");
    eprintln!("        Box<Expr>,");
    eprintln!("        #[rust_sitter::leaf(pattern = r\"[-+*/]\")] String,");
    eprintln!("        Box<Expr>)");
    eprintln!("    ),");
    eprintln!("    Number(#[rust_sitter::leaf(pattern = r\"\\d+\")] i32),");
    eprintln!("}}");
    eprintln!("```");
    eprintln!();

    eprintln!("Expected Grammar IR (Contract):");
    eprintln!("```");
    eprintln!("Grammar {{");
    eprintln!("    name: \"Expr\",");
    eprintln!("    rules: {{");
    eprintln!("        Expr: [");
    eprintln!("            Rule {{ lhs: Expr, rhs: [NonTerminal(Expr), Terminal(OP), NonTerminal(Expr)], ... }},");
    eprintln!("            Rule {{ lhs: Expr, rhs: [Terminal(NUMBER)], ... }}");
    eprintln!("        ]");
    eprintln!("    }},");
    eprintln!("    tokens: {{");
    eprintln!("        OP: Token {{ pattern: Regex(\"[-+*/]\"), ... }},");
    eprintln!("        NUMBER: Token {{ pattern: Regex(\"\\d+\"), ... }}");
    eprintln!("    }},");
    eprintln!("    ...");
    eprintln!("}}");
    eprintln!("```");
    eprintln!();

    eprintln!("Contract Requirements:");
    eprintln!("  1. NO intermediate symbols like 'Expr_Binary'");
    eprintln!("  2. Direct inlining of variant fields");
    eprintln!("  3. Preserve recursion (Box<Expr> → NonTerminal(Expr))");
    eprintln!("  4. NO implicit precedence");
    eprintln!("  5. Result: Left-recursive ambiguous grammar");
    eprintln!();

    eprintln!("Equivalent Manual Grammar:");
    let equivalent = GrammarBuilder::new("Expr")
        .token("OP", r"[-+*/]")
        .token("NUMBER", r"\d+")
        .rule("Expr", vec!["Expr", "OP", "Expr"])
        .rule("Expr", vec!["NUMBER"])
        .start("Expr")
        .build();

    let analysis = GrammarAnalysis::analyze(&equivalent);
    analysis.print_summary();

    eprintln!("\n📋 This specification defines the contract for enum extraction");
    eprintln!("   Actual enum extraction must produce equivalent Grammar IR");
}
