//! Edge-case tests for `try_extract_inner_type` in adze-common.
//!
//! 80+ tests covering:
//!   1–10  Plain / primitive types (no extraction)
//!  11–20  Basic single-layer extraction
//!  21–30  Wrong wrapper / mismatch scenarios
//!  31–40  Nested containers — one layer peeled
//!  41–50  skip_over interactions
//!  51–60  Non-path types (references, tuples, arrays, slices, etc.)
//!  61–70  Multi-arg generics & lifetimes
//!  71–80  Case sensitivity, qualified paths, exotic wrappers
//!  81–86  Sequential / multi-step extraction

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
// 1–10  Plain / primitive types — never extracted
// ===========================================================================

#[test]
fn test_01_plain_i32_no_extraction() {
    let ty: Type = parse_quote!(i32);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_02_plain_u64_no_extraction() {
    let ty: Type = parse_quote!(u64);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "u64");
}

#[test]
fn test_03_plain_bool_no_extraction() {
    let ty: Type = parse_quote!(bool);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn test_04_plain_f32_no_extraction() {
    let ty: Type = parse_quote!(f32);
    let (inner, extracted) = try_extract_inner_type(&ty, "Rc", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "f32");
}

#[test]
fn test_05_string_no_generics_no_extraction() {
    let ty: Type = parse_quote!(String);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_06_string_as_inner_of_string_no_angle_brackets() {
    // "String" matches the ident but has no angle brackets → panics.
    let ty: Type = parse_quote!(String);
    let result = std::panic::catch_unwind(|| {
        try_extract_inner_type(&ty, "String", &skip(&[]));
    });
    assert!(result.is_err());
}

#[test]
fn test_07_usize_no_extraction() {
    let ty: Type = parse_quote!(usize);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "usize");
}

#[test]
fn test_08_char_no_extraction() {
    let ty: Type = parse_quote!(char);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "char");
}

#[test]
fn test_09_unit_type_no_extraction() {
    let ty: Type = parse_quote!(());
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    // Unit parses as Type::Tuple — returned unchanged.
    assert_eq!(ty_str(&inner), "()");
}

#[test]
fn test_10_isize_no_extraction() {
    let ty: Type = parse_quote!(isize);
    let (inner, extracted) = try_extract_inner_type(&ty, "Arc", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "isize");
}

// ===========================================================================
// 11–20  Basic single-layer extraction
// ===========================================================================

#[test]
fn test_11_extract_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_12_extract_option_bool() {
    let ty: Type = parse_quote!(Option<bool>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn test_13_extract_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_14_extract_vec_of_vec_peels_one_layer() {
    let ty: Type = parse_quote!(Vec<Vec<u8>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Vec < u8 >");
}

#[test]
fn test_15_extract_option_of_option_peels_one_layer() {
    let ty: Type = parse_quote!(Option<Option<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Option < i32 >");
}

#[test]
fn test_16_extract_box_of_box_peels_one_layer() {
    let ty: Type = parse_quote!(Box<Box<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Box < i32 >");
}

#[test]
fn test_17_extract_vec_of_option() {
    let ty: Type = parse_quote!(Vec<Option<f64>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Option < f64 >");
}

#[test]
fn test_18_extract_option_of_vec() {
    let ty: Type = parse_quote!(Option<Vec<u32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Vec < u32 >");
}

#[test]
fn test_19_extract_vec_of_complex_inner_preserves_inner() {
    let ty: Type = parse_quote!(Vec<std::collections::HashMap<String, Vec<i32>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(
        ty_str(&inner),
        "std :: collections :: HashMap < String , Vec < i32 > >"
    );
}

#[test]
fn test_20_extract_option_of_tuple_inner() {
    let ty: Type = parse_quote!(Option<(i32, u32)>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "(i32 , u32)");
}

// ===========================================================================
// 21–30  Wrong wrapper / mismatch scenarios
// ===========================================================================

#[test]
fn test_21_extract_option_from_vec_no_match() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn test_22_extract_box_from_option_no_match() {
    let ty: Type = parse_quote!(Option<u8>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Option < u8 >");
}

#[test]
fn test_23_extract_vec_from_box_no_match() {
    let ty: Type = parse_quote!(Box<f64>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Box < f64 >");
}

#[test]
fn test_24_extract_rc_from_arc_no_match() {
    let ty: Type = parse_quote!(Arc<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Rc", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Arc < i32 >");
}

#[test]
fn test_25_extract_custom_from_vec_no_match() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "MyWrapper", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Vec < String >");
}

#[test]
fn test_26_extract_nonexistent_wrapper_from_primitive() {
    let ty: Type = parse_quote!(u128);
    let (inner, extracted) = try_extract_inner_type(&ty, "Nonexistent", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "u128");
}

#[test]
fn test_27_extract_empty_inner_of_returns_unchanged() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn test_28_mismatch_nested_doesnt_recurse_without_skip() {
    // Looking for "Vec" in Option<Vec<i32>> without skip → no match.
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Option < Vec < i32 > >");
}

#[test]
fn test_29_extract_from_custom_generic_no_match() {
    let ty: Type = parse_quote!(MyStruct<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "MyStruct < i32 >");
}

#[test]
fn test_30_extract_custom_wrapper_match() {
    let ty: Type = parse_quote!(MyStruct<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "MyStruct", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

// ===========================================================================
// 31–40  Nested containers — one layer peeled per call
// ===========================================================================

#[test]
fn test_31_triple_nested_vec_peels_one() {
    let ty: Type = parse_quote!(Vec<Vec<Vec<i32>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Vec < Vec < i32 > >");
}

#[test]
fn test_32_peel_second_layer_manually() {
    let ty: Type = parse_quote!(Vec<Vec<Vec<i32>>>);
    let (first, _) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    let (second, extracted) = try_extract_inner_type(&first, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&second), "Vec < i32 >");
}

#[test]
fn test_33_peel_all_three_layers() {
    let ty: Type = parse_quote!(Vec<Vec<Vec<i32>>>);
    let (l1, e1) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(e1);
    let (l2, e2) = try_extract_inner_type(&l1, "Vec", &skip(&[]));
    assert!(e2);
    let (l3, e3) = try_extract_inner_type(&l2, "Vec", &skip(&[]));
    assert!(e3);
    assert_eq!(ty_str(&l3), "i32");
}

#[test]
fn test_34_option_option_option_peel_all() {
    let ty: Type = parse_quote!(Option<Option<Option<bool>>>);
    let (l1, _) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    let (l2, _) = try_extract_inner_type(&l1, "Option", &skip(&[]));
    let (l3, extracted) = try_extract_inner_type(&l2, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&l3), "bool");
}

#[test]
fn test_35_box_box_box_peel_one() {
    let ty: Type = parse_quote!(Box<Box<Box<u64>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Box < Box < u64 > >");
}

#[test]
fn test_36_mixed_nesting_vec_option_extracts_vec() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Option < i32 >");
}

#[test]
fn test_37_mixed_nesting_option_vec_extracts_option() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Vec < String >");
}

#[test]
fn test_38_deeply_nested_preserves_structure() {
    let ty: Type = parse_quote!(Vec<Option<Box<HashMap<String, i32>>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(
        ty_str(&inner),
        "Option < Box < HashMap < String , i32 > > >"
    );
}

#[test]
fn test_39_vec_of_array_type_inner() {
    let ty: Type = parse_quote!(Vec<[u8; 4]>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "[u8 ; 4]");
}

#[test]
fn test_40_option_of_reference_inner() {
    let ty: Type = parse_quote!(Option<&str>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "& str");
}

// ===========================================================================
// 41–50  skip_over interactions
// ===========================================================================

#[test]
fn test_41_skip_box_extract_vec_inside() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_42_skip_box_no_vec_inside_returns_original() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Box < String >");
}

#[test]
fn test_43_skip_option_extract_vec_inside() {
    let ty: Type = parse_quote!(Option<Vec<u32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Option"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn test_44_skip_multiple_layers() {
    let ty: Type = parse_quote!(Box<Option<Vec<i32>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Option"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_45_skip_does_not_affect_direct_match() {
    // Vec is both in skip_over and is the target — direct match wins.
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Vec"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_46_skip_arc_extract_option() {
    let ty: Type = parse_quote!(Arc<Option<bool>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn test_47_skip_rc_extract_box() {
    let ty: Type = parse_quote!(Rc<Box<f32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&["Rc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "f32");
}

#[test]
fn test_48_skip_irrelevant_wrapper_no_effect() {
    // skip_over has "Mutex" but type is Vec<i32> → skip_over not triggered.
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Mutex"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_49_skip_partial_chain_no_match_at_end() {
    // Box<Option<String>> skip Box, but looking for Vec → no match.
    let ty: Type = parse_quote!(Box<Option<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Box < Option < String > >");
}

#[test]
fn test_50_empty_skip_set() {
    let ty: Type = parse_quote!(Vec<i32>);
    let empty: HashSet<&str> = HashSet::new();
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &empty);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

// ===========================================================================
// 51–60  Non-path types (references, tuples, arrays, slices, etc.)
// ===========================================================================

#[test]
fn test_51_reference_type_no_extraction() {
    let ty: Type = parse_quote!(&i32);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "& i32");
}

#[test]
fn test_52_mutable_reference_no_extraction() {
    let ty: Type = parse_quote!(&mut String);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "& mut String");
}

#[test]
fn test_53_tuple_type_no_extraction() {
    let ty: Type = parse_quote!((i32, u32));
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "(i32 , u32)");
}

#[test]
fn test_54_three_element_tuple_no_extraction() {
    let ty: Type = parse_quote!((i32, f64, bool));
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "(i32 , f64 , bool)");
}

#[test]
fn test_55_array_type_no_extraction() {
    let ty: Type = parse_quote!([i32; 4]);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "[i32 ; 4]");
}

#[test]
fn test_56_slice_type_no_extraction() {
    let ty: Type = parse_quote!([u8]);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "[u8]");
}

#[test]
fn test_57_never_type_no_extraction() {
    let ty: Type = parse_quote!(!);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "!");
}

#[test]
fn test_58_fn_pointer_no_extraction() {
    let ty: Type = parse_quote!(fn(i32) -> bool);
    let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&result), "fn (i32) -> bool");
}

#[test]
fn test_59_raw_pointer_no_extraction() {
    let ty: Type = parse_quote!(*const u8);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "* const u8");
}

#[test]
fn test_60_reference_with_lifetime_no_extraction() {
    let ty: Type = parse_quote!(&'static str);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "& 'static str");
}

// ===========================================================================
// 61–70  Multi-arg generics & lifetimes
// ===========================================================================

#[test]
fn test_61_result_extract_first_arg() {
    // Result<i32, String> — extract "Result" takes the first generic arg.
    let ty: Type = parse_quote!(Result<i32, String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Result", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_62_hashmap_extract_first_arg() {
    let ty: Type = parse_quote!(HashMap<String, Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "HashMap", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_63_cow_with_lifetime_extracts_first_type_arg() {
    // Cow<'a, str> — first generic arg is lifetime, not a type.
    // The function expects the first generic arg to be a type and panics otherwise.
    let ty: Type = parse_quote!(Cow<'a, str>);
    let result = std::panic::catch_unwind(|| {
        try_extract_inner_type(&ty, "Cow", &skip(&[]));
    });
    assert!(result.is_err());
}

#[test]
fn test_64_result_no_match_when_looking_for_option() {
    let ty: Type = parse_quote!(Result<i32, String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Result < i32 , String >");
}

#[test]
fn test_65_btreemap_extract_first_arg() {
    let ty: Type = parse_quote!(BTreeMap<u64, String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "BTreeMap", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u64");
}

#[test]
fn test_66_vec_of_result_extract_vec_preserves_result() {
    let ty: Type = parse_quote!(Vec<Result<i32, String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Result < i32 , String >");
}

#[test]
fn test_67_option_of_result_extract_option() {
    let ty: Type = parse_quote!(Option<Result<bool, ()>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Result < bool , () >");
}

#[test]
fn test_68_phantom_data_extract() {
    let ty: Type = parse_quote!(PhantomData<T>);
    let (inner, extracted) = try_extract_inner_type(&ty, "PhantomData", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "T");
}

#[test]
fn test_69_cell_extract() {
    let ty: Type = parse_quote!(Cell<u32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Cell", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn test_70_refcell_extract() {
    let ty: Type = parse_quote!(RefCell<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "RefCell", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

// ===========================================================================
// 71–80  Case sensitivity, qualified paths, exotic wrappers
// ===========================================================================

#[test]
fn test_71_case_sensitive_no_match_lowercase() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn test_72_case_sensitive_no_match_uppercase() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "VEC", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn test_73_case_sensitive_option_vs_option() {
    let ty: Type = parse_quote!(Option<bool>);
    let (inner, extracted) = try_extract_inner_type(&ty, "option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Option < bool >");
}

#[test]
fn test_74_qualified_path_matches_last_segment() {
    // std::vec::Vec<i32> — last segment is "Vec", should match "Vec".
    let ty: Type = parse_quote!(std::vec::Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_75_qualified_path_std_option() {
    let ty: Type = parse_quote!(std::option::Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_76_qualified_path_no_match_full_path_string() {
    // inner_of = "std::vec::Vec" does not match because it checks last segment ident.
    let ty: Type = parse_quote!(std::vec::Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "std::vec::Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "std :: vec :: Vec < i32 >");
}

#[test]
fn test_77_custom_wrapper_exact_match() {
    let ty: Type = parse_quote!(Wrapper<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Wrapper", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_78_long_qualified_custom_type() {
    let ty: Type = parse_quote!(my::module::deep::Container<f64>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Container", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn test_79_skip_qualified_path() {
    // Box in skip_over + qualified path: std::boxed::Box<Vec<i32>>
    let ty: Type = parse_quote!(std::boxed::Box<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_80_extract_from_pin() {
    let ty: Type = parse_quote!(Pin<Box<dyn Future>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Pin", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Box < dyn Future >");
}

// ===========================================================================
// 81–90  Sequential / multi-step extraction, composition
// ===========================================================================

#[test]
fn test_81_sequential_different_targets() {
    // Option<Vec<i32>> — extract Option, then extract Vec.
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let (after_option, e1) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(e1);
    assert_eq!(ty_str(&after_option), "Vec < i32 >");
    let (after_vec, e2) = try_extract_inner_type(&after_option, "Vec", &skip(&[]));
    assert!(e2);
    assert_eq!(ty_str(&after_vec), "i32");
}

#[test]
fn test_82_extract_then_no_more_layers() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, _) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    let (same, extracted) = try_extract_inner_type(&inner, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&same), "i32");
}

#[test]
fn test_83_idempotent_on_non_matching() {
    let ty: Type = parse_quote!(String);
    let (first, e1) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!e1);
    let (second, e2) = try_extract_inner_type(&first, "Vec", &skip(&[]));
    assert!(!e2);
    assert_eq!(ty_str(&first), ty_str(&second));
}

#[test]
fn test_84_extract_interleaves_with_filter() {
    // Box<Vec<Option<i32>>>
    // filter_inner_type with skip={"Box"} → Vec<Option<i32>>
    // then extract "Vec" → Option<i32>
    let ty: Type = parse_quote!(Box<Vec<Option<i32>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < Option < i32 > >");
    let (inner, extracted) = try_extract_inner_type(&filtered, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Option < i32 >");
}

#[test]
fn test_85_extract_then_wrap_leaf() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    // wrap_leaf_type on a non-skip type wraps with adze::WithLeaf
    assert!(ty_str(&wrapped).contains("i32"));
}

#[test]
fn test_86_skip_through_two_boxes_to_vec() {
    let ty: Type = parse_quote!(Box<Box<Vec<u16>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u16");
}

#[test]
fn test_87_skip_three_layers_to_option() {
    let ty: Type = parse_quote!(Arc<Rc<Box<Option<String>>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Arc", "Rc", "Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_88_skip_stops_at_non_skip_non_target() {
    // Mutex<Vec<i32>> skip={"Arc"}, target="Vec" — Mutex not in skip, no match.
    let ty: Type = parse_quote!(Mutex<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Arc"]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Mutex < Vec < i32 > >");
}

#[test]
fn test_89_extract_from_deeply_nested_with_all_skips() {
    let ty: Type = parse_quote!(Box<Option<Arc<Vec<bool>>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Option", "Arc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn test_90_no_match_after_skip_chain() {
    // Box<Option<String>> skip={"Box","Option"}, target="Vec" → no Vec found.
    let ty: Type = parse_quote!(Box<Option<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Option"]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Box < Option < String > >");
}
