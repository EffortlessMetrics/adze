#![allow(clippy::needless_range_loop)]

//! Property-based tests for derive macro expansion in adze-common.
//!
//! Covers: struct type expansion, enum type expansion, generics preservation,
//! lifetime parameters, expansion determinism, empty/many-field structs,
//! and validity of expansion output as parseable Rust.

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{ItemEnum, ItemStruct, Type, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn leaf_type() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize",
        ][..],
    )
}

fn container() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Option", "Vec", "Box"][..])
}

fn skip_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Box", "Arc", "Rc", "Cell"][..])
}

fn pascal_ident() -> impl Strategy<Value = String> {
    prop::string::string_regex("[A-Z][a-zA-Z0-9]{0,12}")
        .unwrap()
        .prop_filter("must be valid ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

fn snake_ident() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,12}")
        .unwrap()
        .prop_filter("must be valid ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

fn lifetime_name() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z]{1,4}")
        .unwrap()
        .prop_filter("must be valid lifetime", |s| {
            !s.is_empty() && syn::parse_str::<syn::Lifetime>(&format!("'{s}")).is_ok()
        })
}

fn generic_param_name() -> impl Strategy<Value = String> {
    prop::sample::select(&["T", "U", "V", "W", "A", "B"][..]).prop_map(String::from)
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn skip_set<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

// ===========================================================================
// 1. Derive expansion for struct types
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Struct with a single field: field name and type survive derive-like expansion.
    #[test]
    fn struct_single_field_expansion(
        name in pascal_ident(),
        field in snake_ident(),
        ty in leaf_type(),
    ) {
        let src = format!("pub struct {name} {{ pub {field}: {ty}, }}");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.ident.to_string(), name);
        let f = parsed.fields.iter().next().unwrap();
        prop_assert_eq!(f.ident.as_ref().unwrap().to_string(), field);
        prop_assert_eq!(f.ty.to_token_stream().to_string(), ty);
    }

    /// Struct field types wrapped in a container are extractable after parsing.
    #[test]
    fn struct_container_field_extractable(
        name in pascal_ident(),
        ctr in container(),
        inner in leaf_type(),
    ) {
        let src = format!("pub struct {name} {{ pub val: {ctr}<{inner}>, }}");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let field_ty = &parsed.fields.iter().next().unwrap().ty;
        let (result, ok) = try_extract_inner_type(field_ty, ctr, &skip_set(&[]));
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&result), inner);
    }

    /// Struct field types can be wrapped with WithLeaf after extraction.
    #[test]
    fn struct_field_wrap_after_extract(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!("pub struct {name} {{ pub val: Option<{inner}>, }}");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let field_ty = &parsed.fields.iter().next().unwrap().ty;
        let (extracted, ok) = try_extract_inner_type(field_ty, "Option", &skip_set(&[]));
        prop_assert!(ok);
        let wrapped = wrap_leaf_type(&extracted, &skip_set(&[]));
        prop_assert_eq!(ty_str(&wrapped), format!("adze :: WithLeaf < {inner} >"));
    }

    /// Struct field types can be filtered then wrapped in a pipeline.
    #[test]
    fn struct_field_filter_wrap_pipeline(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!("pub struct {name} {{ pub val: Box<{inner}>, }}");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let field_ty = &parsed.fields.iter().next().unwrap().ty;
        let filtered = filter_inner_type(field_ty, &skip_set(&["Box"]));
        prop_assert_eq!(ty_str(&filtered), inner);
        let wrapped = wrap_leaf_type(&filtered, &skip_set(&[]));
        prop_assert_eq!(ty_str(&wrapped), format!("adze :: WithLeaf < {inner} >"));
    }
}

// ===========================================================================
// 2. Derive expansion for enum types
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Enum with unit variants: all variant names preserved after parsing.
    #[test]
    fn enum_unit_variants_preserved(
        name in pascal_ident(),
        count in 1usize..=8,
    ) {
        let variants: Vec<String> = (0..count).map(|i| format!("V{i}")).collect();
        let var_strs: Vec<String> = variants.iter().map(|v| format!("    {v},")).collect();
        let src = format!("pub enum {name} {{\n{}\n}}", var_strs.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.variants.len(), count);
        for i in 0..count {
            prop_assert_eq!(parsed.variants[i].ident.to_string(), variants[i].as_str());
        }
    }

    /// Enum tuple variant inner type is extractable via try_extract_inner_type.
    #[test]
    fn enum_tuple_variant_extractable(
        name in pascal_ident(),
        ctr in container(),
        inner in leaf_type(),
    ) {
        let src = format!("pub enum {name} {{ A({ctr}<{inner}>), }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        let field_ty = &parsed.variants[0].fields.iter().next().unwrap().ty;
        let (result, ok) = try_extract_inner_type(field_ty, ctr, &skip_set(&[]));
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&result), inner);
    }

    /// Enum with mixed variant styles: variant count matches.
    #[test]
    fn enum_mixed_variants_count(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!(
            "pub enum {name} {{ Unit, Tuple({inner}), Struct {{ val: {inner} }}, }}"
        );
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.variants.len(), 3);
        prop_assert_eq!(parsed.variants[0].ident.to_string(), "Unit");
        prop_assert_eq!(parsed.variants[1].ident.to_string(), "Tuple");
        prop_assert_eq!(parsed.variants[2].ident.to_string(), "Struct");
    }

    /// Enum struct variant fields can be wrapped with WithLeaf.
    #[test]
    fn enum_struct_variant_wrappable(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!("pub enum {name} {{ V {{ val: {inner} }}, }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        let field_ty = &parsed.variants[0].fields.iter().next().unwrap().ty;
        let wrapped = wrap_leaf_type(field_ty, &skip_set(&[]));
        prop_assert_eq!(ty_str(&wrapped), format!("adze :: WithLeaf < {inner} >"));
    }
}

// ===========================================================================
// 3. Derive expansion preserves generics
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Struct with a generic parameter preserves the generic param.
    #[test]
    fn struct_generic_param_preserved(
        name in pascal_ident(),
        param in generic_param_name(),
    ) {
        let src = format!("pub struct {name}<{param}> {{ pub val: {param}, }}");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.generics.params.len(), 1);
        let field_ty = ty_str(&parsed.fields.iter().next().unwrap().ty);
        prop_assert_eq!(field_ty, param);
    }

    /// Struct with multiple generic parameters preserves all params.
    #[test]
    fn struct_multiple_generics_preserved(
        name in pascal_ident(),
    ) {
        let src = format!("pub struct {name}<T, U> {{ pub a: T, pub b: U, }}");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.generics.params.len(), 2);
        prop_assert_eq!(parsed.fields.len(), 2);
    }

    /// Enum with generic parameter preserves the generic in variants.
    #[test]
    fn enum_generic_param_preserved(
        name in pascal_ident(),
        param in generic_param_name(),
    ) {
        let src = format!("pub enum {name}<{param}> {{ Some({param}), None, }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.generics.params.len(), 1);
        let field = parsed.variants[0].fields.iter().next().unwrap();
        prop_assert_eq!(ty_str(&field.ty), param);
    }

    /// Generic type parameter inside a container is extractable.
    #[test]
    fn generic_inside_container_extractable(
        ctr in container(),
        param in generic_param_name(),
    ) {
        let ty: Type = parse_str(&format!("{ctr}<{param}>")).unwrap();
        let (result, ok) = try_extract_inner_type(&ty, ctr, &skip_set(&[]));
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&result), param);
    }
}

// ===========================================================================
// 4. Derive expansion with lifetime parameters
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Struct with a lifetime parameter parses and preserves the lifetime.
    #[test]
    fn struct_lifetime_preserved(
        name in pascal_ident(),
        lt in lifetime_name(),
    ) {
        let src = format!("pub struct {name}<'{lt}> {{ pub val: &'{lt} str, }}");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.generics.params.len(), 1);
        let field_ty = ty_str(&parsed.fields.iter().next().unwrap().ty);
        prop_assert!(field_ty.contains(&lt));
    }

    /// Enum with a lifetime parameter parses correctly.
    #[test]
    fn enum_lifetime_preserved(
        name in pascal_ident(),
        lt in lifetime_name(),
    ) {
        let src = format!("pub enum {name}<'{lt}> {{ Borrowed(&'{lt} str), Owned(String), }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.generics.params.len(), 1);
        prop_assert_eq!(parsed.variants.len(), 2);
    }

    /// Struct with lifetime + type generic preserves both.
    #[test]
    fn struct_lifetime_and_generic_preserved(
        name in pascal_ident(),
        lt in lifetime_name(),
        param in generic_param_name(),
    ) {
        let src = format!(
            "pub struct {name}<'{lt}, {param}> {{ pub r: &'{lt} str, pub v: {param}, }}"
        );
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.generics.params.len(), 2);
        prop_assert_eq!(parsed.fields.len(), 2);
    }
}

// ===========================================================================
// 5. Derive expansion determinism
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Parsing the same struct source twice yields identical token output.
    #[test]
    fn struct_parse_deterministic(
        name in pascal_ident(),
        ty in leaf_type(),
    ) {
        let src = format!("pub struct {name} {{ pub v: {ty}, }}");
        let a: ItemStruct = parse_str(&src).unwrap();
        let b: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(a.to_token_stream().to_string(), b.to_token_stream().to_string());
    }

    /// Parsing the same enum source twice yields identical token output.
    #[test]
    fn enum_parse_deterministic(
        name in pascal_ident(),
        ty in leaf_type(),
    ) {
        let src = format!("pub enum {name} {{ A({ty}), B, }}");
        let a: ItemEnum = parse_str(&src).unwrap();
        let b: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(a.to_token_stream().to_string(), b.to_token_stream().to_string());
    }

    /// Extract → filter → wrap pipeline is deterministic on struct fields.
    #[test]
    fn pipeline_on_struct_field_deterministic(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!("pub struct {name} {{ pub v: Option<Box<{inner}>>, }}");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let field_ty = &parsed.fields.iter().next().unwrap().ty;
        let run = || {
            let (after, _) = try_extract_inner_type(field_ty, "Option", &skip_set(&[]));
            let filtered = filter_inner_type(&after, &skip_set(&["Box"]));
            ty_str(&wrap_leaf_type(&filtered, &skip_set(&[])))
        };
        prop_assert_eq!(run(), run());
    }

    /// NameValueExpr parsing is deterministic.
    #[test]
    fn name_value_parse_deterministic(
        key in snake_ident(),
    ) {
        let src = format!("{key} = 42");
        let a: NameValueExpr = syn::parse_str(&src).unwrap();
        let b: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(a.path.to_string(), b.path.to_string());
    }
}

// ===========================================================================
// 6. Derive expansion with no fields
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Unit struct (no fields) parses correctly.
    #[test]
    fn unit_struct_no_fields(name in pascal_ident()) {
        let src = format!("pub struct {name};");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.ident.to_string(), name);
        prop_assert_eq!(parsed.fields.len(), 0);
    }

    /// Empty braced struct parses with zero fields.
    #[test]
    fn empty_braced_struct(name in pascal_ident()) {
        let src = format!("pub struct {name} {{}}");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.fields.len(), 0);
    }

    /// Enum with only unit variants has no fields per variant.
    #[test]
    fn enum_all_unit_variants(
        name in pascal_ident(),
        count in 1usize..=6,
    ) {
        let variants: Vec<String> = (0..count).map(|i| format!("    V{i},")).collect();
        let src = format!("pub enum {name} {{\n{}\n}}", variants.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        for v in &parsed.variants {
            prop_assert_eq!(v.fields.len(), 0);
        }
    }
}

// ===========================================================================
// 7. Derive expansion with many fields
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    /// Struct with many fields: all fields accessible and names match.
    #[test]
    fn struct_many_fields_all_accessible(
        name in pascal_ident(),
        count in 8usize..=16,
        ty in leaf_type(),
    ) {
        let fields: Vec<String> = (0..count)
            .map(|i| format!("    pub f{i}: {ty},"))
            .collect();
        let src = format!("pub struct {name} {{\n{}\n}}", fields.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.fields.len(), count);
        for i in 0..count {
            let f = &parsed.fields.iter().nth(i).unwrap();
            prop_assert_eq!(f.ident.as_ref().unwrap().to_string(), format!("f{i}"));
        }
    }

    /// Struct with many container fields: all extractable.
    #[test]
    fn struct_many_container_fields_extractable(
        name in pascal_ident(),
        count in 4usize..=10,
        ctr in container(),
        inner in leaf_type(),
    ) {
        let fields: Vec<String> = (0..count)
            .map(|i| format!("    pub f{i}: {ctr}<{inner}>,"))
            .collect();
        let src = format!("pub struct {name} {{\n{}\n}}", fields.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        for f in &parsed.fields {
            let (result, ok) = try_extract_inner_type(&f.ty, ctr, &skip_set(&[]));
            prop_assert!(ok);
            prop_assert_eq!(ty_str(&result), inner);
        }
    }

    /// Enum with many tuple variants: all variant types are wrappable.
    #[test]
    fn enum_many_variants_wrappable(
        name in pascal_ident(),
        count in 4usize..=12,
        inner in leaf_type(),
    ) {
        let variants: Vec<String> = (0..count)
            .map(|i| format!("    V{i}({inner}),"))
            .collect();
        let src = format!("pub enum {name} {{\n{}\n}}", variants.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.variants.len(), count);
        for v in &parsed.variants {
            let field_ty = &v.fields.iter().next().unwrap().ty;
            let wrapped = wrap_leaf_type(field_ty, &skip_set(&[]));
            prop_assert_eq!(ty_str(&wrapped), format!("adze :: WithLeaf < {inner} >"));
        }
    }

    /// Struct with many heterogeneous field types: each type preserved.
    #[test]
    fn struct_heterogeneous_fields_preserved(
        name in pascal_ident(),
        types in prop::collection::vec(leaf_type(), 3..=8),
    ) {
        let fields: Vec<String> = types
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("    pub f{i}: {ty},"))
            .collect();
        let src = format!("pub struct {name} {{\n{}\n}}", fields.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        for i in 0..types.len() {
            let f = parsed.fields.iter().nth(i).unwrap();
            prop_assert_eq!(f.ty.to_token_stream().to_string(), types[i]);
        }
    }
}

// ===========================================================================
// 8. Derive expansion output is valid Rust
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Struct token output round-trips through parse_str.
    #[test]
    fn struct_output_reparseable(
        name in pascal_ident(),
        ty in leaf_type(),
    ) {
        let src = format!("pub struct {name} {{ pub val: {ty}, }}");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let tokens = parsed.to_token_stream().to_string();
        let reparsed: ItemStruct = parse_str(&tokens).unwrap();
        prop_assert_eq!(
            reparsed.to_token_stream().to_string(),
            parsed.to_token_stream().to_string()
        );
    }

    /// Enum token output round-trips through parse_str.
    #[test]
    fn enum_output_reparseable(
        name in pascal_ident(),
        ty in leaf_type(),
    ) {
        let src = format!("pub enum {name} {{ A({ty}), B, }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        let tokens = parsed.to_token_stream().to_string();
        let reparsed: ItemEnum = parse_str(&tokens).unwrap();
        prop_assert_eq!(
            reparsed.to_token_stream().to_string(),
            parsed.to_token_stream().to_string()
        );
    }

    /// Wrapped type output is valid Rust type syntax.
    #[test]
    fn wrapped_type_is_valid_rust(inner in leaf_type()) {
        let ty: Type = parse_str(inner).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
        let s = ty_str(&wrapped);
        let reparsed: Type = parse_str(&s).unwrap();
        prop_assert_eq!(ty_str(&reparsed), s);
    }

    /// Nested container wrapped output is valid Rust.
    #[test]
    fn nested_wrap_output_valid_rust(
        ctr in container(),
        inner in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{ctr}<{inner}>")).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip_set(&[ctr]));
        let s = ty_str(&wrapped);
        let reparsed: Type = parse_str(&s).unwrap();
        prop_assert_eq!(ty_str(&reparsed), s);
    }

    /// FieldThenParams output round-trips: field type stays valid.
    #[test]
    fn field_then_params_field_type_valid(inner in leaf_type()) {
        let src = format!("{inner}, key = 42");
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        let field_ty_str = parsed.field.ty.to_token_stream().to_string();
        let reparsed: Type = parse_str(&field_ty_str).unwrap();
        prop_assert_eq!(ty_str(&reparsed), field_ty_str);
    }

    /// Full derive-like expansion pipeline produces valid Rust type.
    #[test]
    fn full_pipeline_output_valid_rust(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!("pub struct {name} {{ pub v: Option<Box<{inner}>>, }}");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let field_ty = &parsed.fields.iter().next().unwrap().ty;
        // Extract Option
        let (after_opt, ok) = try_extract_inner_type(field_ty, "Option", &skip_set(&[]));
        prop_assert!(ok);
        // Filter Box
        let filtered = filter_inner_type(&after_opt, &skip_set(&["Box"]));
        // Wrap
        let wrapped = wrap_leaf_type(&filtered, &skip_set(&[]));
        let s = ty_str(&wrapped);
        // Must be valid Rust
        let reparsed: Type = parse_str(&s).unwrap();
        prop_assert_eq!(ty_str(&reparsed), s);
    }

    /// Struct with generic parameters produces reparseable output.
    #[test]
    fn generic_struct_output_reparseable(
        name in pascal_ident(),
        param in generic_param_name(),
        inner in leaf_type(),
    ) {
        let src = format!("pub struct {name}<{param}> {{ pub a: {param}, pub b: {inner}, }}");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let tokens = parsed.to_token_stream().to_string();
        let reparsed: ItemStruct = parse_str(&tokens).unwrap();
        prop_assert_eq!(
            reparsed.to_token_stream().to_string(),
            parsed.to_token_stream().to_string()
        );
    }
}
