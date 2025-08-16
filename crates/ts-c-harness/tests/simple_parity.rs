use tree_sitter_json as ts_json;

#[test]
fn sanity_runtime_accepts_empty_object() {
    // Prove the official runtime parses "{}" without errors
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&ts_json::LANGUAGE.into()).unwrap();

    let tree = parser.parse("{}", None).unwrap();
    let root = tree.root_node();

    println!("Parse tree for '{{}}':");
    println!("  Root: {} (error: {})", root.kind(), root.has_error());

    // Walk children to see structure
    let mut cursor = root.walk();
    if cursor.goto_first_child() {
        loop {
            let node = cursor.node();
            println!("    Child: {} (error: {})", node.kind(), node.has_error());

            // Go deeper for object node
            if node.kind() == "object" {
                let mut obj_cursor = node.walk();
                if obj_cursor.goto_first_child() {
                    loop {
                        let child = obj_cursor.node();
                        println!(
                            "      Object child: {} (error: {})",
                            child.kind(),
                            child.has_error()
                        );
                        if !obj_cursor.goto_next_sibling() {
                            break;
                        }
                    }
                }
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    // Key assertion: does the official runtime report an error?
    assert!(
        !root.has_error(),
        "Official runtime reports an error on {{}}"
    );
}

#[test]
fn empty_array_parses_without_error() {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&ts_json::LANGUAGE.into()).unwrap();

    let tree = parser.parse("[]", None).unwrap();
    let root = tree.root_node();

    println!("\nParse tree for '[]':");
    println!("  Root: {} (error: {})", root.kind(), root.has_error());

    assert!(!root.has_error(), "Official runtime reports an error on []");
}

#[test]
fn object_with_one_pair_parses() {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&ts_json::LANGUAGE.into()).unwrap();

    let tree = parser.parse(r#"{"key": "value"}"#, None).unwrap();
    let root = tree.root_node();

    println!("\nParse tree for single pair object:");
    println!("  Root: {} (error: {})", root.kind(), root.has_error());

    assert!(
        !root.has_error(),
        "Official runtime reports an error on single pair"
    );
}
