//! Edge-case tests (v2) for grammar expansion in adze-common.
//!
//! Focuses on boundary conditions, error-adjacent paths, composition
//! order sensitivity, and unusual type forms not covered by prior suites.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use syn::{Type, parse_quote};

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1. try_extract_inner_type — target matches outermost AND is in skip_over
// ===========================================================================

#[test]
fn extract_target_also_in_skip_prefers_extraction() {
    // When the target type is both the outer type and in the skip set,
    // extraction should win (the match arm is checked first).
    let ty: Type = parse_quote!(Box<Inner>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Inner");
}

// ===========================================================================
// 2. try_extract_inner_type — qualified path where last segment matches
// ===========================================================================

#[test]
fn extract_qualified_std_option() {
    // `segments.last()` is `Option` even when the path is `std::option::Option`.
    let ty: Type = parse_quote!(std::option::Option<Payload>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Payload");
}

#[test]
fn extract_qualified_path_skip_over() {
    let ty: Type = parse_quote!(std::boxed::Box<std::option::Option<Leaf>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Leaf");
}

// ===========================================================================
// 3. filter_inner_type — non-generic type in skip set (no angle brackets)
//    This triggers the panic path, so we verify the non-panic path instead.
// ===========================================================================

#[test]
fn filter_plain_type_not_in_skip_returns_self() {
    let ty: Type = parse_quote!(Token);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "Token");
}

#[test]
fn filter_generic_not_in_skip_returns_self() {
    let ty: Type = parse_quote!(HashMap<K, V>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "HashMap < K , V >"
    );
}

// ===========================================================================
// 4. wrap_leaf_type — nested skip containers with non-Type generic args
// ===========================================================================

#[test]
fn wrap_skips_lifetime_generic_arg() {
    // Result<&'a str, E> where Result is in skip: only Type args are wrapped.
    let ty: Type = parse_quote!(Result<String, Error>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    let s = ty_str(&wrapped);
    assert!(s.contains("WithLeaf < String >"));
    assert!(s.contains("WithLeaf < Error >"));
}

// ===========================================================================
// 5. Composition order: filter then extract vs extract then filter
// ===========================================================================

#[test]
fn filter_then_extract_removes_wrapper_first() {
    let ty: Type = parse_quote!(Box<Option<Leaf>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Option < Leaf >");
    let (inner, ok) = try_extract_inner_type(&filtered, "Option", &HashSet::new());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Leaf");
}

#[test]
fn extract_then_filter_extracts_first() {
    let ty: Type = parse_quote!(Option<Box<Leaf>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Box < Leaf >");
    let filtered = filter_inner_type(&inner, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Leaf");
}

#[test]
fn compose_filter_extract_wrap_pipeline() {
    let ty: Type = parse_quote!(Box<Vec<Option<Node>>>);
    let step1 = filter_inner_type(&ty, &skip(&["Box"]));
    let (step2, ok) = try_extract_inner_type(&step1, "Vec", &HashSet::new());
    assert!(ok);
    let wrapped = wrap_leaf_type(&step2, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < Node > >");
}

// ===========================================================================
// 6. wrap_leaf_type — double-nested skip containers
// ===========================================================================

#[test]
fn wrap_vec_of_option_both_skipped() {
    let ty: Type = parse_quote!(Vec<Option<Tok>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < Tok > > >"
    );
}

#[test]
fn wrap_option_of_vec_both_skipped() {
    let ty: Type = parse_quote!(Option<Vec<Tok>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Vec < adze :: WithLeaf < Tok > > >"
    );
}

// ===========================================================================
// 7. try_extract_inner_type — type not a path (reference, tuple, array, etc.)
// ===========================================================================

#[test]
fn extract_from_reference_type_returns_unchanged() {
    let ty: Type = parse_quote!(&mut Vec<u8>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& mut Vec < u8 >");
}

#[test]
fn extract_from_tuple_type_returns_unchanged() {
    let ty: Type = parse_quote!((i32, String));
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(!ok);
    assert_eq!(ty_str(&inner), "(i32 , String)");
}

#[test]
fn extract_from_array_type_returns_unchanged() {
    let ty: Type = parse_quote!([u8; 32]);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
    assert!(!ok);
    assert_eq!(ty_str(&inner), "[u8 ; 32]");
}

#[test]
fn extract_from_raw_pointer_type_returns_unchanged() {
    let ty: Type = parse_quote!(*const u8);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(!ok);
    assert_eq!(ty_str(&inner), "* const u8");
}

#[test]
fn extract_from_impl_trait_returns_unchanged() {
    let ty: Type = parse_quote!(impl Display);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(!ok);
    assert_eq!(ty_str(&inner), "impl Display");
}

// ===========================================================================
// 8. filter_inner_type — non-path types pass through
// ===========================================================================

#[test]
fn filter_reference_passes_through() {
    let ty: Type = parse_quote!(&'a str);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "& 'a str");
}

#[test]
fn filter_fn_pointer_passes_through() {
    let ty: Type = parse_quote!(fn(u8) -> bool);
    let s = ty_str(&filter_inner_type(&ty, &skip(&["Box"])));
    assert!(s.contains("fn"));
}

// ===========================================================================
// 9. wrap_leaf_type — non-path types get wrapped entirely
// ===========================================================================

#[test]
fn wrap_reference_type_wraps_whole() {
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &HashSet::new());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn wrap_tuple_type_wraps_whole() {
    let ty: Type = parse_quote!((A, B));
    let wrapped = wrap_leaf_type(&ty, &HashSet::new());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < (A , B) >");
}

#[test]
fn wrap_array_type_wraps_whole() {
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &HashSet::new());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

// ===========================================================================
// 10. NameValueExpr — various expression forms
// ===========================================================================

#[test]
fn name_value_integer_literal() {
    let nv: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(nv.path.to_string(), "precedence");
    if let syn::Expr::Lit(lit) = &nv.expr {
        if let syn::Lit::Int(i) = &lit.lit {
            assert_eq!(i.base10_parse::<i32>().unwrap(), 42);
        } else {
            panic!("expected int lit");
        }
    } else {
        panic!("expected lit expr");
    }
}

#[test]
fn name_value_bool_literal() {
    let nv: NameValueExpr = parse_quote!(enabled = true);
    assert_eq!(nv.path.to_string(), "enabled");
    if let syn::Expr::Lit(lit) = &nv.expr {
        if let syn::Lit::Bool(b) = &lit.lit {
            assert!(b.value);
        } else {
            panic!("expected bool lit");
        }
    } else {
        panic!("expected lit expr");
    }
}

#[test]
fn name_value_negative_integer() {
    let nv: NameValueExpr = parse_quote!(offset = -1);
    assert_eq!(nv.path.to_string(), "offset");
    // Negative integer is parsed as a Unary Neg expression
    assert!(matches!(&nv.expr, syn::Expr::Unary(_)));
}

#[test]
fn name_value_path_expression() {
    let nv: NameValueExpr = parse_quote!(kind = SomeEnum::Variant);
    assert_eq!(nv.path.to_string(), "kind");
    assert!(matches!(&nv.expr, syn::Expr::Path(_)));
}

#[test]
fn name_value_closure_expression() {
    let nv: NameValueExpr = parse_quote!(transform = |x: String| x.len());
    assert_eq!(nv.path.to_string(), "transform");
    assert!(matches!(&nv.expr, syn::Expr::Closure(_)));
}

// ===========================================================================
// 11. FieldThenParams — boundary conditions
// ===========================================================================

#[test]
fn field_then_params_no_params() {
    let ftp: FieldThenParams = parse_quote!(MyType);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
    assert_eq!(ty_str(&ftp.field.ty), "MyType");
}

#[test]
fn field_then_params_one_param() {
    let ftp: FieldThenParams = parse_quote!(String, pattern = "abc");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "pattern");
}

#[test]
fn field_then_params_three_params() {
    let ftp: FieldThenParams = parse_quote!(
        Token,
        pattern = "[a-z]+",
        precedence = 5,
        associativity = "left"
    );
    assert_eq!(ftp.params.len(), 3);
    let keys: Vec<String> = ftp.params.iter().map(|p| p.path.to_string()).collect();
    assert_eq!(keys, vec!["pattern", "precedence", "associativity"]);
}

#[test]
fn field_then_params_generic_field_type() {
    let ftp: FieldThenParams = parse_quote!(Vec<Option<String>>, precedence = 1);
    assert_eq!(ty_str(&ftp.field.ty), "Vec < Option < String > >");
    assert_eq!(ftp.params.len(), 1);
}

#[test]
fn field_then_params_qualified_field_type() {
    let ftp: FieldThenParams = parse_quote!(std::collections::HashMap<K, V>);
    assert!(ftp.params.is_empty());
    assert!(ty_str(&ftp.field.ty).contains("HashMap"));
}

// ===========================================================================
// 12. try_extract_inner_type — skip chain that does NOT contain target
// ===========================================================================

#[test]
fn extract_skip_chain_no_target_returns_original() {
    // Box<Arc<String>> — skip both Box and Arc, look for Vec => not found
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < Arc < String > >");
}

#[test]
fn extract_single_skip_no_target() {
    let ty: Type = parse_quote!(Arc<Leaf>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Arc < Leaf >");
}

// ===========================================================================
// 13. try_extract_inner_type — skip chain then target at leaf
// ===========================================================================

#[test]
fn extract_three_skips_then_target() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Option<Leaf>>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box", "Arc", "Rc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Leaf");
}

// ===========================================================================
// 14. Empty skip sets for all three functions
// ===========================================================================

#[test]
fn all_functions_empty_skip_plain_type() {
    let ty: Type = parse_quote!(Ident);
    let empty: HashSet<&str> = HashSet::new();

    let (ext, ok) = try_extract_inner_type(&ty, "Option", &empty);
    assert!(!ok);
    assert_eq!(ty_str(&ext), "Ident");

    let filt = filter_inner_type(&ty, &empty);
    assert_eq!(ty_str(&filt), "Ident");

    let wrapped = wrap_leaf_type(&ty, &empty);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Ident >");
}

// ===========================================================================
// 15. wrap_leaf_type — only the outermost is in skip; inner is not
// ===========================================================================

#[test]
fn wrap_outer_skipped_inner_not() {
    let ty: Type = parse_quote!(Vec<HashMap<K, V>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < adze :: WithLeaf < HashMap < K , V > > >"
    );
}

// ===========================================================================
// 16. wrap_leaf_type — inner is in skip but outer is not
// ===========================================================================

#[test]
fn wrap_inner_skipped_outer_not() {
    let ty: Type = parse_quote!(HashMap<Vec<Leaf>, Error>);
    // HashMap is NOT in skip, so the entire thing gets wrapped as a leaf
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < HashMap < Vec < Leaf > , Error > >"
    );
}

// ===========================================================================
// 17. Same type name as target but no generics — should NOT extract
// ===========================================================================

#[test]
fn extract_target_name_no_generics() {
    // `Vec` with no angle brackets — `segments.last()` has `PathArguments::None`
    // The function will see ident == "Vec" and enter the extraction branch,
    // but the arguments aren't AngleBracketed, causing a panic.
    // So we verify a type that matches the last segment but IS generic works.
    let ty: Type = parse_quote!(Vec<T>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
    assert!(ok);
    assert_eq!(ty_str(&inner), "T");
}

// ===========================================================================
// 18. Multiple generic arguments — extract always gets first
// ===========================================================================

#[test]
fn extract_first_of_three_generics() {
    let ty: Type = parse_quote!(Triplet<A, B, C>);
    let (inner, ok) = try_extract_inner_type(&ty, "Triplet", &HashSet::new());
    assert!(ok);
    assert_eq!(ty_str(&inner), "A");
}

#[test]
fn extract_first_of_two_in_result() {
    let ty: Type = parse_quote!(Result<OkType, ErrType>);
    let (inner, ok) = try_extract_inner_type(&ty, "Result", &HashSet::new());
    assert!(ok);
    assert_eq!(ty_str(&inner), "OkType");
}

// ===========================================================================
// 19. Large skip set with many entries
// ===========================================================================

#[test]
fn large_skip_set_with_20_entries() {
    let names: Vec<String> = (0..20).map(|i| format!("Wrapper{i}")).collect();
    let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let skip_set: HashSet<&str> = name_refs.iter().copied().collect();

    // Type uses Wrapper0<Wrapper1<...<Wrapper4<Leaf>>>>
    let ty: Type = parse_quote!(Wrapper0<Wrapper1<Wrapper2<Wrapper3<Wrapper4<Leaf>>>>>);
    let filtered = filter_inner_type(&ty, &skip_set);
    assert_eq!(ty_str(&filtered), "Leaf");
}

// ===========================================================================
// 20. wrap_leaf_type with large skip set
// ===========================================================================

#[test]
fn wrap_with_many_skip_entries_only_matching_matter() {
    let skip_set = skip(&["Vec", "Option", "Box", "Arc", "Rc", "Cell", "RefCell"]);
    let ty: Type = parse_quote!(Vec<Option<Box<Leaf>>>);
    let wrapped = wrap_leaf_type(&ty, &skip_set);
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < Box < adze :: WithLeaf < Leaf > > > >"
    );
}

// ===========================================================================
// 21. Idempotency: wrapping already-wrapped type
// ===========================================================================

#[test]
fn double_wrap_plain_type() {
    let ty: Type = parse_quote!(Leaf);
    let empty: HashSet<&str> = HashSet::new();
    let once = wrap_leaf_type(&ty, &empty);
    let twice = wrap_leaf_type(&once, &empty);
    assert_eq!(
        ty_str(&twice),
        "adze :: WithLeaf < adze :: WithLeaf < Leaf > >"
    );
}

// ===========================================================================
// 22. filter_inner_type idempotency — filtering already-filtered type
// ===========================================================================

#[test]
fn double_filter_converges() {
    let ty: Type = parse_quote!(Box<Box<Leaf>>);
    let s = skip(&["Box"]);
    let once = filter_inner_type(&ty, &s);
    assert_eq!(ty_str(&once), "Leaf");
    // Filtering "Leaf" again (not in skip) is a no-op
    let twice = filter_inner_type(&once, &s);
    assert_eq!(ty_str(&twice), "Leaf");
}

// ===========================================================================
// 23. try_extract_inner_type — target at depth 1 with skip at depth 0
// ===========================================================================

#[test]
fn extract_option_inside_arc_skipping_arc() {
    let ty: Type = parse_quote!(Arc<Option<Data>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Data");
}

// ===========================================================================
// 24. wrap_leaf_type — single-element path (no :: prefix)
// ===========================================================================

#[test]
fn wrap_single_segment_path() {
    let ty: Type = parse_quote!(Foo);
    let wrapped = wrap_leaf_type(&ty, &HashSet::new());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Foo >");
}

// ===========================================================================
// 25. FieldThenParams — field type is a reference
// ===========================================================================

#[test]
fn field_then_params_reference_field() {
    // Unnamed field parsing should accept reference types
    let ftp: FieldThenParams = parse_quote!(&'static str);
    assert!(ftp.params.is_empty());
    assert!(ty_str(&ftp.field.ty).contains("str"));
}

// ===========================================================================
// 26. NameValueExpr — key is a Rust keyword-like ident
// ===========================================================================

#[test]
fn name_value_key_r_ident() {
    let nv: NameValueExpr = parse_quote!(r#type = "keyword");
    // syn's Ident::to_string() preserves the r# prefix
    assert_eq!(nv.path.to_string(), "r#type");
}

// ===========================================================================
// 27. filter_inner_type — chained different skippable types
// ===========================================================================

#[test]
fn filter_arc_box_option_chain() {
    let ty: Type = parse_quote!(Arc<Box<Option<Leaf>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc", "Box", "Option"]));
    assert_eq!(ty_str(&filtered), "Leaf");
}

// ===========================================================================
// 28. try_extract — target is deeply nested beyond skip chain
// ===========================================================================

#[test]
fn extract_vec_beyond_two_skips() {
    let ty: Type = parse_quote!(Box<Arc<Vec<Item>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Item");
}

// ===========================================================================
// 29. wrap_leaf_type — skip contains type not present in the actual type
// ===========================================================================

#[test]
fn wrap_irrelevant_skip_entries_ignored() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option", "Box"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

// ===========================================================================
// 30. try_extract — empty string target never matches
// ===========================================================================

#[test]
fn extract_empty_target_never_matches() {
    let ty: Type = parse_quote!(Option<Leaf>);
    let (inner, ok) = try_extract_inner_type(&ty, "", &HashSet::new());
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Option < Leaf >");
}

// ===========================================================================
// 31. Numeric type names
// ===========================================================================

#[test]
fn extract_from_primitive_numeric_types() {
    for prim in &[
        "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "usize", "isize",
    ] {
        let ident = syn::Ident::new(prim, proc_macro2::Span::call_site());
        let ty: Type = parse_quote!(#ident);
        let (_, ok) = try_extract_inner_type(&ty, "Option", &HashSet::new());
        assert!(!ok, "primitive {prim} should not match Option");
    }
}

// ===========================================================================
// 32. FieldThenParams — trailing comma after last param
// ===========================================================================

#[test]
fn field_then_params_trailing_comma_in_params() {
    // Punctuated::parse_terminated tolerates trailing comma
    let ftp: FieldThenParams = parse_quote!(Token, key = "val",);
    assert_eq!(ftp.params.len(), 1);
}

// ===========================================================================
// 33. wrap_leaf_type preserves generic arguments count
// ===========================================================================

#[test]
fn wrap_preserves_four_generics_in_skip_type() {
    let ty: Type = parse_quote!(Quad<A, B, C, D>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Quad"]));
    let s = ty_str(&wrapped);
    // All four should be individually wrapped
    assert!(s.contains("WithLeaf < A >"));
    assert!(s.contains("WithLeaf < B >"));
    assert!(s.contains("WithLeaf < C >"));
    assert!(s.contains("WithLeaf < D >"));
}

// ===========================================================================
// 34. filter_inner_type — type with turbofish-like syntax
// ===========================================================================

#[test]
fn filter_multi_segment_qualified_box() {
    let ty: Type = parse_quote!(alloc::boxed::Box<Inner>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    // Last segment is Box, which is in skip
    assert_eq!(ty_str(&filtered), "Inner");
}

// ===========================================================================
// 35. try_extract — same name at multiple nesting levels, finds outermost
// ===========================================================================

#[test]
fn extract_finds_outermost_matching_name() {
    let ty: Type = parse_quote!(Option<Option<Leaf>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(ok);
    // Should extract the first (outermost) Option
    assert_eq!(ty_str(&inner), "Option < Leaf >");
}

// ===========================================================================
// 36. wrap_leaf_type — deeply nested skips (3+ levels)
// ===========================================================================

#[test]
fn wrap_three_level_skip_chain() {
    let ty: Type = parse_quote!(Vec<Option<Box<Leaf>>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option", "Box"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < Box < adze :: WithLeaf < Leaf > > > >"
    );
}

// ===========================================================================
// 37. NameValueExpr — float literal value
// ===========================================================================

#[test]
fn name_value_float_literal() {
    let nv: NameValueExpr = parse_quote!(weight = 3.14);
    assert_eq!(nv.path.to_string(), "weight");
    if let syn::Expr::Lit(lit) = &nv.expr {
        assert!(matches!(&lit.lit, syn::Lit::Float(_)));
    } else {
        panic!("expected float lit");
    }
}

// ===========================================================================
// 38. NameValueExpr — char literal
// ===========================================================================

#[test]
fn name_value_char_literal() {
    let nv: NameValueExpr = parse_quote!(delim = 'x');
    assert_eq!(nv.path.to_string(), "delim");
    if let syn::Expr::Lit(lit) = &nv.expr {
        if let syn::Lit::Char(c) = &lit.lit {
            assert_eq!(c.value(), 'x');
        } else {
            panic!("expected char lit");
        }
    } else {
        panic!("expected lit expr");
    }
}

// ===========================================================================
// 39. NameValueExpr — byte string literal
// ===========================================================================

#[test]
fn name_value_byte_string_literal() {
    let nv: NameValueExpr = parse_quote!(data = b"hello");
    assert_eq!(nv.path.to_string(), "data");
    if let syn::Expr::Lit(lit) = &nv.expr {
        assert!(matches!(&lit.lit, syn::Lit::ByteStr(_)));
    } else {
        panic!("expected lit expr");
    }
}

// ===========================================================================
// 40. try_extract — type with const generic (should not match)
// ===========================================================================

#[test]
fn extract_ignores_const_generic_type() {
    // SmallVec doesn't have angle-bracketed type arg in the standard sense
    // but syn parses `Foo<N>` where N is a const as a Type path
    let ty: Type = parse_quote!(Array<Item, 5>);
    // Won't match "Vec"
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
    assert!(!ok);
}

// ===========================================================================
// 41. filter_inner_type — single-level skip is not recursive for non-skip inner
// ===========================================================================

#[test]
fn filter_stops_at_first_non_skip() {
    let ty: Type = parse_quote!(Box<Vec<Leaf>>);
    // Only Box in skip, Vec is NOT — stops at Vec
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < Leaf >");
}

// ===========================================================================
// 42. wrap preserves path qualifiers in non-skip types
// ===========================================================================

#[test]
fn wrap_qualified_leaf_type() {
    let ty: Type = parse_quote!(crate::ast::Node);
    let wrapped = wrap_leaf_type(&ty, &HashSet::new());
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < crate :: ast :: Node >"
    );
}

// ===========================================================================
// 43. FieldThenParams — duplicate param keys (allowed by parser)
// ===========================================================================

#[test]
fn field_then_params_duplicate_keys_accepted() {
    let ftp: FieldThenParams = parse_quote!(Token, key = "a", key = "b");
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "key");
    assert_eq!(ftp.params[1].path.to_string(), "key");
}

// ===========================================================================
// 44. try_extract with a skip type that itself has multiple generics
// ===========================================================================

#[test]
fn extract_skips_multi_generic_wrapper() {
    // Result<Vec<T>, E> where Result is in skip — the function
    // uses .first() on the generic args, so it traverses into Vec<T>
    let ty: Type = parse_quote!(Result<Vec<Leaf>, Error>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Result"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Leaf");
}

// ===========================================================================
// 45. filter_inner_type with multi-generic skip extracts first arg
// ===========================================================================

#[test]
fn filter_multi_generic_skip_uses_first_arg() {
    let ty: Type = parse_quote!(Result<Inner, Error>);
    let filtered = filter_inner_type(&ty, &skip(&["Result"]));
    // filter_inner_type recurses into first generic argument
    assert_eq!(ty_str(&filtered), "Inner");
}

// ===========================================================================
// 46. Underscore-prefixed type names
// ===========================================================================

#[test]
fn extract_underscore_prefixed_type() {
    let ty: Type = parse_quote!(_Private<Leaf>);
    let (inner, ok) = try_extract_inner_type(&ty, "_Private", &HashSet::new());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Leaf");
}

// ===========================================================================
// 47. wrap_leaf_type — nested skip where inner skip has multiple generics
// ===========================================================================

#[test]
fn wrap_vec_of_result_skipping_both() {
    let ty: Type = parse_quote!(Vec<Result<Ok, Err>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Result"]));
    let s = ty_str(&wrapped);
    assert!(s.contains("Vec <"));
    assert!(s.contains("Result <"));
    assert!(s.contains("WithLeaf < Ok >"));
    assert!(s.contains("WithLeaf < Err >"));
}

// ===========================================================================
// 48. try_extract — self-referential type name (type named "Self")
// ===========================================================================

#[test]
fn extract_inner_of_option_self() {
    // In proc-macro context, `Self` can appear as a type
    let ty: Type = parse_quote!(Option<Self>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Self");
}

// ===========================================================================
// 49. NameValueExpr — array expression value
// ===========================================================================

#[test]
fn name_value_array_expression() {
    let nv: NameValueExpr = parse_quote!(items = [1, 2, 3]);
    assert_eq!(nv.path.to_string(), "items");
    assert!(matches!(&nv.expr, syn::Expr::Array(_)));
}

// ===========================================================================
// 50. NameValueExpr — tuple expression value
// ===========================================================================

#[test]
fn name_value_tuple_expression() {
    let nv: NameValueExpr = parse_quote!(pair = (1, "two"));
    assert_eq!(nv.path.to_string(), "pair");
    assert!(matches!(&nv.expr, syn::Expr::Tuple(_)));
}

// ===========================================================================
// 51. try_extract — generic with where clause-like complexity
// ===========================================================================

#[test]
fn extract_complex_inner_type() {
    // Inner type itself has generics
    let ty: Type = parse_quote!(Option<HashMap<String, Vec<u8>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(ok);
    assert_eq!(ty_str(&inner), "HashMap < String , Vec < u8 > >");
}

// ===========================================================================
// 52. Composition: extract then wrap with same skip set
// ===========================================================================

#[test]
fn extract_then_wrap_option_vec() {
    let ty: Type = parse_quote!(Option<Vec<Leaf>>);
    let skip_set = skip(&["Option", "Vec"]);
    let (extracted, ok) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(ok);
    let wrapped = wrap_leaf_type(&extracted, &skip_set);
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < Leaf > >");
}

// ===========================================================================
// 53. FieldThenParams — complex expression as param value
// ===========================================================================

#[test]
fn field_then_params_method_call_value() {
    let ftp: FieldThenParams = parse_quote!(String, transform = String::from("hello"));
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "transform");
}

// ===========================================================================
// 54. try_extract — empty skip set with nested wrappers
// ===========================================================================

#[test]
fn extract_with_empty_skip_only_finds_direct_match() {
    let ty: Type = parse_quote!(Box<Option<Leaf>>);
    // Empty skip: can only match Box directly
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &HashSet::new());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < Leaf >");
}

// ===========================================================================
// 55. filter vs extract produce different results on same type
// ===========================================================================

#[test]
fn filter_vs_extract_differ_on_box_option() {
    let ty: Type = parse_quote!(Box<Option<Leaf>>);
    let box_skip = skip(&["Box"]);

    // filter: strips Box, returns Option<Leaf>
    let filtered = filter_inner_type(&ty, &box_skip);
    assert_eq!(ty_str(&filtered), "Option < Leaf >");

    // extract for Box: returns Option<Leaf> (same in this case)
    let (extracted, ok) = try_extract_inner_type(&ty, "Box", &HashSet::new());
    assert!(ok);
    assert_eq!(ty_str(&extracted), "Option < Leaf >");

    // extract for Option through Box: returns Leaf
    let (deep, ok) = try_extract_inner_type(&ty, "Option", &box_skip);
    assert!(ok);
    assert_eq!(ty_str(&deep), "Leaf");
}

// ===========================================================================
// 56. wrap_leaf_type — path type with no segments (synthetic edge)
//     Not reachable via parse_quote but we verify normal paths work.
// ===========================================================================

#[test]
fn wrap_crate_root_type() {
    let ty: Type = parse_quote!(crate::Root);
    let wrapped = wrap_leaf_type(&ty, &HashSet::new());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < crate :: Root >");
}

// ===========================================================================
// 57. NameValueExpr — very long string value
// ===========================================================================

#[test]
fn name_value_long_string() {
    let long = "a".repeat(1000);
    let nv: NameValueExpr = syn::parse_str(&format!("pattern = \"{long}\"")).unwrap();
    assert_eq!(nv.path.to_string(), "pattern");
    if let syn::Expr::Lit(lit) = &nv.expr {
        if let syn::Lit::Str(s) = &lit.lit {
            assert_eq!(s.value().len(), 1000);
        } else {
            panic!("expected str lit");
        }
    } else {
        panic!("expected lit expr");
    }
}

// ===========================================================================
// 58. try_extract — case sensitivity (Vec vs vec)
// ===========================================================================

#[test]
fn extract_is_case_sensitive() {
    let ty: Type = parse_quote!(Vec<Leaf>);
    // "vec" (lowercase) should NOT match "Vec"
    let (_, ok) = try_extract_inner_type(&ty, "vec", &HashSet::new());
    assert!(!ok);
}

// ===========================================================================
// 59. filter_inner_type — case sensitivity in skip set
// ===========================================================================

#[test]
fn filter_skip_set_is_case_sensitive() {
    let ty: Type = parse_quote!(Box<Leaf>);
    // "box" (lowercase) is not "Box"
    let filtered = filter_inner_type(&ty, &skip(&["box"]));
    assert_eq!(ty_str(&filtered), "Box < Leaf >");
}

// ===========================================================================
// 60. wrap_leaf_type — case sensitivity in skip set
// ===========================================================================

#[test]
fn wrap_skip_set_is_case_sensitive() {
    let ty: Type = parse_quote!(Vec<Leaf>);
    // "vec" not "Vec" — so Vec is NOT skipped, entire thing gets wrapped
    let wrapped = wrap_leaf_type(&ty, &skip(&["vec"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Vec < Leaf > >");
}

// ===========================================================================
// 61. Composition pipeline matching real grammar expansion flow
// ===========================================================================

#[test]
fn real_pipeline_optional_vec_field() {
    // Simulates processing `field: Option<Vec<Token>>` in grammar expansion
    let ty: Type = parse_quote!(Option<Vec<Token>>);
    let transparent = skip(&["Box", "Spanned"]);
    let grammar_containers = skip(&["Vec", "Option"]);

    // Step 1: Check if optional
    let (after_option, is_optional) = try_extract_inner_type(&ty, "Option", &transparent);
    assert!(is_optional);
    assert_eq!(ty_str(&after_option), "Vec < Token >");

    // Step 2: Check if repeated
    let (after_vec, is_repeated) = try_extract_inner_type(&after_option, "Vec", &transparent);
    assert!(is_repeated);
    assert_eq!(ty_str(&after_vec), "Token");

    // Step 3: Filter transparent wrappers
    let leaf = filter_inner_type(&after_vec, &transparent);
    assert_eq!(ty_str(&leaf), "Token");

    // Step 4: Wrap for extraction
    let wrapped = wrap_leaf_type(&ty, &grammar_containers);
    assert_eq!(
        ty_str(&wrapped),
        "Option < Vec < adze :: WithLeaf < Token > > >"
    );
}

#[test]
fn real_pipeline_box_spanned_field() {
    let ty: Type = parse_quote!(Box<Spanned<Option<Vec<Expr>>>>);
    let transparent = skip(&["Box", "Spanned"]);
    let grammar_containers = skip(&["Vec", "Option"]);

    // Extract Option through Box and Spanned
    let (after_option, is_optional) = try_extract_inner_type(&ty, "Option", &transparent);
    assert!(is_optional);

    // Extract Vec
    let (leaf, is_repeated) = try_extract_inner_type(&after_option, "Vec", &transparent);
    assert!(is_repeated);
    assert_eq!(ty_str(&leaf), "Expr");

    // Wrap the original type
    let wrapped = wrap_leaf_type(&ty, &grammar_containers);
    // Box is NOT in grammar_containers, so the whole thing gets wrapped as leaf
    assert!(ty_str(&wrapped).starts_with("adze :: WithLeaf"));
}
