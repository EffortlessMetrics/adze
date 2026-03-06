//! Comprehensive tests for syn parsing patterns used in adze-common.
//!
//! Validates parsing of Rust syntax constructs via syn, quote, and proc_macro2.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, Expr, FnArg, GenericParam, ImplItem, ItemEnum, ItemFn, ItemImpl, ItemMod,
    ItemStruct, ItemTrait, ItemType, ItemUse, Lit, Pat, ReturnType, TraitItem, Type, Visibility,
    WhereClause, parse2,
};

/// Helper: parse a pattern using `Pat::parse_single`.
fn parse_pat_single(ts: TokenStream) -> Pat {
    use syn::parse::Parser;
    Pat::parse_single.parse2(ts).unwrap()
}

/// Helper: parse a multi-pattern (with `|`) using `Pat::parse_multi_with_leading_vert`.
fn parse_pat_multi(ts: TokenStream) -> Pat {
    use syn::parse::Parser;
    Pat::parse_multi_with_leading_vert.parse2(ts).unwrap()
}

// ============================================================
// 1. Struct definitions
// ============================================================

#[test]
fn parse_unit_struct() {
    let ts: TokenStream = quote! { struct Unit; };
    let item: ItemStruct = parse2(ts).unwrap();
    assert_eq!(item.ident, "Unit");
    assert!(item.fields.is_empty());
}

#[test]
fn parse_named_fields_struct() {
    let ts = quote! {
        struct Point {
            x: f64,
            y: f64,
        }
    };
    let item: ItemStruct = parse2(ts).unwrap();
    assert_eq!(item.ident, "Point");
    assert_eq!(item.fields.len(), 2);
}

#[test]
fn parse_tuple_struct() {
    let ts = quote! { struct Wrapper(i32, String); };
    let item: ItemStruct = parse2(ts).unwrap();
    assert_eq!(item.ident, "Wrapper");
    assert_eq!(item.fields.len(), 2);
}

#[test]
fn parse_struct_with_option_field() {
    let ts = quote! {
        struct Cfg {
            name: Option<String>,
        }
    };
    let item: ItemStruct = parse2(ts).unwrap();
    let field = item.fields.iter().next().unwrap();
    assert_eq!(field.ident.as_ref().unwrap(), "name");
}

#[test]
fn parse_struct_with_vec_field() {
    let ts = quote! {
        struct List {
            items: Vec<u32>,
        }
    };
    let item: ItemStruct = parse2(ts).unwrap();
    assert_eq!(item.fields.len(), 1);
}

#[test]
fn parse_struct_with_box_field() {
    let ts = quote! {
        struct Node {
            left: Box<Node>,
            right: Box<Node>,
        }
    };
    let item: ItemStruct = parse2(ts).unwrap();
    assert_eq!(item.fields.len(), 2);
}

// ============================================================
// 2. Enum definitions
// ============================================================

#[test]
fn parse_enum_unit_variants() {
    let ts = quote! {
        enum Color { Red, Green, Blue }
    };
    let item: ItemEnum = parse2(ts).unwrap();
    assert_eq!(item.ident, "Color");
    assert_eq!(item.variants.len(), 3);
}

#[test]
fn parse_enum_tuple_variant() {
    let ts = quote! {
        enum Expr {
            Lit(i64),
            Add(Box<Expr>, Box<Expr>),
        }
    };
    let item: ItemEnum = parse2(ts).unwrap();
    assert_eq!(item.variants.len(), 2);
    assert_eq!(item.variants[1].fields.len(), 2);
}

#[test]
fn parse_enum_struct_variant() {
    let ts = quote! {
        enum Shape {
            Circle { radius: f64 },
            Rect { width: f64, height: f64 },
        }
    };
    let item: ItemEnum = parse2(ts).unwrap();
    assert_eq!(item.variants.len(), 2);
    assert_eq!(item.variants[1].fields.len(), 2);
}

#[test]
fn parse_enum_mixed_variants() {
    let ts = quote! {
        enum Token {
            Eof,
            Number(f64),
            Ident { name: String },
        }
    };
    let item: ItemEnum = parse2(ts).unwrap();
    assert_eq!(item.variants.len(), 3);
}

#[test]
fn parse_enum_with_discriminant() {
    let ts = quote! {
        enum ErrorCode {
            Ok = 0,
            NotFound = 404,
            Internal = 500,
        }
    };
    let item: ItemEnum = parse2(ts).unwrap();
    assert!(item.variants[0].discriminant.is_some());
    assert!(item.variants[2].discriminant.is_some());
}

// ============================================================
// 3. Function signatures
// ============================================================

#[test]
fn parse_fn_no_args_no_return() {
    let ts = quote! { fn noop() {} };
    let item: ItemFn = parse2(ts).unwrap();
    assert_eq!(item.sig.ident, "noop");
    assert!(item.sig.inputs.is_empty());
    assert!(matches!(item.sig.output, ReturnType::Default));
}

#[test]
fn parse_fn_with_return_type() {
    let ts = quote! { fn answer() -> i32 { 42 } };
    let item: ItemFn = parse2(ts).unwrap();
    assert!(matches!(item.sig.output, ReturnType::Type(..)));
}

#[test]
fn parse_fn_with_args() {
    let ts = quote! { fn add(a: i32, b: i32) -> i32 { a + b } };
    let item: ItemFn = parse2(ts).unwrap();
    assert_eq!(item.sig.inputs.len(), 2);
}

#[test]
fn parse_async_fn() {
    let ts = quote! { async fn fetch() -> String { String::new() } };
    let item: ItemFn = parse2(ts).unwrap();
    assert!(item.sig.asyncness.is_some());
}

#[test]
fn parse_const_fn() {
    let ts = quote! { const fn max_size() -> usize { 1024 } };
    let item: ItemFn = parse2(ts).unwrap();
    assert!(item.sig.constness.is_some());
}

#[test]
fn parse_unsafe_fn() {
    let ts = quote! { unsafe fn raw_ptr() -> *const u8 { std::ptr::null() } };
    let item: ItemFn = parse2(ts).unwrap();
    assert!(item.sig.unsafety.is_some());
}

#[test]
fn parse_fn_with_self_receiver() {
    let ts = quote! {
        impl Foo {
            fn method(&self) -> i32 { 0 }
        }
    };
    let item: ItemImpl = parse2(ts).unwrap();
    if let ImplItem::Fn(m) = &item.items[0] {
        assert!(matches!(&m.sig.inputs[0], FnArg::Receiver(_)));
    } else {
        panic!("expected method");
    }
}

// ============================================================
// 4. Trait definitions
// ============================================================

#[test]
fn parse_empty_trait() {
    let ts = quote! { trait Marker {} };
    let item: ItemTrait = parse2(ts).unwrap();
    assert_eq!(item.ident, "Marker");
    assert!(item.items.is_empty());
}

#[test]
fn parse_trait_with_method() {
    let ts = quote! {
        trait Greet {
            fn hello(&self) -> String;
        }
    };
    let item: ItemTrait = parse2(ts).unwrap();
    assert_eq!(item.items.len(), 1);
    assert!(matches!(&item.items[0], TraitItem::Fn(_)));
}

#[test]
fn parse_trait_with_default_method() {
    let ts = quote! {
        trait HasDefault {
            fn value(&self) -> i32 { 0 }
        }
    };
    let item: ItemTrait = parse2(ts).unwrap();
    if let TraitItem::Fn(m) = &item.items[0] {
        assert!(m.default.is_some());
    } else {
        panic!("expected fn");
    }
}

#[test]
fn parse_trait_with_associated_type() {
    let ts = quote! {
        trait Iterator {
            type Item;
            fn next(&mut self) -> Option<Self::Item>;
        }
    };
    let item: ItemTrait = parse2(ts).unwrap();
    assert_eq!(item.items.len(), 2);
    assert!(matches!(&item.items[0], TraitItem::Type(_)));
}

#[test]
fn parse_trait_with_supertrait() {
    let ts = quote! {
        trait Printable: std::fmt::Display + std::fmt::Debug {}
    };
    let item: ItemTrait = parse2(ts).unwrap();
    assert_eq!(item.supertraits.len(), 2);
}

// ============================================================
// 5. Impl blocks
// ============================================================

#[test]
fn parse_inherent_impl() {
    let ts = quote! {
        impl Foo {
            fn new() -> Self { Foo }
        }
    };
    let item: ItemImpl = parse2(ts).unwrap();
    assert!(item.trait_.is_none());
    assert_eq!(item.items.len(), 1);
}

#[test]
fn parse_trait_impl() {
    let ts = quote! {
        impl Clone for Foo {
            fn clone(&self) -> Self { Foo }
        }
    };
    let item: ItemImpl = parse2(ts).unwrap();
    assert!(item.trait_.is_some());
}

#[test]
fn parse_impl_with_const() {
    let ts = quote! {
        impl Foo {
            const MAX: usize = 100;
        }
    };
    let item: ItemImpl = parse2(ts).unwrap();
    assert!(matches!(&item.items[0], ImplItem::Const(_)));
}

#[test]
fn parse_impl_with_type_alias() {
    let ts = quote! {
        impl MyTrait for Bar {
            type Output = String;
        }
    };
    let item: ItemImpl = parse2(ts).unwrap();
    assert!(matches!(&item.items[0], ImplItem::Type(_)));
}

// ============================================================
// 6. Type aliases
// ============================================================

#[test]
fn parse_simple_type_alias() {
    let ts = quote! { type Name = String; };
    let item: ItemType = parse2(ts).unwrap();
    assert_eq!(item.ident, "Name");
}

#[test]
fn parse_generic_type_alias() {
    let ts = quote! { type Result<T> = std::result::Result<T, MyError>; };
    let item: ItemType = parse2(ts).unwrap();
    assert_eq!(item.generics.params.len(), 1);
}

#[test]
fn parse_type_alias_with_lifetime() {
    let ts = quote! { type StrRef<'a> = &'a str; };
    let item: ItemType = parse2(ts).unwrap();
    assert!(matches!(
        &item.generics.params[0],
        GenericParam::Lifetime(_)
    ));
}

// ============================================================
// 7. Attribute macros
// ============================================================

#[test]
fn parse_derive_attribute() {
    let ts = quote! {
        #[derive(Debug, Clone)]
        struct S;
    };
    let item: ItemStruct = parse2(ts).unwrap();
    assert!(!item.attrs.is_empty());
    let attr = &item.attrs[0];
    assert!(attr.path().is_ident("derive"));
}

#[test]
fn parse_multiple_attributes() {
    let ts = quote! {
        #[allow(dead_code)]
        #[repr(C)]
        struct Ffi {
            x: i32,
        }
    };
    let item: ItemStruct = parse2(ts).unwrap();
    assert_eq!(item.attrs.len(), 2);
}

#[test]
fn parse_cfg_attribute() {
    let ts = quote! {
        #[cfg(target_os = "linux")]
        fn linux_only() {}
    };
    let item: ItemFn = parse2(ts).unwrap();
    assert!(item.attrs[0].path().is_ident("cfg"));
}

#[test]
fn parse_doc_attribute() {
    let ts = quote! {
        #[doc = "A documented item"]
        struct Documented;
    };
    let item: ItemStruct = parse2(ts).unwrap();
    assert!(item.attrs[0].path().is_ident("doc"));
}

#[test]
fn parse_custom_attribute_on_enum() {
    let ts = quote! {
        #[serde(rename_all = "camelCase")]
        enum Api { GetUser, PostData }
    };
    let item: ItemEnum = parse2(ts).unwrap();
    assert!(item.attrs[0].path().is_ident("serde"));
}

// ============================================================
// 8. Visibility modifiers
// ============================================================

#[test]
fn parse_pub_struct() {
    let ts = quote! { pub struct Public; };
    let item: ItemStruct = parse2(ts).unwrap();
    assert!(matches!(item.vis, Visibility::Public(_)));
}

#[test]
fn parse_pub_crate_struct() {
    let ts = quote! { pub(crate) struct Internal; };
    let item: ItemStruct = parse2(ts).unwrap();
    assert!(matches!(item.vis, Visibility::Restricted(_)));
}

#[test]
fn parse_pub_super_fn() {
    let ts = quote! { pub(super) fn parent_visible() {} };
    let item: ItemFn = parse2(ts).unwrap();
    assert!(matches!(item.vis, Visibility::Restricted(_)));
}

#[test]
fn parse_private_fn() {
    let ts = quote! { fn private() {} };
    let item: ItemFn = parse2(ts).unwrap();
    assert!(matches!(item.vis, Visibility::Inherited));
}

#[test]
fn parse_pub_in_path() {
    let ts = quote! { pub(in crate::module) struct Scoped; };
    let item: ItemStruct = parse2(ts).unwrap();
    assert!(matches!(item.vis, Visibility::Restricted(_)));
}

// ============================================================
// 9. Generic types with bounds
// ============================================================

#[test]
fn parse_generic_struct() {
    let ts = quote! { struct Container<T> { value: T } };
    let item: ItemStruct = parse2(ts).unwrap();
    assert_eq!(item.generics.params.len(), 1);
}

#[test]
fn parse_generic_with_trait_bound() {
    let ts = quote! {
        fn print_it<T: std::fmt::Display>(val: T) {}
    };
    let item: ItemFn = parse2(ts).unwrap();
    if let GenericParam::Type(tp) = &item.sig.generics.params[0] {
        assert!(!tp.bounds.is_empty());
    } else {
        panic!("expected type param");
    }
}

#[test]
fn parse_multiple_generic_params() {
    let ts = quote! {
        struct Pair<A, B> { first: A, second: B }
    };
    let item: ItemStruct = parse2(ts).unwrap();
    assert_eq!(item.generics.params.len(), 2);
}

#[test]
fn parse_generic_with_default() {
    let ts = quote! {
        struct Alloc<A = std::alloc::Global> { alloc: A }
    };
    let item: ItemStruct = parse2(ts).unwrap();
    if let GenericParam::Type(tp) = &item.generics.params[0] {
        assert!(tp.default.is_some());
    } else {
        panic!("expected type param with default");
    }
}

#[test]
fn parse_const_generic() {
    let ts = quote! {
        struct Array<const N: usize> { data: [u8; N] }
    };
    let item: ItemStruct = parse2(ts).unwrap();
    assert!(matches!(&item.generics.params[0], GenericParam::Const(_)));
}

#[test]
fn parse_multiple_trait_bounds() {
    let ts = quote! {
        fn constrained<T: Clone + Send + Sync>(val: T) {}
    };
    let item: ItemFn = parse2(ts).unwrap();
    if let GenericParam::Type(tp) = &item.sig.generics.params[0] {
        assert!(tp.bounds.len() >= 3);
    } else {
        panic!("expected type param");
    }
}

// ============================================================
// 10. Lifetime annotations
// ============================================================

#[test]
fn parse_lifetime_in_struct() {
    let ts = quote! {
        struct Ref<'a> { data: &'a str }
    };
    let item: ItemStruct = parse2(ts).unwrap();
    assert!(matches!(
        &item.generics.params[0],
        GenericParam::Lifetime(_)
    ));
}

#[test]
fn parse_multiple_lifetimes() {
    let ts = quote! {
        struct Multi<'a, 'b> { a: &'a str, b: &'b str }
    };
    let item: ItemStruct = parse2(ts).unwrap();
    assert_eq!(item.generics.params.len(), 2);
}

#[test]
fn parse_lifetime_bound() {
    let ts = quote! {
        struct Outlives<'a, 'b: 'a> { data: &'b &'a str }
    };
    let item: ItemStruct = parse2(ts).unwrap();
    if let GenericParam::Lifetime(lt) = &item.generics.params[1] {
        assert!(!lt.bounds.is_empty());
    } else {
        panic!("expected lifetime with bound");
    }
}

#[test]
fn parse_fn_with_lifetime() {
    let ts = quote! {
        fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { x }
    };
    let item: ItemFn = parse2(ts).unwrap();
    assert_eq!(item.sig.generics.params.len(), 1);
}

// ============================================================
// 11. Where clauses
// ============================================================

#[test]
fn parse_where_clause_on_fn() {
    let ts = quote! {
        fn process<T>(val: T) where T: Clone + Send {}
    };
    let item: ItemFn = parse2(ts).unwrap();
    let wc: &WhereClause = item.sig.generics.where_clause.as_ref().unwrap();
    assert_eq!(wc.predicates.len(), 1);
}

#[test]
fn parse_where_clause_on_struct() {
    let ts = quote! {
        struct Bounded<T> where T: Default { inner: T }
    };
    let item: ItemStruct = parse2(ts).unwrap();
    assert!(item.generics.where_clause.is_some());
}

#[test]
fn parse_where_clause_multiple_predicates() {
    let ts = quote! {
        fn multi<A, B>(a: A, b: B) where A: Clone, B: Default {}
    };
    let item: ItemFn = parse2(ts).unwrap();
    let wc = item.sig.generics.where_clause.as_ref().unwrap();
    assert_eq!(wc.predicates.len(), 2);
}

#[test]
fn parse_where_clause_with_lifetime() {
    let ts = quote! {
        fn with_lt<'a, T>(val: &'a T) where T: 'a {}
    };
    let item: ItemFn = parse2(ts).unwrap();
    assert!(item.sig.generics.where_clause.is_some());
}

// ============================================================
// 12. Expressions
// ============================================================

#[test]
fn parse_integer_literal() {
    let ts = quote! { 42 };
    let expr: Expr = parse2(ts).unwrap();
    if let Expr::Lit(lit) = &expr {
        assert!(matches!(&lit.lit, Lit::Int(_)));
    } else {
        panic!("expected int literal");
    }
}

#[test]
fn parse_string_literal() {
    let ts = quote! { "hello" };
    let expr: Expr = parse2(ts).unwrap();
    if let Expr::Lit(lit) = &expr {
        assert!(matches!(&lit.lit, Lit::Str(_)));
    } else {
        panic!("expected string literal");
    }
}

#[test]
fn parse_bool_literal() {
    let ts = quote! { true };
    let expr: Expr = parse2(ts).unwrap();
    if let Expr::Lit(lit) = &expr {
        assert!(matches!(&lit.lit, Lit::Bool(_)));
    } else {
        panic!("expected bool literal");
    }
}

#[test]
fn parse_path_expression() {
    let ts = quote! { std::collections::HashMap };
    let expr: Expr = parse2(ts).unwrap();
    assert!(matches!(expr, Expr::Path(_)));
}

#[test]
fn parse_function_call_expr() {
    let ts = quote! { foo(1, 2) };
    let expr: Expr = parse2(ts).unwrap();
    if let Expr::Call(call) = &expr {
        assert_eq!(call.args.len(), 2);
    } else {
        panic!("expected call expr");
    }
}

#[test]
fn parse_method_call_expr() {
    let ts = quote! { x.method(arg) };
    let expr: Expr = parse2(ts).unwrap();
    if let Expr::MethodCall(mc) = &expr {
        assert_eq!(mc.method, "method");
        assert_eq!(mc.args.len(), 1);
    } else {
        panic!("expected method call");
    }
}

#[test]
fn parse_binary_expr() {
    let ts = quote! { a + b };
    let expr: Expr = parse2(ts).unwrap();
    assert!(matches!(expr, Expr::Binary(_)));
}

#[test]
fn parse_if_expr() {
    let ts = quote! { if x { 1 } else { 2 } };
    let expr: Expr = parse2(ts).unwrap();
    assert!(matches!(expr, Expr::If(_)));
}

#[test]
fn parse_closure_expr() {
    let ts = quote! { |x: i32| x + 1 };
    let expr: Expr = parse2(ts).unwrap();
    assert!(matches!(expr, Expr::Closure(_)));
}

#[test]
fn parse_tuple_expr() {
    let ts = quote! { (1, 2, 3) };
    let expr: Expr = parse2(ts).unwrap();
    assert!(matches!(expr, Expr::Tuple(_)));
}

#[test]
fn parse_array_expr() {
    let ts = quote! { [1, 2, 3] };
    let expr: Expr = parse2(ts).unwrap();
    assert!(matches!(expr, Expr::Array(_)));
}

#[test]
fn parse_reference_expr() {
    let ts = quote! { &x };
    let expr: Expr = parse2(ts).unwrap();
    assert!(matches!(expr, Expr::Reference(_)));
}

#[test]
fn parse_block_expr() {
    let ts = quote! { { let x = 1; x } };
    let expr: Expr = parse2(ts).unwrap();
    assert!(matches!(expr, Expr::Block(_)));
}

// ============================================================
// 13. Patterns
// ============================================================

#[test]
fn parse_ident_pattern() {
    let pat = parse_pat_single(quote! { x });
    assert!(matches!(pat, Pat::Ident(_)));
}

#[test]
fn parse_tuple_pattern() {
    let pat = parse_pat_single(quote! { (a, b) });
    assert!(matches!(pat, Pat::Tuple(_)));
}

#[test]
fn parse_struct_pattern() {
    let pat = parse_pat_single(quote! { Point { x, y } });
    assert!(matches!(pat, Pat::Struct(_)));
}

#[test]
fn parse_tuple_struct_pattern() {
    let pat = parse_pat_single(quote! { Some(value) });
    assert!(matches!(pat, Pat::TupleStruct(_)));
}

#[test]
fn parse_wildcard_pattern() {
    let pat = parse_pat_single(quote! { _ });
    assert!(matches!(pat, Pat::Wild(_)));
}

#[test]
fn parse_ref_pattern() {
    let pat = parse_pat_single(quote! { &x });
    assert!(matches!(pat, Pat::Reference(_)));
}

#[test]
fn parse_or_pattern() {
    let pat = parse_pat_multi(quote! { A | B | C });
    assert!(matches!(pat, Pat::Or(_)));
}

#[test]
fn parse_slice_pattern() {
    let pat = parse_pat_single(quote! { [first, rest @ ..] });
    assert!(matches!(pat, Pat::Slice(_)));
}

// ============================================================
// 14. Use statements
// ============================================================

#[test]
fn parse_simple_use() {
    let ts = quote! { use std::collections::HashMap; };
    let item: ItemUse = parse2(ts).unwrap();
    assert!(item.leading_colon.is_none());
}

#[test]
fn parse_glob_use() {
    let ts = quote! { use std::io::*; };
    let _item: ItemUse = parse2(ts).unwrap();
}

#[test]
fn parse_grouped_use() {
    let ts = quote! { use std::collections::{HashMap, HashSet}; };
    let _item: ItemUse = parse2(ts).unwrap();
}

#[test]
fn parse_renamed_use() {
    let ts = quote! { use std::io::Result as IoResult; };
    let _item: ItemUse = parse2(ts).unwrap();
}

#[test]
fn parse_pub_use() {
    let ts = quote! { pub use crate::inner::Api; };
    let item: ItemUse = parse2(ts).unwrap();
    assert!(matches!(item.vis, Visibility::Public(_)));
}

// ============================================================
// Additional coverage: modules, complex types, edge cases
// ============================================================

#[test]
fn parse_module_declaration() {
    let ts = quote! {
        mod inner {
            fn secret() {}
        }
    };
    let item: ItemMod = parse2(ts).unwrap();
    assert_eq!(item.ident, "inner");
    assert!(item.content.is_some());
}

#[test]
fn parse_type_path_with_turbofish() {
    let ts = quote! { Vec::<i32> };
    let ty: Type = parse2(ts).unwrap();
    assert!(matches!(ty, Type::Path(_)));
}

#[test]
fn parse_reference_type() {
    let ts = quote! { &'a mut str };
    let ty: Type = parse2(ts).unwrap();
    assert!(matches!(ty, Type::Reference(_)));
}

#[test]
fn parse_tuple_type() {
    let ts = quote! { (i32, String, bool) };
    let ty: Type = parse2(ts).unwrap();
    assert!(matches!(ty, Type::Tuple(_)));
}

#[test]
fn parse_array_type() {
    let ts = quote! { [u8; 256] };
    let ty: Type = parse2(ts).unwrap();
    assert!(matches!(ty, Type::Array(_)));
}

#[test]
fn parse_slice_type() {
    let ts = quote! { [u8] };
    let ty: Type = parse2(ts).unwrap();
    assert!(matches!(ty, Type::Slice(_)));
}

#[test]
fn parse_fn_pointer_type() {
    let ts = quote! { fn(i32) -> bool };
    let ty: Type = parse2(ts).unwrap();
    assert!(matches!(ty, Type::BareFn(_)));
}

#[test]
fn parse_dyn_trait_type() {
    let ts = quote! { dyn Iterator<Item = u32> };
    let ty: Type = parse2(ts).unwrap();
    assert!(matches!(ty, Type::TraitObject(_)));
}

#[test]
fn parse_impl_trait_type() {
    let ts = quote! { impl Iterator<Item = u32> };
    let ty: Type = parse2(ts).unwrap();
    assert!(matches!(ty, Type::ImplTrait(_)));
}

#[test]
fn parse_never_type() {
    let ts = quote! { ! };
    let ty: Type = parse2(ts).unwrap();
    assert!(matches!(ty, Type::Never(_)));
}

#[test]
fn parse_nested_generic_type() {
    let ts = quote! { HashMap<String, Vec<Option<i32>>> };
    let ty: Type = parse2(ts).unwrap();
    assert!(matches!(ty, Type::Path(_)));
}

#[test]
fn parse_item_roundtrip_via_tokenstream() {
    let original = quote! {
        pub struct Config<'a, T: Clone> where T: Send {
            name: &'a str,
            value: T,
        }
    };
    let parsed: ItemStruct = parse2(original.clone()).unwrap();
    assert_eq!(parsed.ident, "Config");
    assert_eq!(parsed.generics.params.len(), 2);
    assert!(parsed.generics.where_clause.is_some());
    assert_eq!(parsed.fields.len(), 2);
    assert!(matches!(parsed.vis, Visibility::Public(_)));
}

#[test]
fn parse_attribute_inner_tokens() {
    let ts = quote! {
        #[allow(unused_variables, dead_code)]
        fn suppressed() {}
    };
    let item: ItemFn = parse2(ts).unwrap();
    let attr: &Attribute = &item.attrs[0];
    assert!(attr.path().is_ident("allow"));
}

#[test]
fn parse_raw_identifier_field() {
    let ts = quote! {
        struct WithRaw {
            r#type: String,
        }
    };
    let item: ItemStruct = parse2(ts).unwrap();
    let field = item.fields.iter().next().unwrap();
    assert_eq!(field.ident.as_ref().unwrap().to_string(), "r#type");
}

#[test]
fn parse_item_enum_generic() {
    let ts = quote! {
        enum Either<L, R> {
            Left(L),
            Right(R),
        }
    };
    let item: ItemEnum = parse2(ts).unwrap();
    assert_eq!(item.generics.params.len(), 2);
    assert_eq!(item.variants.len(), 2);
}

#[test]
fn parse_trait_with_const() {
    let ts = quote! {
        trait HasMax {
            const MAX: usize;
        }
    };
    let item: ItemTrait = parse2(ts).unwrap();
    assert!(matches!(&item.items[0], TraitItem::Const(_)));
}

#[test]
fn parse_extern_fn() {
    let ts = quote! {
        extern "C" fn callback(x: i32) -> i32 { x }
    };
    let item: ItemFn = parse2(ts).unwrap();
    assert!(item.sig.abi.is_some());
}

#[test]
fn parse_generic_fn_returning_impl_trait() {
    let ts = quote! {
        fn make_iter() -> impl Iterator<Item = i32> {
            vec![1, 2, 3].into_iter()
        }
    };
    let item: ItemFn = parse2(ts).unwrap();
    if let ReturnType::Type(_, ty) = &item.sig.output {
        assert!(matches!(ty.as_ref(), Type::ImplTrait(_)));
    } else {
        panic!("expected return type");
    }
}
