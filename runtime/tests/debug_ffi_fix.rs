#[cfg(feature = "json_example")]
#[test]
fn debug_unified_json_helper() {
    #[path = "support/unified_json_helper.rs"]
    mod unified_json_helper;
    
    // Test the function pointer retrieval first
    let raw_lang_fn = tree_sitter_json::LANGUAGE.into_raw();
    println!("Got raw function pointer: {:p}", raw_lang_fn as *const ());
    
    // Try to call it directly to get the language pointer
    let lang_ptr = unsafe { raw_lang_fn() };
    println!("Language pointer: {:p}", lang_ptr);
    
    if lang_ptr.is_null() {
        panic!("Language pointer is null - this indicates a linking issue");
    }
    
    println!("Basic function call works, now testing unified helper...");
    
    // Test if we can create the language without segfaulting
    match unified_json_helper::unified_json_language() {
        Ok(_lang) => {
            println!("Success: unified_json_language() worked");
        }
        Err(e) => {
            println!("Error creating language: {}", e);
            panic!("Failed to create unified JSON language: {}", e);
        }
    }
}