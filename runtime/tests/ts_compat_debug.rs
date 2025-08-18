#![cfg(all(feature = "ts-compat", feature = "pure-rust"))]

use rust_sitter::ts_compat::Parser;

#[test]
fn debug_parse_flow() {
    let lang = rust_sitter_example::ts_langs::arithmetic();
    let mut parser = Parser::new();
    parser.set_language(lang.clone());

    println!("=== Language Info ===");
    println!("Start symbol: {:?}", lang.table.start_symbol);
    println!("EOF symbol: {:?}", lang.table.eof_symbol);
    println!("Token count: {}", lang.table.token_count);
    println!("State count: {}", lang.table.state_count);

    println!("\n=== Rules ===");
    for (i, rule) in lang.table.rules.iter().enumerate() {
        let lhs_name = lang
            .grammar
            .rule_names
            .get(&rule.lhs)
            .map(|s| s.as_str())
            .unwrap_or("?");
        println!(
            "Rule {}: {} (id {:?}) -> {} symbols",
            i, lhs_name, rule.lhs, rule.rhs_len
        );
    }

    println!("\n=== Action Table State 0 ===");
    for (sym_id, actions) in lang.table.action_table[0].iter().enumerate() {
        if !actions.is_empty() {
            let sym_name = lang
                .grammar
                .rule_names
                .get(&rust_sitter::rust_sitter_ir::SymbolId(sym_id as u16))
                .map(|s| s.as_str())
                .unwrap_or("?");
            println!("  Symbol {} ({}) -> {:?}", sym_id, sym_name, actions);
        }
    }

    println!("\n=== Action Table State 2 (after number) ===");
    for (sym_id, actions) in lang.table.action_table[2].iter().enumerate() {
        if !actions.is_empty() {
            let sym_name = lang
                .grammar
                .rule_names
                .get(&rust_sitter::rust_sitter_ir::SymbolId(sym_id as u16))
                .map(|s| s.as_str())
                .unwrap_or("?");
            println!("  Symbol {} ({}) -> {:?}", sym_id, sym_name, actions);
        }
    }

    println!("\n=== Action Table State 3 (after Expression) ===");
    for (sym_id, actions) in lang.table.action_table[3].iter().enumerate() {
        if !actions.is_empty() {
            let sym_name = lang
                .grammar
                .rule_names
                .get(&rust_sitter::rust_sitter_ir::SymbolId(sym_id as u16))
                .map(|s| s.as_str())
                .unwrap_or("?");
            println!("  Symbol {} ({}) -> {:?}", sym_id, sym_name, actions);
        }
    }

    println!("\n=== Goto Table ===");
    for state in 0..lang.table.state_count {
        for sym in 0..lang.table.symbol_count {
            let goto_state = lang.table.goto_table[state][sym];
            if goto_state.0 != 0 {
                let sym_name = lang
                    .grammar
                    .rule_names
                    .get(&rust_sitter::rust_sitter_ir::SymbolId(sym as u16))
                    .map(|s| s.as_str())
                    .unwrap_or("?");
                println!(
                    "  State {} + Symbol {} ({}) -> State {}",
                    state, sym, sym_name, goto_state.0
                );
            }
        }
    }

    println!("\n=== Attempting Parse ===");
    let tree = parser.parse("1", None);

    if let Some(tree) = tree {
        println!("SUCCESS! Root kind: {}", tree.root_kind());

        // Also test "1+2+3"
        println!("\n=== Testing 1+2+3 ===");
        let tree2 = parser.parse("1+2+3", None);
        if let Some(tree2) = tree2 {
            println!("SUCCESS! Root kind: {}", tree2.root_kind());
        } else {
            println!("FAILED for 1+2+3");
        }
    } else {
        println!("FAILED - returned None");
    }
}
