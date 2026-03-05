//! Property-based tests (v4) for DeriveInput parsing and field analysis
//! in the `adze-macro` crate.
//!
//! Exercises `syn::DeriveInput` parsing of generated struct/enum strings,
//! field count invariants, variant counting, type-analysis determinism,
//! token-stream output, field-name preservation, generic detection,
//! attribute preservation, and edge cases.

use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use syn::{DeriveInput, Fields, Type, parse_str};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn skip_set<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn is_parameterized(ty: &Type) -> bool {
    if let Type::Path(p) = ty
        && let Some(seg) = p.path.segments.last()
    {
        return matches!(seg.arguments, syn::PathArguments::AngleBracketed(_));
    }
    false
}

// ── Strategies ──────────────────────────────────────────────────────────────

fn ident_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{1,10}".prop_filter("no keywords", |s| {
        !matches!(
            s.as_str(),
            "type"
                | "fn"
                | "let"
                | "mut"
                | "ref"
                | "pub"
                | "mod"
                | "use"
                | "self"
                | "super"
                | "crate"
                | "struct"
                | "enum"
                | "impl"
                | "trait"
                | "where"
                | "for"
                | "loop"
                | "while"
                | "if"
                | "else"
                | "match"
                | "return"
                | "break"
                | "continue"
                | "as"
                | "in"
                | "move"
                | "box"
                | "dyn"
                | "async"
                | "await"
                | "try"
                | "yield"
                | "macro"
                | "const"
                | "static"
                | "unsafe"
                | "extern"
                | "do"
                | "gen"
                | "abstract"
                | "become"
                | "final"
                | "override"
                | "priv"
                | "typeof"
                | "unsized"
                | "virtual"
                | "true"
                | "false"
                | "union"
        )
    })
}

fn pascal_name() -> impl Strategy<Value = String> {
    ident_strategy().prop_map(|s| {
        let mut c = s.chars();
        match c.next() {
            Some(ch) => ch.to_uppercase().collect::<String>() + c.as_str(),
            None => "Ty".to_string(),
        }
    })
}

fn simple_type() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
        "String", "usize", "isize",
    ])
    .prop_map(|s| s.to_string())
}

fn wrapper_name() -> impl Strategy<Value = String> {
    prop::sample::select(vec!["Vec", "Option", "Box"]).prop_map(|s| s.to_string())
}

/// Build a struct string: `struct Name { field1: Type1, ... }`
fn struct_string(name: String, fields: Vec<(String, String)>) -> String {
    let body: Vec<String> = fields
        .iter()
        .map(|(f, t)| format!("    pub {f}: {t},"))
        .collect();
    format!("struct {} {{\n{}\n}}", name, body.join("\n"))
}

/// Build an enum string: `enum Name { V1(T1), V2(T2), ... }`
fn enum_string(name: String, variants: Vec<(String, String)>) -> String {
    let body: Vec<String> = variants
        .iter()
        .map(|(v, t)| format!("    {v}({t}),"))
        .collect();
    format!("enum {} {{\n{}\n}}", name, body.join("\n"))
}

/// Strategy for 1..=n named fields as (name, type) pairs.
fn fields_strategy(max: usize) -> impl Strategy<Value = Vec<(String, String)>> {
    prop::collection::vec((ident_strategy(), simple_type()), 1..=max)
}

/// Strategy for 1..=n enum variants as (VariantName, Type) pairs.
fn variants_strategy(max: usize) -> impl Strategy<Value = Vec<(String, String)>> {
    prop::collection::vec((pascal_name(), simple_type()), 1..=max)
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. DeriveInput parsing succeeds for valid struct strings (5 properties)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn parse_simple_struct_succeeds(
        name in pascal_name(),
        fields in fields_strategy(4),
    ) {
        let src = struct_string(name, fields);
        let result = parse_str::<DeriveInput>(&src);
        prop_assert!(result.is_ok(), "failed to parse: {src}");
    }

    #[test]
    fn parse_unit_struct_succeeds(name in pascal_name()) {
        let src = format!("struct {name};");
        prop_assert!(parse_str::<DeriveInput>(&src).is_ok());
    }

    #[test]
    fn parse_tuple_struct_succeeds(
        name in pascal_name(),
        ty in simple_type(),
    ) {
        let src = format!("struct {name}({ty});");
        prop_assert!(parse_str::<DeriveInput>(&src).is_ok());
    }

    #[test]
    fn parse_struct_with_generic_field_succeeds(
        name in pascal_name(),
        field in ident_strategy(),
        wrapper in wrapper_name(),
        inner in simple_type(),
    ) {
        let src = format!("struct {name} {{ pub {field}: {wrapper}<{inner}> }}");
        prop_assert!(parse_str::<DeriveInput>(&src).is_ok());
    }

    #[test]
    fn parse_struct_with_multiple_generics_succeeds(
        name in pascal_name(),
        f1 in ident_strategy(),
        f2 in ident_strategy(),
        w1 in wrapper_name(),
        w2 in wrapper_name(),
        t1 in simple_type(),
        t2 in simple_type(),
    ) {
        let src = format!(
            "struct {name} {{ pub {f1}: {w1}<{t1}>, pub {f2}: {w2}<{t2}> }}"
        );
        prop_assert!(parse_str::<DeriveInput>(&src).is_ok());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Struct field count matches (5 properties)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn struct_field_count_matches_one(
        name in pascal_name(),
        field in ident_strategy(),
        ty in simple_type(),
    ) {
        let src = format!("struct {name} {{ pub {field}: {ty} }}");
        let di = parse_str::<DeriveInput>(&src).unwrap();
        if let syn::Data::Struct(data) = &di.data {
            prop_assert_eq!(data.fields.len(), 1);
        } else {
            prop_assert!(false, "expected struct");
        }
    }

    #[test]
    fn struct_field_count_matches_vec(
        name in pascal_name(),
        fields in fields_strategy(6),
    ) {
        let count = fields.len();
        let src = struct_string(name, fields);
        let di = parse_str::<DeriveInput>(&src).unwrap();
        if let syn::Data::Struct(data) = &di.data {
            prop_assert_eq!(data.fields.len(), count);
        } else {
            prop_assert!(false, "expected struct");
        }
    }

    #[test]
    fn unit_struct_has_zero_fields(name in pascal_name()) {
        let src = format!("struct {name};");
        let di = parse_str::<DeriveInput>(&src).unwrap();
        if let syn::Data::Struct(data) = &di.data {
            prop_assert_eq!(data.fields.len(), 0);
        } else {
            prop_assert!(false, "expected struct");
        }
    }

    #[test]
    fn tuple_struct_field_count(
        name in pascal_name(),
        types in prop::collection::vec(simple_type(), 1..=4),
    ) {
        let count = types.len();
        let inner = types.join(", ");
        let src = format!("struct {name}({inner});");
        let di = parse_str::<DeriveInput>(&src).unwrap();
        if let syn::Data::Struct(data) = &di.data {
            prop_assert_eq!(data.fields.len(), count);
        } else {
            prop_assert!(false, "expected struct");
        }
    }

    #[test]
    fn struct_fields_are_named(
        name in pascal_name(),
        fields in fields_strategy(3),
    ) {
        let src = struct_string(name, fields);
        let di = parse_str::<DeriveInput>(&src).unwrap();
        if let syn::Data::Struct(data) = &di.data {
            prop_assert!(matches!(data.fields, Fields::Named(_)));
        } else {
            prop_assert!(false, "expected struct");
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Enum variant count matches (5 properties)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn enum_single_variant_count(
        name in pascal_name(),
        variant in pascal_name(),
        ty in simple_type(),
    ) {
        let src = format!("enum {name} {{ {variant}({ty}) }}");
        let di = parse_str::<DeriveInput>(&src).unwrap();
        if let syn::Data::Enum(data) = &di.data {
            prop_assert_eq!(data.variants.len(), 1);
        } else {
            prop_assert!(false, "expected enum");
        }
    }

    #[test]
    fn enum_variant_count_matches(
        name in pascal_name(),
        variants in variants_strategy(5),
    ) {
        let count = variants.len();
        let src = enum_string(name, variants);
        let di = parse_str::<DeriveInput>(&src).unwrap();
        if let syn::Data::Enum(data) = &di.data {
            prop_assert_eq!(data.variants.len(), count);
        } else {
            prop_assert!(false, "expected enum");
        }
    }

    #[test]
    fn enum_unit_variants_count(
        name in pascal_name(),
        vs in prop::collection::vec(pascal_name(), 1..=5),
    ) {
        let count = vs.len();
        let body = vs.iter().map(|v| format!("    {v},")).collect::<Vec<_>>().join("\n");
        let src = format!("enum {name} {{\n{body}\n}}");
        let di = parse_str::<DeriveInput>(&src).unwrap();
        if let syn::Data::Enum(data) = &di.data {
            prop_assert_eq!(data.variants.len(), count);
        } else {
            prop_assert!(false, "expected enum");
        }
    }

    #[test]
    fn enum_parses_as_derive_input(
        name in pascal_name(),
        variants in variants_strategy(3),
    ) {
        let src = enum_string(name, variants);
        prop_assert!(parse_str::<DeriveInput>(&src).is_ok());
    }

    #[test]
    fn enum_data_tag_is_enum(
        name in pascal_name(),
        variant in pascal_name(),
    ) {
        let src = format!("enum {name} {{ {variant} }}");
        let di = parse_str::<DeriveInput>(&src).unwrap();
        prop_assert!(matches!(di.data, syn::Data::Enum(_)));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Type analysis functions are deterministic (5 properties)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn try_extract_is_deterministic(
        wrapper in wrapper_name(),
        inner in simple_type(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let empty = skip_set(&[]);
        let (r1, e1) = try_extract_inner_type(&ty, &wrapper, &empty);
        let (r2, e2) = try_extract_inner_type(&ty, &wrapper, &empty);
        prop_assert_eq!(e1, e2);
        prop_assert_eq!(ty_str(&r1), ty_str(&r2));
    }

    #[test]
    fn filter_inner_type_is_deterministic(
        wrapper in wrapper_name(),
        inner in simple_type(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let wrapper_ref = wrapper.as_str();
        let names = [wrapper_ref];
        let skip = skip_set(&names);
        let a = filter_inner_type(&ty, &skip);
        let b = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&a), ty_str(&b));
    }

    #[test]
    fn wrap_leaf_type_is_deterministic(inner in simple_type()) {
        let ty: Type = parse_str(&inner).unwrap();
        let skip = skip_set(&[]);
        let a = wrap_leaf_type(&ty, &skip);
        let b = wrap_leaf_type(&ty, &skip);
        prop_assert_eq!(ty_str(&a), ty_str(&b));
    }

    #[test]
    fn is_parameterized_is_deterministic(
        wrapper in wrapper_name(),
        inner in simple_type(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        prop_assert_eq!(is_parameterized(&ty), is_parameterized(&ty));
    }

    #[test]
    fn extract_then_filter_deterministic(
        inner in simple_type(),
    ) {
        let src = format!("Box<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let skip = skip_set(&["Box"]);
        let (r1, _) = try_extract_inner_type(&ty, "Box", &skip);
        let f1 = filter_inner_type(&ty, &skip);
        let (r2, _) = try_extract_inner_type(&ty, "Box", &skip);
        let f2 = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&r1), ty_str(&r2));
        prop_assert_eq!(ty_str(&f1), ty_str(&f2));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. TokenStream output is non-empty (5 properties)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn struct_token_stream_nonempty(
        name in pascal_name(),
        fields in fields_strategy(3),
    ) {
        let src = struct_string(name, fields);
        let di = parse_str::<DeriveInput>(&src).unwrap();
        let ts = di.to_token_stream().to_string();
        prop_assert!(!ts.is_empty());
    }

    #[test]
    fn enum_token_stream_nonempty(
        name in pascal_name(),
        variants in variants_strategy(3),
    ) {
        let src = enum_string(name, variants);
        let di = parse_str::<DeriveInput>(&src).unwrap();
        let ts = di.to_token_stream().to_string();
        prop_assert!(!ts.is_empty());
    }

    #[test]
    fn wrap_leaf_output_nonempty(inner in simple_type()) {
        let ty: Type = parse_str(&inner).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
        let out = ty_str(&wrapped);
        prop_assert!(!out.is_empty());
    }

    #[test]
    fn filter_inner_output_nonempty(
        wrapper in wrapper_name(),
        inner in simple_type(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let wrapper_ref = wrapper.as_str();
        let names = [wrapper_ref];
        let skip = skip_set(&names);
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert!(!ty_str(&filtered).is_empty());
    }

    #[test]
    fn extract_inner_output_nonempty(
        wrapper in wrapper_name(),
        inner in simple_type(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let empty = skip_set(&[]);
        let (result, _) = try_extract_inner_type(&ty, &wrapper, &empty);
        prop_assert!(!ty_str(&result).is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Field names preserved through parsing (5 properties)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn struct_name_preserved(
        name in pascal_name(),
        fields in fields_strategy(2),
    ) {
        let src = struct_string(name.clone(), fields);
        let di = parse_str::<DeriveInput>(&src).unwrap();
        prop_assert_eq!(di.ident.to_string(), name);
    }

    #[test]
    fn field_names_preserved_in_order(
        name in pascal_name(),
        fields in fields_strategy(4),
    ) {
        let expected: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
        let src = struct_string(name, fields);
        let di = parse_str::<DeriveInput>(&src).unwrap();
        if let syn::Data::Struct(data) = &di.data {
            let actual: Vec<String> = data
                .fields
                .iter()
                .map(|f| f.ident.as_ref().unwrap().to_string())
                .collect();
            prop_assert_eq!(actual, expected);
        }
    }

    #[test]
    fn enum_name_preserved(
        name in pascal_name(),
        variant in pascal_name(),
    ) {
        let src = format!("enum {name} {{ {variant} }}");
        let di = parse_str::<DeriveInput>(&src).unwrap();
        prop_assert_eq!(di.ident.to_string(), name);
    }

    #[test]
    fn enum_variant_names_preserved(
        name in pascal_name(),
        variants in variants_strategy(4),
    ) {
        let expected: Vec<String> = variants.iter().map(|(v, _)| v.clone()).collect();
        let src = enum_string(name, variants);
        let di = parse_str::<DeriveInput>(&src).unwrap();
        if let syn::Data::Enum(data) = &di.data {
            let actual: Vec<String> = data
                .variants
                .iter()
                .map(|v| v.ident.to_string())
                .collect();
            prop_assert_eq!(actual, expected);
        }
    }

    #[test]
    fn field_type_strings_preserved(
        name in pascal_name(),
        fields in fields_strategy(3),
    ) {
        let expected: Vec<String> = fields.iter().map(|(_, t)| t.clone()).collect();
        let src = struct_string(name, fields);
        let di = parse_str::<DeriveInput>(&src).unwrap();
        if let syn::Data::Struct(data) = &di.data {
            let actual: Vec<String> = data
                .fields
                .iter()
                .map(|f| ty_str(&f.ty))
                .collect();
            prop_assert_eq!(actual, expected);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Generic parameters detected correctly (5 properties)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn parameterized_type_detected(
        wrapper in wrapper_name(),
        inner in simple_type(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        prop_assert!(is_parameterized(&ty));
    }

    #[test]
    fn simple_type_not_parameterized(inner in simple_type()) {
        let ty: Type = parse_str(&inner).unwrap();
        prop_assert!(!is_parameterized(&ty));
    }

    #[test]
    fn nested_wrapper_is_parameterized(
        outer in wrapper_name(),
        mid in wrapper_name(),
        inner in simple_type(),
    ) {
        let src = format!("{outer}<{mid}<{inner}>>");
        let ty: Type = parse_str(&src).unwrap();
        prop_assert!(is_parameterized(&ty));
    }

    #[test]
    fn extract_succeeds_for_parameterized(
        wrapper in wrapper_name(),
        inner in simple_type(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let empty = skip_set(&[]);
        let (_, extracted) = try_extract_inner_type(&ty, &wrapper, &empty);
        prop_assert!(extracted);
    }

    #[test]
    fn extract_fails_for_wrong_wrapper(
        wrapper in wrapper_name(),
        inner in simple_type(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let empty = skip_set(&[]);
        let (_, extracted) = try_extract_inner_type(&ty, "NonExistent", &empty);
        prop_assert!(!extracted);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Attribute count preserved (5 properties)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn struct_no_attrs_has_empty_attrs(
        name in pascal_name(),
        field in ident_strategy(),
        ty in simple_type(),
    ) {
        let src = format!("struct {name} {{ pub {field}: {ty} }}");
        let di = parse_str::<DeriveInput>(&src).unwrap();
        prop_assert!(di.attrs.is_empty());
    }

    #[test]
    fn struct_single_derive_attr_preserved(
        name in pascal_name(),
        field in ident_strategy(),
        ty in simple_type(),
    ) {
        let src = format!("#[derive(Debug)]\nstruct {name} {{ pub {field}: {ty} }}");
        let di = parse_str::<DeriveInput>(&src).unwrap();
        prop_assert_eq!(di.attrs.len(), 1);
    }

    #[test]
    fn struct_two_attrs_preserved(
        name in pascal_name(),
        field in ident_strategy(),
        ty in simple_type(),
    ) {
        let src = format!(
            "#[derive(Debug)]\n#[derive(Clone)]\nstruct {name} {{ pub {field}: {ty} }}"
        );
        let di = parse_str::<DeriveInput>(&src).unwrap();
        prop_assert_eq!(di.attrs.len(), 2);
    }

    #[test]
    fn enum_no_attrs_has_empty_attrs(
        name in pascal_name(),
        variant in pascal_name(),
    ) {
        let src = format!("enum {name} {{ {variant} }}");
        let di = parse_str::<DeriveInput>(&src).unwrap();
        prop_assert!(di.attrs.is_empty());
    }

    #[test]
    fn enum_single_derive_attr_preserved(
        name in pascal_name(),
        variant in pascal_name(),
    ) {
        let src = format!("#[derive(Debug)]\nenum {name} {{ {variant} }}");
        let di = parse_str::<DeriveInput>(&src).unwrap();
        prop_assert_eq!(di.attrs.len(), 1);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Edge cases (6 properties)
// ═══════════════════════════════════════════════════════════════════════════

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn filter_noop_when_skip_empty(
        wrapper in wrapper_name(),
        inner in simple_type(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let empty = skip_set(&[]);
        let filtered = filter_inner_type(&ty, &empty);
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty));
    }

    #[test]
    fn filter_unwraps_matching_wrapper(
        inner in simple_type(),
    ) {
        let src = format!("Box<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let skip = skip_set(&["Box"]);
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), inner);
    }

    #[test]
    fn wrap_leaf_wraps_simple_type(inner in simple_type()) {
        let ty: Type = parse_str(&inner).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
        let out = ty_str(&wrapped);
        prop_assert!(out.contains("WithLeaf"));
        prop_assert!(out.contains(&inner));
    }

    #[test]
    fn wrap_leaf_skips_wrapper_wraps_inner(
        inner in simple_type(),
    ) {
        let src = format!("Vec<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let skip = skip_set(&["Vec"]);
        let wrapped = wrap_leaf_type(&ty, &skip);
        let out = ty_str(&wrapped);
        prop_assert!(out.starts_with("Vec"));
        prop_assert!(out.contains("WithLeaf"));
    }

    #[test]
    fn extract_inner_type_matches_filter(
        inner in simple_type(),
    ) {
        let src = format!("Box<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let skip = skip_set(&["Box"]);
        let (extracted, ok) = try_extract_inner_type(&ty, "Box", &skip);
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&extracted), ty_str(&filtered));
    }

    #[test]
    fn struct_generics_empty_for_non_generic(
        name in pascal_name(),
        field in ident_strategy(),
        ty in simple_type(),
    ) {
        let src = format!("struct {name} {{ pub {field}: {ty} }}");
        let di = parse_str::<DeriveInput>(&src).unwrap();
        prop_assert!(di.generics.params.is_empty());
    }
}
