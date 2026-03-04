//! Comprehensive edge-case tests for macro expansion helpers in adze-common.
//!
//! Covers NameValueExpr parsing, FieldThenParams with complex generics,
//! try_extract_inner_type, filter_inner_type, wrap_leaf_type, and various
//! edge cases around empty inputs, deeply nested generics, and multiple
//! type parameters.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use syn::{Type, parse_quote};

// ===========================================================================
// Helpers
// ===========================================================================

fn ts(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

// ===========================================================================
// 1. NameValueExpr parsing — various formats
// ===========================================================================

#[test]
fn nve_string_literal_value() {
    let nve: NameValueExpr = parse_quote!(name = "hello");
    assert_eq!(nve.path.to_string(), "name");
}

#[test]
fn nve_integer_literal_value() {
    let nve: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(nve.path.to_string(), "precedence");
    if let syn::Expr::Lit(lit) = &nve.expr {
        if let syn::Lit::Int(i) = &lit.lit {
            assert_eq!(i.base10_parse::<i32>().unwrap(), 42);
        } else {
            panic!("expected int literal");
        }
    } else {
        panic!("expected Expr::Lit");
    }
}

#[test]
fn nve_negative_integer_value() {
    let nve: NameValueExpr = parse_quote!(offset = -1);
    assert_eq!(nve.path.to_string(), "offset");
}

#[test]
fn nve_boolean_true_value() {
    let nve: NameValueExpr = parse_quote!(enabled = true);
    assert_eq!(nve.path.to_string(), "enabled");
    if let syn::Expr::Lit(lit) = &nve.expr {
        if let syn::Lit::Bool(b) = &lit.lit {
            assert!(b.value);
        } else {
            panic!("expected bool literal");
        }
    } else {
        panic!("expected Expr::Lit");
    }
}

#[test]
fn nve_boolean_false_value() {
    let nve: NameValueExpr = parse_quote!(hidden = false);
    assert_eq!(nve.path.to_string(), "hidden");
}

#[test]
fn nve_float_literal_value() {
    let nve: NameValueExpr = parse_quote!(weight = 3.14);
    assert_eq!(nve.path.to_string(), "weight");
}

#[test]
fn nve_raw_string_literal() {
    let nve: NameValueExpr = parse_quote!(pattern = r"foo\d+bar");
    assert_eq!(nve.path.to_string(), "pattern");
    if let syn::Expr::Lit(lit) = &nve.expr {
        if let syn::Lit::Str(s) = &lit.lit {
            assert_eq!(s.value(), r"foo\d+bar");
        } else {
            panic!("expected str literal");
        }
    } else {
        panic!("expected Expr::Lit");
    }
}

#[test]
fn nve_empty_string_value() {
    let nve: NameValueExpr = parse_quote!(label = "");
    if let syn::Expr::Lit(lit) = &nve.expr {
        if let syn::Lit::Str(s) = &lit.lit {
            assert_eq!(s.value(), "");
        } else {
            panic!("expected str");
        }
    } else {
        panic!("expected Expr::Lit");
    }
}

#[test]
fn nve_char_literal_value() {
    let nve: NameValueExpr = parse_quote!(separator = 'x');
    assert_eq!(nve.path.to_string(), "separator");
}

#[test]
fn nve_path_expr_value() {
    let nve: NameValueExpr = parse_quote!(transform = some_function);
    assert_eq!(nve.path.to_string(), "transform");
}

#[test]
fn nve_closure_expr_value() {
    let nve: NameValueExpr = parse_quote!(transform = |x: i32| x + 1);
    assert_eq!(nve.path.to_string(), "transform");
}

#[test]
fn nve_underscore_in_name() {
    let nve: NameValueExpr = parse_quote!(my_param = "val");
    assert_eq!(nve.path.to_string(), "my_param");
}

#[test]
fn nve_long_name() {
    let long = "a_very_long_parameter_name_for_testing";
    let ident = syn::Ident::new(long, proc_macro2::Span::call_site());
    let nve: NameValueExpr = parse_quote!(#ident = 1);
    assert_eq!(nve.path.to_string(), long);
}

#[test]
fn nve_unicode_string_value() {
    let nve: NameValueExpr = parse_quote!(text = "日本語テスト");
    if let syn::Expr::Lit(lit) = &nve.expr {
        if let syn::Lit::Str(s) = &lit.lit {
            assert_eq!(s.value(), "日本語テスト");
        } else {
            panic!("expected str");
        }
    } else {
        panic!("expected Expr::Lit");
    }
}

#[test]
fn nve_escape_sequences_in_string() {
    let nve: NameValueExpr = parse_quote!(text = "line1\nline2\ttab");
    if let syn::Expr::Lit(lit) = &nve.expr {
        if let syn::Lit::Str(s) = &lit.lit {
            assert_eq!(s.value(), "line1\nline2\ttab");
        } else {
            panic!("expected str");
        }
    } else {
        panic!("expected Expr::Lit");
    }
}

#[test]
fn nve_byte_literal_value() {
    let nve: NameValueExpr = parse_quote!(delim = b'x');
    assert_eq!(nve.path.to_string(), "delim");
}

// ===========================================================================
// 2. FieldThenParams — complex generics
// ===========================================================================

#[test]
fn ftp_bare_type_no_params() {
    let ftp: FieldThenParams = parse_quote!(MyType);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
    assert_eq!(ts(&ftp.field.ty), "MyType");
}

#[test]
fn ftp_generic_type_no_params() {
    let ftp: FieldThenParams = parse_quote!(Vec<String>);
    assert!(ftp.comma.is_none());
    assert_eq!(ts(&ftp.field.ty), "Vec < String >");
}

#[test]
fn ftp_nested_generic_no_params() {
    let ftp: FieldThenParams = parse_quote!(Option<Vec<i32>>);
    assert!(ftp.comma.is_none());
    assert_eq!(ts(&ftp.field.ty), "Option < Vec < i32 > >");
}

#[test]
fn ftp_deeply_nested_generic() {
    let ftp: FieldThenParams = parse_quote!(Box<Option<Vec<HashMap<String, i32>>>>);
    assert!(ftp.comma.is_none());
    assert_eq!(
        ts(&ftp.field.ty),
        "Box < Option < Vec < HashMap < String , i32 > > > >"
    );
}

#[test]
fn ftp_with_single_param() {
    let ftp: FieldThenParams = parse_quote!(Token, pattern = "[a-z]+");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "pattern");
}

#[test]
fn ftp_with_multiple_params() {
    let ftp: FieldThenParams = parse_quote!(String, pattern = "\\d+", precedence = 5);
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "pattern");
    assert_eq!(ftp.params[1].path.to_string(), "precedence");
}

#[test]
fn ftp_with_three_params() {
    let ftp: FieldThenParams = parse_quote!(
        Ident,
        pattern = "[a-z]+",
        precedence = 1,
        transform = to_upper
    );
    assert_eq!(ftp.params.len(), 3);
}

#[test]
fn ftp_generic_with_params() {
    let ftp: FieldThenParams = parse_quote!(Option<Token>, pattern = "keyword");
    assert!(ftp.comma.is_some());
    assert_eq!(ts(&ftp.field.ty), "Option < Token >");
    assert_eq!(ftp.params.len(), 1);
}

#[test]
fn ftp_tuple_type() {
    let ftp: FieldThenParams = parse_quote!((i32, String));
    assert!(ftp.comma.is_none());
    assert_eq!(ts(&ftp.field.ty), "(i32 , String)");
}

#[test]
fn ftp_reference_type() {
    let ftp: FieldThenParams = parse_quote!(&str);
    assert_eq!(ts(&ftp.field.ty), "& str");
}

#[test]
fn ftp_array_type() {
    let ftp: FieldThenParams = parse_quote!([u8; 4]);
    assert_eq!(ts(&ftp.field.ty), "[u8 ; 4]");
}

#[test]
fn ftp_unit_type() {
    let ftp: FieldThenParams = parse_quote!(());
    assert_eq!(ts(&ftp.field.ty), "()");
}

#[test]
fn ftp_qualified_path_type() {
    let ftp: FieldThenParams = parse_quote!(std::collections::HashMap<String, Vec<i32>>);
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_multiple_fields_independently_parsed() {
    let f1: FieldThenParams = parse_quote!(Alpha);
    let f2: FieldThenParams = parse_quote!(Beta, key = "val");
    let f3: FieldThenParams = parse_quote!(Option<Gamma>);
    assert_eq!(ts(&f1.field.ty), "Alpha");
    assert_eq!(ts(&f2.field.ty), "Beta");
    assert_eq!(ts(&f3.field.ty), "Option < Gamma >");
    assert!(f1.params.is_empty());
    assert_eq!(f2.params.len(), 1);
    assert!(f3.params.is_empty());
}

// ===========================================================================
// 3. try_extract_inner_type — nested generics
// ===========================================================================

#[test]
fn extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn extract_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ts(&inner), "i32");
}

#[test]
fn extract_option_vec_t() {
    let ty: Type = parse_quote!(Option<Vec<Token>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ts(&inner), "Vec < Token >");
}

#[test]
fn extract_through_box_to_option() {
    let ty: Type = parse_quote!(Box<Option<u64>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(extracted);
    assert_eq!(ts(&inner), "u64");
}

#[test]
fn extract_through_arc_box_to_vec() {
    let ty: Type = parse_quote!(Arc<Box<Vec<f32>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Arc", "Box"]));
    assert!(extracted);
    assert_eq!(ts(&inner), "f32");
}

#[test]
fn extract_no_match_returns_original() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ts(&inner), "HashMap < String , i32 >");
}

#[test]
fn extract_non_path_type_reference() {
    let ty: Type = parse_quote!(&str);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ts(&inner), "& str");
}

#[test]
fn extract_non_path_type_tuple() {
    let ty: Type = parse_quote!((i32, u32));
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ts(&inner), "(i32 , u32)");
}

#[test]
fn extract_non_path_type_array() {
    let ty: Type = parse_quote!([u8; 16]);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ts(&inner), "[u8 ; 16]");
}

#[test]
fn extract_skip_without_target_inside() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(!extracted);
    assert_eq!(ts(&inner), "Box < String >");
}

#[test]
fn extract_triple_nested_skip() {
    let ty: Type = parse_quote!(Arc<Rc<Box<Option<Leaf>>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Arc", "Rc", "Box"]));
    assert!(extracted);
    assert_eq!(ts(&inner), "Leaf");
}

#[test]
fn extract_target_at_top_level_with_skip_set() {
    let ty: Type = parse_quote!(Vec<Stuff>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(extracted);
    assert_eq!(ts(&inner), "Stuff");
}

#[test]
fn extract_plain_type_no_generics() {
    let ty: Type = parse_quote!(String);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn extract_option_option_nested() {
    let ty: Type = parse_quote!(Option<Option<bool>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ts(&inner), "Option < bool >");
}

#[test]
fn extract_vec_of_option() {
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ts(&inner), "Option < String >");
}

// ===========================================================================
// 4. filter_inner_type — non-matching and complex
// ===========================================================================

#[test]
fn filter_no_match_returns_original() {
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "String");
}

#[test]
fn filter_box_strips() {
    let ty: Type = parse_quote!(Box<Token>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "Token");
}

#[test]
fn filter_arc_strips() {
    let ty: Type = parse_quote!(Arc<Data>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc"]));
    assert_eq!(ts(&filtered), "Data");
}

#[test]
fn filter_double_box() {
    let ty: Type = parse_quote!(Box<Box<Inner>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "Inner");
}

#[test]
fn filter_box_arc_mixed() {
    let ty: Type = parse_quote!(Box<Arc<Box<Leaf>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ts(&filtered), "Leaf");
}

#[test]
fn filter_stops_at_non_skip_type() {
    let ty: Type = parse_quote!(Box<Vec<Inner>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "Vec < Inner >");
}

#[test]
fn filter_non_path_tuple() {
    let ty: Type = parse_quote!((A, B));
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "(A , B)");
}

#[test]
fn filter_non_path_reference() {
    let ty: Type = parse_quote!(&mut i32);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "& mut i32");
}

#[test]
fn filter_empty_skip_set() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ts(&filtered), "Box < String >");
}

#[test]
fn filter_three_layers_all_skippable() {
    let ty: Type = parse_quote!(Rc<Arc<Box<Core>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Rc", "Arc", "Box"]));
    assert_eq!(ts(&filtered), "Core");
}

#[test]
fn filter_option_not_in_skip_stays() {
    let ty: Type = parse_quote!(Option<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "Option < String >");
}

#[test]
fn filter_plain_primitive_type() {
    let ty: Type = parse_quote!(u64);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ts(&filtered), "u64");
}

// ===========================================================================
// 5. wrap_leaf_type — primitive and complex types
// ===========================================================================

#[test]
fn wrap_primitive_i32() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_primitive_bool() {
    let ty: Type = parse_quote!(bool);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < bool >");
}

#[test]
fn wrap_primitive_f64() {
    let ty: Type = parse_quote!(f64);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < f64 >");
}

#[test]
fn wrap_string_type() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_vec_preserves_container() {
    let ty: Type = parse_quote!(Vec<Token>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ts(&wrapped), "Vec < adze :: WithLeaf < Token > >");
}

#[test]
fn wrap_option_preserves_container() {
    let ty: Type = parse_quote!(Option<Node>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ts(&wrapped), "Option < adze :: WithLeaf < Node > >");
}

#[test]
fn wrap_vec_option_both_preserved() {
    let ty: Type = parse_quote!(Vec<Option<Leaf>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(ts(&wrapped), "Vec < Option < adze :: WithLeaf < Leaf > > >");
}

#[test]
fn wrap_option_vec_both_preserved() {
    let ty: Type = parse_quote!(Option<Vec<Leaf>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(ts(&wrapped), "Option < Vec < adze :: WithLeaf < Leaf > > >");
}

#[test]
fn wrap_non_skip_generic_wraps_whole() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ts(&wrapped),
        "adze :: WithLeaf < HashMap < String , i32 > >"
    );
}

#[test]
fn wrap_tuple_type() {
    let ty: Type = parse_quote!((i32, String));
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < (i32 , String) >");
}

#[test]
fn wrap_reference_type() {
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn wrap_array_type() {
    let ty: Type = parse_quote!([u8; 32]);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < [u8 ; 32] >");
}

#[test]
fn wrap_unit_type() {
    let ty: Type = parse_quote!(());
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < () >");
}

#[test]
fn wrap_result_with_both_args_wrapped() {
    let ty: Type = parse_quote!(Result<Good, Bad>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        ts(&wrapped),
        "Result < adze :: WithLeaf < Good > , adze :: WithLeaf < Bad > >"
    );
}

#[test]
fn wrap_skip_not_in_set_wraps_whole() {
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < Vec < i32 > >");
}

#[test]
fn wrap_empty_skip_set_wraps_everything() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < Vec < String > >");
}

// ===========================================================================
// 6. Edge cases — deeply nested generics
// ===========================================================================

#[test]
fn deep_nested_extract_option_inside_four_boxes() {
    let ty: Type = parse_quote!(Box<Box<Box<Box<Option<Leaf>>>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(extracted);
    assert_eq!(ts(&inner), "Leaf");
}

#[test]
fn deep_nested_filter_five_layers() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Box<Arc<Core>>>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]));
    assert_eq!(ts(&filtered), "Core");
}

#[test]
fn deep_nested_wrap_three_skippable_layers() {
    let ty: Type = parse_quote!(Vec<Option<Vec<Leaf>>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ts(&wrapped),
        "Vec < Option < Vec < adze :: WithLeaf < Leaf > > > >"
    );
}

#[test]
fn deep_nested_option_option_option() {
    let ty: Type = parse_quote!(Option<Option<Option<bool>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ts(&inner), "Option < Option < bool > >");
}

#[test]
fn deep_nested_extract_preserves_remaining_nesting() {
    let ty: Type = parse_quote!(Option<Box<Vec<HashMap<String, i32>>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ts(&inner), "Box < Vec < HashMap < String , i32 > > >");
}

// ===========================================================================
// 7. Edge cases — multiple type parameters
// ===========================================================================

#[test]
fn multi_param_result_extract_first_arg() {
    // Result<T, E> has two type params; extract targets the first
    let ty: Type = parse_quote!(Result<String, Error>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Result", &skip(&[]));
    assert!(extracted);
    // Extracts first generic argument
    assert_eq!(ts(&inner), "String");
}

#[test]
fn multi_param_hashmap_not_in_skip() {
    let ty: Type = parse_quote!(HashMap<String, Vec<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "HashMap < String , Vec < i32 > >");
}

#[test]
fn multi_param_wrap_preserves_all_type_args() {
    let ty: Type = parse_quote!(Result<A, B>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        ts(&wrapped),
        "Result < adze :: WithLeaf < A > , adze :: WithLeaf < B > >"
    );
}

#[test]
fn multi_param_wrap_three_generic_args() {
    // Synthetic type with 3 generic args in skip set
    let ty: Type = parse_quote!(Triple<A, B, C>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Triple"]));
    assert_eq!(
        ts(&wrapped),
        "Triple < adze :: WithLeaf < A > , adze :: WithLeaf < B > , adze :: WithLeaf < C > >"
    );
}

// ===========================================================================
// 8. Composition — chaining extract, filter, and wrap
// ===========================================================================

#[test]
fn compose_extract_then_filter() {
    let ty: Type = parse_quote!(Option<Box<Expr>>);
    let (after_opt, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ts(&after_opt), "Box < Expr >");
    let filtered = filter_inner_type(&after_opt, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "Expr");
}

#[test]
fn compose_extract_then_wrap() {
    let ty: Type = parse_quote!(Option<Token>);
    let (after_opt, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    let wrapped = wrap_leaf_type(&after_opt, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < Token >");
}

#[test]
fn compose_filter_then_wrap() {
    let ty: Type = parse_quote!(Box<Vec<Leaf>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "Vec < Leaf >");
    let wrapped = wrap_leaf_type(&filtered, &skip(&["Vec"]));
    assert_eq!(ts(&wrapped), "Vec < adze :: WithLeaf < Leaf > >");
}

#[test]
fn compose_extract_filter_wrap_full_pipeline() {
    let ty: Type = parse_quote!(Option<Box<Arc<Vec<Node>>>>);
    let (after_opt, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Box", "Arc"]));
    assert!(extracted);
    // Skipped Box and Arc, extracted Option → Vec<Node>
    // Actually Box<Arc<Vec<Node>>> is extracted from Option, then skip through Box and Arc
    // Wait: Option is the target. Box and Arc are in skip_over but Option is at top-level,
    // so it matches directly and returns Box<Arc<Vec<Node>>>
    // Actually, re-reading the code: the target is "Option", ty is Option<Box<Arc<Vec<Node>>>>
    // The last segment is "Option", which matches inner_of, so it extracts the first generic arg.
    assert_eq!(ts(&after_opt), "Box < Arc < Vec < Node > > >");

    let filtered = filter_inner_type(&after_opt, &skip(&["Box", "Arc"]));
    assert_eq!(ts(&filtered), "Vec < Node >");

    let wrapped = wrap_leaf_type(&filtered, &skip(&["Vec"]));
    assert_eq!(ts(&wrapped), "Vec < adze :: WithLeaf < Node > >");
}

// ===========================================================================
// 9. NameValueExpr list parsing
// ===========================================================================

#[test]
fn nve_parsed_from_str() {
    let nve: NameValueExpr = syn::parse_str("key = 100").unwrap();
    assert_eq!(nve.path.to_string(), "key");
}

#[test]
fn nve_list_from_str() {
    use syn::parse::Parser;
    use syn::punctuated::Punctuated;

    let parser = Punctuated::<NameValueExpr, syn::Token![,]>::parse_terminated;
    let list = parser.parse_str("a = 1, b = 2, c = 3").unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list[0].path.to_string(), "a");
    assert_eq!(list[1].path.to_string(), "b");
    assert_eq!(list[2].path.to_string(), "c");
}

#[test]
fn nve_single_item_list() {
    use syn::parse::Parser;
    use syn::punctuated::Punctuated;

    let parser = Punctuated::<NameValueExpr, syn::Token![,]>::parse_terminated;
    let list = parser.parse_str("only = true").unwrap();
    assert_eq!(list.len(), 1);
}

#[test]
fn nve_empty_list() {
    use syn::parse::Parser;
    use syn::punctuated::Punctuated;

    let parser = Punctuated::<NameValueExpr, syn::Token![,]>::parse_terminated;
    let list = parser.parse_str("").unwrap();
    assert_eq!(list.len(), 0);
}

// ===========================================================================
// 10. FieldThenParams — edge cases
// ===========================================================================

#[test]
fn ftp_type_with_lifetime() {
    let ftp: FieldThenParams = parse_quote!(Cow<'static, str>);
    let type_str = ts(&ftp.field.ty);
    assert!(type_str.contains("Cow"));
    assert!(type_str.contains("str"));
}

#[test]
fn ftp_fn_pointer_type() {
    let ftp: FieldThenParams = parse_quote!(fn(i32) -> bool);
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_never_type() {
    let ftp: FieldThenParams = parse_quote!(!);
    assert_eq!(ts(&ftp.field.ty), "!");
}

// ===========================================================================
// 11. Idempotency and identity
// ===========================================================================

#[test]
fn filter_idempotent() {
    let s = skip(&["Box"]);
    let ty: Type = parse_quote!(Box<Inner>);
    let once = filter_inner_type(&ty, &s);
    let twice = filter_inner_type(&once, &s);
    assert_eq!(ts(&once), "Inner");
    assert_eq!(ts(&twice), "Inner");
}

#[test]
fn filter_already_unwrapped_is_identity() {
    let ty: Type = parse_quote!(Leaf);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ts(&filtered), "Leaf");
}

#[test]
fn extract_already_extracted_no_double_extraction() {
    let ty: Type = parse_quote!(Option<Inner>);
    let (first, ex1) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ex1);
    assert_eq!(ts(&first), "Inner");
    let (second, ex2) = try_extract_inner_type(&first, "Option", &skip(&[]));
    assert!(!ex2);
    assert_eq!(ts(&second), "Inner");
}

// ===========================================================================
// 12. Qualified / path types
// ===========================================================================

#[test]
fn extract_qualified_std_option() {
    // std::option::Option — last segment is "Option"
    let ty: Type = parse_quote!(std::option::Option<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ts(&inner), "i32");
}

#[test]
fn filter_qualified_std_boxed_box() {
    let ty: Type = parse_quote!(std::boxed::Box<Val>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "Val");
}

#[test]
fn wrap_qualified_path() {
    let ty: Type = parse_quote!(std::vec::Vec<Item>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        ts(&wrapped),
        "std :: vec :: Vec < adze :: WithLeaf < Item > >"
    );
}

// ===========================================================================
// 13. Bulk operations — process many types at once
// ===========================================================================

#[test]
fn bulk_wrap_30_types() {
    let s = skip(&[]);
    let types: Vec<Type> = (0..30)
        .map(|i| {
            let name = syn::Ident::new(&format!("T{i}"), proc_macro2::Span::call_site());
            parse_quote!(#name)
        })
        .collect();

    for (i, ty) in types.iter().enumerate() {
        let wrapped = wrap_leaf_type(ty, &s);
        assert_eq!(ts(&wrapped), format!("adze :: WithLeaf < T{i} >"));
    }
}

#[test]
fn bulk_extract_mixed_option_and_plain() {
    let s = skip(&[]);
    let types: Vec<Type> = (0..20)
        .map(|i| {
            let name = syn::Ident::new(&format!("X{i}"), proc_macro2::Span::call_site());
            if i % 2 == 0 {
                parse_quote!(Option<#name>)
            } else {
                parse_quote!(#name)
            }
        })
        .collect();

    let extracted_count = types
        .iter()
        .filter(|ty| try_extract_inner_type(ty, "Option", &s).1)
        .count();
    assert_eq!(extracted_count, 10);
}

#[test]
fn bulk_filter_alternating_box_and_plain() {
    let s = skip(&["Box"]);
    let types: Vec<Type> = (0..16)
        .map(|i| {
            let name = syn::Ident::new(&format!("N{i}"), proc_macro2::Span::call_site());
            if i % 2 == 0 {
                parse_quote!(Box<#name>)
            } else {
                parse_quote!(#name)
            }
        })
        .collect();

    for (i, ty) in types.iter().enumerate() {
        let filtered = filter_inner_type(ty, &s);
        assert_eq!(ts(&filtered), format!("N{i}"));
    }
}

// ===========================================================================
// 14. FieldThenParams param values
// ===========================================================================

#[test]
fn ftp_param_value_is_integer() {
    let ftp: FieldThenParams = parse_quote!(Tok, precedence = 10);
    if let syn::Expr::Lit(lit) = &ftp.params[0].expr {
        if let syn::Lit::Int(i) = &lit.lit {
            assert_eq!(i.base10_parse::<i32>().unwrap(), 10);
        } else {
            panic!("expected int");
        }
    } else {
        panic!("expected lit");
    }
}

#[test]
fn ftp_param_value_is_bool() {
    let ftp: FieldThenParams = parse_quote!(Tok, optional = true);
    assert_eq!(ftp.params[0].path.to_string(), "optional");
}

#[test]
fn ftp_param_value_is_path() {
    let ftp: FieldThenParams = parse_quote!(Tok, handler = my_handler);
    assert_eq!(ftp.params[0].path.to_string(), "handler");
}

// ===========================================================================
// 15. Wrap with nested skip types
// ===========================================================================

#[test]
fn wrap_vec_of_vec_both_skipped() {
    let ty: Type = parse_quote!(Vec<Vec<Leaf>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ts(&wrapped), "Vec < Vec < adze :: WithLeaf < Leaf > > >");
}

#[test]
fn wrap_option_of_option_both_skipped() {
    let ty: Type = parse_quote!(Option<Option<Leaf>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(
        ts(&wrapped),
        "Option < Option < adze :: WithLeaf < Leaf > > >"
    );
}

#[test]
fn wrap_box_not_in_skip_wraps_entire() {
    let ty: Type = parse_quote!(Box<Leaf>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < Box < Leaf > >");
}

// ===========================================================================
// 16. try_extract_inner_type — skip_over set interactions
// ===========================================================================

#[test]
fn extract_skip_set_with_many_entries() {
    let ty: Type = parse_quote!(Rc<Vec<u8>>);
    let s = skip(&["Box", "Arc", "Rc", "Pin", "Cow"]);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &s);
    assert!(extracted);
    assert_eq!(ts(&inner), "u8");
}

#[test]
fn extract_target_same_as_skip_entry() {
    // If target and skip_over overlap, target takes precedence (checked first)
    let ty: Type = parse_quote!(Box<i32>);
    let s = skip(&["Box"]);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &s);
    assert!(extracted);
    assert_eq!(ts(&inner), "i32");
}

#[test]
fn extract_unrelated_type_with_full_skip_set() {
    let ty: Type = parse_quote!(MyCustomType);
    let s = skip(&["Box", "Arc", "Rc"]);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &s);
    assert!(!extracted);
    assert_eq!(ts(&inner), "MyCustomType");
}

// ===========================================================================
// 17. NameValueExpr — Debug and Clone
// ===========================================================================

#[test]
fn nve_debug_impl() {
    let nve: NameValueExpr = parse_quote!(key = "val");
    let debug = format!("{:?}", nve);
    assert!(debug.contains("NameValueExpr"));
}

#[test]
fn nve_clone_independence() {
    let nve: NameValueExpr = parse_quote!(key = "val");
    let cloned = nve.clone();
    assert_eq!(nve.path.to_string(), cloned.path.to_string());
}

// ===========================================================================
// 18. FieldThenParams — Debug and Clone
// ===========================================================================

#[test]
fn ftp_debug_impl() {
    let ftp: FieldThenParams = parse_quote!(MyType);
    let debug = format!("{:?}", ftp);
    assert!(debug.contains("FieldThenParams"));
}

#[test]
fn ftp_clone_preserves_type() {
    let ftp: FieldThenParams = parse_quote!(Vec<i32>, key = "v");
    let cloned = ftp.clone();
    assert_eq!(ts(&ftp.field.ty), ts(&cloned.field.ty));
    assert_eq!(ftp.params.len(), cloned.params.len());
}

// ===========================================================================
// 19. Type stability — token stream roundtrip
// ===========================================================================

#[test]
fn roundtrip_filter_preserves_non_skip_type() {
    let ty: Type = parse_quote!(Vec<HashMap<String, Vec<i32>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&ty), ts(&filtered));
}

#[test]
fn roundtrip_extract_non_matching_preserves() {
    let ty: Type = parse_quote!(BTreeMap<String, Vec<Option<bool>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ts(&inner), ts(&ty));
}
