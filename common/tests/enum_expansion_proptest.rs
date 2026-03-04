#![allow(clippy::needless_range_loop)]

//! Property-based tests for enum type expansion in adze-common.
//!
//! Covers: enum variant extraction, enum to CHOICE rule expansion, unit variant
//! expansion, tuple variant expansion, struct variant expansion, mixed variant
//! expansion, enum expansion determinism, and enums with many variants.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Fields, ItemEnum, Type, parse_str};

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

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn skip_set<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

// ===========================================================================
// 1. Enum variant extraction
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Parsing an enum preserves the enum name.
    #[test]
    fn variant_extraction_enum_name_preserved(
        name in pascal_ident(),
        count in 1usize..=6,
    ) {
        let variants: Vec<String> = (0..count).map(|i| format!("    V{i},")).collect();
        let src = format!("pub enum {name} {{\n{}\n}}", variants.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.ident.to_string(), name);
    }

    /// Each variant ident is recoverable after parsing.
    #[test]
    fn variant_extraction_idents_recovered(
        name in pascal_ident(),
        count in 1usize..=8,
    ) {
        let expected: Vec<String> = (0..count).map(|i| format!("Alt{i}")).collect();
        let var_strs: Vec<String> = expected.iter().map(|v| format!("    {v},")).collect();
        let src = format!("pub enum {name} {{\n{}\n}}", var_strs.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.variants.len(), count);
        for i in 0..count {
            prop_assert_eq!(parsed.variants[i].ident.to_string(), expected[i].as_str());
        }
    }

    /// Tuple variant inner type can be extracted via the utility.
    #[test]
    fn variant_extraction_tuple_inner_type(
        name in pascal_ident(),
        ctr in container(),
        inner in leaf_type(),
    ) {
        let src = format!("pub enum {name} {{ V({ctr}<{inner}>), }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        let field_ty = &parsed.variants[0].fields.iter().next().unwrap().ty;
        let (result, ok) = try_extract_inner_type(field_ty, ctr, &skip_set(&[]));
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&result), inner);
    }

    /// Struct variant field type can be extracted.
    #[test]
    fn variant_extraction_struct_field_type(
        name in pascal_ident(),
        ctr in container(),
        inner in leaf_type(),
    ) {
        let src = format!("pub enum {name} {{ V {{ val: {ctr}<{inner}> }}, }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        let field_ty = &parsed.variants[0].fields.iter().next().unwrap().ty;
        let (result, ok) = try_extract_inner_type(field_ty, ctr, &skip_set(&[]));
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&result), inner);
    }
}

// ===========================================================================
// 2. Enum to CHOICE rule expansion
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Each tuple variant type wraps independently — simulating CHOICE alternatives.
    #[test]
    fn choice_expansion_independent_wrapping(
        name in pascal_ident(),
        count in 2usize..=6,
        inner in leaf_type(),
    ) {
        let variants: Vec<String> = (0..count).map(|i| format!("    V{i}({inner}),")).collect();
        let src = format!("pub enum {name} {{\n{}\n}}", variants.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        let mut wrapped: Vec<String> = Vec::new();
        for v in &parsed.variants {
            let field_ty = &v.fields.iter().next().unwrap().ty;
            wrapped.push(ty_str(&wrap_leaf_type(field_ty, &skip_set(&[]))));
        }
        prop_assert_eq!(wrapped.len(), count);
        for w in &wrapped {
            prop_assert_eq!(w.as_str(), &format!("adze :: WithLeaf < {inner} >"));
        }
    }

    /// Distinct variant types produce distinct CHOICE alternatives.
    #[test]
    fn choice_expansion_distinct_alternatives(
        name in pascal_ident(),
        types in prop::collection::vec(leaf_type(), 2..=5),
    ) {
        let variants: Vec<String> = types
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("    V{i}({ty}),"))
            .collect();
        let src = format!("pub enum {name} {{\n{}\n}}", variants.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        let wrapped: Vec<String> = parsed
            .variants
            .iter()
            .map(|v| {
                let field_ty = &v.fields.iter().next().unwrap().ty;
                ty_str(&wrap_leaf_type(field_ty, &skip_set(&[])))
            })
            .collect();
        // All wrapped types correspond to their input types.
        for i in 0..types.len() {
            prop_assert!(wrapped[i].contains(types[i]));
        }
    }

    /// CHOICE alternatives count equals variant count.
    #[test]
    fn choice_expansion_count_matches(
        name in pascal_ident(),
        count in 1usize..=10,
        inner in leaf_type(),
    ) {
        let variants: Vec<String> = (0..count).map(|i| format!("    V{i}({inner}),")).collect();
        let src = format!("pub enum {name} {{\n{}\n}}", variants.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.variants.len(), count);
    }

    /// CHOICE expansion with container types filters then wraps.
    #[test]
    fn choice_expansion_filter_then_wrap(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!(
            "pub enum {name} {{ A(Box<{inner}>), B(Box<{inner}>), }}"
        );
        let parsed: ItemEnum = parse_str(&src).unwrap();
        for v in &parsed.variants {
            let field_ty = &v.fields.iter().next().unwrap().ty;
            let filtered = filter_inner_type(field_ty, &skip_set(&["Box"]));
            let wrapped = wrap_leaf_type(&filtered, &skip_set(&[]));
            prop_assert_eq!(ty_str(&wrapped), format!("adze :: WithLeaf < {inner} >"));
        }
    }
}

// ===========================================================================
// 3. Unit variant expansion
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Unit variants have no fields.
    #[test]
    fn unit_variant_no_fields(
        name in pascal_ident(),
        count in 1usize..=8,
    ) {
        let variants: Vec<String> = (0..count).map(|i| format!("    U{i},")).collect();
        let src = format!("pub enum {name} {{\n{}\n}}", variants.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        for v in &parsed.variants {
            prop_assert!(matches!(v.fields, Fields::Unit));
        }
    }

    /// Unit variant names are preserved after parsing.
    #[test]
    fn unit_variant_names_preserved(
        name in pascal_ident(),
        count in 1usize..=6,
    ) {
        let expected: Vec<String> = (0..count).map(|i| format!("Kind{i}")).collect();
        let var_strs: Vec<String> = expected.iter().map(|v| format!("    {v},")).collect();
        let src = format!("pub enum {name} {{\n{}\n}}", var_strs.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        for i in 0..count {
            prop_assert_eq!(parsed.variants[i].ident.to_string(), expected[i].as_str());
            prop_assert_eq!(parsed.variants[i].fields.len(), 0);
        }
    }

    /// A single unit variant enum parses correctly.
    #[test]
    fn unit_variant_singleton(name in pascal_ident()) {
        let src = format!("pub enum {name} {{ Only, }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.variants.len(), 1);
        prop_assert!(matches!(parsed.variants[0].fields, Fields::Unit));
    }
}

// ===========================================================================
// 4. Tuple variant expansion
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Single-element tuple variant field type is preserved.
    #[test]
    fn tuple_variant_single_field_type(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!("pub enum {name} {{ V({inner}), }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        let field = parsed.variants[0].fields.iter().next().unwrap();
        prop_assert_eq!(ty_str(&field.ty), inner);
    }

    /// Tuple variant with container type: inner is extractable then wrappable.
    #[test]
    fn tuple_variant_extract_then_wrap(
        name in pascal_ident(),
        ctr in container(),
        inner in leaf_type(),
    ) {
        let src = format!("pub enum {name} {{ V({ctr}<{inner}>), }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        let field_ty = &parsed.variants[0].fields.iter().next().unwrap().ty;
        let (extracted, ok) = try_extract_inner_type(field_ty, ctr, &skip_set(&[]));
        prop_assert!(ok);
        let wrapped = wrap_leaf_type(&extracted, &skip_set(&[]));
        prop_assert_eq!(ty_str(&wrapped), format!("adze :: WithLeaf < {inner} >"));
    }

    /// Multi-element tuple variant has correct field count.
    #[test]
    fn tuple_variant_multi_field_count(
        name in pascal_ident(),
        count in 2usize..=4,
        inner in leaf_type(),
    ) {
        let fields = vec![inner; count].join(", ");
        let src = format!("pub enum {name} {{ V({fields}), }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.variants[0].fields.len(), count);
    }
}

// ===========================================================================
// 5. Struct variant expansion
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Struct variant field names are preserved.
    #[test]
    fn struct_variant_field_names_preserved(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!("pub enum {name} {{ V {{ alpha: {inner}, beta: {inner} }}, }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        let fields: Vec<String> = parsed.variants[0]
            .fields
            .iter()
            .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
            .collect();
        prop_assert_eq!(fields, vec!["alpha".to_string(), "beta".to_string()]);
    }

    /// Struct variant field types survive wrapping.
    #[test]
    fn struct_variant_field_types_wrappable(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!("pub enum {name} {{ V {{ val: {inner} }}, }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        let field_ty = &parsed.variants[0].fields.iter().next().unwrap().ty;
        let wrapped = wrap_leaf_type(field_ty, &skip_set(&[]));
        prop_assert_eq!(ty_str(&wrapped), format!("adze :: WithLeaf < {inner} >"));
    }

    /// Struct variant with container field: filter then wrap works.
    #[test]
    fn struct_variant_filter_wrap_pipeline(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!("pub enum {name} {{ V {{ val: Box<{inner}> }}, }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        let field_ty = &parsed.variants[0].fields.iter().next().unwrap().ty;
        let filtered = filter_inner_type(field_ty, &skip_set(&["Box"]));
        prop_assert_eq!(ty_str(&filtered), inner);
        let wrapped = wrap_leaf_type(&filtered, &skip_set(&[]));
        prop_assert_eq!(ty_str(&wrapped), format!("adze :: WithLeaf < {inner} >"));
    }
}

// ===========================================================================
// 6. Mixed variant expansion
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Enum with unit + tuple + struct variants: correct variant count.
    #[test]
    fn mixed_variant_count(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!(
            "pub enum {name} {{ Unit, Tuple({inner}), Named {{ val: {inner} }}, }}"
        );
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.variants.len(), 3);
    }

    /// Mixed variants: unit has no fields, tuple has unnamed, struct has named.
    #[test]
    fn mixed_variant_field_kinds(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!(
            "pub enum {name} {{ A, B({inner}), C {{ x: {inner} }}, }}"
        );
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert!(matches!(parsed.variants[0].fields, Fields::Unit));
        prop_assert!(matches!(parsed.variants[1].fields, Fields::Unnamed(_)));
        prop_assert!(matches!(parsed.variants[2].fields, Fields::Named(_)));
    }

    /// Mixed variants: tuple variant type is wrappable.
    #[test]
    fn mixed_variant_tuple_wrappable(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!(
            "pub enum {name} {{ A, B({inner}), C {{ x: {inner} }}, }}"
        );
        let parsed: ItemEnum = parse_str(&src).unwrap();
        let tuple_ty = &parsed.variants[1].fields.iter().next().unwrap().ty;
        let wrapped = wrap_leaf_type(tuple_ty, &skip_set(&[]));
        prop_assert_eq!(ty_str(&wrapped), format!("adze :: WithLeaf < {inner} >"));
    }

    /// Mixed variants: struct variant field is wrappable.
    #[test]
    fn mixed_variant_struct_wrappable(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!(
            "pub enum {name} {{ A, B({inner}), C {{ x: {inner} }}, }}"
        );
        let parsed: ItemEnum = parse_str(&src).unwrap();
        let struct_ty = &parsed.variants[2].fields.iter().next().unwrap().ty;
        let wrapped = wrap_leaf_type(struct_ty, &skip_set(&[]));
        prop_assert_eq!(ty_str(&wrapped), format!("adze :: WithLeaf < {inner} >"));
    }
}

// ===========================================================================
// 7. Enum expansion determinism
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Parsing the same enum source twice yields identical variant names.
    #[test]
    fn determinism_enum_parse_variants(
        name in pascal_ident(),
        count in 1usize..=6,
        inner in leaf_type(),
    ) {
        let variants: Vec<String> = (0..count).map(|i| format!("    V{i}({inner}),")).collect();
        let src = format!("pub enum {name} {{\n{}\n}}", variants.join("\n"));
        let a: ItemEnum = parse_str(&src).unwrap();
        let b: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(a.variants.len(), b.variants.len());
        for i in 0..count {
            prop_assert_eq!(
                a.variants[i].ident.to_string(),
                b.variants[i].ident.to_string()
            );
        }
    }

    /// Enum token output round-trips through parse_str.
    #[test]
    fn determinism_enum_token_roundtrip(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!("pub enum {name} {{ A({inner}), B, C {{ v: {inner} }}, }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        let tokens = parsed.to_token_stream().to_string();
        let reparsed: ItemEnum = parse_str(&tokens).unwrap();
        prop_assert_eq!(
            reparsed.to_token_stream().to_string(),
            parsed.to_token_stream().to_string()
        );
    }

    /// Wrapping variant types twice gives the same result.
    #[test]
    fn determinism_variant_wrap(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!("pub enum {name} {{ V({inner}), }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        let field_ty = &parsed.variants[0].fields.iter().next().unwrap().ty;
        let a = ty_str(&wrap_leaf_type(field_ty, &skip_set(&[])));
        let b = ty_str(&wrap_leaf_type(field_ty, &skip_set(&[])));
        prop_assert_eq!(a, b);
    }

    /// Full CHOICE expansion pipeline is deterministic.
    #[test]
    fn determinism_choice_pipeline(
        name in pascal_ident(),
        inner in leaf_type(),
    ) {
        let src = format!(
            "pub enum {name} {{ A(Box<{inner}>), B(Option<{inner}>), }}"
        );
        let parsed: ItemEnum = parse_str(&src).unwrap();
        let run = || -> Vec<String> {
            parsed
                .variants
                .iter()
                .map(|v| {
                    let field_ty = &v.fields.iter().next().unwrap().ty;
                    let filtered = filter_inner_type(field_ty, &skip_set(&["Box"]));
                    let (inner_ty, _) = try_extract_inner_type(&filtered, "Option", &skip_set(&[]));
                    ty_str(&wrap_leaf_type(&inner_ty, &skip_set(&[])))
                })
                .collect()
        };
        prop_assert_eq!(run(), run());
    }
}

// ===========================================================================
// 8. Enum with many variants
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    /// Enum with many tuple variants: all variant types are wrappable.
    #[test]
    fn many_variants_all_wrappable(
        name in pascal_ident(),
        count in 8usize..=20,
        inner in leaf_type(),
    ) {
        let variants: Vec<String> = (0..count).map(|i| format!("    V{i}({inner}),")).collect();
        let src = format!("pub enum {name} {{\n{}\n}}", variants.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.variants.len(), count);
        for v in &parsed.variants {
            let field_ty = &v.fields.iter().next().unwrap().ty;
            let wrapped = wrap_leaf_type(field_ty, &skip_set(&[]));
            prop_assert_eq!(ty_str(&wrapped), format!("adze :: WithLeaf < {inner} >"));
        }
    }

    /// Enum with many unit variants: all have zero fields.
    #[test]
    fn many_unit_variants_all_empty(
        name in pascal_ident(),
        count in 10usize..=25,
    ) {
        let variants: Vec<String> = (0..count).map(|i| format!("    U{i},")).collect();
        let src = format!("pub enum {name} {{\n{}\n}}", variants.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.variants.len(), count);
        for v in &parsed.variants {
            prop_assert_eq!(v.fields.len(), 0);
        }
    }

    /// Enum with many heterogeneous variant types: each type preserved.
    #[test]
    fn many_variants_heterogeneous_types(
        name in pascal_ident(),
        types in prop::collection::vec(leaf_type(), 4..=12),
    ) {
        let variants: Vec<String> = types
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("    V{i}({ty}),"))
            .collect();
        let src = format!("pub enum {name} {{\n{}\n}}", variants.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        for i in 0..types.len() {
            let field = parsed.variants[i].fields.iter().next().unwrap();
            prop_assert_eq!(ty_str(&field.ty), types[i]);
        }
    }

    /// Enum with many mixed variants: unit/tuple/struct pattern repeats.
    #[test]
    fn many_mixed_variants_pattern(
        name in pascal_ident(),
        repetitions in 3usize..=6,
        inner in leaf_type(),
    ) {
        let mut variants = Vec::new();
        for r in 0..repetitions {
            variants.push(format!("    U{r},"));
            variants.push(format!("    T{r}({inner}),"));
            variants.push(format!("    S{r} {{ val: {inner} }},"));
        }
        let src = format!("pub enum {name} {{\n{}\n}}", variants.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.variants.len(), repetitions * 3);
        for r in 0..repetitions {
            let base = r * 3;
            prop_assert!(matches!(parsed.variants[base].fields, Fields::Unit));
            prop_assert!(matches!(parsed.variants[base + 1].fields, Fields::Unnamed(_)));
            prop_assert!(matches!(parsed.variants[base + 2].fields, Fields::Named(_)));
        }
    }

    /// Many-variant enum with skip-wrapped types: filter + wrap pipeline.
    #[test]
    fn many_variants_filter_wrap_pipeline(
        name in pascal_ident(),
        count in 5usize..=15,
        inner in leaf_type(),
    ) {
        let variants: Vec<String> = (0..count)
            .map(|i| format!("    V{i}(Box<{inner}>),"))
            .collect();
        let src = format!("pub enum {name} {{\n{}\n}}", variants.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        for v in &parsed.variants {
            let field_ty = &v.fields.iter().next().unwrap().ty;
            let filtered = filter_inner_type(field_ty, &skip_set(&["Box"]));
            prop_assert_eq!(ty_str(&filtered), inner);
            let wrapped = wrap_leaf_type(&filtered, &skip_set(&[]));
            prop_assert_eq!(ty_str(&wrapped), format!("adze :: WithLeaf < {inner} >"));
        }
    }
}
