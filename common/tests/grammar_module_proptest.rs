#![allow(clippy::needless_range_loop)]

//! Property-based tests for grammar module handling in adze-common.
//!
//! Tests grammar module detection, content processing, type extraction,
//! grammar name derivation, and handling of various item types within
//! modules annotated with `#[adze::grammar]`.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Attribute, Item, ItemMod, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Valid Rust identifiers for module/grammar names.
fn ident_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,12}")
        .unwrap()
        .prop_filter("must be valid ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

/// Grammar name strings (used inside `#[adze::grammar("...")]`).
fn grammar_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,20}")
        .unwrap()
        .prop_filter("must be valid grammar name", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

/// Simple leaf type names.
fn leaf_type_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize",
        ][..],
    )
}

/// Type names usable for struct fields.
fn field_type_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["i32", "u32", "f64", "bool", "String", "usize", "u8", "i64"][..])
}

/// Item kind selector for module content generation.
#[derive(Debug, Clone, Copy)]
enum ItemKind {
    Struct,
    Enum,
    Fn,
    Const,
    TypeAlias,
}

fn item_kind_strategy() -> impl Strategy<Value = ItemKind> {
    prop::sample::select(
        &[
            ItemKind::Struct,
            ItemKind::Enum,
            ItemKind::Fn,
            ItemKind::Const,
            ItemKind::TypeAlias,
        ][..],
    )
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a grammar module source string.
fn build_grammar_module(grammar_name: &str, mod_name: &str, body: &str) -> String {
    format!(
        r#"#[adze::grammar("{grammar_name}")]
mod {mod_name} {{
{body}
}}"#
    )
}

/// Build a simple struct item string.
fn build_struct_item(name: &str, field_ty: &str) -> String {
    format!("    pub struct {name} {{ pub value: {field_ty}, }}")
}

/// Build a simple enum item string.
fn build_enum_item(name: &str) -> String {
    format!("    pub enum {name} {{ A, B, C, }}")
}

/// Build a simple fn item string.
fn build_fn_item(name: &str) -> String {
    format!("    fn {name}() {{}}")
}

/// Build a simple const item string.
fn build_const_item(name: &str) -> String {
    format!("    const {name}: i32 = 0;")
}

/// Build a type alias item string.
fn build_type_alias_item(name: &str, ty: &str) -> String {
    format!("    type {name} = {ty};")
}

/// Check whether an attribute path matches `adze::grammar`.
fn is_grammar_attr(attr: &Attribute) -> bool {
    let path = attr.path();
    let segments: Vec<_> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    segments == ["adze", "grammar"]
}

/// Extract grammar name from a `#[adze::grammar("name")]` attribute.
fn extract_grammar_name(attr: &Attribute) -> Option<String> {
    if !is_grammar_attr(attr) {
        return None;
    }
    let tokens = attr.meta.to_token_stream().to_string();
    // Look for the string literal in the attribute
    if let Some(start) = tokens.find('"')
        && let Some(end) = tokens[start + 1..].find('"')
    {
        return Some(tokens[start + 1..start + 1 + end].to_string());
    }
    None
}

/// Capitalize first letter of a string (for PascalCase derivation).
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}

/// Derive a PascalCase grammar type name from a snake_case module name.
fn derive_grammar_type_name(mod_name: &str) -> String {
    mod_name.split('_').map(capitalize_first).collect()
}

// ---------------------------------------------------------------------------
// Tests: Grammar module detection with various names
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 1. Grammar module with random name parses as ItemMod
    #[test]
    fn grammar_module_parses_as_item_mod(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
    ) {
        let src = build_grammar_module(&grammar_name, &mod_name, "");
        let parsed: ItemMod = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.ident.to_string(), mod_name);
    }

    // 2. Grammar attribute is detected on module
    #[test]
    fn grammar_attr_detected(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
    ) {
        let src = build_grammar_module(&grammar_name, &mod_name, "");
        let parsed: ItemMod = parse_str(&src).unwrap();
        let has_grammar = parsed.attrs.iter().any(is_grammar_attr);
        prop_assert!(has_grammar, "grammar attribute should be detected");
    }

    // 3. Grammar name extracted from attribute matches input
    #[test]
    fn grammar_name_extracted_matches(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
    ) {
        let src = build_grammar_module(&grammar_name, &mod_name, "");
        let parsed: ItemMod = parse_str(&src).unwrap();
        let extracted = parsed.attrs.iter()
            .find_map(extract_grammar_name)
            .expect("should find grammar name");
        prop_assert_eq!(extracted, grammar_name);
    }

    // 4. Module without grammar attribute is not detected
    #[test]
    fn non_grammar_module_not_detected(mod_name in ident_strategy()) {
        let src = format!("mod {mod_name} {{}}");
        let parsed: ItemMod = parse_str(&src).unwrap();
        let has_grammar = parsed.attrs.iter().any(is_grammar_attr);
        prop_assert!(!has_grammar, "non-grammar module should not have grammar attr");
    }

    // 5. Grammar module ident is preserved exactly
    #[test]
    fn grammar_module_ident_preserved(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
    ) {
        let src = build_grammar_module(&grammar_name, &mod_name, "");
        let parsed: ItemMod = parse_str(&src).unwrap();
        let ident_str = parsed.ident.to_string();
        prop_assert_eq!(ident_str, mod_name);
    }
}

// ---------------------------------------------------------------------------
// Tests: Module content processing
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 6. Empty grammar module has zero items
    #[test]
    fn empty_module_has_no_items(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
    ) {
        let src = build_grammar_module(&grammar_name, &mod_name, "");
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        prop_assert!(items.is_empty());
    }

    // 7. Module with one struct has exactly one item
    #[test]
    fn module_with_one_struct(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
        field_ty in field_type_name(),
    ) {
        let body = build_struct_item("MyStruct", field_ty);
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        prop_assert_eq!(items.len(), 1);
    }

    // 8. Module with one enum has exactly one item
    #[test]
    fn module_with_one_enum(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
    ) {
        let body = build_enum_item("MyEnum");
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        prop_assert_eq!(items.len(), 1);
    }

    // 9. Module with a function item parses correctly
    #[test]
    fn module_with_fn_item(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
        fn_name in ident_strategy(),
    ) {
        let body = build_fn_item(&fn_name);
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        prop_assert_eq!(items.len(), 1);
        match &items[0] {
            Item::Fn(f) => prop_assert_eq!(f.sig.ident.to_string(), fn_name),
            other => prop_assert!(false, "expected Fn item, got: {:?}", other),
        }
    }

    // 10. Module with a const item parses correctly
    #[test]
    fn module_with_const_item(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
    ) {
        let body = build_const_item("MY_CONST");
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        prop_assert_eq!(items.len(), 1);
        match &items[0] {
            Item::Const(_) => {}
            other => prop_assert!(false, "expected Const item, got: {:?}", other),
        }
    }

    // 11. Module with type alias parses correctly
    #[test]
    fn module_with_type_alias(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
        ty in leaf_type_name(),
    ) {
        let body = build_type_alias_item("MyType", ty);
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        prop_assert_eq!(items.len(), 1);
        match &items[0] {
            Item::Type(_) => {}
            other => prop_assert!(false, "expected Type alias, got: {:?}", other),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests: Type extraction from modules with random numbers of items
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 12. Module item count matches number of structs inserted
    #[test]
    fn module_item_count_matches_struct_count(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
        count in 1usize..=8,
    ) {
        let body: String = (0..count)
            .map(|i| build_struct_item(&format!("S{i}"), "i32"))
            .collect::<Vec<_>>()
            .join("\n");
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        prop_assert_eq!(items.len(), count);
    }

    // 13. All structs in module are detected as Item::Struct
    #[test]
    fn all_structs_detected(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
        count in 1usize..=6,
    ) {
        let body: String = (0..count)
            .map(|i| build_struct_item(&format!("S{i}"), "u32"))
            .collect::<Vec<_>>()
            .join("\n");
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        let struct_count = items.iter()
            .filter(|item| matches!(item, Item::Struct(_)))
            .count();
        prop_assert_eq!(struct_count, count);
    }

    // 14. Struct names are preserved in order
    #[test]
    fn struct_names_preserved_in_order(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
        count in 1usize..=6,
    ) {
        let names: Vec<String> = (0..count).map(|i| format!("Item{i}")).collect();
        let body: String = names.iter()
            .map(|n| build_struct_item(n, "i32"))
            .collect::<Vec<_>>()
            .join("\n");
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        for i in 0..count {
            if let Item::Struct(s) = &items[i] {
                prop_assert_eq!(s.ident.to_string(), names[i].as_str());
            } else {
                prop_assert!(false, "expected struct at index {i}");
            }
        }
    }

    // 15. Extracting field types from struct items in a module
    #[test]
    fn extract_field_types_from_module_structs(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
        field_ty in field_type_name(),
    ) {
        let body = build_struct_item("MyNode", field_ty);
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Struct(s) = &items[0] {
            let first_field = s.fields.iter().next().unwrap();
            let ty_str = first_field.ty.to_token_stream().to_string();
            prop_assert_eq!(ty_str, field_ty);
        } else {
            prop_assert!(false, "expected struct item");
        }
    }

    // 16. Type extraction works on types found inside module struct fields
    #[test]
    fn try_extract_on_module_field_type(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
        inner in leaf_type_name(),
    ) {
        let body = format!("    pub struct Node {{ pub items: Vec<{inner}>, }}");
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Struct(s) = &items[0] {
            let field = s.fields.iter().next().unwrap();
            let skip: HashSet<&str> = HashSet::new();
            let (result, extracted) = try_extract_inner_type(&field.ty, "Vec", &skip);
            prop_assert!(extracted);
            prop_assert_eq!(result.to_token_stream().to_string(), inner);
        }
    }

    // 17. filter_inner_type works on Box-wrapped types in module structs
    #[test]
    fn filter_on_module_field_type(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
        inner in leaf_type_name(),
    ) {
        let body = format!("    pub struct Node {{ pub child: Box<{inner}>, }}");
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Struct(s) = &items[0] {
            let field = s.fields.iter().next().unwrap();
            let skip: HashSet<&str> = ["Box"].into_iter().collect();
            let filtered = filter_inner_type(&field.ty, &skip);
            prop_assert_eq!(filtered.to_token_stream().to_string(), inner);
        }
    }

    // 18. wrap_leaf_type works on types extracted from module items
    #[test]
    fn wrap_on_module_field_type(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
        inner in leaf_type_name(),
    ) {
        let body = build_struct_item("Node", inner);
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Struct(s) = &items[0] {
            let field = s.fields.iter().next().unwrap();
            let skip: HashSet<&str> = HashSet::new();
            let wrapped = wrap_leaf_type(&field.ty, &skip);
            let s = wrapped.to_token_stream().to_string();
            prop_assert!(s.contains("adze :: WithLeaf"), "expected WithLeaf: {s}");
        }
    }
}

// ---------------------------------------------------------------------------
// Tests: Grammar name derivation from module name
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 19. Derived PascalCase name is non-empty for any valid ident
    #[test]
    fn derived_name_non_empty(mod_name in ident_strategy()) {
        let derived = derive_grammar_type_name(&mod_name);
        prop_assert!(!derived.is_empty());
    }

    // 20. Derived name starts with uppercase letter
    #[test]
    fn derived_name_starts_uppercase(mod_name in ident_strategy()) {
        let derived = derive_grammar_type_name(&mod_name);
        let first = derived.chars().next().unwrap();
        prop_assert!(first.is_uppercase(), "expected uppercase start: {derived}");
    }

    // 21. Derived name contains no underscores
    #[test]
    fn derived_name_no_underscores(mod_name in ident_strategy()) {
        let derived = derive_grammar_type_name(&mod_name);
        prop_assert!(!derived.contains('_'), "should have no underscores: {derived}");
    }

    // 22. Single-segment module names produce capitalized version
    #[test]
    fn single_segment_capitalized(
        name in prop::string::string_regex("[a-z]{1,10}").unwrap()
            .prop_filter("valid ident", |s| !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok())
    ) {
        let derived = derive_grammar_type_name(&name);
        let expected = capitalize_first(&name);
        prop_assert_eq!(derived, expected);
    }

    // 23. Grammar name from attribute and module name are independent
    #[test]
    fn grammar_name_independent_of_module_name(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
    ) {
        let src = build_grammar_module(&grammar_name, &mod_name, "");
        let parsed: ItemMod = parse_str(&src).unwrap();
        let attr_name = parsed.attrs.iter()
            .find_map(extract_grammar_name)
            .unwrap();
        let mod_ident = parsed.ident.to_string();
        // They can be equal by coincidence, but both should be their respective inputs
        prop_assert_eq!(attr_name, grammar_name);
        prop_assert_eq!(mod_ident, mod_name);
    }

    // 24. Derived name is deterministic
    #[test]
    fn derived_name_deterministic(mod_name in ident_strategy()) {
        let a = derive_grammar_type_name(&mod_name);
        let b = derive_grammar_type_name(&mod_name);
        prop_assert_eq!(a, b);
    }
}

// ---------------------------------------------------------------------------
// Tests: Module with various item types (struct, enum, fn, const)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 25. Mixed item types: struct + enum counted correctly
    #[test]
    fn mixed_struct_and_enum(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
        n_structs in 1usize..=4,
        n_enums in 1usize..=4,
    ) {
        let mut body_parts = Vec::new();
        for i in 0..n_structs {
            body_parts.push(build_struct_item(&format!("S{i}"), "i32"));
        }
        for i in 0..n_enums {
            body_parts.push(build_enum_item(&format!("E{i}")));
        }
        let body = body_parts.join("\n");
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        prop_assert_eq!(items.len(), n_structs + n_enums);
    }

    // 26. Mixed item types: categorization is correct
    #[test]
    fn mixed_items_categorized(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
    ) {
        let body = [
            build_struct_item("MyStruct", "i32"),
            build_enum_item("MyEnum"),
            build_fn_item("my_fn"),
            build_const_item("MY_CONST"),
        ].join("\n");
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        prop_assert_eq!(items.len(), 4);
        prop_assert!(matches!(&items[0], Item::Struct(_)));
        prop_assert!(matches!(&items[1], Item::Enum(_)));
        prop_assert!(matches!(&items[2], Item::Fn(_)));
        prop_assert!(matches!(&items[3], Item::Const(_)));
    }

    // 27. Module parsing is deterministic
    #[test]
    fn module_parsing_deterministic(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
    ) {
        let body = build_struct_item("Node", "i32");
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let a: ItemMod = parse_str(&src).unwrap();
        let b: ItemMod = parse_str(&src).unwrap();
        prop_assert_eq!(a.ident.to_string(), b.ident.to_string());
        prop_assert_eq!(
            a.content.unwrap().1.len(),
            b.content.unwrap().1.len(),
        );
    }

    // 28. Random item kind generates parseable module
    #[test]
    fn random_item_kind_parseable(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
        kind in item_kind_strategy(),
    ) {
        let body = match kind {
            ItemKind::Struct => build_struct_item("Thing", "u32"),
            ItemKind::Enum => build_enum_item("Thing"),
            ItemKind::Fn => build_fn_item("thing"),
            ItemKind::Const => build_const_item("THING"),
            ItemKind::TypeAlias => build_type_alias_item("Thing", "i32"),
        };
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.content.unwrap().1.len(), 1);
    }

    // 29. Multiple random item kinds in one module
    #[test]
    fn multiple_random_item_kinds(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
        kinds in prop::collection::vec(item_kind_strategy(), 1..=6),
    ) {
        let body: String = kinds.iter().enumerate().map(|(i, kind)| {
            match kind {
                ItemKind::Struct => build_struct_item(&format!("S{i}"), "i32"),
                ItemKind::Enum => build_enum_item(&format!("E{i}"),),
                ItemKind::Fn => build_fn_item(&format!("f{i}")),
                ItemKind::Const => build_const_item(&format!("C{i}")),
                ItemKind::TypeAlias => build_type_alias_item(&format!("T{i}"), "u8"),
            }
        }).collect::<Vec<_>>().join("\n");
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        prop_assert_eq!(items.len(), kinds.len());
    }

    // 30. Enum variants are preserved in module
    #[test]
    fn enum_variants_preserved(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
    ) {
        let body = "    pub enum MyEnum { Alpha, Beta, Gamma, }";
        let src = build_grammar_module(&grammar_name, &mod_name, body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Enum(e) = &items[0] {
            prop_assert_eq!(e.variants.len(), 3);
            prop_assert_eq!(e.variants[0].ident.to_string(), "Alpha");
            prop_assert_eq!(e.variants[1].ident.to_string(), "Beta");
            prop_assert_eq!(e.variants[2].ident.to_string(), "Gamma");
        } else {
            prop_assert!(false, "expected enum");
        }
    }

    // 31. Struct field names are preserved
    #[test]
    fn struct_field_names_preserved(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
        field_ty in field_type_name(),
    ) {
        let body = format!("    pub struct Node {{ pub first: {field_ty}, pub second: {field_ty}, }}");
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Struct(s) = &items[0] {
            let field_names: Vec<_> = s.fields.iter()
                .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
                .collect();
            prop_assert_eq!(field_names, vec!["first", "second"]);
        } else {
            prop_assert!(false, "expected struct");
        }
    }
}

// ---------------------------------------------------------------------------
// Tests: Additional cross-cutting properties
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 32. Module with NameValueExpr-style attribute content parses
    #[test]
    fn module_with_nve_attr_content(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
    ) {
        // Verify that leaf-style attributes inside module items parse correctly
        let body = r#"    pub struct Node {
        #[adze::leaf(pattern = r"\d+")]
        pub value: String,
    }"#;
        let src = build_grammar_module(&grammar_name, &mod_name, body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Struct(s) = &items[0] {
            let field = s.fields.iter().next().unwrap();
            let has_leaf = field.attrs.iter().any(|a| {
                let segs: Vec<_> = a.path().segments.iter().map(|s| s.ident.to_string()).collect();
                segs == ["adze", "leaf"]
            });
            prop_assert!(has_leaf, "should have adze::leaf attribute");
        }
    }

    // 33. Grammar module content survives token round-trip
    #[test]
    fn module_token_roundtrip(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
    ) {
        let body = build_struct_item("RoundTrip", "i32");
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let tokens = parsed.to_token_stream().to_string();
        let reparsed: ItemMod = parse_str(&tokens).unwrap();
        prop_assert_eq!(parsed.ident.to_string(), reparsed.ident.to_string());
        prop_assert_eq!(
            parsed.content.unwrap().1.len(),
            reparsed.content.unwrap().1.len(),
        );
    }

    // 34. Grammar module with Option<T> field enables type extraction
    #[test]
    fn optional_field_extraction_in_module(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
        inner in leaf_type_name(),
    ) {
        let body = format!("    pub struct Node {{ pub opt: Option<{inner}>, }}");
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        if let Item::Struct(s) = &items[0] {
            let field = s.fields.iter().next().unwrap();
            let skip: HashSet<&str> = HashSet::new();
            let (result, extracted) = try_extract_inner_type(&field.ty, "Option", &skip);
            prop_assert!(extracted);
            prop_assert_eq!(result.to_token_stream().to_string(), inner);
        }
    }

    // 35. Module attribute count is exactly one for grammar modules
    #[test]
    fn grammar_module_has_one_grammar_attr(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
    ) {
        let src = build_grammar_module(&grammar_name, &mod_name, "");
        let parsed: ItemMod = parse_str(&src).unwrap();
        let grammar_attr_count = parsed.attrs.iter()
            .filter(|a| is_grammar_attr(a))
            .count();
        prop_assert_eq!(grammar_attr_count, 1);
    }
}
