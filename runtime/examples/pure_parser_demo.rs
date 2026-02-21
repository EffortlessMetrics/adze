// Demo of the pure-Rust parser implementation
use adze::pure_parser::{ParsedNode, Parser};

fn print_tree(node: &ParsedNode, depth: usize) {
    let indent = "  ".repeat(depth);
    println!(
        "{}Symbol {}: [{}, {}] - [{}, {}]",
        indent,
        node.symbol(),
        node.start_byte(),
        node.end_byte(),
        node.start_point().row,
        node.start_point().column
    );

    for child in node.children() {
        print_tree(child, depth + 1);
    }
}

fn main() {
    println!("Pure-Rust Tree-sitter Parser Demo");
    println!("==================================\n");

    // Create a parser
    let parser = Parser::new();
    println!("✓ Created parser");

    // In a real scenario, you would load a language from a generated module
    // For now, we'll demonstrate the API
    println!("\nNote: To use this demo with a real language:");
    println!("1. Generate a language using adze-tool");
    println!("2. Load the generated language module");
    println!("3. Pass it to parser.set_language()");

    // Example of what parsing would look like:
    /*
    let language = my_language::LANGUAGE;
    parser.set_language(&language).expect("Failed to set language");

    let source_code = "function hello() { return 42; }";
    let result = parser.parse_string(source_code);

    match result.root {
        Some(root) => {
            println!("\nParse tree:");
            print_tree(&root, 0);
        }
        None => {
            println!("\nParse failed!");
            for error in result.errors {
                println!("  Error at {}:{} - expected one of: {:?}, found: {}",
                    error.point.row, error.point.column,
                    error.expected, error.found
                );
            }
        }
    }
    */

    println!("\n✓ Pure-Rust parser is ready for use!");
    println!("\nFeatures:");
    println!("- No C dependencies");
    println!("- WASM-compatible");
    println!("- Tree-sitter ABI compatible");
    println!("- Supports timeouts and cancellation");
    println!("- Full error recovery");
}
