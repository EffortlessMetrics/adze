// Test the pure-Rust arithmetic parser
use adze::pure_parser::*;
use std::cmp::min;

fn main() {
    #[cfg(feature = "pure-rust")]
    {
        println!("Testing pure-Rust arithmetic parser...");

        // Include the generated parser module
        mod parser {
            include!(concat!(
                env!("OUT_DIR"),
                "/grammar_arithmetic/parser_arithmetic.rs"
            ));
        }

        use parser::*;

        // Define LexerState struct for testing
        #[repr(C)]
        struct LexerState {
            input: *const u8,
            input_len: usize,
            position: usize,
            point_row: u32,
            point_column: u32,
            result_symbol: u16,
            result_length: usize,
        }

        // Get the language
        let language = unsafe { &LANGUAGE };
        println!(
            "Language: {} symbols, {} states",
            language.symbol_count, language.state_count
        );

        // Create parser
        let mut parser = Parser::new();
        parser
            .set_language(language)
            .expect("Failed to set language");

        // Test cases - note: this grammar only supports - and *, not +
        let test_cases = vec!["42", "1 - 2", "3 * 4", "5 - 6", "1 - 2 * 3", "1 * 2 - 3"];

        for input in test_cases {
            println!("\nParsing: '{}'", input);

            // First, let's tokenize the input manually to see what's happening
            let bytes = input.as_bytes();
            println!("Input bytes: {:?}", bytes);

            // Test the lexer first
            let mut pos = 0;
            println!("Lexing tokens:");
            while pos < bytes.len() {
                let mut lex_state = LexerState {
                    input: bytes.as_ptr(),
                    input_len: bytes.len(),
                    position: pos,
                    point_row: 0,
                    point_column: pos as u32,
                    result_symbol: 0,
                    result_length: 0,
                };

                let lex_mode = TSLexState {
                    lex_state: 0,
                    external_lex_state: 0,
                };
                let success = unsafe {
                    (language.lex_fn.unwrap())(
                        &mut lex_state as *mut _ as *mut std::ffi::c_void,
                        lex_mode,
                    )
                };

                if success {
                    let token_str = std::str::from_utf8(&bytes[pos..pos + lex_state.result_length])
                        .unwrap_or("?");
                    println!(
                        "  Token at {}: '{}' -> symbol={}, length={}",
                        pos, token_str, lex_state.result_symbol, lex_state.result_length
                    );
                    pos += lex_state.result_length;
                } else {
                    println!(
                        "  No token at position {} (char={})",
                        pos, bytes[pos] as char
                    );
                    break;
                }
            }

            // Add debug logging
            println!("\nParse table info:");
            println!("  Symbol count: {}", language.symbol_count);
            println!("  State count: {}", language.state_count);
            println!("  Token count: {}", language.token_count);

            // Print compressed parse table info
            static PRINTED: std::sync::Once = std::sync::Once::new();
            PRINTED.call_once(|| {
                println!("\nCompressed parse table debug:");
                unsafe {
                    // Print small_parse_table_map offsets
                    let map = std::slice::from_raw_parts(
                        language.small_parse_table_map,
                        (language.state_count + 1) as usize,
                    );
                    println!("  State offsets:");
                    for i in 0..min(5, language.state_count as usize) {
                        println!("    State {}: offset={}", i, map[i]);
                    }

                    // Print compressed parse table entries
                    let table_size = map[language.state_count as usize] as usize * 2;
                    let table = std::slice::from_raw_parts(language.parse_table, table_size);
                    // Print symbol names
                    println!("\n  Symbol names:");
                    let symbol_names = std::slice::from_raw_parts(
                        language.symbol_names,
                        language.symbol_count as usize,
                    );
                    for i in 0..language.symbol_count as usize {
                        let name_ptr = symbol_names[i];
                        if !name_ptr.is_null() {
                            let name =
                                std::ffi::CStr::from_ptr(name_ptr as *const i8).to_string_lossy();
                            println!("    Symbol {}: {}", i, name);
                        } else {
                            println!("    Symbol {}: <null>", i);
                        }
                    }

                    println!("\n  Parse table entries (first 20):");
                    for i in 0..min(20, table_size / 2) {
                        let symbol = table[i * 2];
                        let action = table[i * 2 + 1];
                        print!("    [{:2}] symbol={:2}, action=", i, symbol);
                        if action == 0 {
                            println!("Error");
                        } else if action & 0x8000 != 0 {
                            println!("Reduce({})", action & 0x7FFF);
                        } else if action == 0x7FFF {
                            println!("Accept");
                        } else {
                            println!("Shift({})", action);
                        }
                    }
                }
            });

            let result = parser.parse_string(input);

            if let Some(root) = result.root {
                println!(
                    "Success! Root node: symbol={}, named={}",
                    root.symbol, root.is_named
                );
                // Print tree structure
                print_node(&root, 0);
            } else {
                println!("Failed to parse!");
            }

            if !result.errors.is_empty() {
                println!("Errors:");
                for err in &result.errors {
                    println!(
                        "  - At position {}: expected {:?}, found {}",
                        err.position, err.expected, err.found
                    );
                }
            }
        }
    }

    #[cfg(not(feature = "pure-rust"))]
    {
        println!("This example requires the 'pure-rust' feature");
        println!("Run with: cargo run --example test_arithmetic_pure --features pure-rust");
    }
}

#[cfg(feature = "pure-rust")]
fn print_node(node: &ParsedNode, indent: usize) {
    let indent_str = " ".repeat(indent);

    println!(
        "{}Node(symbol={}, named={}, range={}..{})",
        indent_str, node.symbol, node.is_named, node.start_byte, node.end_byte
    );

    for child in &node.children {
        print_node(child, indent + 2);
    }
}
