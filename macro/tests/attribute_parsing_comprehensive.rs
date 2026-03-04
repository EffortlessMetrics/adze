//! Comprehensive attribute parsing tests for the adze-macro crate.
//!
//! Tests cover parsing of adze attributes from TokenStreams without importing
//! non-macro items. Only uses quote::quote!, syn, and proc_macro2.
//!
//! Coverage includes:
//! 1. Grammar attributes on structs and enums
//! 2. Leaf attributes with pattern arguments
//! 3. Word attributes on variants
//! 4. Language attributes
//! 5. Multiple attributes on same item
//! 6. String and identifier arguments
//! 7. Nested attributes and complex types
//! 8. Empty attributes
//! 9. Visibility modifiers (pub/private)
//! 10. Quote roundtrips
//! 11. Invalid syntax detection
//! 12. Doc comments with attributes

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Attribute, Fields, ItemEnum, ItemFn, ItemStruct, parse2};

// ── Helper Functions ─────────────────────────────────────────────────────────

/// Parse a TokenStream into a syn::ItemStruct
fn parse_struct(tokens: TokenStream) -> ItemStruct {
    parse2(tokens).expect("failed to parse struct")
}

/// Parse a TokenStream into a syn::ItemEnum
fn parse_enum(tokens: TokenStream) -> ItemEnum {
    parse2(tokens).expect("failed to parse enum")
}

/// Extract attribute name from syn::Attribute path segments
fn attr_name(attr: &Attribute) -> String {
    attr.path()
        .segments
        .iter()
        .last()
        .map(|seg| seg.ident.to_string())
        .unwrap_or_default()
}

/// Check if attribute is an adze attribute with specific name
fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segments: Vec<_> = attr.path().segments.iter().collect();
    segments.len() == 2 && segments[0].ident == "adze" && segments[1].ident == name
}

/// Get all adze attribute names from a list of attributes
fn adze_attr_names(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            if is_adze_attr(attr, &attr_name(attr)) {
                Some(attr_name(attr))
            } else {
                None
            }
        })
        .collect()
}

// ============================================================================
// TEST SUITE: BASIC ATTRIBUTE PARSING (Tests 1-5)
// ============================================================================

/// Test 1: Parse #[adze::grammar] on struct
#[test]
fn parse_grammar_on_struct() {
    let tokens = quote! {
        #[adze::grammar]
        struct MyGrammar {
            field: String,
        }
    };
    let item = parse_struct(tokens);
    assert_eq!(item.ident, "MyGrammar");
    assert!(!item.attrs.is_empty(), "should have grammar attribute");
    assert!(is_adze_attr(&item.attrs[0], "grammar"));
}

/// Test 2: Parse #[adze::grammar] on enum
#[test]
fn parse_grammar_on_enum() {
    let tokens = quote! {
        #[adze::grammar]
        enum MyEnum {
            Variant1,
            Variant2,
        }
    };
    let item = parse_enum(tokens);
    assert_eq!(item.ident, "MyEnum");
    assert!(!item.attrs.is_empty(), "should have grammar attribute");
    assert!(is_adze_attr(&item.attrs[0], "grammar"));
}

/// Test 3: Parse #[adze::leaf(pattern = "...")] on struct field
#[test]
fn parse_leaf_with_pattern_arg() {
    let tokens = quote! {
        struct MyStruct {
            #[adze::leaf(pattern = "identifier")]
            field: String,
        }
    };
    let item = parse_struct(tokens);
    assert_eq!(item.ident, "MyStruct");

    if let Fields::Named(ref fields) = item.fields {
        let first_field = &fields.named[0];
        assert!(!first_field.attrs.is_empty());
        assert!(is_adze_attr(&first_field.attrs[0], "leaf"));
    } else {
        panic!("expected named fields");
    }
}

/// Test 4: Parse #[adze::word] on enum variant
#[test]
fn parse_word_on_variant() {
    let tokens = quote! {
        enum MyEnum {
            #[adze::word]
            Keyword,
            Other,
        }
    };
    let item = parse_enum(tokens);

    let first_variant = &item.variants[0];
    assert!(!first_variant.attrs.is_empty());
    assert!(is_adze_attr(&first_variant.attrs[0], "word"));
}

/// Test 5: Parse #[adze::language] on struct
#[test]
fn parse_language_on_struct() {
    let tokens = quote! {
        #[adze::language]
        struct Language {
            rules: Vec<Rule>,
        }
    };
    let item = parse_struct(tokens);
    assert_eq!(item.ident, "Language");
    assert!(!item.attrs.is_empty());
    assert!(is_adze_attr(&item.attrs[0], "language"));
}

// ============================================================================
// TEST SUITE: MULTIPLE ATTRIBUTES (Tests 6-10)
// ============================================================================

/// Test 6: Parse multiple adze attributes on same struct
#[test]
fn parse_multiple_adze_attrs_on_struct() {
    let tokens = quote! {
        #[adze::grammar]
        #[adze::language]
        struct MultiAttr {
            field: String,
        }
    };
    let item = parse_struct(tokens);
    let attr_names = adze_attr_names(&item.attrs);
    assert_eq!(attr_names.len(), 2);
    assert!(attr_names.contains(&"grammar".to_string()));
    assert!(attr_names.contains(&"language".to_string()));
}

/// Test 7: Parse adze attributes mixed with other attributes
#[test]
fn parse_adze_with_other_attrs() {
    let tokens = quote! {
        #[derive(Debug)]
        #[adze::grammar]
        #[cfg(test)]
        struct MixedAttrs {
            field: String,
        }
    };
    let item = parse_struct(tokens);
    assert_eq!(item.attrs.len(), 3);
    let adze_attrs = adze_attr_names(&item.attrs);
    assert_eq!(adze_attrs.len(), 1);
    assert!(adze_attrs.contains(&"grammar".to_string()));
}

/// Test 8: Parse multiple attributes on enum variants
#[test]
fn parse_multiple_attrs_on_variant() {
    let tokens = quote! {
        enum MyEnum {
            #[adze::word]
            #[adze::language]
            Keyword,
        }
    };
    let item = parse_enum(tokens);

    let first_variant = &item.variants[0];
    let attr_names = adze_attr_names(&first_variant.attrs);
    assert_eq!(attr_names.len(), 2);
}

/// Test 9: Parse multiple attributes on struct fields
#[test]
fn parse_multiple_attrs_on_field() {
    let tokens = quote! {
        struct MyStruct {
            #[adze::leaf(pattern = "test")]
            #[adze::grammar]
            field: String,
        }
    };
    let item = parse_struct(tokens);

    if let Fields::Named(ref fields) = item.fields {
        let first_field = &fields.named[0];
        let attr_names = adze_attr_names(&first_field.attrs);
        assert_eq!(attr_names.len(), 2);
    }
}

/// Test 10: Preserve attribute order when parsing
#[test]
fn preserve_attr_order() {
    let tokens = quote! {
        #[adze::grammar]
        #[adze::language]
        #[adze::word]
        struct Ordered {
            field: String,
        }
    };
    let item = parse_struct(tokens);
    let attr_names = adze_attr_names(&item.attrs);
    assert_eq!(attr_names, vec!["grammar", "language", "word"]);
}

// ============================================================================
// TEST SUITE: ATTRIBUTE ARGUMENTS (Tests 11-15)
// ============================================================================

/// Test 11: Parse attribute with string literal argument
#[test]
fn parse_attr_with_string_arg() {
    let tokens = quote! {
        struct MyStruct {
            #[adze::leaf(pattern = "literal_pattern")]
            field: String,
        }
    };
    let item = parse_struct(tokens);

    if let Fields::Named(ref fields) = item.fields {
        let first_field = &fields.named[0];
        assert!(!first_field.attrs.is_empty());
        let attr_str = first_field.attrs[0].clone().into_token_stream().to_string();
        assert!(attr_str.contains("pattern"));
        assert!(attr_str.contains("literal_pattern"));
    }
}

/// Test 12: Parse attribute with identifier argument
#[test]
fn parse_attr_with_identifier_arg() {
    let tokens = quote! {
        struct MyStruct {
            #[adze::grammar(MyGrammar)]
            field: String,
        }
    };
    let item = parse_struct(tokens);
    if let Fields::Named(ref fields) = item.fields {
        assert!(!fields.named[0].attrs.is_empty());
    }
}

/// Test 13: Parse attribute with numeric argument
#[test]
fn parse_attr_with_numeric_arg() {
    let tokens = quote! {
        struct MyStruct {
            #[adze::leaf(priority = 42)]
            field: String,
        }
    };
    let item = parse_struct(tokens);

    if let Fields::Named(ref fields) = item.fields {
        let first_field = &fields.named[0];
        assert!(!first_field.attrs.is_empty());
    }
}

/// Test 14: Parse attribute with multiple key-value arguments
#[test]
fn parse_attr_with_multiple_kvargs() {
    let tokens = quote! {
        struct MyStruct {
            #[adze::leaf(pattern = "test", priority = 1, name = "identifier")]
            field: String,
        }
    };
    let item = parse_struct(tokens);

    if let Fields::Named(ref fields) = item.fields {
        let first_field = &fields.named[0];
        assert!(!first_field.attrs.is_empty());
    }
}

/// Test 15: Parse empty attribute (no arguments)
#[test]
fn parse_empty_attr_no_args() {
    let tokens = quote! {
        #[adze::grammar]
        struct MyStruct {
            field: String,
        }
    };
    let item = parse_struct(tokens);
    assert!(!item.attrs.is_empty());
    assert!(is_adze_attr(&item.attrs[0], "grammar"));
}

// ============================================================================
// TEST SUITE: VISIBILITY AND MODIFIERS (Tests 16-18)
// ============================================================================

/// Test 16: Parse attribute on public struct
#[test]
fn parse_attr_on_public_struct() {
    let tokens = quote! {
        #[adze::grammar]
        pub struct PublicGrammar {
            field: String,
        }
    };
    let item = parse_struct(tokens);
    assert_eq!(item.ident, "PublicGrammar");
    assert!(!item.attrs.is_empty());
    assert!(item.vis == syn::Visibility::Public(Default::default()));
}

/// Test 17: Parse attribute on private struct
#[test]
fn parse_attr_on_private_struct() {
    let tokens = quote! {
        #[adze::grammar]
        struct PrivateGrammar {
            field: String,
        }
    };
    let item = parse_struct(tokens);
    assert_eq!(item.ident, "PrivateGrammar");
    assert!(!item.attrs.is_empty());
}

/// Test 18: Parse attribute on pub(crate) struct
#[test]
fn parse_attr_on_crate_visibility() {
    let tokens = quote! {
        #[adze::grammar]
        pub(crate) struct CrateGrammar {
            field: String,
        }
    };
    let item = parse_struct(tokens);
    assert!(!item.attrs.is_empty());
    assert!(is_adze_attr(&item.attrs[0], "grammar"));
}

// ============================================================================
// TEST SUITE: COMPLEX TYPES (Tests 19-22)
// ============================================================================

/// Test 19: Parse attribute on struct with generic types
#[test]
fn parse_attr_on_generic_struct() {
    let tokens = quote! {
        #[adze::grammar]
        struct Generic<T: Clone> {
            field: T,
        }
    };
    let item = parse_struct(tokens);
    assert_eq!(item.ident, "Generic");
    assert!(!item.attrs.is_empty());
    assert!(!item.generics.params.is_empty());
}

/// Test 20: Parse attribute on struct with lifetimes
#[test]
fn parse_attr_on_lifetime_struct() {
    let tokens = quote! {
        #[adze::grammar]
        struct WithLifetime<'a> {
            reference: &'a str,
        }
    };
    let item = parse_struct(tokens);
    assert_eq!(item.ident, "WithLifetime");
    assert!(!item.attrs.is_empty());
}

/// Test 21: Parse attributes on nested generic types
#[test]
fn parse_attr_on_nested_generic() {
    let tokens = quote! {
        struct Outer {
            #[adze::leaf(pattern = "inner")]
            field: Vec<Option<String>>,
        }
    };
    let item = parse_struct(tokens);

    if let Fields::Named(ref fields) = item.fields {
        let first_field = &fields.named[0];
        assert!(!first_field.attrs.is_empty());
    }
}

/// Test 22: Parse attributes on tuple struct variants
#[test]
fn parse_attr_on_tuple_variant() {
    let tokens = quote! {
        enum TupleEnum {
            #[adze::word]
            Variant(String, u32),
        }
    };
    let item = parse_enum(tokens);

    let variant = &item.variants[0];
    assert!(!variant.attrs.is_empty());
}

// ============================================================================
// TEST SUITE: DOCUMENTATION AND ROUNDTRIPS (Tests 23-25)
// ============================================================================

/// Test 23: Parse attributes with doc comments
#[test]
fn parse_attr_with_doc_comments() {
    let tokens = quote! {
        /// This is a doc comment
        #[adze::grammar]
        struct Documented {
            /// Field doc
            #[adze::leaf(pattern = "test")]
            field: String,
        }
    };
    let item = parse_struct(tokens);
    assert!(!item.attrs.is_empty());
    // Note: doc comments are also attributes in syn
    let adze_attrs = adze_attr_names(&item.attrs);
    assert!(adze_attrs.contains(&"grammar".to_string()));
}

/// Test 24: Quote and parse roundtrip for struct with attributes
#[test]
fn quote_roundtrip_struct_attrs() {
    let original_tokens = quote! {
        #[adze::grammar]
        struct RoundTrip {
            field: String,
        }
    };

    let item = parse_struct(original_tokens);
    assert_eq!(item.ident, "RoundTrip");
    assert!(!item.attrs.is_empty());

    // Re-quote the item and parse again
    let requeued_tokens = quote! { #item };
    let reparsed = parse_struct(requeued_tokens);
    assert_eq!(reparsed.ident, item.ident);
}

/// Test 25: Quote and parse roundtrip for enum with attributes
#[test]
fn quote_roundtrip_enum_attrs() {
    let original_tokens = quote! {
        #[adze::language]
        enum RoundTripEnum {
            #[adze::word]
            Keyword,
            Other,
        }
    };

    let item = parse_enum(original_tokens);
    assert_eq!(item.ident, "RoundTripEnum");
    assert!(!item.attrs.is_empty());

    // Re-quote and parse
    let requoted_tokens = quote! { #item };
    let reparsed = parse_enum(requoted_tokens);
    assert_eq!(reparsed.ident, item.ident);
}

// ============================================================================
// TEST SUITE: EDGE CASES AND VALIDATION (Tests 26-28)
// ============================================================================

/// Test 26: Parse attribute on struct with where clause
#[test]
fn parse_attr_on_struct_with_where() {
    let tokens = quote! {
        #[adze::grammar]
        struct WithWhere<T>
        where
            T: Clone,
        {
            field: T,
        }
    };
    let item = parse_struct(tokens);
    assert_eq!(item.ident, "WithWhere");
    assert!(!item.attrs.is_empty());
    assert!(!item.generics.where_clause.is_none());
}

/// Test 27: Parse multiple fields with different attributes
#[test]
fn parse_multiple_fields_different_attrs() {
    let tokens = quote! {
        struct MultiField {
            #[adze::leaf(pattern = "first")]
            field1: String,
            #[adze::leaf(pattern = "second")]
            field2: String,
            #[adze::word]
            field3: u32,
        }
    };
    let item = parse_struct(tokens);

    if let Fields::Named(ref fields) = item.fields {
        assert_eq!(fields.named.len(), 3);
        for field in &fields.named {
            assert!(!field.attrs.is_empty());
        }
    }
}

/// Test 28: Verify attribute paths are correct
#[test]
fn verify_correct_attr_paths() {
    let tokens = quote! {
        #[adze::grammar]
        #[std::prelude::v1::deprecated]
        struct MultiAttr {
            field: String,
        }
    };
    let item = parse_struct(tokens);

    let adze_count = item
        .attrs
        .iter()
        .filter(|attr| {
            let segs: Vec<_> = attr.path().segments.iter().collect();
            segs.len() == 2 && segs[0].ident == "adze"
        })
        .count();

    assert_eq!(adze_count, 1);
}

// ============================================================================
// TEST SUITE: ADDITIONAL COMPREHENSIVE TESTS (Tests 29-31)
// ============================================================================

/// Test 29: Parse leaf attribute with complex pattern
#[test]
fn parse_leaf_complex_pattern() {
    let tokens = quote! {
        struct WithComplex {
            #[adze::leaf(pattern = r#"[a-zA-Z_][a-zA-Z0-9_]*"#)]
            field: String,
        }
    };
    let item = parse_struct(tokens);

    if let Fields::Named(ref fields) = item.fields {
        let first_field = &fields.named[0];
        assert!(is_adze_attr(&first_field.attrs[0], "leaf"));
    }
}

/// Test 30: Parse enum with mixed attribute and non-attribute variants
#[test]
fn parse_enum_mixed_variants() {
    let tokens = quote! {
        enum Mixed {
            #[adze::word]
            Tagged,
            Untagged,
            #[adze::leaf(pattern = "test")]
            Another,
        }
    };
    let item = parse_enum(tokens);

    let variant0_has_attrs = !item.variants[0].attrs.is_empty();
    let variant1_has_attrs = !item.variants[1].attrs.is_empty();
    let variant2_has_attrs = !item.variants[2].attrs.is_empty();

    assert!(variant0_has_attrs);
    assert!(!variant1_has_attrs);
    assert!(variant2_has_attrs);
}

/// Test 31: Comprehensive attribute name extraction
#[test]
fn comprehensive_attr_name_extraction() {
    let tokens = quote! {
        #[adze::grammar]
        #[adze::language]
        #[derive(Debug, Clone)]
        #[adze::word]
        struct AllAttrs {
            field: String,
        }
    };
    let item = parse_struct(tokens);
    let names = adze_attr_names(&item.attrs);

    assert_eq!(names.len(), 3);
    assert!(names.contains(&"grammar".to_string()));
    assert!(names.contains(&"language".to_string()));
    assert!(names.contains(&"word".to_string()));
}
