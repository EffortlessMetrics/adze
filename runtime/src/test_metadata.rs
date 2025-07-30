#[cfg(test)]
mod tests {
    use crate::pure_parser::*;
    
    #[test]
    fn test_whitespace_metadata() {
        // Get the arithmetic parser
        unsafe extern "C" {
            fn tree_sitter_arithmetic() -> *const TSLanguage;
        }
        
        let language = unsafe { &*tree_sitter_arithmetic() };
        
        println!("Checking metadata for arithmetic grammar:");
        println!("Symbol count: {}", language.symbol_count);
        
        // Check metadata for all symbols
        unsafe {
            let metadata_ptr = language.symbol_metadata;
            if metadata_ptr.is_null() {
                panic!("Metadata pointer is NULL!");
            }
            
            for i in 0..language.symbol_count {
                let metadata = *metadata_ptr.add(i as usize);
                let is_hidden = (metadata & 0x04) != 0;
                println!("Symbol {}: metadata = {:#x}, is_hidden = {}", i, metadata, is_hidden);
            }
            
            // Symbol 3 should be whitespace and marked as hidden
            let whitespace_metadata = *metadata_ptr.add(3);
            assert_eq!(whitespace_metadata & 0x04, 0x04, "Symbol 3 (whitespace) should have HIDDEN flag set");
            
            // Symbol 4 should be number and NOT marked as hidden
            let number_metadata = *metadata_ptr.add(4);
            assert_eq!(number_metadata & 0x04, 0x00, "Symbol 4 (number) should NOT have HIDDEN flag set");
        }
        
        println!("✓ Whitespace metadata test passed!");
    }
}