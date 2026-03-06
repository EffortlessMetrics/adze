//! Comprehensive v3 tests for quote! macro patterns used in adze-macro code generation.
//!
//! Tests basic generation, interpolation, repetition, conditional generation,
//! nested quote!, type generation, impl block generation, complex real-world
//! patterns, and edge cases.

use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, ItemFn, ItemImpl, ItemStruct, Type, parse_quote, parse2};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn ts_to_string(ts: &TokenStream) -> String {
    ts.to_string()
}

fn parse_as_item_struct(ts: TokenStream) -> ItemStruct {
    parse2(ts).expect("failed to parse as ItemStruct")
}

fn parse_as_item_impl(ts: TokenStream) -> ItemImpl {
    parse2(ts).expect("failed to parse as ItemImpl")
}

fn parse_as_item_fn(ts: TokenStream) -> ItemFn {
    parse2(ts).expect("failed to parse as ItemFn")
}

fn parse_as_derive_input(ts: TokenStream) -> DeriveInput {
    parse2(ts).expect("failed to parse as DeriveInput")
}

fn roundtrip_derive(ts: TokenStream) -> DeriveInput {
    let parsed: DeriveInput = parse2(ts).expect("parse");
    let requoted = quote!(#parsed);
    parse2(requoted).expect("reparse")
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Basic quote! generation (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_basic_empty_quote() {
    let ts = quote! {};
    assert!(ts.is_empty());
}

#[test]
fn test_basic_struct_generation() {
    let ts = quote! {
        struct Foo;
    };
    let s = ts_to_string(&ts);
    assert!(s.contains("struct"));
    assert!(s.contains("Foo"));
}

#[test]
fn test_basic_struct_with_fields() {
    let ts = quote! {
        struct Point {
            x: f64,
            y: f64,
        }
    };
    let item = parse_as_item_struct(ts);
    assert_eq!(item.ident, "Point");
    assert_eq!(item.fields.len(), 2);
}

#[test]
fn test_basic_enum_generation() {
    let ts = quote! {
        enum Color {
            Red,
            Green,
            Blue,
        }
    };
    let di = parse_as_derive_input(ts);
    assert_eq!(di.ident, "Color");
    if let syn::Data::Enum(e) = &di.data {
        assert_eq!(e.variants.len(), 3);
    } else {
        panic!("expected enum");
    }
}

#[test]
fn test_basic_function_generation() {
    let ts = quote! {
        fn hello() -> String {
            String::from("world")
        }
    };
    let f = parse_as_item_fn(ts);
    assert_eq!(f.sig.ident, "hello");
    assert!(f.sig.output != syn::ReturnType::Default);
}

#[test]
fn test_basic_let_binding() {
    let ts = quote! {
        fn example() {
            let x: u32 = 42;
        }
    };
    let s = ts_to_string(&ts);
    assert!(s.contains("let"));
    assert!(s.contains("42"));
}

#[test]
fn test_basic_attribute_generation() {
    let ts = quote! {
        #[derive(Debug, Clone)]
        struct Tagged;
    };
    let di = parse_as_derive_input(ts);
    assert_eq!(di.attrs.len(), 1);
    assert_eq!(di.ident, "Tagged");
}

#[test]
fn test_basic_pub_visibility() {
    let ts = quote! {
        pub struct Public {
            pub field: u32,
        }
    };
    let item = parse_as_item_struct(ts);
    assert!(matches!(item.vis, syn::Visibility::Public(_)));
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Interpolation patterns (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_interp_ident() {
    let name = format_ident!("MyStruct");
    let ts = quote! { struct #name; };
    let di = parse_as_derive_input(ts);
    assert_eq!(di.ident, "MyStruct");
}

#[test]
fn test_interp_type() {
    let ty: Type = parse_quote!(Vec<u32>);
    let ts = quote! {
        struct Wrapper {
            inner: #ty,
        }
    };
    let item = parse_as_item_struct(ts);
    assert_eq!(item.fields.len(), 1);
}

#[test]
fn test_interp_literal_integer() {
    let val: u32 = 99;
    let ts = quote! {
        fn constant() -> u32 { #val }
    };
    let s = ts_to_string(&ts);
    assert!(s.contains("99"));
}

#[test]
fn test_interp_string_literal() {
    let msg = "hello world";
    let ts = quote! {
        fn message() -> &'static str { #msg }
    };
    let s = ts_to_string(&ts);
    assert!(s.contains("hello world"));
}

#[test]
fn test_interp_format_ident_suffix() {
    let base = "Rule";
    let name = format_ident!("{}Impl", base);
    let ts = quote! { struct #name; };
    let di = parse_as_derive_input(ts);
    assert_eq!(di.ident, "RuleImpl");
}

#[test]
fn test_interp_format_ident_prefix() {
    let field_name = "value";
    let getter = format_ident!("get_{}", field_name);
    let ts = quote! {
        fn #getter() -> u32 { 0 }
    };
    let f = parse_as_item_fn(ts);
    assert_eq!(f.sig.ident, "get_value");
}

#[test]
fn test_interp_token_stream() {
    let body: TokenStream = quote! { x + y };
    let ts = quote! {
        fn add(x: i32, y: i32) -> i32 { #body }
    };
    let f = parse_as_item_fn(ts.clone());
    assert_eq!(f.sig.ident, "add");
    let s = ts_to_string(&ts);
    assert!(s.contains("x + y"));
}

#[test]
fn test_interp_multiple_variables() {
    let struct_name = format_ident!("Config");
    let field_name = format_ident!("timeout");
    let field_ty: Type = parse_quote!(u64);
    let ts = quote! {
        struct #struct_name {
            #field_name: #field_ty,
        }
    };
    let item = parse_as_item_struct(ts);
    assert_eq!(item.ident, "Config");
    assert_eq!(item.fields.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Repetition patterns (#(#items)*) (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_rep_empty_list() {
    let items: Vec<Ident> = vec![];
    let ts = quote! { #(#items),* };
    assert!(ts.is_empty());
}

#[test]
fn test_rep_single_element() {
    let names = vec![format_ident!("alpha")];
    let ts = quote! {
        enum E { #(#names),* }
    };
    let di = parse_as_derive_input(ts);
    if let syn::Data::Enum(e) = &di.data {
        assert_eq!(e.variants.len(), 1);
    } else {
        panic!("expected enum");
    }
}

#[test]
fn test_rep_multiple_elements() {
    let variants = vec![format_ident!("A"), format_ident!("B"), format_ident!("C")];
    let ts = quote! {
        enum Letters { #(#variants),* }
    };
    let di = parse_as_derive_input(ts);
    if let syn::Data::Enum(e) = &di.data {
        assert_eq!(e.variants.len(), 3);
    } else {
        panic!("expected enum");
    }
}

#[test]
fn test_rep_struct_fields() {
    let field_names = vec![format_ident!("x"), format_ident!("y"), format_ident!("z")];
    let field_types: Vec<Type> = vec![parse_quote!(f32), parse_quote!(f32), parse_quote!(f32)];
    let ts = quote! {
        struct Vec3 {
            #(#field_names: #field_types),*
        }
    };
    let item = parse_as_item_struct(ts);
    assert_eq!(item.ident, "Vec3");
    assert_eq!(item.fields.len(), 3);
}

#[test]
fn test_rep_function_args() {
    let arg_names = vec![format_ident!("a"), format_ident!("b")];
    let arg_types: Vec<Type> = vec![parse_quote!(i32), parse_quote!(i32)];
    let ts = quote! {
        fn add(#(#arg_names: #arg_types),*) -> i32 { 0 }
    };
    let f = parse_as_item_fn(ts);
    assert_eq!(f.sig.inputs.len(), 2);
}

#[test]
fn test_rep_statements() {
    let stmts: Vec<TokenStream> = vec![
        quote! { let a = 1; },
        quote! { let b = 2; },
        quote! { let c = 3; },
    ];
    let ts = quote! {
        fn setup() {
            #(#stmts)*
        }
    };
    let f = parse_as_item_fn(ts.clone());
    assert_eq!(f.sig.ident, "setup");
    let s = ts_to_string(&ts);
    assert!(s.contains("let a"));
    assert!(s.contains("let c"));
}

#[test]
fn test_rep_nested_with_separator() {
    let pairs: Vec<(Ident, u32)> = vec![
        (format_ident!("WIDTH"), 800),
        (format_ident!("HEIGHT"), 600),
    ];
    let names = pairs.iter().map(|(n, _)| n);
    let vals = pairs.iter().map(|(_, v)| v);
    let ts = quote! {
        fn defaults() {
            #(let #names: u32 = #vals;)*
        }
    };
    let s = ts_to_string(&ts);
    assert!(s.contains("WIDTH"));
    assert!(s.contains("800"));
    assert!(s.contains("HEIGHT"));
    assert!(s.contains("600"));
}

#[test]
fn test_rep_derive_attributes() {
    let derives: Vec<Ident> = vec![
        format_ident!("Debug"),
        format_ident!("Clone"),
        format_ident!("PartialEq"),
    ];
    let ts = quote! {
        #[derive(#(#derives),*)]
        struct Derived;
    };
    let di = parse_as_derive_input(ts);
    assert_eq!(di.attrs.len(), 1);
    let attr_str = di.attrs[0].to_token_stream().to_string();
    assert!(attr_str.contains("Debug"));
    assert!(attr_str.contains("Clone"));
    assert!(attr_str.contains("PartialEq"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Conditional generation (if/match) (8 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_cond_optional_field_some() {
    let include_field = true;
    let extra = if include_field {
        quote! { extra: String, }
    } else {
        quote! {}
    };
    let ts = quote! {
        struct WithExtra {
            base: u32,
            #extra
        }
    };
    let item = parse_as_item_struct(ts);
    assert_eq!(item.fields.len(), 2);
}

#[test]
fn test_cond_optional_field_none() {
    let include_field = false;
    let extra = if include_field {
        quote! { extra: String, }
    } else {
        quote! {}
    };
    let ts = quote! {
        struct WithoutExtra {
            base: u32,
            #extra
        }
    };
    let item = parse_as_item_struct(ts);
    assert_eq!(item.fields.len(), 1);
}

#[test]
fn test_cond_match_visibility_pub() {
    let vis_kind = "public";
    let vis = match vis_kind {
        "public" => quote! { pub },
        "crate" => quote! { pub(crate) },
        _ => quote! {},
    };
    let name = format_ident!("MyType");
    let ts = quote! { #vis struct #name; };
    let di = parse_as_derive_input(ts);
    assert!(matches!(di.vis, syn::Visibility::Public(_)));
}

#[test]
fn test_cond_match_visibility_private() {
    let vis_kind = "private";
    let vis = match vis_kind {
        "public" => quote! { pub },
        "crate" => quote! { pub(crate) },
        _ => quote! {},
    };
    let name = format_ident!("Private");
    let ts = quote! { #vis struct #name; };
    let di = parse_as_derive_input(ts);
    assert!(matches!(di.vis, syn::Visibility::Inherited));
}

#[test]
fn test_cond_optional_derive() {
    let want_serde = true;
    let serde_derive = if want_serde {
        quote! { #[derive(serde::Serialize)] }
    } else {
        quote! {}
    };
    let ts = quote! {
        #serde_derive
        struct Data {
            value: u32,
        }
    };
    let di = parse_as_derive_input(ts);
    assert_eq!(di.attrs.len(), 1);
}

#[test]
fn test_cond_optional_derive_skipped() {
    let want_serde = false;
    let serde_derive = if want_serde {
        quote! { #[derive(serde::Serialize)] }
    } else {
        quote! {}
    };
    let ts = quote! {
        #serde_derive
        struct Bare {
            value: u32,
        }
    };
    let di = parse_as_derive_input(ts);
    assert!(di.attrs.is_empty());
}

#[test]
fn test_cond_return_type_option() {
    let nullable = true;
    let ret_ty: Type = if nullable {
        parse_quote!(Option<String>)
    } else {
        parse_quote!(String)
    };
    let ts = quote! {
        fn maybe_get() -> #ret_ty { todo!() }
    };
    let f = parse_as_item_fn(ts);
    let ret_str = f.sig.output.to_token_stream().to_string();
    assert!(ret_str.contains("Option"));
}

#[test]
fn test_cond_return_type_plain() {
    let nullable = false;
    let ret_ty: Type = if nullable {
        parse_quote!(Option<String>)
    } else {
        parse_quote!(String)
    };
    let ts = quote! {
        fn always_get() -> #ret_ty { todo!() }
    };
    let f = parse_as_item_fn(ts);
    let ret_str = f.sig.output.to_token_stream().to_string();
    assert!(!ret_str.contains("Option"));
    assert!(ret_str.contains("String"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Nested quote! (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_nested_quote_in_vec() {
    let field_names = ["alpha", "beta"];
    let fields: Vec<TokenStream> = field_names
        .iter()
        .map(|name| {
            let ident = format_ident!("{}", name);
            quote! { #ident: u32 }
        })
        .collect();
    let ts = quote! {
        struct Composite {
            #(#fields),*
        }
    };
    let item = parse_as_item_struct(ts);
    assert_eq!(item.fields.len(), 2);
}

#[test]
fn test_nested_quote_methods() {
    let method_names = ["start", "stop"];
    let methods: Vec<TokenStream> = method_names
        .iter()
        .map(|name| {
            let ident = format_ident!("{}", name);
            quote! {
                fn #ident(&self) {}
            }
        })
        .collect();
    let struct_name = format_ident!("Service");
    let ts = quote! {
        impl #struct_name {
            #(#methods)*
        }
    };
    let item = parse_as_item_impl(ts);
    assert_eq!(item.items.len(), 2);
}

#[test]
fn test_nested_quote_match_arms() {
    let variants = [("Red", 0u8), ("Green", 1u8), ("Blue", 2u8)];
    let arms: Vec<TokenStream> = variants
        .iter()
        .map(|(name, val)| {
            let ident = format_ident!("{}", name);
            quote! { Color::#ident => #val, }
        })
        .collect();
    let ts = quote! {
        fn color_id(c: Color) -> u8 {
            match c {
                #(#arms)*
            }
        }
    };
    let f = parse_as_item_fn(ts.clone());
    assert_eq!(f.sig.ident, "color_id");
    let s = ts_to_string(&ts);
    assert!(s.contains("Red"));
    assert!(s.contains("Blue"));
}

#[test]
fn test_nested_quote_builder_setters() {
    let fields = [("name", "String"), ("age", "u32")];
    let setters: Vec<TokenStream> = fields
        .iter()
        .map(|(name, ty_str)| {
            let name_id = format_ident!("{}", name);
            let ty: Type = syn::parse_str(ty_str).unwrap();
            quote! {
                fn #name_id(mut self, val: #ty) -> Self {
                    self.#name_id = Some(val);
                    self
                }
            }
        })
        .collect();
    let ts = quote! {
        impl PersonBuilder {
            #(#setters)*
        }
    };
    let item = parse_as_item_impl(ts);
    assert_eq!(item.items.len(), 2);
}

#[test]
fn test_nested_quote_struct_and_impl_together() {
    let type_name = format_ident!("Counter");
    let field = format_ident!("count");

    let struct_def = quote! {
        struct #type_name {
            #field: usize,
        }
    };
    let impl_def = quote! {
        impl #type_name {
            fn new() -> Self {
                Self { #field: 0 }
            }
        }
    };

    let item_s = parse_as_item_struct(struct_def);
    assert_eq!(item_s.ident, "Counter");

    let item_i = parse_as_item_impl(impl_def);
    assert_eq!(item_i.items.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Type generation patterns (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_type_gen_generic_struct() {
    let name = format_ident!("Wrapper");
    let ts = quote! {
        struct #name<T> {
            inner: T,
        }
    };
    let di = parse_as_derive_input(ts);
    assert_eq!(di.ident, "Wrapper");
    assert_eq!(di.generics.params.len(), 1);
}

#[test]
fn test_type_gen_bounded_generic() {
    let name = format_ident!("Processor");
    let ts = quote! {
        struct #name<T: Clone + Send> {
            data: Vec<T>,
        }
    };
    let di = parse_as_derive_input(ts);
    assert_eq!(di.generics.params.len(), 1);
}

#[test]
fn test_type_gen_lifetime_struct() {
    let name = format_ident!("Ref");
    let ts = quote! {
        struct #name<'a> {
            data: &'a str,
        }
    };
    let di = parse_as_derive_input(ts);
    assert_eq!(di.generics.params.len(), 1);
}

#[test]
fn test_type_gen_multiple_generics() {
    let ts = quote! {
        struct Pair<A, B> {
            first: A,
            second: B,
        }
    };
    let di = parse_as_derive_input(ts);
    assert_eq!(di.generics.params.len(), 2);
    if let syn::Data::Struct(ref s) = di.data {
        assert_eq!(s.fields.len(), 2);
    } else {
        panic!("expected struct");
    }
}

#[test]
fn test_type_gen_where_clause() {
    let type_name = format_ident!("Mapper");
    let ts = quote! {
        struct #type_name<K, V>
        where
            K: Eq + std::hash::Hash,
            V: Clone,
        {
            map: std::collections::HashMap<K, V>,
        }
    };
    let di = parse_as_derive_input(ts);
    assert_eq!(di.ident, "Mapper");
    assert!(di.generics.where_clause.is_some());
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Impl block generation (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_impl_basic_method() {
    let type_name = format_ident!("Widget");
    let ts = quote! {
        impl #type_name {
            fn new() -> Self {
                Self {}
            }
        }
    };
    let item = parse_as_item_impl(ts);
    assert_eq!(item.items.len(), 1);
}

#[test]
fn test_impl_trait_for_type() {
    let type_name = format_ident!("MyType");
    let ts = quote! {
        impl std::fmt::Display for #type_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "MyType")
            }
        }
    };
    let item = parse_as_item_impl(ts);
    assert!(item.trait_.is_some());
    assert_eq!(item.items.len(), 1);
}

#[test]
fn test_impl_multiple_methods() {
    let methods = ["push", "pop", "peek"];
    let method_defs: Vec<TokenStream> = methods
        .iter()
        .map(|m| {
            let ident = format_ident!("{}", m);
            quote! { fn #ident(&self) {} }
        })
        .collect();
    let ts = quote! {
        impl Stack {
            #(#method_defs)*
        }
    };
    let item = parse_as_item_impl(ts);
    assert_eq!(item.items.len(), 3);
}

#[test]
fn test_impl_with_generics() {
    let ts = quote! {
        impl<T: Clone> Container<T> {
            fn get(&self) -> T {
                self.inner.clone()
            }
        }
    };
    let item = parse_as_item_impl(ts);
    assert_eq!(item.generics.params.len(), 1);
    assert_eq!(item.items.len(), 1);
}

#[test]
fn test_impl_associated_const_and_fn() {
    let type_name = format_ident!("Limits");
    let ts = quote! {
        impl #type_name {
            const MAX: usize = 1024;
            fn max() -> usize { Self::MAX }
        }
    };
    let item = parse_as_item_impl(ts);
    assert_eq!(item.items.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Complex real-world patterns (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_complex_derive_with_attributes() {
    let name = format_ident!("GrammarNode");
    let derives = vec![format_ident!("Debug"), format_ident!("Clone")];
    let ts = quote! {
        #[derive(#(#derives),*)]
        #[allow(dead_code)]
        pub struct #name {
            pub kind: u16,
            pub children: Vec<Box<#name>>,
        }
    };
    let di = parse_as_derive_input(ts);
    assert_eq!(di.ident, "GrammarNode");
    assert_eq!(di.attrs.len(), 2);
}

#[test]
fn test_complex_enum_with_data_variants() {
    let variant_defs: Vec<TokenStream> = vec![
        quote! { Leaf(String) },
        quote! { Branch { left: Box<Node>, right: Box<Node> } },
        quote! { Empty },
    ];
    let ts = quote! {
        enum Node {
            #(#variant_defs),*
        }
    };
    let di = parse_as_derive_input(ts);
    if let syn::Data::Enum(e) = &di.data {
        assert_eq!(e.variants.len(), 3);
    } else {
        panic!("expected enum");
    }
}

#[test]
fn test_complex_generated_parser_skeleton() {
    let grammar_name = format_ident!("JsonParser");
    let rule_names = ["value", "array", "object"];
    let rule_fns: Vec<TokenStream> = rule_names
        .iter()
        .map(|r| {
            let fn_name = format_ident!("parse_{}", r);
            quote! {
                fn #fn_name(&self, input: &str) -> Result<(), String> {
                    Ok(())
                }
            }
        })
        .collect();
    let ts = quote! {
        impl #grammar_name {
            #(#rule_fns)*
        }
    };
    let item = parse_as_item_impl(ts);
    assert_eq!(item.items.len(), 3);
}

#[test]
fn test_complex_from_impl_for_enum() {
    let type_name = format_ident!("ParseError");
    let from_types: Vec<(Type, TokenStream)> = vec![
        (parse_quote!(std::io::Error), quote! { ParseError::Io(err) }),
        (
            parse_quote!(std::fmt::Error),
            quote! { ParseError::Format(err) },
        ),
    ];
    let impls: Vec<TokenStream> = from_types
        .iter()
        .map(|(from_ty, constructor)| {
            quote! {
                impl From<#from_ty> for #type_name {
                    fn from(err: #from_ty) -> Self {
                        #constructor
                    }
                }
            }
        })
        .collect();
    for imp in &impls {
        let item = parse_as_item_impl(imp.clone());
        assert!(item.trait_.is_some());
    }
    assert_eq!(impls.len(), 2);
}

#[test]
fn test_complex_multi_struct_codegen() {
    let structs: Vec<(&str, Vec<(&str, &str)>)> = vec![
        ("Request", vec![("url", "String"), ("method", "String")]),
        ("Response", vec![("status", "u16"), ("body", "String")]),
    ];
    let defs: Vec<TokenStream> = structs
        .iter()
        .map(|(name, fields)| {
            let name_id = format_ident!("{}", name);
            let field_defs: Vec<TokenStream> = fields
                .iter()
                .map(|(fname, ftype)| {
                    let fid = format_ident!("{}", fname);
                    let fty: Type = syn::parse_str(ftype).unwrap();
                    quote! { pub #fid: #fty }
                })
                .collect();
            quote! {
                pub struct #name_id {
                    #(#field_defs),*
                }
            }
        })
        .collect();
    for def in &defs {
        let item = parse_as_item_struct(def.clone());
        assert_eq!(item.fields.len(), 2);
    }
    assert_eq!(defs.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Edge cases (3 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_edge_reserved_raw_ident() {
    // `gen` is reserved in Rust 2024; use raw identifier
    let field = format_ident!("r#gen");
    let ts = quote! {
        struct HasReserved {
            #field: u32,
        }
    };
    let item = parse_as_item_struct(ts);
    assert_eq!(item.fields.len(), 1);
}

#[test]
fn test_edge_empty_struct() {
    let name = format_ident!("Unit");
    let ts = quote! { struct #name; };
    let di = parse_as_derive_input(ts);
    assert_eq!(di.ident, "Unit");
    assert!(matches!(di.data, syn::Data::Struct(ref s) if s.fields.is_empty()));
}

#[test]
fn test_edge_roundtrip_complex_type() {
    let ts = quote! {
        #[derive(Debug)]
        pub struct Tree<'a, T: Clone> {
            pub label: &'a str,
            pub value: T,
            pub children: Vec<Box<Tree<'a, T>>>,
        }
    };
    let reparsed = roundtrip_derive(ts);
    assert_eq!(reparsed.ident, "Tree");
    assert_eq!(reparsed.generics.params.len(), 2);
    assert_eq!(reparsed.attrs.len(), 1);
}
