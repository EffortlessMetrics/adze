//! Comprehensive token stream and parsing tests for adze-macro.
//!
//! This module tests token stream handling and parsing edge cases using
//! syn::parse2, quote::quote!, and proc_macro2::TokenStream.
//! Each test focuses on a specific edge case or feature.

use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use std::str::FromStr;
use syn::{
    Expr, GenericParam, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, parse_quote,
    parse2,
};

// ── Helper Functions ───────────────────────────────────────────────────────

/// Parse a token stream as a single `syn::Item`.
fn _parse_item(tokens: TokenStream) -> syn::Item {
    parse2(tokens).expect("failed to parse item")
}

/// Parse a token stream as an `ItemStruct`.
fn parse_struct(tokens: TokenStream) -> ItemStruct {
    parse2(tokens).expect("failed to parse struct")
}

/// Parse a token stream as an `ItemEnum`.
fn parse_enum(tokens: TokenStream) -> ItemEnum {
    parse2(tokens).expect("failed to parse enum")
}

/// Parse a token stream as an `ItemFn`.
fn parse_fn(tokens: TokenStream) -> ItemFn {
    parse2(tokens).expect("failed to parse function")
}

/// Parse a token stream as an `ItemMod`.
fn parse_mod(tokens: TokenStream) -> ItemMod {
    parse2(tokens).expect("failed to parse module")
}

/// Parse a token stream as an `ItemTrait`.
fn parse_trait(tokens: TokenStream) -> ItemTrait {
    parse2(tokens).expect("failed to parse trait")
}

/// Parse a token stream as an `ItemImpl`.
fn parse_impl(tokens: TokenStream) -> ItemImpl {
    parse2(tokens).expect("failed to parse impl block")
}

// ── 1: Empty Token Stream ──────────────────────────────────────────────────

#[test]
fn parse_empty_token_stream_gracefully() {
    let tokens = TokenStream::new();
    // Parsing an empty token stream should fail gracefully (not panic)
    let result: Result<ItemStruct, _> = parse2(tokens);
    assert!(result.is_err(), "Empty token stream should fail to parse");
}

// ── 2: Single Token ────────────────────────────────────────────────────────

#[test]
fn parse_single_token_identifier() {
    let tokens = quote! { identifier };
    // A single identifier token can be parsed as an expression
    let result: Result<Expr, _> = parse2(tokens);
    assert!(
        result.is_ok(),
        "Single identifier token should parse as expression"
    );
}

#[test]
fn parse_single_token_literal() {
    let tokens = quote! { 42 };
    let result: Result<Expr, _> = parse2(tokens);
    assert!(
        result.is_ok(),
        "Single integer literal should parse as expression"
    );
}

// ── 3: Complex Nested Token Streams ────────────────────────────────────────

#[test]
fn parse_complex_nested_token_streams() {
    let tokens = quote! {
        struct Complex {
            field: Vec<Option<Result<String, Box<dyn std::error::Error>>>>,
        }
    };
    let s = parse_struct(tokens);
    assert_eq!(s.ident, "Complex");
    assert_eq!(s.fields.len(), 1);
}

// ── 4: All Rust Literal Types ──────────────────────────────────────────────

#[test]
fn parse_string_literal() {
    let tokens = quote! { "hello world" };
    let result: Result<Expr, _> = parse2(tokens);
    assert!(result.is_ok(), "String literal should parse");
}

#[test]
fn parse_integer_literal() {
    let tokens = quote! { 12345i64 };
    let result: Result<Expr, _> = parse2(tokens);
    assert!(result.is_ok(), "Integer literal should parse");
}

#[test]
fn parse_float_literal() {
    let tokens = quote! { 3.14159f64 };
    let result: Result<Expr, _> = parse2(tokens);
    assert!(result.is_ok(), "Float literal should parse");
}

#[test]
fn parse_char_literal() {
    let tokens = quote! { 'c' };
    let result: Result<Expr, _> = parse2(tokens);
    assert!(result.is_ok(), "Char literal should parse");
}

#[test]
fn parse_bool_literal_true() {
    let tokens = quote! { true };
    let result: Result<Expr, _> = parse2(tokens);
    assert!(result.is_ok(), "Boolean true literal should parse");
}

#[test]
fn parse_bool_literal_false() {
    let tokens = quote! { false };
    let result: Result<Expr, _> = parse2(tokens);
    assert!(result.is_ok(), "Boolean false literal should parse");
}

// ── 5: Identifiers ─────────────────────────────────────────────────────────

#[test]
fn parse_token_stream_with_identifiers() {
    let tokens = quote! {
        struct MyStruct {
            field_name: TypeName,
        }
    };
    let s = parse_struct(tokens);
    assert_eq!(s.ident, "MyStruct");
    let field = s.fields.iter().next().unwrap();
    // Check that the field has the expected identifier
    assert!(field.ident.as_ref().map(|id| id.to_string()) == Some("field_name".to_string()));
}

// ── 6: Punctuation ─────────────────────────────────────────────────────────

#[test]
fn parse_token_stream_with_punctuation() {
    let tokens = quote! {
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }
    };
    let f = parse_fn(tokens);
    assert_eq!(f.sig.ident, "add");
}

// ── 7: Groups (Braces, Brackets, Parens) ──────────────────────────────────

#[test]
fn parse_token_stream_with_brace_group() {
    let tokens = quote! {
        struct Braces {
            field: u32,
        }
    };
    let s = parse_struct(tokens);
    assert_eq!(s.ident, "Braces");
}

#[test]
fn parse_token_stream_with_bracket_group() {
    let tokens = quote! {
        struct BracketField {
            data: Vec<u32>,
        }
    };
    let s = parse_struct(tokens);
    assert_eq!(s.ident, "BracketField");
}

#[test]
fn parse_token_stream_with_paren_group() {
    let tokens = quote! {
        fn parens(a: (u32, String)) { }
    };
    let f = parse_fn(tokens);
    assert_eq!(f.sig.ident, "parens");
}

// ── 8: Comments (Should Be Stripped) ───────────────────────────────────────

#[test]
fn parse_token_stream_with_line_comments_stripped() {
    // Note: syn doesn't include comments in TokenStream (they're stripped)
    let tokens = quote! {
        struct CommentTest {
            // This is a comment
            field: u32,
        }
    };
    let s = parse_struct(tokens);
    assert_eq!(s.ident, "CommentTest");
    assert_eq!(s.fields.len(), 1);
}

#[test]
fn parse_token_stream_with_block_comments_stripped() {
    let tokens = quote! {
        struct BlockCommentTest {
            /* Block comment */ field: u32,
        }
    };
    let s = parse_struct(tokens);
    assert_eq!(s.ident, "BlockCommentTest");
}

// ── 9: Attributes ────────────────────────────────────────────────────────

#[test]
fn parse_token_stream_with_simple_attribute() {
    let tokens = quote! {
        #[derive(Debug)]
        struct AttrTest {
            field: u32,
        }
    };
    let s = parse_struct(tokens);
    assert!(
        s.attrs
            .iter()
            .any(|a| { a.path().segments.iter().any(|seg| seg.ident == "derive") })
    );
}

#[test]
fn parse_token_stream_with_multiple_attributes() {
    let tokens = quote! {
        #[derive(Debug, Clone)]
        #[allow(dead_code)]
        struct MultiAttr {
            field: u32,
        }
    };
    let s = parse_struct(tokens);
    assert_eq!(s.attrs.len(), 2);
}

// ── 10: Lifetime Annotations ───────────────────────────────────────────────

#[test]
fn parse_token_stream_with_lifetime_annotations() {
    let tokens = quote! {
        struct Borrower<'a> {
            reference: &'a str,
        }
    };
    let s = parse_struct(tokens);
    assert_eq!(s.ident, "Borrower");
    let generics = &s.generics;
    assert!(
        generics
            .params
            .iter()
            .any(|p| { matches!(p, GenericParam::Lifetime(_)) })
    );
}

// ── 11: Generic Parameters ────────────────────────────────────────────────

#[test]
fn parse_token_stream_with_generic_parameters() {
    let tokens = quote! {
        struct Container<T> {
            item: T,
        }
    };
    let s = parse_struct(tokens);
    assert_eq!(s.ident, "Container");
    assert!(
        s.generics
            .params
            .iter()
            .any(|p| { matches!(p, GenericParam::Type(_)) })
    );
}

// ── 12: Where Clauses ─────────────────────────────────────────────────────

#[test]
fn parse_token_stream_with_where_clause() {
    let tokens = quote! {
        fn generic<T>(item: T) where T: std::fmt::Debug { }
    };
    let f = parse_fn(tokens);
    assert!(f.sig.generics.where_clause.is_some());
}

// ── 13: Impl Blocks ────────────────────────────────────────────────────────

#[test]
fn parse_token_stream_with_impl_block() {
    let tokens = quote! {
        impl MyStruct {
            fn new() -> Self {
                MyStruct { }
            }
        }
    };
    let i = parse_impl(tokens);
    assert_eq!(i.items.len(), 1);
}

// ── 14: Trait Bounds ────────────────────────────────────────────────────────

#[test]
fn parse_token_stream_with_trait_bounds() {
    let tokens = quote! {
        fn constrained<T: Clone + Debug>(item: T) { }
    };
    let f = parse_fn(tokens);
    let gen_params = &f.sig.generics.params;
    assert!(!gen_params.is_empty());
}

// ── 15: Associated Types ───────────────────────────────────────────────────

#[test]
fn parse_token_stream_with_associated_types() {
    let tokens = quote! {
        trait HasAssociated {
            type Item;
            fn get_item(&self) -> Self::Item;
        }
    };
    let t = parse_trait(tokens);
    assert!(
        t.items
            .iter()
            .any(|item| { matches!(item, syn::TraitItem::Type(_)) })
    );
}

// ── 16: Closures ─────────────────────────────────────────────────────────

#[test]
fn parse_token_stream_with_closures() {
    let tokens = quote! {
        fn apply_closure() {
            let closure = |x: i32| -> i32 { x + 1 };
            let result = closure(5);
        }
    };
    let result: Result<syn::ItemFn, _> = parse2(tokens);
    assert!(result.is_ok(), "Function with closure should parse");
}

// ── 17: Token Stream Roundtrip (to_string → parse) ────────────────────────

#[test]
fn token_stream_roundtrip() {
    let original = quote! {
        struct Original {
            field: u32,
        }
    };
    let roundtrip = original.to_string();
    let reparsed: TokenStream = TokenStream::from_str(&roundtrip).expect("Failed to reparse");
    let s = parse_struct(reparsed);
    assert_eq!(s.ident, "Original");
}

// ── 18: Token Stream Equality Comparison ───────────────────────────────────

#[test]
fn token_stream_equality_comparison() {
    let ts1 = quote! { struct Eq { field: u32 } };
    let ts2 = quote! { struct Eq { field: u32 } };
    assert_eq!(ts1.to_string(), ts2.to_string());
}

// ── 19: Token Stream Concatenation ────────────────────────────────────────

#[test]
fn token_stream_concatenation() {
    let part1 = quote! { struct Concat };
    let part2 = quote! { { field: u32, } };
    let mut combined = part1;
    combined.extend(part2);
    let reparsed: TokenStream = TokenStream::from_str(&combined.to_string()).expect("reparse");
    // Just verify it parses without panic
    let _ = parse_struct(reparsed);
}

// ── 20: Token Stream with Doc Comments ────────────────────────────────────

#[test]
fn parse_token_stream_with_doc_comments() {
    let tokens = quote! {
        /// This is a doc comment
        struct Documented {
            field: u32,
        }
    };
    let s = parse_struct(tokens);
    assert_eq!(s.ident, "Documented");
    // Doc comments become attributes
    assert!(s.attrs.iter().any(|a| a.style == syn::AttrStyle::Outer));
}

// ── 21: Parse Struct Definition Token Stream ──────────────────────────────

#[test]
fn parse_struct_definition_token_stream() {
    let tokens = quote! {
        pub struct Person {
            name: String,
            age: u32,
            email: Option<String>,
        }
    };
    let s = parse_struct(tokens);
    assert_eq!(s.ident, "Person");
    assert_eq!(s.fields.len(), 3);
    assert!(s.vis.to_token_stream().to_string().contains("pub"));
}

// ── 22: Parse Enum Definition Token Stream ────────────────────────────────

#[test]
fn parse_enum_definition_token_stream() {
    let tokens = quote! {
        enum Result<T, E> {
            Ok(T),
            Err(E),
        }
    };
    let e = parse_enum(tokens);
    assert_eq!(e.ident, "Result");
    assert_eq!(e.variants.len(), 2);
}

// ── 23: Parse Function Definition Token Stream ─────────────────────────────

#[test]
fn parse_function_definition_token_stream() {
    let tokens = quote! {
        pub async fn fetch(url: &str) -> Result<String, std::io::Error> {
            Ok(String::new())
        }
    };
    let f = parse_fn(tokens);
    assert_eq!(f.sig.ident, "fetch");
    assert!(f.sig.asyncness.is_some());
}

// ── 24: Parse Module Definition Token Stream ──────────────────────────────

#[test]
fn parse_module_definition_token_stream() {
    let tokens = quote! {
        pub mod utils {
            pub fn helper() { }
        }
    };
    let m = parse_mod(tokens);
    assert_eq!(m.ident, "utils");
}

// ── 25: Token Stream with Raw Identifiers (r#keyword) ──────────────────────

#[test]
fn parse_token_stream_with_raw_identifiers() {
    let tokens = quote! {
        struct RawIdents {
            r#type: u32,
            r#match: String,
        }
    };
    let s = parse_struct(tokens);
    assert_eq!(s.ident, "RawIdents");
    assert_eq!(s.fields.len(), 2);
}

// ── 26: Token Stream with Byte Strings ────────────────────────────────────

#[test]
fn parse_token_stream_with_byte_strings() {
    let tokens = quote! {
        const BYTES: &[u8] = b"hello";
    };
    let result: Result<syn::ItemConst, _> = parse2(tokens);
    assert!(result.is_ok(), "Byte string constant should parse");
}

// ── 27: Token Stream with Raw Strings ─────────────────────────────────────

#[test]
fn parse_token_stream_with_raw_strings() {
    let tokens = quote! {
        const RAW: &str = r#"raw "quoted" string"#;
    };
    let result: Result<syn::ItemConst, _> = parse2(tokens);
    assert!(result.is_ok(), "Raw string constant should parse");
}

// ── 28: Verify quote! macro produces expected output ──────────────────────

#[test]
fn verify_quote_macro_produces_expected_output() {
    let ident = Ident::new("MyType", proc_macro2::Span::call_site());
    let tokens = quote! { struct #ident { field: u32 } };
    let s = parse_struct(tokens);
    assert_eq!(s.ident, ident);
}

// ── 29: Verify parse_quote! produces expected types ──────────────────────

#[test]
fn verify_parse_quote_produces_expected_types() {
    let s: ItemStruct = parse_quote! {
        struct Parsed {
            data: Vec<u32>,
        }
    };
    assert_eq!(s.ident, "Parsed");
    assert_eq!(s.fields.len(), 1);
}

// ── 30: Token Stream with Nested Attributes ───────────────────────────────

#[test]
fn parse_token_stream_with_nested_attributes() {
    let tokens = quote! {
        #[outer(inner("value"))]
        struct NestedAttrs {
            #[field_attr]
            field: u32,
        }
    };
    let s = parse_struct(tokens);
    assert_eq!(s.attrs.len(), 1);
    // Check outer attribute exists
    assert!(
        s.attrs
            .iter()
            .any(|a| { a.path().segments.iter().any(|seg| seg.ident == "outer") })
    );
}

// ── 31: Token Stream with Complex Type Paths ──────────────────────────────

#[test]
fn parse_token_stream_with_complex_type_paths() {
    let tokens = quote! {
        struct TypePaths {
            a: std::collections::HashMap<String, Vec<Box<dyn Fn() -> u32>>>,
            b: <T as Trait>::AssocType,
        }
    };
    let s = parse_struct(tokens);
    assert_eq!(s.fields.len(), 2);
}

// ── 32: Token Stream with Visibility Modifiers ────────────────────────────

#[test]
fn parse_token_stream_with_visibility_modifiers() {
    let tokens = quote! {
        pub struct Public {
            pub field: u32,
            pub(crate) limited: String,
            private: bool,
        }
    };
    let s = parse_struct(tokens);
    assert_eq!(s.fields.len(), 3);
    // First field is public
    let first = s.fields.iter().next().unwrap();
    assert!(first.vis.to_token_stream().to_string().contains("pub"));
}

// ── 33: Token Stream with Default Trait Impl ──────────────────────────────

#[test]
fn parse_token_stream_with_default_trait_impl() {
    let tokens = quote! {
        impl Default for MyStruct {
            fn default() -> Self {
                MyStruct { field: 0 }
            }
        }
    };
    let i = parse_impl(tokens);
    assert!(!i.items.is_empty());
}

// ── 34: Token Stream with Macro Invocations ────────────────────────────────

#[test]
fn parse_token_stream_with_macro_invocations() {
    let tokens = quote! {
        struct WithMacro {
            field: vec![1, 2, 3],
        }
    };
    // The macro call is preserved in the token stream
    let s = parse_struct(tokens);
    assert_eq!(s.ident, "WithMacro");
}

// ── 35: Token Stream with Const Generics ──────────────────────────────────

#[test]
fn parse_token_stream_with_const_generics() {
    let tokens = quote! {
        struct Array<const N: usize> {
            data: [u32; N],
        }
    };
    let s = parse_struct(tokens);
    assert_eq!(s.ident, "Array");
    assert!(
        s.generics
            .params
            .iter()
            .any(|p| { matches!(p, GenericParam::Const(_)) })
    );
}

// ── 36: Verify TokenTree Construction ──────────────────────────────────────

#[test]
fn verify_token_tree_construction() {
    // Manually build a token stream from TokenTree elements
    let mut tokens = TokenStream::new();
    tokens.extend(std::iter::once(TokenTree::Ident(Ident::new(
        "struct",
        proc_macro2::Span::call_site(),
    ))));
    tokens.extend(std::iter::once(TokenTree::Ident(Ident::new(
        "Manual",
        proc_macro2::Span::call_site(),
    ))));

    let mut group = TokenStream::new();
    group.extend(std::iter::once(TokenTree::Ident(Ident::new(
        "field",
        proc_macro2::Span::call_site(),
    ))));
    group.extend(std::iter::once(TokenTree::Punct(Punct::new(
        ':',
        Spacing::Alone,
    ))));
    group.extend(std::iter::once(TokenTree::Ident(Ident::new(
        "u32",
        proc_macro2::Span::call_site(),
    ))));

    tokens.extend(std::iter::once(TokenTree::Group(Group::new(
        Delimiter::Brace,
        group,
    ))));

    let s = parse_struct(tokens);
    assert_eq!(s.ident, "Manual");
}

// ── 37: Parse Token Stream with Self Type ─────────────────────────────────

#[test]
fn parse_token_stream_with_self_type() {
    let tokens = quote! {
        impl Display for MyType {
            fn fmt(&self, f: &mut Formatter) -> Result {
                Ok(())
            }
        }
    };
    let i = parse_impl(tokens);
    assert!(!i.items.is_empty());
}

// ── 38: Mutable and Reference Token Streams ────────────────────────────────

#[test]
fn parse_token_stream_with_mutable_references() {
    let tokens = quote! {
        struct MutRef {
            mutable: &mut String,
            reference: &u32,
        }
    };
    let s = parse_struct(tokens);
    assert_eq!(s.fields.len(), 2);
}

// ── 39: Verify Token Stream Preservation Across Macro Expansion ────────────

#[test]
fn token_stream_preservation_across_macro_expansion() {
    // Original token stream
    let original: TokenStream = quote! {
        struct Preserved {
            field: u32,
        }
    };

    // Simulate roundtrip through proc macro
    let string_repr = original.to_string();
    let reconstructed = TokenStream::from_str(&string_repr).expect("reparse");

    // Parse reconstructed
    let s = parse_struct(reconstructed);
    assert_eq!(s.ident, "Preserved");
}

// ── 40: Complex Expression Token Streams ───────────────────────────────────

#[test]
fn parse_complex_expression_token_streams() {
    let tokens = quote! {
        (a + b) * c - d / e % f & g | h ^ i
    };
    let result: Result<Expr, _> = parse2(tokens);
    assert!(result.is_ok(), "Complex expression should parse");
}
