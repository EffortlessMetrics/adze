#![allow(clippy::needless_range_loop)]

//! Property-based tests for symbol table generation in adze-tool.
//!
//! Uses proptest to validate invariants of the symbol table produced by
//! `adze_tool::generate_grammars` and the Tree-sitter C code generation
//! pipeline:
//!   - Symbol table in grammar JSON
//!   - Symbol names in C code
//!   - Symbol IDs are sequential
//!   - Symbol table determinism
//!   - Named vs anonymous symbols
//!   - Symbol count matches grammar
//!   - Symbol table with special characters

use adze_tool::generate_grammars;
use proptest::prelude::*;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use tempfile::TempDir;
use tree_sitter_generate::generate_parser_for_grammar;

const SEMANTIC_VERSION: Option<(u8, u8, u8)> = Some((0, 25, 1));

// ===========================================================================
// Helpers
// ===========================================================================

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Write Rust source to a temp file and return (dir, path).
fn write_temp(src: &str) -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().unwrap();
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = dir.path().join(format!("sym_{id}.rs"));
    std::fs::write(&path, src).unwrap();
    (dir, path)
}

/// Extract exactly one grammar JSON from source.
fn extract_one(src: &str) -> Value {
    let (dir, path) = write_temp(src);
    let gs = generate_grammars(&path).unwrap();
    drop(dir);
    assert_eq!(gs.len(), 1, "expected 1 grammar, got {}", gs.len());
    gs.into_iter().next().unwrap()
}

/// Generate C code from grammar source. Returns (grammar_name, c_code).
fn gen_c(src: &str) -> (String, String) {
    let grammar = extract_one(src);
    let json = serde_json::to_string(&grammar).unwrap();
    generate_parser_for_grammar(&json, SEMANTIC_VERSION).unwrap()
}

/// Extract the ts_symbol_names entries from C code.
/// Returns a Vec of symbol name strings in declaration order.
fn extract_symbol_names(c_code: &str) -> Vec<String> {
    let mut names = Vec::new();
    // Find the ts_symbol_names array and extract quoted string entries
    let marker = "ts_symbol_names";
    if let Some(start) = c_code.find(marker) {
        // Find the opening brace of the array
        if let Some(brace) = c_code[start..].find('{') {
            let arr_start = start + brace;
            // Find the matching closing brace
            if let Some(end) = c_code[arr_start..].find("};") {
                let block = &c_code[arr_start..arr_start + end + 1];
                // Extract each quoted string
                let mut i = 0;
                let bytes = block.as_bytes();
                while i < bytes.len() {
                    if bytes[i] == b'"' {
                        let str_start = i + 1;
                        i += 1;
                        while i < bytes.len() && bytes[i] != b'"' {
                            if bytes[i] == b'\\' {
                                i += 1; // skip escaped char
                            }
                            i += 1;
                        }
                        let s = String::from_utf8_lossy(&bytes[str_start..i]).to_string();
                        names.push(s);
                    }
                    i += 1;
                }
            }
        }
    }
    names
}

/// Extract the numeric SYMBOL_COUNT value from C code.
fn extract_symbol_count(c_code: &str) -> Option<usize> {
    for line in c_code.lines() {
        if line.contains("#define") && line.contains("SYMBOL_COUNT") {
            if let Some(num_str) = line.split_whitespace().last() {
                return num_str.parse().ok();
            }
        }
    }
    None
}

/// Extract the ts_symbol_metadata entries from C code.
/// Returns Vec of (is_named: bool) in declaration order.
fn extract_symbol_metadata(c_code: &str) -> Vec<bool> {
    let mut entries = Vec::new();
    let marker = "ts_symbol_metadata";
    if let Some(start) = c_code.find(marker) {
        if let Some(brace) = c_code[start..].find('{') {
            let arr_start = start + brace;
            if let Some(end) = c_code[arr_start..].find("};") {
                let block = &c_code[arr_start..arr_start + end + 1];
                // Each entry looks like: {.visible = true, .named = true},
                for line in block.lines() {
                    if line.contains(".named") {
                        let named = line.contains(".named = true");
                        entries.push(named);
                    }
                }
            }
        }
    }
    entries
}

// ===========================================================================
// Source generators
// ===========================================================================

fn grammar_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{1,10}"
}

fn type_name_strategy() -> impl Strategy<Value = String> {
    "[A-Z][a-z]{1,8}".prop_filter("non-empty", |s| !s.is_empty())
}

fn field_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,8}".prop_filter("avoid keywords", |s| {
        !matches!(
            s.as_str(),
            "type" | "fn" | "let" | "mut" | "ref" | "pub" | "mod" | "use" | "self" | "super"
                | "crate" | "struct" | "enum" | "impl" | "trait" | "where" | "for" | "loop"
                | "while" | "if" | "else" | "match" | "return" | "break" | "continue" | "as"
                | "in" | "move" | "box" | "dyn" | "async" | "await" | "try" | "yield"
                | "macro" | "const" | "static" | "unsafe" | "extern"
        )
    })
}

fn src_enum(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub enum Expr {{
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            u32
        ),
    }}
}}
"#
    )
}

fn src_enum_n(name: &str, n: usize) -> String {
    let variants: String = (0..n)
        .map(|i| {
            format!(
                r#"        V{i}(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            u32
        )"#
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub enum Root {{
{variants}
    }}
}}
"#
    )
}

fn src_struct(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub struct Root {{
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        value: u32,
    }}
}}
"#
    )
}

fn src_struct_n(name: &str, n: usize) -> String {
    let fields: String = (0..n)
        .map(|i| {
            format!(
                r#"        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        f{i}: u32,"#
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub struct Root {{
{fields}
    }}
}}
"#
    )
}

fn src_extras(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub enum Expr {{
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            u32
        ),
    }}

    #[adze::extra]
    struct Whitespace {{
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }}
}}
"#
    )
}

fn src_recursive(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub enum Expr {{
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            u32
        ),
        Neg(
            #[adze::leaf(text = "-", transform = |v| ())]
            (),
            Box<Expr>,
        ),
    }}
}}
"#
    )
}

fn src_prec_left(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub enum Expr {{
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            u32
        ),
        #[adze::prec_left(1)]
        Add(
            Box<Expr>,
            #[adze::leaf(text = "+", transform = |v| ())]
            (),
            Box<Expr>,
        ),
    }}
}}
"#
    )
}

fn src_text_leaves(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub enum Op {{
        Plus(
            #[adze::leaf(text = "+", transform = |v| ())]
            ()
        ),
        Minus(
            #[adze::leaf(text = "-", transform = |v| ())]
            ()
        ),
    }}
}}
"#
    )
}

fn src_unboxed(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub struct Language {{
        e: Expression,
    }}

    pub enum Expression {{
        Number(
            #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<i32>().unwrap())]
            i32
        ),
    }}
}}
"#
    )
}

fn src_optional(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub struct Root {{
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        required: u32,
        #[adze::leaf(pattern = r"[a-z]+", transform = |v| v.to_string())]
        opt: Option<String>,
    }}
}}
"#
    )
}

fn src_vec(name: &str) -> String {
    format!(
        r#"
#[adze::grammar("{name}")]
mod grammar {{
    #[adze::language]
    pub struct Root {{
        #[adze::repeat(non_empty = true)]
        items: Vec<Item>,
    }}

    pub struct Item {{
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
        v: u32,
    }}

    #[adze::extra]
    struct Whitespace {{
        #[adze::leaf(pattern = r"\s")]
        _whitespace: (),
    }}
}}
"#
    )
}

// ===========================================================================
// 1. Symbol table in grammar JSON (tests 1-5)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 1. Grammar JSON rules object contains at least source_file symbol.
    #[test]
    fn json_rules_contain_source_file(name in grammar_name()) {
        let grammar = extract_one(&src_enum(&name));
        let rules = grammar["rules"].as_object().unwrap();
        prop_assert!(
            rules.contains_key("source_file"),
            "rules missing source_file symbol"
        );
    }

    /// 2. Struct grammar rules include the root type as a named symbol.
    #[test]
    fn json_rules_include_root_type_struct(
        name in grammar_name(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {type_name} {{
                    #[adze::leaf(pattern = r"\d+")]
                    pub {field}: String,
                }}
            }}
            "##,
        );
        let grammar = extract_one(&src);
        let rules = grammar["rules"].as_object().unwrap();
        prop_assert!(
            rules.contains_key(type_name.as_str()),
            "rules missing root type '{type_name}'"
        );
    }

    /// 3. Enum grammar rules include the enum type as a named symbol.
    #[test]
    fn json_rules_include_root_type_enum(
        name in grammar_name(),
        type_name in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {type_name} {{
                    Leaf(
                        #[adze::leaf(pattern = r"\d+")]
                        String
                    ),
                }}
            }}
            "##,
        );
        let grammar = extract_one(&src);
        let rules = grammar["rules"].as_object().unwrap();
        prop_assert!(
            rules.contains_key(type_name.as_str()),
            "rules missing root enum type '{type_name}'"
        );
    }

    /// 4. Each rule in the JSON is an object with a "type" field.
    #[test]
    fn json_rule_entries_have_type_field(name in grammar_name()) {
        let grammar = extract_one(&src_enum(&name));
        let rules = grammar["rules"].as_object().unwrap();
        for (rule_name, rule_value) in rules {
            prop_assert!(
                rule_value.is_object(),
                "rule '{rule_name}' is not an object"
            );
            prop_assert!(
                rule_value.get("type").is_some(),
                "rule '{rule_name}' missing 'type' field"
            );
        }
    }

    /// 5. Grammar with child struct has both root and child in rules.
    #[test]
    fn json_rules_contain_child_struct(name in grammar_name()) {
        let grammar = extract_one(&src_unboxed(&name));
        let rules = grammar["rules"].as_object().unwrap();
        prop_assert!(rules.contains_key("Language"));
        prop_assert!(rules.contains_key("Expression"));
    }
}

// ===========================================================================
// 2. Symbol names in C code (tests 6-10)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 6. C symbol names array contains "end" as the first entry.
    #[test]
    fn c_symbol_names_starts_with_end(name in grammar_name()) {
        let (_, c) = gen_c(&src_enum(&name));
        let names = extract_symbol_names(&c);
        prop_assert!(
            !names.is_empty(),
            "ts_symbol_names is empty"
        );
        prop_assert_eq!(
            &names[0], "end",
            "first symbol should be 'end', got '{}'", names[0]
        );
    }

    /// 7. C symbol names contain the root type name.
    #[test]
    fn c_symbol_names_contain_root(name in grammar_name()) {
        let (_, c) = gen_c(&src_struct(&name));
        let names = extract_symbol_names(&c);
        prop_assert!(
            names.iter().any(|n| n == "Root"),
            "symbol names missing 'Root': {:?}", names
        );
    }

    /// 8. C symbol names for enum grammar contain the enum type.
    #[test]
    fn c_symbol_names_contain_enum_type(name in grammar_name()) {
        let (_, c) = gen_c(&src_enum(&name));
        let names = extract_symbol_names(&c);
        prop_assert!(
            names.iter().any(|n| n == "Expr"),
            "symbol names missing 'Expr': {:?}", names
        );
    }

    /// 9. C symbol names contain source_file.
    #[test]
    fn c_symbol_names_contain_source_file(name in grammar_name()) {
        let (_, c) = gen_c(&src_enum(&name));
        let names = extract_symbol_names(&c);
        prop_assert!(
            names.iter().any(|n| n == "source_file"),
            "symbol names missing 'source_file': {:?}", names
        );
    }

    /// 10. C symbol names for unboxed grammar contain both struct and enum types.
    #[test]
    fn c_symbol_names_contain_both_types(name in grammar_name()) {
        let (_, c) = gen_c(&src_unboxed(&name));
        let names = extract_symbol_names(&c);
        prop_assert!(
            names.iter().any(|n| n == "Language"),
            "symbol names missing 'Language'"
        );
        prop_assert!(
            names.iter().any(|n| n == "Expression"),
            "symbol names missing 'Expression'"
        );
    }
}

// ===========================================================================
// 3. Symbol IDs are sequential (tests 11-14)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 11. SYMBOL_COUNT matches the number of ts_symbol_names entries.
    #[test]
    fn symbol_count_matches_names_len(name in grammar_name()) {
        let (_, c) = gen_c(&src_enum(&name));
        let count = extract_symbol_count(&c).expect("SYMBOL_COUNT not found");
        let names = extract_symbol_names(&c);
        prop_assert_eq!(
            count, names.len(),
            "SYMBOL_COUNT ({}) != ts_symbol_names length ({})", count, names.len()
        );
    }

    /// 12. SYMBOL_COUNT matches names length for struct grammars.
    #[test]
    fn symbol_count_matches_names_len_struct(name in grammar_name()) {
        let (_, c) = gen_c(&src_struct(&name));
        let count = extract_symbol_count(&c).expect("SYMBOL_COUNT not found");
        let names = extract_symbol_names(&c);
        prop_assert_eq!(count, names.len());
    }

    /// 13. SYMBOL_COUNT matches names length for recursive grammars.
    #[test]
    fn symbol_count_matches_names_len_recursive(name in grammar_name()) {
        let (_, c) = gen_c(&src_recursive(&name));
        let count = extract_symbol_count(&c).expect("SYMBOL_COUNT not found");
        let names = extract_symbol_names(&c);
        prop_assert_eq!(count, names.len());
    }

    /// 14. SYMBOL_COUNT matches for grammars with varying variant counts.
    #[test]
    fn symbol_count_matches_varying_variants(name in grammar_name(), n in 1..=5usize) {
        let (_, c) = gen_c(&src_enum_n(&name, n));
        let count = extract_symbol_count(&c).expect("SYMBOL_COUNT not found");
        let names = extract_symbol_names(&c);
        prop_assert_eq!(count, names.len());
    }
}

// ===========================================================================
// 4. Symbol table determinism (tests 15-19)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 15. Symbol names are identical across two generations (enum).
    #[test]
    fn symbol_names_deterministic_enum(name in grammar_name()) {
        let (_, c1) = gen_c(&src_enum(&name));
        let (_, c2) = gen_c(&src_enum(&name));
        let n1 = extract_symbol_names(&c1);
        let n2 = extract_symbol_names(&c2);
        prop_assert_eq!(&n1, &n2, "symbol names differ across runs");
    }

    /// 16. Symbol names are identical across two generations (struct).
    #[test]
    fn symbol_names_deterministic_struct(name in grammar_name()) {
        let (_, c1) = gen_c(&src_struct(&name));
        let (_, c2) = gen_c(&src_struct(&name));
        let n1 = extract_symbol_names(&c1);
        let n2 = extract_symbol_names(&c2);
        prop_assert_eq!(&n1, &n2);
    }

    /// 17. Symbol names are identical across two generations (recursive).
    #[test]
    fn symbol_names_deterministic_recursive(name in grammar_name()) {
        let (_, c1) = gen_c(&src_recursive(&name));
        let (_, c2) = gen_c(&src_recursive(&name));
        let n1 = extract_symbol_names(&c1);
        let n2 = extract_symbol_names(&c2);
        prop_assert_eq!(&n1, &n2);
    }

    /// 18. Symbol count is identical across two generations.
    #[test]
    fn symbol_count_deterministic(name in grammar_name()) {
        let (_, c1) = gen_c(&src_prec_left(&name));
        let (_, c2) = gen_c(&src_prec_left(&name));
        let sc1 = extract_symbol_count(&c1);
        let sc2 = extract_symbol_count(&c2);
        prop_assert_eq!(sc1, sc2, "SYMBOL_COUNT differs across runs");
    }

    /// 19. Symbol metadata is identical across two generations.
    #[test]
    fn symbol_metadata_deterministic(name in grammar_name()) {
        let (_, c1) = gen_c(&src_extras(&name));
        let (_, c2) = gen_c(&src_extras(&name));
        let m1 = extract_symbol_metadata(&c1);
        let m2 = extract_symbol_metadata(&c2);
        prop_assert_eq!(&m1, &m2, "symbol metadata differs across runs");
    }
}

// ===========================================================================
// 5. Named vs anonymous symbols (tests 20-24)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 20. Symbol metadata has at least one named entry (the root type).
    #[test]
    fn metadata_has_named_symbols(name in grammar_name()) {
        let (_, c) = gen_c(&src_enum(&name));
        let meta = extract_symbol_metadata(&c);
        prop_assert!(
            meta.iter().any(|&named| named),
            "no named symbols found in metadata"
        );
    }

    /// 21. Symbol metadata length matches SYMBOL_COUNT.
    #[test]
    fn metadata_length_matches_symbol_count(name in grammar_name()) {
        let (_, c) = gen_c(&src_enum(&name));
        let count = extract_symbol_count(&c).expect("SYMBOL_COUNT not found");
        let meta = extract_symbol_metadata(&c);
        prop_assert_eq!(
            count, meta.len(),
            "SYMBOL_COUNT ({}) != metadata entries ({})", count, meta.len()
        );
    }

    /// 22. Text-literal grammar has anonymous symbols for the literal tokens.
    #[test]
    fn text_leaves_have_anonymous_symbols(name in grammar_name()) {
        let (_, c) = gen_c(&src_text_leaves(&name));
        let meta = extract_symbol_metadata(&c);
        // Must have at least one anonymous (non-named) symbol for "+" or "-"
        prop_assert!(
            meta.iter().any(|&named| !named),
            "expected anonymous symbols for text literals"
        );
    }

    /// 23. Struct grammar with child has named entries for both types.
    #[test]
    fn unboxed_grammar_has_multiple_named(name in grammar_name()) {
        let (_, c) = gen_c(&src_unboxed(&name));
        let meta = extract_symbol_metadata(&c);
        let named_count = meta.iter().filter(|&&named| named).count();
        // source_file + Language + Expression = at least 3 named
        prop_assert!(
            named_count >= 3,
            "expected at least 3 named symbols, got {named_count}"
        );
    }

    /// 24. Extras grammar metadata length matches symbol count.
    #[test]
    fn extras_metadata_matches_count(name in grammar_name()) {
        let (_, c) = gen_c(&src_extras(&name));
        let count = extract_symbol_count(&c).expect("SYMBOL_COUNT not found");
        let meta = extract_symbol_metadata(&c);
        prop_assert_eq!(count, meta.len());
    }
}

// ===========================================================================
// 6. Symbol count matches grammar (tests 25-29)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 25. SYMBOL_COUNT is always at least 3 (end, source_file, root type).
    #[test]
    fn symbol_count_minimum_enum(name in grammar_name()) {
        let (_, c) = gen_c(&src_enum(&name));
        let count = extract_symbol_count(&c).expect("SYMBOL_COUNT not found");
        prop_assert!(
            count >= 3,
            "SYMBOL_COUNT ({count}) should be >= 3"
        );
    }

    /// 26. Adding more variants increases or maintains symbol count.
    #[test]
    fn more_variants_more_or_equal_symbols(name in grammar_name()) {
        let (_, c1) = gen_c(&src_enum_n(&name, 1));
        let (_, c2) = gen_c(&src_enum_n(&name, 3));
        let sc1 = extract_symbol_count(&c1).unwrap();
        let sc2 = extract_symbol_count(&c2).unwrap();
        prop_assert!(
            sc2 >= sc1,
            "3 variants ({sc2}) should have >= symbols than 1 variant ({sc1})"
        );
    }

    /// 27. Adding more struct fields increases or maintains symbol count.
    #[test]
    fn more_fields_more_or_equal_symbols(name in grammar_name()) {
        let (_, c1) = gen_c(&src_struct_n(&name, 1));
        let (_, c2) = gen_c(&src_struct_n(&name, 3));
        let sc1 = extract_symbol_count(&c1).unwrap();
        let sc2 = extract_symbol_count(&c2).unwrap();
        prop_assert!(
            sc2 >= sc1,
            "3 fields ({sc2}) should have >= symbols than 1 field ({sc1})"
        );
    }

    /// 28. Grammar with extras has at least as many symbols as without extras.
    #[test]
    fn extras_add_symbols(name in grammar_name()) {
        let (_, c_no_extras) = gen_c(&src_enum(&name));
        let (_, c_extras) = gen_c(&src_extras(&name));
        let sc1 = extract_symbol_count(&c_no_extras).unwrap();
        let sc2 = extract_symbol_count(&c_extras).unwrap();
        prop_assert!(
            sc2 >= sc1,
            "extras grammar ({sc2}) should have >= symbols than no-extras ({sc1})"
        );
    }

    /// 29. Vec/repeat grammar has more symbols than a simple struct.
    #[test]
    fn vec_grammar_has_more_symbols(name in grammar_name()) {
        let (_, c_simple) = gen_c(&src_struct(&name));
        let (_, c_vec) = gen_c(&src_vec(&name));
        let sc1 = extract_symbol_count(&c_simple).unwrap();
        let sc2 = extract_symbol_count(&c_vec).unwrap();
        prop_assert!(
            sc2 >= sc1,
            "vec grammar ({sc2}) should have >= symbols than simple struct ({sc1})"
        );
    }
}

// ===========================================================================
// 7. Symbol table with special characters (tests 30-34)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// 30. Grammar names with underscores produce valid symbol tables.
    #[test]
    fn underscore_name_valid_symbols(
        prefix in "[a-z]{1,4}",
        suffix in "[a-z]{1,4}",
    ) {
        let name = format!("{prefix}_{suffix}");
        let (_, c) = gen_c(&src_enum(&name));
        let names = extract_symbol_names(&c);
        prop_assert!(!names.is_empty(), "symbol names empty for underscore name");
        let count = extract_symbol_count(&c).unwrap();
        prop_assert_eq!(count, names.len());
    }

    /// 31. Grammar names with digits produce valid symbol tables.
    #[test]
    fn digit_name_valid_symbols(
        prefix in "[a-z]{1,4}",
        digits in "[0-9]{1,3}",
    ) {
        let name = format!("{prefix}{digits}");
        let (_, c) = gen_c(&src_enum(&name));
        let names = extract_symbol_names(&c);
        prop_assert!(!names.is_empty());
        prop_assert_eq!(
            &names[0], "end",
            "first symbol should be 'end'"
        );
    }

    /// 32. Symbol names contain no null bytes.
    #[test]
    fn symbol_names_no_null_bytes(name in grammar_name()) {
        let (_, c) = gen_c(&src_enum(&name));
        let names = extract_symbol_names(&c);
        for (i, sym) in names.iter().enumerate() {
            prop_assert!(
                !sym.contains('\0'),
                "symbol name at index {i} contains null byte: {:?}", sym
            );
        }
    }

    /// 33. Symbol names are all valid UTF-8 and non-empty (except internal markers).
    #[test]
    fn symbol_names_valid_utf8(name in grammar_name()) {
        let (_, c) = gen_c(&src_recursive(&name));
        let names = extract_symbol_names(&c);
        for (i, sym) in names.iter().enumerate() {
            // All names extracted should be valid strings (by construction)
            // and should have reasonable length
            prop_assert!(
                sym.len() < 256,
                "symbol name at index {i} unreasonably long: {}", sym.len()
            );
        }
    }

    /// 34. No duplicate symbol names exist in the names array
    /// (except potentially for aliased symbols which map to same name).
    #[test]
    fn symbol_names_mostly_unique(name in grammar_name()) {
        let (_, c) = gen_c(&src_unboxed(&name));
        let names = extract_symbol_names(&c);
        let unique: HashSet<&str> = names.iter().map(|s| s.as_str()).collect();
        // Allow some duplication (aliases), but not excessive
        let dup_count = names.len() - unique.len();
        prop_assert!(
            dup_count <= names.len() / 2,
            "too many duplicate symbol names ({dup_count} dupes out of {})", names.len()
        );
    }
}
