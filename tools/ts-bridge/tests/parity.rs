#![cfg(all(feature = "with-grammars", not(feature = "stub-ts")))]

use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use ts_bridge::{
    extract,
    ffi::{SafeLang, TSLanguage, TsbAction, TsbActionKind},
    schema::Action,
};

type LangFn = unsafe extern "C" fn() -> *const TSLanguage;

// Use the LANGUAGE constant from tree-sitter-json crate
fn get_json_language() -> *const TSLanguage {
    unsafe { tree_sitter_json::LANGUAGE.into_raw()() as *const TSLanguage }
}

fn tree_sitter_json_fn() -> LangFn {
    unsafe { std::mem::transmute(tree_sitter_json::LANGUAGE.into_raw()) }
}

#[test]
fn parity_actions_and_gotos_json() {
    // 1) Load language + extract tables
    let lang_fn = tree_sitter_json_fn();

    let data = extract(lang_fn).expect("extract() failed");
    let lang = SafeLang(get_json_language());

    let (symc, stc, tokc, extc) = lang.counts();
    let term_boundary = tokc + extc;

    // Sanity: counts agree with extractor
    assert_eq!(data.symbol_count as u32, symc, "symbol_count mismatch");
    assert_eq!(data.state_count as u32, stc, "state_count mismatch");

    // 2) Index extracted tables for O(1) lookup
    //    actions: key = (state, symbol) -> Vec<Action>
    let mut act_map: HashMap<(u16, u16), &Vec<Action>> = HashMap::new();
    for cell in &data.actions {
        act_map.insert((cell.state, cell.symbol), &cell.actions);
    }
    //    gotos: key = (state, symbol) -> next_state (if any)
    let mut goto_map: HashMap<(u16, u16), u16> = HashMap::new();
    for cell in &data.gotos {
        if let Some(next) = cell.next_state {
            goto_map.insert((cell.state, cell.symbol), next);
        }
    }

    // 3) Sample cells (keeps test time reasonable)
    let mut rng = rand::rngs::StdRng::seed_from_u64(0x0123456789ABCDEF);
    let samples = 5_000.min((stc * symc) as usize);

    for _ in 0..samples {
        let s = rng.gen_range(0..stc) as u16;
        let x = rng.gen_range(0..symc) as u16;

        if (x as u32) < term_boundary {
            // Terminal cell: compare actions
            if let Some((hdr, idx)) = lang.entry(s as u32, x as u32) {
                let mut buf = vec![TsbAction::default(); hdr.count as usize];
                let n = lang.unpack(idx, hdr.count, &mut buf);
                assert_eq!(n, hdr.count as usize, "unpack count mismatch at ({s},{x})");

                let got = act_map.get(&(s, x)).map(|v| v.as_slice()).unwrap_or(&[]);
                assert_eq!(
                    got.len(),
                    n,
                    "action count diff at ({s},{x}): expected {}, got {}",
                    n,
                    got.len()
                );

                for (i, a) in buf.iter().enumerate() {
                    let ea = &got[i];
                    match (a.kind, ea) {
                        (TsbActionKind::Shift, Action::Shift { state, .. }) => {
                            assert_eq!(*state, a.state, "shift state diff at ({s},{x})");
                        }
                        (TsbActionKind::Reduce, Action::Reduce { rule, dyn_prec }) => {
                            assert_eq!(
                                *dyn_prec, a.dynamic_precedence,
                                "dyn_prec diff at ({s},{x})"
                            );
                            // We can't compare rule numbers directly (they're assigned),
                            // but we can check (lhs, rhs_len, production_id) matches the rule metadata.
                            let r = &data.rules[*rule as usize];
                            assert_eq!(r.lhs, a.lhs, "lhs diff at ({s},{x})");
                            assert_eq!(r.rhs_len, a.rhs_len, "rhs_len diff at ({s},{x})");
                            assert_eq!(
                                r.production_id, a.production_id,
                                "prod_id diff at ({s},{x})"
                            );
                        }
                        (TsbActionKind::Accept, Action::Accept) => {}
                        (TsbActionKind::Recover, Action::Recover) => {}
                        // All other cases are mismatches
                        _ => panic!(
                            "action kind mismatch at ({s},{x}): runtime {:?}, extracted {:?}",
                            a.kind, ea
                        ),
                    }
                }
            } else {
                // No actions expected
                let got = act_map.get(&(s, x)).map(|v| v.len()).unwrap_or(0);
                assert_eq!(got, 0, "unexpected actions at ({s},{x})");
            }
        } else {
            // Nonterminal cell: compare goto (next state)
            let next_rt = lang.next_state(s as u32, x as u32);
            match goto_map.get(&(s, x)).copied() {
                Some(nxt) => assert_eq!(
                    nxt as u32, next_rt,
                    "goto diff at ({s},{x}): expected {}, got {}",
                    nxt, next_rt
                ),
                None => assert_eq!(
                    next_rt, 0,
                    "unexpected goto at ({s},{x}): runtime returned {}",
                    next_rt
                ),
            }
        }
    }

    // 4) Start symbol check
    let rt_start = lang.detect_start_symbol();
    assert_eq!(
        data.start_symbol as u32, rt_start,
        "start symbol mismatch: extracted {}, runtime {}",
        data.start_symbol, rt_start
    );

    println!("✅ Parity test passed for JSON grammar!");
    println!("   - {} states, {} symbols", stc, symc);
    println!("   - {} sampled cells", samples);
    println!(
        "   - {} action cells, {} goto cells",
        data.actions.len(),
        data.gotos.len()
    );
    println!("   - {} rules", data.rules.len());
}

#[test]
fn parity_metadata_json() {
    // Test symbol names
    let lang_fn = tree_sitter_json_fn();
    let data = extract(lang_fn).expect("extract() failed");
    let lang = SafeLang(get_json_language());

    // Verify symbol names
    for (i, sym) in data.symbols.iter().enumerate() {
        let name = lang.symbol_name(i as u32);
        assert_eq!(
            sym.name, name,
            "symbol name mismatch at {}: extracted '{}', runtime '{}'",
            i, sym.name, name
        );
    }

    println!("✅ Metadata parity test passed for JSON grammar!");
}
