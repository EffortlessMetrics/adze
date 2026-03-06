#![allow(clippy::needless_range_loop)]

//! Property-based tests for rule naming conventions in adze-common.
//!
//! In adze, rule names are derived from Rust type identifiers (structs and enums).
//! Tree-sitter grammars conventionally use snake_case rule names. These tests verify
//! properties of that conversion pipeline: snake_case correctness, uniqueness,
//! determinism, sanitization, and handling of generics and nested types.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::{HashMap, HashSet};
use syn::{Item, ItemMod, parse_str};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a PascalCase identifier to snake_case, mimicking how adze derives
/// rule names from Rust struct/enum names for Tree-sitter grammars.
fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            // Insert underscore before uppercase letter that follows a lowercase
            // letter or precedes a lowercase letter in an acronym run.
            let prev = name.chars().nth(i - 1).unwrap_or('_');
            if prev.is_lowercase() || prev.is_ascii_digit() {
                result.push('_');
            } else if let Some(next) = name.chars().nth(i + 1)
                && next.is_lowercase()
            {
                result.push('_');
            }
        }
        result.push(ch.to_lowercase().next().unwrap());
    }
    result
}

/// Derive a rule name from a Rust type path string.
/// Uses the last segment of the path and converts to snake_case.
fn rule_name_from_type_path(path: &str) -> String {
    let last_segment = path.rsplit("::").next().unwrap_or(path);
    // Strip any generic suffix like "<T>"
    let base = if let Some(idx) = last_segment.find('<') {
        &last_segment[..idx]
    } else {
        last_segment
    };
    to_snake_case(base)
}

/// Sanitize a rule name: replace non-alphanumeric/underscore chars with
/// underscores and collapse runs of underscores.
fn sanitize_rule_name(name: &str) -> String {
    let replaced: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    // Collapse consecutive underscores
    let mut result = String::new();
    let mut prev_underscore = false;
    for c in replaced.chars() {
        if c == '_' {
            if !prev_underscore {
                result.push(c);
            }
            prev_underscore = true;
        } else {
            result.push(c);
            prev_underscore = false;
        }
    }
    // Trim leading/trailing underscores
    result.trim_matches('_').to_string()
}

/// Build a grammar module source string with a body.
fn build_grammar_module(mod_name: &str, body: &str) -> String {
    format!(
        r#"#[adze::grammar("test")]
mod {mod_name} {{
{body}
}}"#
    )
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Valid PascalCase identifiers for struct/enum names.
fn pascal_case_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[A-Z][a-z]{1,6}([A-Z][a-z]{1,6}){0,3}")
        .unwrap()
        .prop_filter("must be valid ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

/// Valid snake_case identifiers for module names.
fn snake_case_ident() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,12}")
        .unwrap()
        .prop_filter("must be valid ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

/// Simple leaf type names.
fn leaf_type_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["i32", "u32", "f64", "bool", "String", "usize", "u8"][..])
}

/// Distinct pairs of PascalCase names for uniqueness tests.
fn distinct_pascal_pair() -> impl Strategy<Value = (String, String)> {
    (pascal_case_strategy(), pascal_case_strategy()).prop_filter("must differ", |(a, b)| a != b)
}

// ---------------------------------------------------------------------------
// Tests: Snake_case conversion for struct names
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 1. Snake_case of a PascalCase name is all lowercase
    #[test]
    fn snake_case_is_all_lowercase(name in pascal_case_strategy()) {
        let snake = to_snake_case(&name);
        prop_assert!(
            snake.chars().all(|c| c.is_lowercase() || c == '_' || c.is_ascii_digit()),
            "expected all lowercase: {snake}"
        );
    }

    // 2. Snake_case of a struct name is non-empty
    #[test]
    fn snake_case_non_empty(name in pascal_case_strategy()) {
        let snake = to_snake_case(&name);
        prop_assert!(!snake.is_empty(), "snake_case should not be empty");
    }

    // 3. Struct ident converts to snake_case rule name in grammar module
    #[test]
    fn struct_ident_to_snake_case_rule(
        struct_name in pascal_case_strategy(),
        mod_name in snake_case_ident(),
    ) {
        let body = format!("    pub struct {struct_name} {{ pub value: i32, }}");
        let src = build_grammar_module(&mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Struct(s) = &items[0] {
            let rule = to_snake_case(&s.ident.to_string());
            prop_assert!(
                rule.chars().all(|c| c.is_lowercase() || c == '_' || c.is_ascii_digit()),
                "rule name should be snake_case: {rule}"
            );
        }
    }

    // 4. Snake_case preserves alphabetic content (letters match when underscores removed)
    #[test]
    fn snake_case_preserves_letters(name in pascal_case_strategy()) {
        let snake = to_snake_case(&name);
        let snake_letters: String = snake.chars().filter(|c| c.is_alphabetic()).collect();
        let orig_lower: String = name.chars().map(|c| c.to_lowercase().next().unwrap()).collect();
        prop_assert_eq!(snake_letters, orig_lower);
    }

    // 5. Single-word PascalCase (no internal uppercase) produces no underscores
    #[test]
    fn single_word_no_underscores(
        word in prop::string::string_regex("[A-Z][a-z]{1,8}")
            .unwrap()
            .prop_filter("valid ident", |s| syn::parse_str::<syn::Ident>(s).is_ok())
    ) {
        let snake = to_snake_case(&word);
        prop_assert!(!snake.contains('_'), "single word should have no underscores: {snake}");
    }
}

// ---------------------------------------------------------------------------
// Tests: Snake_case conversion for enum names
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 6. Enum ident converts to snake_case rule name
    #[test]
    fn enum_ident_to_snake_case_rule(
        enum_name in pascal_case_strategy(),
        mod_name in snake_case_ident(),
    ) {
        let body = format!("    pub enum {enum_name} {{ A, B, }}");
        let src = build_grammar_module(&mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Enum(e) = &items[0] {
            let rule = to_snake_case(&e.ident.to_string());
            prop_assert!(
                rule.chars().all(|c| c.is_lowercase() || c == '_' || c.is_ascii_digit()),
                "enum rule name should be snake_case: {rule}"
            );
        }
    }

    // 7. Enum variant names also produce valid snake_case
    #[test]
    fn enum_variant_snake_case(
        variant_name in pascal_case_strategy(),
        mod_name in snake_case_ident(),
    ) {
        let body = format!("    pub enum MyEnum {{ {variant_name}, Other, }}");
        let src = build_grammar_module(&mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Enum(e) = &items[0] {
            let first_variant = e.variants.first().unwrap();
            let rule = to_snake_case(&first_variant.ident.to_string());
            prop_assert!(
                rule.chars().all(|c| c.is_lowercase() || c == '_' || c.is_ascii_digit()),
                "variant rule: {rule}"
            );
        }
    }

    // 8. Enum name and variant produce distinct rule names
    #[test]
    fn enum_and_variant_distinct_rules(
        enum_name in pascal_case_strategy(),
        mod_name in snake_case_ident(),
    ) {
        // The enum itself and its variant "Alpha" should have different rule names
        let body = format!("    pub enum {enum_name} {{ Alpha, Beta, }}");
        let src = build_grammar_module(&mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Enum(e) = &items[0] {
            let enum_rule = to_snake_case(&e.ident.to_string());
            let variant_rule = to_snake_case(&e.variants[0].ident.to_string());
            // They should generally differ unless enum_name happens to be "Alpha"
            if e.ident != "Alpha" {
                prop_assert_ne!(enum_rule, variant_rule);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests: Rule name uniqueness
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 9. Distinct PascalCase names produce distinct snake_case rule names
    #[test]
    fn distinct_names_produce_distinct_rules((a, b) in distinct_pascal_pair()) {
        let rule_a = to_snake_case(&a);
        let rule_b = to_snake_case(&b);
        prop_assert_ne!(rule_a, rule_b, "{} and {} should produce different rules", a, b);
    }

    // 10. Multiple structs in a module produce unique rule names
    #[test]
    fn multiple_structs_unique_rules(
        mod_name in snake_case_ident(),
        count in 2usize..=6,
    ) {
        let names: Vec<String> = (0..count).map(|i| format!("Type{}", (b'A' + i as u8) as char)).collect();
        let body: String = names.iter()
            .map(|n| format!("    pub struct {n} {{ pub v: i32, }}"))
            .collect::<Vec<_>>()
            .join("\n");
        let src = build_grammar_module(&mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        let rules: HashSet<String> = items.iter().filter_map(|item| {
            if let Item::Struct(s) = item {
                Some(to_snake_case(&s.ident.to_string()))
            } else {
                None
            }
        }).collect();
        prop_assert_eq!(rules.len(), count, "all rule names should be unique");
    }

    // 11. Mixed struct/enum names produce unique rule names
    #[test]
    fn mixed_types_unique_rules(mod_name in snake_case_ident()) {
        let body = [
            "    pub struct Foo { pub v: i32, }",
            "    pub enum Bar { A, B, }",
            "    pub struct BazItem { pub v: u32, }",
        ].join("\n");
        let src = build_grammar_module(&mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        let rules: HashSet<String> = items.iter().filter_map(|item| {
            match item {
                Item::Struct(s) => Some(to_snake_case(&s.ident.to_string())),
                Item::Enum(e) => Some(to_snake_case(&e.ident.to_string())),
                _ => None,
            }
        }).collect();
        prop_assert_eq!(rules.len(), 3);
    }
}

// ---------------------------------------------------------------------------
// Tests: Rule name from type path
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 12. Rule name from simple type path uses last segment
    #[test]
    fn rule_name_from_simple_path(name in pascal_case_strategy()) {
        let path = format!("crate::grammar::{name}");
        let rule = rule_name_from_type_path(&path);
        let expected = to_snake_case(&name);
        prop_assert_eq!(rule, expected);
    }

    // 13. Rule name from single-segment path
    #[test]
    fn rule_name_from_single_segment(name in pascal_case_strategy()) {
        let rule = rule_name_from_type_path(&name);
        let expected = to_snake_case(&name);
        prop_assert_eq!(rule, expected);
    }

    // 14. Rule name from deep path still extracts last segment
    #[test]
    fn rule_name_from_deep_path(name in pascal_case_strategy()) {
        let path = format!("a::b::c::d::{name}");
        let rule = rule_name_from_type_path(&path);
        let expected = to_snake_case(&name);
        prop_assert_eq!(rule, expected);
    }

    // 15. Rule name from type path with module prefix matches direct conversion
    #[test]
    fn rule_name_path_matches_direct(
        name in pascal_case_strategy(),
        prefix in snake_case_ident(),
    ) {
        let path_rule = rule_name_from_type_path(&format!("{prefix}::{name}"));
        let direct_rule = to_snake_case(&name);
        prop_assert_eq!(path_rule, direct_rule);
    }
}

// ---------------------------------------------------------------------------
// Tests: Rule name with generics
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 16. Generics are stripped from rule name derivation
    #[test]
    fn generics_stripped_from_rule_name(name in pascal_case_strategy()) {
        let path = format!("{name}<T>");
        let rule = rule_name_from_type_path(&path);
        let expected = to_snake_case(&name);
        prop_assert_eq!(rule, expected, "generics should not affect rule name");
    }

    // 17. Generic type in struct field doesn't affect struct rule name
    #[test]
    fn generic_field_does_not_affect_struct_rule(
        struct_name in pascal_case_strategy(),
        inner in leaf_type_name(),
        mod_name in snake_case_ident(),
    ) {
        let body = format!("    pub struct {struct_name} {{ pub items: Vec<{inner}>, }}");
        let src = build_grammar_module(&mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Struct(s) = &items[0] {
            let rule = to_snake_case(&s.ident.to_string());
            let expected = to_snake_case(&struct_name);
            prop_assert_eq!(rule, expected);
        }
    }

    // 18. Complex generic path still extracts correct base name
    #[test]
    fn complex_generic_path(name in pascal_case_strategy()) {
        let path = format!("mod_a::mod_b::{name}<Vec<String>>");
        let rule = rule_name_from_type_path(&path);
        let expected = to_snake_case(&name);
        prop_assert_eq!(rule, expected);
    }

    // 19. Wrapped generic types (via wrap_leaf_type) preserve rule name derivation
    #[test]
    fn wrapped_type_preserves_rule_name(
        struct_name in pascal_case_strategy(),
        inner in leaf_type_name(),
        mod_name in snake_case_ident(),
    ) {
        let body = format!("    pub struct {struct_name} {{ pub val: {inner}, }}");
        let src = build_grammar_module(&mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Struct(s) = &items[0] {
            // Wrapping the field type doesn't change the struct's rule name
            let field = s.fields.iter().next().unwrap();
            let skip: HashSet<&str> = HashSet::new();
            let _wrapped = wrap_leaf_type(&field.ty, &skip);
            let rule = to_snake_case(&s.ident.to_string());
            let expected = to_snake_case(&struct_name);
            prop_assert_eq!(rule, expected);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests: Rule name determinism
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 20. Snake_case conversion is deterministic
    #[test]
    fn snake_case_deterministic(name in pascal_case_strategy()) {
        let a = to_snake_case(&name);
        let b = to_snake_case(&name);
        prop_assert_eq!(a, b);
    }

    // 21. Rule name from type path is deterministic
    #[test]
    fn rule_name_from_path_deterministic(name in pascal_case_strategy()) {
        let path = format!("crate::grammar::{name}");
        let a = rule_name_from_type_path(&path);
        let b = rule_name_from_type_path(&path);
        prop_assert_eq!(a, b);
    }

    // 22. Sanitize is deterministic
    #[test]
    fn sanitize_deterministic(name in pascal_case_strategy()) {
        let a = sanitize_rule_name(&name);
        let b = sanitize_rule_name(&name);
        prop_assert_eq!(a, b);
    }

    // 23. Full pipeline (parse → extract ident → to_snake_case) is deterministic
    #[test]
    fn full_pipeline_deterministic(
        struct_name in pascal_case_strategy(),
        mod_name in snake_case_ident(),
    ) {
        let body = format!("    pub struct {struct_name} {{ pub v: i32, }}");
        let src = build_grammar_module(&mod_name, &body);
        let get_rule = || -> String {
            let parsed: ItemMod = parse_str(&src).unwrap();
            let items = &parsed.content.unwrap().1;
            if let Item::Struct(s) = &items[0] {
                to_snake_case(&s.ident.to_string())
            } else {
                panic!("expected struct");
            }
        };
        prop_assert_eq!(get_rule(), get_rule());
    }
}

// ---------------------------------------------------------------------------
// Tests: Rule name sanitization
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 24. Sanitized name contains only alphanumerics and underscores
    #[test]
    fn sanitized_name_valid_chars(name in pascal_case_strategy()) {
        let sanitized = sanitize_rule_name(&name);
        prop_assert!(
            sanitized.chars().all(|c| c.is_alphanumeric() || c == '_'),
            "invalid chars in: {sanitized}"
        );
    }

    // 25. Sanitized name has no consecutive underscores
    #[test]
    fn sanitized_no_consecutive_underscores(name in "[A-Za-z_]{1,10}[!@#%]{0,3}[A-Za-z]{0,5}") {
        let sanitized = sanitize_rule_name(&name);
        prop_assert!(
            !sanitized.contains("__"),
            "consecutive underscores in: {sanitized}"
        );
    }

    // 26. Sanitized name does not start or end with underscore
    #[test]
    fn sanitized_no_leading_trailing_underscore(name in "[_]{0,3}[A-Za-z]{1,8}[_]{0,3}") {
        let sanitized = sanitize_rule_name(&name);
        if !sanitized.is_empty() {
            prop_assert!(!sanitized.starts_with('_'), "leading underscore: {sanitized}");
            prop_assert!(!sanitized.ends_with('_'), "trailing underscore: {sanitized}");
        }
    }

    // 27. Sanitizing a valid snake_case name is idempotent
    #[test]
    fn sanitize_idempotent_on_valid(name in snake_case_ident()) {
        let once = sanitize_rule_name(&name);
        let twice = sanitize_rule_name(&once);
        prop_assert_eq!(once, twice);
    }

    // 28. Sanitizing then converting to snake_case produces valid rule name
    #[test]
    fn sanitize_then_snake_case_valid(name in pascal_case_strategy()) {
        let snake = to_snake_case(&name);
        let sanitized = sanitize_rule_name(&snake);
        prop_assert!(
            sanitized.chars().all(|c| c.is_lowercase() || c == '_' || c.is_ascii_digit()),
            "should be valid rule name: {sanitized}"
        );
    }
}

// ---------------------------------------------------------------------------
// Tests: Rule name for nested types
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 29. Nested struct in module produces valid rule name
    #[test]
    fn nested_struct_rule_name(
        outer in pascal_case_strategy(),
        inner in pascal_case_strategy(),
        mod_name in snake_case_ident(),
    ) {
        let body = format!(
            "    pub struct {outer} {{ pub child: {inner}, }}\n    pub struct {inner} {{ pub v: i32, }}"
        );
        let src = build_grammar_module(&mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        // Both structs should produce valid snake_case rule names
        for item in items {
            if let Item::Struct(s) = item {
                let rule = to_snake_case(&s.ident.to_string());
                prop_assert!(
                    rule.chars().all(|c| c.is_lowercase() || c == '_' || c.is_ascii_digit()),
                    "nested type rule should be valid: {rule}"
                );
            }
        }
    }

    // 30. Box-wrapped nested type: rule name comes from outer struct, not inner
    #[test]
    fn box_wrapped_nested_type_rule(
        outer in pascal_case_strategy(),
        inner in leaf_type_name(),
        mod_name in snake_case_ident(),
    ) {
        let body = format!("    pub struct {outer} {{ pub child: Box<{inner}>, }}");
        let src = build_grammar_module(&mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Struct(s) = &items[0] {
            let skip: HashSet<&str> = ["Box"].into_iter().collect();
            let field = s.fields.iter().next().unwrap();
            let filtered = filter_inner_type(&field.ty, &skip);
            // The struct's rule name is based on its own ident, not the field type
            let struct_rule = to_snake_case(&s.ident.to_string());
            let field_type_name = filtered.to_token_stream().to_string();
            // struct rule should match outer name
            prop_assert_eq!(struct_rule, to_snake_case(&outer));
            // field type should be the inner type (Box unwrapped)
            prop_assert_eq!(field_type_name, inner);
        }
    }

    // 31. Vec nested type: extracted inner type does not influence struct rule name
    #[test]
    fn vec_nested_type_rule(
        outer in pascal_case_strategy(),
        inner in leaf_type_name(),
        mod_name in snake_case_ident(),
    ) {
        let body = format!("    pub struct {outer} {{ pub items: Vec<{inner}>, }}");
        let src = build_grammar_module(&mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Struct(s) = &items[0] {
            let field = s.fields.iter().next().unwrap();
            let skip: HashSet<&str> = HashSet::new();
            let (extracted, was_extracted) = try_extract_inner_type(&field.ty, "Vec", &skip);
            prop_assert!(was_extracted);
            prop_assert_eq!(extracted.to_token_stream().to_string(), inner);
            // Struct rule name is independent
            prop_assert_eq!(to_snake_case(&s.ident.to_string()), to_snake_case(&outer));
        }
    }

    // 32. Option nested type preserves struct rule name
    #[test]
    fn option_nested_type_rule(
        outer in pascal_case_strategy(),
        inner in leaf_type_name(),
        mod_name in snake_case_ident(),
    ) {
        let body = format!("    pub struct {outer} {{ pub opt: Option<{inner}>, }}");
        let src = build_grammar_module(&mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Struct(s) = &items[0] {
            let field = s.fields.iter().next().unwrap();
            let skip: HashSet<&str> = HashSet::new();
            let (extracted, was_extracted) = try_extract_inner_type(&field.ty, "Option", &skip);
            prop_assert!(was_extracted);
            prop_assert_eq!(extracted.to_token_stream().to_string(), inner);
            prop_assert_eq!(to_snake_case(&s.ident.to_string()), to_snake_case(&outer));
        }
    }

    // 33. Deeply nested Box<Arc<T>> unwraps but doesn't change struct rule
    #[test]
    fn deep_nested_type_rule(
        outer in pascal_case_strategy(),
        inner in leaf_type_name(),
        mod_name in snake_case_ident(),
    ) {
        let body = format!("    pub struct {outer} {{ pub deep: Box<Arc<{inner}>>, }}");
        let src = build_grammar_module(&mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Struct(s) = &items[0] {
            let field = s.fields.iter().next().unwrap();
            let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
            let filtered = filter_inner_type(&field.ty, &skip);
            prop_assert_eq!(filtered.to_token_stream().to_string(), inner);
            prop_assert_eq!(to_snake_case(&s.ident.to_string()), to_snake_case(&outer));
        }
    }

    // 34. Enum with tuple variant: variant field type doesn't leak into rule name
    #[test]
    fn enum_tuple_variant_rule(
        enum_name in pascal_case_strategy(),
        mod_name in snake_case_ident(),
        inner in leaf_type_name(),
    ) {
        let body = format!("    pub enum {enum_name} {{ Leaf({inner}), Empty, }}");
        let src = build_grammar_module(&mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Enum(e) = &items[0] {
            let enum_rule = to_snake_case(&e.ident.to_string());
            let variant_rule = to_snake_case("Leaf");
            prop_assert_eq!(enum_rule, to_snake_case(&enum_name));
            prop_assert_eq!(variant_rule, "leaf");
        }
    }

    // 35. Rule names form a consistent mapping across a grammar module
    #[test]
    fn rule_name_mapping_consistent(mod_name in snake_case_ident()) {
        let body = [
            "    pub struct ExprAdd { pub left: i32, pub right: i32, }",
            "    pub struct ExprMul { pub left: i32, pub right: i32, }",
            "    pub enum Expr { Add, Mul, }",
        ].join("\n");
        let src = build_grammar_module(&mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        let mut name_map: HashMap<String, String> = HashMap::new();
        for item in items {
            let (ident, rule) = match item {
                Item::Struct(s) => (s.ident.to_string(), to_snake_case(&s.ident.to_string())),
                Item::Enum(e) => (e.ident.to_string(), to_snake_case(&e.ident.to_string())),
                _ => continue,
            };
            if let Some(existing) = name_map.get(&ident) {
                prop_assert_eq!(existing, &rule, "mapping should be consistent for {}", ident);
            }
            name_map.insert(ident, rule);
        }
        // All three types should be in the map
        prop_assert_eq!(name_map.len(), 3);
        prop_assert_eq!(&name_map["ExprAdd"], "expr_add");
        prop_assert_eq!(&name_map["ExprMul"], "expr_mul");
        prop_assert_eq!(&name_map["Expr"], "expr");
    }
}
