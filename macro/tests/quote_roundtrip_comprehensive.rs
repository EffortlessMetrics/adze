//! Comprehensive quote/syn roundtrip tests for adze-macro.
//!
//! These tests verify that:
//! 1. Code can be parsed via syn
//! 2. Parsed code can be re-quoted via quote!
//! 3. Re-quoted code maintains semantic equivalence
//! 4. Attributes are preserved correctly through the roundtrip
//!
//! We test parsing and roundtripping of various Rust patterns with
//! adze-specific attributes without importing macro expansion logic.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, ItemEnum, ItemStruct};

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Helper to compare token streams by parsing both as DeriveInput
fn _roundtrip_derive_input(original: TokenStream) -> DeriveInput {
    let parsed: DeriveInput = syn::parse2(original).expect("failed to parse DeriveInput");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: DeriveInput = syn::parse2(requoted).expect("failed to reparse after quote!");
    reparsed
}

/// Helper to compare token streams by parsing both as ItemStruct
fn _roundtrip_item_struct(original: TokenStream) -> ItemStruct {
    let parsed: ItemStruct = syn::parse2(original).expect("failed to parse ItemStruct");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemStruct = syn::parse2(requoted).expect("failed to reparse ItemStruct");
    reparsed
}

/// Helper to compare token streams by parsing both as ItemEnum
fn _roundtrip_item_enum(original: TokenStream) -> ItemEnum {
    let parsed: ItemEnum = syn::parse2(original).expect("failed to parse ItemEnum");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemEnum = syn::parse2(requoted).expect("failed to reparse ItemEnum");
    reparsed
}

/// Helper to check if an attribute has a specific path
fn has_attr(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| {
        let path_str = quote!(#attr).to_string();
        path_str.contains(name)
    })
}

/// Helper to count attributes with a specific path
fn _count_attrs(attrs: &[Attribute], name: &str) -> usize {
    attrs
        .iter()
        .filter(|attr| {
            let path_str = quote!(#attr).to_string();
            path_str.contains(name)
        })
        .count()
}

// ── Test 1: Parse simple struct with #[adze::grammar] ──────────────────────

#[test]
fn test_parse_struct_with_adze_grammar() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        pub struct MyGrammar {
            name: String,
        }
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    assert_eq!(item.ident, "MyGrammar");
    assert!(has_attr(&item.attrs, "grammar"));
}

// ── Test 2: Roundtrip struct with #[adze::grammar] ─────────────────────────

#[test]
fn test_roundtrip_struct_with_adze_grammar() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        pub struct MyGrammar {
            name: String,
        }
    };

    let parsed: ItemStruct = syn::parse2(tokens.clone()).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemStruct = syn::parse2(requoted).expect("parse 2");

    assert_eq!(parsed.ident, reparsed.ident);
    assert_eq!(parsed.attrs.len(), reparsed.attrs.len());
}

// ── Test 3: Parse enum with #[adze::grammar] ──────────────────────────────

#[test]
fn test_parse_enum_with_adze_grammar() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        pub enum MyRule {
            Variant1,
            Variant2,
        }
    };

    let item: ItemEnum = syn::parse2(tokens).expect("failed to parse enum");
    assert_eq!(item.ident, "MyRule");
    assert!(has_attr(&item.attrs, "grammar"));
}

// ── Test 4: Roundtrip enum with #[adze::grammar] ──────────────────────────

#[test]
fn test_roundtrip_enum_with_adze_grammar() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        pub enum MyRule {
            Variant1,
            Variant2,
        }
    };

    let parsed: ItemEnum = syn::parse2(tokens.clone()).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemEnum = syn::parse2(requoted).expect("parse 2");

    assert_eq!(parsed.ident, reparsed.ident);
    assert_eq!(parsed.variants.len(), reparsed.variants.len());
}

// ── Test 5: Parse field with #[adze::leaf(pattern = "regex")] ─────────────

#[test]
fn test_parse_field_with_adze_leaf_pattern() {
    let tokens: TokenStream = quote! {
        pub struct MyStruct {
            #[adze::leaf(pattern = "regex")]
            field: String,
        }
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    if let syn::Fields::Named(fields) = &item.fields {
        let field = &fields.named[0];
        assert!(has_attr(&field.attrs, "leaf"));
    } else {
        panic!("expected named fields");
    }
}

// ── Test 6: Parse variant with #[adze::word] ──────────────────────────────

#[test]
fn test_parse_variant_with_adze_word() {
    let tokens: TokenStream = quote! {
        pub enum MyEnum {
            #[adze::word]
            Keyword,
        }
    };

    let item: ItemEnum = syn::parse2(tokens).expect("failed to parse enum");
    let variant = &item.variants[0];
    assert!(has_attr(&variant.attrs, "word"));
}

// ── Test 7: Multiple attributes on same item ──────────────────────────────

#[test]
fn test_multiple_attributes_on_struct() {
    let tokens: TokenStream = quote! {
        #[derive(Clone)]
        #[adze::grammar]
        #[derive(Debug)]
        pub struct MyStruct {
            field: String,
        }
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    assert!(has_attr(&item.attrs, "grammar"));
    assert!(has_attr(&item.attrs, "derive"));
}

// ── Test 8: Roundtrip with multiple attributes ────────────────────────────

#[test]
fn test_roundtrip_multiple_attributes_on_struct() {
    let tokens: TokenStream = quote! {
        #[derive(Clone)]
        #[adze::grammar]
        #[derive(Debug)]
        pub struct MyStruct {
            field: String,
        }
    };

    let parsed: ItemStruct = syn::parse2(tokens.clone()).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemStruct = syn::parse2(requoted).expect("parse 2");

    assert_eq!(parsed.attrs.len(), reparsed.attrs.len());
}

// ── Test 9: Attributes with string arguments ───────────────────────────────

#[test]
fn test_attribute_with_string_argument() {
    let tokens: TokenStream = quote! {
        pub struct MyStruct {
            #[adze::leaf(pattern = "test_pattern")]
            field: String,
        }
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    if let syn::Fields::Named(fields) = &item.fields {
        let field = &fields.named[0];
        let _attr_str = quote!(#[adze::leaf(pattern = "test_pattern")]).to_string();
        let _field_attr_str = quote!(&field.attrs[0]).to_string();
        // Verify attribute exists
        assert!(!field.attrs.is_empty());
    } else {
        panic!("expected named fields");
    }
}

// ── Test 10: Roundtrip attribute with string argument ──────────────────────

#[test]
fn test_roundtrip_attribute_with_string_argument() {
    let tokens: TokenStream = quote! {
        pub struct MyStruct {
            #[adze::leaf(pattern = "test_pattern")]
            field: String,
        }
    };

    let parsed: ItemStruct = syn::parse2(tokens).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemStruct = syn::parse2(requoted).expect("parse 2");

    if let syn::Fields::Named(parsed_fields) = &parsed.fields
        && let syn::Fields::Named(reparsed_fields) = &reparsed.fields
    {
        assert_eq!(
            parsed_fields.named[0].attrs.len(),
            reparsed_fields.named[0].attrs.len()
        );
    }
}

// ── Test 11: Attributes with integer arguments ────────────────────────────

#[test]
fn test_attribute_with_integer_argument() {
    let tokens: TokenStream = quote! {
        pub struct MyStruct {
            #[adze::repeat(min = 1, max = 10)]
            field: Vec<String>,
        }
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    if let syn::Fields::Named(fields) = &item.fields {
        assert!(!fields.named[0].attrs.is_empty());
    }
}

// ── Test 12: Attributes with path arguments ────────────────────────────────

#[test]
fn test_attribute_with_path_argument() {
    let tokens: TokenStream = quote! {
        pub struct MyStruct {
            #[adze::extract(ty = some::Path)]
            field: String,
        }
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    if let syn::Fields::Named(fields) = &item.fields {
        assert!(!fields.named[0].attrs.is_empty());
    }
}

// ── Test 13: Nested generics in attributed types ────────────────────────────

#[test]
fn test_nested_generics_in_attributed_field() {
    let tokens: TokenStream = quote! {
        pub struct MyStruct<T> {
            #[adze::leaf(pattern = "test")]
            field: Vec<Option<T>>,
        }
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    assert_eq!(item.generics.params.len(), 1);
}

// ── Test 14: Roundtrip generics with attributes ────────────────────────────

#[test]
fn test_roundtrip_generics_with_attributes() {
    let tokens: TokenStream = quote! {
        pub struct MyStruct<T> {
            #[adze::leaf(pattern = "test")]
            field: Vec<Option<T>>,
        }
    };

    let parsed: ItemStruct = syn::parse2(tokens).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemStruct = syn::parse2(requoted).expect("parse 2");

    assert_eq!(parsed.generics.params.len(), reparsed.generics.params.len());
}

// ── Test 15: Pub visibility preservation ────────────────────────────────────

#[test]
fn test_pub_visibility_preservation() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        pub struct MyStruct {
            pub field: String,
        }
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    assert!(matches!(item.vis, syn::Visibility::Public(_)));
    if let syn::Fields::Named(fields) = &item.fields {
        assert!(matches!(fields.named[0].vis, syn::Visibility::Public(_)));
    }
}

// ── Test 16: Private visibility preservation ─────────────────────────────────

#[test]
fn test_private_visibility_preservation() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        struct MyStruct {
            field: String,
        }
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    assert!(matches!(item.vis, syn::Visibility::Inherited));
}

// ── Test 17: Roundtrip pub/private visibility ───────────────────────────────

#[test]
fn test_roundtrip_visibility() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        pub struct MyStruct {
            pub field: String,
        }
    };

    let parsed: ItemStruct = syn::parse2(tokens).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemStruct = syn::parse2(requoted).expect("parse 2");

    let parsed_vis_str = quote!(#parsed.vis).to_string();
    let reparsed_vis_str = quote!(#reparsed.vis).to_string();
    assert_eq!(parsed_vis_str, reparsed_vis_str);
}

// ── Test 18: Doc comments alongside adze attributes ────────────────────────

#[test]
fn test_doc_comments_with_adze_attributes() {
    let tokens: TokenStream = quote! {
        /// This is a doc comment
        #[adze::grammar]
        pub struct MyStruct {
            /// Field doc
            #[adze::leaf(pattern = "test")]
            field: String,
        }
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    assert!(!item.attrs.is_empty());
}

// ── Test 19: Roundtrip doc comments with attributes ────────────────────────

#[test]
fn test_roundtrip_doc_comments_with_attributes() {
    let tokens: TokenStream = quote! {
        /// This is a doc comment
        #[adze::grammar]
        pub struct MyStruct {
            field: String,
        }
    };

    let parsed: ItemStruct = syn::parse2(tokens).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemStruct = syn::parse2(requoted).expect("parse 2");

    assert_eq!(parsed.attrs.len(), reparsed.attrs.len());
}

// ── Test 20: Attributes on generic structs ──────────────────────────────────

#[test]
fn test_attributes_on_generic_struct() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        pub struct MyStruct<T, U> {
            field1: T,
            field2: U,
        }
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    assert_eq!(item.generics.params.len(), 2);
    assert!(has_attr(&item.attrs, "grammar"));
}

// ── Test 21: Roundtrip multiple generic parameters ──────────────────────────

#[test]
fn test_roundtrip_multiple_generics_with_attributes() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        pub struct MyStruct<T, U> {
            field1: T,
            field2: U,
        }
    };

    let parsed: ItemStruct = syn::parse2(tokens).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemStruct = syn::parse2(requoted).expect("parse 2");

    assert_eq!(parsed.generics.params.len(), reparsed.generics.params.len());
}

// ── Test 22: Empty struct preservation ──────────────────────────────────────

#[test]
fn test_empty_struct_preservation() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        pub struct MyStruct {}
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    assert!(item.fields.is_empty());
}

// ── Test 23: Roundtrip empty struct ────────────────────────────────────────

#[test]
fn test_roundtrip_empty_struct() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        pub struct MyStruct {}
    };

    let parsed: ItemStruct = syn::parse2(tokens).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemStruct = syn::parse2(requoted).expect("parse 2");

    assert!(reparsed.fields.is_empty());
}

// ── Test 24: Empty enum preservation ───────────────────────────────────────

#[test]
fn test_empty_enum_preservation() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        pub enum MyEnum {}
    };

    let item: ItemEnum = syn::parse2(tokens).expect("failed to parse enum");
    assert!(item.variants.is_empty());
}

// ── Test 25: Roundtrip empty enum ──────────────────────────────────────────

#[test]
fn test_roundtrip_empty_enum() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        pub enum MyEnum {}
    };

    let parsed: ItemEnum = syn::parse2(tokens).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemEnum = syn::parse2(requoted).expect("parse 2");

    assert!(reparsed.variants.is_empty());
}

// ── Test 26: Multiple fields with different attributes ──────────────────────

#[test]
fn test_multiple_fields_different_attributes() {
    let tokens: TokenStream = quote! {
        pub struct MyStruct {
            #[adze::leaf(pattern = "a")]
            field1: String,
            #[adze::word]
            field2: String,
            #[adze::leaf(pattern = "b")]
            field3: String,
        }
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    if let syn::Fields::Named(fields) = &item.fields {
        assert_eq!(fields.named.len(), 3);
        assert!(!fields.named[0].attrs.is_empty());
        assert!(!fields.named[1].attrs.is_empty());
        assert!(!fields.named[2].attrs.is_empty());
    }
}

// ── Test 27: Roundtrip multiple fields with different attributes ────────────

#[test]
fn test_roundtrip_multiple_fields_different_attributes() {
    let tokens: TokenStream = quote! {
        pub struct MyStruct {
            #[adze::leaf(pattern = "a")]
            field1: String,
            #[adze::word]
            field2: String,
            #[adze::leaf(pattern = "b")]
            field3: String,
        }
    };

    let parsed: ItemStruct = syn::parse2(tokens).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemStruct = syn::parse2(requoted).expect("parse 2");

    if let syn::Fields::Named(parsed_fields) = &parsed.fields
        && let syn::Fields::Named(reparsed_fields) = &reparsed.fields
    {
        assert_eq!(parsed_fields.named.len(), reparsed_fields.named.len());
        for (p, r) in parsed_fields.named.iter().zip(reparsed_fields.named.iter()) {
            assert_eq!(p.attrs.len(), r.attrs.len());
        }
    }
}

// ── Test 28: Stacked attributes (multiple on same item) ──────────────────────

#[test]
fn test_stacked_attributes_on_field() {
    let tokens: TokenStream = quote! {
        pub struct MyStruct {
            #[adze::leaf(pattern = "test")]
            #[adze::extract(ty = String)]
            field: String,
        }
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    if let syn::Fields::Named(fields) = &item.fields {
        assert_eq!(fields.named[0].attrs.len(), 2);
    }
}

// ── Test 29: Roundtrip stacked attributes ──────────────────────────────────

#[test]
fn test_roundtrip_stacked_attributes() {
    let tokens: TokenStream = quote! {
        pub struct MyStruct {
            #[adze::leaf(pattern = "test")]
            #[adze::extract(ty = String)]
            field: String,
        }
    };

    let parsed: ItemStruct = syn::parse2(tokens).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemStruct = syn::parse2(requoted).expect("parse 2");

    if let syn::Fields::Named(parsed_fields) = &parsed.fields
        && let syn::Fields::Named(reparsed_fields) = &reparsed.fields
    {
        assert_eq!(
            parsed_fields.named[0].attrs.len(),
            reparsed_fields.named[0].attrs.len()
        );
    }
}

// ── Test 30: Large number of variants ──────────────────────────────────────

#[test]
fn test_large_number_of_variants() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        pub enum MyEnum {
            Var1, Var2, Var3, Var4, Var5,
            Var6, Var7, Var8, Var9, Var10,
            Var11, Var12, Var13, Var14, Var15,
        }
    };

    let item: ItemEnum = syn::parse2(tokens).expect("failed to parse enum");
    assert_eq!(item.variants.len(), 15);
}

// ── Test 31: Roundtrip large number of variants ────────────────────────────

#[test]
fn test_roundtrip_large_number_of_variants() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        pub enum MyEnum {
            Var1, Var2, Var3, Var4, Var5,
            Var6, Var7, Var8, Var9, Var10,
            Var11, Var12, Var13, Var14, Var15,
        }
    };

    let parsed: ItemEnum = syn::parse2(tokens).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemEnum = syn::parse2(requoted).expect("parse 2");

    assert_eq!(parsed.variants.len(), reparsed.variants.len());
}

// ── Test 32: Complex type parameters in fields ─────────────────────────────

#[test]
fn test_complex_type_parameters_in_fields() {
    let tokens: TokenStream = quote! {
        pub struct MyStruct<T, U: Clone>
        where
            T: Default,
        {
            field: Vec<T>,
        }
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    assert_eq!(item.generics.params.len(), 2);
    assert!(item.generics.where_clause.is_some());
}

// ── Test 33: Roundtrip complex type parameters ────────────────────────────

#[test]
fn test_roundtrip_complex_type_parameters() {
    let tokens: TokenStream = quote! {
        pub struct MyStruct<T, U: Clone>
        where
            T: Default,
        {
            field: Vec<T>,
        }
    };

    let parsed: ItemStruct = syn::parse2(tokens).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemStruct = syn::parse2(requoted).expect("parse 2");

    assert_eq!(parsed.generics.params.len(), reparsed.generics.params.len());
    assert_eq!(
        parsed.generics.where_clause.is_some(),
        reparsed.generics.where_clause.is_some()
    );
}

// ── Test 34: Attribute with nested parentheses ────────────────────────────

#[test]
fn test_attribute_with_nested_parentheses() {
    let tokens: TokenStream = quote! {
        pub struct MyStruct {
            #[adze::option(ty = Option<String>)]
            field: Option<String>,
        }
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    if let syn::Fields::Named(fields) = &item.fields {
        assert!(!fields.named[0].attrs.is_empty());
    }
}

// ── Test 35: Roundtrip attribute with nested parentheses ─────────────────────

#[test]
fn test_roundtrip_attribute_with_nested_parentheses() {
    let tokens: TokenStream = quote! {
        pub struct MyStruct {
            #[adze::option(ty = Option<String>)]
            field: Option<String>,
        }
    };

    let parsed: ItemStruct = syn::parse2(tokens).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemStruct = syn::parse2(requoted).expect("parse 2");

    if let syn::Fields::Named(parsed_fields) = &parsed.fields
        && let syn::Fields::Named(reparsed_fields) = &reparsed.fields
    {
        assert_eq!(
            parsed_fields.named[0].attrs.len(),
            reparsed_fields.named[0].attrs.len()
        );
    }
}

// ── Test 36: Struct with lifetime parameters ──────────────────────────────

#[test]
fn test_struct_with_lifetime_parameters() {
    let tokens: TokenStream = quote! {
        pub struct MyStruct<'a> {
            field: &'a str,
        }
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    assert!(!item.generics.params.is_empty());
}

// ── Test 37: Roundtrip lifetime parameters ──────────────────────────────────

#[test]
fn test_roundtrip_lifetime_parameters() {
    let tokens: TokenStream = quote! {
        pub struct MyStruct<'a> {
            field: &'a str,
        }
    };

    let parsed: ItemStruct = syn::parse2(tokens).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemStruct = syn::parse2(requoted).expect("parse 2");

    assert_eq!(parsed.generics.params.len(), reparsed.generics.params.len());
}

// ── Test 38: Enum variant with data ────────────────────────────────────────

#[test]
fn test_enum_variant_with_data() {
    let tokens: TokenStream = quote! {
        pub enum MyEnum {
            Var1(String),
            Var2 { field: String },
            Var3,
        }
    };

    let item: ItemEnum = syn::parse2(tokens).expect("failed to parse enum");
    assert_eq!(item.variants.len(), 3);
}

// ── Test 39: Roundtrip enum variant with data ──────────────────────────────

#[test]
fn test_roundtrip_enum_variant_with_data() {
    let tokens: TokenStream = quote! {
        pub enum MyEnum {
            Var1(String),
            Var2 { field: String },
            Var3,
        }
    };

    let parsed: ItemEnum = syn::parse2(tokens).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemEnum = syn::parse2(requoted).expect("parse 2");

    assert_eq!(parsed.variants.len(), reparsed.variants.len());
}

// ── Test 40: Combined attributes with lifetimes and generics ───────────────

#[test]
fn test_combined_attributes_with_lifetimes_and_generics() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        pub struct MyStruct<'a, T>
        where
            T: Clone + 'a,
        {
            #[adze::leaf(pattern = "test")]
            field: &'a T,
        }
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    assert_eq!(item.generics.params.len(), 2);
    assert!(has_attr(&item.attrs, "grammar"));
    if let syn::Fields::Named(fields) = &item.fields {
        assert!(!fields.named[0].attrs.is_empty());
    }
}

// ── Test 41: Roundtrip combined attributes ────────────────────────────────

#[test]
fn test_roundtrip_combined_attributes_with_lifetimes_and_generics() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        pub struct MyStruct<'a, T>
        where
            T: Clone + 'a,
        {
            #[adze::leaf(pattern = "test")]
            field: &'a T,
        }
    };

    let parsed: ItemStruct = syn::parse2(tokens).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemStruct = syn::parse2(requoted).expect("parse 2");

    assert_eq!(parsed.generics.params.len(), reparsed.generics.params.len());
    assert_eq!(parsed.attrs.len(), reparsed.attrs.len());
}

// ── Test 42: Tuple struct with attributes ──────────────────────────────────

#[test]
fn test_tuple_struct_with_attributes() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        pub struct MyStruct(String, u32);
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    assert!(matches!(item.fields, syn::Fields::Unnamed(_)));
}

// ── Test 43: Roundtrip tuple struct ────────────────────────────────────────

#[test]
fn test_roundtrip_tuple_struct() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        pub struct MyStruct(String, u32);
    };

    let parsed: ItemStruct = syn::parse2(tokens).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemStruct = syn::parse2(requoted).expect("parse 2");

    assert_eq!(parsed.fields.len(), reparsed.fields.len());
}

// ── Test 44: Complex nested generics in attributes ────────────────────────

#[test]
fn test_complex_nested_generics_in_attributes() {
    let tokens: TokenStream = quote! {
        pub struct MyStruct {
            #[adze::extract(ty = Result<Vec<String>, Box<dyn std::error::Error>>)]
            field: String,
        }
    };

    let item: ItemStruct = syn::parse2(tokens).expect("failed to parse struct");
    if let syn::Fields::Named(fields) = &item.fields {
        assert!(!fields.named[0].attrs.is_empty());
    }
}

// ── Test 45: Roundtrip complex nested generics ────────────────────────────

#[test]
fn test_roundtrip_complex_nested_generics() {
    let tokens: TokenStream = quote! {
        pub struct MyStruct {
            #[adze::extract(ty = Result<Vec<String>, Box<dyn std::error::Error>>)]
            field: String,
        }
    };

    let parsed: ItemStruct = syn::parse2(tokens).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemStruct = syn::parse2(requoted).expect("parse 2");

    if let syn::Fields::Named(parsed_fields) = &parsed.fields
        && let syn::Fields::Named(reparsed_fields) = &reparsed.fields
    {
        assert_eq!(parsed_fields.named.len(), reparsed_fields.named.len());
    }
}

// ── Test 46: Parse + roundtrip comprehensive DeriveInput ─────────────────────

#[test]
fn test_comprehensive_derive_input_roundtrip() {
    let tokens: TokenStream = quote! {
        /// Documentation
        #[derive(Clone)]
        #[adze::grammar]
        pub struct ComplexStruct<'a, T: Clone>
        where
            T: Default,
        {
            /// Field doc
            #[adze::leaf(pattern = "pattern")]
            pub field1: &'a T,
            #[adze::word]
            field2: String,
        }
    };

    let parsed: DeriveInput = syn::parse2(tokens).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: DeriveInput = syn::parse2(requoted).expect("parse 2");

    assert_eq!(parsed.ident, reparsed.ident);
    assert_eq!(parsed.attrs.len(), reparsed.attrs.len());
}

// ── Test 47: Enum variant attributes preservation ──────────────────────────

#[test]
fn test_enum_variant_attributes_preservation() {
    let tokens: TokenStream = quote! {
        pub enum MyEnum {
            #[adze::word]
            Keyword,
            #[adze::leaf(pattern = "pattern")]
            #[adze::extract(ty = String)]
            Terminal,
        }
    };

    let item: ItemEnum = syn::parse2(tokens).expect("failed to parse enum");
    assert!(!item.variants[0].attrs.is_empty());
    assert!(!item.variants[1].attrs.is_empty());
}

// ── Test 48: Roundtrip enum variant attributes ────────────────────────────

#[test]
fn test_roundtrip_enum_variant_attributes() {
    let tokens: TokenStream = quote! {
        pub enum MyEnum {
            #[adze::word]
            Keyword,
            #[adze::leaf(pattern = "pattern")]
            #[adze::extract(ty = String)]
            Terminal,
        }
    };

    let parsed: ItemEnum = syn::parse2(tokens).expect("parse 1");
    let requoted: TokenStream = quote!(#parsed);
    let reparsed: ItemEnum = syn::parse2(requoted).expect("parse 2");

    assert_eq!(parsed.variants.len(), reparsed.variants.len());
    assert_eq!(
        parsed.variants[0].attrs.len(),
        reparsed.variants[0].attrs.len()
    );
    assert_eq!(
        parsed.variants[1].attrs.len(),
        reparsed.variants[1].attrs.len()
    );
}

// ── Test 49: Attribute parsing consistency ────────────────────────────────

#[test]
fn test_attribute_parsing_consistency() {
    let tokens: TokenStream = quote! {
        #[adze::grammar]
        pub struct TestStruct {
            field: String,
        }
    };

    let item1: ItemStruct = syn::parse2(tokens.clone()).expect("parse 1");
    let item2: ItemStruct = syn::parse2(tokens.clone()).expect("parse 2");

    assert_eq!(item1.attrs.len(), item2.attrs.len());
}

// ── Test 50: Multiple roundtrip cycles ────────────────────────────────────

#[test]
fn test_multiple_roundtrip_cycles() {
    let original: TokenStream = quote! {
        #[adze::grammar]
        pub struct MyStruct {
            #[adze::leaf(pattern = "test")]
            field: String,
        }
    };

    // First cycle
    let parsed1: ItemStruct = syn::parse2(original).expect("parse 1");
    let requoted1: TokenStream = quote!(#parsed1);
    let parsed2: ItemStruct = syn::parse2(requoted1).expect("parse 2");

    // Second cycle
    let requoted2: TokenStream = quote!(#parsed2);
    let parsed3: ItemStruct = syn::parse2(requoted2).expect("parse 3");

    // Verify consistency across cycles
    assert_eq!(parsed1.ident, parsed2.ident);
    assert_eq!(parsed2.ident, parsed3.ident);
    assert_eq!(parsed1.attrs.len(), parsed3.attrs.len());
}
