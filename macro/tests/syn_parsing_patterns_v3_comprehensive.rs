// Comprehensive tests for syn parsing patterns used in proc-macro development.
// Covers: Expr, Type, Statement, Function, Trait, Attribute, Use, Module,
// WhereClause, and pattern-matching arm parsing.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Expr, ExprArray, ExprBlock, ExprCall, ExprCast, ExprField, ExprIf, ExprIndex, ExprMatch,
    ExprMethodCall, Item, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, ItemUse, Pat, Stmt,
    Type, UseTree, parse2,
};

// =====================================================================
// Helpers
// =====================================================================

fn parse_expr(tokens: TokenStream) -> Expr {
    parse2::<Expr>(tokens).expect("Failed to parse Expr")
}

fn parse_type(tokens: TokenStream) -> Type {
    parse2::<Type>(tokens).expect("Failed to parse Type")
}

fn _parse_item(tokens: TokenStream) -> Item {
    parse2::<Item>(tokens).expect("Failed to parse Item")
}

fn parse_fn(tokens: TokenStream) -> ItemFn {
    parse2::<ItemFn>(tokens).expect("Failed to parse ItemFn")
}

// =====================================================================
// 1. Expr types
// =====================================================================

#[test]
fn expr_match_basic() {
    let e = parse_expr(quote! { match x { 1 => "one", _ => "other" } });
    assert!(matches!(e, Expr::Match(ExprMatch { .. })));
}

#[test]
fn expr_match_with_guard() {
    let e = parse_expr(quote! { match val { n if n > 0 => n, _ => 0 } });
    if let Expr::Match(m) = e {
        assert_eq!(m.arms.len(), 2);
        assert!(m.arms[0].guard.is_some());
    } else {
        panic!("Expected match expr");
    }
}

#[test]
fn expr_match_tuple_pattern() {
    let e = parse_expr(quote! { match pair { (a, b) => a + b } });
    if let Expr::Match(m) = e {
        assert_eq!(m.arms.len(), 1);
    } else {
        panic!("Expected match expr");
    }
}

#[test]
fn expr_if_basic() {
    let e = parse_expr(quote! { if cond { 1 } else { 2 } });
    assert!(matches!(e, Expr::If(ExprIf { .. })));
}

#[test]
fn expr_if_let() {
    let e = parse_expr(quote! { if let Some(x) = opt { x } else { 0 } });
    assert!(matches!(e, Expr::If(_)));
}

#[test]
fn expr_if_nested() {
    let e = parse_expr(quote! { if a { if b { 1 } else { 2 } } else { 3 } });
    assert!(matches!(e, Expr::If(_)));
}

#[test]
fn expr_block_simple() {
    let e = parse_expr(quote! { { let x = 1; x + 2 } });
    assert!(matches!(e, Expr::Block(ExprBlock { .. })));
}

#[test]
fn expr_block_empty() {
    let e = parse_expr(quote! { {} });
    assert!(matches!(e, Expr::Block(_)));
}

#[test]
fn expr_call_no_args() {
    let e = parse_expr(quote! { foo() });
    assert!(matches!(e, Expr::Call(ExprCall { .. })));
}

#[test]
fn expr_call_multiple_args() {
    let e = parse_expr(quote! { foo(1, "hello", true) });
    if let Expr::Call(c) = e {
        assert_eq!(c.args.len(), 3);
    } else {
        panic!("Expected call expr");
    }
}

#[test]
fn expr_call_nested() {
    let e = parse_expr(quote! { outer(inner(x)) });
    assert!(matches!(e, Expr::Call(_)));
}

#[test]
fn expr_method_call_basic() {
    let e = parse_expr(quote! { receiver.method() });
    assert!(matches!(e, Expr::MethodCall(ExprMethodCall { .. })));
}

#[test]
fn expr_method_call_chained() {
    let e = parse_expr(quote! { a.b().c().d() });
    assert!(matches!(e, Expr::MethodCall(_)));
}

#[test]
fn expr_method_call_with_turbofish() {
    let e = parse_expr(quote! { iter.collect::<Vec<_>>() });
    if let Expr::MethodCall(m) = e {
        assert!(m.turbofish.is_some());
    } else {
        panic!("Expected method call");
    }
}

#[test]
fn expr_field_access() {
    let e = parse_expr(quote! { point.x });
    assert!(matches!(e, Expr::Field(ExprField { .. })));
}

#[test]
fn expr_field_nested() {
    let e = parse_expr(quote! { a.b.c });
    assert!(matches!(e, Expr::Field(_)));
}

#[test]
fn expr_index_basic() {
    let e = parse_expr(quote! { arr[0] });
    assert!(matches!(e, Expr::Index(ExprIndex { .. })));
}

#[test]
fn expr_index_nested() {
    let e = parse_expr(quote! { matrix[i][j] });
    assert!(matches!(e, Expr::Index(_)));
}

#[test]
fn expr_cast_basic() {
    let e = parse_expr(quote! { x as u64 });
    assert!(matches!(e, Expr::Cast(ExprCast { .. })));
}

#[test]
fn expr_cast_chained() {
    let e = parse_expr(quote! { (x as i64) as u64 });
    assert!(matches!(e, Expr::Cast(_)));
}

#[test]
fn expr_closure() {
    let e = parse_expr(quote! { |x: i32| x + 1 });
    assert!(matches!(e, Expr::Closure(_)));
}

#[test]
fn expr_closure_move() {
    let e = parse_expr(quote! { move || value });
    assert!(matches!(e, Expr::Closure(_)));
}

#[test]
fn expr_array_literal() {
    let e = parse_expr(quote! { [1, 2, 3] });
    assert!(matches!(e, Expr::Array(ExprArray { .. })));
}

#[test]
fn expr_tuple() {
    let e = parse_expr(quote! { (1, "two", 3.0) });
    assert!(matches!(e, Expr::Tuple(_)));
}

#[test]
fn expr_reference() {
    let e = parse_expr(quote! { &mut x });
    assert!(matches!(e, Expr::Reference(_)));
}

#[test]
fn expr_loop() {
    let e = parse_expr(quote! { loop { break 42; } });
    assert!(matches!(e, Expr::Loop(_)));
}

#[test]
fn expr_while() {
    let e = parse_expr(quote! { while cond { step(); } });
    assert!(matches!(e, Expr::While(_)));
}

#[test]
fn expr_for_loop() {
    let e = parse_expr(quote! { for i in 0..10 { use_i(i); } });
    assert!(matches!(e, Expr::ForLoop(_)));
}

#[test]
fn expr_return() {
    let e = parse_expr(quote! { return Some(42) });
    assert!(matches!(e, Expr::Return(_)));
}

#[test]
fn expr_struct_literal() {
    let e = parse_expr(quote! { Point { x: 1, y: 2 } });
    assert!(matches!(e, Expr::Struct(_)));
}

// =====================================================================
// 2. Type patterns
// =====================================================================

#[test]
fn type_path_simple() {
    let t = parse_type(quote! { String });
    assert!(matches!(t, Type::Path(_)));
}

#[test]
fn type_path_qualified() {
    let t = parse_type(quote! { std::collections::HashMap<String, i32> });
    assert!(matches!(t, Type::Path(_)));
}

#[test]
fn type_reference_shared() {
    let t = parse_type(quote! { &str });
    assert!(matches!(t, Type::Reference(_)));
}

#[test]
fn type_reference_mutable() {
    let t = parse_type(quote! { &mut Vec<u8> });
    if let Type::Reference(r) = t {
        assert!(r.mutability.is_some());
    } else {
        panic!("Expected reference type");
    }
}

#[test]
fn type_reference_with_lifetime() {
    let t = parse_type(quote! { &'a str });
    if let Type::Reference(r) = t {
        assert!(r.lifetime.is_some());
    } else {
        panic!("Expected reference type");
    }
}

#[test]
fn type_array() {
    let t = parse_type(quote! { [u8; 32] });
    assert!(matches!(t, Type::Array(_)));
}

#[test]
fn type_tuple_pair() {
    let t = parse_type(quote! { (i32, String) });
    assert!(matches!(t, Type::Tuple(_)));
}

#[test]
fn type_tuple_unit() {
    let t = parse_type(quote! { () });
    assert!(matches!(t, Type::Tuple(_)));
}

#[test]
fn type_slice() {
    let t = parse_type(quote! { [u8] });
    assert!(matches!(t, Type::Slice(_)));
}

#[test]
fn type_never() {
    let t = parse_type(quote! { ! });
    assert!(matches!(t, Type::Never(_)));
}

#[test]
fn type_bare_fn() {
    let t = parse_type(quote! { fn(i32) -> bool });
    assert!(matches!(t, Type::BareFn(_)));
}

#[test]
fn type_impl_trait() {
    let t = parse_type(quote! { impl Iterator<Item = u8> });
    assert!(matches!(t, Type::ImplTrait(_)));
}

#[test]
fn type_dyn_trait() {
    let t = parse_type(quote! { dyn Iterator<Item = u8> });
    assert!(matches!(t, Type::TraitObject(_)));
}

#[test]
fn type_nested_generics() {
    let t = parse_type(quote! { Option<Vec<HashMap<String, Vec<u8>>>> });
    assert!(matches!(t, Type::Path(_)));
}

// =====================================================================
// 3. Statement patterns
// =====================================================================

#[test]
fn stmt_let_basic() {
    let block: ExprBlock = parse2(quote! { { let x = 5; } }).unwrap();
    let stmts = &block.block.stmts;
    assert_eq!(stmts.len(), 1);
    assert!(matches!(&stmts[0], Stmt::Local(_)));
}

#[test]
fn stmt_let_with_type() {
    let block: ExprBlock = parse2(quote! { { let x: i32 = 5; } }).unwrap();
    if let Stmt::Local(local) = &block.block.stmts[0] {
        // In syn 2.x the type annotation is part of the pattern (Pat::Type)
        assert!(matches!(&local.pat, Pat::Type(_)));
    } else {
        panic!("Expected let statement");
    }
}

#[test]
fn stmt_let_pattern_destructure() {
    let block: ExprBlock = parse2(quote! { { let (a, b) = pair; } }).unwrap();
    if let Stmt::Local(local) = &block.block.stmts[0] {
        assert!(matches!(&local.pat, Pat::Tuple(_)));
    } else {
        panic!("Expected let statement");
    }
}

#[test]
fn stmt_expr_without_semi() {
    let block: ExprBlock = parse2(quote! { { x + 1 } }).unwrap();
    assert_eq!(block.block.stmts.len(), 1);
    assert!(matches!(&block.block.stmts[0], Stmt::Expr(_, None)));
}

#[test]
fn stmt_expr_with_semi() {
    let block: ExprBlock = parse2(quote! { { foo(); } }).unwrap();
    assert_eq!(block.block.stmts.len(), 1);
    assert!(matches!(&block.block.stmts[0], Stmt::Expr(_, Some(_))));
}

#[test]
fn stmt_multiple() {
    let block: ExprBlock = parse2(quote! { { let a = 1; let b = 2; a + b } }).unwrap();
    assert_eq!(block.block.stmts.len(), 3);
}

// =====================================================================
// 4. Function signature patterns
// =====================================================================

#[test]
fn fn_no_params() {
    let f = parse_fn(quote! { fn noop() {} });
    assert_eq!(f.sig.ident.to_string(), "noop");
    assert!(f.sig.inputs.is_empty());
}

#[test]
fn fn_with_params() {
    let f = parse_fn(quote! { fn add(a: i32, b: i32) -> i32 { a + b } });
    assert_eq!(f.sig.inputs.len(), 2);
    assert!(f.sig.output != syn::ReturnType::Default);
}

#[test]
fn fn_async() {
    let f = parse_fn(quote! { async fn fetch() -> String { String::new() } });
    assert!(f.sig.asyncness.is_some());
}

#[test]
fn fn_unsafe() {
    let f = parse_fn(quote! { unsafe fn danger() {} });
    assert!(f.sig.unsafety.is_some());
}

#[test]
fn fn_const() {
    let f = parse_fn(quote! { const fn constant() -> i32 { 42 } });
    assert!(f.sig.constness.is_some());
}

#[test]
fn fn_generic() {
    let f = parse_fn(quote! { fn identity<T>(x: T) -> T { x } });
    assert_eq!(f.sig.generics.params.len(), 1);
}

#[test]
fn fn_lifetime_param() {
    let f = parse_fn(quote! { fn borrow<'a>(s: &'a str) -> &'a str { s } });
    assert_eq!(f.sig.generics.params.len(), 1);
}

#[test]
fn fn_where_clause() {
    let f = parse_fn(quote! { fn print<T>(x: T) where T: std::fmt::Display { } });
    assert!(f.sig.generics.where_clause.is_some());
}

#[test]
fn fn_return_impl_trait() {
    let f =
        parse_fn(quote! { fn make_iter() -> impl Iterator<Item = i32> { vec![1].into_iter() } });
    if let syn::ReturnType::Type(_, ty) = &f.sig.output {
        assert!(matches!(ty.as_ref(), Type::ImplTrait(_)));
    } else {
        panic!("Expected return type");
    }
}

#[test]
fn fn_variadic_style_generics() {
    let f = parse_fn(quote! { fn multi<A, B, C>(a: A, b: B, c: C) {} });
    assert_eq!(f.sig.generics.params.len(), 3);
    assert_eq!(f.sig.inputs.len(), 3);
}

// =====================================================================
// 5. Trait bounds
// =====================================================================

#[test]
fn trait_bound_single() {
    let f = parse_fn(quote! { fn show<T: Display>(t: T) {} });
    assert_eq!(f.sig.generics.params.len(), 1);
}

#[test]
fn trait_bound_multiple() {
    let f = parse_fn(quote! { fn show<T: Display + Debug>(t: T) {} });
    assert_eq!(f.sig.generics.params.len(), 1);
}

#[test]
fn trait_bound_where_multiple_types() {
    let f = parse_fn(quote! {
        fn combine<A, B>(a: A, b: B)
        where
            A: Clone + Send,
            B: Clone + Sync,
        {}
    });
    let wc = f.sig.generics.where_clause.as_ref().unwrap();
    assert_eq!(wc.predicates.len(), 2);
}

#[test]
fn trait_bound_lifetime_in_where() {
    let f = parse_fn(quote! {
        fn process<'a, T>(x: &'a T)
        where
            T: 'a + Clone,
        {}
    });
    assert!(f.sig.generics.where_clause.is_some());
}

#[test]
fn trait_definition_basic() {
    let item: ItemTrait = parse2(quote! {
        trait Greet {
            fn hello(&self) -> String;
        }
    })
    .unwrap();
    assert_eq!(item.ident.to_string(), "Greet");
    assert_eq!(item.items.len(), 1);
}

#[test]
fn trait_with_default_method() {
    let item: ItemTrait = parse2(quote! {
        trait Greet {
            fn hello(&self) -> String {
                String::from("hello")
            }
        }
    })
    .unwrap();
    assert_eq!(item.items.len(), 1);
}

#[test]
fn trait_with_associated_type() {
    let item: ItemTrait = parse2(quote! {
        trait Container {
            type Item;
            fn get(&self) -> &Self::Item;
        }
    })
    .unwrap();
    assert_eq!(item.items.len(), 2);
}

#[test]
fn trait_with_supertraits() {
    let item: ItemTrait = parse2(quote! {
        trait Printable: Display + Debug {}
    })
    .unwrap();
    assert_eq!(item.supertraits.len(), 2);
}

// =====================================================================
// 6. Attribute meta patterns
// =====================================================================

#[test]
fn attr_derive() {
    let s: ItemStruct = parse2(quote! {
        #[derive(Debug, Clone, PartialEq)]
        struct Foo { x: i32 }
    })
    .unwrap();
    assert_eq!(s.attrs.len(), 1);
}

#[test]
fn attr_multiple() {
    let s: ItemStruct = parse2(quote! {
        #[derive(Debug)]
        #[allow(dead_code)]
        struct Bar;
    })
    .unwrap();
    assert_eq!(s.attrs.len(), 2);
}

#[test]
fn attr_cfg() {
    let f = parse_fn(quote! {
        #[cfg(test)]
        fn test_only() {}
    });
    assert_eq!(f.attrs.len(), 1);
}

#[test]
fn attr_doc_comment() {
    let s: ItemStruct = parse2(quote! {
        /// A documented struct
        struct Documented;
    })
    .unwrap();
    assert!(!s.attrs.is_empty());
}

#[test]
fn attr_repr() {
    let s: ItemStruct = parse2(quote! {
        #[repr(C)]
        struct CCompat { a: u32, b: u64 }
    })
    .unwrap();
    assert_eq!(s.attrs.len(), 1);
}

// =====================================================================
// 7. Use statements
// =====================================================================

#[test]
fn use_simple_path() {
    let u: ItemUse = parse2(quote! { use std::collections::HashMap; }).unwrap();
    assert!(matches!(u.tree, UseTree::Path(_)));
}

#[test]
fn use_glob() {
    let u: ItemUse = parse2(quote! { use std::collections::*; }).unwrap();
    // The tree is a Path whose last segment is a Glob
    fn has_glob(tree: &UseTree) -> bool {
        match tree {
            UseTree::Glob(_) => true,
            UseTree::Path(p) => has_glob(&p.tree),
            _ => false,
        }
    }
    assert!(has_glob(&u.tree));
}

#[test]
fn use_rename() {
    let u: ItemUse = parse2(quote! { use std::collections::HashMap as Map; }).unwrap();
    fn has_rename(tree: &UseTree) -> bool {
        match tree {
            UseTree::Rename(_) => true,
            UseTree::Path(p) => has_rename(&p.tree),
            _ => false,
        }
    }
    assert!(has_rename(&u.tree));
}

#[test]
fn use_group() {
    let u: ItemUse = parse2(quote! { use std::{io, fs}; }).unwrap();
    fn has_group(tree: &UseTree) -> bool {
        match tree {
            UseTree::Group(_) => true,
            UseTree::Path(p) => has_group(&p.tree),
            _ => false,
        }
    }
    assert!(has_group(&u.tree));
}

#[test]
fn use_nested_group() {
    let u: ItemUse = parse2(quote! { use std::{io::{self, Read}, collections::HashMap}; }).unwrap();
    assert!(matches!(u.tree, UseTree::Path(_)));
}

// =====================================================================
// 8. Module patterns
// =====================================================================

#[test]
fn mod_empty_inline() {
    let m: ItemMod = parse2(quote! { mod empty {} }).unwrap();
    assert_eq!(m.ident.to_string(), "empty");
    assert!(m.content.is_some());
}

#[test]
fn mod_with_function() {
    let m: ItemMod = parse2(quote! {
        mod inner {
            fn helper() -> i32 { 42 }
        }
    })
    .unwrap();
    let (_, items) = m.content.unwrap();
    assert_eq!(items.len(), 1);
}

#[test]
fn mod_with_struct_and_impl() {
    let m: ItemMod = parse2(quote! {
        mod shapes {
            struct Circle { radius: f64 }
            impl Circle {
                fn area(&self) -> f64 { 3.14 * self.radius * self.radius }
            }
        }
    })
    .unwrap();
    let (_, items) = m.content.unwrap();
    assert_eq!(items.len(), 2);
}

#[test]
fn mod_pub() {
    let m: ItemMod = parse2(quote! { pub mod public {} }).unwrap();
    assert!(matches!(m.vis, syn::Visibility::Public(_)));
}

// =====================================================================
// 9. Where clauses
// =====================================================================

#[test]
fn where_clause_single_predicate() {
    let f = parse_fn(quote! { fn f<T>() where T: Clone {} });
    let wc = f.sig.generics.where_clause.unwrap();
    assert_eq!(wc.predicates.len(), 1);
}

#[test]
fn where_clause_multiple_predicates() {
    let f = parse_fn(quote! {
        fn f<T, U>() where T: Clone, U: Default {}
    });
    let wc = f.sig.generics.where_clause.unwrap();
    assert_eq!(wc.predicates.len(), 2);
}

#[test]
fn where_clause_complex_bounds() {
    let f = parse_fn(quote! {
        fn f<T>() where T: Iterator<Item = u8> + Send + 'static {}
    });
    let wc = f.sig.generics.where_clause.unwrap();
    assert_eq!(wc.predicates.len(), 1);
}

#[test]
fn where_clause_on_impl() {
    let imp: ItemImpl = parse2(quote! {
        impl<T> MyStruct<T> where T: Debug {
            fn show(&self) {}
        }
    })
    .unwrap();
    assert!(imp.generics.where_clause.is_some());
}

// =====================================================================
// 10. Pattern matching arms
// =====================================================================

#[test]
fn arm_literal_patterns() {
    let e = parse_expr(quote! {
        match x {
            0 => "zero",
            1 => "one",
            _ => "many",
        }
    });
    if let Expr::Match(m) = e {
        assert_eq!(m.arms.len(), 3);
    } else {
        panic!("Expected match");
    }
}

#[test]
fn arm_enum_variant() {
    let e = parse_expr(quote! {
        match opt {
            Some(val) => val,
            None => 0,
        }
    });
    if let Expr::Match(m) = e {
        assert_eq!(m.arms.len(), 2);
    } else {
        panic!("Expected match");
    }
}

#[test]
fn arm_struct_pattern() {
    let e = parse_expr(quote! {
        match point {
            Point { x, y: 0 } => x,
            Point { x: 0, y } => y,
            Point { x, y } => x + y,
        }
    });
    if let Expr::Match(m) = e {
        assert_eq!(m.arms.len(), 3);
    } else {
        panic!("Expected match");
    }
}

#[test]
fn arm_or_pattern() {
    let e = parse_expr(quote! {
        match x {
            1 | 2 | 3 => "small",
            _ => "big",
        }
    });
    if let Expr::Match(m) = e {
        assert_eq!(m.arms.len(), 2);
    } else {
        panic!("Expected match");
    }
}

#[test]
fn arm_nested_match() {
    let e = parse_expr(quote! {
        match outer {
            Some(inner) => match inner {
                Ok(v) => v,
                Err(_) => 0,
            },
            None => 0,
        }
    });
    assert!(matches!(e, Expr::Match(_)));
}

#[test]
fn arm_range_pattern() {
    let e = parse_expr(quote! {
        match ch {
            'a'..='z' => "lower",
            'A'..='Z' => "upper",
            _ => "other",
        }
    });
    if let Expr::Match(m) = e {
        assert_eq!(m.arms.len(), 3);
    } else {
        panic!("Expected match");
    }
}

#[test]
fn arm_tuple_destructure() {
    let e = parse_expr(quote! {
        match pair {
            (0, _) => "first zero",
            (_, 0) => "second zero",
            (a, b) => "neither",
        }
    });
    if let Expr::Match(m) = e {
        assert_eq!(m.arms.len(), 3);
    } else {
        panic!("Expected match");
    }
}

// =====================================================================
// Additional coverage: Impl blocks
// =====================================================================

#[test]
fn impl_basic() {
    let imp: ItemImpl = parse2(quote! {
        impl Foo {
            fn new() -> Self { Foo }
        }
    })
    .unwrap();
    assert!(imp.trait_.is_none());
    assert_eq!(imp.items.len(), 1);
}

#[test]
fn impl_trait_for_type() {
    let imp: ItemImpl = parse2(quote! {
        impl Display for Foo {
            fn fmt(&self, f: &mut Formatter) -> Result { Ok(()) }
        }
    })
    .unwrap();
    assert!(imp.trait_.is_some());
}

#[test]
fn impl_with_generics() {
    let imp: ItemImpl = parse2(quote! {
        impl<T: Clone> Container<T> {
            fn clone_inner(&self) -> T { self.inner.clone() }
        }
    })
    .unwrap();
    assert_eq!(imp.generics.params.len(), 1);
}

// =====================================================================
// Additional coverage: parse_str
// =====================================================================

#[test]
fn parse_str_expr() {
    let e: Expr = syn::parse_str("1 + 2 * 3").unwrap();
    assert!(matches!(e, Expr::Binary(_)));
}

#[test]
fn parse_str_type() {
    let t: Type = syn::parse_str("Vec<String>").unwrap();
    assert!(matches!(t, Type::Path(_)));
}

#[test]
fn parse_str_item_fn() {
    let f: ItemFn = syn::parse_str("fn hello() {}").unwrap();
    assert_eq!(f.sig.ident.to_string(), "hello");
}

// =====================================================================
// Additional coverage: quote roundtrip
// =====================================================================

#[test]
fn quote_roundtrip_struct() {
    let tokens = quote! { struct Roundtrip { field: u32 } };
    let s: ItemStruct = parse2(tokens.clone()).unwrap();
    assert_eq!(s.ident.to_string(), "Roundtrip");
    let reparsed: ItemStruct = parse2(quote! { #s }).unwrap();
    assert_eq!(reparsed.ident.to_string(), "Roundtrip");
}

#[test]
fn quote_roundtrip_fn() {
    let f = parse_fn(quote! { fn rt(x: i32) -> i32 { x } });
    let reparsed: ItemFn = parse2(quote! { #f }).unwrap();
    assert_eq!(reparsed.sig.ident.to_string(), "rt");
}

#[test]
fn quote_interpolation_ident() {
    let name = quote::format_ident!("dynamic_fn");
    let f = parse_fn(quote! { fn #name() {} });
    assert_eq!(f.sig.ident.to_string(), "dynamic_fn");
}
