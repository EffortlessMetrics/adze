#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for rule transformation and grammar rule manipulation
//! in adze-common.
//!
//! Exercises: `try_extract_inner_type`, `filter_inner_type`, `wrap_leaf_type`,
//! `NameValueExpr`, `FieldThenParams` — covering extraction chaining, filter
//! composition, wrap idempotency, deeply nested generics, mixed pipelines,
//! parameter parsing, and property-based invariants.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proptest::prelude::*;
use quote::ToTokens;
use syn::{Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn skip<'a>(names: &[&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn ts(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1. try_extract_inner_type — basic extraction
// ===========================================================================

#[test]
fn extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn extract_option_u32() {
    let ty: Type = parse_quote!(Option<u32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "u32");
}

#[test]
fn extract_target_mismatch_returns_original() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "Vec < i32 >");
}

#[test]
fn extract_plain_type_returns_unchanged() {
    let ty: Type = parse_quote!(MyStruct);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "MyStruct");
}

#[test]
fn extract_reference_type_returns_unchanged() {
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "& str");
}

#[test]
fn extract_tuple_type_returns_unchanged() {
    let ty: Type = parse_quote!((u8, u16));
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "(u8 , u16)");
}

// ===========================================================================
// 2. try_extract_inner_type — skip-over behaviour
// ===========================================================================

#[test]
fn extract_through_single_skip() {
    let ty: Type = parse_quote!(Box<Option<f64>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ts(&inner), "f64");
}

#[test]
fn extract_through_double_skip() {
    let ty: Type = parse_quote!(Arc<Box<Vec<u8>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Arc", "Box"]));
    assert!(ok);
    assert_eq!(ts(&inner), "u8");
}

#[test]
fn extract_skip_no_target_inside_returns_original() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(ts(&inner), "Box < String >");
}

#[test]
fn extract_skip_chain_stops_at_non_skip() {
    // Box is skip, Vec is NOT skip, so extraction cannot reach through Vec.
    let ty: Type = parse_quote!(Box<Vec<Option<bool>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(ts(&inner), "Box < Vec < Option < bool > > >");
}

// ===========================================================================
// 3. try_extract_inner_type — chained extraction
// ===========================================================================

#[test]
fn chained_extract_option_then_vec() {
    let ty: Type = parse_quote!(Option<Vec<Token>>);
    let (after_opt, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    let (after_vec, ok) = try_extract_inner_type(&after_opt, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&after_vec), "Token");
}

#[test]
fn chained_extract_vec_then_option() {
    let ty: Type = parse_quote!(Vec<Option<Node>>);
    let (after_vec, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    let (after_opt, ok) = try_extract_inner_type(&after_vec, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&after_opt), "Node");
}

#[test]
fn chained_extract_three_layers() {
    let ty: Type = parse_quote!(Option<Vec<Box<Leaf>>>);
    let (a, ok1) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok1);
    let (b, ok2) = try_extract_inner_type(&a, "Vec", &skip(&[]));
    assert!(ok2);
    let (c, ok3) = try_extract_inner_type(&b, "Box", &skip(&[]));
    assert!(ok3);
    assert_eq!(ts(&c), "Leaf");
}

// ===========================================================================
// 4. filter_inner_type — basic filtering
// ===========================================================================

#[test]
fn filter_box_reveals_inner() {
    let ty: Type = parse_quote!(Box<Payload>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip(&["Box"]))), "Payload");
}

#[test]
fn filter_arc_reveals_inner() {
    let ty: Type = parse_quote!(Arc<Data>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip(&["Arc"]))), "Data");
}

#[test]
fn filter_non_skip_type_unchanged() {
    let ty: Type = parse_quote!(Vec<u32>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip(&["Box"]))), "Vec < u32 >");
}

#[test]
fn filter_empty_skip_preserves_all() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Inner>>>);
    assert_eq!(
        ts(&filter_inner_type(&ty, &skip(&[]))),
        "Box < Arc < Rc < Inner > > >"
    );
}

#[test]
fn filter_nested_skips_peels_all() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Core>>>);
    assert_eq!(
        ts(&filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]))),
        "Core"
    );
}

#[test]
fn filter_stops_at_first_non_skip() {
    let ty: Type = parse_quote!(Box<Vec<Inner>>);
    assert_eq!(
        ts(&filter_inner_type(&ty, &skip(&["Box"]))),
        "Vec < Inner >"
    );
}

#[test]
fn filter_reference_type_unchanged() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(ts(&filter_inner_type(&ty, &skip(&["Box"]))), "& str");
}

#[test]
fn filter_tuple_type_unchanged() {
    let ty: Type = parse_quote!((i32, i64));
    assert_eq!(ts(&filter_inner_type(&ty, &skip(&["Box"]))), "(i32 , i64)");
}

// ===========================================================================
// 5. wrap_leaf_type — basic wrapping
// ===========================================================================

#[test]
fn wrap_simple_ident() {
    let ty: Type = parse_quote!(Foo);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < Foo >"
    );
}

#[test]
fn wrap_primitive_bool() {
    let ty: Type = parse_quote!(bool);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < bool >"
    );
}

#[test]
fn wrap_reference_wraps_whole() {
    let ty: Type = parse_quote!(&[u8]);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < & [u8] >"
    );
}

#[test]
fn wrap_array_type() {
    let ty: Type = parse_quote!([u8; 4]);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < [u8 ; 4] >"
    );
}

// ===========================================================================
// 6. wrap_leaf_type — skip containers
// ===========================================================================

#[test]
fn wrap_vec_skip_wraps_element() {
    let ty: Type = parse_quote!(Vec<Stmt>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "Vec < adze :: WithLeaf < Stmt > >"
    );
}

#[test]
fn wrap_option_skip_wraps_element() {
    let ty: Type = parse_quote!(Option<Clause>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&["Option"]))),
        "Option < adze :: WithLeaf < Clause > >"
    );
}

#[test]
fn wrap_nested_skips() {
    let ty: Type = parse_quote!(Option<Vec<Item>>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&["Option", "Vec"]))),
        "Option < Vec < adze :: WithLeaf < Item > > >"
    );
}

#[test]
fn wrap_skip_not_in_set_wraps_whole_container() {
    // Box is NOT in skip set → entire Box<X> gets wrapped.
    let ty: Type = parse_quote!(Box<Inner>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "adze :: WithLeaf < Box < Inner > >"
    );
}

#[test]
fn wrap_result_skip_wraps_both_args() {
    let ty: Type = parse_quote!(Result<Good, Bad>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&["Result"]))),
        "Result < adze :: WithLeaf < Good > , adze :: WithLeaf < Bad > >"
    );
}

#[test]
fn wrap_three_layer_skip() {
    let ty: Type = parse_quote!(Option<Vec<Box<Leaf>>>);
    assert_eq!(
        ts(&wrap_leaf_type(&ty, &skip(&["Option", "Vec", "Box"]))),
        "Option < Vec < Box < adze :: WithLeaf < Leaf > > > >"
    );
}

// ===========================================================================
// 7. Combined filter → wrap pipeline
// ===========================================================================

#[test]
fn pipeline_filter_then_wrap_box() {
    let ty: Type = parse_quote!(Box<Expr>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < Expr >");
}

#[test]
fn pipeline_filter_then_wrap_nested() {
    let ty: Type = parse_quote!(Arc<Box<Literal>>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc", "Box"]));
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < Literal >");
}

#[test]
fn pipeline_extract_filter_wrap() {
    let ty: Type = parse_quote!(Vec<Box<Statement>>);
    let (after_vec, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    let filtered = filter_inner_type(&after_vec, &skip(&["Box"]));
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < Statement >");
}

#[test]
fn pipeline_option_vec_box_full_strip() {
    let ty: Type = parse_quote!(Option<Vec<Box<Tok>>>);
    let (a, ok1) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok1);
    let (b, ok2) = try_extract_inner_type(&a, "Vec", &skip(&[]));
    assert!(ok2);
    let filtered = filter_inner_type(&b, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "Tok");
}

// ===========================================================================
// 8. NameValueExpr parsing
// ===========================================================================

#[test]
fn nve_string_value() {
    let nv: NameValueExpr = parse_quote!(name = "hello");
    assert_eq!(nv.path.to_string(), "name");
}

#[test]
fn nve_integer_value() {
    let nv: NameValueExpr = parse_quote!(count = 42);
    assert_eq!(nv.path.to_string(), "count");
}

#[test]
fn nve_negative_integer() {
    let nv: NameValueExpr = parse_quote!(offset = -3);
    assert_eq!(nv.path.to_string(), "offset");
    assert!(matches!(nv.expr, syn::Expr::Unary(_)));
}

#[test]
fn nve_bool_value() {
    let nv: NameValueExpr = parse_quote!(enabled = true);
    assert_eq!(nv.path.to_string(), "enabled");
}

#[test]
fn nve_path_value() {
    let nv: NameValueExpr = parse_quote!(kind = MyEnum::Variant);
    assert_eq!(nv.path.to_string(), "kind");
}

#[test]
fn nve_clone_equality() {
    let nv: NameValueExpr = parse_quote!(key = "val");
    let nv2 = nv.clone();
    assert_eq!(nv, nv2);
}

// ===========================================================================
// 9. FieldThenParams parsing
// ===========================================================================

#[test]
fn ftp_bare_type() {
    let ftp: FieldThenParams = parse_quote!(Identifier);
    assert_eq!(ts(&ftp.field.ty), "Identifier");
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_single_param() {
    let ftp: FieldThenParams = parse_quote!(Expr, precedence = 5);
    assert_eq!(ts(&ftp.field.ty), "Expr");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "precedence");
}

#[test]
fn ftp_multiple_params() {
    let ftp: FieldThenParams = parse_quote!(BinOp, precedence = 3, associativity = "left");
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "precedence");
    assert_eq!(ftp.params[1].path.to_string(), "associativity");
}

#[test]
fn ftp_generic_type_with_params() {
    let ftp: FieldThenParams = parse_quote!(Vec<Item>, separator = ",");
    assert_eq!(ts(&ftp.field.ty), "Vec < Item >");
    assert_eq!(ftp.params[0].path.to_string(), "separator");
}

#[test]
fn ftp_box_type_preserved() {
    let ftp: FieldThenParams = parse_quote!(Box<Block>);
    assert_eq!(ts(&ftp.field.ty), "Box < Block >");
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_clone_equality() {
    let ftp: FieldThenParams = parse_quote!(MyType, key = "val");
    let ftp2 = ftp.clone();
    assert_eq!(ftp, ftp2);
}

// ===========================================================================
// 10. Full expansion simulations
// ===========================================================================

#[test]
fn simulate_if_statement_expansion() {
    // struct IfStmt { cond: Expr, body: Box<Block>, else_: Option<Box<Block>> }
    let fields: Vec<(&str, Type)> = vec![
        ("cond", parse_quote!(Expr)),
        ("body", parse_quote!(Box<Block>)),
        ("else_", parse_quote!(Option<Box<Block>>)),
    ];
    let filter_skip = skip(&["Box"]);

    let mut result = Vec::new();
    for (name, ty) in &fields {
        let (opt_inner, is_opt) = try_extract_inner_type(ty, "Option", &filter_skip);
        let base = if is_opt { opt_inner } else { ty.clone() };
        let filtered = filter_inner_type(&base, &filter_skip);
        result.push((*name, is_opt, ts(&filtered)));
    }

    assert_eq!(result[0], ("cond", false, "Expr".to_string()));
    assert_eq!(result[1], ("body", false, "Block".to_string()));
    assert_eq!(result[2], ("else_", true, "Block".to_string()));
}

#[test]
fn simulate_function_params_with_separator() {
    let ftp: FieldThenParams = parse_quote!(Vec<Parameter>, separator = ",");
    let (inner, ok) = try_extract_inner_type(&ftp.field.ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Parameter");
    assert_eq!(ftp.params[0].path.to_string(), "separator");
}

#[test]
fn simulate_enum_choice_with_precedence() {
    // Simulating variants with different precedences
    let variants: Vec<FieldThenParams> = vec![
        parse_quote!(AddExpr, precedence = 1),
        parse_quote!(MulExpr, precedence = 2),
        parse_quote!(UnaryExpr, precedence = 3),
    ];
    for (i, v) in variants.iter().enumerate() {
        assert_eq!(v.params[0].path.to_string(), "precedence");
        if let syn::Expr::Lit(lit) = &v.params[0].expr {
            if let syn::Lit::Int(int_lit) = &lit.lit {
                assert_eq!(int_lit.base10_parse::<i32>().unwrap(), (i as i32) + 1);
            } else {
                panic!("Expected int literal");
            }
        } else {
            panic!("Expected literal expression");
        }
    }
}

// ===========================================================================
// 11. Edge cases and corner cases
// ===========================================================================

#[test]
fn extract_same_type_as_target() {
    // Type name matches the target exactly but has no generics => not a wrapper
    // parse_quote!(Vec) without angle brackets won't parse as Vec<T> — it's just an ident
    let ty: Type = parse_quote!(NotAnOption);
    let (_, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
}

#[test]
fn filter_deeply_nested_skip() {
    let ty: Type = parse_quote!(Box<Box<Box<Box<Core>>>>);
    assert_eq!(ts(&filter_inner_type(&ty, &skip(&["Box"]))), "Core");
}

#[test]
fn wrap_already_qualified_path() {
    let ty: Type = parse_quote!(std::string::String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < std :: string :: String >");
}

#[test]
fn extract_with_qualified_path_no_match() {
    // Qualified paths: last segment is checked.
    let ty: Type = parse_quote!(std::vec::Vec<u8>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "u8");
}

#[test]
fn filter_with_qualified_skip_type() {
    // filter_inner_type checks last segment, so std::boxed::Box should match "Box".
    let ty: Type = parse_quote!(std::boxed::Box<Payload>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "Payload");
}

#[test]
fn wrap_unit_type() {
    let ty: Type = parse_quote!(());
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    // () is a tuple type, not a path, so wraps the whole thing.
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < () >");
}

#[test]
fn extract_from_hashmap_not_vec() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "HashMap < String , i32 >");
}

#[test]
fn wrap_hashmap_in_skip_wraps_both_args() {
    let ty: Type = parse_quote!(HashMap<K, V>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["HashMap"]));
    assert_eq!(
        ts(&wrapped),
        "HashMap < adze :: WithLeaf < K > , adze :: WithLeaf < V > >"
    );
}

// ===========================================================================
// 12. Composition and idempotency
// ===========================================================================

#[test]
fn filter_idempotent_on_plain_type() {
    let ty: Type = parse_quote!(String);
    let first = filter_inner_type(&ty, &skip(&["Box"]));
    let second = filter_inner_type(&first, &skip(&["Box"]));
    assert_eq!(ts(&first), ts(&second));
}

#[test]
fn extract_idempotent_after_first_peel() {
    let ty: Type = parse_quote!(Option<Inner>);
    let (inner, ok1) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok1);
    // Inner is not Option, so second extract fails.
    let (same, ok2) = try_extract_inner_type(&inner, "Option", &skip(&[]));
    assert!(!ok2);
    assert_eq!(ts(&same), ts(&inner));
}

#[test]
fn wrap_then_extract_does_not_roundtrip() {
    // Wrapping adds adze::WithLeaf; extraction of "WithLeaf" would need it in skip.
    let ty: Type = parse_quote!(Token);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    // The wrapped type has path starting with `adze`, last segment `WithLeaf`.
    let (_, ok) = try_extract_inner_type(&wrapped, "WithLeaf", &skip(&[]));
    // Last segment of the path is "WithLeaf", so this should extract.
    assert!(ok);
}

// ===========================================================================
// 13. Batch processing patterns
// ===========================================================================

#[test]
fn batch_wrap_multiple_types() {
    let types: Vec<Type> = vec![
        parse_quote!(A),
        parse_quote!(B),
        parse_quote!(C),
        parse_quote!(D),
    ];
    let s = skip(&[]);
    let wrapped: Vec<String> = types.iter().map(|t| ts(&wrap_leaf_type(t, &s))).collect();
    assert!(wrapped.iter().all(|w| w.starts_with("adze :: WithLeaf <")));
    assert_eq!(wrapped.len(), 4);
    // All distinct
    let unique: HashSet<_> = wrapped.iter().collect();
    assert_eq!(unique.len(), 4);
}

#[test]
fn batch_filter_preserves_order() {
    let types: Vec<Type> = vec![
        parse_quote!(Box<Alpha>),
        parse_quote!(Box<Beta>),
        parse_quote!(Box<Gamma>),
    ];
    let s = skip(&["Box"]);
    let filtered: Vec<String> = types
        .iter()
        .map(|t| ts(&filter_inner_type(t, &s)))
        .collect();
    assert_eq!(filtered, vec!["Alpha", "Beta", "Gamma"]);
}

#[test]
fn batch_extract_mixed_success_failure() {
    let types: Vec<Type> = vec![
        parse_quote!(Option<A>),
        parse_quote!(Vec<B>),
        parse_quote!(Option<C>),
        parse_quote!(String),
    ];
    let results: Vec<bool> = types
        .iter()
        .map(|t| try_extract_inner_type(t, "Option", &skip(&[])).1)
        .collect();
    assert_eq!(results, vec![true, false, true, false]);
}

// ===========================================================================
// 14. Complex grammar simulation
// ===========================================================================

#[test]
fn simulate_binary_expr_rule() {
    // BinaryExpr: lhs: Box<Expr>, op: Operator, rhs: Box<Expr>
    let lhs: Type = parse_quote!(Box<Expr>);
    let op: Type = parse_quote!(Operator);
    let rhs: Type = parse_quote!(Box<Expr>);

    let fs = skip(&["Box"]);
    let ws = skip(&[]);
    let lhs_w = wrap_leaf_type(&filter_inner_type(&lhs, &fs), &ws);
    let op_w = wrap_leaf_type(&filter_inner_type(&op, &fs), &ws);
    let rhs_w = wrap_leaf_type(&filter_inner_type(&rhs, &fs), &ws);

    assert_eq!(ts(&lhs_w), "adze :: WithLeaf < Expr >");
    assert_eq!(ts(&op_w), "adze :: WithLeaf < Operator >");
    assert_eq!(ts(&rhs_w), "adze :: WithLeaf < Expr >");
}

#[test]
fn simulate_list_with_optional_trailing_comma() {
    // items: Vec<Item>, trailing_comma: Option<Punct>
    let items_ty: Type = parse_quote!(Vec<Item>);
    let comma_ty: Type = parse_quote!(Option<Punct>);

    let (item_inner, ok1) = try_extract_inner_type(&items_ty, "Vec", &skip(&[]));
    assert!(ok1);
    assert_eq!(ts(&item_inner), "Item");

    let (comma_inner, ok2) = try_extract_inner_type(&comma_ty, "Option", &skip(&[]));
    assert!(ok2);
    assert_eq!(ts(&comma_inner), "Punct");
}

#[test]
fn simulate_match_arm_expansion() {
    // pattern: Pattern, guard: Option<Box<Expr>>, body: Box<Expr>
    let pattern: Type = parse_quote!(Pattern);
    let guard: Type = parse_quote!(Option<Box<Expr>>);
    let body: Type = parse_quote!(Box<Expr>);

    let fs = skip(&["Box"]);

    // guard: extract Option, then filter Box
    let (guard_inner, ok) = try_extract_inner_type(&guard, "Option", &fs);
    assert!(ok);
    let guard_filtered = filter_inner_type(&guard_inner, &fs);
    assert_eq!(ts(&guard_filtered), "Expr");

    // body: just filter Box
    let body_filtered = filter_inner_type(&body, &fs);
    assert_eq!(ts(&body_filtered), "Expr");

    // pattern: no transformation needed
    assert_eq!(ts(&pattern), "Pattern");
}

// ===========================================================================
// 15. Property-based tests
// ===========================================================================

proptest! {
    #[test]
    fn prop_filter_empty_skip_is_identity(idx in 0..5usize) {
        let types: Vec<Type> = vec![
            parse_quote!(Foo),
            parse_quote!(Bar),
            parse_quote!(Vec<X>),
            parse_quote!(Box<Y>),
            parse_quote!(Option<Z>),
        ];
        let ty = &types[idx];
        let filtered = filter_inner_type(ty, &skip(&[]));
        prop_assert_eq!(ts(&filtered), ts(ty));
    }

    #[test]
    fn prop_extract_wrong_target_never_extracts(idx in 0..4usize) {
        let types: Vec<Type> = vec![
            parse_quote!(String),
            parse_quote!(i32),
            parse_quote!(bool),
            parse_quote!(MyType),
        ];
        let ty = &types[idx];
        let (_, ok) = try_extract_inner_type(ty, "Option", &skip(&[]));
        prop_assert!(!ok);
    }

    #[test]
    fn prop_wrap_always_produces_with_leaf(idx in 0..5usize) {
        let types: Vec<Type> = vec![
            parse_quote!(A),
            parse_quote!(i32),
            parse_quote!(bool),
            parse_quote!(String),
            parse_quote!(MyStruct),
        ];
        let ty = &types[idx];
        let wrapped = wrap_leaf_type(ty, &skip(&[]));
        let s = ts(&wrapped);
        prop_assert!(s.starts_with("adze :: WithLeaf <"), "got: {}", s);
    }

    #[test]
    fn prop_extract_option_always_succeeds_on_option(idx in 0..4usize) {
        let types: Vec<Type> = vec![
            parse_quote!(Option<A>),
            parse_quote!(Option<Vec<B>>),
            parse_quote!(Option<Box<C>>),
            parse_quote!(Option<Option<D>>),
        ];
        let ty = &types[idx];
        let (_, ok) = try_extract_inner_type(ty, "Option", &skip(&[]));
        prop_assert!(ok);
    }

    #[test]
    fn prop_filter_box_removes_exactly_one_layer(idx in 0..3usize) {
        // Box<T> with T not being Box → filter gives T.
        let types: Vec<Type> = vec![
            parse_quote!(Box<Alpha>),
            parse_quote!(Box<Beta>),
            parse_quote!(Box<Gamma>),
        ];
        let expected = ["Alpha", "Beta", "Gamma"];
        let ty = &types[idx];
        let filtered = filter_inner_type(ty, &skip(&["Box"]));
        prop_assert_eq!(ts(&filtered), expected[idx]);
    }

    #[test]
    fn prop_wrap_skip_preserves_outer_container(idx in 0..3usize) {
        let types: Vec<Type> = vec![
            parse_quote!(Vec<X>),
            parse_quote!(Option<Y>),
            parse_quote!(Vec<Z>),
        ];
        let containers = ["Vec", "Option", "Vec"];
        let ty = &types[idx];
        let wrapped = wrap_leaf_type(ty, &skip(&["Vec", "Option"]));
        let s = ts(&wrapped);
        prop_assert!(s.starts_with(containers[idx]), "got: {}", s);
    }
}
