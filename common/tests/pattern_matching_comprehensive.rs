#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for pattern matching behavior across all three core
//! functions: `try_extract_inner_type`, `filter_inner_type`, and `wrap_leaf_type`.
//!
//! Covers: primitive types, nested generics, multiple skip sets, non-path types,
//! identity/round-trip properties, deeply nested chains, multi-arg generics, and
//! edge cases around the interaction of all three functions.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use syn::{Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1. try_extract_inner_type — basic extraction
// ===========================================================================

#[test]
fn extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_vec_bool() {
    let ty: Type = parse_quote!(Vec<bool>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_returns_false_for_wrong_target() {
    let ty: Type = parse_quote!(Vec<u8>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Vec < u8 >");
}

#[test]
fn extract_plain_type_returns_false() {
    let ty: Type = parse_quote!(String);
    let (_inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
}

#[test]
fn extract_preserves_original_when_not_matched() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), ty_str(&ty));
}

// ===========================================================================
// 2. try_extract_inner_type — skip-over behavior
// ===========================================================================

#[test]
fn extract_through_box() {
    let ty: Type = parse_quote!(Box<Vec<u32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn extract_through_arc() {
    let ty: Type = parse_quote!(Arc<Option<f64>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn extract_through_two_skips() {
    let ty: Type = parse_quote!(Box<Arc<Vec<String>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_skip_but_target_not_inside() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < String >");
}

#[test]
fn extract_skip_stops_at_non_skip_non_target() {
    let ty: Type = parse_quote!(Box<HashMap<String, i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < HashMap < String , i32 > >");
}

// ===========================================================================
// 3. try_extract_inner_type — non-path types
// ===========================================================================

#[test]
fn extract_reference_type_returns_false() {
    let ty: Type = parse_quote!(&str);
    let (_inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
}

#[test]
fn extract_tuple_type_returns_false() {
    let ty: Type = parse_quote!((i32, u32));
    let (_inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
}

#[test]
fn extract_array_type_returns_false() {
    let ty: Type = parse_quote!([u8; 4]);
    let (_inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
}

#[test]
fn extract_slice_ref_returns_false() {
    let ty: Type = parse_quote!(&[u8]);
    let (_inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
}

#[test]
fn extract_fn_pointer_returns_false() {
    let ty: Type = parse_quote!(fn() -> i32);
    let (_inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
}

// ===========================================================================
// 4. filter_inner_type — single wrapper
// ===========================================================================

#[test]
fn filter_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "String");
}

#[test]
fn filter_arc_u64() {
    let ty: Type = parse_quote!(Arc<u64>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Arc"]))), "u64");
}

#[test]
fn filter_option_bool() {
    let ty: Type = parse_quote!(Option<bool>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Option"]))), "bool");
}

#[test]
fn filter_rc_f32() {
    let ty: Type = parse_quote!(Rc<f32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Rc"]))), "f32");
}

// ===========================================================================
// 5. filter_inner_type — nested wrappers
// ===========================================================================

#[test]
fn filter_box_arc_string() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Arc"]))),
        "String"
    );
}

#[test]
fn filter_three_deep() {
    let ty: Type = parse_quote!(Box<Arc<Rc<i32>>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]))),
        "i32"
    );
}

#[test]
fn filter_stops_at_non_skip_type() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    // Vec is NOT in skip set, so filter stops there
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "Vec < String >"
    );
}

#[test]
fn filter_no_skip_returns_original() {
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&[]))),
        "Box < String >"
    );
}

// ===========================================================================
// 6. filter_inner_type — non-path types
// ===========================================================================

#[test]
fn filter_reference_type_unchanged() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "& str");
}

#[test]
fn filter_tuple_type_unchanged() {
    let ty: Type = parse_quote!((i32, bool));
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "(i32 , bool)"
    );
}

#[test]
fn filter_array_type_unchanged() {
    let ty: Type = parse_quote!([u8; 16]);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "[u8 ; 16]"
    );
}

// ===========================================================================
// 7. wrap_leaf_type — basic wrapping
// ===========================================================================

#[test]
fn wrap_plain_string() {
    let ty: Type = parse_quote!(String);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < String >"
    );
}

#[test]
fn wrap_plain_i32() {
    let ty: Type = parse_quote!(i32);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < i32 >"
    );
}

#[test]
fn wrap_plain_bool() {
    let ty: Type = parse_quote!(bool);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < bool >"
    );
}

// ===========================================================================
// 8. wrap_leaf_type — skip containers wrap inner
// ===========================================================================

#[test]
fn wrap_vec_string_skips_vec() {
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "Vec < adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_option_i32_skips_option() {
    let ty: Type = parse_quote!(Option<i32>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Option"]))),
        "Option < adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_option_vec_nested_skip() {
    let ty: Type = parse_quote!(Option<Vec<Token>>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Option", "Vec"]))),
        "Option < Vec < adze :: WithLeaf < Token > > >"
    );
}

#[test]
fn wrap_vec_option_nested_skip() {
    let ty: Type = parse_quote!(Vec<Option<Leaf>>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec", "Option"]))),
        "Vec < Option < adze :: WithLeaf < Leaf > > >"
    );
}

// ===========================================================================
// 9. wrap_leaf_type — non-skip container gets wrapped entirely
// ===========================================================================

#[test]
fn wrap_hashmap_not_in_skip() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "adze :: WithLeaf < HashMap < String , i32 > >"
    );
}

#[test]
fn wrap_vec_not_in_skip_wraps_entirely() {
    let ty: Type = parse_quote!(Vec<u8>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < Vec < u8 > >"
    );
}

// ===========================================================================
// 10. wrap_leaf_type — non-path types
// ===========================================================================

#[test]
fn wrap_reference_type() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < & str >"
    );
}

#[test]
fn wrap_tuple_type() {
    let ty: Type = parse_quote!((i32, u32));
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < (i32 , u32) >"
    );
}

#[test]
fn wrap_array_type() {
    let ty: Type = parse_quote!([u8; 4]);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < [u8 ; 4] >"
    );
}

// ===========================================================================
// 11. wrap_leaf_type — multi-arg generics in skip set
// ===========================================================================

#[test]
fn wrap_result_in_skip_wraps_both_args() {
    let ty: Type = parse_quote!(Result<String, Error>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Result"]))),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < Error > >"
    );
}

#[test]
fn wrap_result_with_nested_option() {
    let ty: Type = parse_quote!(Result<Option<i32>, String>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Result", "Option"]))),
        "Result < Option < adze :: WithLeaf < i32 > > , adze :: WithLeaf < String > >"
    );
}

// ===========================================================================
// 12. Interaction: extract then filter
// ===========================================================================

#[test]
fn extract_then_filter_vec_box_string() {
    let ty: Type = parse_quote!(Vec<Box<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    // inner is Box<String>, now filter Box away
    let filtered = filter_inner_type(&inner, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn extract_through_skip_then_filter_identity() {
    let ty: Type = parse_quote!(Arc<Vec<u16>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Arc"]));
    assert!(ok);
    // inner is u16, filter with empty skip is identity
    let filtered = filter_inner_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&filtered), "u16");
}

// ===========================================================================
// 13. Interaction: extract then wrap
// ===========================================================================

#[test]
fn extract_then_wrap_leaf() {
    let ty: Type = parse_quote!(Option<Token>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Token >");
}

#[test]
fn extract_vec_then_wrap_with_option_skip() {
    let ty: Type = parse_quote!(Vec<Option<Leaf>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    // inner is Option<Leaf>
    let wrapped = wrap_leaf_type(&inner, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < Leaf > >");
}

// ===========================================================================
// 14. Interaction: filter then wrap
// ===========================================================================

#[test]
fn filter_then_wrap() {
    let ty: Type = parse_quote!(Box<Expr>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Expr");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Expr >");
}

#[test]
fn filter_nested_then_wrap_with_vec_skip() {
    let ty: Type = parse_quote!(Box<Arc<Vec<Stmt>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "Vec < Stmt >");
    let wrapped = wrap_leaf_type(&filtered, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < Stmt > >");
}

// ===========================================================================
// 15. Empty and single-element skip sets
// ===========================================================================

#[test]
fn extract_empty_skip_direct_match() {
    let ty: Type = parse_quote!(Option<u8>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_irrelevant_skip_still_matches() {
    let ty: Type = parse_quote!(Vec<f64>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Mutex", "RwLock"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn filter_empty_skip_preserves_type() {
    let ty: Type = parse_quote!(Option<String>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&[]))),
        "Option < String >"
    );
}

// ===========================================================================
// 16. Custom / user-defined type names
// ===========================================================================

#[test]
fn extract_custom_container() {
    let ty: Type = parse_quote!(MyList<Item>);
    let (inner, ok) = try_extract_inner_type(&ty, "MyList", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Item");
}

#[test]
fn filter_custom_container() {
    let ty: Type = parse_quote!(Wrapper<Inner>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Wrapper"]))),
        "Inner"
    );
}

#[test]
fn wrap_custom_skip_container() {
    let ty: Type = parse_quote!(Container<Leaf>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Container"]))),
        "Container < adze :: WithLeaf < Leaf > >"
    );
}

// ===========================================================================
// 17. NameValueExpr parsing
// ===========================================================================

#[test]
fn parse_nve_string_literal() {
    let nve: NameValueExpr = parse_quote!(name = "hello");
    assert_eq!(nve.path.to_string(), "name");
}

#[test]
fn parse_nve_integer_literal() {
    let nve: NameValueExpr = parse_quote!(count = 42);
    assert_eq!(nve.path.to_string(), "count");
}

#[test]
fn parse_nve_bool_literal() {
    let nve: NameValueExpr = parse_quote!(enabled = true);
    assert_eq!(nve.path.to_string(), "enabled");
}

#[test]
fn parse_nve_negative_number() {
    let nve: NameValueExpr = parse_quote!(offset = -1);
    assert_eq!(nve.path.to_string(), "offset");
}

#[test]
fn parse_nve_path_expr() {
    let nve: NameValueExpr = parse_quote!(kind = MyEnum::Variant);
    assert_eq!(nve.path.to_string(), "kind");
}

// ===========================================================================
// 18. FieldThenParams parsing
// ===========================================================================

#[test]
fn parse_ftp_type_only() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn parse_ftp_with_one_param() {
    let ftp: FieldThenParams = parse_quote!(i32, precedence = 5);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "precedence");
}

#[test]
fn parse_ftp_with_two_params() {
    let ftp: FieldThenParams = parse_quote!(Token, name = "plus", assoc = "left");
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "name");
    assert_eq!(ftp.params[1].path.to_string(), "assoc");
}

#[test]
fn parse_ftp_with_three_params() {
    let ftp: FieldThenParams = parse_quote!(Expr, prec = 1, assoc = "right", name = "power");
    assert_eq!(ftp.params.len(), 3);
    assert_eq!(ftp.params[2].path.to_string(), "name");
}

// ===========================================================================
// 19. Deeply nested extraction chains
// ===========================================================================

#[test]
fn extract_four_deep_skip() {
    // Box<Arc<Rc<Mutex<Vec<u8>>>>>
    let ty: Type = parse_quote!(Box<Arc<Rc<Mutex<Vec<u8>>>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc", "Rc", "Mutex"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn filter_four_deep() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Mutex<Token>>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc", "Mutex"]));
    assert_eq!(ty_str(&filtered), "Token");
}

// ===========================================================================
// 20. Idempotence / identity properties
// ===========================================================================

#[test]
fn filter_plain_type_is_identity() {
    let ty: Type = parse_quote!(u64);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "u64");
}

#[test]
fn extract_non_matching_preserves_type_exactly() {
    let ty: Type = parse_quote!(MyStruct);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "MyStruct");
}

#[test]
fn wrap_then_extract_gives_different_type() {
    // Wrapping and then extracting WithLeaf is a no-op if we could extract it,
    // but since WithLeaf is a path, extraction looks for "WithLeaf" not our target.
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    let (inner, ok) = try_extract_inner_type(&wrapped, "Vec", &skip(&[]));
    assert!(!ok);
    // The wrapped type is returned unchanged
    assert_eq!(ty_str(&inner), "adze :: WithLeaf < String >");
}

// ===========================================================================
// 21. Various primitive types
// ===========================================================================

#[test]
fn extract_vec_of_usize() {
    let ty: Type = parse_quote!(Vec<usize>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "usize");
}

#[test]
fn extract_option_of_char() {
    let ty: Type = parse_quote!(Option<char>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "char");
}

#[test]
fn wrap_f64_leaf() {
    let ty: Type = parse_quote!(f64);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < f64 >"
    );
}

#[test]
fn wrap_u128_leaf() {
    let ty: Type = parse_quote!(u128);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < u128 >"
    );
}

// ===========================================================================
// 22. Generic type as inner
// ===========================================================================

#[test]
fn extract_vec_of_generic() {
    let ty: Type = parse_quote!(Vec<HashMap<String, i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "HashMap < String , i32 >");
}

#[test]
fn filter_box_of_generic() {
    let ty: Type = parse_quote!(Box<Result<String, Error>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Result < String , Error >");
}

#[test]
fn wrap_skip_over_with_generic_inner() {
    let ty: Type = parse_quote!(Vec<HashMap<K, V>>);
    // Vec is skip, HashMap is not — wraps HashMap entirely
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "Vec < adze :: WithLeaf < HashMap < K , V > > >"
    );
}

// ===========================================================================
// 23. Same type in skip and target
// ===========================================================================

#[test]
fn extract_when_target_equals_skip_item() {
    // When Vec is both the target AND in skip, the target match takes priority
    // because the code checks target first.
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Vec"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

// ===========================================================================
// 24. NameValueExpr equality
// ===========================================================================

#[test]
fn nve_clone_equals_original() {
    let nve: NameValueExpr = parse_quote!(key = "value");
    let cloned = nve.clone();
    assert_eq!(nve, cloned);
}

#[test]
fn nve_debug_output_contains_path() {
    let nve: NameValueExpr = parse_quote!(mode = "fast");
    let debug = format!("{:?}", nve);
    assert!(debug.contains("NameValueExpr"));
}

// ===========================================================================
// 25. FieldThenParams edge cases
// ===========================================================================

#[test]
fn ftp_generic_field_type() {
    let ftp: FieldThenParams = parse_quote!(Vec<String>);
    assert!(ftp.params.is_empty());
    let field_ty = &ftp.field.ty;
    assert_eq!(ty_str(field_ty), "Vec < String >");
}

#[test]
fn ftp_with_generic_field_and_params() {
    let ftp: FieldThenParams = parse_quote!(Option<i32>, default = 0);
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "default");
}

#[test]
fn ftp_clone_equals_original() {
    let ftp: FieldThenParams = parse_quote!(u8, min = 0, max = 255);
    let cloned = ftp.clone();
    assert_eq!(ftp, cloned);
}

// ===========================================================================
// 26. Full pipeline: filter → extract → wrap
// ===========================================================================

#[test]
fn pipeline_box_vec_token() {
    let ty: Type = parse_quote!(Box<Vec<Token>>);
    // Step 1: filter off Box
    let no_box = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&no_box), "Vec < Token >");
    // Step 2: extract Vec inner
    let (inner, ok) = try_extract_inner_type(&no_box, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Token");
    // Step 3: wrap
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Token >");
}

#[test]
fn pipeline_arc_option_leaf() {
    let ty: Type = parse_quote!(Arc<Option<Leaf>>);
    let no_arc = filter_inner_type(&ty, &skip(&["Arc"]));
    assert_eq!(ty_str(&no_arc), "Option < Leaf >");
    let (inner, ok) = try_extract_inner_type(&no_arc, "Option", &skip(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Leaf >");
}

// ===========================================================================
// 27. Miscellaneous edge cases
// ===========================================================================

#[test]
fn extract_option_of_option() {
    let ty: Type = parse_quote!(Option<Option<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < i32 >");
}

#[test]
fn filter_box_of_box() {
    let ty: Type = parse_quote!(Box<Box<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn wrap_unit_type() {
    let ty: Type = parse_quote!(());
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < () >"
    );
}

#[test]
fn wrap_never_type() {
    let ty: Type = parse_quote!(!);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < ! >"
    );
}

#[test]
fn extract_with_lifetime_in_type() {
    let ty: Type = parse_quote!(Vec<&'a str>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "& 'a str");
}

#[test]
fn wrap_dyn_trait_type() {
    let ty: Type = parse_quote!(Box<dyn Fn()>);
    // Box is not in skip, so whole thing gets wrapped
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < Box < dyn Fn () > >"
    );
}

#[test]
fn wrap_box_dyn_with_box_skip() {
    let ty: Type = parse_quote!(Box<dyn Fn()>);
    // Box IS in skip, inner dyn Fn() is not a path type, so it gets wrapped
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Box"]))),
        "Box < adze :: WithLeaf < dyn Fn () > >"
    );
}
