#[cfg(feature = "with-grammars")]
#[test]
fn debug_extraction_state_0() {
    use tree_sitter_json::LANGUAGE;
    use ts_bridge::ffi::{TSLanguage, SafeLang};
    
    // Create a wrapper for the language
    type LangFn = unsafe extern "C" fn() -> *const TSLanguage;
    let lang_fn: LangFn = unsafe {
        std::mem::transmute(LANGUAGE.into_raw())
    };
    
    SafeLang::assert_abi();
    let lang = SafeLang(unsafe { lang_fn() });
    
    println!("=== Debugging state 0 actions ===");
    
    // Get symbol counts
    let (symc, stc, tokc, extc) = lang.counts();
    println!("Counts: symbols={}, states={}, tokens={}, external={}", symc, stc, tokc, extc);
    
    // Print first 20 symbol names
    println!("\nSymbol names:");
    for i in 0..20.min(symc) {
        let name = lang.symbol_name(i);
        println!("  {}: {}", i, name);
    }
    
    // Check what's at state 0 for symbol 1 (should be '{')
    println!("\n=== State 0 actions ===");
    for sym in 0..15 {
        if let Some((hdr, idx)) = lang.entry(0, sym) {
            println!("State 0, symbol {} ({}): {} actions at index {}", 
                sym, lang.symbol_name(sym), hdr.count, idx);
            
            // Unpack the actions to see what they are
            let mut buf = vec![ts_bridge::ffi::TsbAction {
                kind: ts_bridge::ffi::TsbActionKind::Accept,
                state: 0,
                lhs: 0,
                rhs_len: 0,
                dynamic_precedence: 0,
                production_id: 0,
                extra: false,
                repetition: false,
            }; hdr.count as usize];
            
            let n = lang.unpack(idx, hdr.count, &mut buf);
            for action in &buf[..n] {
                println!("  -> {:?}", action.kind);
            }
        }
    }
    
    // Explicitly check state 0, symbol 1
    // Also check state 1 which Tree-sitter usually uses as entry
    println!("\n=== State 1 actions ===");
    for sym in 0..15 {
        if let Some((hdr, idx)) = lang.entry(1, sym) {
            let name = lang.symbol_name(sym);
            println!("State 1, symbol {} ({}): {} actions at index {}", 
                sym, name, hdr.count, idx);
        }
    }
    
    println!("\n=== Check State 1, Symbol 1 ('{{') - the real initial? ===");
    if let Some((hdr, idx)) = lang.entry(1, 1) {
        println!("Found {} actions at index {}", hdr.count, idx);
        
        let mut buf = vec![ts_bridge::ffi::TsbAction {
            kind: ts_bridge::ffi::TsbActionKind::Accept,
            state: 0,
            lhs: 0,
            rhs_len: 0,
            dynamic_precedence: 0,
            production_id: 0,
            extra: false,
            repetition: false,
        }; hdr.count as usize];
        
        let n = lang.unpack(idx, hdr.count, &mut buf);
        for action in &buf[..n] {
            println!("  Action: {:?} -> state {}", action.kind, action.state);
        }
    }
    
    // Check the language metadata to find the real initial state
    println!("\n=== Checking language metadata ===");
    extern "C" {
        fn ts_language_version(lang: *const TSLanguage) -> u32;
    }
    let version = unsafe { ts_language_version(lang.0) };
    println!("Language version: {}", version);
    
    // State 0 is typically used for error recovery in Tree-sitter
    // The real initial state is usually 1
    println!("\nConclusion: Tree-sitter JSON uses state 1 as the initial parse state");
    println!("State 0 is reserved for error recovery (all Recover actions)");
    
    // Check all actions available at state 16
    println!("\n=== All State 16 actions ===");
    for sym in 0..25 {
        if let Some((hdr, idx)) = lang.entry(16, sym) {
            let name = lang.symbol_name(sym);
            println!("State 16, symbol {} ({}): {} actions", sym, name, hdr.count);
            
            let mut buf = vec![ts_bridge::ffi::TsbAction {
                kind: ts_bridge::ffi::TsbActionKind::Accept,
                state: 0,
                lhs: 0,
                rhs_len: 0,
                dynamic_precedence: 0,
                production_id: 0,
                extra: false,
                repetition: false,
            }; hdr.count as usize];
            
            let n = lang.unpack(idx, hdr.count, &mut buf);
            for action in &buf[..n] {
                match action.kind {
                    ts_bridge::ffi::TsbActionKind::Shift => {
                        println!("  -> Shift to state {}", action.state);
                    }
                    ts_bridge::ffi::TsbActionKind::Reduce => {
                        println!("  -> Reduce: lhs={} ({}), rhs_len={}", 
                            action.lhs, lang.symbol_name(action.lhs as u32), action.rhs_len);
                    }
                    _ => {
                        println!("  -> {:?}", action.kind);
                    }
                }
            }
        }
    }
    
    // Now check what state 16 does on '}'
    println!("\n=== Check State 16, Symbol 3 ('}}') ===");
    if let Some((hdr, idx)) = lang.entry(16, 3) {
        println!("Found {} actions at index {}", hdr.count, idx);
        
        let mut buf = vec![ts_bridge::ffi::TsbAction {
            kind: ts_bridge::ffi::TsbActionKind::Accept,
            state: 0,
            lhs: 0,
            rhs_len: 0,
            dynamic_precedence: 0,
            production_id: 0,
            extra: false,
            repetition: false,
        }; hdr.count as usize];
        
        let n = lang.unpack(idx, hdr.count, &mut buf);
        for action in &buf[..n] {
            println!("  Action: {:?}", action.kind);
            if action.kind == ts_bridge::ffi::TsbActionKind::Shift {
                println!("    -> shift to state {}", action.state);
            } else if action.kind == ts_bridge::ffi::TsbActionKind::Reduce {
                println!("    -> reduce: lhs={}, rhs_len={}", action.lhs, action.rhs_len);
            }
        }
    } else {
        println!("NO ACTIONS FOUND for state 16 on '}}'!");
    }
}