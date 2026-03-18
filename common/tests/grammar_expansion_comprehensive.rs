#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for grammar expansion logic in adze-common.
//!
//! Covers: basic expansion, type annotation processing, optional/repetition/choice
//! expansion, nested types, edge cases, error cases, field mapping, multi-rule
//! interaction patterns, and property-based testing with proptest.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proptest::prelude::*;
use quote::ToTokens;
use syn::{Type, parse_quote};

fn skip_set<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn type_to_string(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1. Basic grammar expansion — core extraction behaviour
// ===========================================================================

#[test]
fn basic_extract_option_returns_inner_type() {
    let ty: Type = parse_quote!(Option<Identifier>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
    assert!(ok);
    assert_eq!(type_to_string(&inner), "Identifier");
}

#[test]
fn basic_extract_vec_returns_element_type() {
    let ty: Type = parse_quote!(Vec<Statement>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&[]));
    assert!(ok);
    assert_eq!(type_to_string(&inner), "Statement");
}

#[test]
fn basic_filter_strips_single_wrapper() {
    let ty: Type = parse_quote!(Box<Expression>);
    let filtered = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(type_to_string(&filtered), "Expression");
}

#[test]
fn basic_wrap_adds_with_leaf() {
    let ty: Type = parse_quote!(Token);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < Token >");
}

// ===========================================================================
// 2. Type annotation processing — NameValueExpr parsing variants
// ===========================================================================

#[test]
fn annotation_name_value_tuple_expr() {
    let nv: NameValueExpr = parse_quote!(range = (0, 100));
    assert_eq!(nv.path.to_string(), "range");
    assert!(matches!(nv.expr, syn::Expr::Tuple(_)));
}

#[test]
fn annotation_name_value_field_access() {
    let nv: NameValueExpr = parse_quote!(source = config.grammar_path);
    assert_eq!(nv.path.to_string(), "source");
    assert!(matches!(nv.expr, syn::Expr::Field(_)));
}

#[test]
fn annotation_field_then_params_multiple_closures() {
    let parsed: FieldThenParams = parse_quote!(
        u64,
        transform = |s: String| s.parse::<u64>().unwrap(),
        validate = |v: u64| v > 0
    );
    assert_eq!(parsed.params.len(), 2);
    assert!(matches!(parsed.params[0].expr, syn::Expr::Closure(_)));
    assert!(matches!(parsed.params[1].expr, syn::Expr::Closure(_)));
}

// ===========================================================================
// 3. Optional types expansion — Option<T> through various wrappers
// ===========================================================================

#[test]
fn optional_direct_option_with_complex_inner() {
    let ty: Type = parse_quote!(Option<Vec<Box<ASTNode>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
    assert!(ok);
    assert_eq!(type_to_string(&inner), "Vec < Box < ASTNode > >");
}

#[test]
fn optional_nested_option_extracts_outermost_only() {
    // Option<Option<T>> — extraction should yield the outer Option's arg
    let ty: Type = parse_quote!(Option<Option<Leaf>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
    assert!(ok);
    assert_eq!(type_to_string(&inner), "Option < Leaf >");
}

#[test]
fn optional_through_rc_and_box_skip() {
    let ty: Type = parse_quote!(Rc<Box<Option<FnDecl>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&["Rc", "Box"]));
    assert!(ok);
    assert_eq!(type_to_string(&inner), "FnDecl");
}

// ===========================================================================
// 4. Repetition types expansion — Vec<T> patterns
// ===========================================================================

#[test]
fn repetition_vec_through_arc_skip() {
    let ty: Type = parse_quote!(Arc<Vec<Parameter>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&["Arc"]));
    assert!(ok);
    assert_eq!(type_to_string(&inner), "Parameter");
}

#[test]
fn repetition_vec_preserves_nested_generics() {
    let ty: Type = parse_quote!(Vec<Option<Box<Stmt>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&[]));
    assert!(ok);
    assert_eq!(type_to_string(&inner), "Option < Box < Stmt > >");
}

#[test]
fn repetition_vec_not_found_returns_original() {
    let ty: Type = parse_quote!(HashMap<String, Vec<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&[]));
    assert!(!ok);
    assert_eq!(type_to_string(&inner), "HashMap < String , Vec < i32 > >");
}

// ===========================================================================
// 5. Choice/enum types expansion — Result-like and custom generics
// ===========================================================================

#[test]
fn choice_extract_from_either_type() {
    let ty: Type = parse_quote!(Either<LHS, RHS>);
    let (inner, ok) = try_extract_inner_type(&ty, "Either", &skip_set(&[]));
    assert!(ok);
    // Extracts only the first generic argument
    assert_eq!(type_to_string(&inner), "LHS");
}

#[test]
fn choice_wrap_preserves_multi_arg_skip_type() {
    let ty: Type = parse_quote!(Either<Alpha, Beta>);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&["Either"]));
    assert_eq!(
        type_to_string(&wrapped),
        "Either < adze :: WithLeaf < Alpha > , adze :: WithLeaf < Beta > >"
    );
}

// ===========================================================================
// 6. Nested type expansion — deeply nested and mixed containers
// ===========================================================================

#[test]
fn nested_four_deep_filter() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Cell<Payload>>>>);
    let filtered = filter_inner_type(&ty, &skip_set(&["Box", "Arc", "Rc", "Cell"]));
    assert_eq!(type_to_string(&filtered), "Payload");
}

#[test]
fn nested_wrap_with_interleaved_skip_and_non_skip() {
    // Vec is skip, HashMap is not — wrapping stops at HashMap
    let ty: Type = parse_quote!(Vec<HashMap<String, i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&["Vec"]));
    assert_eq!(
        type_to_string(&wrapped),
        "Vec < adze :: WithLeaf < HashMap < String , i32 > > >"
    );
}

#[test]
fn nested_extract_then_filter_then_wrap_pipeline() {
    // Full pipeline: Option<Box<Arc<Vec<Leaf>>>>
    let ty: Type = parse_quote!(Option<Box<Arc<Vec<Leaf>>>>);

    // Step 1: extract Option (it's the outermost, so direct match)
    let (after_opt, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
    assert!(ok);
    assert_eq!(type_to_string(&after_opt), "Box < Arc < Vec < Leaf > > >");

    // Step 2: filter Box, Arc
    let filtered = filter_inner_type(&after_opt, &skip_set(&["Box", "Arc"]));
    assert_eq!(type_to_string(&filtered), "Vec < Leaf >");

    // Step 3: wrap with Vec in skip
    let wrapped = wrap_leaf_type(&filtered, &skip_set(&["Vec"]));
    assert_eq!(
        type_to_string(&wrapped),
        "Vec < adze :: WithLeaf < Leaf > >"
    );
}

// ===========================================================================
// 7. Edge cases — empty types, unusual syntax, recursive-like types
// ===========================================================================

#[test]
fn edge_raw_pointer_type_wrap() {
    let ty: Type = parse_quote!(*const u8);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert!(type_to_string(&wrapped).contains("adze :: WithLeaf"));
}

#[test]
fn edge_impl_trait_type_wrap() {
    let ty: Type = parse_quote!(impl Iterator<Item = u8>);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert!(type_to_string(&wrapped).starts_with("adze :: WithLeaf"));
}

#[test]
fn edge_self_referential_name_extracts_inner() {
    // Type named same as target — should still extract inner
    let ty: Type = parse_quote!(Node<Node<Leaf>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Node", &skip_set(&[]));
    assert!(ok);
    assert_eq!(type_to_string(&inner), "Node < Leaf >");
}

#[test]
fn edge_empty_skip_set_filter_preserves_all() {
    let ty: Type = parse_quote!(Box<Arc<Vec<Option<i32>>>>);
    let filtered = filter_inner_type(&ty, &skip_set(&[]));
    assert_eq!(
        type_to_string(&filtered),
        "Box < Arc < Vec < Option < i32 > > > >"
    );
}

// ===========================================================================
// 8. Error cases — panics on malformed input
// ===========================================================================

#[test]
fn error_filter_skip_type_without_generics_preserves_plain_type() {
    // A bare type named "Box" is not necessarily a generic wrapper.
    let ty: Type = parse_quote!(Box);
    let filtered = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(type_to_string(&filtered), "Box");
}

#[test]
fn error_extract_skip_type_without_generics_preserves_plain_type() {
    let ty: Type = parse_quote!(Arc);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&["Arc"]));
    assert!(!ok);
    assert_eq!(type_to_string(&inner), "Arc");
}

#[test]
fn error_wrap_skip_type_without_generics_preserves_plain_type() {
    let ty: Type = parse_quote!(Vec);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&["Vec"]));
    assert_eq!(type_to_string(&wrapped), "Vec");
}

// ===========================================================================
// 9. Field mapping — FieldThenParams extracting field type info
// ===========================================================================

#[test]
fn field_mapping_option_field_extract_inner() {
    let parsed: FieldThenParams = parse_quote!(Option<ReturnType>, default = "void");
    let field_ty = &parsed.field.ty;
    let (inner, ok) = try_extract_inner_type(field_ty, "Option", &skip_set(&[]));
    assert!(ok);
    assert_eq!(type_to_string(&inner), "ReturnType");
    assert_eq!(parsed.params[0].path.to_string(), "default");
}

#[test]
fn field_mapping_vec_field_with_separator() {
    let parsed: FieldThenParams = parse_quote!(Vec<Argument>, separator = ",");
    let field_ty = &parsed.field.ty;
    let (inner, ok) = try_extract_inner_type(field_ty, "Vec", &skip_set(&[]));
    assert!(ok);
    assert_eq!(type_to_string(&inner), "Argument");
    if let syn::Expr::Lit(lit) = &parsed.params[0].expr
        && let syn::Lit::Str(s) = &lit.lit
    {
        assert_eq!(s.value(), ",");
    } else {
        panic!("Expected string literal separator");
    }
}

#[test]
fn field_mapping_box_field_filter_then_wrap() {
    let parsed: FieldThenParams = parse_quote!(Box<Expression>, precedence = 5);
    let field_ty = &parsed.field.ty;
    let filtered = filter_inner_type(field_ty, &skip_set(&["Box"]));
    assert_eq!(type_to_string(&filtered), "Expression");
    let wrapped = wrap_leaf_type(&filtered, &skip_set(&[]));
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < Expression >");
}

// ===========================================================================
// 10. Multiple rules interaction — batch processing simulating grammar structs
// ===========================================================================

#[test]
fn multi_rule_batch_optional_and_required_fields() {
    // Simulate a grammar struct with mixed required and optional fields
    let fields: Vec<Type> = vec![
        parse_quote!(Keyword),
        parse_quote!(Option<Modifier>),
        parse_quote!(Vec<Param>),
        parse_quote!(Option<Vec<Annotation>>),
        parse_quote!(Box<Body>),
    ];

    let extract_skip = skip_set(&["Box"]);
    let wrap_skip = skip_set(&["Vec", "Option"]);

    let mut optionality = Vec::new();
    let mut wrapped_types = Vec::new();

    for i in 0..fields.len() {
        let (after, is_opt) = try_extract_inner_type(&fields[i], "Option", &extract_skip);
        optionality.push(is_opt);
        let to_wrap = if is_opt { after } else { fields[i].clone() };
        wrapped_types.push(type_to_string(&wrap_leaf_type(&to_wrap, &wrap_skip)));
    }

    // Keyword — not optional
    assert!(!optionality[0]);
    assert_eq!(wrapped_types[0], "adze :: WithLeaf < Keyword >");

    // Option<Modifier> — optional
    assert!(optionality[1]);
    assert_eq!(wrapped_types[1], "adze :: WithLeaf < Modifier >");

    // Vec<Param> — not optional (Vec is a repetition, not Option)
    assert!(!optionality[2]);
    assert_eq!(wrapped_types[2], "Vec < adze :: WithLeaf < Param > >");

    // Option<Vec<Annotation>> — optional
    assert!(optionality[3]);
    assert_eq!(wrapped_types[3], "Vec < adze :: WithLeaf < Annotation > >");

    // Box<Body> — not optional (Box not in skip_set for extract)
    assert!(!optionality[4]);
    assert_eq!(wrapped_types[4], "adze :: WithLeaf < Box < Body > >");
}

#[test]
fn multi_rule_filter_and_wrap_consistent_across_similar_types() {
    // Ensure filter+wrap produces identical results for equivalent type structures
    let types: Vec<Type> = vec![parse_quote!(Box<Leaf>), parse_quote!(std::boxed::Box<Leaf>)];
    let filter_skip = skip_set(&["Box"]);
    let wrap_skip = skip_set(&[]);

    let results: Vec<String> = types
        .iter()
        .map(|ty| {
            let filtered = filter_inner_type(ty, &filter_skip);
            type_to_string(&wrap_leaf_type(&filtered, &wrap_skip))
        })
        .collect();

    // Both should resolve to the same wrapped leaf type
    assert_eq!(results[0], results[1]);
    assert_eq!(results[0], "adze :: WithLeaf < Leaf >");
}

#[test]
fn multi_rule_extract_vec_then_option_sequentially() {
    // Extracting Vec first, then Option from the inner type
    let ty: Type = parse_quote!(Vec<Option<Token>>);

    let (after_vec, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&[]));
    assert!(ok);
    assert_eq!(type_to_string(&after_vec), "Option < Token >");

    let (after_opt, ok) = try_extract_inner_type(&after_vec, "Option", &skip_set(&[]));
    assert!(ok);
    assert_eq!(type_to_string(&after_opt), "Token");
}

// ===========================================================================
// 11. Type Extraction from Reference Types (&T, &mut T)
// ===========================================================================

#[test]
fn reference_type_immutable_extract_returns_unchanged() {
    let ty: Type = parse_quote!(&String);
    let (inner, extracted) = try_extract_inner_type(&ty, "String", &skip_set(&[]));
    assert!(!extracted);
    assert_eq!(type_to_string(&inner), "& String");
}

#[test]
fn reference_type_mutable_extract_returns_unchanged() {
    let ty: Type = parse_quote!(&mut String);
    let (inner, extracted) = try_extract_inner_type(&ty, "String", &skip_set(&[]));
    assert!(!extracted);
    assert_eq!(type_to_string(&inner), "& mut String");
}

#[test]
fn reference_type_filter_returns_unchanged() {
    let ty: Type = parse_quote!(&String);
    let filtered = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(type_to_string(&filtered), "& String");
}

#[test]
fn reference_type_wrap_wraps_entirely() {
    let ty: Type = parse_quote!(&String);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < & String >");
}

#[test]
fn mutable_reference_type_wrap_wraps_entirely() {
    let ty: Type = parse_quote!(&mut String);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(
        type_to_string(&wrapped),
        "adze :: WithLeaf < & mut String >"
    );
}

// ===========================================================================
// 12. Tuple Type Handling
// ===========================================================================

#[test]
fn tuple_type_extract_returns_unchanged() {
    let ty: Type = parse_quote!((String, i32));
    let (inner, extracted) = try_extract_inner_type(&ty, "String", &skip_set(&[]));
    assert!(!extracted);
    assert_eq!(type_to_string(&inner), "(String , i32)");
}

#[test]
fn tuple_type_filter_returns_unchanged() {
    let ty: Type = parse_quote!((String, i32));
    let filtered = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(type_to_string(&filtered), "(String , i32)");
}

#[test]
fn tuple_type_wrap_wraps_entirely() {
    let ty: Type = parse_quote!((String, i32));
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(
        type_to_string(&wrapped),
        "adze :: WithLeaf < (String , i32) >"
    );
}

#[test]
fn complex_tuple_with_containers_wrap() {
    let ty: Type = parse_quote!((Vec<String>, Option<i32>));
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert!(type_to_string(&wrapped).contains("adze :: WithLeaf"));
}

// ===========================================================================
// 13. Array Type Handling
// ===========================================================================

#[test]
fn array_type_extract_returns_unchanged() {
    let ty: Type = parse_quote!([u8; 4]);
    let (inner, extracted) = try_extract_inner_type(&ty, "u8", &skip_set(&[]));
    assert!(!extracted);
    assert_eq!(type_to_string(&inner), "[u8 ; 4]");
}

#[test]
fn array_type_filter_returns_unchanged() {
    let ty: Type = parse_quote!([u8; 4]);
    let filtered = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(type_to_string(&filtered), "[u8 ; 4]");
}

#[test]
fn array_type_wrap_wraps_entirely() {
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn array_type_with_generic_element_wrap() {
    let ty: Type = parse_quote!([T; N]);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < [T ; N] >");
}

// ===========================================================================
// 14. Types with Lifetimes
// ===========================================================================

#[test]
fn reference_with_lifetime_extract_returns_unchanged() {
    let ty: Type = parse_quote!(&'a String);
    let (inner, extracted) = try_extract_inner_type(&ty, "String", &skip_set(&[]));
    assert!(!extracted);
    assert_eq!(type_to_string(&inner), "& 'a String");
}

#[test]
fn reference_with_lifetime_wrap_wraps_entirely() {
    let ty: Type = parse_quote!(&'a String);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < & 'a String >");
}

#[test]
fn generic_type_param_basic() {
    let ty: Type = parse_quote!(T);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < T >");
}

// ===========================================================================
// 15. Qualified Paths (std::vec::Vec<T>, std::option::Option<T>)
// ===========================================================================

#[test]
fn qualified_vec_extract_inner() {
    let ty: Type = parse_quote!(::std::vec::Vec<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_set(&[]));
    assert!(extracted);
    assert_eq!(type_to_string(&inner), "String");
}

#[test]
fn qualified_option_extract_inner() {
    let ty: Type = parse_quote!(std::option::Option<i64>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
    assert!(extracted);
    assert_eq!(type_to_string(&inner), "i64");
}

#[test]
fn qualified_box_filter_inner() {
    let ty: Type = parse_quote!(std::boxed::Box<String>);
    let filtered = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(type_to_string(&filtered), "String");
}

#[test]
fn qualified_vec_wrap_with_skip() {
    let ty: Type = parse_quote!(::std::vec::Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&["Vec"]));
    assert_eq!(
        type_to_string(&wrapped),
        ":: std :: vec :: Vec < adze :: WithLeaf < String > >"
    );
}

// ===========================================================================
// 16. Complex Generic Types (Multiple Type Parameters)
// ===========================================================================

#[test]
fn hashmap_like_extract_first_param() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "HashMap", &skip_set(&[]));
    assert!(extracted);
    assert_eq!(type_to_string(&inner), "String");
}

#[test]
fn result_type_wrap_both_params() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&["Result"]));
    assert_eq!(
        type_to_string(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn custom_generic_extract_inner() {
    let ty: Type = parse_quote!(CustomType<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "CustomType", &skip_set(&[]));
    assert!(extracted);
    assert_eq!(type_to_string(&inner), "String");
}

#[test]
fn triple_generic_param_wrap_all_params() {
    let ty: Type = parse_quote!(Triple<A, B, C>);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&["Triple"]));
    let wrapped_str = type_to_string(&wrapped);
    assert!(wrapped_str.contains("adze :: WithLeaf < A >"));
    assert!(wrapped_str.contains("adze :: WithLeaf < B >"));
    assert!(wrapped_str.contains("adze :: WithLeaf < C >"));
}

// ===========================================================================
// 17. Comprehensive Pipeline Integration Tests
// ===========================================================================

#[test]
fn pipeline_box_vec_extract_filter_wrap() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let skip_extract = skip_set(&["Box"]);
    let skip_filter = skip_set(&[]);
    let skip_wrap = skip_set(&["Vec"]);

    // Extract Vec inner type (String) through Box
    let (extracted, ok) = try_extract_inner_type(&ty, "Vec", &skip_extract);
    assert!(ok);
    assert_eq!(type_to_string(&extracted), "String");

    let filtered = filter_inner_type(&extracted, &skip_filter);
    let wrapped = wrap_leaf_type(&filtered, &skip_wrap);

    // Result is just the wrapped leaf type since we extracted the Vec element
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn pipeline_option_vec_option_complex_nesting() {
    let ty: Type = parse_quote!(Option<Vec<Option<String>>>);
    let skip_extract = skip_set(&["Option"]);
    let skip_wrap = skip_set(&["Vec", "Option"]);

    // Extract Vec inner type through Option
    let (extracted, ok) = try_extract_inner_type(&ty, "Vec", &skip_extract);
    assert!(ok);
    // When we extract Vec from Option<Vec<Option<String>>>, we get Option<String>
    assert_eq!(type_to_string(&extracted), "Option < String >");

    let wrapped = wrap_leaf_type(&extracted, &skip_wrap);
    // When wrapping with Option in skip set, we only wrap the String leaf
    assert_eq!(
        type_to_string(&wrapped),
        "Option < adze :: WithLeaf < String > >"
    );
}

#[test]
fn pipeline_deeply_nested_five_levels() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Option<Vec<String>>>>>);
    let skip_extract = skip_set(&["Box", "Arc", "Rc", "Option"]);
    let skip_filter = skip_set(&["Box", "Arc", "Rc", "Option"]);
    let skip_wrap = skip_set(&["Vec"]);

    // Extract Vec through all wrappers
    let (extracted, ok) = try_extract_inner_type(&ty, "Vec", &skip_extract);
    assert!(ok);
    assert_eq!(type_to_string(&extracted), "String");

    // Filter all wrappers
    let filtered = filter_inner_type(&ty, &skip_filter);
    assert_eq!(type_to_string(&filtered), "Vec < String >");

    // Wrap result
    let wrapped = wrap_leaf_type(&filtered, &skip_wrap);
    assert_eq!(
        type_to_string(&wrapped),
        "Vec < adze :: WithLeaf < String > >"
    );
}

#[test]
fn pipeline_result_vec_result_multi_type_param() {
    let ty: Type = parse_quote!(Result<Vec<String>, Box<Error>>);
    let skip_wrap = skip_set(&["Vec", "Result"]);

    // Wrap both type parameters of Result
    let wrapped = wrap_leaf_type(&ty, &skip_wrap);
    let wrapped_str = type_to_string(&wrapped);

    assert!(wrapped_str.contains("Vec < adze :: WithLeaf < String > >"));
    assert!(wrapped_str.contains("Box < Error >"));
}

#[test]
fn pipeline_extract_nonexistent_target_returns_original() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let skip_extract = skip_set(&["Box"]);

    // Try to extract Option which doesn't exist
    let (extracted, ok) = try_extract_inner_type(&ty, "Option", &skip_extract);
    assert!(!ok);
    assert_eq!(type_to_string(&extracted), "Box < Vec < String > >");
}

#[test]
fn pipeline_sequential_filtering_stages() {
    let ty: Type = parse_quote!(Box<Arc<Option<Vec<String>>>>);

    // First filter stage: remove Box
    let stage1 = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(type_to_string(&stage1), "Arc < Option < Vec < String > > >");

    // Second filter stage: remove Arc
    let stage2 = filter_inner_type(&stage1, &skip_set(&["Arc"]));
    assert_eq!(type_to_string(&stage2), "Option < Vec < String > >");

    // Third filter stage: remove Option
    let stage3 = filter_inner_type(&stage2, &skip_set(&["Option"]));
    assert_eq!(type_to_string(&stage3), "Vec < String >");
}

#[test]
fn pipeline_comprehensive_transform_scenario() {
    // Real-world scenario: transform Option<Box<Vec<Identifier>>> for grammar processing
    let ty: Type = parse_quote!(Option<Box<Vec<Identifier>>>);

    // Check optionality
    let (without_option, is_optional) = try_extract_inner_type(&ty, "Option", &skip_set(&["Box"]));
    assert!(is_optional);

    // Remove Box wrapper
    let without_box = filter_inner_type(&without_option, &skip_set(&["Box"]));

    // Extract Vec element type
    let (element_type, has_vec) = try_extract_inner_type(&without_box, "Vec", &skip_set(&[]));
    assert!(has_vec);
    assert_eq!(type_to_string(&element_type), "Identifier");

    // Wrap the final type
    let final_type = wrap_leaf_type(&element_type, &skip_set(&[]));
    assert_eq!(
        type_to_string(&final_type),
        "adze :: WithLeaf < Identifier >"
    );
}

// ===========================================================================
// 18. Token pattern handling — leaf vs non-leaf token types
// ===========================================================================

#[test]
fn token_pattern_primitive_types_wrap_as_leaves() {
    let primitives: Vec<Type> = vec![
        parse_quote!(i8),
        parse_quote!(i16),
        parse_quote!(i32),
        parse_quote!(i64),
        parse_quote!(u8),
        parse_quote!(u16),
        parse_quote!(u32),
        parse_quote!(u64),
        parse_quote!(f32),
        parse_quote!(f64),
        parse_quote!(bool),
        parse_quote!(char),
        parse_quote!(usize),
        parse_quote!(isize),
    ];
    let skip = skip_set(&[]);
    for ty in &primitives {
        let wrapped = wrap_leaf_type(ty, &skip);
        let s = type_to_string(&wrapped);
        assert!(
            s.starts_with("adze :: WithLeaf <"),
            "Primitive {} should be wrapped as leaf, got: {}",
            type_to_string(ty),
            s
        );
    }
}

#[test]
fn token_pattern_string_type_is_leaf() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn token_pattern_unit_type_wraps() {
    let ty: Type = parse_quote!(());
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < () >");
}

#[test]
fn token_pattern_never_type_wraps() {
    let ty: Type = parse_quote!(!);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert!(type_to_string(&wrapped).contains("adze :: WithLeaf"));
}

// ===========================================================================
// 19. Rule composition — combining extraction/filter/wrap in grammar patterns
// ===========================================================================

#[test]
fn rule_composition_idempotent_double_filter() {
    let ty: Type = parse_quote!(Box<String>);
    let skip = skip_set(&["Box"]);
    let once = filter_inner_type(&ty, &skip);
    let twice = filter_inner_type(&once, &skip);
    assert_eq!(type_to_string(&once), type_to_string(&twice));
}

#[test]
fn rule_composition_extract_then_wrap_roundtrip_type_name() {
    let ty: Type = parse_quote!(Vec<Expr>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip_set(&[]));
    assert!(type_to_string(&wrapped).contains("Expr"));
}

#[test]
fn rule_composition_multiple_skip_types_filter_chain() {
    let ty: Type = parse_quote!(Arc<Rc<Leaf>>);
    let filtered = filter_inner_type(&ty, &skip_set(&["Arc", "Rc"]));
    assert_eq!(type_to_string(&filtered), "Leaf");
}

#[test]
fn rule_composition_wrap_preserves_option_vec_nesting() {
    let ty: Type = parse_quote!(Option<Vec<Leaf>>);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&["Option", "Vec"]));
    assert_eq!(
        type_to_string(&wrapped),
        "Option < Vec < adze :: WithLeaf < Leaf > > >"
    );
}

// ===========================================================================
// 20. Error handling for malformed grammar definitions
// ===========================================================================

#[test]
fn error_target_type_no_angle_brackets_extract_is_unchanged() {
    let ty: Type = parse_quote!(Option);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
    assert!(!ok);
    assert_eq!(type_to_string(&inner), "Option");
}

#[test]
fn error_no_panic_for_non_matching_no_generics() {
    // A type with no generics that doesn't match target/skip — no panic
    let ty: Type = parse_quote!(Foo);
    let (inner, ok) = try_extract_inner_type(&ty, "Bar", &skip_set(&[]));
    assert!(!ok);
    assert_eq!(type_to_string(&inner), "Foo");
}

#[test]
fn error_no_panic_filter_non_matching() {
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(type_to_string(&filtered), "String");
}

#[test]
fn error_no_panic_wrap_plain_type() {
    let ty: Type = parse_quote!(MyType);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&["Box"]));
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < MyType >");
}

#[test]
fn error_wrap_skip_type_parenthesized_generics_preserves_plain_type() {
    // A bare type named "Fn" should be treated as a plain identifier, not a generic wrapper.
    let ty: Type = parse_quote!(Fn);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&["Fn"]));
    assert_eq!(type_to_string(&wrapped), "Fn");
}

// ===========================================================================
// 21. NameValueExpr edge cases
// ===========================================================================

#[test]
fn nv_expr_bool_literal() {
    let nv: NameValueExpr = parse_quote!(enabled = true);
    assert_eq!(nv.path.to_string(), "enabled");
}

#[test]
fn nv_expr_negative_number() {
    let nv: NameValueExpr = parse_quote!(offset = -42);
    assert_eq!(nv.path.to_string(), "offset");
}

#[test]
fn nv_expr_method_call() {
    let nv: NameValueExpr = parse_quote!(default = String::new());
    assert_eq!(nv.path.to_string(), "default");
}

#[test]
fn nv_expr_array_literal() {
    let nv: NameValueExpr = parse_quote!(tokens = [1, 2, 3]);
    assert_eq!(nv.path.to_string(), "tokens");
    assert!(matches!(nv.expr, syn::Expr::Array(_)));
}

// ===========================================================================
// 22. FieldThenParams edge cases
// ===========================================================================

#[test]
fn field_then_params_complex_type_no_params() {
    let parsed: FieldThenParams = parse_quote!(Vec<Option<Box<String>>>);
    assert!(parsed.comma.is_none());
    assert!(parsed.params.is_empty());
}

#[test]
fn field_then_params_single_param() {
    let parsed: FieldThenParams = parse_quote!(String, pattern = r"[a-z]+");
    assert!(parsed.comma.is_some());
    assert_eq!(parsed.params.len(), 1);
    assert_eq!(parsed.params[0].path.to_string(), "pattern");
}

#[test]
fn field_then_params_three_params() {
    let parsed: FieldThenParams = parse_quote!(u32, min = 0, max = 100, step = 1);
    assert_eq!(parsed.params.len(), 3);
    assert_eq!(parsed.params[0].path.to_string(), "min");
    assert_eq!(parsed.params[1].path.to_string(), "max");
    assert_eq!(parsed.params[2].path.to_string(), "step");
}

// ===========================================================================
// 23. Wrap leaf type with deep skip nesting
// ===========================================================================

#[test]
fn wrap_deeply_nested_skip_types_reaches_leaf() {
    let ty: Type = parse_quote!(Vec<Option<Vec<Option<Leaf>>>>);
    let skip = skip_set(&["Vec", "Option"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let s = type_to_string(&wrapped);
    assert!(s.contains("adze :: WithLeaf < Leaf >"));
    // Verify the containers are preserved
    assert!(s.starts_with("Vec"));
    assert!(s.contains("Option"));
}

#[test]
fn wrap_all_containers_are_skip_reaches_innermost() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Cell<Leaf>>>>);
    let skip = skip_set(&["Box", "Arc", "Rc", "Cell"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert!(type_to_string(&wrapped).contains("adze :: WithLeaf < Leaf >"));
}

// ===========================================================================
// 24. Extraction with multiple skip types in chain
// ===========================================================================

#[test]
fn extract_through_three_skip_types() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Vec<Item>>>>);
    let skip = skip_set(&["Box", "Arc", "Rc"]);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(type_to_string(&inner), "Item");
}

#[test]
fn extract_skip_chain_stops_when_target_not_inside() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let skip = skip_set(&["Box", "Arc"]);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!ok);
    assert_eq!(type_to_string(&inner), "Box < Arc < String > >");
}

// ===========================================================================
// 25. Property-based tests with proptest
// ===========================================================================

fn leaf_type_strategy() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i32", "u32", "i64", "u64", "f32", "f64", "bool", "String", "char", "usize",
        ][..],
    )
}

fn container_strategy() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Option", "Vec", "Box", "Arc", "Rc"][..])
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn prop_extract_always_returns_valid_type(
        leaf in leaf_type_strategy(),
        container in container_strategy(),
    ) {
        let src = format!("{container}<{leaf}>");
        let ty: Type = syn::parse_str(&src).unwrap();
        let (inner, extracted) = try_extract_inner_type(&ty, container, &HashSet::new());
        if extracted {
            // Inner type should parse as a valid type
            let s = inner.to_token_stream().to_string();
            prop_assert!(!s.is_empty());
            prop_assert_eq!(s.trim(), leaf);
        }
    }

    #[test]
    fn prop_filter_idempotent(
        leaf in leaf_type_strategy(),
        skip in container_strategy(),
    ) {
        let src = format!("{skip}<{leaf}>");
        let ty: Type = syn::parse_str(&src).unwrap();
        let skip_set: HashSet<&str> = [skip].into_iter().collect();
        let once = filter_inner_type(&ty, &skip_set);
        let twice = filter_inner_type(&once, &skip_set);
        prop_assert_eq!(
            once.to_token_stream().to_string(),
            twice.to_token_stream().to_string()
        );
    }

    #[test]
    fn prop_wrap_always_contains_with_leaf(
        leaf in leaf_type_strategy(),
    ) {
        let ty: Type = syn::parse_str(leaf).unwrap();
        let wrapped = wrap_leaf_type(&ty, &HashSet::new());
        let s = wrapped.to_token_stream().to_string();
        prop_assert!(s.contains("adze :: WithLeaf"), "Expected WithLeaf wrapper in: {}", s);
    }

    #[test]
    fn prop_extract_nonmatch_returns_original(
        leaf in leaf_type_strategy(),
    ) {
        let ty: Type = syn::parse_str(leaf).unwrap();
        let (returned, extracted) = try_extract_inner_type(&ty, "NonExistentType", &HashSet::new());
        prop_assert!(!extracted);
        prop_assert_eq!(
            ty.to_token_stream().to_string(),
            returned.to_token_stream().to_string()
        );
    }

    #[test]
    fn prop_filter_no_skip_returns_original(
        leaf in leaf_type_strategy(),
        container in container_strategy(),
    ) {
        let src = format!("{container}<{leaf}>");
        let ty: Type = syn::parse_str(&src).unwrap();
        let empty_skip: HashSet<&str> = HashSet::new();
        let filtered = filter_inner_type(&ty, &empty_skip);
        prop_assert_eq!(
            ty.to_token_stream().to_string(),
            filtered.to_token_stream().to_string()
        );
    }

    #[test]
    fn prop_wrap_skip_container_wraps_inner_only(
        leaf in leaf_type_strategy(),
        container in container_strategy(),
    ) {
        let src = format!("{container}<{leaf}>");
        let ty: Type = syn::parse_str(&src).unwrap();
        let skip: HashSet<&str> = [container].into_iter().collect();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = wrapped.to_token_stream().to_string();
        // Container should still be present
        prop_assert!(s.contains(container), "Container {} lost in: {}", container, s);
        // Inner leaf should be wrapped
        prop_assert!(s.contains("adze :: WithLeaf"), "Missing WithLeaf in: {}", s);
    }

    #[test]
    fn prop_extract_and_wrap_preserves_leaf_name(
        leaf in leaf_type_strategy(),
        container in container_strategy(),
    ) {
        let src = format!("{container}<{leaf}>");
        let ty: Type = syn::parse_str(&src).unwrap();
        let (inner, extracted) = try_extract_inner_type(&ty, container, &HashSet::new());
        prop_assert!(extracted);
        let wrapped = wrap_leaf_type(&inner, &HashSet::new());
        let s = wrapped.to_token_stream().to_string();
        prop_assert!(s.contains(leaf), "Leaf {} lost after extract+wrap: {}", leaf, s);
    }

    #[test]
    fn prop_nested_extract_through_skip(
        leaf in leaf_type_strategy(),
        skip_c in prop::sample::select(&["Box", "Arc", "Rc"][..]),
        target_c in prop::sample::select(&["Option", "Vec"][..]),
    ) {
        let src = format!("{skip_c}<{target_c}<{leaf}>>");
        let ty: Type = syn::parse_str(&src).unwrap();
        let skip: HashSet<&str> = [skip_c].into_iter().collect();
        let (inner, extracted) = try_extract_inner_type(&ty, target_c, &skip);
        prop_assert!(extracted, "Should extract {target_c} through {skip_c}");
        let inner_str = inner.to_token_stream().to_string();
        prop_assert_eq!(inner_str.trim(), leaf);
    }

    #[test]
    fn prop_filter_then_wrap_produces_valid_output(
        leaf in leaf_type_strategy(),
        container in prop::sample::select(&["Box", "Arc", "Rc"][..]),
    ) {
        let src = format!("{container}<{leaf}>");
        let ty: Type = syn::parse_str(&src).unwrap();
        let skip: HashSet<&str> = [container].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        let wrapped = wrap_leaf_type(&filtered, &HashSet::new());
        let s = wrapped.to_token_stream().to_string();
        prop_assert!(s.contains("adze :: WithLeaf"));
        prop_assert!(s.contains(leaf));
    }

    #[test]
    fn prop_double_wrap_nests_with_leaf(
        leaf in leaf_type_strategy(),
    ) {
        let ty: Type = syn::parse_str(leaf).unwrap();
        let empty: HashSet<&str> = HashSet::new();
        let once = wrap_leaf_type(&ty, &empty);
        let twice = wrap_leaf_type(&once, &empty);
        let s = twice.to_token_stream().to_string();
        // Should contain nested WithLeaf
        prop_assert!(s.starts_with("adze :: WithLeaf < adze :: WithLeaf"));
    }
}

// ===========================================================================
// 26. Additional edge cases for completeness
// ===========================================================================

#[test]
fn edge_dyn_trait_type_wraps() {
    let ty: Type = parse_quote!(dyn Iterator<Item = u8>);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert!(type_to_string(&wrapped).contains("adze :: WithLeaf"));
}

#[test]
fn edge_fn_pointer_type_wraps() {
    let ty: Type = parse_quote!(fn(i32) -> bool);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert!(type_to_string(&wrapped).contains("adze :: WithLeaf"));
}

#[test]
fn edge_slice_type_wraps() {
    let ty: Type = parse_quote!(&[u8]);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert!(type_to_string(&wrapped).contains("adze :: WithLeaf"));
}

#[test]
fn filter_single_element_skip_set() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    // Only Box in skip set: filter unwraps Box, but not Arc
    let filtered = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(type_to_string(&filtered), "Arc < String >");
}

#[test]
fn extract_same_type_as_both_target_and_skip() {
    // If Vec is both target and in skip set, target match takes precedence
    let ty: Type = parse_quote!(Vec<String>);
    let skip = skip_set(&["Vec"]);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(type_to_string(&inner), "String");
}

#[test]
fn wrap_option_inside_vec_with_both_in_skip() {
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let skip = skip_set(&["Vec", "Option"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        type_to_string(&wrapped),
        "Vec < Option < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn name_value_expr_block_expression() {
    let nv: NameValueExpr = parse_quote!(compute = { 1 + 2 });
    assert_eq!(nv.path.to_string(), "compute");
    assert!(matches!(nv.expr, syn::Expr::Block(_)));
}

#[test]
fn field_then_params_preserves_field_type_info() {
    let parsed: FieldThenParams = parse_quote!(HashMap<String, Vec<i32>>, key = "data");
    let field_ty = &parsed.field.ty;
    let (inner, ok) = try_extract_inner_type(field_ty, "HashMap", &skip_set(&[]));
    assert!(ok);
    assert_eq!(type_to_string(&inner), "String");
}
