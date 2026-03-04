#![allow(clippy::needless_range_loop)]

//! Property-based tests for struct type expansion in adze-common.
//!
//! Covers: struct field extraction, struct to SEQ rule expansion, single field
//! struct, multi field struct, struct with named fields, struct expansion
//! determinism, struct with Option fields, and struct with Vec fields.

use adze_common::{FieldThenParams, filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{ItemStruct, Type, parse_str};

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

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn skip_set<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

/// Build a named-field struct source string from field names and types.
fn build_struct(name: &str, fields: &[(&str, &str)]) -> String {
    let field_strs: Vec<String> = fields
        .iter()
        .map(|(fname, fty)| format!("    pub {fname}: {fty},"))
        .collect();
    format!("pub struct {name} {{\n{}\n}}", field_strs.join("\n"))
}

/// Collect all field types from a parsed struct through the expansion pipeline
/// (extract inner from container, filter wrappers, wrap leaf).
fn seq_rule_types(parsed: &ItemStruct, extract: &str, filter: &[&str]) -> Vec<String> {
    parsed
        .fields
        .iter()
        .map(|f| {
            let (inner, extracted) = try_extract_inner_type(&f.ty, extract, &skip_set(&[]));
            let base = if extracted { inner } else { f.ty.clone() };
            let filtered = filter_inner_type(&base, &skip_set(filter));
            ty_str(&wrap_leaf_type(&filtered, &skip_set(&[])))
        })
        .collect()
}

// ===========================================================================
// 1. Struct field extraction
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Parsing a struct preserves the struct name.
    #[test]
    fn field_extraction_struct_name_preserved(
        name in pascal_ident(),
        field in snake_ident(),
        ty in leaf_type(),
    ) {
        let src = build_struct(&name, &[(&field, ty)]);
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.ident.to_string(), name);
    }

    /// Field ident survives round-trip through syn parsing.
    #[test]
    fn field_extraction_ident_roundtrip(
        name in pascal_ident(),
        field in snake_ident(),
        ty in leaf_type(),
    ) {
        let src = build_struct(&name, &[(&field, ty)]);
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let f = parsed.fields.iter().next().unwrap();
        prop_assert_eq!(f.ident.as_ref().unwrap().to_string(), field);
    }

    /// Field type survives extraction when no container is present.
    #[test]
    fn field_extraction_plain_type_unchanged(
        name in pascal_ident(),
        ty in leaf_type(),
    ) {
        let src = build_struct(&name, &[("val", ty)]);
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let field_ty = &parsed.fields.iter().next().unwrap().ty;
        let (result, extracted) = try_extract_inner_type(field_ty, "Option", &skip_set(&[]));
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&result), ty);
    }

    /// Extracting inner type from a container field succeeds.
    #[test]
    fn field_extraction_container_inner(
        name in pascal_ident(),
        ctr in container(),
        inner in leaf_type(),
    ) {
        let ty_src = format!("{ctr}<{inner}>");
        let src = build_struct(&name, &[("val", &ty_src)]);
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let field_ty = &parsed.fields.iter().next().unwrap().ty;
        let (result, ok) = try_extract_inner_type(field_ty, ctr, &skip_set(&[]));
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&result), inner);
    }
}

// ===========================================================================
// 2. Struct to SEQ rule expansion
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// SEQ expansion produces one element per struct field.
    #[test]
    fn seq_expansion_element_count(
        name in pascal_ident(),
        count in 1usize..=8,
        ty in leaf_type(),
    ) {
        let fields: Vec<(&str, &str)> = (0..count)
            .map(|_| ("x", ty))
            .collect();
        // Use numbered field names to avoid duplicates.
        let field_strs: Vec<String> = (0..count)
            .map(|i| format!("    pub f{i}: {ty},"))
            .collect();
        let src = format!("pub struct {name} {{\n{}\n}}", field_strs.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let seq = seq_rule_types(&parsed, "Option", &[]);
        prop_assert_eq!(seq.len(), count);
    }

    /// SEQ expansion preserves field ordering.
    #[test]
    fn seq_expansion_preserves_order(
        name in pascal_ident(),
        types in prop::collection::vec(leaf_type(), 2..=6),
    ) {
        let field_strs: Vec<String> = types
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("    pub f{i}: {ty},"))
            .collect();
        let src = format!("pub struct {name} {{\n{}\n}}", field_strs.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        for i in 0..types.len() {
            let f = parsed.fields.iter().nth(i).unwrap();
            prop_assert_eq!(f.ty.to_token_stream().to_string(), types[i]);
        }
    }

    /// SEQ expansion of plain fields wraps each leaf type.
    #[test]
    fn seq_expansion_wraps_all_leaves(
        name in pascal_ident(),
        count in 2usize..=5,
        ty in leaf_type(),
    ) {
        let field_strs: Vec<String> = (0..count)
            .map(|i| format!("    pub f{i}: {ty},"))
            .collect();
        let src = format!("pub struct {name} {{\n{}\n}}", field_strs.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let seq = seq_rule_types(&parsed, "Option", &[]);
        for s in &seq {
            prop_assert_eq!(s.as_str(), &format!("adze :: WithLeaf < {ty} >"));
        }
    }
}

// ===========================================================================
// 3. Single field struct
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Single-field struct has exactly one field.
    #[test]
    fn single_field_struct_count(
        name in pascal_ident(),
        field in snake_ident(),
        ty in leaf_type(),
    ) {
        let src = build_struct(&name, &[(&field, ty)]);
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.fields.len(), 1);
    }

    /// Single-field struct: wrapping produces one-element SEQ.
    #[test]
    fn single_field_struct_seq_len_one(
        name in pascal_ident(),
        ty in leaf_type(),
    ) {
        let src = build_struct(&name, &[("val", ty)]);
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let seq = seq_rule_types(&parsed, "Option", &[]);
        prop_assert_eq!(seq.len(), 1);
        prop_assert_eq!(seq[0].as_str(), format!("adze :: WithLeaf < {ty} >"));
    }

    /// Single-field struct with container: extractable then wrappable.
    #[test]
    fn single_field_container_extract_wrap(
        name in pascal_ident(),
        ctr in container(),
        inner in leaf_type(),
    ) {
        let ty_src = format!("{ctr}<{inner}>");
        let src = build_struct(&name, &[("val", &ty_src)]);
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let field_ty = &parsed.fields.iter().next().unwrap().ty;
        let (extracted, ok) = try_extract_inner_type(field_ty, ctr, &skip_set(&[]));
        prop_assert!(ok);
        let wrapped = wrap_leaf_type(&extracted, &skip_set(&[]));
        prop_assert_eq!(ty_str(&wrapped), format!("adze :: WithLeaf < {inner} >"));
    }
}

// ===========================================================================
// 4. Multi field struct
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    /// Multi-field struct preserves all field names.
    #[test]
    fn multi_field_names_preserved(
        name in pascal_ident(),
        count in 3usize..=12,
        ty in leaf_type(),
    ) {
        let field_strs: Vec<String> = (0..count)
            .map(|i| format!("    pub f{i}: {ty},"))
            .collect();
        let src = format!("pub struct {name} {{\n{}\n}}", field_strs.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.fields.len(), count);
        for i in 0..count {
            let f = parsed.fields.iter().nth(i).unwrap();
            prop_assert_eq!(f.ident.as_ref().unwrap().to_string(), format!("f{i}"));
        }
    }

    /// Multi-field struct with heterogeneous types: each type preserved.
    #[test]
    fn multi_field_heterogeneous_types(
        name in pascal_ident(),
        types in prop::collection::vec(leaf_type(), 3..=8),
    ) {
        let field_strs: Vec<String> = types
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("    pub f{i}: {ty},"))
            .collect();
        let src = format!("pub struct {name} {{\n{}\n}}", field_strs.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        for i in 0..types.len() {
            let f = parsed.fields.iter().nth(i).unwrap();
            prop_assert_eq!(ty_str(&f.ty), types[i]);
        }
    }

    /// Multi-field struct: wrapping all fields gives correct SEQ length.
    #[test]
    fn multi_field_seq_length(
        name in pascal_ident(),
        count in 4usize..=10,
        ty in leaf_type(),
    ) {
        let field_strs: Vec<String> = (0..count)
            .map(|i| format!("    pub f{i}: {ty},"))
            .collect();
        let src = format!("pub struct {name} {{\n{}\n}}", field_strs.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let seq = seq_rule_types(&parsed, "Option", &[]);
        prop_assert_eq!(seq.len(), count);
    }

    /// Multi-field struct: each SEQ element contains the correct type.
    #[test]
    fn multi_field_seq_types_correct(
        name in pascal_ident(),
        types in prop::collection::vec(leaf_type(), 2..=6),
    ) {
        let field_strs: Vec<String> = types
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("    pub f{i}: {ty},"))
            .collect();
        let src = format!("pub struct {name} {{\n{}\n}}", field_strs.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let seq = seq_rule_types(&parsed, "Option", &[]);
        for i in 0..types.len() {
            prop_assert!(seq[i].contains(types[i]));
        }
    }
}

// ===========================================================================
// 5. Struct with named fields
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Named field idents are all recoverable.
    #[test]
    fn named_fields_idents_recoverable(
        name in pascal_ident(),
        ty in leaf_type(),
    ) {
        let src = format!("pub struct {name} {{ pub alpha: {ty}, pub beta: {ty}, pub gamma: {ty}, }}");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let names: Vec<String> = parsed
            .fields
            .iter()
            .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
            .collect();
        prop_assert_eq!(names, vec!["alpha", "beta", "gamma"]);
    }

    /// Named field type is wrappable regardless of name.
    #[test]
    fn named_field_type_wrappable(
        name in pascal_ident(),
        field in snake_ident(),
        ty in leaf_type(),
    ) {
        let src = build_struct(&name, &[(&field, ty)]);
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let field_ty = &parsed.fields.iter().next().unwrap().ty;
        let wrapped = wrap_leaf_type(field_ty, &skip_set(&[]));
        prop_assert_eq!(ty_str(&wrapped), format!("adze :: WithLeaf < {ty} >"));
    }

    /// Named fields with container types: extraction uses correct container.
    #[test]
    fn named_fields_container_extraction(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!(
            "pub struct {name} {{ pub a: Option<{inner}>, pub b: Vec<{inner}>, }}"
        );
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let fields: Vec<_> = parsed.fields.iter().collect();

        let (r1, ok1) = try_extract_inner_type(&fields[0].ty, "Option", &skip_set(&[]));
        prop_assert!(ok1);
        prop_assert_eq!(ty_str(&r1), inner);

        let (r2, ok2) = try_extract_inner_type(&fields[1].ty, "Vec", &skip_set(&[]));
        prop_assert!(ok2);
        prop_assert_eq!(ty_str(&r2), inner);
    }
}

// ===========================================================================
// 6. Struct expansion determinism
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Parsing the same struct source twice yields identical tokens.
    #[test]
    fn determinism_parse_tokens(
        name in pascal_ident(),
        ty in leaf_type(),
    ) {
        let src = build_struct(&name, &[("val", ty)]);
        let a: ItemStruct = parse_str(&src).unwrap();
        let b: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(a.to_token_stream().to_string(), b.to_token_stream().to_string());
    }

    /// SEQ expansion is deterministic across two runs.
    #[test]
    fn determinism_seq_expansion(
        name in pascal_ident(),
        count in 2usize..=6,
        ty in leaf_type(),
    ) {
        let field_strs: Vec<String> = (0..count)
            .map(|i| format!("    pub f{i}: {ty},"))
            .collect();
        let src = format!("pub struct {name} {{\n{}\n}}", field_strs.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let run1 = seq_rule_types(&parsed, "Option", &[]);
        let run2 = seq_rule_types(&parsed, "Option", &[]);
        prop_assert_eq!(run1, run2);
    }

    /// Extract + filter + wrap pipeline on struct fields is deterministic.
    #[test]
    fn determinism_extract_filter_wrap(
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

    /// Struct token output round-trips through parse_str.
    #[test]
    fn determinism_token_roundtrip(
        name in pascal_ident(),
        ty in leaf_type(),
    ) {
        let src = build_struct(&name, &[("val", ty)]);
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let tokens = parsed.to_token_stream().to_string();
        let reparsed: ItemStruct = parse_str(&tokens).unwrap();
        prop_assert_eq!(
            reparsed.to_token_stream().to_string(),
            parsed.to_token_stream().to_string()
        );
    }
}

// ===========================================================================
// 7. Struct with Option fields
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Option field inner type is extractable.
    #[test]
    fn option_field_extractable(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = build_struct(&name, &[("val", &format!("Option<{inner}>"))]);
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let field_ty = &parsed.fields.iter().next().unwrap().ty;
        let (result, ok) = try_extract_inner_type(field_ty, "Option", &skip_set(&[]));
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&result), inner);
    }

    /// Option<Box<T>> extracts through skip set to inner T.
    #[test]
    fn option_box_field_skip_extract(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let ty_src = format!("Option<Box<{inner}>>");
        let src = build_struct(&name, &[("val", &ty_src)]);
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let field_ty = &parsed.fields.iter().next().unwrap().ty;
        let (result, ok) = try_extract_inner_type(field_ty, "Option", &skip_set(&["Box"]));
        prop_assert!(ok);
        // Extraction stops at Option, returning Box<inner>.
        let filtered = filter_inner_type(&result, &skip_set(&["Box"]));
        prop_assert_eq!(ty_str(&filtered), inner);
    }

    /// Multiple Option fields are all independently extractable.
    #[test]
    fn option_fields_all_extractable(
        name in pascal_ident(),
        types in prop::collection::vec(leaf_type(), 2..=5),
    ) {
        let field_strs: Vec<String> = types
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("    pub f{i}: Option<{ty}>,"))
            .collect();
        let src = format!("pub struct {name} {{\n{}\n}}", field_strs.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        for (i, f) in parsed.fields.iter().enumerate() {
            let (result, ok) = try_extract_inner_type(&f.ty, "Option", &skip_set(&[]));
            prop_assert!(ok);
            prop_assert_eq!(ty_str(&result), types[i]);
        }
    }

    /// Wrapping an Option field preserves the Option wrapper with inner wrapped.
    #[test]
    fn option_field_wrap_preserves_option(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = build_struct(&name, &[("val", &format!("Option<{inner}>"))]);
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let field_ty = &parsed.fields.iter().next().unwrap().ty;
        let wrapped = wrap_leaf_type(field_ty, &skip_set(&["Option"]));
        let expected = format!("Option < adze :: WithLeaf < {inner} > >");
        prop_assert_eq!(ty_str(&wrapped), expected);
    }
}

// ===========================================================================
// 8. Struct with Vec fields
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Vec field inner type is extractable.
    #[test]
    fn vec_field_extractable(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = build_struct(&name, &[("items", &format!("Vec<{inner}>"))]);
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let field_ty = &parsed.fields.iter().next().unwrap().ty;
        let (result, ok) = try_extract_inner_type(field_ty, "Vec", &skip_set(&[]));
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&result), inner);
    }

    /// Vec<Box<T>> extracts Vec then filters Box to get T.
    #[test]
    fn vec_box_field_extract_filter(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let ty_src = format!("Vec<Box<{inner}>>");
        let src = build_struct(&name, &[("items", &ty_src)]);
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let field_ty = &parsed.fields.iter().next().unwrap().ty;
        let (result, ok) = try_extract_inner_type(field_ty, "Vec", &skip_set(&[]));
        prop_assert!(ok);
        let filtered = filter_inner_type(&result, &skip_set(&["Box"]));
        prop_assert_eq!(ty_str(&filtered), inner);
    }

    /// Multiple Vec fields are all independently extractable.
    #[test]
    fn vec_fields_all_extractable(
        name in pascal_ident(),
        types in prop::collection::vec(leaf_type(), 2..=5),
    ) {
        let field_strs: Vec<String> = types
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("    pub f{i}: Vec<{ty}>,"))
            .collect();
        let src = format!("pub struct {name} {{\n{}\n}}", field_strs.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        for (i, f) in parsed.fields.iter().enumerate() {
            let (result, ok) = try_extract_inner_type(&f.ty, "Vec", &skip_set(&[]));
            prop_assert!(ok);
            prop_assert_eq!(ty_str(&result), types[i]);
        }
    }

    /// Wrapping a Vec field preserves the Vec wrapper with inner wrapped.
    #[test]
    fn vec_field_wrap_preserves_vec(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = build_struct(&name, &[("items", &format!("Vec<{inner}>"))]);
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let field_ty = &parsed.fields.iter().next().unwrap().ty;
        let wrapped = wrap_leaf_type(field_ty, &skip_set(&["Vec"]));
        let expected = format!("Vec < adze :: WithLeaf < {inner} > >");
        prop_assert_eq!(ty_str(&wrapped), expected);
    }

    /// Mixed Option and Vec fields in same struct: each extracts correctly.
    #[test]
    fn mixed_option_vec_fields(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!(
            "pub struct {name} {{ pub opt: Option<{inner}>, pub items: Vec<{inner}>, pub plain: {inner}, }}"
        );
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let fields: Vec<_> = parsed.fields.iter().collect();

        let (r_opt, ok_opt) = try_extract_inner_type(&fields[0].ty, "Option", &skip_set(&[]));
        prop_assert!(ok_opt);
        prop_assert_eq!(ty_str(&r_opt), inner);

        let (r_vec, ok_vec) = try_extract_inner_type(&fields[1].ty, "Vec", &skip_set(&[]));
        prop_assert!(ok_vec);
        prop_assert_eq!(ty_str(&r_vec), inner);

        let (_, ok_plain) = try_extract_inner_type(&fields[2].ty, "Option", &skip_set(&[]));
        prop_assert!(!ok_plain);
    }

    /// Full pipeline on Vec field: extract, filter, wrap produces valid type.
    #[test]
    fn vec_field_full_pipeline_valid(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let ty_src = format!("Vec<Box<{inner}>>");
        let src = build_struct(&name, &[("items", &ty_src)]);
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let field_ty = &parsed.fields.iter().next().unwrap().ty;
        let (extracted, ok) = try_extract_inner_type(field_ty, "Vec", &skip_set(&[]));
        prop_assert!(ok);
        let filtered = filter_inner_type(&extracted, &skip_set(&["Box"]));
        let wrapped = wrap_leaf_type(&filtered, &skip_set(&[]));
        let s = ty_str(&wrapped);
        let reparsed: Type = parse_str(&s).unwrap();
        prop_assert_eq!(ty_str(&reparsed), s);
    }

    /// FieldThenParams with Vec type preserves the field type.
    #[test]
    fn field_then_params_vec_type(inner in leaf_type()) {
        let src = format!("Vec<{inner}>, key = 42");
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        let field_ty_str = parsed.field.ty.to_token_stream().to_string();
        prop_assert!(field_ty_str.contains("Vec"));
        prop_assert!(field_ty_str.contains(inner));
    }
}
