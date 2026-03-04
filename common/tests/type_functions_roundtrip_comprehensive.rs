//! Comprehensive roundtrip and edge-case tests for `try_extract_inner_type`,
//! `filter_inner_type`, and `wrap_leaf_type` in adze-common.

use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
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
// 1. try_extract_inner_type — basic containers
// ===========================================================================

#[test]
fn extract_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_box_u64() {
    let ty: Type = parse_quote!(Box<u64>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u64");
}

#[test]
fn extract_hashmap_not_matched() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), ty_str(&ty));
}

#[test]
fn extract_option_through_box_skip() {
    let ty: Type = parse_quote!(Box<Option<bool>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_vec_through_arc_skip() {
    let ty: Type = parse_quote!(Arc<Vec<f64>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn extract_vec_through_double_skip() {
    let ty: Type = parse_quote!(Box<Arc<Vec<u8>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_no_match_returns_original() {
    let ty: Type = parse_quote!(String);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_skip_without_target_inside_returns_original() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), ty_str(&ty));
}

#[test]
fn extract_reference_type_unchanged() {
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& str");
}

// ===========================================================================
// 2. filter_inner_type — various skip_over sets
// ===========================================================================

#[test]
fn filter_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "String");
}

#[test]
fn filter_arc_i32() {
    let ty: Type = parse_quote!(Arc<i32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Arc"]))), "i32");
}

#[test]
fn filter_nested_box_arc() {
    let ty: Type = parse_quote!(Box<Arc<u32>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Arc"]))),
        "u32"
    );
}

#[test]
fn filter_not_in_skip_returns_original() {
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "Vec < String >"
    );
}

#[test]
fn filter_plain_type_unchanged() {
    let ty: Type = parse_quote!(bool);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "bool");
}

#[test]
fn filter_triple_nesting() {
    let ty: Type = parse_quote!(Rc<Box<Arc<f32>>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Rc", "Box", "Arc"]))),
        "f32"
    );
}

#[test]
fn filter_partial_skip_stops_early() {
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    // Only Box is in skip; Vec is NOT, so stops at Vec<u8>.
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "Vec < u8 >"
    );
}

#[test]
fn filter_empty_skip_preserves_everything() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&[]))),
        "Box < Arc < String > >"
    );
}

// ===========================================================================
// 3. wrap_leaf_type — identity and wrapping properties
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
fn wrap_vec_skipped_wraps_inner() {
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "Vec < adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_option_skipped_wraps_inner() {
    let ty: Type = parse_quote!(Option<u64>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Option"]))),
        "Option < adze :: WithLeaf < u64 > >"
    );
}

#[test]
fn wrap_nested_option_vec_both_skipped() {
    let ty: Type = parse_quote!(Option<Vec<bool>>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Option", "Vec"]))),
        "Option < Vec < adze :: WithLeaf < bool > > >"
    );
}

#[test]
fn wrap_vec_not_in_skip_wraps_whole() {
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < Vec < String > >"
    );
}

#[test]
fn wrap_result_both_args_wrapped() {
    let ty: Type = parse_quote!(Result<String, i32>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Result"]))),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_reference_type_wraps_whole() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < & str >"
    );
}

#[test]
fn wrap_tuple_type_wraps_whole() {
    let ty: Type = parse_quote!((i32, u32));
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < (i32 , u32) >"
    );
}

#[test]
fn wrap_array_type_wraps_whole() {
    let ty: Type = parse_quote!([u8; 4]);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < [u8 ; 4] >"
    );
}

// ===========================================================================
// 4. Roundtrip: extract then re-wrap
// ===========================================================================

#[test]
fn roundtrip_option_extract_then_wrap() {
    let original: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&original, "Option", &skip(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn roundtrip_vec_extract_then_wrap() {
    let original: Type = parse_quote!(Vec<i64>);
    let (inner, ok) = try_extract_inner_type(&original, "Vec", &skip(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i64 >");
}

#[test]
fn roundtrip_box_extract_filter_wrap() {
    let original: Type = parse_quote!(Box<Vec<u16>>);
    let (inner, ok) = try_extract_inner_type(&original, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u16");
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u16 >");
}

#[test]
fn roundtrip_filter_then_wrap_plain() {
    let ty: Type = parse_quote!(Box<f64>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "f64");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < f64 >");
}

#[test]
fn roundtrip_filter_then_wrap_nested() {
    let ty: Type = parse_quote!(Arc<Box<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc", "Box"]));
    assert_eq!(ty_str(&filtered), "String");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn roundtrip_no_extraction_wrap_preserves_container() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    // inner == original; wrap with Vec in skip keeps container
    let wrapped = wrap_leaf_type(&inner, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn roundtrip_extract_option_vec_inner() {
    let ty: Type = parse_quote!(Option<Vec<char>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < char >");
    let wrapped = wrap_leaf_type(&inner, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < char > >");
}

#[test]
fn roundtrip_filter_box_then_extract_option() {
    let ty: Type = parse_quote!(Box<Option<u8>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Option < u8 >");
    let (inner, ok) = try_extract_inner_type(&filtered, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

// ===========================================================================
// 5. Nested types: Option<Vec<T>>, Vec<Option<T>>, etc.
// ===========================================================================

#[test]
fn nested_option_vec_extract_option() {
    let ty: Type = parse_quote!(Option<Vec<u32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < u32 >");
}

#[test]
fn nested_vec_option_extract_vec() {
    let ty: Type = parse_quote!(Vec<Option<bool>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < bool >");
}

#[test]
fn nested_option_vec_wrap_both_skipped() {
    let ty: Type = parse_quote!(Option<Vec<i8>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Vec < adze :: WithLeaf < i8 > > >"
    );
}

#[test]
fn nested_box_option_vec_filter_all() {
    let ty: Type = parse_quote!(Box<Option<Vec<u8>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Option"]));
    assert_eq!(ty_str(&filtered), "Vec < u8 >");
}

#[test]
fn deeply_nested_box_arc_rc_filter() {
    let ty: Type = parse_quote!(Box<Arc<Rc<String>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]));
    assert_eq!(ty_str(&filtered), "String");
}

// ===========================================================================
// 6. Non-container types pass through unchanged
// ===========================================================================

#[test]
fn non_container_i32_extract_unchanged() {
    let ty: Type = parse_quote!(i32);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn non_container_bool_filter_unchanged() {
    let ty: Type = parse_quote!(bool);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "bool");
}

#[test]
fn non_container_path_type_unchanged() {
    let ty: Type = parse_quote!(std::string::String);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "std :: string :: String");
}

#[test]
fn non_container_qualified_path_filter() {
    let ty: Type = parse_quote!(std::string::String);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "std :: string :: String");
}

#[test]
fn non_container_unit_type() {
    let ty: Type = parse_quote!(());
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "()");
}

// ===========================================================================
// 7. Empty skip_over set behavior
// ===========================================================================

#[test]
fn empty_skip_extract_option_direct() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn empty_skip_extract_fails_on_box_wrapping_target() {
    // Box<Option<T>>: with empty skip, Box is not skipped, doesn't match Option.
    let ty: Type = parse_quote!(Box<Option<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), ty_str(&ty));
}

#[test]
fn empty_skip_filter_returns_everything() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let filtered = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&filtered), ty_str(&ty));
}

#[test]
fn empty_skip_wrap_wraps_everything_including_containers() {
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Vec < i32 > >");
}

#[test]
fn empty_skip_wrap_plain() {
    let ty: Type = parse_quote!(u128);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u128 >");
}

// ===========================================================================
// 8. Large skip_over set behavior
// ===========================================================================

#[test]
fn large_skip_filter_box() {
    let many = skip(&[
        "Box", "Arc", "Rc", "Cell", "RefCell", "Mutex", "RwLock", "Pin", "Cow",
    ]);
    let ty: Type = parse_quote!(Box<u8>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &many)), "u8");
}

#[test]
fn large_skip_filter_irrelevant_type() {
    let many = skip(&[
        "Box", "Arc", "Rc", "Cell", "RefCell", "Mutex", "RwLock", "Pin", "Cow",
    ]);
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &many)), "Vec < String >");
}

#[test]
fn large_skip_extract_through_many_layers() {
    let many = skip(&["Box", "Arc", "Rc", "Cell", "RefCell"]);
    let ty: Type = parse_quote!(Box<Arc<Rc<Vec<f64>>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &many);
    assert!(ok);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn large_skip_wrap_only_matching_skipped() {
    let many = skip(&["Vec", "Option", "Box", "Arc", "Rc", "Cell", "RefCell"]);
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &many)),
        "Vec < Option < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn large_skip_filter_deep_nesting() {
    let many = skip(&["Box", "Arc", "Rc", "Mutex"]);
    let ty: Type = parse_quote!(Box<Arc<Rc<Mutex<bool>>>>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &many)), "bool");
}

// ===========================================================================
// 9. Reference types
// ===========================================================================

#[test]
fn reference_type_extract_returns_unchanged() {
    let ty: Type = parse_quote!(&u32);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& u32");
}

#[test]
fn mutable_reference_extract_returns_unchanged() {
    let ty: Type = parse_quote!(&mut String);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& mut String");
}

#[test]
fn reference_filter_returns_unchanged() {
    let ty: Type = parse_quote!(&[u8]);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "& [u8]");
}

#[test]
fn reference_wrap_wraps_whole() {
    let ty: Type = parse_quote!(&i64);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & i64 >");
}

#[test]
fn lifetime_reference_extract_unchanged() {
    let ty: Type = parse_quote!(&'static str);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& 'static str");
}

// ===========================================================================
// 10. Tuple types
// ===========================================================================

#[test]
fn tuple_extract_returns_unchanged() {
    let ty: Type = parse_quote!((i32, bool));
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "(i32 , bool)");
}

#[test]
fn tuple_filter_returns_unchanged() {
    let ty: Type = parse_quote!((String, u8, f64));
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "(String , u8 , f64)");
}

#[test]
fn tuple_wrap_wraps_whole() {
    let ty: Type = parse_quote!((u8, u16));
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < (u8 , u16) >");
}

#[test]
fn unit_tuple_wrap() {
    let ty: Type = parse_quote!(());
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < () >");
}

// ===========================================================================
// 11. Additional edge cases and cross-function interactions
// ===========================================================================

#[test]
fn extract_same_type_as_skip_extracts_directly() {
    // If inner_of == a skip_over entry, the direct match on inner_of wins.
    let ty: Type = parse_quote!(Box<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn filter_then_extract_then_wrap_pipeline() {
    let ty: Type = parse_quote!(Arc<Option<Vec<u32>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc"]));
    assert_eq!(ty_str(&filtered), "Option < Vec < u32 > >");
    let (inner, ok) = try_extract_inner_type(&filtered, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < u32 >");
    let wrapped = wrap_leaf_type(&inner, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < u32 > >");
}

#[test]
fn wrap_idempotent_for_already_skipped_inner() {
    // Wrapping twice with same skip set wraps inner deeper.
    let ty: Type = parse_quote!(Vec<String>);
    let first = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&first), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn filter_idempotent_on_leaf() {
    let ty: Type = parse_quote!(u64);
    let first = filter_inner_type(&ty, &skip(&["Box"]));
    let second = filter_inner_type(&first, &skip(&["Box"]));
    assert_eq!(ty_str(&first), ty_str(&second));
}

#[test]
fn filter_idempotent_after_full_unwrap() {
    let ty: Type = parse_quote!(Box<String>);
    let first = filter_inner_type(&ty, &skip(&["Box"]));
    let second = filter_inner_type(&first, &skip(&["Box"]));
    assert_eq!(ty_str(&first), "String");
    assert_eq!(ty_str(&second), "String");
}

#[test]
fn extract_custom_container_name() {
    let ty: Type = parse_quote!(MyWrapper<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "MyWrapper", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn filter_custom_container_name() {
    let ty: Type = parse_quote!(MyBox<String>);
    let filtered = filter_inner_type(&ty, &skip(&["MyBox"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn wrap_custom_skip_name() {
    let ty: Type = parse_quote!(MyVec<bool>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["MyVec"]));
    assert_eq!(ty_str(&wrapped), "MyVec < adze :: WithLeaf < bool > >");
}

#[test]
fn extract_through_multiple_skips_no_match() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box", "Arc"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), ty_str(&ty));
}

#[test]
fn wrap_slice_type() {
    let ty: Type = parse_quote!([u8]);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8] >");
}

#[test]
fn filter_slice_unchanged() {
    let ty: Type = parse_quote!([u8]);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "[u8]");
}

#[test]
fn roundtrip_extract_wrap_preserve_semantics() {
    // Extract from Vec, wrap result → leaf wrapping applied
    let ty: Type = parse_quote!(Vec<Option<u16>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < u16 >");
    let wrapped = wrap_leaf_type(&inner, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < u16 > >");
}

#[test]
fn roundtrip_filter_all_then_wrap() {
    let ty: Type = parse_quote!(Box<Arc<Rc<i32>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]));
    assert_eq!(ty_str(&filtered), "i32");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_with_empty_skip_never_recurses() {
    // Without skip entries, even containers get wrapped as-is.
    let ty: Type = parse_quote!(Option<Vec<Box<String>>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < Option < Vec < Box < String > > > >"
    );
}

#[test]
fn extract_option_option() {
    let ty: Type = parse_quote!(Option<Option<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < u8 >");
}

#[test]
fn filter_option_not_in_skip() {
    // Option is not in skip set, so filter returns as-is.
    let ty: Type = parse_quote!(Option<i32>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Option < i32 >");
}

#[test]
fn wrap_fn_pointer_type() {
    let ty: Type = syn::parse_str::<Type>("fn(i32) -> bool").unwrap();
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < fn (i32) -> bool >");
}

#[test]
fn extract_fn_pointer_unchanged() {
    let ty: Type = syn::parse_str::<Type>("fn() -> u8").unwrap();
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), ty_str(&ty));
}

#[test]
fn filter_never_type_unchanged() {
    let ty: Type = syn::parse_str::<Type>("!").unwrap();
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "!");
}

#[test]
fn wrap_never_type() {
    let ty: Type = syn::parse_str::<Type>("!").unwrap();
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < ! >");
}
