//! Comprehensive tests for `filter_inner_type` with various `skip_over` sets,
//! plus composition with `try_extract_inner_type`.

use adze_common::{filter_inner_type, try_extract_inner_type};
use quote::quote;
use std::collections::HashSet;
use syn::{parse_quote, Type};

/// Render a `Type` to its token string for comparison.
fn type_str(ty: &Type) -> String {
    quote!(#ty).to_string()
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 1 – filter_inner_type with EMPTY skip_over (always unchanged)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn filter_empty_skip_vec_i32_unchanged() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = HashSet::new();
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_empty_skip_i32_unchanged() {
    let ty: Type = parse_quote!(i32);
    let skip: HashSet<&str> = HashSet::new();
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_empty_skip_option_string_unchanged() {
    let ty: Type = parse_quote!(Option<String>);
    let skip: HashSet<&str> = HashSet::new();
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_empty_skip_box_u8_unchanged() {
    let ty: Type = parse_quote!(Box<u8>);
    let skip: HashSet<&str> = HashSet::new();
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_empty_skip_nested_vec_unchanged() {
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let skip: HashSet<&str> = HashSet::new();
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_empty_skip_option_vec_string_unchanged() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let skip: HashSet<&str> = HashSet::new();
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_empty_skip_string_unchanged() {
    let ty: Type = parse_quote!(String);
    let skip: HashSet<&str> = HashSet::new();
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_empty_skip_bool_unchanged() {
    let ty: Type = parse_quote!(bool);
    let skip: HashSet<&str> = HashSet::new();
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_empty_skip_triple_nested_unchanged() {
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let skip: HashSet<&str> = HashSet::new();
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_empty_skip_u64_unchanged() {
    let ty: Type = parse_quote!(u64);
    let skip: HashSet<&str> = HashSet::new();
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 2 – filter_inner_type with single-entry skip_over (matching)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn filter_skip_vec_on_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_skip_option_on_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "String");
}

#[test]
fn filter_skip_box_on_box_u8() {
    let ty: Type = parse_quote!(Box<u8>);
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "u8");
}

#[test]
fn filter_skip_vec_on_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "String");
}

#[test]
fn filter_skip_option_on_option_bool() {
    let ty: Type = parse_quote!(Option<bool>);
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "bool");
}

#[test]
fn filter_skip_box_on_box_vec_i32() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "Vec < i32 >");
}

#[test]
fn filter_skip_vec_recursive_two_layers() {
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    // Both Vec layers are stripped recursively.
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_skip_option_recursive_two_layers() {
    let ty: Type = parse_quote!(Option<Option<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_skip_box_recursive_two_layers() {
    let ty: Type = parse_quote!(Box<Box<String>>);
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "String");
}

#[test]
fn filter_skip_vec_stops_at_option() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    // Only Vec is stripped; Option is not in skip set.
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "Option < i32 >");
}

#[test]
fn filter_skip_vec_stops_at_box() {
    let ty: Type = parse_quote!(Vec<Box<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "Box < i32 >");
}

#[test]
fn filter_skip_box_triple_recursive() {
    let ty: Type = parse_quote!(Box<Box<Box<i32>>>);
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_skip_vec_on_vec_u64() {
    let ty: Type = parse_quote!(Vec<u64>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "u64");
}

#[test]
fn filter_skip_option_on_option_u16() {
    let ty: Type = parse_quote!(Option<u16>);
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "u16");
}

#[test]
fn filter_skip_vec_on_vec_f64() {
    let ty: Type = parse_quote!(Vec<f64>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "f64");
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 3 – filter_inner_type with single-entry skip_over (NON-matching)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn filter_skip_vec_on_i32_unchanged() {
    let ty: Type = parse_quote!(i32);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_skip_vec_on_string_unchanged() {
    let ty: Type = parse_quote!(String);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_skip_vec_on_bool_unchanged() {
    let ty: Type = parse_quote!(bool);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_skip_option_on_vec_i32_unchanged() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_skip_box_on_option_i32_unchanged() {
    let ty: Type = parse_quote!(Option<i32>);
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_skip_rc_on_arc_i32_unchanged() {
    let ty: Type = parse_quote!(Arc<i32>);
    let skip: HashSet<&str> = HashSet::from(["Rc"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_skip_mutex_on_vec_i32_unchanged() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = HashSet::from(["Mutex"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_skip_foo_bar_on_vec_i32_unchanged() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = HashSet::from(["Foo", "Bar"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 4 – filter_inner_type with multi-entry skip_over
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn filter_skip_vec_option_on_vec_option_i32() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_skip_vec_option_on_option_vec_i32() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_skip_box_option_on_box_option_string() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let skip: HashSet<&str> = HashSet::from(["Box", "Option"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "String");
}

#[test]
fn filter_skip_vec_box_on_vec_box_i32() {
    let ty: Type = parse_quote!(Vec<Box<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Box"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_skip_vec_option_box_on_triple_nested() {
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option", "Box"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_skip_vec_option_box_reversed_nesting() {
    let ty: Type = parse_quote!(Box<Vec<Option<i32>>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option", "Box"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_skip_vec_option_on_vec_i32_partial() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_skip_vec_option_on_option_string_partial() {
    let ty: Type = parse_quote!(Option<String>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "String");
}

#[test]
fn filter_skip_vec_option_on_box_i32_no_match() {
    let ty: Type = parse_quote!(Box<i32>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_skip_vec_option_on_i32_no_match() {
    let ty: Type = parse_quote!(i32);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_skip_vec_option_box_deep_nesting() {
    let ty: Type = parse_quote!(Vec<Vec<Option<Box<i32>>>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option", "Box"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_skip_vec_option_alternating_deep() {
    let ty: Type = parse_quote!(Vec<Option<Vec<Option<i32>>>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 5 – Custom type names in skip_over
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn filter_skip_arc_on_arc_i32() {
    let ty: Type = parse_quote!(Arc<i32>);
    let skip: HashSet<&str> = HashSet::from(["Arc"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_skip_rc_on_rc_string() {
    let ty: Type = parse_quote!(Rc<String>);
    let skip: HashSet<&str> = HashSet::from(["Rc"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "String");
}

#[test]
fn filter_skip_mutex_arc_on_arc_mutex_i32() {
    let ty: Type = parse_quote!(Arc<Mutex<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Mutex", "Arc"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_skip_cell_on_cell_bool() {
    let ty: Type = parse_quote!(Cell<bool>);
    let skip: HashSet<&str> = HashSet::from(["Cell"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "bool");
}

#[test]
fn filter_skip_refcell_on_refcell_vec_i32() {
    let ty: Type = parse_quote!(RefCell<Vec<i32>>);
    let skip: HashSet<&str> = HashSet::from(["RefCell"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "Vec < i32 >");
}

#[test]
fn filter_skip_mycontainer_on_custom_wrapper() {
    let ty: Type = parse_quote!(MyContainer<u32>);
    let skip: HashSet<&str> = HashSet::from(["MyContainer"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "u32");
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 6 – Non-Path types (references, tuples, arrays, slices)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn filter_skip_vec_on_reference_i32_unchanged() {
    let ty: Type = parse_quote!(&i32);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_skip_vec_on_mut_reference_unchanged() {
    let ty: Type = parse_quote!(&mut String);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_skip_vec_on_tuple_unchanged() {
    let ty: Type = parse_quote!((i32, String));
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_skip_vec_on_array_unchanged() {
    let ty: Type = parse_quote!([i32; 5]);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_skip_vec_on_slice_ref_unchanged() {
    let ty: Type = parse_quote!(&[i32]);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_skip_vec_option_on_reference_unchanged() {
    let ty: Type = parse_quote!(&i32);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_skip_on_unit_tuple_unchanged() {
    let ty: Type = parse_quote!(());
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 7 – Idempotency: filtering twice yields the same result
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn filter_idempotent_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let once = filter_inner_type(&ty, &skip);
    let twice = filter_inner_type(&once, &skip);
    assert_eq!(type_str(&once), type_str(&twice));
}

#[test]
fn filter_idempotent_i32_non_matching() {
    let ty: Type = parse_quote!(i32);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let once = filter_inner_type(&ty, &skip);
    let twice = filter_inner_type(&once, &skip);
    assert_eq!(type_str(&once), type_str(&twice));
}

#[test]
fn filter_idempotent_option_vec_i32() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Option", "Vec"]);
    let once = filter_inner_type(&ty, &skip);
    let twice = filter_inner_type(&once, &skip);
    assert_eq!(type_str(&once), type_str(&twice));
}

#[test]
fn filter_idempotent_nested_non_matching() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    let once = filter_inner_type(&ty, &skip);
    let twice = filter_inner_type(&once, &skip);
    assert_eq!(type_str(&once), type_str(&twice));
}

#[test]
fn filter_idempotent_empty_skip() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let skip: HashSet<&str> = HashSet::new();
    let once = filter_inner_type(&ty, &skip);
    let twice = filter_inner_type(&once, &skip);
    assert_eq!(type_str(&once), type_str(&twice));
}

#[test]
fn filter_idempotent_triple_application() {
    let ty: Type = parse_quote!(Box<Box<Box<i32>>>);
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let once = filter_inner_type(&ty, &skip);
    let twice = filter_inner_type(&once, &skip);
    let thrice = filter_inner_type(&twice, &skip);
    assert_eq!(type_str(&once), type_str(&thrice));
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 8 – Large skip sets
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn filter_large_skip_5_entries_one_matches() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option", "Box", "Arc", "Rc"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_large_skip_5_entries_none_match() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option", "Box", "Arc", "Rc"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_large_skip_10_entries_matching_nested() {
    let ty: Type = parse_quote!(Arc<Box<i32>>);
    let skip: HashSet<&str> =
        HashSet::from(["Vec", "Option", "Box", "Arc", "Rc", "Mutex", "Cell", "RefCell", "Cow", "Pin"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_large_skip_10_entries_on_primitive() {
    let ty: Type = parse_quote!(u8);
    let skip: HashSet<&str> =
        HashSet::from(["Vec", "Option", "Box", "Arc", "Rc", "Mutex", "Cell", "RefCell", "Cow", "Pin"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "u8");
}

#[test]
fn filter_large_skip_deep_nesting() {
    let ty: Type = parse_quote!(Arc<Rc<Box<Option<Vec<i32>>>>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option", "Box", "Arc", "Rc"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_large_skip_irrelevant_entries_dont_affect_result() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip_small: HashSet<&str> = HashSet::from(["Vec"]);
    let skip_large: HashSet<&str> =
        HashSet::from(["Vec", "Foo", "Bar", "Baz", "Quux", "Waldo", "Fred", "Plugh", "Xyzzy", "Thud"]);
    assert_eq!(
        type_str(&filter_inner_type(&ty, &skip_small)),
        type_str(&filter_inner_type(&ty, &skip_large)),
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 9 – try_extract_inner_type basics
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn extract_vec_i32_inner_of_vec_empty_skip() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = HashSet::new();
    let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(type_str(&result), "i32");
}

#[test]
fn extract_option_string_inner_of_option_empty_skip() {
    let ty: Type = parse_quote!(Option<String>);
    let skip: HashSet<&str> = HashSet::new();
    let (result, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(type_str(&result), "String");
}

#[test]
fn extract_i32_inner_of_vec_not_found() {
    let ty: Type = parse_quote!(i32);
    let skip: HashSet<&str> = HashSet::new();
    let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
    assert_eq!(type_str(&result), "i32");
}

#[test]
fn extract_vec_i32_inner_of_option_not_found() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = HashSet::new();
    let (result, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
    assert_eq!(type_str(&result), type_str(&ty));
}

#[test]
fn extract_box_option_i32_inner_of_option_skip_box() {
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let (result, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(type_str(&result), "i32");
}

#[test]
fn extract_vec_box_i32_inner_of_vec_no_skip_needed() {
    let ty: Type = parse_quote!(Vec<Box<i32>>);
    let skip: HashSet<&str> = HashSet::new();
    let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(type_str(&result), "Box < i32 >");
}

#[test]
fn extract_option_vec_i32_inner_of_vec_skip_option() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(type_str(&result), "i32");
}

#[test]
fn extract_i32_inner_of_vec_skip_option_not_found() {
    let ty: Type = parse_quote!(i32);
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
    assert_eq!(type_str(&result), "i32");
}

#[test]
fn extract_reference_inner_of_vec_not_found() {
    let ty: Type = parse_quote!(&i32);
    let skip: HashSet<&str> = HashSet::new();
    let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
    assert_eq!(type_str(&result), type_str(&ty));
}

#[test]
fn extract_box_i32_inner_of_box_empty_skip() {
    let ty: Type = parse_quote!(Box<i32>);
    let skip: HashSet<&str> = HashSet::new();
    let (result, extracted) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(extracted);
    assert_eq!(type_str(&result), "i32");
}

#[test]
fn extract_skip_not_matching_inner_returns_original() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    // Skips Box, then sees Vec — but inner_of is "Option", so no match.
    let (result, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
    assert_eq!(type_str(&result), type_str(&ty));
}

#[test]
fn extract_arc_mutex_i32_inner_of_mutex_skip_arc() {
    let ty: Type = parse_quote!(Arc<Mutex<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Arc"]);
    let (result, extracted) = try_extract_inner_type(&ty, "Mutex", &skip);
    assert!(extracted);
    assert_eq!(type_str(&result), "i32");
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 10 – Composition: filter then extract
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn filter_then_extract_vec_option_i32() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let filtered = filter_inner_type(&ty, &skip);
    // After filtering Vec, we have Option<i32>.
    assert_eq!(type_str(&filtered), "Option < i32 >");
    let (result, extracted) = try_extract_inner_type(&filtered, "Option", &HashSet::new());
    assert!(extracted);
    assert_eq!(type_str(&result), "i32");
}

#[test]
fn filter_then_extract_box_vec_string() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(type_str(&filtered), "Vec < String >");
    let (result, extracted) = try_extract_inner_type(&filtered, "Vec", &HashSet::new());
    assert!(extracted);
    assert_eq!(type_str(&result), "String");
}

#[test]
fn filter_preserves_non_skip_wrappers_for_extract() {
    let ty: Type = parse_quote!(Arc<Option<Vec<i32>>>);
    let skip: HashSet<&str> = HashSet::from(["Arc"]);
    let filtered = filter_inner_type(&ty, &skip);
    // Arc stripped, Option<Vec<i32>> remains.
    assert_eq!(type_str(&filtered), "Option < Vec < i32 > >");
    let (result, extracted) = try_extract_inner_type(&filtered, "Vec", &HashSet::from(["Option"]));
    assert!(extracted);
    assert_eq!(type_str(&result), "i32");
}

#[test]
fn both_return_same_for_non_matching_input() {
    let ty: Type = parse_quote!(i32);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let filtered = filter_inner_type(&ty, &skip);
    let (extracted_ty, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
    assert_eq!(type_str(&filtered), type_str(&extracted_ty));
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 11 – Consistency across type patterns
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn filter_same_type_different_skip_sets() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let skip_vec: HashSet<&str> = HashSet::from(["Vec"]);
    let skip_option: HashSet<&str> = HashSet::from(["Option"]);
    let skip_both: HashSet<&str> = HashSet::from(["Vec", "Option"]);

    assert_eq!(type_str(&filter_inner_type(&ty, &skip_vec)), "Option < i32 >");
    assert_eq!(type_str(&filter_inner_type(&ty, &skip_option)), type_str(&ty));
    assert_eq!(type_str(&filter_inner_type(&ty, &skip_both)), "i32");
}

#[test]
fn filter_adding_to_skip_can_only_unwrap_more() {
    let ty: Type = parse_quote!(Box<Option<Vec<i32>>>);
    let skip_small: HashSet<&str> = HashSet::from(["Box"]);
    let skip_medium: HashSet<&str> = HashSet::from(["Box", "Option"]);
    let skip_large: HashSet<&str> = HashSet::from(["Box", "Option", "Vec"]);

    let r_small = type_str(&filter_inner_type(&ty, &skip_small));
    let r_medium = type_str(&filter_inner_type(&ty, &skip_medium));
    let r_large = type_str(&filter_inner_type(&ty, &skip_large));

    assert_eq!(r_small, "Option < Vec < i32 > >");
    assert_eq!(r_medium, "Vec < i32 >");
    assert_eq!(r_large, "i32");
}

#[test]
fn filter_subset_skip_gives_less_unwrapping() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let subset: HashSet<&str> = HashSet::from(["Vec"]);
    let superset: HashSet<&str> = HashSet::from(["Vec", "Option"]);

    let r_sub = type_str(&filter_inner_type(&ty, &subset));
    let r_sup = type_str(&filter_inner_type(&ty, &superset));

    // Subset leaves Option intact; superset strips it.
    assert_eq!(r_sub, "Option < i32 >");
    assert_eq!(r_sup, "i32");
}

#[test]
fn filter_skip_matching_wrapper_always_strips() {
    // For any X<Y> where X is in skip, filter returns filter(Y).
    let ty_a: Type = parse_quote!(Wrapper<u32>);
    let ty_b: Type = parse_quote!(Wrapper<String>);
    let ty_c: Type = parse_quote!(Wrapper<bool>);
    let skip: HashSet<&str> = HashSet::from(["Wrapper"]);
    assert_eq!(type_str(&filter_inner_type(&ty_a, &skip)), "u32");
    assert_eq!(type_str(&filter_inner_type(&ty_b, &skip)), "String");
    assert_eq!(type_str(&filter_inner_type(&ty_c, &skip)), "bool");
}

#[test]
fn filter_order_of_nesting_matters() {
    let ty_vo: Type = parse_quote!(Vec<Option<i32>>);
    let ty_ov: Type = parse_quote!(Option<Vec<i32>>);
    let skip_vec_only: HashSet<&str> = HashSet::from(["Vec"]);

    // Vec<Option<i32>> with skip=Vec → Option<i32>
    assert_eq!(type_str(&filter_inner_type(&ty_vo, &skip_vec_only)), "Option < i32 >");
    // Option<Vec<i32>> with skip=Vec → unchanged (outer is Option, not in skip)
    assert_eq!(
        type_str(&filter_inner_type(&ty_ov, &skip_vec_only)),
        type_str(&ty_ov),
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 12 – Various skip set sizes (1, 2, 5, 10)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn filter_skip_size_1() {
    let ty: Type = parse_quote!(Box<f32>);
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "f32");
}

#[test]
fn filter_skip_size_2() {
    let ty: Type = parse_quote!(Box<Option<f32>>);
    let skip: HashSet<&str> = HashSet::from(["Box", "Option"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "f32");
}

#[test]
fn filter_skip_size_5_matching() {
    let ty: Type = parse_quote!(Rc<i32>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option", "Box", "Arc", "Rc"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_skip_size_5_no_match() {
    let ty: Type = parse_quote!(Mutex<i32>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option", "Box", "Arc", "Rc"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn filter_skip_size_10_matching() {
    let ty: Type = parse_quote!(Pin<u16>);
    let skip: HashSet<&str> =
        HashSet::from(["Vec", "Option", "Box", "Arc", "Rc", "Mutex", "Cell", "RefCell", "Cow", "Pin"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "u16");
}

#[test]
fn filter_skip_size_10_no_match() {
    let ty: Type = parse_quote!(Custom<u16>);
    let skip: HashSet<&str> =
        HashSet::from(["Vec", "Option", "Box", "Arc", "Rc", "Mutex", "Cell", "RefCell", "Cow", "Pin"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 13 – Nested generic types
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn filter_nested_option_option_option_i32() {
    let ty: Type = parse_quote!(Option<Option<Option<i32>>>);
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_nested_vec_four_layers() {
    let ty: Type = parse_quote!(Vec<Vec<Vec<Vec<i32>>>>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_nested_mixed_stops_at_first_non_skip() {
    let ty: Type = parse_quote!(Vec<HashMap<String, i32>>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "HashMap < String , i32 >");
}

#[test]
fn filter_nested_box_option_vec_string_all_skipped() {
    let ty: Type = parse_quote!(Box<Option<Vec<String>>>);
    let skip: HashSet<&str> = HashSet::from(["Box", "Option", "Vec"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "String");
}

#[test]
fn filter_result_preserves_inner_generics() {
    let ty: Type = parse_quote!(Vec<Result<i32, String>>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "Result < i32 , String >");
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 14 – try_extract_inner_type with skip_over chains
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn extract_arc_box_vec_i32_inner_of_vec_skip_arc_box() {
    let ty: Type = parse_quote!(Arc<Box<Vec<i32>>>);
    let skip: HashSet<&str> = HashSet::from(["Arc", "Box"]);
    let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(type_str(&result), "i32");
}

#[test]
fn extract_option_option_vec_i32_inner_of_vec_skip_option() {
    let ty: Type = parse_quote!(Option<Option<Vec<i32>>>);
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(type_str(&result), "i32");
}

#[test]
fn extract_skip_chain_no_target_found() {
    let ty: Type = parse_quote!(Box<Arc<Rc<i32>>>);
    let skip: HashSet<&str> = HashSet::from(["Box", "Arc", "Rc"]);
    // Looking for Vec, but after skipping Box→Arc→Rc we find i32, not Vec.
    let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
    assert_eq!(type_str(&result), type_str(&ty));
}

#[test]
fn extract_direct_match_ignores_skip() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    // Vec matches inner_of directly — no skipping needed.
    let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(type_str(&result), "i32");
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 15 – Edge cases and miscellaneous
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn filter_single_segment_path_no_generics_unchanged() {
    let ty: Type = parse_quote!(MyType);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), "MyType");
}

#[test]
fn filter_skip_with_only_irrelevant_entries() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = HashSet::from(["Zog", "Zap", "Zip"]);
    assert_eq!(type_str(&filter_inner_type(&ty, &skip)), type_str(&ty));
}

#[test]
fn extract_and_filter_agree_on_single_layer() {
    // When skip = {X} and the type is X<T>, filter returns T.
    // When inner_of = X and skip is empty, extract returns (T, true).
    let ty: Type = parse_quote!(Vec<i32>);
    let skip_filter: HashSet<&str> = HashSet::from(["Vec"]);
    let skip_extract: HashSet<&str> = HashSet::new();

    let filtered = filter_inner_type(&ty, &skip_filter);
    let (extracted, did_extract) = try_extract_inner_type(&ty, "Vec", &skip_extract);
    assert!(did_extract);
    assert_eq!(type_str(&filtered), type_str(&extracted));
}

#[test]
fn filter_empty_skip_is_identity() {
    let types: Vec<Type> = vec![
        parse_quote!(i32),
        parse_quote!(Vec<i32>),
        parse_quote!(Option<String>),
        parse_quote!(Box<Vec<Option<bool>>>),
        parse_quote!(&u8),
    ];
    let skip: HashSet<&str> = HashSet::new();
    for ty in &types {
        assert_eq!(type_str(&filter_inner_type(ty, &skip)), type_str(ty));
    }
}

#[test]
fn filter_consistent_across_calls() {
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option", "Box"]);
    let r1 = type_str(&filter_inner_type(&ty, &skip));
    let r2 = type_str(&filter_inner_type(&ty, &skip));
    let r3 = type_str(&filter_inner_type(&ty, &skip));
    assert_eq!(r1, r2);
    assert_eq!(r2, r3);
}
