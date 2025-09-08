use rust_sitter_runtime::{Parser, Token, test_helpers::stub_language_with_tokens};

fn main() {
    // Create a stub language with test tokens to validate the tokenizer hookup
    // This will error on actual parsing since we don't have real parse tables yet
    let lang = stub_language_with_tokens(vec![
        Token {
            kind: 1,
            start: 0,
            end: 1,
        }, // First token 'a'
        Token {
            kind: 2,
            start: 1,
            end: 2,
        }, // Second token 'b'
        Token {
            kind: 0,
            start: 2,
            end: 2,
        }, // EOF token (kind 0)
    ]);

    let mut p = Parser::new();

    // Try to set language - this will fail in GLR mode if no parse table
    match p.set_language(lang) {
        Ok(_) => println!("✓ Language set successfully"),
        Err(e) => {
            println!(
                "✗ Failed to set language (expected in GLR mode without tables): {}",
                e
            );
            println!("  This is expected until parse tables are generated.");
            return;
        }
    }

    println!("Attempting to parse 'ab' with manual tokens...");
    match p.parse("ab", None) {
        Ok(tree) => {
            println!("✓ Parsed successfully!");
            println!("  Root node: {:?}", tree.root_node());
            println!("  Root kind: {}", tree.root_kind());
        }
        Err(e) => {
            println!("✗ Parse error (expected without real tables): {}", e);
        }
    }
}
