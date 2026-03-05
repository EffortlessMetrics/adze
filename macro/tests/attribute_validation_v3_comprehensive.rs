//! Comprehensive v3 attribute validation tests for adze-macro.
//!
//! Since proc-macro attributes can only be invoked at compile time, these tests
//! exercise the supporting logic: syn-based parsing, attribute name validation,
//! token stream construction, struct/enum field patterns, attribute combinations,
//! and error patterns for invalid inputs.

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Attribute, DeriveInput, Expr, Fields, Ident, ItemEnum, ItemMod, ItemStruct, Type, parse_quote,
    parse2,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

/// All 12 adze proc-macro attribute names.
const ADZE_ATTRS: &[&str] = &[
    "grammar",
    "language",
    "leaf",
    "skip",
    "prec",
    "prec_left",
    "prec_right",
    "delimited",
    "repeat",
    "extra",
    "external",
    "word",
];

fn last_segment(attr: &Attribute) -> Option<String> {
    attr.path().segments.last().map(|seg| seg.ident.to_string())
}

fn is_adze_path(attr: &Attribute, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == name
}

fn collect_adze_names(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter_map(|a| {
            let segs: Vec<_> = a.path().segments.iter().collect();
            if segs.len() == 2 && segs[0].ident == "adze" {
                Some(segs[1].ident.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn type_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// =============================================================================
// 1. Valid adze attribute names and values (10 tests)
// =============================================================================

#[test]
fn valid_attr_grammar_with_string_arg() {
    let module: ItemMod = parse_quote! {
        #[adze::grammar("my_lang")]
        mod grammar {}
    };
    assert!(module.attrs.iter().any(|a| is_adze_path(a, "grammar")));
    let attr = module
        .attrs
        .iter()
        .find(|a| is_adze_path(a, "grammar"))
        .unwrap();
    let name_expr: Expr = attr.parse_args().unwrap();
    let rendered = quote!(#name_expr).to_string();
    assert!(rendered.contains("my_lang"));
}

#[test]
fn valid_attr_language_no_args() {
    let item: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Program;
    };
    assert!(item.attrs.iter().any(|a| is_adze_path(a, "language")));
}

#[test]
fn valid_attr_leaf_text() {
    let item: ItemStruct = parse_quote! {
        #[adze::leaf(text = "+")]
        struct Plus;
    };
    let attr = item.attrs.iter().find(|a| is_adze_path(a, "leaf")).unwrap();
    let args: TokenStream = attr.parse_args().unwrap();
    let rendered = args.to_string();
    assert!(rendered.contains("text"));
    assert!(rendered.contains('+'));
}

#[test]
fn valid_attr_leaf_pattern() {
    let item: ItemStruct = parse_quote! {
        #[adze::leaf(pattern = r"\d+")]
        struct Number;
    };
    let attr = item.attrs.iter().find(|a| is_adze_path(a, "leaf")).unwrap();
    let args: TokenStream = attr.parse_args().unwrap();
    assert!(args.to_string().contains("pattern"));
}

#[test]
fn valid_attr_skip_with_default_value() {
    let item: DeriveInput = parse_quote! {
        #[adze::skip(false)]
        struct Dummy { visited: bool }
    };
    assert!(item.attrs.iter().any(|a| is_adze_path(a, "skip")));
}

#[test]
fn valid_attr_prec_with_level() {
    let item: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec(1)]
            Variant(u32),
        }
    };
    let variant = &item.variants[0];
    assert!(variant.attrs.iter().any(|a| is_adze_path(a, "prec")));
    let attr = variant
        .attrs
        .iter()
        .find(|a| is_adze_path(a, "prec"))
        .unwrap();
    let level: Expr = attr.parse_args().unwrap();
    let rendered = quote!(#level).to_string();
    assert_eq!(rendered, "1");
}

#[test]
fn valid_attr_prec_left_with_level() {
    let item: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec_left(2)]
            Add(u32),
        }
    };
    let variant = &item.variants[0];
    assert!(variant.attrs.iter().any(|a| is_adze_path(a, "prec_left")));
}

#[test]
fn valid_attr_prec_right_with_level() {
    let item: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec_right(3)]
            Cons(u32),
        }
    };
    let variant = &item.variants[0];
    assert!(variant.attrs.iter().any(|a| is_adze_path(a, "prec_right")));
}

#[test]
fn valid_attr_repeat_non_empty() {
    let item: ItemStruct = parse_quote! {
        struct List {
            #[adze::repeat(non_empty = true)]
            items: Vec<u32>,
        }
    };
    let field = item.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_path(a, "repeat"))
        .unwrap();
    let args: TokenStream = attr.parse_args().unwrap();
    assert!(args.to_string().contains("non_empty"));
}

#[test]
fn valid_attr_extra_and_word() {
    let item: ItemStruct = parse_quote! {
        #[adze::extra]
        #[adze::word]
        struct Token;
    };
    let names = collect_adze_names(&item.attrs);
    assert!(names.contains(&"extra".to_string()));
    assert!(names.contains(&"word".to_string()));
}

// =============================================================================
// 2. Type parsing with syn (10 tests)
// =============================================================================

#[test]
fn parse_simple_type() {
    let ty: Type = parse_quote!(u32);
    assert_eq!(type_str(&ty), "u32");
}

#[test]
fn parse_string_type() {
    let ty: Type = parse_quote!(String);
    assert_eq!(type_str(&ty), "String");
}

#[test]
fn parse_option_type() {
    let ty: Type = parse_quote!(Option<u32>);
    let rendered = type_str(&ty);
    assert!(rendered.contains("Option"));
    assert!(rendered.contains("u32"));
}

#[test]
fn parse_vec_type() {
    let ty: Type = parse_quote!(Vec<String>);
    let rendered = type_str(&ty);
    assert!(rendered.contains("Vec"));
    assert!(rendered.contains("String"));
}

#[test]
fn parse_box_type() {
    let ty: Type = parse_quote!(Box<Expr>);
    let rendered = type_str(&ty);
    assert!(rendered.contains("Box"));
    assert!(rendered.contains("Expr"));
}

#[test]
fn parse_nested_generic_type() {
    let ty: Type = parse_quote!(Vec<Option<Box<Expr>>>);
    let rendered = type_str(&ty);
    assert!(rendered.contains("Vec"));
    assert!(rendered.contains("Option"));
    assert!(rendered.contains("Box"));
    assert!(rendered.contains("Expr"));
}

#[test]
fn parse_tuple_type() {
    let ty: Type = parse_quote!((u32, String));
    let rendered = type_str(&ty);
    assert!(rendered.contains("u32"));
    assert!(rendered.contains("String"));
}

#[test]
fn parse_unit_type() {
    let ty: Type = parse_quote!(());
    assert_eq!(type_str(&ty), "()");
}

#[test]
fn parse_reference_type() {
    let ty: Type = parse_quote!(&str);
    let rendered = type_str(&ty);
    assert!(rendered.contains("str"));
}

#[test]
fn parse_path_type_with_module() {
    let ty: Type = parse_quote!(std::collections::HashMap<String, u32>);
    let rendered = type_str(&ty);
    assert!(rendered.contains("HashMap"));
    assert!(rendered.contains("String"));
    assert!(rendered.contains("u32"));
}

// =============================================================================
// 3. Struct/enum field patterns (10 tests)
// =============================================================================

#[test]
fn struct_named_fields_count() {
    let item: ItemStruct = parse_quote! {
        struct Node {
            left: Box<Node>,
            right: Box<Node>,
            value: u32,
        }
    };
    if let Fields::Named(fields) = &item.fields {
        assert_eq!(fields.named.len(), 3);
    } else {
        panic!("Expected named fields");
    }
}

#[test]
fn struct_unnamed_fields_count() {
    let item: ItemStruct = parse_quote! {
        struct Pair(u32, String);
    };
    if let Fields::Unnamed(fields) = &item.fields {
        assert_eq!(fields.unnamed.len(), 2);
    } else {
        panic!("Expected unnamed fields");
    }
}

#[test]
fn struct_unit_has_no_fields() {
    let item: ItemStruct = parse_quote! {
        struct Marker;
    };
    assert!(matches!(item.fields, Fields::Unit));
}

#[test]
fn enum_variant_count() {
    let item: ItemEnum = parse_quote! {
        enum Token {
            Number(u32),
            Plus,
            Minus,
            Star,
        }
    };
    assert_eq!(item.variants.len(), 4);
}

#[test]
fn enum_variant_with_named_fields() {
    let item: ItemEnum = parse_quote! {
        enum Ast {
            BinOp { left: Box<Ast>, op: String, right: Box<Ast> },
        }
    };
    let variant = &item.variants[0];
    if let Fields::Named(fields) = &variant.fields {
        assert_eq!(fields.named.len(), 3);
    } else {
        panic!("Expected named fields in variant");
    }
}

#[test]
fn enum_variant_unnamed_fields() {
    let item: ItemEnum = parse_quote! {
        enum Expr {
            Add(Box<Expr>, Box<Expr>),
        }
    };
    let variant = &item.variants[0];
    if let Fields::Unnamed(fields) = &variant.fields {
        assert_eq!(fields.unnamed.len(), 2);
    } else {
        panic!("Expected unnamed fields in variant");
    }
}

#[test]
fn field_type_is_option() {
    let item: ItemStruct = parse_quote! {
        struct Node {
            child: Option<Box<Node>>,
        }
    };
    let field = item.fields.iter().next().unwrap();
    let rendered = type_str(&field.ty);
    assert!(rendered.contains("Option"));
}

#[test]
fn field_type_is_vec() {
    let item: ItemStruct = parse_quote! {
        struct List {
            items: Vec<Item>,
        }
    };
    let field = item.fields.iter().next().unwrap();
    let rendered = type_str(&field.ty);
    assert!(rendered.contains("Vec"));
}

#[test]
fn field_with_adze_leaf_attribute() {
    let item: ItemStruct = parse_quote! {
        struct Number {
            #[adze::leaf(pattern = r"\d+")]
            value: String,
        }
    };
    let field = item.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_path(a, "leaf")));
}

#[test]
fn derive_input_struct_ident() {
    let item: DeriveInput = parse_quote! {
        struct MyParser {
            field: u32,
        }
    };
    assert_eq!(item.ident, "MyParser");
}

// =============================================================================
// 4. Token stream generation patterns (10 tests)
// =============================================================================

#[test]
fn quote_generates_struct_definition() {
    let name = Ident::new("Foo", Span::call_site());
    let tokens = quote! {
        struct #name {
            value: u32,
        }
    };
    let item: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(item.ident, "Foo");
}

#[test]
fn quote_generates_enum_definition() {
    let name = Ident::new("Expr", Span::call_site());
    let tokens = quote! {
        enum #name {
            Add,
            Sub,
            Mul,
        }
    };
    let item: ItemEnum = parse2(tokens).unwrap();
    assert_eq!(item.ident, "Expr");
    assert_eq!(item.variants.len(), 3);
}

#[test]
fn quote_generates_module() {
    let tokens = quote! {
        mod inner {
            pub struct Node;
        }
    };
    let module: ItemMod = parse2(tokens).unwrap();
    assert_eq!(module.ident, "inner");
}

#[test]
fn quote_interpolates_type() {
    let ty: Type = parse_quote!(Vec<u32>);
    let tokens = quote! {
        fn items() -> #ty {
            Vec::new()
        }
    };
    let rendered = tokens.to_string();
    assert!(rendered.contains("Vec"));
    assert!(rendered.contains("u32"));
}

#[test]
fn quote_interpolates_ident_in_attribute() {
    let attr_name = Ident::new("language", Span::call_site());
    let tokens = quote! {
        #[adze::#attr_name]
        struct Entry;
    };
    let item: ItemStruct = parse2(tokens).unwrap();
    assert!(
        item.attrs
            .iter()
            .any(|a| last_segment(a) == Some("language".to_string()))
    );
}

#[test]
fn quote_generates_impl_block() {
    let struct_name = Ident::new("Parser", Span::call_site());
    let tokens = quote! {
        impl #struct_name {
            fn new() -> Self {
                Self
            }
        }
    };
    let rendered = tokens.to_string();
    assert!(rendered.contains("Parser"));
    assert!(rendered.contains("new"));
}

#[test]
fn token_stream_from_string() {
    let ts: TokenStream = "struct Foo;".parse().unwrap();
    let item: ItemStruct = parse2(ts).unwrap();
    assert_eq!(item.ident, "Foo");
}

#[test]
fn token_stream_is_empty_check() {
    let empty = TokenStream::new();
    assert!(empty.is_empty());
    let non_empty = quote!(
        struct Bar;
    );
    assert!(!non_empty.is_empty());
}

#[test]
fn quote_repetition_pattern() {
    let field_names: Vec<Ident> = vec![
        Ident::new("alpha", Span::call_site()),
        Ident::new("beta", Span::call_site()),
        Ident::new("gamma", Span::call_site()),
    ];
    let tokens = quote! {
        struct Data {
            #(#field_names: u32),*
        }
    };
    let item: ItemStruct = parse2(tokens).unwrap();
    if let Fields::Named(fields) = &item.fields {
        assert_eq!(fields.named.len(), 3);
        let names: Vec<String> = fields
            .named
            .iter()
            .map(|f| f.ident.as_ref().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["alpha", "beta", "gamma"]);
    } else {
        panic!("Expected named fields");
    }
}

#[test]
fn quote_conditional_attribute() {
    let add_language = true;
    let attr_tokens = if add_language {
        quote!(#[adze::language])
    } else {
        quote!()
    };
    let tokens = quote! {
        #attr_tokens
        struct Root;
    };
    let item: ItemStruct = parse2(tokens).unwrap();
    assert!(
        item.attrs
            .iter()
            .any(|a| last_segment(a) == Some("language".to_string()))
    );
}

// =============================================================================
// 5. Attribute combination validation (8 tests)
// =============================================================================

#[test]
fn language_plus_extra_on_struct() {
    let item: ItemStruct = parse_quote! {
        #[adze::language]
        #[adze::extra]
        struct Root;
    };
    let names = collect_adze_names(&item.attrs);
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"language".to_string()));
    assert!(names.contains(&"extra".to_string()));
}

#[test]
fn leaf_plus_skip_on_field() {
    let item: ItemStruct = parse_quote! {
        struct Node {
            #[adze::leaf(text = "x")]
            #[adze::skip(false)]
            value: (),
        }
    };
    let field = item.fields.iter().next().unwrap();
    let names = collect_adze_names(&field.attrs);
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"leaf".to_string()));
    assert!(names.contains(&"skip".to_string()));
}

#[test]
fn prec_left_plus_leaf_on_variant() {
    let item: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec_left(1)]
            #[adze::leaf(text = "+")]
            Add,
        }
    };
    let variant = &item.variants[0];
    let names = collect_adze_names(&variant.attrs);
    assert!(names.contains(&"prec_left".to_string()));
    assert!(names.contains(&"leaf".to_string()));
}

#[test]
fn repeat_plus_delimited_on_field() {
    let item: ItemStruct = parse_quote! {
        struct List {
            #[adze::repeat(non_empty = true)]
            #[adze::delimited(())]
            items: Vec<u32>,
        }
    };
    let field = item.fields.iter().next().unwrap();
    let names = collect_adze_names(&field.attrs);
    assert!(names.contains(&"repeat".to_string()));
    assert!(names.contains(&"delimited".to_string()));
}

#[test]
fn grammar_with_language_inside_module() {
    let module: ItemMod = parse_quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root;
        }
    };
    assert!(module.attrs.iter().any(|a| is_adze_path(a, "grammar")));
    let (_, items) = module.content.unwrap();
    let struct_item = items.iter().find_map(|item| {
        if let syn::Item::Struct(s) = item {
            Some(s)
        } else {
            None
        }
    });
    assert!(struct_item.is_some());
    assert!(
        struct_item
            .unwrap()
            .attrs
            .iter()
            .any(|a| is_adze_path(a, "language"))
    );
}

#[test]
fn external_plus_word_on_struct() {
    let item: ItemStruct = parse_quote! {
        #[adze::external]
        #[adze::word]
        struct Identifier;
    };
    let names = collect_adze_names(&item.attrs);
    assert!(names.contains(&"external".to_string()));
    assert!(names.contains(&"word".to_string()));
}

#[test]
fn all_precedence_variants_parseable() {
    let item: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec(1)]
            Plain(u32),
            #[adze::prec_left(2)]
            Left(u32),
            #[adze::prec_right(3)]
            Right(u32),
        }
    };
    let prec_names: Vec<String> = item
        .variants
        .iter()
        .flat_map(|v| collect_adze_names(&v.attrs))
        .collect();
    assert_eq!(prec_names, vec!["prec", "prec_left", "prec_right"]);
}

#[test]
fn module_with_extra_and_language_structs() {
    let module: ItemMod = parse_quote! {
        #[adze::grammar("example")]
        mod grammar {
            #[adze::language]
            pub struct Program;

            #[adze::extra]
            pub struct Whitespace;
        }
    };
    let (_, items) = module.content.unwrap();
    let structs: Vec<&ItemStruct> = items
        .iter()
        .filter_map(|item| {
            if let syn::Item::Struct(s) = item {
                Some(s)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(structs.len(), 2);
    assert!(structs[0].attrs.iter().any(|a| is_adze_path(a, "language")));
    assert!(structs[1].attrs.iter().any(|a| is_adze_path(a, "extra")));
}

// =============================================================================
// 6. Error patterns for invalid inputs (7 tests)
// =============================================================================

#[test]
fn parse_invalid_type_fails() {
    let tokens = quote!(123notaType);
    let result = parse2::<Type>(tokens);
    assert!(result.is_err());
}

#[test]
fn parse_empty_struct_body_is_valid() {
    // Empty braces are valid in Rust
    let tokens = quote!(
        struct Empty {}
    );
    let result = parse2::<ItemStruct>(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();
    assert_eq!(item.fields.len(), 0);
}

#[test]
fn parse_malformed_attribute_args_detected() {
    // Attribute with unparseable args — the attribute itself is parsed fine by syn,
    // but extracting a typed value from it fails
    let item: ItemStruct = parse_quote! {
        #[adze::leaf(not_a_valid = )]
        struct Bad;
    };
    let attr = item.attrs.iter().find(|a| is_adze_path(a, "leaf")).unwrap();
    let result = attr.parse_args::<Expr>();
    assert!(result.is_err());
}

#[test]
fn parse_missing_module_content_detected() {
    // Module without body parses but content is None
    let module: ItemMod = parse_quote! {
        mod declared;
    };
    assert!(module.content.is_none());
}

#[test]
fn parse_duplicate_ident_detection() {
    let item: ItemEnum = parse_quote! {
        enum Dup {
            A,
            B,
            A,
        }
    };
    // syn parses this fine; it's a semantic error, not syntax
    let names: Vec<String> = item.variants.iter().map(|v| v.ident.to_string()).collect();
    let unique: std::collections::HashSet<&String> = names.iter().collect();
    assert!(names.len() > unique.len(), "Duplicate variants detected");
}

#[test]
fn empty_token_stream_is_not_valid_struct() {
    let tokens = TokenStream::new();
    let result = parse2::<ItemStruct>(tokens);
    assert!(result.is_err());
}

#[test]
fn non_adze_attributes_are_filtered() {
    let item: ItemStruct = parse_quote! {
        #[derive(Debug)]
        #[serde(rename_all = "camelCase")]
        #[adze::language]
        struct Node;
    };
    let adze_names = collect_adze_names(&item.attrs);
    assert_eq!(adze_names.len(), 1);
    assert_eq!(adze_names[0], "language");
    // Total attributes include non-adze ones
    assert_eq!(item.attrs.len(), 3);
}

// =============================================================================
// Additional tests to reach 55+ (supplementary coverage)
// =============================================================================

#[test]
fn all_known_attr_names_are_valid_idents() {
    for name in ADZE_ATTRS {
        let ident = Ident::new(name, Span::call_site());
        assert_eq!(ident.to_string(), *name);
    }
}

#[test]
fn attr_name_count_matches_expected() {
    assert_eq!(ADZE_ATTRS.len(), 12);
}

#[test]
fn derive_input_enum_ident() {
    let item: DeriveInput = parse_quote! {
        enum TokenKind {
            Num,
            Op,
        }
    };
    assert_eq!(item.ident, "TokenKind");
}

#[test]
fn derive_input_generics_empty() {
    let item: DeriveInput = parse_quote! {
        struct Simple;
    };
    assert!(item.generics.params.is_empty());
}

#[test]
fn derive_input_with_lifetime() {
    let item: DeriveInput = parse_quote! {
        struct Borrowed<'a> {
            data: &'a str,
        }
    };
    assert_eq!(item.generics.params.len(), 1);
}

#[test]
fn derive_input_with_type_param() {
    let item: DeriveInput = parse_quote! {
        struct Wrapper<T> {
            inner: T,
        }
    };
    assert_eq!(item.generics.params.len(), 1);
}

#[test]
fn leaf_text_value_extraction() {
    let item: ItemStruct = parse_quote! {
        #[adze::leaf(text = "+=")]
        struct PlusEq;
    };
    let attr = item.attrs.iter().find(|a| is_adze_path(a, "leaf")).unwrap();
    let args: TokenStream = attr.parse_args().unwrap();
    let rendered = args.to_string();
    assert!(rendered.contains("text"));
    assert!(rendered.contains("+="));
}

#[test]
fn prec_level_extraction() {
    let item: ItemEnum = parse_quote! {
        enum Expr {
            #[adze::prec_left(42)]
            Op(u32),
        }
    };
    let variant = &item.variants[0];
    let attr = variant
        .attrs
        .iter()
        .find(|a| is_adze_path(a, "prec_left"))
        .unwrap();
    let level: Expr = attr.parse_args().unwrap();
    let rendered = quote!(#level).to_string();
    assert_eq!(rendered, "42");
}

#[test]
fn quote_roundtrip_struct() {
    let original: ItemStruct = parse_quote! {
        struct Roundtrip {
            field_a: u32,
            field_b: String,
        }
    };
    let tokens = original.to_token_stream();
    let reparsed: ItemStruct = parse2(tokens).unwrap();
    assert_eq!(original.ident, reparsed.ident);
    assert_eq!(original.fields.len(), reparsed.fields.len());
}

#[test]
fn quote_roundtrip_enum() {
    let original: ItemEnum = parse_quote! {
        enum Direction {
            Up,
            Down,
            Left,
            Right,
        }
    };
    let tokens = original.to_token_stream();
    let reparsed: ItemEnum = parse2(tokens).unwrap();
    assert_eq!(original.ident, reparsed.ident);
    assert_eq!(original.variants.len(), reparsed.variants.len());
}

#[test]
fn ident_comparison_is_string_based() {
    let a = Ident::new("leaf", Span::call_site());
    let b = Ident::new("leaf", Span::call_site());
    assert_eq!(a, b);
    assert_eq!(a.to_string(), "leaf");
}

#[test]
fn token_stream_extend_combines() {
    let mut combined = TokenStream::new();
    combined.extend(quote!(
        struct A;
    ));
    combined.extend(quote!(
        struct B;
    ));
    let rendered = combined.to_string();
    assert!(rendered.contains('A'));
    assert!(rendered.contains('B'));
}

#[test]
fn field_visibility_public() {
    let item: ItemStruct = parse_quote! {
        pub struct Visible {
            pub field: u32,
        }
    };
    let field = item.fields.iter().next().unwrap();
    assert!(matches!(field.vis, syn::Visibility::Public(_)));
}

#[test]
fn field_visibility_private() {
    let item: ItemStruct = parse_quote! {
        struct Private {
            field: u32,
        }
    };
    let field = item.fields.iter().next().unwrap();
    assert!(matches!(field.vis, syn::Visibility::Inherited));
}

#[test]
fn enum_variant_discriminant() {
    let item: ItemEnum = parse_quote! {
        enum Level {
            Low = 1,
            High = 10,
        }
    };
    for variant in &item.variants {
        assert!(variant.discriminant.is_some());
    }
}

#[test]
fn module_items_can_be_enumerated() {
    let module: ItemMod = parse_quote! {
        mod test_mod {
            struct A;
            struct B;
            enum C { X }
        }
    };
    let (_, items) = module.content.unwrap();
    assert_eq!(items.len(), 3);
}

#[test]
fn nested_module_parses() {
    let module: ItemMod = parse_quote! {
        mod outer {
            mod inner {
                struct Deep;
            }
        }
    };
    let (_, items) = module.content.unwrap();
    assert_eq!(items.len(), 1);
    assert!(matches!(items[0], syn::Item::Mod(_)));
}
