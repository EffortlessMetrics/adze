// Basic tests for ts-bridge without requiring actual tree-sitter libraries

use ts_bridge::schema::*;

#[test]
fn test_schema_serialization() {
    let data = ParseTableData {
        version: 1,
        ts_language_version: 15,
        symbol_count: 10,
        state_count: 20,
        token_count: 5,
        external_token_count: 0,
        eof_symbol: 0,
        start_symbol: 1,
        symbol_names: vec![
            "EOF".to_string(),
            "start".to_string(),
            "number".to_string(),
            "+".to_string(),
            "-".to_string(),
            "expression".to_string(),
        ],
        rules: vec![
            Rule { lhs: 1, rhs_len: 1, production_id: 0 },
            Rule { lhs: 5, rhs_len: 3, production_id: 1 },
        ],
        actions: vec![
            ActionCell {
                state: 0,
                terminal: 2,
                seq: vec![Action::Shift { state: 1, extra: false, rep: false }],
            },
            ActionCell {
                state: 1,
                terminal: 0,
                seq: vec![Action::Accept],
            },
        ],
        gotos: vec![
            GotoCell {
                state: 0,
                nonterminal: 5,
                next_state: 2,
            },
        ],
    };
    
    // Test that we can serialize to JSON
    let json = serde_json::to_string_pretty(&data).unwrap();
    assert!(json.contains("\"version\": 1"));
    assert!(json.contains("\"symbol_count\": 10"));
    
    // Test that we can deserialize back
    let data2: ParseTableData = serde_json::from_str(&json).unwrap();
    assert_eq!(data2.version, 1);
    assert_eq!(data2.symbol_count, 10);
    assert_eq!(data2.rules.len(), 2);
    assert_eq!(data2.actions.len(), 2);
    assert_eq!(data2.gotos.len(), 1);
}

#[test]
fn test_action_serialization() {
    let shift = Action::Shift { state: 42, extra: false, rep: true };
    let json = serde_json::to_string(&shift).unwrap();
    assert!(json.contains("\"k\":\"S\""));
    assert!(json.contains("\"state\":42"));
    
    let reduce = Action::Reduce { rule: 5, dyn_prec: -1, prod: 7 };
    let json = serde_json::to_string(&reduce).unwrap();
    assert!(json.contains("\"k\":\"R\""));
    assert!(json.contains("\"rule\":5"));
    
    let accept = Action::Accept;
    let json = serde_json::to_string(&accept).unwrap();
    assert!(json.contains("\"k\":\"A\""));
}