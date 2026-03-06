//! Comprehensive roundtrip tests for type operations in adze-common.
//!
//! Tests the interaction of `wrap_leaf_type`, `try_extract_inner_type`, and
//! `filter_inner_type` across primitives, containers, nesting depths, and
//! composition chains.

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
// 1. wrap then extract "WithLeaf" → original (basic roundtrip)
// ===========================================================================

#[test]
fn roundtrip_wrap_extract_i32() {
    let base: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&base, &skip(&[]));
    let (extracted, ok) = try_extract_inner_type(&wrapped, "WithLeaf", &skip(&["adze"]));
    assert!(ok);
    assert_eq!(ty_str(&extracted), ty_str(&base));
}

#[test]
fn roundtrip_wrap_extract_string() {
    let base: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&base, &skip(&[]));
    let (extracted, ok) = try_extract_inner_type(&wrapped, "WithLeaf", &skip(&["adze"]));
    assert!(ok);
    assert_eq!(ty_str(&extracted), ty_str(&base));
}

#[test]
fn roundtrip_wrap_extract_u8() {
    let base: Type = parse_quote!(u8);
    let wrapped = wrap_leaf_type(&base, &skip(&[]));
    let (extracted, ok) = try_extract_inner_type(&wrapped, "WithLeaf", &skip(&["adze"]));
    assert!(ok);
    assert_eq!(ty_str(&extracted), ty_str(&base));
}

// ===========================================================================
// 2–3. All primitive types: roundtrip through wrap+extract WithLeaf
// ===========================================================================

macro_rules! primitive_roundtrip {
    ($name:ident, $ty:ty) => {
        #[test]
        fn $name() {
            let base: Type = parse_quote!($ty);
            let wrapped = wrap_leaf_type(&base, &skip(&[]));
            // wrap must change the type
            assert_ne!(ty_str(&wrapped), ty_str(&base));
            // extract reverses the wrap
            let (extracted, ok) = try_extract_inner_type(&wrapped, "WithLeaf", &skip(&["adze"]));
            assert!(ok);
            assert_eq!(ty_str(&extracted), ty_str(&base));
        }
    };
}

primitive_roundtrip!(roundtrip_prim_i8, i8);
primitive_roundtrip!(roundtrip_prim_i16, i16);
primitive_roundtrip!(roundtrip_prim_i32, i32);
primitive_roundtrip!(roundtrip_prim_i64, i64);
primitive_roundtrip!(roundtrip_prim_i128, i128);
primitive_roundtrip!(roundtrip_prim_isize, isize);
primitive_roundtrip!(roundtrip_prim_u8, u8);
primitive_roundtrip!(roundtrip_prim_u16, u16);
primitive_roundtrip!(roundtrip_prim_u32, u32);
primitive_roundtrip!(roundtrip_prim_u64, u64);
primitive_roundtrip!(roundtrip_prim_u128, u128);
primitive_roundtrip!(roundtrip_prim_usize, usize);
primitive_roundtrip!(roundtrip_prim_f32, f32);
primitive_roundtrip!(roundtrip_prim_f64, f64);
primitive_roundtrip!(roundtrip_prim_bool, bool);
primitive_roundtrip!(roundtrip_prim_char, char);

// ===========================================================================
// 4. All primitives: roundtrip through Vec (extract Vec → inner)
// ===========================================================================

macro_rules! vec_extract_roundtrip {
    ($name:ident, $ty:ty) => {
        #[test]
        fn $name() {
            let base: Type = parse_quote!($ty);
            let container: Type = parse_quote!(Vec<$ty>);
            let (inner, ok) = try_extract_inner_type(&container, "Vec", &skip(&[]));
            assert!(ok);
            assert_eq!(ty_str(&inner), ty_str(&base));
        }
    };
}

vec_extract_roundtrip!(vec_roundtrip_i32, i32);
vec_extract_roundtrip!(vec_roundtrip_u64, u64);
vec_extract_roundtrip!(vec_roundtrip_f64, f64);
vec_extract_roundtrip!(vec_roundtrip_bool, bool);
vec_extract_roundtrip!(vec_roundtrip_char, char);
vec_extract_roundtrip!(vec_roundtrip_string, String);
vec_extract_roundtrip!(vec_roundtrip_u8, u8);
vec_extract_roundtrip!(vec_roundtrip_usize, usize);

// ===========================================================================
// 5. All primitives: roundtrip through Option (extract Option → inner)
// ===========================================================================

macro_rules! option_extract_roundtrip {
    ($name:ident, $ty:ty) => {
        #[test]
        fn $name() {
            let base: Type = parse_quote!($ty);
            let container: Type = parse_quote!(Option<$ty>);
            let (inner, ok) = try_extract_inner_type(&container, "Option", &skip(&[]));
            assert!(ok);
            assert_eq!(ty_str(&inner), ty_str(&base));
        }
    };
}

option_extract_roundtrip!(option_roundtrip_i32, i32);
option_extract_roundtrip!(option_roundtrip_u64, u64);
option_extract_roundtrip!(option_roundtrip_f64, f64);
option_extract_roundtrip!(option_roundtrip_bool, bool);
option_extract_roundtrip!(option_roundtrip_char, char);
option_extract_roundtrip!(option_roundtrip_string, String);
option_extract_roundtrip!(option_roundtrip_u8, u8);
option_extract_roundtrip!(option_roundtrip_usize, usize);

// ===========================================================================
// 6. All primitives: roundtrip through Box (extract Box → inner)
// ===========================================================================

macro_rules! box_extract_roundtrip {
    ($name:ident, $ty:ty) => {
        #[test]
        fn $name() {
            let base: Type = parse_quote!($ty);
            let container: Type = parse_quote!(Box<$ty>);
            let (inner, ok) = try_extract_inner_type(&container, "Box", &skip(&[]));
            assert!(ok);
            assert_eq!(ty_str(&inner), ty_str(&base));
        }
    };
}

box_extract_roundtrip!(box_roundtrip_i32, i32);
box_extract_roundtrip!(box_roundtrip_u64, u64);
box_extract_roundtrip!(box_roundtrip_f64, f64);
box_extract_roundtrip!(box_roundtrip_bool, bool);
box_extract_roundtrip!(box_roundtrip_string, String);
box_extract_roundtrip!(box_roundtrip_u8, u8);

// ===========================================================================
// 7. Double wrap then double extract
// ===========================================================================

#[test]
fn double_wrap_extract_i32() {
    let base: Type = parse_quote!(i32);
    let once = wrap_leaf_type(&base, &skip(&[]));
    let twice = wrap_leaf_type(&once, &skip(&[]));
    // first extract peels one layer
    let (inner1, ok1) = try_extract_inner_type(&twice, "WithLeaf", &skip(&["adze"]));
    assert!(ok1);
    assert_eq!(ty_str(&inner1), ty_str(&once));
    // second extract peels the other
    let (inner2, ok2) = try_extract_inner_type(&inner1, "WithLeaf", &skip(&["adze"]));
    assert!(ok2);
    assert_eq!(ty_str(&inner2), ty_str(&base));
}

#[test]
fn double_wrap_extract_string() {
    let base: Type = parse_quote!(String);
    let once = wrap_leaf_type(&base, &skip(&[]));
    let twice = wrap_leaf_type(&once, &skip(&[]));
    let (inner1, ok1) = try_extract_inner_type(&twice, "WithLeaf", &skip(&["adze"]));
    assert!(ok1);
    let (inner2, ok2) = try_extract_inner_type(&inner1, "WithLeaf", &skip(&["adze"]));
    assert!(ok2);
    assert_eq!(ty_str(&inner2), ty_str(&base));
}

// ===========================================================================
// 8. wrap then filter (with "WithLeaf" in skip) — filter peels back
// ===========================================================================

#[test]
fn wrap_then_filter_with_leaf_skip_recovers_base() {
    let base: Type = parse_quote!(u32);
    let wrapped = wrap_leaf_type(&base, &skip(&[]));
    // filter with both path segments in skip peels back
    let filtered = filter_inner_type(&wrapped, &skip(&["adze", "WithLeaf"]));
    assert_eq!(ty_str(&filtered), ty_str(&base));
}

#[test]
fn wrap_then_filter_empty_skip_preserves_wrapped() {
    let base: Type = parse_quote!(u32);
    let wrapped = wrap_leaf_type(&base, &skip(&[]));
    // filter with empty skip: wrapped type unchanged
    let filtered = filter_inner_type(&wrapped, &skip(&[]));
    assert_eq!(ty_str(&filtered), ty_str(&wrapped));
}

// ===========================================================================
// 9. Nested containers → multi-step roundtrip
// ===========================================================================

#[test]
fn nested_vec_option_extract_two_layers() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let (inner1, ok1) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok1);
    assert_eq!(ty_str(&inner1), "Option < i32 >");
    let (inner2, ok2) = try_extract_inner_type(&inner1, "Option", &skip(&[]));
    assert!(ok2);
    assert_eq!(ty_str(&inner2), "i32");
}

#[test]
fn nested_option_box_vec_extract_all() {
    let ty: Type = parse_quote!(Option<Box<Vec<u8>>>);
    let (inner1, ok1) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok1);
    let (inner2, ok2) = try_extract_inner_type(&inner1, "Box", &skip(&[]));
    assert!(ok2);
    let (inner3, ok3) = try_extract_inner_type(&inner2, "Vec", &skip(&[]));
    assert!(ok3);
    assert_eq!(ty_str(&inner3), "u8");
}

#[test]
fn nested_box_arc_filter_then_extract_vec() {
    let ty: Type = parse_quote!(Box<Arc<Vec<f32>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "Vec < f32 >");
    let (inner, ok) = try_extract_inner_type(&filtered, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "f32");
}

// ===========================================================================
// 10. wrap Vec<i32> with skip Vec, extract Vec → adze::WithLeaf<i32>
// ===========================================================================

#[test]
fn wrap_vec_i32_skip_vec_extract_vec() {
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < i32 > >");
    let (inner, ok) = try_extract_inner_type(&wrapped, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_vec_string_skip_vec_extract_then_extract_leaf() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    let (inner, ok) = try_extract_inner_type(&wrapped, "Vec", &skip(&[]));
    assert!(ok);
    let (leaf, ok2) = try_extract_inner_type(&inner, "WithLeaf", &skip(&["adze"]));
    assert!(ok2);
    assert_eq!(ty_str(&leaf), "String");
}

// ===========================================================================
// 11. wrap Option<String> with skip Option, extract Option → inner
// ===========================================================================

#[test]
fn wrap_option_string_skip_option_extract_option() {
    let ty: Type = parse_quote!(Option<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < String > >");
    let (inner, ok) = try_extract_inner_type(&wrapped, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_option_bool_full_roundtrip() {
    let base: Type = parse_quote!(bool);
    let option_ty: Type = parse_quote!(Option<bool>);
    let wrapped = wrap_leaf_type(&option_ty, &skip(&["Option"]));
    let (inner, ok) = try_extract_inner_type(&wrapped, "Option", &skip(&[]));
    assert!(ok);
    let (leaf, ok2) = try_extract_inner_type(&inner, "WithLeaf", &skip(&["adze"]));
    assert!(ok2);
    assert_eq!(ty_str(&leaf), ty_str(&base));
}

// ===========================================================================
// 12. Roundtrip preserves type equality
// ===========================================================================

#[test]
fn roundtrip_equality_simple_types() {
    let types: Vec<Type> = vec![
        parse_quote!(i32),
        parse_quote!(String),
        parse_quote!(bool),
        parse_quote!(f64),
    ];
    for base in &types {
        let wrapped = wrap_leaf_type(base, &skip(&[]));
        let (extracted, ok) = try_extract_inner_type(&wrapped, "WithLeaf", &skip(&["adze"]));
        assert!(ok);
        assert_eq!(
            ty_str(&extracted),
            ty_str(base),
            "roundtrip failed for {}",
            ty_str(base)
        );
    }
}

#[test]
fn roundtrip_equality_container_extract() {
    let pairs: Vec<(Type, &str, Type)> = vec![
        (parse_quote!(Vec<i32>), "Vec", parse_quote!(i32)),
        (parse_quote!(Option<String>), "Option", parse_quote!(String)),
        (parse_quote!(Box<bool>), "Box", parse_quote!(bool)),
    ];
    for (container, name, expected) in &pairs {
        let (inner, ok) = try_extract_inner_type(container, name, &skip(&[]));
        assert!(ok);
        assert_eq!(ty_str(&inner), ty_str(expected));
    }
}

// ===========================================================================
// 13. Many roundtrips → stable
// ===========================================================================

#[test]
fn repeated_wrap_extract_is_stable() {
    let base: Type = parse_quote!(u64);
    let mut current = base.clone();
    // wrap and extract 5 times — should always come back to base
    for _ in 0..5 {
        let wrapped = wrap_leaf_type(&current, &skip(&[]));
        let (extracted, ok) = try_extract_inner_type(&wrapped, "WithLeaf", &skip(&["adze"]));
        assert!(ok);
        assert_eq!(ty_str(&extracted), ty_str(&base));
        current = extracted;
    }
}

#[test]
fn repeated_filter_on_plain_type_is_stable() {
    let base: Type = parse_quote!(String);
    let mut current = base.clone();
    for _ in 0..5 {
        current = filter_inner_type(&current, &skip(&["Box", "Arc"]));
    }
    assert_eq!(ty_str(&current), ty_str(&base));
}

// ===========================================================================
// 14. extract non-matching → no change (identity)
// ===========================================================================

#[test]
fn extract_nonmatch_vec_from_option_is_identity() {
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), ty_str(&ty));
}

#[test]
fn extract_nonmatch_option_from_string_is_identity() {
    let ty: Type = parse_quote!(String);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), ty_str(&ty));
}

#[test]
fn extract_nonmatch_box_from_vec_u8_is_identity() {
    let ty: Type = parse_quote!(Vec<u8>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), ty_str(&ty));
}

#[test]
fn extract_nonmatch_on_ref_type_is_identity() {
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn extract_nonmatch_on_tuple_is_identity() {
    let ty: Type = parse_quote!((i32, u32));
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "(i32 , u32)");
}

// ===========================================================================
// 15. filter with empty skip → identity
// ===========================================================================

#[test]
fn filter_empty_skip_preserves_plain_type() {
    let ty: Type = parse_quote!(bool);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&[]))), "bool");
}

#[test]
fn filter_empty_skip_preserves_vec() {
    let ty: Type = parse_quote!(Vec<i32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&[]))), "Vec < i32 >");
}

#[test]
fn filter_empty_skip_preserves_option() {
    let ty: Type = parse_quote!(Option<String>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&[]))),
        "Option < String >"
    );
}

#[test]
fn filter_empty_skip_preserves_box_arc_nested() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&[]))),
        "Box < Arc < String > >"
    );
}

#[test]
fn filter_empty_skip_preserves_ref() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&[]))), "& str");
}

// ===========================================================================
// 16. wrap changes type (wrap is not identity)
// ===========================================================================

#[test]
fn wrap_i32_produces_with_leaf() {
    let base: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&base, &skip(&[]));
    assert_ne!(ty_str(&wrapped), ty_str(&base));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_string_produces_with_leaf() {
    let base: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&base, &skip(&[]));
    assert_ne!(ty_str(&wrapped), ty_str(&base));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_vec_no_skip_wraps_entire_container() {
    let base: Type = parse_quote!(Vec<u8>);
    let wrapped = wrap_leaf_type(&base, &skip(&[]));
    assert_ne!(ty_str(&wrapped), ty_str(&base));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Vec < u8 > >");
}

#[test]
fn wrap_vec_with_skip_wraps_only_inner() {
    let base: Type = parse_quote!(Vec<u8>);
    let wrapped = wrap_leaf_type(&base, &skip(&["Vec"]));
    assert_ne!(ty_str(&wrapped), ty_str(&base));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < u8 > >");
}

// ===========================================================================
// 17. extract reverses wrap
// ===========================================================================

#[test]
fn extract_reverses_wrap_f32() {
    let base: Type = parse_quote!(f32);
    let wrapped = wrap_leaf_type(&base, &skip(&[]));
    let (extracted, ok) = try_extract_inner_type(&wrapped, "WithLeaf", &skip(&["adze"]));
    assert!(ok);
    assert_eq!(ty_str(&extracted), ty_str(&base));
}

#[test]
fn extract_reverses_wrap_bool() {
    let base: Type = parse_quote!(bool);
    let wrapped = wrap_leaf_type(&base, &skip(&[]));
    let (extracted, ok) = try_extract_inner_type(&wrapped, "WithLeaf", &skip(&["adze"]));
    assert!(ok);
    assert_eq!(ty_str(&extracted), ty_str(&base));
}

#[test]
fn extract_reverses_wrap_usize() {
    let base: Type = parse_quote!(usize);
    let wrapped = wrap_leaf_type(&base, &skip(&[]));
    let (extracted, ok) = try_extract_inner_type(&wrapped, "WithLeaf", &skip(&["adze"]));
    assert!(ok);
    assert_eq!(ty_str(&extracted), ty_str(&base));
}

// ===========================================================================
// 18. Composition of wrap/extract is close to identity
// ===========================================================================

#[test]
fn composition_wrap_extract_identity_for_leaf() {
    let types: Vec<Type> = vec![
        parse_quote!(i32),
        parse_quote!(u8),
        parse_quote!(String),
        parse_quote!(bool),
        parse_quote!(char),
        parse_quote!(f64),
    ];
    for base in &types {
        let result = {
            let w = wrap_leaf_type(base, &skip(&[]));
            let (e, ok) = try_extract_inner_type(&w, "WithLeaf", &skip(&["adze"]));
            assert!(ok, "extract should succeed for {}", ty_str(base));
            e
        };
        assert_eq!(ty_str(&result), ty_str(base));
    }
}

#[test]
fn composition_extract_wrap_not_identity_for_container() {
    // extract Vec, then wrap → adze::WithLeaf<inner>, not Vec<inner>
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    let rewrapped = wrap_leaf_type(&inner, &skip(&[]));
    // rewrapped is adze::WithLeaf<i32>, not Vec<i32>
    assert_ne!(ty_str(&rewrapped), ty_str(&ty));
    assert_eq!(ty_str(&rewrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn composition_filter_wrap_roundtrip() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "String");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    let (back, ok) = try_extract_inner_type(&wrapped, "WithLeaf", &skip(&["adze"]));
    assert!(ok);
    assert_eq!(ty_str(&back), "String");
}

// ===========================================================================
// 19. Various wrapper names in skip_over
// ===========================================================================

#[test]
fn skip_arc_extracts_through() {
    let ty: Type = parse_quote!(Arc<Option<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn skip_rc_extracts_through() {
    let ty: Type = parse_quote!(Rc<Vec<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Rc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn skip_cell_extracts_through() {
    let ty: Type = parse_quote!(Cell<Box<u64>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&["Cell"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u64");
}

#[test]
fn skip_mutex_extracts_through() {
    let ty: Type = parse_quote!(Mutex<Option<bool>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Mutex"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn skip_rwlock_extracts_through() {
    let ty: Type = parse_quote!(RwLock<Vec<f32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["RwLock"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "f32");
}

#[test]
fn filter_with_custom_wrapper_names() {
    let ty: Type = parse_quote!(Mutex<RwLock<Arc<i32>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Mutex", "RwLock", "Arc"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn wrap_with_various_skips() {
    let ty: Type = parse_quote!(Arc<Box<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Arc", "Box"]));
    assert_eq!(ty_str(&wrapped), "Arc < Box < adze :: WithLeaf < i32 > > >");
}

// ===========================================================================
// 20. Various base types
// ===========================================================================

#[test]
fn roundtrip_path_type() {
    let base: Type = parse_quote!(std::string::String);
    let wrapped = wrap_leaf_type(&base, &skip(&[]));
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < std :: string :: String >"
    );
}

#[test]
fn roundtrip_custom_type_name() {
    let base: Type = parse_quote!(MyStruct);
    let wrapped = wrap_leaf_type(&base, &skip(&[]));
    let (extracted, ok) = try_extract_inner_type(&wrapped, "WithLeaf", &skip(&["adze"]));
    assert!(ok);
    assert_eq!(ty_str(&extracted), "MyStruct");
}

#[test]
fn roundtrip_generic_custom_type() {
    let base: Type = parse_quote!(Foo<Bar>);
    let wrapped = wrap_leaf_type(&base, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Foo < Bar > >");
}

#[test]
fn filter_preserves_non_skip_generics() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "HashMap < String , i32 >");
}

// ===========================================================================
// Additional roundtrip compositions
// ===========================================================================

#[test]
fn extract_through_skip_then_wrap_roundtrip() {
    let ty: Type = parse_quote!(Box<Option<char>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "char");
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    let (back, ok2) = try_extract_inner_type(&wrapped, "WithLeaf", &skip(&["adze"]));
    assert!(ok2);
    assert_eq!(ty_str(&back), "char");
}

#[test]
fn filter_two_layers_then_wrap_extract() {
    let ty: Type = parse_quote!(Arc<Box<f64>>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc", "Box"]));
    assert_eq!(ty_str(&filtered), "f64");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    let (extracted, ok) = try_extract_inner_type(&wrapped, "WithLeaf", &skip(&["adze"]));
    assert!(ok);
    assert_eq!(ty_str(&extracted), "f64");
}

#[test]
fn wrap_option_vec_skip_both_extract_option_then_vec() {
    let ty: Type = parse_quote!(Option<Vec<u16>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Vec < adze :: WithLeaf < u16 > > >"
    );
    let (vec_inner, ok1) = try_extract_inner_type(&wrapped, "Option", &skip(&[]));
    assert!(ok1);
    assert_eq!(ty_str(&vec_inner), "Vec < adze :: WithLeaf < u16 > >");
    let (leaf, ok2) = try_extract_inner_type(&vec_inner, "Vec", &skip(&[]));
    assert!(ok2);
    assert_eq!(ty_str(&leaf), "adze :: WithLeaf < u16 >");
}

#[test]
fn extract_vec_from_nested_then_filter_inner() {
    let ty: Type = parse_quote!(Vec<Box<Arc<i32>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    let filtered = filter_inner_type(&inner, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn filter_then_wrap_then_extract_full_chain() {
    let ty: Type = parse_quote!(Rc<Box<isize>>);
    let filtered = filter_inner_type(&ty, &skip(&["Rc", "Box"]));
    assert_eq!(ty_str(&filtered), "isize");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < isize >");
    let (extracted, ok) = try_extract_inner_type(&wrapped, "WithLeaf", &skip(&["adze"]));
    assert!(ok);
    assert_eq!(ty_str(&extracted), "isize");
}

#[test]
fn wrap_result_skip_result_extracts_both_args_wrapped() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        ty_str(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_deeply_nested_skip_all_containers() {
    let ty: Type = parse_quote!(Option<Vec<Box<u8>>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec", "Box"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Vec < Box < adze :: WithLeaf < u8 > > > >"
    );
}

#[test]
fn idempotent_filter_on_already_filtered() {
    let ty: Type = parse_quote!(Box<String>);
    let once = filter_inner_type(&ty, &skip(&["Box"]));
    let twice = filter_inner_type(&once, &skip(&["Box"]));
    assert_eq!(ty_str(&once), ty_str(&twice));
}

#[test]
fn extract_with_skip_not_present_returns_original() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), ty_str(&ty));
}

#[test]
fn filter_single_layer_then_extract_remaining() {
    let ty: Type = parse_quote!(Box<Vec<Option<u32>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < Option < u32 > >");
    let (inner, ok) = try_extract_inner_type(&filtered, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < u32 >");
}

#[test]
fn wrap_already_wrapped_type_nests() {
    let base: Type = parse_quote!(i32);
    let w1 = wrap_leaf_type(&base, &skip(&[]));
    let w2 = wrap_leaf_type(&w1, &skip(&[]));
    assert_eq!(ty_str(&w2), "adze :: WithLeaf < adze :: WithLeaf < i32 > >");
}

#[test]
fn extract_from_nongeneric_type_is_identity() {
    let ty: Type = parse_quote!(usize);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "usize");
}

#[test]
fn filter_nongeneric_type_is_identity() {
    let ty: Type = parse_quote!(f64);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]));
    assert_eq!(ty_str(&filtered), "f64");
}
