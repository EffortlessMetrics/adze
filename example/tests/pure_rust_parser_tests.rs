#[cfg(feature = "pure-rust")]
#[cfg(test)]
mod tests {
    // Access the generated parser directly from the build output
    include!(concat!(
        env!("OUT_DIR"),
        "/grammar_arithmetic/parser_arithmetic.rs"
    ));

    #[test]
    fn test_state0_has_token_actions() {
        // Use the LANGUAGE static directly since it's pub
        let lang = &LANGUAGE;

        // Check that we have compressed tables only (no large states)
        let large_state_count = lang.large_state_count as usize;
        assert_eq!(large_state_count, 0, "Expected all states to be compressed");

        // Get state 0's row from the small parse table
        // SAFETY: These are generated static arrays with known bounds
        unsafe {
            let state_0_start = *lang.small_parse_table_map.add(0) as usize;
            let state_0_end = *lang.small_parse_table_map.add(1) as usize;

            // Check that state 0 has at least one token action
            let mut found_token_action = false;
            let mut offset = state_0_start;

            while offset + 1 < state_0_end {
                let symbol_index = *lang.small_parse_table.add(offset);
                let action = *lang.small_parse_table.add(offset + 1);
                offset += 2;

                // Check if this is a token (symbol index < token_count)
                if symbol_index < lang.token_count as u16 {
                    found_token_action = true;
                    println!(
                        "State 0 has action for token symbol {} -> action {}",
                        symbol_index, action
                    );
                    break;
                }
            }

            assert!(
                found_token_action,
                "State 0 must have at least one shift action for a token to begin parsing. \
                 Currently it only has GOTO entries for non-terminals, which prevents any input from being accepted."
            );
        }
    }
}
