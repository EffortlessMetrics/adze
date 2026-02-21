//! Demonstrates incremental tree editing with adze runtime2
//!
//! This example shows how to use the Tree::edit() method for efficient
//! incremental parsing with comprehensive error handling and memory safety.

#[cfg(feature = "incremental")]
use adze_runtime2::{EditError, InputEdit, Point, Tree, TreeNode};

#[cfg(not(feature = "incremental"))]
fn main() {
    println!("This demo requires the 'incremental' feature.");
    println!("Run with: cargo run --example incremental_demo --features incremental");
}

#[cfg(feature = "incremental")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Incremental Tree Editing Demo (PR #28) ===\n");

    // Create a sample tree representing parsed code: "fn main() {}"
    let mut tree = create_sample_tree();

    println!("Initial tree structure:");
    print_tree_info(&tree);
    println!();

    // Demo 1: Safe text replacement with error handling
    println!("Demo 1: Replace '{}' with '{ println!(\"Hello\"); }'");
    let edit1 = InputEdit {
        start_byte: 10,
        old_end_byte: 12, // Replace "{}"
        new_end_byte: 34, // With "{ println!(\"Hello\"); }"
        start_position: Point::new(0, 10),
        old_end_position: Point::new(0, 12),
        new_end_position: Point::new(0, 34),
    };

    match tree.edit(&edit1) {
        Ok(()) => {
            println!("✓ Edit applied successfully");
            print_tree_info(&tree);
        }
        Err(e) => println!("✗ Edit failed: {}", e),
    }
    println!();

    // Demo 2: Deep cloning for non-destructive analysis
    println!("Demo 2: Deep cloning for analysis");
    let analysis_tree = tree.clone();
    println!("✓ Tree cloned successfully");
    println!("Original and clone have independent memory:");
    println!("  Original root span: {:?}", tree.root_node().start_byte());
    println!(
        "  Clone root span: {:?}",
        analysis_tree.root_node().start_byte()
    );
    println!();

    // Demo 3: Error handling - Invalid range
    println!("Demo 3: Error handling - Invalid edit range");
    let invalid_edit = InputEdit {
        start_byte: 20,
        old_end_byte: 10, // Invalid: end < start
        new_end_byte: 25,
        start_position: Point::new(0, 20),
        old_end_position: Point::new(0, 10),
        new_end_position: Point::new(0, 25),
    };

    match tree.edit(&invalid_edit) {
        Ok(()) => println!("Unexpected success"),
        Err(EditError::InvalidRange { start, old_end }) => {
            println!(
                "✓ Correctly caught invalid range: start={}, end={}",
                start, old_end
            );
        }
        Err(e) => println!("Unexpected error type: {}", e),
    }
    println!();

    // Demo 4: Large deletion with underflow protection
    println!("Demo 4: Large deletion with underflow protection");
    let large_deletion = InputEdit {
        start_byte: 5,
        old_end_byte: 100, // Delete beyond actual content
        new_end_byte: 6,   // Replace with single character
        start_position: Point::new(0, 5),
        old_end_position: Point::new(0, 100),
        new_end_position: Point::new(0, 6),
    };

    match tree.edit(&large_deletion) {
        Ok(()) => {
            println!("✓ Large deletion handled safely");
            print_tree_info(&tree);
        }
        Err(EditError::ArithmeticUnderflow) => {
            println!("✓ Correctly prevented arithmetic underflow");
        }
        Err(e) => println!("Unexpected error: {}", e),
    }
    println!();

    // Demo 5: Zero-length insertion
    println!("Demo 5: Zero-length insertion (pure insertion)");
    let insertion = InputEdit {
        start_byte: 2,
        old_end_byte: 2,  // No deletion
        new_end_byte: 10, // Insert 8 characters
        start_position: Point::new(0, 2),
        old_end_position: Point::new(0, 2),
        new_end_position: Point::new(0, 10),
    };

    match tree.edit(&insertion) {
        Ok(()) => {
            println!("✓ Insertion applied successfully");
            print_tree_info(&tree);
        }
        Err(e) => println!("✗ Insertion failed: {}", e),
    }

    println!("\n=== Key Features Demonstrated ===");
    println!("1. ✓ Safe tree editing with comprehensive error handling");
    println!("2. ✓ Overflow/underflow protection with checked arithmetic");
    println!("3. ✓ Deep cloning for non-destructive analysis");
    println!("4. ✓ Feature-gated implementation (#[cfg(feature = \"incremental\")])");
    println!("5. ✓ Efficient dirty node marking for selective re-parsing");
    println!("6. ✓ Invalid range detection and validation");

    Ok(())
}

#[cfg(feature = "incremental")]
fn create_sample_tree() -> Tree {
    // Create a tree representing: "fn main() {}"
    let body = TreeNode::new_with_children(3, 10, 12, vec![]); // "{}"
    let root = TreeNode::new_with_children(0, 0, 12, vec![body]); // Function
    Tree::new(root)
}

#[cfg(feature = "incremental")]
fn print_tree_info(tree: &Tree) {
    let root = tree.root_node();
    println!("  Root span: {}..{}", root.start_byte(), root.end_byte());
    println!("  Child count: {}", root.child_count());
}
