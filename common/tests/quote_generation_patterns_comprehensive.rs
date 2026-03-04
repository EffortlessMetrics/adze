//! Comprehensive tests for quote! macro code generation patterns.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

// ── 1. Struct generation ──

#[test]
fn struct_unit() {
    let ts: TokenStream = quote! { struct Unit; };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert_eq!(item.ident, "Unit");
    assert!(item.fields.is_empty());
}

#[test]
fn struct_named_fields() {
    let ts = quote! {
        struct Point {
            x: f64,
            y: f64,
        }
    };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert_eq!(item.ident, "Point");
    assert_eq!(item.fields.len(), 2);
}

#[test]
fn struct_tuple_fields() {
    let ts = quote! { struct Pair(i32, i32); };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert_eq!(item.ident, "Pair");
    assert_eq!(item.fields.len(), 2);
}

#[test]
fn struct_with_generics() {
    let ts = quote! {
        struct Wrapper<T> {
            inner: T,
        }
    };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert_eq!(item.generics.params.len(), 1);
}

#[test]
fn struct_with_lifetime() {
    let ts = quote! {
        struct Borrowed<'a> {
            data: &'a str,
        }
    };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert_eq!(item.generics.params.len(), 1);
}

#[test]
fn struct_with_derive() {
    let ts = quote! {
        #[derive(Debug, Clone)]
        struct Tagged {
            label: String,
        }
    };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert_eq!(item.attrs.len(), 1);
}

#[test]
fn struct_pub_visibility() {
    let ts = quote! {
        pub struct Public {
            pub field: u8,
        }
    };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert!(matches!(item.vis, syn::Visibility::Public(_)));
}

// ── 2. Enum generation ──

#[test]
fn enum_unit_variants() {
    let ts = quote! {
        enum Color {
            Red,
            Green,
            Blue,
        }
    };
    let item: syn::ItemEnum = syn::parse2(ts).unwrap();
    assert_eq!(item.ident, "Color");
    assert_eq!(item.variants.len(), 3);
}

#[test]
fn enum_tuple_variant() {
    let ts = quote! {
        enum Shape {
            Circle(f64),
            Rect(f64, f64),
        }
    };
    let item: syn::ItemEnum = syn::parse2(ts).unwrap();
    assert_eq!(item.variants.len(), 2);
}

#[test]
fn enum_struct_variant() {
    let ts = quote! {
        enum Event {
            Click { x: i32, y: i32 },
            Key { code: u32 },
        }
    };
    let item: syn::ItemEnum = syn::parse2(ts).unwrap();
    assert_eq!(item.variants.len(), 2);
}

#[test]
fn enum_with_discriminant() {
    let ts = quote! {
        enum Status {
            Ok = 0,
            Err = 1,
        }
    };
    let item: syn::ItemEnum = syn::parse2(ts).unwrap();
    assert!(item.variants[0].discriminant.is_some());
}

#[test]
fn enum_with_generics() {
    let ts = quote! {
        enum Maybe<T> {
            Some(T),
            None,
        }
    };
    let item: syn::ItemEnum = syn::parse2(ts).unwrap();
    assert_eq!(item.generics.params.len(), 1);
}

// ── 3. Function generation ──

#[test]
fn fn_empty_body() {
    let ts = quote! { fn noop() {} };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert_eq!(item.sig.ident, "noop");
    assert!(item.sig.inputs.is_empty());
}

#[test]
fn fn_with_args() {
    let ts = quote! { fn add(a: i32, b: i32) -> i32 { a + b } };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert_eq!(item.sig.inputs.len(), 2);
    assert!(item.sig.output != syn::ReturnType::Default);
}

#[test]
fn fn_async() {
    let ts = quote! { async fn fetch() -> String { String::new() } };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert!(item.sig.asyncness.is_some());
}

#[test]
fn fn_unsafe() {
    let ts = quote! { unsafe fn dangerous() {} };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert!(item.sig.unsafety.is_some());
}

#[test]
fn fn_with_where_clause() {
    let ts = quote! {
        fn process<T>(val: T) where T: std::fmt::Debug {}
    };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert!(item.sig.generics.where_clause.is_some());
}

#[test]
fn fn_with_body_statements() {
    let ts = quote! {
        fn compute() -> u32 {
            let x = 1;
            let y = 2;
            x + y
        }
    };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert_eq!(item.block.stmts.len(), 3);
}

// ── 4. Impl block generation ──

#[test]
fn impl_inherent() {
    let ts = quote! {
        impl Foo {
            fn bar(&self) {}
        }
    };
    let item: syn::ItemImpl = syn::parse2(ts).unwrap();
    assert!(item.trait_.is_none());
}

#[test]
fn impl_trait() {
    let ts = quote! {
        impl Clone for Foo {
            fn clone(&self) -> Self { Foo }
        }
    };
    let item: syn::ItemImpl = syn::parse2(ts).unwrap();
    assert!(item.trait_.is_some());
}

#[test]
fn impl_multiple_methods() {
    let ts = quote! {
        impl Counter {
            fn new() -> Self { Counter }
            fn increment(&mut self) {}
            fn value(&self) -> u32 { 0 }
        }
    };
    let item: syn::ItemImpl = syn::parse2(ts).unwrap();
    assert_eq!(item.items.len(), 3);
}

#[test]
fn impl_with_generics() {
    let ts = quote! {
        impl<T: Clone> Container<T> {
            fn get(&self) -> T { unimplemented!() }
        }
    };
    let item: syn::ItemImpl = syn::parse2(ts).unwrap();
    assert_eq!(item.generics.params.len(), 1);
}

#[test]
fn impl_associated_const() {
    let ts = quote! {
        impl Limits {
            const MAX: u32 = 100;
        }
    };
    let item: syn::ItemImpl = syn::parse2(ts).unwrap();
    assert_eq!(item.items.len(), 1);
}

// ── 5. Interpolation (#ident) ──

#[test]
fn interpolate_ident() {
    let name = format_ident!("Foo");
    let ts = quote! { struct #name; };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert_eq!(item.ident, "Foo");
}

#[test]
fn interpolate_type() {
    let ty: syn::Type = syn::parse_str("Vec<u8>").unwrap();
    let ts = quote! {
        struct Holder {
            data: #ty,
        }
    };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert_eq!(item.fields.len(), 1);
}

#[test]
fn interpolate_literal_integer() {
    let val = 42u32;
    let ts = quote! { const N: u32 = #val; };
    let text = ts.to_string();
    assert!(text.contains("42"));
}

#[test]
fn interpolate_literal_string() {
    let msg = "hello world";
    let ts = quote! { const MSG: &str = #msg; };
    let text = ts.to_string();
    assert!(text.contains("hello world"));
}

#[test]
fn interpolate_expr() {
    let expr: syn::Expr = syn::parse_str("a + b").unwrap();
    let ts = quote! { fn result() -> i32 { #expr } };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert_eq!(item.sig.ident, "result");
}

#[test]
fn interpolate_multiple_idents() {
    let struct_name = format_ident!("MyStruct");
    let field_name = format_ident!("my_field");
    let ts = quote! {
        struct #struct_name {
            #field_name: u32,
        }
    };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert_eq!(item.ident, "MyStruct");
}

#[test]
fn interpolate_token_stream() {
    let body: TokenStream = quote! { x + 1 };
    let ts = quote! { fn calc() -> i32 { #body } };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert!(!item.block.stmts.is_empty());
}

// ── 6. Repetition (#(..)* ) ──

#[test]
fn repetition_fields() {
    let names = vec![format_ident!("a"), format_ident!("b"), format_ident!("c")];
    let ts = quote! {
        struct S {
            #(#names: u32,)*
        }
    };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert_eq!(item.fields.len(), 3);
}

#[test]
fn repetition_enum_variants() {
    let variants = vec![format_ident!("X"), format_ident!("Y"), format_ident!("Z")];
    let ts = quote! {
        enum E {
            #(#variants,)*
        }
    };
    let item: syn::ItemEnum = syn::parse2(ts).unwrap();
    assert_eq!(item.variants.len(), 3);
}

#[test]
fn repetition_statements() {
    let assignments: Vec<TokenStream> = (0u32..3)
        .map(|i| {
            let var = format_ident!("x{}", i);
            let val = i;
            quote! { let #var = #val; }
        })
        .collect();
    let ts = quote! { fn init() { #(#assignments)* } };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert_eq!(item.block.stmts.len(), 3);
}

#[test]
fn repetition_with_separator() {
    let args = vec![format_ident!("a"), format_ident!("b"), format_ident!("c")];
    let ts = quote! { fn f(#(#args: i32),*) {} };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert_eq!(item.sig.inputs.len(), 3);
}

#[test]
fn repetition_nested_pairs() {
    let names = vec![format_ident!("x"), format_ident!("y")];
    let types: Vec<syn::Type> = vec![
        syn::parse_str("i32").unwrap(),
        syn::parse_str("f64").unwrap(),
    ];
    let ts = quote! {
        struct Pair {
            #(#names: #types,)*
        }
    };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert_eq!(item.fields.len(), 2);
}

#[test]
fn repetition_empty() {
    let items: Vec<TokenStream> = vec![];
    let ts = quote! {
        struct Empty {
            #(#items)*
        }
    };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert!(item.fields.is_empty());
}

#[test]
fn repetition_trait_bounds() {
    let bounds = vec![quote! { Clone }, quote! { Debug }, quote! { Send }];
    let ts = quote! { fn constrained<T: #(#bounds)+*>() {} };
    let text = ts.to_string();
    assert!(text.contains("Clone"));
    assert!(text.contains("Debug"));
    assert!(text.contains("Send"));
}

#[test]
fn repetition_match_arms() {
    let variants = vec![format_ident!("A"), format_ident!("B")];
    let values: Vec<u32> = vec![1, 2];
    let ts = quote! {
        fn to_num(e: E) -> u32 {
            match e {
                #(E::#variants => #values,)*
            }
        }
    };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert_eq!(item.sig.ident, "to_num");
}

// ── 7. format_ident! patterns ──

#[test]
fn format_ident_simple() {
    let id = format_ident!("my_var");
    assert_eq!(id.to_string(), "my_var");
}

#[test]
fn format_ident_with_number() {
    let id = format_ident!("field_{}", 42u32);
    assert_eq!(id.to_string(), "field_42");
}

#[test]
fn format_ident_prefix_suffix() {
    let base = format_ident!("Node");
    let id = format_ident!("visit_{}", base);
    assert_eq!(id.to_string(), "visit_Node");
}

#[test]
fn format_ident_in_loop() {
    let idents: Vec<_> = (0u32..5).map(|i| format_ident!("arg{}", i)).collect();
    assert_eq!(idents.len(), 5);
    assert_eq!(idents[3].to_string(), "arg3");
}

#[test]
fn format_ident_uppercase() {
    let name = "widget";
    let id = format_ident!("CONST_{}", name.to_uppercase());
    assert_eq!(id.to_string(), "CONST_WIDGET");
}

#[test]
fn format_ident_in_struct_generation() {
    let prefix = "my";
    let struct_name = format_ident!("{}Struct", prefix);
    let getter = format_ident!("get_{}", prefix);
    let ts = quote! {
        impl #struct_name {
            fn #getter(&self) -> u32 { 0 }
        }
    };
    let item: syn::ItemImpl = syn::parse2(ts).unwrap();
    assert_eq!(item.items.len(), 1);
}

// ── 8. Nested quote! ──

#[test]
fn nested_quote_method_bodies() {
    let methods: Vec<TokenStream> = vec!["alpha", "beta"]
        .into_iter()
        .map(|n| {
            let name = format_ident!("{}", n);
            quote! { fn #name(&self) -> &str { stringify!(#name) } }
        })
        .collect();
    let ts = quote! {
        impl Widget {
            #(#methods)*
        }
    };
    let item: syn::ItemImpl = syn::parse2(ts).unwrap();
    assert_eq!(item.items.len(), 2);
}

#[test]
fn nested_quote_struct_and_impl() {
    let name = format_ident!("Config");
    let struct_ts = quote! {
        struct #name {
            value: u32,
        }
    };
    let impl_ts = quote! {
        impl #name {
            fn new() -> Self { #name { value: 0 } }
        }
    };
    let combined = quote! { #struct_ts #impl_ts };
    let file: syn::File = syn::parse2(combined).unwrap();
    assert_eq!(file.items.len(), 2);
}

#[test]
fn nested_quote_enum_with_impl() {
    let variants: Vec<TokenStream> = vec!["On", "Off"]
        .into_iter()
        .map(|v| {
            let id = format_ident!("{}", v);
            quote! { #id }
        })
        .collect();
    let ts = quote! {
        enum Toggle {
            #(#variants,)*
        }
    };
    let item: syn::ItemEnum = syn::parse2(ts).unwrap();
    assert_eq!(item.variants.len(), 2);
}

#[test]
fn nested_quote_conditional_field() {
    let include_extra = true;
    let extra_field = if include_extra {
        quote! { extra: bool, }
    } else {
        quote! {}
    };
    let ts = quote! {
        struct Flexible {
            base: u32,
            #extra_field
        }
    };
    let item: syn::ItemStruct = syn::parse2(ts).unwrap();
    assert_eq!(item.fields.len(), 2);
}

#[test]
fn nested_quote_builder_pattern() {
    let fields = vec![("width", "u32"), ("height", "u32")];
    let setters: Vec<TokenStream> = fields
        .iter()
        .map(|(name, _ty)| {
            let field = format_ident!("{}", name);
            quote! {
                fn #field(mut self, val: u32) -> Self {
                    self.#field = val;
                    self
                }
            }
        })
        .collect();
    let ts = quote! {
        impl Builder {
            #(#setters)*
        }
    };
    let item: syn::ItemImpl = syn::parse2(ts).unwrap();
    assert_eq!(item.items.len(), 2);
}

// ── 9. TokenStream to string ──

#[test]
fn to_string_simple() {
    let ts = quote! { let x = 5; };
    let s = ts.to_string();
    assert!(s.contains("let"));
    assert!(s.contains("x"));
    assert!(s.contains("5"));
}

#[test]
fn to_string_preserves_structure() {
    let ts = quote! { fn add(a: i32, b: i32) -> i32 { a + b } };
    let s = ts.to_string();
    assert!(s.contains("fn add"));
    assert!(s.contains("-> i32"));
}

#[test]
fn to_string_with_attributes() {
    let ts = quote! {
        #[inline]
        fn fast() {}
    };
    let s = ts.to_string();
    assert!(s.contains("inline"));
}

#[test]
fn to_string_interpolated_values() {
    let n = format_ident!("counter");
    let v = 99u32;
    let ts = quote! { static #n: u32 = #v; };
    let s = ts.to_string();
    assert!(s.contains("counter"));
    assert!(s.contains("99"));
}

#[test]
fn to_string_roundtrip_parse() {
    let ts = quote! { struct Roundtrip { val: u8 } };
    let s = ts.to_string();
    let reparsed: TokenStream = s.parse().unwrap();
    let item: syn::ItemStruct = syn::parse2(reparsed).unwrap();
    assert_eq!(item.ident, "Roundtrip");
}

// ── 10. Parse back with syn ──

#[test]
fn parse_back_file() {
    let ts = quote! {
        struct A;
        struct B;
        fn c() {}
    };
    let file: syn::File = syn::parse2(ts).unwrap();
    assert_eq!(file.items.len(), 3);
}

#[test]
fn parse_back_use_statement() {
    let ts = quote! { use std::collections::HashMap; };
    let item: syn::ItemUse = syn::parse2(ts).unwrap();
    let text = quote! { #item }.to_string();
    assert!(text.contains("HashMap"));
}

#[test]
fn parse_back_trait_def() {
    let ts = quote! {
        trait Greet {
            fn hello(&self) -> String;
        }
    };
    let item: syn::ItemTrait = syn::parse2(ts).unwrap();
    assert_eq!(item.ident, "Greet");
    assert_eq!(item.items.len(), 1);
}

#[test]
fn parse_back_type_alias() {
    let ts = quote! { type Result<T> = std::result::Result<T, Error>; };
    let item: syn::ItemType = syn::parse2(ts).unwrap();
    assert_eq!(item.ident, "Result");
}

#[test]
fn parse_back_const() {
    let ts = quote! { const PI: f64 = 3.14; };
    let item: syn::ItemConst = syn::parse2(ts).unwrap();
    assert_eq!(item.ident, "PI");
}

#[test]
fn parse_back_static() {
    let ts = quote! { static COUNT: u32 = 0; };
    let item: syn::ItemStatic = syn::parse2(ts).unwrap();
    assert_eq!(item.ident, "COUNT");
}

#[test]
fn parse_back_mod() {
    let ts = quote! {
        mod inner {
            fn hidden() {}
        }
    };
    let item: syn::ItemMod = syn::parse2(ts).unwrap();
    assert_eq!(item.ident, "inner");
}

#[test]
fn parse_back_extern_block() {
    let ts = quote! {
        unsafe extern "C" {
            fn puts(s: *const u8) -> i32;
        }
    };
    let item: syn::ItemForeignMod = syn::parse2(ts).unwrap();
    assert_eq!(item.items.len(), 1);
}

// ── Additional patterns ──

#[test]
fn generate_from_trait_impl() {
    let source = format_ident!("String");
    let target = format_ident!("MyType");
    let ts = quote! {
        impl From<#source> for #target {
            fn from(val: #source) -> Self {
                #target
            }
        }
    };
    let item: syn::ItemImpl = syn::parse2(ts).unwrap();
    assert!(item.trait_.is_some());
}

#[test]
fn generate_display_impl() {
    let name = format_ident!("Token");
    let ts = quote! {
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", stringify!(#name))
            }
        }
    };
    let item: syn::ItemImpl = syn::parse2(ts).unwrap();
    assert!(item.trait_.is_some());
}

#[test]
fn generate_default_impl() {
    let name = format_ident!("Settings");
    let ts = quote! {
        impl Default for #name {
            fn default() -> Self {
                #name { verbose: false }
            }
        }
    };
    let item: syn::ItemImpl = syn::parse2(ts).unwrap();
    assert!(item.trait_.is_some());
}

#[test]
fn generate_new_constructor() {
    let fields = vec![("name", "String"), ("age", "u32")];
    let params: Vec<TokenStream> = fields
        .iter()
        .map(|(n, t)| {
            let n = format_ident!("{}", n);
            let t: syn::Type = syn::parse_str(t).unwrap();
            quote! { #n: #t }
        })
        .collect();
    let inits: Vec<TokenStream> = fields
        .iter()
        .map(|(n, _)| {
            let n = format_ident!("{}", n);
            quote! { #n }
        })
        .collect();
    let ts = quote! {
        impl Person {
            fn new(#(#params),*) -> Self {
                Self { #(#inits),* }
            }
        }
    };
    let item: syn::ItemImpl = syn::parse2(ts).unwrap();
    assert_eq!(item.items.len(), 1);
}

#[test]
fn generate_getters() {
    let fields = vec![("name", "String"), ("id", "u64")];
    let methods: Vec<TokenStream> = fields
        .iter()
        .map(|(name, ty)| {
            let getter = format_ident!("{}", name);
            let ret: syn::Type = syn::parse_str(ty).unwrap();
            quote! { fn #getter(&self) -> &#ret { &self.#getter } }
        })
        .collect();
    let ts = quote! {
        impl Record {
            #(#methods)*
        }
    };
    let item: syn::ItemImpl = syn::parse2(ts).unwrap();
    assert_eq!(item.items.len(), 2);
}

#[test]
fn generate_module_with_reexport() {
    let mod_name = format_ident!("generated");
    let ts = quote! {
        mod #mod_name {
            pub struct Inner;
        }
    };
    let item: syn::ItemMod = syn::parse2(ts).unwrap();
    assert_eq!(item.ident, "generated");
}

#[test]
fn generate_closure_expression() {
    let ts = quote! {
        fn apply() -> u32 {
            let f = |x: u32| x + 1;
            f(5)
        }
    };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert_eq!(item.sig.ident, "apply");
}

#[test]
fn interpolate_path() {
    let ts = quote! {
        fn make() -> std::io::Result<()> {
            Ok(())
        }
    };
    let item: syn::ItemFn = syn::parse2(ts).unwrap();
    assert!(item.sig.output != syn::ReturnType::Default);
}

#[test]
fn repetition_chained_calls() {
    let methods = vec![format_ident!("step1"), format_ident!("step2")];
    let ts = quote! {
        fn chain(b: Builder) -> Builder {
            b #(.#methods())*
        }
    };
    let text = ts.to_string();
    assert!(text.contains("step1"));
    assert!(text.contains("step2"));
}

#[test]
fn empty_token_stream() {
    let ts = quote! {};
    assert!(ts.is_empty());
}

#[test]
fn token_stream_extend() {
    let mut ts = quote! { struct A; };
    let more = quote! { struct B; };
    ts.extend(more);
    let file: syn::File = syn::parse2(ts).unwrap();
    assert_eq!(file.items.len(), 2);
}

#[test]
fn format_ident_raw_keyword() {
    let id = format_ident!("r#type");
    let ts = quote! { let #id = 5; };
    let text = ts.to_string();
    assert!(text.contains("r#type") || text.contains("r # type"));
}
