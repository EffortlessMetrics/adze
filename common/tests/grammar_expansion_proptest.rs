#![allow(clippy::needless_range_loop)]

//! Property-based tests for grammar expansion logic in adze-common.
//!
//! Covers: struct expansion, enum expansion, field-to-rule expansion,
//! optional field expansion, Vec field expansion, nested type expansion,
//! determinism, and field preservation.

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Item, ItemEnum, ItemMod, ItemStruct, Type, parse_quote, parse_str};

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

fn ident_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,12}")
        .unwrap()
        .prop_filter("must be valid ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

fn pascal_ident() -> impl Strategy<Value = String> {
    prop::string::string_regex("[A-Z][a-zA-Z0-9]{0,12}")
        .unwrap()
        .prop_filter("must be valid ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

fn grammar_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,15}")
        .unwrap()
        .prop_filter("must be valid grammar name", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

/// Builds a struct source string with named fields.
fn build_struct(name: &str, fields: &[(&str, &str)]) -> String {
    let field_strs: Vec<String> = fields
        .iter()
        .map(|(fname, ftype)| format!("    pub {fname}: {ftype},"))
        .collect();
    format!("pub struct {name} {{\n{}\n}}", field_strs.join("\n"))
}

/// Builds an enum source string with tuple variants.
fn build_enum(name: &str, variants: &[(&str, &str)]) -> String {
    let var_strs: Vec<String> = variants
        .iter()
        .map(|(vname, vtype)| format!("    {vname}({vtype}),"))
        .collect();
    format!("pub enum {name} {{\n{}\n}}", var_strs.join("\n"))
}

fn build_grammar_module(grammar_name: &str, mod_name: &str, body: &str) -> String {
    format!(
        r#"#[adze::grammar("{grammar_name}")]
mod {mod_name} {{
{body}
}}"#
    )
}

fn type_to_string(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn skip_set<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

// ===========================================================================
// 1. Grammar expansion from struct — struct fields are parseable and preserved
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 1. Struct with a single field parses and field name is preserved
    #[test]
    fn struct_single_field_name_preserved(
        struct_name in pascal_ident(),
        field_name in ident_strategy(),
        field_ty in leaf_type(),
    ) {
        let src = format!("pub struct {struct_name} {{ pub {field_name}: {field_ty}, }}");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.ident.to_string(), struct_name);
        let f = parsed.fields.iter().next().unwrap();
        prop_assert_eq!(f.ident.as_ref().unwrap().to_string(), field_name);
    }

    // 2. Struct with multiple fields preserves field count
    #[test]
    fn struct_field_count_preserved(
        struct_name in pascal_ident(),
        count in 1usize..=8,
        field_ty in leaf_type(),
    ) {
        let fields: Vec<String> = (0..count)
            .map(|i| format!("    pub f{i}: {field_ty},"))
            .collect();
        let src = format!("pub struct {struct_name} {{\n{}\n}}", fields.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.fields.len(), count);
    }

    // 3. Struct field types survive token round-trip
    #[test]
    fn struct_field_type_roundtrip(
        struct_name in pascal_ident(),
        field_ty in leaf_type(),
    ) {
        let src = format!("pub struct {struct_name} {{ pub val: {field_ty}, }}");
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let f = parsed.fields.iter().next().unwrap();
        prop_assert_eq!(f.ty.to_token_stream().to_string(), field_ty);
    }

    // 4. Struct expansion in grammar module preserves struct identity
    #[test]
    fn struct_in_grammar_module_preserved(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
        struct_name in pascal_ident(),
        field_ty in leaf_type(),
    ) {
        let body = format!("    pub struct {struct_name} {{ pub val: {field_ty}, }}");
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        prop_assert_eq!(items.len(), 1);
        if let Item::Struct(s) = &items[0] {
            prop_assert_eq!(s.ident.to_string(), struct_name);
        } else {
            prop_assert!(false, "expected struct");
        }
    }
}

// ===========================================================================
// 2. Grammar expansion from enum — variants preserved
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 5. Enum with unit variants preserves variant count
    #[test]
    fn enum_variant_count_preserved(
        enum_name in pascal_ident(),
        count in 1usize..=8,
    ) {
        let variants: Vec<String> = (0..count)
            .map(|i| format!("    V{i},"))
            .collect();
        let src = format!("pub enum {enum_name} {{\n{}\n}}", variants.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.variants.len(), count);
    }

    // 6. Enum variant names are preserved in order
    #[test]
    fn enum_variant_names_in_order(
        enum_name in pascal_ident(),
        count in 1usize..=6,
    ) {
        let names: Vec<String> = (0..count).map(|i| format!("Var{i}")).collect();
        let variants: Vec<String> = names.iter().map(|n| format!("    {n},")).collect();
        let src = format!("pub enum {enum_name} {{\n{}\n}}", variants.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        for i in 0..count {
            prop_assert_eq!(parsed.variants[i].ident.to_string(), names[i].as_str());
        }
    }

    // 7. Enum with tuple variants preserves inner types
    #[test]
    fn enum_tuple_variant_types_preserved(
        enum_name in pascal_ident(),
        inner_ty in leaf_type(),
    ) {
        let src = format!("pub enum {enum_name} {{ A({inner_ty}), B({inner_ty}), }}");
        let parsed: ItemEnum = parse_str(&src).unwrap();
        for variant in &parsed.variants {
            let field = variant.fields.iter().next().unwrap();
            prop_assert_eq!(field.ty.to_token_stream().to_string(), inner_ty);
        }
    }

    // 8. Enum in grammar module is detected as Item::Enum
    #[test]
    fn enum_in_grammar_module(
        grammar_name in grammar_name_strategy(),
        mod_name in ident_strategy(),
        enum_name in pascal_ident(),
    ) {
        let body = format!("    pub enum {enum_name} {{ A, B, C, }}");
        let src = build_grammar_module(&grammar_name, &mod_name, &body);
        let parsed: ItemMod = parse_str(&src).unwrap();
        let items = &parsed.content.unwrap().1;
        prop_assert_eq!(items.len(), 1);
        prop_assert!(matches!(&items[0], Item::Enum(_)));
    }
}

// ===========================================================================
// 3. Field expansion to rules — try_extract, filter, wrap on field types
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 9. Field type extraction for target container yields inner type
    #[test]
    fn field_extract_yields_inner(
        ctr in container(),
        inner in leaf_type(),
    ) {
        let ty_str = format!("{ctr}<{inner}>");
        let ty: Type = parse_str(&ty_str).unwrap();
        let (result, extracted) = try_extract_inner_type(&ty, ctr, &skip_set(&[]));
        prop_assert!(extracted);
        prop_assert_eq!(type_to_string(&result), inner);
    }

    // 10. Non-matching container does not extract
    #[test]
    fn field_extract_non_matching_returns_original(
        inner in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("Option<{inner}>")).unwrap();
        let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip_set(&[]));
        prop_assert!(!extracted);
        prop_assert_eq!(type_to_string(&result), format!("Option < {inner} >"));
    }

    // 11. Filter strips wrapper when in skip set
    #[test]
    fn field_filter_strips_wrapper(
        wrapper in skip_name(),
        inner in leaf_type(),
    ) {
        let ty_str = format!("{wrapper}<{inner}>");
        let ty: Type = parse_str(&ty_str).unwrap();
        let filtered = filter_inner_type(&ty, &skip_set(&[wrapper]));
        prop_assert_eq!(type_to_string(&filtered), inner);
    }

    // 12. Filter does not strip when wrapper not in skip set
    #[test]
    fn field_filter_preserves_when_not_in_skip(
        inner in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("Box<{inner}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip_set(&[]));
        prop_assert_eq!(type_to_string(&filtered), format!("Box < {inner} >"));
    }

    // 13. Wrap adds WithLeaf to non-skip leaf types
    #[test]
    fn field_wrap_adds_with_leaf(inner in leaf_type()) {
        let ty: Type = parse_str(inner).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
        prop_assert_eq!(type_to_string(&wrapped), format!("adze :: WithLeaf < {inner} >"));
    }
}

// ===========================================================================
// 4. Optional field expansion — Option<T> extraction
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 14. Option<T> always extracts T
    #[test]
    fn optional_extracts_inner(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<{inner}>")).unwrap();
        let (result, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
        prop_assert!(ok);
        prop_assert_eq!(type_to_string(&result), inner);
    }

    // 15. Option through skip wrapper extracts inner
    #[test]
    fn optional_through_skip_extracts(
        wrapper in skip_name(),
        inner in leaf_type(),
    ) {
        let ty_str = format!("{wrapper}<Option<{inner}>>");
        let ty: Type = parse_str(&ty_str).unwrap();
        let (result, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&[wrapper]));
        prop_assert!(ok);
        prop_assert_eq!(type_to_string(&result), inner);
    }

    // 16. Nested Option<Option<T>> extracts outer Option's arg
    #[test]
    fn optional_nested_extracts_outermost(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<Option<{inner}>>")).unwrap();
        let (result, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
        prop_assert!(ok);
        prop_assert_eq!(type_to_string(&result), format!("Option < {inner} >"));
    }

    // 17. FieldThenParams with Option type preserves params
    #[test]
    fn optional_field_then_params_preserves(inner in leaf_type()) {
        let src = format!("Option<{inner}>, default = \"none\"");
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        let (_, ok) = try_extract_inner_type(&parsed.field.ty, "Option", &skip_set(&[]));
        prop_assert!(ok);
        prop_assert_eq!(parsed.params.len(), 1);
        prop_assert_eq!(parsed.params[0].path.to_string(), "default");
    }
}

// ===========================================================================
// 5. Vec field expansion — Vec<T> extraction
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 18. Vec<T> always extracts T
    #[test]
    fn vec_extracts_inner(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<{inner}>")).unwrap();
        let (result, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&[]));
        prop_assert!(ok);
        prop_assert_eq!(type_to_string(&result), inner);
    }

    // 19. Vec through skip wrapper extracts inner
    #[test]
    fn vec_through_skip_extracts(
        wrapper in skip_name(),
        inner in leaf_type(),
    ) {
        let ty_str = format!("{wrapper}<Vec<{inner}>>");
        let ty: Type = parse_str(&ty_str).unwrap();
        let (result, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&[wrapper]));
        prop_assert!(ok);
        prop_assert_eq!(type_to_string(&result), inner);
    }

    // 20. Vec<Option<T>> extracts Option<T> as the Vec element
    #[test]
    fn vec_of_option_extracts_option(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<Option<{inner}>>")).unwrap();
        let (result, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&[]));
        prop_assert!(ok);
        prop_assert_eq!(type_to_string(&result), format!("Option < {inner} >"));
    }

    // 21. FieldThenParams with Vec type and separator param
    #[test]
    fn vec_field_then_params_preserves(inner in leaf_type()) {
        let src = format!("Vec<{inner}>, separator = \",\"");
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        let (_, ok) = try_extract_inner_type(&parsed.field.ty, "Vec", &skip_set(&[]));
        prop_assert!(ok);
        prop_assert_eq!(parsed.params[0].path.to_string(), "separator");
    }
}

// ===========================================================================
// 6. Nested type expansion — multi-layer containers
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 22. Double-nested filter strips both layers
    #[test]
    fn nested_double_filter(
        w1 in skip_name(),
        w2 in skip_name(),
        inner in leaf_type(),
    ) {
        let ty_str = format!("{w1}<{w2}<{inner}>>");
        let ty: Type = parse_str(&ty_str).unwrap();
        let filtered = filter_inner_type(&ty, &skip_set(&[w1, w2]));
        prop_assert_eq!(type_to_string(&filtered), inner);
    }

    // 23. Wrap on container in skip set wraps inner but not container
    #[test]
    fn nested_wrap_skips_container(
        ctr in container(),
        inner in leaf_type(),
    ) {
        let ty_str = format!("{ctr}<{inner}>");
        let ty: Type = parse_str(&ty_str).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip_set(&[ctr]));
        let expected = format!("{ctr} < adze :: WithLeaf < {inner} > >");
        prop_assert_eq!(type_to_string(&wrapped), expected);
    }

    // 24. Extract then filter then wrap pipeline
    #[test]
    fn nested_pipeline_extract_filter_wrap(
        inner in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("Option<Box<{inner}>>")).unwrap();
        // Extract Option
        let (after_opt, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
        prop_assert!(ok);
        // Filter Box
        let filtered = filter_inner_type(&after_opt, &skip_set(&["Box"]));
        prop_assert_eq!(type_to_string(&filtered), inner);
        // Wrap
        let wrapped = wrap_leaf_type(&filtered, &skip_set(&[]));
        prop_assert_eq!(type_to_string(&wrapped), format!("adze :: WithLeaf < {inner} >"));
    }

    // 25. Triple-nested filter strips all three layers
    #[test]
    fn nested_triple_filter(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Arc<Rc<{inner}>>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip_set(&["Box", "Arc", "Rc"]));
        prop_assert_eq!(type_to_string(&filtered), inner);
    }
}

// ===========================================================================
// 7. Grammar expansion determinism
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 26. try_extract_inner_type is deterministic
    #[test]
    fn determinism_extract(
        ctr in container(),
        inner in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{ctr}<{inner}>")).unwrap();
        let (r1, e1) = try_extract_inner_type(&ty, ctr, &skip_set(&[]));
        let (r2, e2) = try_extract_inner_type(&ty, ctr, &skip_set(&[]));
        prop_assert_eq!(e1, e2);
        prop_assert_eq!(type_to_string(&r1), type_to_string(&r2));
    }

    // 27. filter_inner_type is deterministic
    #[test]
    fn determinism_filter(
        wrapper in skip_name(),
        inner in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{wrapper}<{inner}>")).unwrap();
        let wrapper_arr = [wrapper];
        let skip = skip_set(&wrapper_arr);
        let a = filter_inner_type(&ty, &skip);
        let b = filter_inner_type(&ty, &skip);
        prop_assert_eq!(type_to_string(&a), type_to_string(&b));
    }

    // 28. wrap_leaf_type is deterministic
    #[test]
    fn determinism_wrap(inner in leaf_type()) {
        let ty: Type = parse_str(inner).unwrap();
        let skip = skip_set(&[]);
        let a = wrap_leaf_type(&ty, &skip);
        let b = wrap_leaf_type(&ty, &skip);
        prop_assert_eq!(type_to_string(&a), type_to_string(&b));
    }

    // 29. Full pipeline (extract → filter → wrap) is deterministic
    #[test]
    fn determinism_pipeline(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<Box<{inner}>>")).unwrap();
        let run = || {
            let (after, _) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
            let filtered = filter_inner_type(&after, &skip_set(&["Box"]));
            type_to_string(&wrap_leaf_type(&filtered, &skip_set(&[])))
        };
        prop_assert_eq!(run(), run());
    }

    // 30. Struct parsing from source is deterministic
    #[test]
    fn determinism_struct_parse(
        name in pascal_ident(),
        ty in leaf_type(),
    ) {
        let src = format!("pub struct {name} {{ pub v: {ty}, }}");
        let a: ItemStruct = parse_str(&src).unwrap();
        let b: ItemStruct = parse_str(&src).unwrap();
        prop_assert_eq!(a.ident.to_string(), b.ident.to_string());
        prop_assert_eq!(a.fields.len(), b.fields.len());
    }
}

// ===========================================================================
// 8. Grammar expansion preserves all fields
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 31. All field names in a multi-field struct are preserved
    #[test]
    fn preserves_all_field_names(
        name in pascal_ident(),
        count in 1usize..=6,
        ty in leaf_type(),
    ) {
        let expected_names: Vec<String> = (0..count).map(|i| format!("field{i}")).collect();
        let fields: Vec<String> = expected_names
            .iter()
            .map(|n| format!("    pub {n}: {ty},"))
            .collect();
        let src = format!("pub struct {name} {{\n{}\n}}", fields.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let actual_names: Vec<String> = parsed
            .fields
            .iter()
            .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
            .collect();
        prop_assert_eq!(actual_names, expected_names);
    }

    // 32. All field types in a multi-field struct are preserved
    #[test]
    fn preserves_all_field_types(
        name in pascal_ident(),
        types in prop::collection::vec(leaf_type(), 1..=6),
    ) {
        let fields: Vec<String> = types
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("    pub f{i}: {ty},"))
            .collect();
        let src = format!("pub struct {name} {{\n{}\n}}", fields.join("\n"));
        let parsed: ItemStruct = parse_str(&src).unwrap();
        let actual_types: Vec<String> = parsed
            .fields
            .iter()
            .map(|f| f.ty.to_token_stream().to_string())
            .collect();
        for i in 0..types.len() {
            prop_assert_eq!(&actual_types[i], types[i]);
        }
    }

    // 33. Enum variant count matches input after parsing
    #[test]
    fn preserves_all_enum_variants(
        name in pascal_ident(),
        count in 1usize..=8,
    ) {
        let variants: Vec<String> = (0..count).map(|i| format!("    V{i},")).collect();
        let src = format!("pub enum {name} {{\n{}\n}}", variants.join("\n"));
        let parsed: ItemEnum = parse_str(&src).unwrap();
        prop_assert_eq!(parsed.variants.len(), count);
    }

    // 34. FieldThenParams preserves all named parameters
    #[test]
    fn preserves_field_params(
        ty in leaf_type(),
        param_count in 1usize..=3,
    ) {
        let param_names: Vec<String> = (0..param_count).map(|i| format!("p{i}")).collect();
        let params_str: String = param_names
            .iter()
            .map(|n| format!("{n} = 0"))
            .collect::<Vec<_>>()
            .join(", ");
        let src = format!("{ty}, {params_str}");
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert_eq!(parsed.params.len(), param_count);
        for i in 0..param_count {
            prop_assert_eq!(parsed.params[i].path.to_string(), param_names[i].as_str());
        }
    }

    // 35. Mixed optional and required fields: extract+wrap pipeline preserves semantics
    #[test]
    fn preserves_mixed_field_semantics(
        inner in leaf_type(),
    ) {
        let field_types: Vec<String> = vec![
            inner.to_string(),
            format!("Option<{inner}>"),
            format!("Vec<{inner}>"),
            format!("Box<{inner}>"),
        ];

        let wrap_skip = skip_set(&["Vec", "Option"]);
        let extract_skip = skip_set(&["Box"]);

        for i in 0..field_types.len() {
            let ty: Type = parse_str(&field_types[i]).unwrap();
            let (after_opt, is_opt) = try_extract_inner_type(&ty, "Option", &extract_skip);
            let (after_vec, is_vec) = try_extract_inner_type(&ty, "Vec", &extract_skip);

            match i {
                0 => {
                    // plain type: not optional, not vec
                    prop_assert!(!is_opt);
                    prop_assert!(!is_vec);
                }
                1 => {
                    // Option<T>: optional
                    prop_assert!(is_opt);
                    prop_assert_eq!(type_to_string(&after_opt), inner);
                }
                2 => {
                    // Vec<T>: is vec
                    prop_assert!(is_vec);
                    prop_assert_eq!(type_to_string(&after_vec), inner);
                }
                3 => {
                    // Box<T>: neither optional nor vec (Box in extract_skip but inner is T, not Option/Vec)
                    prop_assert!(!is_opt);
                    prop_assert!(!is_vec);
                }
                _ => unreachable!(),
            }

            // wrap always works
            let wrapped = wrap_leaf_type(&ty, &wrap_skip);
            let ws = type_to_string(&wrapped);
            prop_assert!(!ws.is_empty());
        }
    }
}
