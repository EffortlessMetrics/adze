//! Comprehensive integration tests for the adze-tablegen crate.
//!
//! Tests cover:
//! 1. Full code generation pipeline: grammar → tables → Language struct code
//! 2. Compression → decompression roundtrip
//! 3. NODE_TYPES JSON correctness for various grammars
//! 4. ABI version compatibility checks
//! 5. Symbol metadata correctness
//! 6. FFI struct layout verification
//! 7. Edge cases: empty grammar, single-rule grammar, many-state grammar

use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use adze_ir::*;
use adze_tablegen::compression::{
    compress_action_table, compress_goto_table, decompress_action, decompress_goto,
};
use adze_tablegen::{
    AbiLanguageBuilder, NodeTypesGenerator, StaticLanguageGenerator, TableCompressor,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a simple parentheses grammar: expr -> '(' expr ')' | ε
fn make_parens_grammar() -> Grammar {
    let mut grammar = Grammar::new("parens".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "(".to_string(),
            pattern: TokenPattern::String("(".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: ")".to_string(),
            pattern: TokenPattern::String(")".to_string()),
            fragile: false,
        },
    );

    let expr_id = SymbolId(3);
    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(SymbolId(2)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    grammar.rule_names.insert(expr_id, "expr".to_string());
    grammar
}

/// Build a classic arithmetic grammar:
///   expr -> expr '+' term | term
///   term -> term '*' factor | factor
///   factor -> number
fn make_arithmetic_grammar() -> Grammar {
    let mut grammar = Grammar::new("arithmetic".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "+".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(3),
        Token {
            name: "*".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );

    let expr_id = SymbolId(4);
    let term_id = SymbolId(5);
    let factor_id = SymbolId(6);

    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(term_id),
        ],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::NonTerminal(term_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    grammar.add_rule(Rule {
        lhs: term_id,
        rhs: vec![
            Symbol::NonTerminal(term_id),
            Symbol::Terminal(SymbolId(3)),
            Symbol::NonTerminal(factor_id),
        ],
        precedence: Some(PrecedenceKind::Static(2)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(2),
    });
    grammar.add_rule(Rule {
        lhs: term_id,
        rhs: vec![Symbol::NonTerminal(factor_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(3),
    });
    grammar.add_rule(Rule {
        lhs: factor_id,
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(4),
    });
    grammar.rule_names.insert(expr_id, "expression".to_string());
    grammar.rule_names.insert(term_id, "term".to_string());
    grammar.rule_names.insert(factor_id, "factor".to_string());
    grammar
}

/// Build a single-rule grammar: root -> token
fn make_single_rule_grammar() -> Grammar {
    let mut grammar = Grammar::new("single".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "word".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );
    let root_id = SymbolId(2);
    grammar.add_rule(Rule {
        lhs: root_id,
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.rule_names.insert(root_id, "root".to_string());
    grammar
}

/// Build a grammar with field annotations: assignment -> name:id '=' value:id
fn make_fields_grammar() -> Grammar {
    let mut grammar = Grammar::new("fields".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "=".to_string(),
            pattern: TokenPattern::String("=".to_string()),
            fragile: false,
        },
    );
    grammar.fields.insert(FieldId(0), "name".to_string());
    grammar.fields.insert(FieldId(1), "value".to_string());
    let assign_id = SymbolId(3);
    grammar.add_rule(Rule {
        lhs: assign_id,
        rhs: vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(2)),
            Symbol::Terminal(SymbolId(1)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0), (FieldId(1), 2)],
        production_id: ProductionId(0),
    });
    grammar
        .rule_names
        .insert(assign_id, "assignment".to_string());
    grammar
}

/// Build grammar → FIRST/FOLLOW → LR(1) automaton → ParseTable
fn build_parse_table(grammar: &Grammar) -> adze_glr_core::ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW computation");
    build_lr1_automaton(grammar, &ff).expect("LR(1) automaton construction")
}

// ===========================================================================
// 1. Full code generation pipeline
// ===========================================================================

#[test]
fn pipeline_parens_grammar_generates_language_code() {
    let grammar = make_parens_grammar();
    let table = build_parse_table(&grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);

    let code = builder.generate().to_string();

    assert!(code.contains("TSLanguage"), "must emit TSLanguage struct");
    assert!(code.contains("LANGUAGE_VERSION"), "must embed ABI version");
    assert!(
        code.contains("tree_sitter_parens"),
        "function name derives from grammar name"
    );
    assert!(code.contains("PARSE_TABLE"), "must emit parse table data");
    assert!(
        code.contains("symbol_count"),
        "must include symbol_count field"
    );
}

#[test]
fn pipeline_arithmetic_grammar_symbol_counts() {
    let grammar = make_arithmetic_grammar();
    let table = build_parse_table(&grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);

    let code = builder.generate().to_string();

    // 7 symbols: EOF(0), number(1), +(2), *(3), expression(4), term(5), factor(6)
    assert!(
        code.contains("symbol_count : 7"),
        "arithmetic grammar should have 7 symbols, got code: {}",
        &code[..code.len().min(500)]
    );
    // token_count = grammar.tokens.len() + 1 (EOF)
    assert!(
        code.contains("token_count : 4"),
        "should have 4 tokens (3 user + EOF)"
    );
}

#[test]
fn pipeline_single_rule_grammar_generates_valid_code() {
    let grammar = make_single_rule_grammar();
    let table = build_parse_table(&grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);

    let code = builder.generate().to_string();
    assert!(code.contains("TSLanguage"));
    assert!(code.contains("tree_sitter_single"));
}

#[test]
fn pipeline_static_language_generator_compress_and_generate() {
    let grammar = make_parens_grammar();
    let table = build_parse_table(&grammar);

    let mut slgen = StaticLanguageGenerator::new(grammar, table);
    assert!(
        slgen.compress_tables().is_ok(),
        "compression should succeed"
    );

    let code = slgen.generate_language_code().to_string();
    assert!(code.contains("SYMBOL_NAMES"));
    assert!(code.contains("LANGUAGE_VERSION"));
}

#[test]
fn pipeline_fields_grammar_emits_field_metadata() {
    let grammar = make_fields_grammar();
    let table = build_parse_table(&grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);

    let code = builder.generate().to_string();
    assert!(
        code.contains("field_count : 2"),
        "should have 2 field definitions"
    );
    assert!(code.contains("FIELD_NAME_"));
}

// ===========================================================================
// 2. Compression → decompression roundtrip
// ===========================================================================

#[test]
fn roundtrip_action_table_row_dedup() {
    let table = vec![
        vec![
            vec![Action::Shift(StateId(1))],
            vec![Action::Error],
            vec![Action::Reduce(RuleId(0))],
        ],
        // duplicate of row 0
        vec![
            vec![Action::Shift(StateId(1))],
            vec![Action::Error],
            vec![Action::Reduce(RuleId(0))],
        ],
        vec![
            vec![Action::Accept],
            vec![Action::Shift(StateId(2))],
            vec![Action::Error],
        ],
    ];

    let compressed = compress_action_table(&table);

    // Roundtrip every cell
    for (s, row) in table.iter().enumerate() {
        for (sym, cell) in row.iter().enumerate() {
            let expected = cell.first().cloned().unwrap_or(Action::Error);
            let got = decompress_action(&compressed, s, sym);
            assert_eq!(got, expected, "mismatch at state={s}, symbol={sym}");
        }
    }
}

#[test]
fn roundtrip_goto_table_sparse() {
    let table: Vec<Vec<Option<StateId>>> = vec![
        vec![None, Some(StateId(1)), None, None],
        vec![Some(StateId(2)), None, None, Some(StateId(5))],
        vec![None, None, Some(StateId(3)), None],
        vec![None, None, None, None],
    ];

    let compressed = compress_goto_table(&table);

    for (s, row) in table.iter().enumerate() {
        for (sym, &expected) in row.iter().enumerate() {
            let got = decompress_goto(&compressed, s, sym);
            assert_eq!(got, expected, "mismatch at state={s}, symbol={sym}");
        }
    }
}

#[test]
fn roundtrip_real_grammar_compression() {
    let grammar = make_arithmetic_grammar();
    let table = build_parse_table(&grammar);

    let compressor = TableCompressor::new();
    let token_indices = adze_tablegen::collect_token_indices(&grammar, &table);

    let compressed = compressor
        .compress(&table, &token_indices, false)
        .expect("compression of real grammar should succeed");

    // Structural checks on compressed output
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        table.state_count + 1,
        "row_offsets must cover every state plus sentinel"
    );
    assert_eq!(
        compressed.action_table.default_actions.len(),
        table.state_count,
        "one default action per state"
    );
    assert_eq!(
        compressed.goto_table.row_offsets.len(),
        table.state_count + 1,
    );
}

// ===========================================================================
// 3. NODE_TYPES JSON correctness
// ===========================================================================

#[test]
fn node_types_parens_grammar_valid_json() {
    let grammar = make_parens_grammar();
    let ntgen = NodeTypesGenerator::new(&grammar);
    let json_str = ntgen.generate().expect("NODE_TYPES generation");

    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("must produce valid JSON");
    assert!(parsed.is_array(), "NODE_TYPES must be a JSON array");
}

#[test]
fn node_types_arithmetic_grammar_contains_rule_names() {
    let grammar = make_arithmetic_grammar();
    let ntgen = NodeTypesGenerator::new(&grammar);
    let json_str = ntgen.generate().expect("NODE_TYPES generation");

    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let arr = parsed.as_array().unwrap();

    // Collect all type names
    let type_names: Vec<&str> = arr
        .iter()
        .filter_map(|v| v.get("type").and_then(|t| t.as_str()))
        .collect();

    assert!(
        type_names.contains(&"expression"),
        "should contain 'expression' node type, got: {:?}",
        type_names
    );
    assert!(
        type_names.contains(&"term"),
        "should contain 'term' node type"
    );
    assert!(
        type_names.contains(&"factor"),
        "should contain 'factor' node type"
    );
}

#[test]
fn node_types_single_rule_grammar() {
    let grammar = make_single_rule_grammar();
    let ntgen = NodeTypesGenerator::new(&grammar);
    let json_str = ntgen.generate().expect("NODE_TYPES generation");

    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let arr = parsed.as_array().unwrap();

    let type_names: Vec<&str> = arr
        .iter()
        .filter_map(|v| v.get("type").and_then(|t| t.as_str()))
        .collect();

    assert!(
        type_names.contains(&"root"),
        "should contain 'root' node type"
    );
}

#[test]
fn node_types_fields_grammar_includes_fields() {
    let grammar = make_fields_grammar();
    let ntgen = NodeTypesGenerator::new(&grammar);
    let json_str = ntgen.generate().expect("NODE_TYPES generation");

    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let arr = parsed.as_array().unwrap();

    // Find the assignment node type
    let assignment = arr
        .iter()
        .find(|v| v.get("type").and_then(|t| t.as_str()) == Some("assignment"))
        .expect("should have an 'assignment' node type");

    // It should have fields
    let fields = assignment.get("fields");
    assert!(
        fields.is_some(),
        "assignment node should have fields metadata"
    );
    let fields = fields.unwrap().as_object().unwrap();
    assert!(fields.contains_key("name"), "should have 'name' field");
    assert!(fields.contains_key("value"), "should have 'value' field");
}

#[test]
fn node_types_string_tokens_are_unnamed() {
    let grammar = make_parens_grammar();
    let ntgen = NodeTypesGenerator::new(&grammar);
    let json_str = ntgen.generate().expect("NODE_TYPES generation");

    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let arr = parsed.as_array().unwrap();

    // String-pattern tokens like "(" should appear as unnamed
    let unnamed: Vec<&str> = arr
        .iter()
        .filter(|v| v.get("named").and_then(|n| n.as_bool()) == Some(false))
        .filter_map(|v| v.get("type").and_then(|t| t.as_str()))
        .collect();

    assert!(
        unnamed.contains(&"("),
        "string token '(' should be unnamed, got: {:?}",
        unnamed
    );
    assert!(unnamed.contains(&")"), "string token ')' should be unnamed");
}

// ===========================================================================
// 4. ABI version compatibility checks
// ===========================================================================

#[test]
fn abi_version_constants() {
    use adze_tablegen::abi::*;

    assert_eq!(TREE_SITTER_LANGUAGE_VERSION, 15, "must target ABI v15");
    // Use const blocks to satisfy clippy::assertions_on_constants
    const { assert!(TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION <= TREE_SITTER_LANGUAGE_VERSION) };
    const { assert!(TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION >= 13) };
}

#[test]
fn abi_generated_code_embeds_version_15() {
    let grammar = make_single_rule_grammar();
    let table = build_parse_table(&grammar);
    let builder = AbiLanguageBuilder::new(&grammar, &table);

    let code = builder.generate().to_string();
    assert!(
        code.contains("LANGUAGE_VERSION"),
        "generated code must reference LANGUAGE_VERSION"
    );
}

#[test]
fn abi_language_builder_uses_correct_version() {
    let grammar = make_parens_grammar();
    let table = build_parse_table(&grammar);

    let builder = adze_tablegen::generate::LanguageBuilder::new(grammar, table);
    let lang = builder
        .generate_language()
        .expect("LanguageBuilder should succeed");

    assert_eq!(lang.version, 15, "Language struct version must be 15");
}

// ===========================================================================
// 5. Symbol metadata correctness
// ===========================================================================

#[test]
fn symbol_metadata_flags_combinatorics() {
    use adze_tablegen::abi::{create_symbol_metadata, symbol_metadata};

    // visible + named
    let m = create_symbol_metadata(true, true, false, false, false);
    assert_eq!(m, symbol_metadata::VISIBLE | symbol_metadata::NAMED);

    // hidden only
    let m = create_symbol_metadata(false, false, true, false, false);
    assert_eq!(m, symbol_metadata::HIDDEN);

    // supertype
    let m = create_symbol_metadata(true, true, false, false, true);
    assert_eq!(
        m,
        symbol_metadata::VISIBLE | symbol_metadata::NAMED | symbol_metadata::SUPERTYPE
    );

    // all flags
    let m = create_symbol_metadata(true, true, true, true, true);
    assert_eq!(
        m,
        symbol_metadata::VISIBLE
            | symbol_metadata::NAMED
            | symbol_metadata::HIDDEN
            | symbol_metadata::AUXILIARY
            | symbol_metadata::SUPERTYPE
    );

    // none
    let m = create_symbol_metadata(false, false, false, false, false);
    assert_eq!(m, 0);
}

#[test]
fn symbol_metadata_in_generated_code() {
    let grammar = make_arithmetic_grammar();
    let table = build_parse_table(&grammar);

    let slgen = StaticLanguageGenerator::new(grammar, table);
    let code = slgen.generate_language_code().to_string();

    assert!(
        code.contains("SYMBOL_METADATA"),
        "generated code must include SYMBOL_METADATA"
    );
}

#[test]
fn symbol_metadata_language_builder_counts() {
    let grammar = make_arithmetic_grammar();
    let table = build_parse_table(&grammar);

    let builder = adze_tablegen::generate::LanguageBuilder::new(grammar, table);
    let lang = builder.generate_language().expect("should succeed");

    // 7 symbols: EOF + 3 tokens + 3 non-terminals
    assert_eq!(lang.symbol_count, 7);
    // token_count = grammar.tokens.len() + 1 (EOF) = 4
    assert_eq!(lang.token_count, 4);
    assert_eq!(lang.external_token_count, 0);
}

// ===========================================================================
// 6. FFI struct layout verification
// ===========================================================================

#[test]
fn ffi_struct_sizes() {
    use adze_tablegen::abi::*;
    use std::mem;

    assert_eq!(mem::size_of::<TSSymbol>(), 2, "TSSymbol must be 2 bytes");
    assert_eq!(mem::size_of::<TSStateId>(), 2, "TSStateId must be 2 bytes");
    assert_eq!(mem::size_of::<TSFieldId>(), 2, "TSFieldId must be 2 bytes");
    assert_eq!(
        mem::size_of::<TSParseAction>(),
        6,
        "TSParseAction must be 6 bytes"
    );
    assert_eq!(
        mem::size_of::<TSLexState>(),
        4,
        "TSLexState must be 4 bytes"
    );
}

#[test]
fn ffi_struct_alignment() {
    use adze_tablegen::abi::TSLanguage;
    use std::mem;

    assert_eq!(
        mem::align_of::<TSLanguage>(),
        mem::align_of::<*const u8>(),
        "TSLanguage must be pointer-aligned for FFI"
    );
}

#[test]
fn ffi_external_scanner_default_is_null() {
    use adze_tablegen::abi::ExternalScanner;

    let scanner = ExternalScanner::default();
    assert!(scanner.states.is_null());
    assert!(scanner.symbol_map.is_null());
    assert!(scanner.create.is_none());
    assert!(scanner.destroy.is_none());
    assert!(scanner.scan.is_none());
    assert!(scanner.serialize.is_none());
    assert!(scanner.deserialize.is_none());
}

// ===========================================================================
// 7. Edge cases
// ===========================================================================

#[test]
fn edge_case_single_rule_grammar_full_pipeline() {
    let grammar = make_single_rule_grammar();
    let table = build_parse_table(&grammar);

    // ABI code generation
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();
    assert!(code.contains("TSLanguage"));

    // Compression
    let compressor = TableCompressor::new();
    let token_indices = adze_tablegen::collect_token_indices(&grammar, &table);
    let compressed = compressor.compress(&table, &token_indices, false);
    assert!(compressed.is_ok(), "single-rule grammar should compress");

    // NODE_TYPES
    let ntgen = NodeTypesGenerator::new(&grammar);
    let json = ntgen.generate().expect("NODE_TYPES");
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_array());

    // LanguageBuilder
    let builder2 = adze_tablegen::generate::LanguageBuilder::new(grammar, table);
    let lang = builder2.generate_language().expect("LanguageBuilder");
    assert_eq!(lang.version, 15);
}

#[test]
fn edge_case_grammar_with_external_tokens() {
    let mut grammar = make_single_rule_grammar();
    grammar.externals.push(ExternalToken {
        name: "comment".to_string(),
        symbol_id: SymbolId(100),
    });

    let table = build_parse_table(&grammar);
    let slgen = StaticLanguageGenerator::new(grammar, table);
    let code = slgen.generate_language_code().to_string();

    assert!(
        code.contains("EXTERNAL_TOKEN_COUNT"),
        "should declare external token count"
    );
    assert!(
        code.contains("EXTERNAL_SCANNER"),
        "should reference external scanner"
    );
}

#[test]
fn edge_case_many_state_grammar_compresses() {
    // The arithmetic grammar produces a non-trivial number of states
    let grammar = make_arithmetic_grammar();
    let table = build_parse_table(&grammar);

    assert!(
        table.state_count >= 5,
        "arithmetic grammar should produce at least 5 states, got {}",
        table.state_count
    );

    let compressor = TableCompressor::new();
    let token_indices = adze_tablegen::collect_token_indices(&grammar, &table);
    let compressed = compressor
        .compress(&table, &token_indices, false)
        .expect("compression should succeed for multi-state grammar");

    assert_eq!(
        compressed.action_table.row_offsets.len(),
        table.state_count + 1
    );
    assert_eq!(
        compressed.goto_table.row_offsets.len(),
        table.state_count + 1
    );
}

#[test]
fn edge_case_compression_of_all_error_cells() {
    // Table where every cell is an error except EOF→Accept in last state
    let table = vec![
        vec![vec![Action::Error], vec![Action::Error]],
        vec![vec![Action::Error], vec![Action::Error]],
    ];
    let compressed = compress_action_table(&table);

    // Verify all cells decompress to Error
    for s in 0..2 {
        for sym in 0..2 {
            assert_eq!(decompress_action(&compressed, s, sym), Action::Error);
        }
    }
}

#[test]
fn edge_case_empty_goto_table() {
    let table: Vec<Vec<Option<StateId>>> = vec![vec![None, None], vec![None, None]];
    let compressed = compress_goto_table(&table);

    // Verify all cells decompress to None
    for s in 0..2 {
        for sym in 0..2 {
            assert_eq!(decompress_goto(&compressed, s, sym), None);
        }
    }
}

#[test]
fn edge_case_node_types_empty_rules_grammar() {
    // Grammar with only tokens, no non-terminal rules
    let mut grammar = Grammar::new("empty_rules".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );

    let ntgen = NodeTypesGenerator::new(&grammar);
    let json_str = ntgen.generate().expect("should succeed even with no rules");
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn edge_case_language_builder_field_count() {
    let grammar = make_fields_grammar();
    let table = build_parse_table(&grammar);

    let builder = adze_tablegen::generate::LanguageBuilder::new(grammar, table);
    let lang = builder.generate_language().expect("should succeed");

    assert_eq!(lang.field_count, 2, "should have 2 fields (name, value)");
}

#[test]
fn edge_case_grammar_validates() {
    let grammar = make_arithmetic_grammar();
    assert!(
        grammar.validate().is_ok(),
        "well-formed grammar must pass validation"
    );

    let grammar = make_parens_grammar();
    assert!(grammar.validate().is_ok());

    let grammar = make_single_rule_grammar();
    assert!(grammar.validate().is_ok());

    let grammar = make_fields_grammar();
    assert!(grammar.validate().is_ok());
}
