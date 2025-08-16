#[cfg(feature = "with-grammars")]
#[test]
fn test_json_empty_object_path() {
    use tree_sitter_json::LANGUAGE;
    use ts_bridge::ffi::TSLanguage;

    // Create a wrapper for the language
    type LangFn = unsafe extern "C" fn() -> *const TSLanguage;
    let lang_fn: LangFn = unsafe { std::mem::transmute(LANGUAGE.into_raw()) };

    // Extract the parse tables
    let data = ts_bridge::extract(lang_fn).expect("Failed to extract");

    // Debug: Look for the empty object path
    println!("Looking for empty object parse path...");

    // Check state 16 (after {)
    let state_16_actions: Vec<_> = data.actions.iter().filter(|a| a.state == 16).collect();

    println!("State 16 actions:");
    for action_cell in &state_16_actions {
        println!("  Symbol {}: {:?}", action_cell.symbol, action_cell.actions);
    }

    // Look for rules that produce object with 2 children (likely { })
    let object_rules: Vec<_> = data
        .rules
        .iter()
        .filter(|r| r.lhs == 17) // object symbol
        .collect();

    println!("\nObject production rules:");
    for (i, rule) in object_rules.iter().enumerate() {
        println!("  Rule {}: object -> {} children", i, rule.rhs_len);
    }

    // The issue: Tree-sitter uses a special internal state or error recovery
    // for empty objects. Let's check if there's a hidden epsilon production
    // or if it requires the pair list to be optional.

    // Look for any reduce action that could create an empty pair list
    let empty_reduces: Vec<_> = data
        .actions
        .iter()
        .filter(|a| {
            a.actions.iter().any(|act| {
                matches!(act, ts_bridge::schema::Action::Reduce { rule, .. } if {
                    data.rules.get(*rule as usize)
                        .map_or(false, |r| r.rhs_len == 0)
                })
            })
        })
        .collect();

    println!("\nStates with empty reductions:");
    for action_cell in &empty_reduces {
        println!(
            "  State {}, Symbol {}: {:?}",
            action_cell.state, action_cell.symbol, action_cell.actions
        );
    }

    // This test is for debugging - it always passes
    assert!(true);
}
