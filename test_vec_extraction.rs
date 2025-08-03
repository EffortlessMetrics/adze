fn main() {
    use rust_sitter::pure_parser::{Parser, ParsedNode};
    use rust_sitter::Extract;
    
    // Create a simple parse tree to test Vec extraction
    let vec_contents_node = ParsedNode {
        symbol: 28, // Module_body_vec_contents
        kind: |_| "Module_body_vec_contents",
        start_byte: 0,
        end_byte: 2,
        is_extra: false,
        children: vec![
            ParsedNode {
                symbol: 17, // Statement
                kind: |_| "Statement",  
                start_byte: 0,
                end_byte: 2,
                is_extra: false,
                children: vec![],
                has_error: false,
            }
        ],
        has_error: false,
    };
    
    // Test Vec<T>::extract directly
    let source = b"42";
    let result = <Vec<()> as Extract<Vec<()>>>::extract(Some(&vec_contents_node), source, 0, None);
    println!("Vec extraction result length: {}", result.len());
}