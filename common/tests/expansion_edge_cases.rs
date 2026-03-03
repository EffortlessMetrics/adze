//! Edge case tests for the common crate's type expansion functionality.
//!
//! Tests cover various edge cases in the grammar expansion pipeline including:
//! - Expanding with minimal input (no rules, single rule)
//! - Complex nesting patterns (optional within repetition, nested extractions)
//! - Empty or homogeneous rule sets
//! - Idempotency and structure preservation

use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use quote::ToTokens;
use syn::{Type, parse_quote};

// ---------------------------------------------------------------------------
// 1. Minimal grammar: no rules (empty field list)
// ---------------------------------------------------------------------------

#[test]
fn expansion_empty_rule_set_no_panic() {
    // Processing zero rules should not panic
    let empty_rules: Vec<(&str, Type)> = vec![];
    let skip_over: HashSet<&str> = HashSet::new();
    let skip_wrap: HashSet<&str> = HashSet::new();

    for (_name, ty) in &empty_rules {
        let (_inner, _extracted) = try_extract_inner_type(ty, "Option", &skip_over);
        let _filtered = filter_inner_type(ty, &skip_over);
        let _wrapped = wrap_leaf_type(ty, &skip_wrap);
    }
    // Test passes if we reach here without panic
}

// ---------------------------------------------------------------------------
// 2. Single rule
// ---------------------------------------------------------------------------

#[test]
fn expansion_single_rule() {
    let skip_over: HashSet<&str> = HashSet::new();
    let skip_wrap: HashSet<&str> = HashSet::new();

    let ty: Type = parse_quote!(String);

    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip_over);
    assert!(!extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");

    let filtered = filter_inner_type(&ty, &skip_over);
    assert_eq!(filtered.to_token_stream().to_string(), "String");

    let wrapped = wrap_leaf_type(&ty, &skip_wrap);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < String >"
    );
}

// ---------------------------------------------------------------------------
// 3. All rules have same LHS (all same type being processed)
// ---------------------------------------------------------------------------

#[test]
fn expansion_homogeneous_rules() {
    // Multiple rules, all same type
    let rules = vec![
        ("first", parse_quote!(i32)),
        ("second", parse_quote!(i32)),
        ("third", parse_quote!(i32)),
    ];

    let skip_over: HashSet<&str> = HashSet::new();
    let skip_wrap: HashSet<&str> = HashSet::new();

    let mut results = vec![];
    for (_name, ty) in rules {
        let filtered = filter_inner_type(&ty, &skip_over);
        let wrapped = wrap_leaf_type(&filtered, &skip_wrap);
        results.push(wrapped.to_token_stream().to_string());
    }

    // All results should be identical
    assert_eq!(results[0], results[1]);
    assert_eq!(results[1], results[2]);
    assert!(results[0].contains("adze :: WithLeaf < i32 >"));
}

// ---------------------------------------------------------------------------
// 4. Optional inside repetition: Option<Vec<T>>
// ---------------------------------------------------------------------------

#[test]
fn expansion_optional_inside_repetition() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let skip_over: HashSet<&str> = ["Box", "Spanned"].into_iter().collect();
    let skip_wrap: HashSet<&str> = ["Option", "Vec"].into_iter().collect();

    // First extract Option (should find it at top level)
    let (after_option, found_option) = try_extract_inner_type(&ty, "Option", &skip_over);
    assert!(found_option);
    assert_eq!(after_option.to_token_stream().to_string(), "Vec < String >");

    // Now extract Vec from the inner type
    let (leaf, found_vec) = try_extract_inner_type(&after_option, "Vec", &skip_over);
    assert!(found_vec);
    assert_eq!(leaf.to_token_stream().to_string(), "String");

    // Full wrap should preserve both containers
    let wrapped = wrap_leaf_type(&ty, &skip_wrap);
    let wrapped_str = wrapped.to_token_stream().to_string();
    assert!(wrapped_str.contains("Option"));
    assert!(wrapped_str.contains("Vec"));
    assert!(wrapped_str.contains("WithLeaf < String >"));
}

// ---------------------------------------------------------------------------
// 5. Nested choices (nested extractions from complex types)
// ---------------------------------------------------------------------------

#[test]
fn expansion_nested_extraction() {
    // Type: Option<Box<Vec<Option<String>>>>
    let ty: Type = parse_quote!(Option<Box<Vec<Option<String>>>>);
    let skip_over: HashSet<&str> = ["Box", "Spanned"].into_iter().collect();

    // Extract first Option
    let (step1, found1) = try_extract_inner_type(&ty, "Option", &skip_over);
    assert!(found1);
    let step1_str = step1.to_token_stream().to_string();
    assert!(step1_str.contains("Box"));

    // Step 2: Now we have Box<Vec<Option<String>>>, extract Vec (skipping Box)
    let (step2, found2) = try_extract_inner_type(&step1, "Vec", &skip_over);
    assert!(found2);
    assert!(step2.to_token_stream().to_string().contains("Option"));

    // Extract Option from Vec<Option<String>>
    let (step3, found3) = try_extract_inner_type(&step2, "Option", &skip_over);
    assert!(found3);
    assert_eq!(step3.to_token_stream().to_string(), "String");
}

// ---------------------------------------------------------------------------
// 6. Empty alternatives / edge case types
// ---------------------------------------------------------------------------

#[test]
fn expansion_unit_type() {
    let ty: Type = parse_quote!(());
    let skip_over: HashSet<&str> = HashSet::new();
    let skip_wrap: HashSet<&str> = HashSet::new();

    let (_inner, extracted) = try_extract_inner_type(&ty, "Option", &skip_over);
    assert!(!extracted);

    let filtered = filter_inner_type(&ty, &skip_over);
    assert_eq!(filtered.to_token_stream().to_string(), "()");

    let wrapped = wrap_leaf_type(&ty, &skip_wrap);
    // Unit type gets wrapped in WithLeaf
    assert!(
        wrapped
            .to_token_stream()
            .to_string()
            .contains("adze :: WithLeaf")
    );
}

#[test]
fn expansion_never_type() {
    let ty: Type = parse_quote!(!);
    let skip_over: HashSet<&str> = HashSet::new();
    let skip_wrap: HashSet<&str> = HashSet::new();

    let (_inner, extracted) = try_extract_inner_type(&ty, "Option", &skip_over);
    assert!(!extracted);

    let filtered = filter_inner_type(&ty, &skip_over);
    assert_eq!(filtered.to_token_stream().to_string(), "!");

    let wrapped = wrap_leaf_type(&ty, &skip_wrap);
    assert!(wrapped.to_token_stream().to_string().contains("WithLeaf"));
}

// ---------------------------------------------------------------------------
// 7. Idempotency: multiple expansions of same input produce identical output
// ---------------------------------------------------------------------------

#[test]
fn expansion_idempotent_extraction() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let skip_over: HashSet<&str> = ["Box"].into_iter().collect();

    // First expansion
    let (result1, _) = try_extract_inner_type(&ty, "Vec", &skip_over);
    let result1_str = result1.to_token_stream().to_string();

    // Second expansion (same input, same params)
    let (result2, _) = try_extract_inner_type(&ty, "Vec", &skip_over);
    let result2_str = result2.to_token_stream().to_string();

    // Third expansion (same input, same params)
    let (result3, _) = try_extract_inner_type(&ty, "Vec", &skip_over);
    let result3_str = result3.to_token_stream().to_string();

    assert_eq!(result1_str, result2_str);
    assert_eq!(result2_str, result3_str);
}

#[test]
fn expansion_idempotent_wrapping() {
    let ty: Type = parse_quote!(Vec<String>);
    let skip_wrap: HashSet<&str> = ["Vec"].into_iter().collect();

    // First wrapping
    let wrapped1 = wrap_leaf_type(&ty, &skip_wrap);
    let wrapped1_str = wrapped1.to_token_stream().to_string();

    // Second wrapping (same input, same params)
    let wrapped2 = wrap_leaf_type(&ty, &skip_wrap);
    let wrapped2_str = wrapped2.to_token_stream().to_string();

    // Third wrapping (same input, same params)
    let wrapped3 = wrap_leaf_type(&ty, &skip_wrap);
    let wrapped3_str = wrapped3.to_token_stream().to_string();

    assert_eq!(wrapped1_str, wrapped2_str);
    assert_eq!(wrapped2_str, wrapped3_str);
}

#[test]
fn expansion_idempotent_filtering() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let skip_over: HashSet<&str> = ["Box", "Option"].into_iter().collect();

    // First filtering
    let filtered1 = filter_inner_type(&ty, &skip_over);
    let filtered1_str = filtered1.to_token_stream().to_string();

    // Second filtering
    let filtered2 = filter_inner_type(&ty, &skip_over);
    let filtered2_str = filtered2.to_token_stream().to_string();

    // Third filtering
    let filtered3 = filter_inner_type(&ty, &skip_over);
    let filtered3_str = filtered3.to_token_stream().to_string();

    assert_eq!(filtered1_str, filtered2_str);
    assert_eq!(filtered2_str, filtered3_str);
}

// ---------------------------------------------------------------------------
// 8. Structural preservation: type identity maintained through operations
// ---------------------------------------------------------------------------

#[test]
fn expansion_preserves_type_identity_on_failed_extraction() {
    let ty: Type = parse_quote!(String);
    let skip_over: HashSet<&str> = HashSet::new();

    // Extract for a type that doesn't match
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip_over);
    assert!(!found);

    // Result should equal input
    assert_eq!(
        result.to_token_stream().to_string(),
        ty.to_token_stream().to_string()
    );
}

#[test]
fn expansion_preserves_type_identity_on_failed_filter() {
    let ty: Type = parse_quote!(String);
    let skip_over: HashSet<&str> = ["Box"].into_iter().collect();

    // Filter for a type that's not in the skip set
    let result = filter_inner_type(&ty, &skip_over);

    // Result should equal input
    assert_eq!(
        result.to_token_stream().to_string(),
        ty.to_token_stream().to_string()
    );
}

#[test]
fn expansion_structure_preservation_skip_over_empty() {
    // When skip_over is empty, extraction should only work for direct matches
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let skip_over: HashSet<&str> = HashSet::new();

    // Without Box in skip_over, we can't reach Vec
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip_over);
    assert!(!found);
    assert_eq!(
        result.to_token_stream().to_string(),
        "Box < Vec < String > >"
    );
}

#[test]
fn expansion_structure_preservation_with_skip_over() {
    // Same type, but with Box in skip_over
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let skip_over: HashSet<&str> = ["Box"].into_iter().collect();

    // Now we can reach Vec
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip_over);
    assert!(found);
    assert_eq!(result.to_token_stream().to_string(), "String");
}

#[test]
fn expansion_wrap_preserves_container_types() {
    // Verify that wrap_leaf_type preserves container types in skip_wrap
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let skip_wrap: HashSet<&str> = ["Vec", "Option"].into_iter().collect();

    let wrapped = wrap_leaf_type(&ty, &skip_wrap);
    let wrapped_str = wrapped.to_token_stream().to_string();

    // Should preserve both Vec and Option
    assert!(wrapped_str.contains("Vec"));
    assert!(wrapped_str.contains("Option"));
    // Should wrap the leaf String
    assert!(wrapped_str.contains("WithLeaf < String >"));
}

// ---------------------------------------------------------------------------
// Edge case: Box and Spanned commonly used together
// ---------------------------------------------------------------------------

#[test]
fn expansion_common_skip_over_pattern() {
    // Simulate real usage pattern from the main codebase
    let skip_over: HashSet<&str> = ["Spanned", "Box"].into_iter().collect();

    let ty1: Type = parse_quote!(Box<Spanned<Vec<Token>>>);
    let (inner1, found1) = try_extract_inner_type(&ty1, "Vec", &skip_over);
    assert!(found1);
    assert_eq!(inner1.to_token_stream().to_string(), "Token");

    let ty2: Type = parse_quote!(Spanned<Box<Option<Ident>>>);
    let (inner2, found2) = try_extract_inner_type(&ty2, "Option", &skip_over);
    assert!(found2);
    assert_eq!(inner2.to_token_stream().to_string(), "Ident");

    let ty3: Type = parse_quote!(Option<Spanned<Box<Vec<Item>>>>);
    let (after_opt, found3) = try_extract_inner_type(&ty3, "Option", &skip_over);
    assert!(found3);

    let (final_inner, found4) = try_extract_inner_type(&after_opt, "Vec", &skip_over);
    assert!(found4);
    assert_eq!(final_inner.to_token_stream().to_string(), "Item");
}

// ---------------------------------------------------------------------------
// Edge case: Multiple nested Options or Vecs
// ---------------------------------------------------------------------------

#[test]
fn expansion_consecutive_same_containers() {
    let ty: Type = parse_quote!(Vec<Vec<String>>);
    let skip_over: HashSet<&str> = HashSet::new();

    // Extract first Vec
    let (step1, found1) = try_extract_inner_type(&ty, "Vec", &skip_over);
    assert!(found1);
    assert!(
        step1
            .to_token_stream()
            .to_string()
            .contains("Vec < String >")
    );

    // Extract second Vec
    let (step2, found2) = try_extract_inner_type(&step1, "Vec", &skip_over);
    assert!(found2);
    assert_eq!(step2.to_token_stream().to_string(), "String");
}

#[test]
fn expansion_consecutive_optional() {
    let ty: Type = parse_quote!(Option<Option<String>>);
    let skip_over: HashSet<&str> = HashSet::new();

    // Extract first Option
    let (step1, found1) = try_extract_inner_type(&ty, "Option", &skip_over);
    assert!(found1);
    assert!(
        step1
            .to_token_stream()
            .to_string()
            .contains("Option < String >")
    );

    // Extract second Option
    let (step2, found2) = try_extract_inner_type(&step1, "Option", &skip_over);
    assert!(found2);
    assert_eq!(step2.to_token_stream().to_string(), "String");
}
