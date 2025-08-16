#![cfg(all(feature = "ts-ffi-raw", feature = "with-grammars"))]

use ts_bridge::ffi;

#[test]
fn test_raw_ffi_linking() {
    // This test verifies that we can successfully link and call
    // the internal Tree-sitter functions through our shim
    unsafe {
        // Use the Rust crate's exported language - into_raw() returns a function pointer
        // that we need to call to get the actual language pointer
        let lang_fn = tree_sitter_json::LANGUAGE.into_raw();
        let lang = lang_fn() as *const ffi::TSLanguage;
        assert!(!lang.is_null());

        // Try to get some basic counts
        let mut symc = 0u32;
        let mut stc = 0u32;
        let mut tokc = 0u32;
        let mut extc = 0u32;

        ffi::tsb_counts(lang, &mut symc, &mut stc, &mut tokc, &mut extc);

        // JSON grammar should have symbols and states
        assert!(symc > 0, "Expected symbols, got {}", symc);
        assert!(stc > 0, "Expected states, got {}", stc);

        // Try to get symbol names for a few symbols
        for i in 0..5.min(symc) {
            let name_ptr = ffi::tsb_symbol_name(lang, i);
            assert!(!name_ptr.is_null());
            let name = std::ffi::CStr::from_ptr(name_ptr).to_str().unwrap();
            println!("Symbol {}: {}", i, name);
        }

        // Test table entry access (state 0, symbol 1)
        let mut hdr = ffi::TsbEntryHeader {
            count: 0,
            reusable: false,
            action_index: 0,
        };
        let entry = ffi::tsb_table_entry(lang, 0, 1, &mut hdr);
        println!(
            "Entry for state 0, symbol 1: index={}, count={}",
            entry, hdr.count
        );

        // Test next state function
        let next = ffi::tsb_next_state(lang, 0, 1);
        println!("Next state from 0 on symbol 1: {}", next);
    }
}
