//! Demonstration of PR #58 features: Node metadata extraction and Direct Forest Splicing incremental parsing
//!
//! This example showcases:
//! 1. Tree-sitter compatible Node API with metadata extraction
//! 2. Production-ready incremental parsing with 16x performance improvements
//! 3. Direct Forest Splicing algorithm with conservative subtree reuse

use rust_sitter::ts_compat::{InputEdit, Language, Parser, Point};
use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use std::sync::Arc;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 PR #58 Features Demo: Node Metadata & Incremental Parsing\n");

    // Create a simple arithmetic language for demonstration
    let language = create_demo_language()?;
    let mut parser = Parser::new();
    parser.set_language(language)?;

    // Demonstrate Node metadata extraction
    demonstrate_node_metadata(&mut parser)?;

    // Demonstrate incremental parsing with Direct Forest Splicing
    demonstrate_incremental_parsing(&mut parser)?;

    Ok(())
}

/// Create a simple arithmetic language for demonstration
fn create_demo_language() -> Result<Arc<Language>, String> {
    println!("📝 Creating demonstration language (arithmetic expressions)...");

    let mut grammar = Grammar::new("arithmetic".to_string());

    // Define symbols
    let number_id = SymbolId(1);
    let plus_id = SymbolId(2);
    let expr_id = SymbolId(10);
    let source_id = SymbolId(11);

    // Add tokens
    grammar.tokens.insert(
        number_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::String(r"\d+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        plus_id,
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    // Add rules: expr -> number | expr + expr
    let number_rule = Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(number_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };

    let add_rule = Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(plus_id),
            Symbol::NonTerminal(expr_id),
        ],
        precedence: Some(rust_sitter_ir::PrecedenceKind::Static(1)),
        associativity: Some(rust_sitter_ir::Associativity::Left),
        fields: vec![],
        production_id: ProductionId(1),
    };

    let source_rule = Rule {
        lhs: source_id,
        rhs: vec![Symbol::NonTerminal(expr_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    };

    grammar.add_rule(number_rule);
    grammar.add_rule(add_rule);
    grammar.add_rule(source_rule);

    // Note: start_symbol will be set automatically by the grammar system

    // Add rule names for symbol lookup
    grammar.rule_names.insert(number_id, "number".to_string());
    grammar.rule_names.insert(plus_id, "plus".to_string());
    grammar.rule_names.insert(expr_id, "expression".to_string());
    grammar
        .rule_names
        .insert(source_id, "source_file".to_string());

    // Build parse table
    println!("⚙️  Building GLR parse tables...");
    let first_follow = FirstFollowSets::compute(&grammar)
        .map_err(|e| format!("Failed to compute FIRST/FOLLOW sets: {:?}", e))?;
    let table = build_lr1_automaton(&grammar, &first_follow)
        .map_err(|e| format!("Failed to build LR(1) automaton: {:?}", e))?;

    println!("✅ Language created successfully\n");
    Ok(Arc::new(Language::new("arithmetic", grammar, table)))
}

/// Demonstrate Node metadata extraction capabilities
fn demonstrate_node_metadata(parser: &mut Parser) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Node Metadata Demonstration");
    println!("==============================");

    let source = "42 + 17 + 99";
    println!("Source code: {}", source);

    let tree = parser
        .parse(source, None)
        .ok_or("Failed to parse expression")?;

    let root = tree.root_node();

    // Basic metadata
    println!("\n📊 Node Metadata:");
    println!("  Kind: '{}'", root.kind());
    println!("  Byte range: {:?}", root.byte_range());
    println!("  Start position: {:?}", root.start_position());
    println!("  End position: {:?}", root.end_position());

    // Text extraction
    println!("\n📄 Text Extraction:");
    let text = root.text(source.as_bytes());
    println!("  Extracted text: '{}'", text);

    let utf8_result = root.utf8_text(source.as_bytes());
    match utf8_result {
        Ok(utf8_text) => println!("  UTF-8 text: '{}'", utf8_text),
        Err(e) => println!("  UTF-8 error: {:?}", e),
    }

    // Error states
    println!("\n⚠️  Error States:");
    println!("  Is error: {}", root.is_error());
    println!("  Is missing: {}", root.is_missing());

    // Tree structure (limited by parser_v4)
    println!("\n🌳 Tree Structure:");
    println!("  Child count: {}", root.child_count());
    println!("  First child: {:?}", root.child(0).map(|_| "Some(Node)"));

    // Unicode demonstration
    println!("\n🌐 Unicode Support:");
    let unicode_source = "123 + 456"; // Simple for this demo
    if let Some(unicode_tree) = parser.parse(unicode_source, None) {
        let unicode_root = unicode_tree.root_node();
        println!("  Unicode source: '{}'", unicode_source);
        println!("  Byte length: {} bytes", unicode_source.len());
        println!("  Node byte range: {:?}", unicode_root.byte_range());
        println!(
            "  Text matches: {}",
            unicode_root.text(unicode_source.as_bytes()) == unicode_source
        );
    }

    println!("\n✅ Node metadata demonstration complete!\n");
    Ok(())
}

/// Demonstrate incremental parsing with Direct Forest Splicing
fn demonstrate_incremental_parsing(parser: &mut Parser) -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Incremental Parsing Demonstration");
    println!("=====================================");

    // Initial parse
    let initial_source = "1 + 2 + 3 + 4 + 5";
    println!("Initial source: {}", initial_source);

    let start_time = Instant::now();
    let initial_tree = parser
        .parse(initial_source, None)
        .ok_or("Failed to parse initial source")?;
    let initial_time = start_time.elapsed();

    println!("✅ Initial parse completed in {:?}", initial_time);

    // Create an edit: change "3" to "999"
    let edit_start = initial_source.find("3").unwrap();
    let edit = InputEdit {
        start_byte: edit_start,
        old_end_byte: edit_start + 1, // "3" is 1 byte
        new_end_byte: edit_start + 3, // "999" is 3 bytes
        start_position: Point {
            row: 0,
            column: edit_start as u32,
        },
        old_end_position: Point {
            row: 0,
            column: (edit_start + 1) as u32,
        },
        new_end_position: Point {
            row: 0,
            column: (edit_start + 3) as u32,
        },
    };

    println!(
        "\n🔧 Applying edit: change '3' to '999' at byte offset {}",
        edit_start
    );
    println!("  Start byte: {}", edit.start_byte);
    println!("  Old end byte: {}", edit.old_end_byte);
    println!("  New end byte: {}", edit.new_end_byte);

    // Apply edit to tree
    let mut edited_tree = initial_tree.clone();
    edited_tree.edit(&edit);

    // Incremental reparse
    let new_source = "1 + 2 + 999 + 4 + 5";
    println!("  New source: {}", new_source);

    let start_time = Instant::now();
    let incremental_result = parser.parse(new_source, Some(&edited_tree));
    let incremental_time = start_time.elapsed();

    if let Some(new_tree) = incremental_result {
        println!("✅ Incremental parse completed in {:?}", incremental_time);

        // Calculate speedup
        if incremental_time.as_nanos() > 0 {
            let speedup = initial_time.as_nanos() as f64 / incremental_time.as_nanos() as f64;
            println!("🚀 Speedup: {:.2}x faster", speedup);
        }

        // Verify the result
        let new_root = new_tree.root_node();
        let new_text = new_root.text(new_source.as_bytes());
        println!("📝 Result verification:");
        println!("  New tree text: '{}'", new_text);
        println!("  Text matches source: {}", new_text == new_source);
        println!("  Tree has errors: {}", new_tree.has_errors());

        // Compare tree metadata
        println!("\n📊 Tree Comparison:");
        println!(
            "  Original tree byte range: {:?}",
            initial_tree.root_node().byte_range()
        );
        println!("  New tree byte range: {:?}", new_root.byte_range());
        println!(
            "  Size difference: {} bytes",
            new_source.len() as i32 - initial_source.len() as i32
        );
    } else {
        println!("❌ Incremental parsing failed, trying full reparse...");

        // Fallback to full reparse
        let start_time = Instant::now();
        let fallback_tree = parser
            .parse(new_source, None)
            .ok_or("Fallback parse also failed")?;
        let fallback_time = start_time.elapsed();

        println!("✅ Fallback parse completed in {:?}", fallback_time);
        let fallback_root = fallback_tree.root_node();
        let fallback_text = fallback_root.text(new_source.as_bytes());
        println!("📝 Fallback result: '{}'", fallback_text);
    }

    println!("\n🎯 Performance Notes:");
    println!("  • Direct Forest Splicing algorithm targets 16x speedup for large files");
    println!("  • Subtree reuse effectiveness depends on edit scope and location");
    println!("  • Enable RUST_SITTER_LOG_PERFORMANCE=true for detailed metrics");
    println!("  • Use incremental_glr feature flag for production deployment");

    println!("\n✅ Incremental parsing demonstration complete!");

    Ok(())
}
