#![cfg(feature = "with-grammars")]  // compile only when enabled

use rand::{Rng, SeedableRng};
use ts_bridge::{extract, ffi::SafeLang};

type LangFn = unsafe extern "C" fn() -> *const ts_bridge::ffi::TSLanguage;

#[test]
fn parity_actions_and_gotos_json() {
    extern "C" { 
        fn tree_sitter_json() -> *const ts_bridge::ffi::TSLanguage; 
    }
    let lang_fn: LangFn = tree_sitter_json;

    let data = extract(lang_fn).expect("extract failed");
    let lang = SafeLang(unsafe { tree_sitter_json() });

    let (symc, stc, tokc, extc) = lang.counts();
    let term = tokc + extc;

    println!("Testing parity for grammar with:");
    println!("  {} symbols ({} terminals, {} nonterminals)", symc, term, symc - term);
    println!("  {} states", stc);
    println!("  {} rules", data.rules.len());

    let mut rng = rand::rngs::StdRng::seed_from_u64(0xC0FFEE);

    // Actions parity: 500 random samples
    let mut tested_actions = 0;
    for _ in 0..500 {
        let s = rng.gen_range(0..stc);
        let a = rng.gen_range(0..term);
        
        let (oracle_seq, _) = lang.table_actions(s, a);      // C-oracle via shim
        let ours = data.actions.get(&s).and_then(|r| r.get(&a)).cloned().unwrap_or_default();
        
        if !oracle_seq.is_empty() {
            tested_actions += 1;
            assert_eq!(
                oracle_seq, ours, 
                "action mismatch at state {}, terminal {}", 
                s, a
            );
        }
    }
    
    println!("Tested {} non-empty action cells (out of 500 samples)", tested_actions);

    // Gotos parity: 200 random samples
    let mut tested_gotos = 0;
    for _ in 0..200 {
        let s = rng.gen_range(0..stc);
        let A = rng.gen_range(term..symc);
        
        let oracle = lang.next_state(s, A);
        let ours = data.gotos.get(&s).and_then(|r| r.get(&A)).copied().unwrap_or(0);
        
        if oracle != 0 {
            tested_gotos += 1;
            assert_eq!(
                oracle, ours, 
                "goto mismatch at state {}, nonterminal {}", 
                s, A
            );
        }
    }
    
    println!("Tested {} non-empty goto cells (out of 200 samples)", tested_gotos);
}