//! Comprehensive v2 tests for token stream generation patterns in adze-macro.
//!
//! Tests attribute parsing with proc_macro2/syn, adze attribute variants
//! (grammar, language, leaf, etc.), code generation building blocks,
//! error handling for malformed attributes, and integration with syn parsing.

use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use std::collections::HashSet;
use std::str::FromStr;
use syn::{
    Attribute, Expr, Field, Fields, GenericParam, ItemEnum, ItemImpl, ItemMod, ItemStruct, Type,
    parse_quote, parse2,
};

use adze_common::{FieldThenParams, NameValueExpr, try_extract_inner_type, wrap_leaf_type};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn ts(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

/// Parse a field from tokens by wrapping in a struct context.
fn parse_field(tokens: TokenStream) -> Field {
    let wrapped = quote::quote! { struct __Wrapper { #tokens } };
    let s: ItemStruct = parse2(wrapped).expect("failed to parse field wrapper struct");
    s.fields.into_iter().next().expect("no field found")
}

fn parse_struct(tokens: TokenStream) -> ItemStruct {
    parse2(tokens).expect("failed to parse struct")
}

fn parse_enum(tokens: TokenStream) -> ItemEnum {
    parse2(tokens).expect("failed to parse enum")
}

fn _parse_mod(tokens: TokenStream) -> ItemMod {
    parse2(tokens).expect("failed to parse module")
}

fn has_attr_named(attrs: &[Attribute], name: &str) -> bool {
    attrs
        .iter()
        .any(|a| a.path().segments.iter().any(|seg| seg.ident == name))
}

fn last_segment_is(attr: &Attribute, name: &str) -> bool {
    attr.path()
        .segments
        .last()
        .map(|seg| seg.ident == name)
        .unwrap_or(false)
}

// =============================================================================
// 1. Adze attribute parsing — grammar attribute
// =============================================================================

#[test]
fn grammar_attr_parses_string_name() {
    let module: ItemMod = parse_quote! {
        #[adze::grammar("arithmetic")]
        mod grammar {
            pub struct Expr;
        }
    };
    let grammar_attr = module
        .attrs
        .iter()
        .find(|a| last_segment_is(a, "grammar"))
        .expect("grammar attr missing");
    let name_expr: Expr = grammar_attr.parse_args().unwrap();
    let name_str = quote!(#name_expr).to_string();
    assert!(name_str.contains("arithmetic"));
}

#[test]
fn grammar_attr_with_empty_string() {
    let module: ItemMod = parse_quote! {
        #[adze::grammar("")]
        mod grammar {}
    };
    let attr = module
        .attrs
        .iter()
        .find(|a| last_segment_is(a, "grammar"))
        .unwrap();
    let name_expr: Expr = attr.parse_args().unwrap();
    assert_eq!(quote!(#name_expr).to_string(), "\"\"");
}

#[test]
fn grammar_attr_non_string_arg_still_parses_as_expr() {
    let module: ItemMod = parse_quote! {
        #[adze::grammar(42)]
        mod grammar {}
    };
    let attr = module
        .attrs
        .iter()
        .find(|a| last_segment_is(a, "grammar"))
        .unwrap();
    let expr: Expr = attr.parse_args().unwrap();
    assert_eq!(quote!(#expr).to_string(), "42");
}

#[test]
fn grammar_attr_missing_args_is_error() {
    let module: ItemMod = parse_quote! {
        #[adze::grammar]
        mod grammar {}
    };
    let attr = module
        .attrs
        .iter()
        .find(|a| last_segment_is(a, "grammar"))
        .unwrap();
    let result: Result<Expr, _> = attr.parse_args();
    assert!(result.is_err());
}

// =============================================================================
// 2. Adze attribute parsing — language attribute
// =============================================================================

#[test]
fn language_attr_on_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Root {
            field: u32,
        }
    };
    assert!(has_attr_named(&s.attrs, "language"));
}

#[test]
fn language_attr_on_enum() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Expr {
            Lit(i32),
        }
    };
    assert!(has_attr_named(&e.attrs, "language"));
}

#[test]
fn language_attr_has_adze_prefix() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Root;
    };
    let attr = &s.attrs[0];
    let first_seg = attr.path().segments.first().unwrap();
    assert_eq!(first_seg.ident, "adze");
}

// =============================================================================
// 3. Adze attribute parsing — leaf attribute
// =============================================================================

#[test]
fn leaf_attr_text_param() {
    let tokens = quote! {
        #[adze::leaf(text = "+")]
        field: ()
    };
    let field = parse_field(tokens);
    let leaf_attr = field
        .attrs
        .iter()
        .find(|a| last_segment_is(a, "leaf"))
        .unwrap();
    let nve: NameValueExpr = leaf_attr.parse_args().unwrap();
    assert_eq!(nve.path.to_string(), "text");
}

#[test]
fn leaf_attr_pattern_param() {
    let tokens = quote! {
        #[adze::leaf(pattern = r"\d+")]
        value: String
    };
    let field = parse_field(tokens);
    let leaf_attr = field
        .attrs
        .iter()
        .find(|a| last_segment_is(a, "leaf"))
        .unwrap();
    let nve: NameValueExpr = leaf_attr.parse_args().unwrap();
    assert_eq!(nve.path.to_string(), "pattern");
}

#[test]
fn leaf_attr_with_transform() {
    use syn::punctuated::Punctuated;
    let tokens = quote! {
        #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<i32>().unwrap())]
        num: i32
    };
    let field = parse_field(tokens);
    let leaf_attr = field
        .attrs
        .iter()
        .find(|a| last_segment_is(a, "leaf"))
        .unwrap();
    let params = leaf_attr
        .parse_args_with(Punctuated::<NameValueExpr, syn::Token![,]>::parse_terminated)
        .unwrap();
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].path.to_string(), "pattern");
    assert_eq!(params[1].path.to_string(), "transform");
}

#[test]
fn leaf_attr_on_unit_variant() {
    let e: ItemEnum = parse_quote! {
        enum Token {
            #[adze::leaf(text = "+")]
            Plus,
            #[adze::leaf(text = "-")]
            Minus,
        }
    };
    for variant in &e.variants {
        assert!(has_attr_named(&variant.attrs, "leaf"));
    }
}

#[test]
fn leaf_attr_empty_args_is_error() {
    let tokens = quote! {
        #[adze::leaf()]
        value: ()
    };
    let field = parse_field(tokens);
    let leaf_attr = field
        .attrs
        .iter()
        .find(|a| last_segment_is(a, "leaf"))
        .unwrap();
    let result: Result<NameValueExpr, _> = leaf_attr.parse_args();
    assert!(result.is_err());
}

// =============================================================================
// 4. Adze attribute parsing — extra attribute
// =============================================================================

#[test]
fn extra_attr_on_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Whitespace {
            _ws: (),
        }
    };
    assert!(has_attr_named(&s.attrs, "extra"));
}

#[test]
fn extra_attr_no_args() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Comment;
    };
    let attr = &s.attrs[0];
    let result: Result<Expr, _> = attr.parse_args();
    assert!(result.is_err(), "extra should have no args");
}

// =============================================================================
// 5. Adze attribute parsing — precedence attributes
// =============================================================================

#[test]
fn prec_left_attr_parses_integer() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec_left(1)]
            Add(i32, i32),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| last_segment_is(a, "prec_left"))
        .unwrap();
    let level: syn::LitInt = attr.parse_args().unwrap();
    assert_eq!(level.base10_parse::<i32>().unwrap(), 1);
}

#[test]
fn prec_right_attr_parses_integer() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec_right(2)]
            Cons(i32, i32),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| last_segment_is(a, "prec_right"))
        .unwrap();
    let level: syn::LitInt = attr.parse_args().unwrap();
    assert_eq!(level.base10_parse::<i32>().unwrap(), 2);
}

#[test]
fn prec_no_assoc_attr_parses_integer() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec(3)]
            Compare(i32, i32),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| last_segment_is(a, "prec"))
        .unwrap();
    let level: syn::LitInt = attr.parse_args().unwrap();
    assert_eq!(level.base10_parse::<i32>().unwrap(), 3);
}

// =============================================================================
// 6. Adze attribute parsing — skip attribute
// =============================================================================

#[test]
fn skip_attr_with_bool_value() {
    let tokens = quote! {
        #[adze::skip(false)]
        visited: bool
    };
    let field = parse_field(tokens);
    let attr = field
        .attrs
        .iter()
        .find(|a| last_segment_is(a, "skip"))
        .unwrap();
    let expr: Expr = attr.parse_args().unwrap();
    assert_eq!(quote!(#expr).to_string(), "false");
}

#[test]
fn skip_attr_with_default_expr() {
    let tokens = quote! {
        #[adze::skip(Default::default())]
        metadata: Vec<u8>
    };
    let field = parse_field(tokens);
    let attr = field
        .attrs
        .iter()
        .find(|a| last_segment_is(a, "skip"))
        .unwrap();
    let expr: Expr = attr.parse_args().unwrap();
    let s = quote!(#expr).to_string();
    assert!(s.contains("Default"));
}

// =============================================================================
// 7. Adze attribute parsing — repeat and delimited
// =============================================================================

#[test]
fn repeat_attr_non_empty_true() {
    let tokens = quote! {
        #[adze::repeat(non_empty = true)]
        items: Vec<Item>
    };
    let field = parse_field(tokens);
    let attr = field
        .attrs
        .iter()
        .find(|a| last_segment_is(a, "repeat"))
        .unwrap();
    let nve: NameValueExpr = attr.parse_args().unwrap();
    assert_eq!(nve.path.to_string(), "non_empty");
}

#[test]
fn repeat_attr_non_empty_false() {
    let tokens = quote! {
        #[adze::repeat(non_empty = false)]
        items: Vec<Item>
    };
    let field = parse_field(tokens);
    let attr = field
        .attrs
        .iter()
        .find(|a| last_segment_is(a, "repeat"))
        .unwrap();
    let nve: NameValueExpr = attr.parse_args().unwrap();
    assert_eq!(nve.path.to_string(), "non_empty");
}

#[test]
fn delimited_attr_with_leaf_inner() {
    let tokens = quote! {
        #[adze::delimited(
            #[adze::leaf(text = ",")]
            ()
        )]
        numbers: Vec<Number>
    };
    let field = parse_field(tokens);
    assert!(has_attr_named(&field.attrs, "delimited"));
}

// =============================================================================
// 8. Adze attribute parsing — word and external
// =============================================================================

#[test]
fn word_attr_on_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Identifier {
            name: String,
        }
    };
    assert!(has_attr_named(&s.attrs, "word"));
}

#[test]
fn external_attr_on_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::external]
        struct IndentToken;
    };
    assert!(has_attr_named(&s.attrs, "external"));
}

// =============================================================================
// 9. is_sitter_attr pattern — attribute filtering
// =============================================================================

fn is_adze_attr(attr: &Attribute) -> bool {
    attr.path()
        .segments
        .first()
        .map(|seg| seg.ident == "adze")
        .unwrap_or(false)
}

#[test]
fn identify_adze_attrs_among_mixed() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug)]
        #[adze::language]
        #[allow(dead_code)]
        pub struct MyNode {
            field: u32,
        }
    };
    let adze_count = s.attrs.iter().filter(|a| is_adze_attr(a)).count();
    let non_adze_count = s.attrs.iter().filter(|a| !is_adze_attr(a)).count();
    assert_eq!(adze_count, 1);
    assert_eq!(non_adze_count, 2);
}

#[test]
fn retain_non_adze_attrs() {
    let mut s: ItemStruct = parse_quote! {
        #[derive(Clone)]
        #[adze::language]
        #[adze::extra]
        pub struct MyNode;
    };
    s.attrs.retain(|a| !is_adze_attr(a));
    assert_eq!(s.attrs.len(), 1);
    assert!(has_attr_named(&s.attrs, "derive"));
}

#[test]
fn all_adze_variants_detected() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        #[adze::extra]
        #[adze::word]
        #[adze::external]
        pub struct MultiAttr;
    };
    assert_eq!(s.attrs.iter().filter(|a| is_adze_attr(a)).count(), 4);
}

// =============================================================================
// 10. Code generation building blocks — quote interpolation
// =============================================================================

#[test]
fn quote_struct_with_dynamic_ident() {
    let name = Ident::new("DynStruct", Span::call_site());
    let tokens = quote! {
        struct #name {
            value: u32,
        }
    };
    let s = parse_struct(tokens);
    assert_eq!(s.ident, "DynStruct");
}

#[test]
fn quote_enum_with_dynamic_variants() {
    let name = Ident::new("DynEnum", Span::call_site());
    let var1 = Ident::new("Alpha", Span::call_site());
    let var2 = Ident::new("Beta", Span::call_site());
    let tokens = quote! {
        enum #name {
            #var1(i32),
            #var2(String),
        }
    };
    let e = parse_enum(tokens);
    assert_eq!(e.ident, "DynEnum");
    assert_eq!(e.variants.len(), 2);
}

#[test]
fn quote_impl_extract_pattern() {
    let type_name = Ident::new("MyNode", Span::call_site());
    let grammar_name = "test_grammar";
    let tokens = quote! {
        impl ::adze::Extract<#type_name> for #type_name {
            type LeafFn = ();
            const GRAMMAR_NAME: &'static str = #grammar_name;
            fn extract(node: Option<()>, source: &[u8], last_idx: usize, leaf_fn: Option<&Self::LeafFn>) -> Self {
                todo!()
            }
        }
    };
    let imp: ItemImpl = parse2(tokens).unwrap();
    assert_eq!(imp.items.len(), 3);
}

#[test]
fn quote_repeated_field_expansion() {
    let field_names: Vec<Ident> = vec!["alpha", "beta", "gamma"]
        .into_iter()
        .map(|n| Ident::new(n, Span::call_site()))
        .collect();
    let field_types: Vec<Type> = vec![parse_quote!(u32), parse_quote!(String), parse_quote!(bool)];
    let tokens = quote! {
        struct Generated {
            #(#field_names: #field_types),*
        }
    };
    let s = parse_struct(tokens);
    assert_eq!(s.fields.len(), 3);
}

// =============================================================================
// 11. Token stream construction from parts
// =============================================================================

#[test]
fn manual_token_tree_struct() {
    let mut tokens = TokenStream::new();
    tokens.extend([
        TokenTree::Ident(Ident::new("struct", Span::call_site())),
        TokenTree::Ident(Ident::new("Manual", Span::call_site())),
    ]);
    let body = TokenStream::new();
    tokens.extend([TokenTree::Group(Group::new(Delimiter::Brace, body))]);
    let s = parse_struct(tokens);
    assert_eq!(s.ident, "Manual");
    assert_eq!(s.fields.len(), 0);
}

#[test]
fn token_stream_from_str_roundtrip() {
    let original = quote! { struct Roundtrip { field: u32 } };
    let s = original.to_string();
    let reparsed = TokenStream::from_str(&s).unwrap();
    let parsed = parse_struct(reparsed);
    assert_eq!(parsed.ident, "Roundtrip");
}

#[test]
fn concatenate_token_streams() {
    let part1 = quote! { struct };
    let part2 = quote! { Concat };
    let part3 = quote! { { field: i64 } };
    let mut combined = part1;
    combined.extend(part2);
    combined.extend(part3);
    let s = parse_struct(combined);
    assert_eq!(s.ident, "Concat");
    assert_eq!(s.fields.len(), 1);
}

// =============================================================================
// 12. Error handling for malformed token streams
// =============================================================================

#[test]
fn empty_token_stream_fails_struct_parse() {
    let tokens = TokenStream::new();
    let result: Result<ItemStruct, _> = parse2(tokens);
    assert!(result.is_err());
}

#[test]
fn incomplete_struct_fails_parse() {
    let tokens = quote! { struct };
    let result: Result<ItemStruct, _> = parse2(tokens);
    assert!(result.is_err());
}

#[test]
fn malformed_attr_arg_fails_nve_parse() {
    // An attribute with no `= value` part should fail to parse as NameValueExpr
    let s: ItemStruct = parse_quote! {
        struct W {
            #[adze::leaf(text)]
            field: ()
        }
    };
    let field = s.fields.iter().next().unwrap();
    let leaf_attr = field
        .attrs
        .iter()
        .find(|a| last_segment_is(a, "leaf"))
        .unwrap();
    let result: Result<NameValueExpr, _> = leaf_attr.parse_args();
    assert!(result.is_err(), "NameValueExpr requires `key = value` form");
}

#[test]
fn duplicate_attrs_still_parseable() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        #[adze::language]
        pub struct Double;
    };
    let adze_count = s
        .attrs
        .iter()
        .filter(|a| last_segment_is(a, "language"))
        .count();
    assert_eq!(adze_count, 2);
}

// =============================================================================
// 13. Module-level grammar structure parsing
// =============================================================================

#[test]
fn module_with_grammar_and_language() {
    let m: ItemMod = parse_quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(i32),
            }
        }
    };
    assert!(has_attr_named(&m.attrs, "grammar"));
    let (_, items) = m.content.unwrap();
    let has_language = items.iter().any(|item| {
        if let syn::Item::Enum(e) = item {
            has_attr_named(&e.attrs, "language")
        } else {
            false
        }
    });
    assert!(has_language);
}

#[test]
fn module_with_extra_and_language() {
    let m: ItemMod = parse_quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                value: i32,
            }

            #[adze::extra]
            struct Whitespace;
        }
    };
    let (_, items) = m.content.unwrap();
    assert_eq!(items.len(), 2);
}

#[test]
fn module_without_body_parsed() {
    let m: ItemMod = parse_quote! {
        #[adze::grammar("test")]
        mod grammar;
    };
    assert!(m.content.is_none());
}

// =============================================================================
// 14. Variant structure exploration for code generation
// =============================================================================

#[test]
fn enum_variant_field_count_unnamed() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            Binary(Box<Expr>, String, Box<Expr>),
            Unary(String, Box<Expr>),
            Literal(i32),
        }
    };
    let counts: Vec<usize> = e.variants.iter().map(|v| v.fields.len()).collect();
    assert_eq!(counts, vec![3, 2, 1]);
}

#[test]
fn enum_variant_field_count_named() {
    let e: ItemEnum = parse_quote! {
        enum Expr {
            Neg {
                op: String,
                value: Box<Expr>,
            },
        }
    };
    assert_eq!(e.variants[0].fields.len(), 2);
    assert!(matches!(e.variants[0].fields, Fields::Named(_)));
}

#[test]
fn enum_unit_variants_have_no_fields() {
    let e: ItemEnum = parse_quote! {
        enum Token {
            Plus,
            Minus,
            Star,
        }
    };
    for variant in &e.variants {
        assert!(matches!(variant.fields, Fields::Unit));
    }
}

// =============================================================================
// 15. Type extraction for code generation patterns
// =============================================================================

#[test]
fn extract_box_inner_for_recursive_type() {
    let ty: Type = parse_quote!(Box<Expr>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Expr");
}

#[test]
fn extract_vec_for_repeat_field() {
    let ty: Type = parse_quote!(Vec<Statement>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Statement");
}

#[test]
fn extract_option_for_optional_field() {
    let ty: Type = parse_quote!(Option<ElseClause>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "ElseClause");
}

#[test]
fn wrap_leaf_for_transform_pattern() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Spanned", "Box", "Option", "Vec"]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_spanned_box_option_vec_chain() {
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ts(&wrapped),
        "Vec < Option < adze :: WithLeaf < String > > >"
    );
}

// =============================================================================
// 16. NameValueExpr integration with attribute parsing
// =============================================================================

#[test]
fn name_value_from_leaf_text_attr() {
    let s: ItemStruct = parse_quote! {
        struct Tok {
            #[adze::leaf(text = "+")]
            op: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    let leaf = field
        .attrs
        .iter()
        .find(|a| last_segment_is(a, "leaf"))
        .unwrap();
    let nve: NameValueExpr = leaf.parse_args().unwrap();
    assert_eq!(nve.path.to_string(), "text");
}

#[test]
fn field_then_params_from_delimited_attr() {
    let ftp: FieldThenParams = syn::parse_str("(), text = \",\"").unwrap();
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "text");
}

// =============================================================================
// 17. Token stream equality and identity
// =============================================================================

#[test]
fn identical_quotes_produce_same_string() {
    let a = quote! { struct Same { f: u32 } };
    let b = quote! { struct Same { f: u32 } };
    assert_eq!(a.to_string(), b.to_string());
}

#[test]
fn different_quotes_produce_different_string() {
    let a = quote! { struct A; };
    let b = quote! { struct B; };
    assert_ne!(a.to_string(), b.to_string());
}

// =============================================================================
// 18. Complex grammar module structures
// =============================================================================

#[test]
fn full_grammar_module_with_all_attr_types() {
    let m: ItemMod = parse_quote! {
        #[adze::grammar("full")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<i32>().unwrap())]
                    i32
                ),
                #[adze::prec_left(1)]
                Add(Box<Expr>, Box<Expr>),
                #[adze::prec_right(2)]
                Assign(Box<Expr>, Box<Expr>),
                #[adze::prec(3)]
                Compare(Box<Expr>, Box<Expr>),
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }

            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }
        }
    };
    let (_, items) = m.content.unwrap();
    assert_eq!(items.len(), 3);
}

#[test]
fn grammar_module_with_use_statement() {
    let m: ItemMod = parse_quote! {
        #[adze::grammar("test")]
        mod grammar {
            use adze::Spanned;

            #[adze::language]
            pub struct Root {
                items: Vec<Spanned<Item>>,
            }

            pub struct Item {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    };
    let (_, items) = m.content.unwrap();
    // use + Root struct + Item struct
    assert!(items.len() >= 3);
}

// =============================================================================
// 19. Attribute path segment checking
// =============================================================================

#[test]
fn two_segment_adze_path() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        struct S;
    };
    let attr = &s.attrs[0];
    assert_eq!(attr.path().segments.len(), 2);
    assert_eq!(attr.path().segments[0].ident, "adze");
    assert_eq!(attr.path().segments[1].ident, "language");
}

#[test]
fn single_segment_derive_not_adze() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug)]
        struct S;
    };
    assert!(!is_adze_attr(&s.attrs[0]));
}

// =============================================================================
// 20. Token stream with raw identifiers (reserved keywords in 2024)
// =============================================================================

#[test]
fn raw_ident_in_struct_field() {
    let s: ItemStruct = parse_quote! {
        struct HasRaw {
            r#type: String,
            r#match: u32,
        }
    };
    assert_eq!(s.fields.len(), 2);
}

#[test]
fn wrap_leaf_type_with_raw_ident() {
    let ty: Type = parse_quote!(r#type);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < r#type >");
}

// =============================================================================
// 21. Token stream generation for Extract impl pattern
// =============================================================================

#[test]
fn generate_extract_field_call_pattern() {
    let field_name = "value";
    let leaf_type: Type = parse_quote!(String);
    let tokens = quote! {
        ::adze::__private::extract_field::<#leaf_type, _>(
            cursor, source, last_idx, #field_name, None
        )
    };
    let s = tokens.to_string();
    assert!(s.contains("extract_field"));
    assert!(s.contains("String"));
    assert!(s.contains("value"));
}

#[test]
fn generate_extract_struct_pattern() {
    let type_name = Ident::new("MyStruct", Span::call_site());
    let tokens = quote! {
        ::adze::__private::extract_struct_or_variant(node, move |cursor, last_idx| {
            #type_name {
                field: ::adze::__private::extract_field::<String, _>(
                    cursor, source, last_idx, "field", None
                )
            }
        })
    };
    let s = tokens.to_string();
    assert!(s.contains("extract_struct_or_variant"));
    assert!(s.contains("MyStruct"));
}

// =============================================================================
// 22. Visibility handling in generated code
// =============================================================================

#[test]
fn pub_struct_visibility_preserved() {
    let s: ItemStruct = parse_quote! {
        pub struct Public { field: u32 }
    };
    assert!(s.vis.to_token_stream().to_string().contains("pub"));
}

#[test]
fn pub_crate_visibility_preserved() {
    let s: ItemStruct = parse_quote! {
        pub(crate) struct CrateOnly { field: u32 }
    };
    let vis = s.vis.to_token_stream().to_string();
    assert!(vis.contains("pub") && vis.contains("crate"));
}

#[test]
fn private_visibility_empty_tokens() {
    let s: ItemStruct = parse_quote! {
        struct Private { field: u32 }
    };
    assert!(s.vis.to_token_stream().to_string().is_empty());
}

// =============================================================================
// 23. Generics in grammar types
// =============================================================================

#[test]
fn struct_with_lifetime_generic() {
    let s: ItemStruct = parse_quote! {
        struct Borrowed<'a> {
            data: &'a str,
        }
    };
    assert!(
        s.generics
            .params
            .iter()
            .any(|p| matches!(p, GenericParam::Lifetime(_)))
    );
}

#[test]
fn const_generic_in_struct() {
    let s: ItemStruct = parse_quote! {
        struct FixedArray<const N: usize> {
            data: [u8; N],
        }
    };
    assert!(
        s.generics
            .params
            .iter()
            .any(|p| matches!(p, GenericParam::Const(_)))
    );
}
