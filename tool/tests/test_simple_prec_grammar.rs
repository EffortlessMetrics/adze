use adze_tool::grammar_js::{GrammarJsConverter, GrammarJsParserV3};
use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_grammar_js};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_simple_precedence_grammar() {
    // Test a very simple grammar with just precedence
    let grammar_content = r#"
module.exports = grammar({
  name: 'simple_prec',
  
  rules: {
    expression: $ => choice(
      $.number,
      $.add
    ),
    
    add: $ => prec.left(1, seq($.expression, '+', $.expression)),
    
    number: $ => /\d+/
  }
})
"#;

    println!("Testing simple precedence grammar...");

    let mut parser = GrammarJsParserV3::new(grammar_content.to_string());
    match parser.parse() {
        Ok(grammar) => {
            println!("✓ Successfully parsed grammar!");
            println!("  Rules: {:?}", grammar.rules.keys().collect::<Vec<_>>());

            // Convert to IR
            let converter = GrammarJsConverter::new(grammar);
            match converter.convert() {
                Ok(_ir_grammar) => {
                    println!("✓ Successfully converted to IR!");

                    // Build the parser
                    let temp_dir = TempDir::new().unwrap();

                    // Write test grammar
                    let grammar_path = temp_dir.path().join("grammar.js");
                    fs::write(&grammar_path, grammar_content).unwrap();

                    unsafe {
                        std::env::set_var("CARGO_FEATURE_PURE_RUST", "1");
                    }

                    let options = BuildOptions {
                        out_dir: temp_dir.path().to_str().unwrap().to_string(),
                        emit_artifacts: true,
                        compress_tables: false, // Don't compress for easier debugging
                    };

                    match build_parser_from_grammar_js(&grammar_path, options) {
                        Ok(result) => {
                            println!("✓ Successfully built parser!");
                            println!("  Grammar: {}", result.grammar_name);

                            // Check NODE_TYPES
                            let node_types_path = temp_dir.path().join("NODE_TYPES.json");
                            if node_types_path.exists() {
                                println!("✓ NODE_TYPES.json exists");
                            }
                        }
                        Err(e) => {
                            println!("✗ Build failed: {}", e);
                            // Print more context
                            let error_str = format!("{:?}", e);
                            if error_str.contains("symbol") {
                                println!("  Likely symbol resolution issue");
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("✗ IR conversion failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Parse failed: {}", e);
        }
    }
}
