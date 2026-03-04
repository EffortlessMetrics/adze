//! Edge-case tests for adze-common utility functions covering deeply nested
//! generics, exotic type forms, skip-set variations, and unusual identifiers.

use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use quote::ToTokens;
use syn::{Type, parse_quote};

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1. filter_inner_type — deeply nested generics (3+ levels)
// ===========================================================================

#[test]
fn filter_three_level_nesting_all_skipped() {
    let ty: Type = parse_quote!(Box<Option<Vec<Leaf>>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Option", "Vec"]))),
        "Leaf"
    );
}

#[test]
fn filter_five_level_nesting_all_skipped() {
    let ty: Type = parse_quote!(Arc<Box<Rc<Option<Vec<Core>>>>>);
    assert_eq!(
        ty_str(&filter_inner_type(
            &ty,
            &skip(&["Arc", "Box", "Rc", "Option", "Vec"])
        )),
        "Core"
    );
}

#[test]
fn filter_deep_nesting_stops_at_non_skip_middle() {
    // Box and Vec are skipped but HashMap is not — stops at HashMap
    let ty: Type = parse_quote!(Box<Vec<HashMap<String, i32>>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Vec"]))),
        "HashMap < String , i32 >"
    );
}

// ===========================================================================
// 2. filter_inner_type — multiple type parameters
// ===========================================================================

#[test]
fn filter_multi_param_generic_not_in_skip_unchanged() {
    let ty: Type = parse_quote!(HashMap<String, Vec<i32>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "HashMap < String , Vec < i32 > >"
    );
}

#[test]
fn filter_multi_param_in_skip_takes_first_arg() {
    // When Result is in skip, filter_inner_type drills into the first arg only
    let ty: Type = parse_quote!(Result<String, Error>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Result"]))),
        "String"
    );
}

// ===========================================================================
// 3. try_extract_inner_type — Result<T, E>
// ===========================================================================

#[test]
fn extract_result_as_target() {
    let ty: Type = parse_quote!(Result<Token, ParseError>);
    let (inner, ok) = try_extract_inner_type(&ty, "Result", &skip(&[]));
    assert!(ok);
    // Extracts first generic arg
    assert_eq!(ty_str(&inner), "Token");
}

#[test]
fn extract_result_through_box_skip() {
    let ty: Type = parse_quote!(Box<Result<Value, Err>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Result", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Value");
}

#[test]
fn extract_result_not_target_returns_unchanged() {
    let ty: Type = parse_quote!(Result<String, Error>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Result < String , Error >");
}

// ===========================================================================
// 4. try_extract_inner_type — custom wrapper types
// ===========================================================================

#[test]
fn extract_custom_wrapper_as_target() {
    let ty: Type = parse_quote!(Spanned<Identifier>);
    let (inner, ok) = try_extract_inner_type(&ty, "Spanned", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Identifier");
}

#[test]
fn extract_custom_wrapper_in_skip_set() {
    let ty: Type = parse_quote!(Spanned<Option<Token>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Spanned"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Token");
}

#[test]
fn extract_custom_wrapper_nested_skip() {
    let ty: Type = parse_quote!(MyBox<Wrapper<Target<Leaf>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Target", &skip(&["MyBox", "Wrapper"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Leaf");
}

// ===========================================================================
// 5. wrap_leaf_type — associated types (path types)
// ===========================================================================

#[test]
fn wrap_associated_type_path() {
    // <T as Trait>::Output parses as a Type::Path with QSelf
    let ty: Type = parse_quote!(<T as Iterator>::Item);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < < T as Iterator > :: Item >"
    );
}

#[test]
fn wrap_simple_associated_type() {
    let ty: Type = parse_quote!(Self::Output);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < Self :: Output >"
    );
}

// ===========================================================================
// 6. wrap_leaf_type — trait objects (dyn Trait)
// ===========================================================================

#[test]
fn wrap_dyn_trait_object() {
    let ty: Type = parse_quote!(dyn Display);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < dyn Display >"
    );
}

#[test]
fn wrap_box_dyn_trait_in_skip() {
    let ty: Type = parse_quote!(Box<dyn Error>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Box"]))),
        "Box < adze :: WithLeaf < dyn Error > >"
    );
}

#[test]
fn filter_box_dyn_trait_strips_box() {
    let ty: Type = parse_quote!(Box<dyn Send>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "dyn Send");
}

// ===========================================================================
// 7. Type names with paths (std::collections::HashMap)
// ===========================================================================

#[test]
fn filter_fully_qualified_path_not_in_skip() {
    let ty: Type = parse_quote!(std::collections::HashMap<String, i32>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "std :: collections :: HashMap < String , i32 >"
    );
}

#[test]
fn filter_qualified_path_last_segment_matches_skip() {
    // skip set matches on last segment only
    let ty: Type = parse_quote!(std::boxed::Box<Inner>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "Inner");
}

#[test]
fn wrap_qualified_path_not_in_skip() {
    let ty: Type = parse_quote!(std::vec::Vec<Node>);
    // "Vec" is in skip so it should drill in
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "std :: vec :: Vec < adze :: WithLeaf < Node > >"
    );
}

// ===========================================================================
// 8. Type names with lifetimes
// ===========================================================================

#[test]
fn wrap_reference_with_lifetime() {
    let ty: Type = parse_quote!(&'a str);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < & 'a str >"
    );
}

#[test]
fn filter_reference_with_lifetime_passthrough() {
    let ty: Type = parse_quote!(&'static [u8]);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "& 'static [u8]"
    );
}

#[test]
fn extract_with_lifetime_in_generic() {
    let ty: Type = parse_quote!(Option<&'a str>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "& 'a str");
}

// ===========================================================================
// 9. Type names with const generics
// ===========================================================================

#[test]
fn wrap_const_generic_array_type() {
    let ty: Type = parse_quote!([u8; 32]);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < [u8 ; 32] >"
    );
}

#[test]
fn filter_const_generic_array_passthrough() {
    let ty: Type = parse_quote!([f64; 100]);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "[f64 ; 100]"
    );
}

// ===========================================================================
// 10. Empty skip sets vs populated skip sets
// ===========================================================================

#[test]
fn empty_skip_set_filter_preserves_all_wrappers() {
    let ty: Type = parse_quote!(Box<Option<Vec<Token>>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&[]))),
        "Box < Option < Vec < Token > > >"
    );
}

#[test]
fn empty_skip_set_wrap_wraps_entire_generic() {
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < Vec < String > >"
    );
}

#[test]
fn empty_skip_set_extract_finds_direct_target() {
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn populated_skip_set_extract_skips_through_layers() {
    let ty: Type = parse_quote!(Arc<Box<Rc<Option<Leaf>>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc", "Box", "Rc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Leaf");
}

// ===========================================================================
// 11. Unicode in type names
// ===========================================================================

#[test]
fn unicode_type_name_filter() {
    let ty: Type = parse_quote!(Box<Ñoño>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "Ñoño");
}

#[test]
fn unicode_type_name_wrap() {
    let ty: Type = parse_quote!(Ärger);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < Ärger >"
    );
}

#[test]
fn unicode_cjk_in_generic() {
    let ty: Type = parse_quote!(Option<型>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "型");
}

// ===========================================================================
// 12. Very long type names (100+ characters)
// ===========================================================================

#[test]
fn long_type_name_filter() {
    let long = "A".repeat(120);
    let ident = syn::Ident::new(&long, proc_macro2::Span::call_site());
    let ty: Type = parse_quote!(Box<#ident>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), long);
}

#[test]
fn long_type_name_wrap() {
    let long = "Z".repeat(150);
    let ident = syn::Ident::new(&long, proc_macro2::Span::call_site());
    let ty: Type = parse_quote!(#ident);
    let wrapped = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert!(wrapped.contains(&long));
    assert!(wrapped.starts_with("adze :: WithLeaf"));
}

#[test]
fn long_type_name_extract() {
    let long = "B".repeat(200);
    let ident = syn::Ident::new(&long, proc_macro2::Span::call_site());
    let ty: Type = parse_quote!(Vec<#ident>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), long);
}

// ===========================================================================
// 13. Recursive type definitions
// ===========================================================================

#[test]
fn recursive_type_box_self_ref() {
    // Box<Expr> where Expr is recursive — filter should just strip Box
    let skip_set = skip(&["Box"]);
    let ty: Type = parse_quote!(Box<Expr>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip_set)), "Expr");
}

#[test]
fn recursive_type_option_box_chained() {
    let ty: Type = parse_quote!(Option<Box<Expr>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Box < Expr >");

    // Then filter Box away
    assert_eq!(ty_str(&filter_inner_type(&inner, &skip(&["Box"]))), "Expr");
}

#[test]
fn recursive_type_vec_of_self_wrap() {
    let ty: Type = parse_quote!(Vec<Statement>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "Vec < adze :: WithLeaf < Statement > >"
    );
}

// ===========================================================================
// 14. Tuple types
// ===========================================================================

#[test]
fn tuple_type_filter_passthrough() {
    let ty: Type = parse_quote!((i32, String, bool));
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "(i32 , String , bool)"
    );
}

#[test]
fn tuple_type_wrap() {
    let ty: Type = parse_quote!((u8, u16));
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < (u8 , u16) >"
    );
}

#[test]
fn tuple_type_extract_no_match() {
    let ty: Type = parse_quote!((A, B));
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "(A , B)");
}

// ===========================================================================
// 15. Array types [T; N]
// ===========================================================================

#[test]
fn array_type_filter_passthrough() {
    let ty: Type = parse_quote!([u8; 64]);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "[u8 ; 64]"
    );
}

#[test]
fn array_type_wrap() {
    let ty: Type = parse_quote!([f32; 3]);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < [f32 ; 3] >"
    );
}

#[test]
fn array_type_extract_no_match() {
    let ty: Type = parse_quote!([Token; 10]);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "[Token ; 10]");
}

// ===========================================================================
// 16. Reference types &T, &mut T
// ===========================================================================

#[test]
fn ref_type_filter_passthrough() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "& str");
}

#[test]
fn ref_mut_type_wrap() {
    let ty: Type = parse_quote!(&mut Vec<u8>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < & mut Vec < u8 > >"
    );
}

#[test]
fn ref_type_extract_no_match() {
    let ty: Type = parse_quote!(&i32);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& i32");
}

// ===========================================================================
// 17. Raw pointer types *const T, *mut T
// ===========================================================================

#[test]
fn raw_ptr_const_wrap() {
    let ty: Type = parse_quote!(*const u8);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < * const u8 >"
    );
}

#[test]
fn raw_ptr_mut_wrap() {
    let ty: Type = parse_quote!(*mut Node);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < * mut Node >"
    );
}

#[test]
fn raw_ptr_filter_passthrough() {
    let ty: Type = parse_quote!(*const c_void);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "* const c_void"
    );
}

#[test]
fn raw_ptr_extract_no_match() {
    let ty: Type = parse_quote!(*mut u8);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "* mut u8");
}
