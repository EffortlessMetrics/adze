//! Deep edge-case tests for `try_extract_inner_type`, `filter_inner_type`,
//! and `wrap_leaf_type` from the `adze_common` crate.

use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use quote::ToTokens;
use syn::Type;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn ty(s: &str) -> Type {
    syn::parse_str::<Type>(s).unwrap_or_else(|e| panic!("failed to parse `{s}`: {e}"))
}

fn to_s(t: &Type) -> String {
    t.to_token_stream().to_string()
}

fn empty() -> HashSet<&'static str> {
    HashSet::new()
}

fn skip(names: &[&'static str]) -> HashSet<&'static str> {
    names.iter().copied().collect()
}

// =========================================================================
// 1 – Triple-nested Option<Option<Option<T>>>
// =========================================================================

#[test]
fn extract_triple_nested_option_outer() {
    let t = ty("Option<Option<Option<i32>>>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(ok);
    assert_eq!(to_s(&inner), "Option < Option < i32 > >");
}

#[test]
fn extract_triple_nested_option_with_skip() {
    // skip Option to reach inner Option, then extract
    let t = ty("Option<Option<Option<i32>>>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&["Option"]));
    // The outermost matches target "Option" directly, so skip_over is irrelevant
    assert!(ok);
    assert_eq!(to_s(&inner), "Option < Option < i32 > >");
}

#[test]
fn filter_triple_nested_option() {
    let t = ty("Option<Option<Option<i32>>>");
    let f = filter_inner_type(&t, &skip(&["Option"]));
    assert_eq!(to_s(&f), "i32");
}

#[test]
fn wrap_triple_nested_option() {
    let t = ty("Option<Option<Option<i32>>>");
    let w = wrap_leaf_type(&t, &skip(&["Option"]));
    assert_eq!(
        to_s(&w),
        "Option < Option < Option < adze :: WithLeaf < i32 > > > >"
    );
}

// =========================================================================
// 2 – Mixed nesting: Option<Vec<Box<T>>>
// =========================================================================

#[test]
fn extract_mixed_option_vec_box_target_option() {
    let t = ty("Option<Vec<Box<i32>>>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(ok);
    assert_eq!(to_s(&inner), "Vec < Box < i32 > >");
}

#[test]
fn extract_mixed_target_vec_skip_option() {
    let t = ty("Option<Vec<Box<i32>>>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&["Option"]));
    assert!(ok);
    assert_eq!(to_s(&inner), "Box < i32 >");
}

#[test]
fn extract_mixed_target_box_skip_option_vec() {
    let t = ty("Option<Vec<Box<i32>>>");
    let (inner, ok) = try_extract_inner_type(&t, "Box", &skip(&["Option", "Vec"]));
    assert!(ok);
    assert_eq!(to_s(&inner), "i32");
}

#[test]
fn filter_mixed_all_skipped() {
    let t = ty("Option<Vec<Box<i32>>>");
    let f = filter_inner_type(&t, &skip(&["Option", "Vec", "Box"]));
    assert_eq!(to_s(&f), "i32");
}

#[test]
fn wrap_mixed_skip_option_vec() {
    let t = ty("Option<Vec<Box<i32>>>");
    let w = wrap_leaf_type(&t, &skip(&["Option", "Vec"]));
    assert_eq!(
        to_s(&w),
        "Option < Vec < adze :: WithLeaf < Box < i32 > > > >"
    );
}

// =========================================================================
// 3 – Types with lifetimes: &'a T, &'static str
// =========================================================================

#[test]
fn extract_ref_lifetime_not_path() {
    let t = ty("&'a T");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
    assert_eq!(to_s(&inner), "& 'a T");
}

#[test]
fn extract_ref_static_str() {
    let t = ty("&'static str");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &empty());
    assert!(!ok);
    assert_eq!(to_s(&inner), "& 'static str");
}

#[test]
fn filter_ref_lifetime_returns_unchanged() {
    let t = ty("&'a T");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "& 'a T");
}

#[test]
fn wrap_ref_lifetime() {
    let t = ty("&'a T");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < & 'a T >");
}

#[test]
fn wrap_ref_static_str() {
    let t = ty("&'static str");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < & 'static str >");
}

// =========================================================================
// 4 – Types with const generics: [T; N]
// =========================================================================

#[test]
fn extract_array_const_generic() {
    let t = ty("[u8; 16]");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &empty());
    assert!(!ok);
    assert_eq!(to_s(&inner), "[u8 ; 16]");
}

#[test]
fn filter_array_unchanged() {
    let t = ty("[u8; 16]");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "[u8 ; 16]");
}

#[test]
fn wrap_array_const_generic() {
    let t = ty("[u8; 16]");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < [u8 ; 16] >");
}

#[test]
fn extract_generic_array_type_param() {
    // Array<T, N> as a path type with const generic
    let t = ty("ArrayVec<u8, 4>");
    let (inner, ok) = try_extract_inner_type(&t, "ArrayVec", &empty());
    assert!(ok);
    assert_eq!(to_s(&inner), "u8");
}

// =========================================================================
// 5 – Types with where clauses (function signatures)
// =========================================================================

#[test]
fn extract_fn_trait_type() {
    // Fn(A) -> B as a trait bound path type via parse_quote
    let t: Type = syn::parse_quote!(dyn Fn(i32) -> bool);
    let (_, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
}

// =========================================================================
// 6 – Unit type ()
// =========================================================================

#[test]
fn extract_unit_type() {
    let t = ty("()");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
    assert_eq!(to_s(&inner), "()");
}

#[test]
fn filter_unit_type() {
    let t = ty("()");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "()");
}

#[test]
fn wrap_unit_type() {
    let t = ty("()");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < () >");
}

// =========================================================================
// 7 – Never type !
// =========================================================================

#[test]
fn extract_never_type() {
    let t = ty("!");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
    assert_eq!(to_s(&inner), "!");
}

#[test]
fn filter_never_type() {
    let t = ty("!");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "!");
}

#[test]
fn wrap_never_type() {
    let t = ty("!");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < ! >");
}

// =========================================================================
// 8 – Tuple types (A, B, C)
// =========================================================================

#[test]
fn extract_tuple_two() {
    let t = ty("(i32, u64)");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
    assert_eq!(to_s(&inner), "(i32 , u64)");
}

#[test]
fn extract_tuple_three() {
    let t = ty("(i32, u64, String)");
    let (_, ok) = try_extract_inner_type(&t, "Vec", &empty());
    assert!(!ok);
}

#[test]
fn filter_tuple_unchanged() {
    let t = ty("(i32, u64, String)");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "(i32 , u64 , String)");
}

#[test]
fn wrap_tuple_type() {
    let t = ty("(i32, u64)");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < (i32 , u64) >");
}

#[test]
fn extract_option_of_tuple() {
    let t = ty("Option<(i32, bool)>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(ok);
    assert_eq!(to_s(&inner), "(i32 , bool)");
}

// =========================================================================
// 9 – Reference types: &T, &mut T
// =========================================================================

#[test]
fn extract_shared_ref() {
    let t = ty("&i32");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
    assert_eq!(to_s(&inner), "& i32");
}

#[test]
fn extract_mut_ref() {
    let t = ty("&mut i32");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &empty());
    assert!(!ok);
    assert_eq!(to_s(&inner), "& mut i32");
}

#[test]
fn filter_shared_ref() {
    let t = ty("&i32");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "& i32");
}

#[test]
fn filter_mut_ref() {
    let t = ty("&mut String");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "& mut String");
}

#[test]
fn wrap_shared_ref() {
    let t = ty("&i32");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < & i32 >");
}

#[test]
fn wrap_mut_ref() {
    let t = ty("&mut i32");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < & mut i32 >");
}

// =========================================================================
// 10 – Slice types: [T]
// =========================================================================

#[test]
fn extract_slice_type() {
    let t = ty("[u8]");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &empty());
    assert!(!ok);
    assert_eq!(to_s(&inner), "[u8]");
}

#[test]
fn filter_slice_type() {
    let t = ty("[u8]");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "[u8]");
}

#[test]
fn wrap_slice_type() {
    let t = ty("[u8]");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < [u8] >");
}

// =========================================================================
// 11 – Function pointer types: fn(A) -> B
// =========================================================================

#[test]
fn extract_fn_pointer() {
    let t = ty("fn(i32) -> bool");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
    assert_eq!(to_s(&inner), "fn (i32) -> bool");
}

#[test]
fn filter_fn_pointer() {
    let t = ty("fn(i32) -> bool");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "fn (i32) -> bool");
}

#[test]
fn wrap_fn_pointer() {
    let t = ty("fn(i32) -> bool");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < fn (i32) -> bool >");
}

#[test]
fn extract_fn_pointer_no_return() {
    let t = ty("fn(i32, u64)");
    let (_, ok) = try_extract_inner_type(&t, "Vec", &empty());
    assert!(!ok);
}

#[test]
fn wrap_fn_pointer_no_return() {
    let t = ty("fn()");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < fn () >");
}

// =========================================================================
// 12 – Qualified paths: <T as Trait>::Item
// =========================================================================

#[test]
fn extract_qualified_path() {
    let t = ty("<Vec<u8> as IntoIterator>::Item");
    let (_inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
    // QSelf types are not Type::Path in the normal sense
}

#[test]
fn filter_qualified_path() {
    let t = ty("<Vec<u8> as IntoIterator>::Item");
    // The type has a qself, so last segment ident is "Item" which is not in skip
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "< Vec < u8 > as IntoIterator > :: Item");
}

#[test]
fn wrap_qualified_path() {
    let t = ty("<Vec<u8> as IntoIterator>::Item");
    // Type::Path with qself – last segment is "Item" which is not in skip set
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(
        to_s(&w),
        "adze :: WithLeaf < < Vec < u8 > as IntoIterator > :: Item >"
    );
}

// =========================================================================
// 13 – impl Trait types
// =========================================================================

#[test]
fn extract_impl_trait() {
    let t = ty("impl Iterator<Item = i32>");
    let (_inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
    // impl Trait is Type::ImplTrait, not Type::Path
}

#[test]
fn filter_impl_trait() {
    let t = ty("impl Clone");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "impl Clone");
}

#[test]
fn wrap_impl_trait() {
    let t = ty("impl Clone");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < impl Clone >");
}

// =========================================================================
// 14 – Raw pointer types: *const T, *mut T
// =========================================================================

#[test]
fn extract_raw_const_ptr() {
    let t = ty("*const u8");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
    assert_eq!(to_s(&inner), "* const u8");
}

#[test]
fn extract_raw_mut_ptr() {
    let t = ty("*mut u8");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &empty());
    assert!(!ok);
    assert_eq!(to_s(&inner), "* mut u8");
}

#[test]
fn filter_raw_const_ptr() {
    let t = ty("*const u8");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "* const u8");
}

#[test]
fn filter_raw_mut_ptr() {
    let t = ty("*mut u8");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "* mut u8");
}

#[test]
fn wrap_raw_const_ptr() {
    let t = ty("*const u8");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < * const u8 >");
}

#[test]
fn wrap_raw_mut_ptr() {
    let t = ty("*mut u8");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < * mut u8 >");
}

// =========================================================================
// 15 – Array types: [T; 3]
// =========================================================================

#[test]
fn extract_array_literal_size() {
    let t = ty("[i32; 3]");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
    assert_eq!(to_s(&inner), "[i32 ; 3]");
}

#[test]
fn filter_array_literal_size() {
    let t = ty("[i32; 3]");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "[i32 ; 3]");
}

#[test]
fn wrap_array_literal_size() {
    let t = ty("[i32; 3]");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < [i32 ; 3] >");
}

#[test]
fn extract_array_zero_size() {
    let t = ty("[u8; 0]");
    let (_, ok) = try_extract_inner_type(&t, "Vec", &empty());
    assert!(!ok);
}

// =========================================================================
// Additional deep edge cases
// =========================================================================

// --- Deeply nested skip chains ---

#[test]
fn extract_deep_skip_chain_4_levels() {
    let t = ty("Box<Arc<Rc<Cell<i32>>>>");
    let (inner, ok) = try_extract_inner_type(&t, "Cell", &skip(&["Box", "Arc", "Rc"]));
    assert!(ok);
    assert_eq!(to_s(&inner), "i32");
}

#[test]
fn filter_deep_skip_chain_4_levels() {
    let t = ty("Box<Arc<Rc<Cell<i32>>>>");
    let f = filter_inner_type(&t, &skip(&["Box", "Arc", "Rc", "Cell"]));
    assert_eq!(to_s(&f), "i32");
}

// --- Path with module qualifier ---

#[test]
fn extract_fully_qualified_vec() {
    // std::vec::Vec<i32> – last segment is Vec
    let t = ty("std::vec::Vec<i32>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &empty());
    assert!(ok);
    assert_eq!(to_s(&inner), "i32");
}

#[test]
fn filter_fully_qualified_box() {
    let t = ty("std::boxed::Box<String>");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "String");
}

#[test]
fn wrap_fully_qualified_option() {
    let t = ty("std::option::Option<i32>");
    let w = wrap_leaf_type(&t, &skip(&["Option"]));
    assert_eq!(
        to_s(&w),
        "std :: option :: Option < adze :: WithLeaf < i32 > >"
    );
}

// --- Primitives ---

#[test]
fn extract_primitive_i32() {
    let t = ty("i32");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
    assert_eq!(to_s(&inner), "i32");
}

#[test]
fn filter_primitive_bool() {
    let t = ty("bool");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "bool");
}

#[test]
fn wrap_primitive_u8() {
    let t = ty("u8");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < u8 >");
}

// --- String type ---

#[test]
fn wrap_string_type() {
    let t = ty("String");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < String >");
}

// --- Option wrapping with nested non-path ---

#[test]
fn wrap_option_of_ref() {
    let t = ty("Option<&str>");
    let w = wrap_leaf_type(&t, &skip(&["Option"]));
    assert_eq!(to_s(&w), "Option < adze :: WithLeaf < & str > >");
}

#[test]
fn wrap_option_of_tuple() {
    let t = ty("Option<(i32, bool)>");
    let w = wrap_leaf_type(&t, &skip(&["Option"]));
    assert_eq!(to_s(&w), "Option < adze :: WithLeaf < (i32 , bool) > >");
}

#[test]
fn wrap_option_of_array() {
    let t = ty("Option<[u8; 4]>");
    let w = wrap_leaf_type(&t, &skip(&["Option"]));
    assert_eq!(to_s(&w), "Option < adze :: WithLeaf < [u8 ; 4] > >");
}

#[test]
fn wrap_option_of_fn_ptr() {
    let t = ty("Option<fn(i32) -> bool>");
    let w = wrap_leaf_type(&t, &skip(&["Option"]));
    assert_eq!(to_s(&w), "Option < adze :: WithLeaf < fn (i32) -> bool > >");
}

// --- Vec of non-path inner ---

#[test]
fn wrap_vec_of_slice_ref() {
    let t = ty("Vec<&[u8]>");
    let w = wrap_leaf_type(&t, &skip(&["Vec"]));
    assert_eq!(to_s(&w), "Vec < adze :: WithLeaf < & [u8] > >");
}

// --- Result type (multi-generic) ---

#[test]
fn extract_result_ok_type() {
    let t = ty("Result<String, Error>");
    let (inner, ok) = try_extract_inner_type(&t, "Result", &empty());
    assert!(ok);
    // First generic argument is the Ok type
    assert_eq!(to_s(&inner), "String");
}

#[test]
fn filter_result_in_skip() {
    let t = ty("Result<String, Error>");
    let f = filter_inner_type(&t, &skip(&["Result"]));
    // Unwraps first generic argument
    assert_eq!(to_s(&f), "String");
}

#[test]
fn wrap_result_both_args() {
    let t = ty("Result<String, i32>");
    let w = wrap_leaf_type(&t, &skip(&["Result"]));
    assert_eq!(
        to_s(&w),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

// --- HashMap (multi-generic) ---

#[test]
fn extract_hashmap_first_arg() {
    let t = ty("HashMap<String, i32>");
    let (inner, ok) = try_extract_inner_type(&t, "HashMap", &empty());
    assert!(ok);
    assert_eq!(to_s(&inner), "String");
}

// --- Type not in skip set falls through ---

#[test]
fn extract_skip_misses_intermediate() {
    // Rc wraps Vec<i32>, but Rc is not in skip set, so we can't reach Vec
    let t = ty("Rc<Vec<i32>>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(to_s(&inner), "Rc < Vec < i32 > >");
}

// --- Extracting from a type that IS the target but has no generics = panic ---
// (Skipped because it panics – the API expects correct usage)

// --- Type with turbofish-style generics ---

#[test]
fn extract_custom_generic() {
    let t = ty("MyWrapper<Inner>");
    let (inner, ok) = try_extract_inner_type(&t, "MyWrapper", &empty());
    assert!(ok);
    assert_eq!(to_s(&inner), "Inner");
}

#[test]
fn filter_custom_generic_in_skip() {
    let t = ty("MyWrapper<Inner>");
    let f = filter_inner_type(&t, &skip(&["MyWrapper"]));
    assert_eq!(to_s(&f), "Inner");
}

// --- dyn Trait ---

#[test]
fn extract_dyn_trait() {
    let t = ty("dyn Iterator<Item = i32>");
    let (_inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
}

#[test]
fn filter_dyn_trait() {
    let t = ty("dyn Clone");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "dyn Clone");
}

#[test]
fn wrap_dyn_trait() {
    let t = ty("dyn Clone");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < dyn Clone >");
}

// --- Infer type _ ---

#[test]
fn extract_infer_type() {
    let t = ty("_");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
    assert_eq!(to_s(&inner), "_");
}

#[test]
fn wrap_infer_type() {
    let t = ty("_");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < _ >");
}

// --- Nested wrap: Vec<Option<T>> both in skip ---

#[test]
fn wrap_vec_option_both_skip() {
    let t = ty("Vec<Option<i32>>");
    let w = wrap_leaf_type(&t, &skip(&["Vec", "Option"]));
    assert_eq!(to_s(&w), "Vec < Option < adze :: WithLeaf < i32 > > >");
}

// --- Option<Vec<T>> extract Vec through Option ---

#[test]
fn extract_vec_through_option() {
    let t = ty("Option<Vec<i32>>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&["Option"]));
    assert!(ok);
    assert_eq!(to_s(&inner), "i32");
}

// --- Deeply nested wrap: Option<Vec<Option<i32>>> ---

#[test]
fn wrap_option_vec_option_all_skip() {
    let t = ty("Option<Vec<Option<i32>>>");
    let w = wrap_leaf_type(&t, &skip(&["Option", "Vec"]));
    assert_eq!(
        to_s(&w),
        "Option < Vec < Option < adze :: WithLeaf < i32 > > > >"
    );
}

// --- Single segment no-generics path ---

#[test]
fn extract_simple_ident_not_target() {
    let t = ty("Foo");
    let (inner, ok) = try_extract_inner_type(&t, "Bar", &empty());
    assert!(!ok);
    assert_eq!(to_s(&inner), "Foo");
}

#[test]
fn filter_simple_ident_not_in_skip() {
    let t = ty("Foo");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "Foo");
}

// --- Nested Box<Box<T>> ---

#[test]
fn filter_double_box() {
    let t = ty("Box<Box<i32>>");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "i32");
}

#[test]
fn extract_inner_box_through_box() {
    let t = ty("Box<Box<i32>>");
    let (inner, ok) = try_extract_inner_type(&t, "Box", &empty());
    // Outer Box matches directly
    assert!(ok);
    assert_eq!(to_s(&inner), "Box < i32 >");
}

#[test]
fn extract_inner_box_skip_box() {
    let t = ty("Box<Box<i32>>");
    // target=Box, skip=Box: outer Box matches immediately (before skip logic)
    let (inner, ok) = try_extract_inner_type(&t, "Box", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(to_s(&inner), "Box < i32 >");
}

// --- trait object with lifetime ---

#[test]
fn wrap_dyn_trait_with_lifetime() {
    let t = ty("dyn Send + 'static");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < dyn Send + 'static >");
}

// --- Complex qualified path ---

#[test]
fn extract_simple_associated_type() {
    let t = ty("<T as Iterator>::Item");
    let (_, ok) = try_extract_inner_type(&t, "Option", &empty());
    // Has qself so it's a path type but with qualified self
    // The last segment is "Item" which != "Option"
    assert!(!ok);
}

// --- Multiple wrapper layers with wrap ---

#[test]
fn wrap_box_not_in_skip() {
    let t = ty("Box<i32>");
    let w = wrap_leaf_type(&t, &empty());
    // Box is NOT in skip set, so the entire thing gets wrapped
    assert_eq!(to_s(&w), "adze :: WithLeaf < Box < i32 > >");
}

#[test]
fn wrap_box_in_skip() {
    let t = ty("Box<i32>");
    let w = wrap_leaf_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&w), "Box < adze :: WithLeaf < i32 > >");
}

// --- Reference to Option ---

#[test]
fn extract_ref_to_option() {
    // &Option<i32> – outer is a reference, not a path
    let t = ty("&Option<i32>");
    let (_, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
}

// --- Option containing reference ---

#[test]
fn extract_option_containing_ref() {
    let t = ty("Option<&i32>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(ok);
    assert_eq!(to_s(&inner), "& i32");
}

// --- Option containing raw pointer ---

#[test]
fn extract_option_containing_raw_ptr() {
    let t = ty("Option<*const u8>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(ok);
    assert_eq!(to_s(&inner), "* const u8");
}

// --- Option containing fn pointer ---

#[test]
fn extract_option_containing_fn_ptr() {
    let t = ty("Option<fn(i32) -> bool>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(ok);
    assert_eq!(to_s(&inner), "fn (i32) -> bool");
}

// --- filter_inner_type idempotent on leaf ---

#[test]
fn filter_already_leaf_is_idempotent() {
    let t = ty("i32");
    let f1 = filter_inner_type(&t, &skip(&["Box"]));
    let f2 = filter_inner_type(&f1, &skip(&["Box"]));
    assert_eq!(to_s(&f1), to_s(&f2));
}

// --- wrap_leaf_type idempotent check (it wraps again) ---

#[test]
fn wrap_double_wraps_non_skip_type() {
    let t = ty("i32");
    let w1 = wrap_leaf_type(&t, &empty());
    let w2 = wrap_leaf_type(&w1, &empty());
    // adze::WithLeaf is not in skip set, so it gets double wrapped
    assert_eq!(to_s(&w2), "adze :: WithLeaf < adze :: WithLeaf < i32 > >");
}

// --- Parenthesized type ---

#[test]
fn extract_paren_type() {
    let t = ty("(i32)");
    // syn parses (i32) as Type::Paren
    let (_inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
}

#[test]
fn wrap_paren_type() {
    let t = ty("(i32)");
    let w = wrap_leaf_type(&t, &empty());
    // Type::Paren is not Type::Path so it gets wrapped
    assert!(to_s(&w).contains("adze :: WithLeaf"));
}

// --- Empty skip set behavior ---

#[test]
fn extract_with_empty_skip_no_match() {
    let t = ty("Box<Vec<i32>>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &empty());
    // Box is not in skip, so can't reach Vec
    assert!(!ok);
    assert_eq!(to_s(&inner), "Box < Vec < i32 > >");
}

// --- Self type ---

#[test]
fn extract_self_type() {
    let t = ty("Self");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
    assert_eq!(to_s(&inner), "Self");
}

#[test]
fn wrap_self_type() {
    let t = ty("Self");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < Self >");
}

// --- Type with multiple path segments but no generics ---

#[test]
fn extract_module_path_type() {
    let t = ty("std::string::String");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
    assert_eq!(to_s(&inner), "std :: string :: String");
}

#[test]
fn wrap_module_path_type() {
    let t = ty("std::string::String");
    let w = wrap_leaf_type(&t, &empty());
    assert_eq!(to_s(&w), "adze :: WithLeaf < std :: string :: String >");
}

// --- Generic with lifetime param ---

#[test]
fn extract_cow_with_lifetime() {
    let t = ty("Cow<'a, str>");
    // First generic argument is a lifetime, not a type
    // The function expects a Type arg, so this would be the lifetime 'a
    // which is GenericArgument::Lifetime, not GenericArgument::Type
    // For Cow, the first arg is the lifetime – so extracting target "Cow"
    // would panic. Instead, just verify it doesn't match a different target.
    let (_, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
}

// --- Vec<()> ---

#[test]
fn extract_vec_of_unit() {
    let t = ty("Vec<()>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &empty());
    assert!(ok);
    assert_eq!(to_s(&inner), "()");
}

// --- Option<()> ---

#[test]
fn extract_option_of_unit() {
    let t = ty("Option<()>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(ok);
    assert_eq!(to_s(&inner), "()");
}

// --- Nested with different wrapper at each level ---

#[test]
fn filter_alternating_wrappers() {
    let t = ty("Box<Arc<Box<Arc<i32>>>>");
    let f = filter_inner_type(&t, &skip(&["Box", "Arc"]));
    assert_eq!(to_s(&f), "i32");
}

// --- Wrap with large skip set ---

#[test]
fn wrap_with_large_skip_set() {
    let t = ty("Vec<Option<Box<i32>>>");
    let w = wrap_leaf_type(&t, &skip(&["Vec", "Option", "Box"]));
    assert_eq!(
        to_s(&w),
        "Vec < Option < Box < adze :: WithLeaf < i32 > > > >"
    );
}

// --- Extract from type whose name is substring of target ---

#[test]
fn extract_vec2_is_not_vec() {
    let t = ty("Vec2<i32>");
    let (_, ok) = try_extract_inner_type(&t, "Vec", &empty());
    assert!(!ok);
}

#[test]
fn extract_optional_is_not_option() {
    let t = ty("Optional<i32>");
    let (_, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(!ok);
}

// --- filter with same type in skip is idempotent after first peel ---

#[test]
fn filter_single_layer_in_skip() {
    let t = ty("Box<String>");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(to_s(&f), "String");
}

// --- wrap preserves non-type generic args ---

#[test]
fn wrap_result_skip_preserves_both_type_args() {
    let t = ty("Result<Vec<i32>, String>");
    let w = wrap_leaf_type(&t, &skip(&["Result", "Vec"]));
    assert_eq!(
        to_s(&w),
        "Result < Vec < adze :: WithLeaf < i32 > > , adze :: WithLeaf < String > >"
    );
}

// --- Extract never type from Option ---

#[test]
fn extract_option_of_never() {
    let t = ty("Option<!>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &empty());
    assert!(ok);
    assert_eq!(to_s(&inner), "!");
}

// --- Multiple type params: only first is extracted ---

#[test]
fn extract_first_of_multi_generic() {
    let t = ty("Either<Left, Right>");
    let (inner, ok) = try_extract_inner_type(&t, "Either", &empty());
    assert!(ok);
    // Only the first generic argument is returned
    assert_eq!(to_s(&inner), "Left");
}
