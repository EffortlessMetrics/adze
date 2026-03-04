#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for grammar annotation patterns used by adze macros.
//!
//! Covers: struct definitions with Box/Vec/Option fields, enum definitions with
//! mixed variants, repr attributes, doc comments, tuple structs, generic bounds,
//! TokenStream roundtrips, ident naming, type path resolution, nested generics,
//! attribute argument parsing, and multiple attribute combinations.

use std::collections::HashSet;

use adze_common::{NameValueExpr, filter_inner_type, try_extract_inner_type};
use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::{
    Attribute, DeriveInput, Expr, Fields, GenericParam, Item, ItemEnum, ItemMod, ItemStruct,
    Lifetime, Meta, Token, Type, TypePath, Visibility, WhereClause, parse_quote, parse2,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn parse_struct(tokens: TokenStream) -> ItemStruct {
    parse2(tokens).expect("failed to parse struct")
}

fn parse_enum(tokens: TokenStream) -> ItemEnum {
    parse2(tokens).expect("failed to parse enum")
}

fn parse_mod(tokens: TokenStream) -> ItemMod {
    parse2(tokens).expect("failed to parse module")
}

fn module_items(m: &ItemMod) -> &[Item] {
    &m.content.as_ref().expect("module has no content").1
}

fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == name
}

fn adze_attr_names(attrs: &[Attribute]) -> Vec<String> {
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

fn field_types(s: &ItemStruct) -> Vec<String> {
    match &s.fields {
        Fields::Named(f) => f
            .named
            .iter()
            .map(|f| f.ty.to_token_stream().to_string())
            .collect(),
        Fields::Unnamed(f) => f
            .unnamed
            .iter()
            .map(|f| f.ty.to_token_stream().to_string())
            .collect(),
        Fields::Unit => vec![],
    }
}

fn field_names(s: &ItemStruct) -> Vec<String> {
    match &s.fields {
        Fields::Named(f) => f
            .named
            .iter()
            .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
            .collect(),
        _ => vec![],
    }
}

fn leaf_params(attr: &Attribute) -> Punctuated<NameValueExpr, Token![,]> {
    attr.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap()
}

fn type_name(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn skip_set(names: &[&str]) -> HashSet<&str> {
    names.iter().copied().collect()
}

// =============================================================================
// Section 1: Struct definitions with Box<T> fields (recursive AST) (tests 1-8)
// =============================================================================

#[test]
fn box_field_single() {
    let s = parse_struct(quote! {
        struct BinaryExpr {
            left: Box<Expr>,
        }
    });
    assert_eq!(field_types(&s), vec!["Box < Expr >"]);
}

#[test]
fn box_field_recursive_pair() {
    let s = parse_struct(quote! {
        struct IfElse {
            condition: Box<Expr>,
            then_branch: Box<Stmt>,
            else_branch: Box<Stmt>,
        }
    });
    assert_eq!(
        field_names(&s),
        vec!["condition", "then_branch", "else_branch"]
    );
    assert_eq!(field_types(&s).len(), 3);
}

#[test]
fn box_extract_inner_type() {
    let ty: Type = parse_quote!(Box<Expr>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &HashSet::new());
    assert!(found);
    assert_eq!(type_name(&inner), "Expr");
}

#[test]
fn box_filter_inner_type() {
    let ty: Type = parse_quote!(Box<Stmt>);
    let filtered = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(type_name(&filtered), "Stmt");
}

#[test]
fn box_nested_in_option() {
    let ty: Type = parse_quote!(Option<Box<Expr>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip_set(&["Box"]));
    assert!(found);
    assert_eq!(type_name(&inner), "Expr");
}

#[test]
fn box_field_with_adze_leaf() {
    let s: ItemStruct = parse_quote! {
        struct Wrapper {
            #[adze::leaf(text = r"\+")] op: (),
            child: Box<Expr>,
        }
    };
    let leaf_attrs: Vec<_> = s.fields.iter().flat_map(|f| &f.attrs).collect();
    assert!(leaf_attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

#[test]
fn box_field_with_path_type() {
    let ty: Type = parse_quote!(Box<crate::ast::Expr>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &HashSet::new());
    assert!(found);
    assert!(type_name(&inner).contains("ast"));
}

#[test]
fn box_field_preserves_struct_ident() {
    let s = parse_struct(quote! {
        struct RecursiveNode {
            child: Box<RecursiveNode>,
        }
    });
    assert_eq!(s.ident.to_string(), "RecursiveNode");
    assert_eq!(field_types(&s), vec!["Box < RecursiveNode >"]);
}

// =============================================================================
// Section 2: Struct definitions with Vec<T> fields (tests 9-16)
// =============================================================================

#[test]
fn vec_field_single() {
    let s = parse_struct(quote! {
        struct Block {
            stmts: Vec<Stmt>,
        }
    });
    assert_eq!(field_types(&s), vec!["Vec < Stmt >"]);
}

#[test]
fn vec_extract_inner_type() {
    let ty: Type = parse_quote!(Vec<Stmt>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
    assert!(found);
    assert_eq!(type_name(&inner), "Stmt");
}

#[test]
fn vec_filter_inner_type() {
    let ty: Type = parse_quote!(Vec<Token>);
    let filtered = filter_inner_type(&ty, &skip_set(&["Vec"]));
    assert_eq!(type_name(&filtered), "Token");
}

#[test]
fn vec_with_box_inside() {
    let ty: Type = parse_quote!(Vec<Box<Expr>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip_set(&["Box"]));
    assert!(found);
    assert_eq!(type_name(&inner), "Expr");
}

#[test]
fn vec_multiple_fields() {
    let s = parse_struct(quote! {
        struct Program {
            imports: Vec<Import>,
            functions: Vec<Function>,
            constants: Vec<Constant>,
        }
    });
    assert_eq!(field_names(&s).len(), 3);
    for ty in field_types(&s) {
        assert!(ty.starts_with("Vec"));
    }
}

#[test]
fn vec_of_option() {
    let ty: Type = parse_quote!(Vec<Option<Ident>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
    assert!(found);
    assert!(type_name(&inner).contains("Option"));
}

#[test]
fn vec_field_in_module_struct() {
    let m = parse_mod(quote! {
        mod grammar {
            struct Items {
                elems: Vec<Elem>,
            }
        }
    });
    let items = module_items(&m);
    assert_eq!(items.len(), 1);
    if let Item::Struct(s) = &items[0] {
        assert_eq!(field_types(s), vec!["Vec < Elem >"]);
    } else {
        panic!("expected struct");
    }
}

#[test]
fn vec_empty_not_extracted() {
    let ty: Type = parse_quote!(String);
    let (_, found) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
    assert!(!found);
}

// =============================================================================
// Section 3: Struct definitions with Option<T> fields (tests 17-24)
// =============================================================================

#[test]
fn option_field_single() {
    let s = parse_struct(quote! {
        struct MaybeTyped {
            type_ann: Option<TypeExpr>,
        }
    });
    assert_eq!(field_types(&s), vec!["Option < TypeExpr >"]);
}

#[test]
fn option_extract_inner() {
    let ty: Type = parse_quote!(Option<TypeExpr>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(found);
    assert_eq!(type_name(&inner), "TypeExpr");
}

#[test]
fn option_filter_inner() {
    let ty: Type = parse_quote!(Option<Ident>);
    let filtered = filter_inner_type(&ty, &skip_set(&["Option"]));
    assert_eq!(type_name(&filtered), "Ident");
}

#[test]
fn option_of_box() {
    let ty: Type = parse_quote!(Option<Box<Expr>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip_set(&["Box"]));
    assert!(found);
    assert_eq!(type_name(&inner), "Expr");
}

#[test]
fn option_of_vec() {
    let ty: Type = parse_quote!(Option<Vec<Stmt>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(found);
    assert!(type_name(&inner).contains("Vec"));
}

#[test]
fn option_mixed_with_required() {
    let s = parse_struct(quote! {
        struct FuncDecl {
            name: Ident,
            ret: Option<TypeExpr>,
            body: Block,
        }
    });
    let types = field_types(&s);
    assert!(!types[0].contains("Option"));
    assert!(types[1].contains("Option"));
    assert!(!types[2].contains("Option"));
}

#[test]
fn option_field_with_skip() {
    let s: ItemStruct = parse_quote! {
        struct WithSkip {
            #[adze::skip]
            extra: Option<String>,
            value: Expr,
        }
    };
    let skip_field = s.fields.iter().next().unwrap();
    assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
}

#[test]
fn option_not_extracted_for_non_option() {
    let ty: Type = parse_quote!(Result<Expr, Error>);
    let (_, found) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(!found);
}

// =============================================================================
// Section 4: Enum definitions with mixed variants (tests 25-34)
// =============================================================================

#[test]
fn enum_unit_variants() {
    let e = parse_enum(quote! {
        enum Operator { Add, Sub, Mul, Div }
    });
    assert_eq!(e.variants.len(), 4);
    for v in &e.variants {
        assert!(matches!(v.fields, Fields::Unit));
    }
}

#[test]
fn enum_tuple_variants() {
    let e = parse_enum(quote! {
        enum Value {
            Number(f64),
            Text(String),
        }
    });
    for v in &e.variants {
        match &v.fields {
            Fields::Unnamed(f) => assert_eq!(f.unnamed.len(), 1),
            _ => panic!("expected unnamed fields"),
        }
    }
}

#[test]
fn enum_struct_variant() {
    let e = parse_enum(quote! {
        enum Decl {
            Function { name: Ident, body: Block },
        }
    });
    match &e.variants[0].fields {
        Fields::Named(f) => assert_eq!(f.named.len(), 2),
        _ => panic!("expected named fields"),
    }
}

#[test]
fn enum_mixed_variant_kinds() {
    let e = parse_enum(quote! {
        enum Node {
            Empty,
            Leaf(String),
            Branch { left: Box<Node>, right: Box<Node> },
        }
    });
    assert!(matches!(e.variants[0].fields, Fields::Unit));
    assert!(matches!(e.variants[1].fields, Fields::Unnamed(_)));
    assert!(matches!(e.variants[2].fields, Fields::Named(_)));
}

#[test]
fn enum_with_adze_leaf_on_variants() {
    let e: ItemEnum = parse_quote! {
        enum Token {
            #[adze::leaf(text = r"\+")]
            Plus,
            #[adze::leaf(text = r"\-")]
            Minus,
        }
    };
    for v in &e.variants {
        assert_eq!(adze_attr_names(&v.attrs), vec!["leaf"]);
    }
}

#[test]
fn enum_with_prec_on_variants() {
    let e: ItemEnum = parse_quote! {
        enum BinOp {
            #[adze::prec(1)]
            Add(Box<Expr>),
            #[adze::prec(2)]
            Mul(Box<Expr>),
        }
    };
    for v in &e.variants {
        assert_eq!(adze_attr_names(&v.attrs), vec!["prec"]);
        let attr = v.attrs.iter().find(|a| is_adze_attr(a, "prec")).unwrap();
        let expr: syn::Expr = attr.parse_args().unwrap();
        assert!(matches!(expr, syn::Expr::Lit(_)));
    }
}

#[test]
fn enum_with_language_attr() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        enum Expr {
            Num(f64),
            Add(Box<Expr>),
        }
    };
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn enum_variant_names() {
    let e = parse_enum(quote! {
        enum Stmt { Let, Return, Expr, Block }
    });
    let names: Vec<_> = e.variants.iter().map(|v| v.ident.to_string()).collect();
    assert_eq!(names, vec!["Let", "Return", "Expr", "Block"]);
}

#[test]
fn enum_variant_count_after_roundtrip() {
    let tokens = quote! { enum Op { A, B, C, D, E } };
    let e: ItemEnum = parse2(tokens.clone()).unwrap();
    assert_eq!(e.variants.len(), 5);
    let re_tokens = e.to_token_stream();
    let e2: ItemEnum = parse2(re_tokens).unwrap();
    assert_eq!(e2.variants.len(), 5);
}

#[test]
fn enum_in_module_with_grammar() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            enum Root {
                A,
                B(Box<Root>),
            }
        }
    });
    assert!(m.attrs.iter().any(|a| is_adze_attr(a, "grammar")));
    let items = module_items(&m);
    assert!(items.iter().any(|i| matches!(i, Item::Enum(_))));
}

// =============================================================================
// Section 5: Parse repr attributes (tests 35-39)
// =============================================================================

#[test]
fn repr_on_enum() {
    let e: ItemEnum = parse_quote! {
        #[repr(u8)]
        enum Kind { A, B, C }
    };
    let paths: Vec<_> = e
        .attrs
        .iter()
        .map(|a| a.path().to_token_stream().to_string())
        .collect();
    assert!(paths.contains(&"repr".to_string()));
}

#[test]
fn repr_c_on_struct() {
    let s: ItemStruct = parse_quote! {
        #[repr(C)]
        struct FfiNode { kind: u32, start: u32 }
    };
    assert!(s.attrs.iter().any(|a| a.path().is_ident("repr")));
}

#[test]
fn repr_transparent() {
    let s: ItemStruct = parse_quote! {
        #[repr(transparent)]
        struct Wrapper(Inner);
    };
    let meta = &s.attrs[0].meta;
    assert!(matches!(meta, Meta::List(_)));
}

#[test]
fn repr_combined_with_adze() {
    let e: ItemEnum = parse_quote! {
        #[repr(u16)]
        #[adze::language]
        enum Token { Plus, Minus }
    };
    assert_eq!(e.attrs.len(), 2);
    assert!(e.attrs.iter().any(|a| a.path().is_ident("repr")));
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn repr_arg_extraction() {
    let e: ItemEnum = parse_quote! {
        #[repr(i32)]
        enum Signed { A, B }
    };
    let repr_attr = &e.attrs[0];
    let inner: TokenStream = repr_attr.parse_args().unwrap();
    assert_eq!(inner.to_string(), "i32");
}

// =============================================================================
// Section 6: Parse doc comments as attributes (tests 40-45)
// =============================================================================

#[test]
fn doc_comment_on_struct() {
    let s: ItemStruct = parse_quote! {
        /// This is a node.
        struct Node { val: i32 }
    };
    assert!(s.attrs.iter().any(|a| a.path().is_ident("doc")));
}

#[test]
fn doc_comment_on_enum() {
    let e: ItemEnum = parse_quote! {
        /// An expression.
        enum Expr { Lit(i64), Add(Box<Expr>) }
    };
    assert!(!e.attrs.is_empty());
    assert!(e.attrs[0].path().is_ident("doc"));
}

#[test]
fn doc_comment_content_extraction() {
    let s: ItemStruct = parse_quote! {
        /// Hello world
        struct Documented;
    };
    let doc_attr = &s.attrs[0];
    if let Meta::NameValue(nv) = &doc_attr.meta {
        let val = nv.value.to_token_stream().to_string();
        assert!(val.contains("Hello world"));
    } else {
        panic!("expected name-value meta");
    }
}

#[test]
fn multiple_doc_lines() {
    let s: ItemStruct = parse_quote! {
        /// Line one.
        /// Line two.
        /// Line three.
        struct MultiDoc;
    };
    assert_eq!(s.attrs.len(), 3);
    for a in &s.attrs {
        assert!(a.path().is_ident("doc"));
    }
}

#[test]
fn doc_plus_adze_attr() {
    let s: ItemStruct = parse_quote! {
        /// Grammar root.
        #[adze::language]
        struct Root;
    };
    assert_eq!(s.attrs.len(), 2);
    assert!(s.attrs[0].path().is_ident("doc"));
    assert!(is_adze_attr(&s.attrs[1], "language"));
}

#[test]
fn doc_on_enum_variant() {
    let e: ItemEnum = parse_quote! {
        enum Op {
            /// Addition
            Add,
            /// Subtraction
            Sub,
        }
    };
    for v in &e.variants {
        assert!(v.attrs.iter().any(|a| a.path().is_ident("doc")));
    }
}

// =============================================================================
// Section 7: Parse tuple structs (tests 46-50)
// =============================================================================

#[test]
fn tuple_struct_single() {
    let s = parse_struct(quote! { struct Wrapper(Expr); });
    match &s.fields {
        Fields::Unnamed(f) => assert_eq!(f.unnamed.len(), 1),
        _ => panic!("expected unnamed"),
    }
}

#[test]
fn tuple_struct_multiple() {
    let s = parse_struct(quote! { struct Pair(Expr, Expr); });
    match &s.fields {
        Fields::Unnamed(f) => assert_eq!(f.unnamed.len(), 2),
        _ => panic!("expected unnamed"),
    }
}

#[test]
fn tuple_struct_with_box() {
    let s = parse_struct(quote! { struct Indirect(Box<Node>); });
    let types = field_types(&s);
    assert_eq!(types.len(), 1);
    assert!(types[0].contains("Box"));
}

#[test]
fn tuple_struct_with_visibility() {
    let s: ItemStruct = parse_quote! { pub struct PubTuple(pub Expr); };
    assert!(matches!(s.vis, Visibility::Public(_)));
    let field = s.fields.iter().next().unwrap();
    assert!(matches!(field.vis, Visibility::Public(_)));
}

#[test]
fn tuple_struct_field_types() {
    let s = parse_struct(quote! { struct Triple(i32, String, bool); });
    let types = field_types(&s);
    assert_eq!(types.len(), 3);
    assert_eq!(types[0], "i32");
    assert_eq!(types[1], "String");
    assert_eq!(types[2], "bool");
}

// =============================================================================
// Section 8: Parse generic bounds (tests 51-56)
// =============================================================================

#[test]
fn struct_with_lifetime() {
    let s: ItemStruct = parse_quote! {
        struct Ref<'a> { data: &'a str }
    };
    assert_eq!(s.generics.params.len(), 1);
    assert!(matches!(&s.generics.params[0], GenericParam::Lifetime(_)));
}

#[test]
fn struct_with_type_param() {
    let s: ItemStruct = parse_quote! {
        struct Container<T> { value: T }
    };
    assert_eq!(s.generics.params.len(), 1);
    assert!(matches!(&s.generics.params[0], GenericParam::Type(_)));
}

#[test]
fn struct_with_where_clause() {
    let s: ItemStruct = parse_quote! {
        struct Bounded<T> where T: Clone { value: T }
    };
    assert!(s.generics.where_clause.is_some());
    let wc = s.generics.where_clause.as_ref().unwrap();
    assert_eq!(wc.predicates.len(), 1);
}

#[test]
fn enum_with_lifetime() {
    let e: ItemEnum = parse_quote! {
        enum Cow<'a> {
            Borrowed(&'a str),
            Owned(String),
        }
    };
    assert_eq!(e.generics.params.len(), 1);
}

#[test]
fn multiple_generic_params() {
    let s: ItemStruct = parse_quote! {
        struct Map<K, V> { key: K, val: V }
    };
    assert_eq!(s.generics.params.len(), 2);
}

#[test]
fn generic_with_bound() {
    let s: ItemStruct = parse_quote! {
        struct Sortable<T: Ord + Clone> { items: Vec<T> }
    };
    if let GenericParam::Type(tp) = &s.generics.params[0] {
        assert_eq!(tp.bounds.len(), 2);
    } else {
        panic!("expected type param");
    }
}

// =============================================================================
// Section 9: TokenStream from/to string roundtrip (tests 57-63)
// =============================================================================

#[test]
fn roundtrip_struct() {
    let original = quote! { struct Foo { x: i32 } };
    let s: ItemStruct = parse2(original).unwrap();
    let back = s.to_token_stream();
    let s2: ItemStruct = parse2(back).unwrap();
    assert_eq!(s2.ident.to_string(), "Foo");
}

#[test]
fn roundtrip_enum() {
    let original = quote! { enum Bar { A, B(i32) } };
    let e: ItemEnum = parse2(original).unwrap();
    let back = e.to_token_stream();
    let e2: ItemEnum = parse2(back).unwrap();
    assert_eq!(e2.variants.len(), 2);
}

#[test]
fn roundtrip_module() {
    let original = quote! {
        mod m {
            struct S;
            enum E { V }
        }
    };
    let m: ItemMod = parse2(original).unwrap();
    let back = m.to_token_stream();
    let m2: ItemMod = parse2(back).unwrap();
    assert_eq!(module_items(&m2).len(), 2);
}

#[test]
fn tokenstream_from_string() {
    let code = "struct FromStr { val: u64 }";
    let ts: TokenStream = code.parse().unwrap();
    let s: ItemStruct = parse2(ts).unwrap();
    assert_eq!(s.ident.to_string(), "FromStr");
}

#[test]
fn tokenstream_to_string_deterministic() {
    let ts1 = quote! { fn foo() {} };
    let ts2 = quote! { fn foo() {} };
    assert_eq!(ts1.to_string(), ts2.to_string());
}

#[test]
fn tokenstream_preserves_attributes() {
    let original = quote! {
        #[adze::language]
        struct Root { val: Expr }
    };
    let s: ItemStruct = parse2(original).unwrap();
    let back = s.to_token_stream();
    let s2: ItemStruct = parse2(back).unwrap();
    assert!(s2.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn empty_tokenstream() {
    let ts = TokenStream::new();
    assert!(ts.is_empty());
    assert_eq!(ts.to_string(), "");
}

// =============================================================================
// Section 10: Ident naming patterns for grammar rules (tests 64-70)
// =============================================================================

#[test]
fn ident_from_string() {
    let id = Ident::new("expression", Span::call_site());
    assert_eq!(id.to_string(), "expression");
}

#[test]
fn ident_snake_case() {
    let id = Ident::new("binary_expr", Span::call_site());
    assert_eq!(id.to_string(), "binary_expr");
}

#[test]
fn ident_pascal_case() {
    let id = Ident::new("BinaryExpr", Span::call_site());
    assert_eq!(id.to_string(), "BinaryExpr");
}

#[test]
fn ident_raw_keyword() {
    let id = Ident::new_raw("r#type", Span::call_site());
    assert_eq!(id.to_string(), "r#type");
}

#[test]
fn ident_comparison() {
    let a = Ident::new("foo", Span::call_site());
    let b = Ident::new("foo", Span::call_site());
    assert_eq!(a, b);
}

#[test]
fn ident_in_struct() {
    let s: ItemStruct = parse_quote! { struct my_rule; };
    assert_eq!(s.ident.to_string(), "my_rule");
}

#[test]
fn ident_underscore_prefix() {
    let s: ItemStruct = parse_quote! { struct _Hidden { val: i32 } };
    assert_eq!(s.ident.to_string(), "_Hidden");
}

// =============================================================================
// Section 11: Type path resolution (tests 71-76)
// =============================================================================

#[test]
fn type_path_simple() {
    let ty: Type = parse_quote!(Expr);
    if let Type::Path(tp) = &ty {
        assert_eq!(tp.path.segments.len(), 1);
        assert_eq!(tp.path.segments[0].ident.to_string(), "Expr");
    } else {
        panic!("expected TypePath");
    }
}

#[test]
fn type_path_qualified() {
    let ty: Type = parse_quote!(crate::ast::Node);
    if let Type::Path(tp) = &ty {
        assert_eq!(tp.path.segments.len(), 3);
    } else {
        panic!("expected TypePath");
    }
}

#[test]
fn type_path_with_generic() {
    let ty: Type = parse_quote!(Vec<i32>);
    if let Type::Path(tp) = &ty {
        let seg = &tp.path.segments[0];
        assert_eq!(seg.ident.to_string(), "Vec");
        assert!(matches!(
            seg.arguments,
            syn::PathArguments::AngleBracketed(_)
        ));
    } else {
        panic!("expected TypePath");
    }
}

#[test]
fn type_path_std_prefix() {
    let ty: Type = parse_quote!(std::collections::HashMap<String, i32>);
    if let Type::Path(tp) = &ty {
        assert_eq!(tp.path.segments.len(), 3);
        assert_eq!(tp.path.segments[2].ident.to_string(), "HashMap");
    } else {
        panic!("expected TypePath");
    }
}

#[test]
fn type_reference() {
    let ty: Type = parse_quote!(&str);
    assert!(matches!(ty, Type::Reference(_)));
}

#[test]
fn type_tuple() {
    let ty: Type = parse_quote!((i32, String));
    assert!(matches!(ty, Type::Tuple(_)));
}

// =============================================================================
// Section 12: Nested generic types (tests 77-82)
// =============================================================================

#[test]
fn nested_vec_option() {
    let ty: Type = parse_quote!(Vec<Option<Expr>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
    assert!(found);
    assert!(type_name(&inner).contains("Option"));
}

#[test]
fn nested_option_box() {
    let ty: Type = parse_quote!(Option<Box<Stmt>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip_set(&["Box"]));
    assert!(found);
    assert_eq!(type_name(&inner), "Stmt");
}

#[test]
fn nested_vec_box() {
    let ty: Type = parse_quote!(Vec<Box<Node>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip_set(&["Box"]));
    assert!(found);
    assert_eq!(type_name(&inner), "Node");
}

#[test]
fn triple_nested_option_vec_box() {
    let s = parse_struct(quote! {
        struct Deep {
            items: Option<Vec<Box<Expr>>>,
        }
    });
    let types = field_types(&s);
    assert!(types[0].contains("Option"));
    assert!(types[0].contains("Vec"));
    assert!(types[0].contains("Box"));
}

#[test]
fn deeply_nested_filter() {
    let ty: Type = parse_quote!(Box<Vec<Option<Leaf>>>);
    let filtered = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert!(type_name(&filtered).contains("Vec"));
    assert!(!type_name(&filtered).contains("Box"));
}

#[test]
fn nested_generics_in_struct_field() {
    let s = parse_struct(quote! {
        struct Collector {
            required: Vec<Box<Expr>>,
            optional: Option<Vec<Token>>,
        }
    });
    assert_eq!(field_names(&s), vec!["required", "optional"]);
    assert!(field_types(&s)[0].contains("Vec"));
    assert!(field_types(&s)[1].contains("Option"));
}

// =============================================================================
// Section 13: Attribute argument parsing patterns (tests 83-89)
// =============================================================================

#[test]
fn leaf_attr_text_param() {
    let s: ItemStruct = parse_quote! {
        struct Plus {
            #[adze::leaf(text = r"\+")]
            token: (),
        }
    };
    let attr = s
        .fields
        .iter()
        .next()
        .unwrap()
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].path.to_string(), "text");
}

#[test]
fn leaf_attr_multiple_params() {
    let s: ItemStruct = parse_quote! {
        struct Kw {
            #[adze::leaf(text = "let", transform = "|v| v")]
            kw: (),
        }
    };
    let attr = s
        .fields
        .iter()
        .next()
        .unwrap()
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    assert_eq!(params.len(), 2);
}

#[test]
fn prec_attr_integer_arg() {
    let e: ItemEnum = parse_quote! {
        enum Op {
            #[adze::prec(5)]
            Add(Box<Expr>),
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec"))
        .unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(i),
        ..
    }) = expr
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 5);
    } else {
        panic!("expected int lit");
    }
}

#[test]
fn grammar_attr_string_arg() {
    let m: ItemMod = parse_quote! {
        #[adze::grammar("arithmetic")]
        mod arith {}
    };
    let attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = expr
    {
        assert_eq!(s.value(), "arithmetic");
    } else {
        panic!("expected string lit");
    }
}

#[test]
fn attr_with_no_args() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        struct Root;
    };
    let attr = &s.attrs[0];
    assert!(is_adze_attr(attr, "language"));
    assert!(matches!(attr.meta, Meta::Path(_)));
}

#[test]
fn attr_path_segment_count() {
    let s: ItemStruct = parse_quote! {
        #[adze::leaf(text = "x")]
        struct X;
    };
    let segs: Vec<_> = s.attrs[0].path().segments.iter().collect();
    assert_eq!(segs.len(), 2);
    assert_eq!(segs[0].ident.to_string(), "adze");
    assert_eq!(segs[1].ident.to_string(), "leaf");
}

#[test]
fn skip_attr_on_field() {
    let s: ItemStruct = parse_quote! {
        struct WithExtra {
            #[adze::skip]
            span: (u32, u32),
            value: Expr,
        }
    };
    let field0 = s.fields.iter().next().unwrap();
    assert!(field0.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    let field1 = s.fields.iter().nth(1).unwrap();
    assert!(field1.attrs.is_empty());
}

// =============================================================================
// Section 14: Multiple attribute combinations (tests 90-97)
// =============================================================================

#[test]
fn derive_plus_adze() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug, Clone)]
        #[adze::language]
        struct Root { val: i32 }
    };
    assert_eq!(s.attrs.len(), 2);
    assert!(s.attrs[0].path().is_ident("derive"));
    assert!(is_adze_attr(&s.attrs[1], "language"));
}

#[test]
fn doc_derive_adze_triple() {
    let s: ItemStruct = parse_quote! {
        /// Root node.
        #[derive(Debug)]
        #[adze::language]
        struct Root;
    };
    assert_eq!(s.attrs.len(), 3);
    assert!(s.attrs[0].path().is_ident("doc"));
    assert!(s.attrs[1].path().is_ident("derive"));
    assert!(is_adze_attr(&s.attrs[2], "language"));
}

#[test]
fn multiple_adze_attrs_on_field() {
    let s: ItemStruct = parse_quote! {
        struct Node {
            #[adze::leaf(text = r"\d+")]
            #[adze::prec(1)]
            num: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    let names = adze_attr_names(&field.attrs);
    assert_eq!(names, vec!["leaf", "prec"]);
}

#[test]
fn enum_variant_with_doc_and_leaf() {
    let e: ItemEnum = parse_quote! {
        enum Token {
            /// Plus sign.
            #[adze::leaf(text = r"\+")]
            Plus,
        }
    };
    let v = &e.variants[0];
    assert_eq!(v.attrs.len(), 2);
    assert!(v.attrs[0].path().is_ident("doc"));
    assert!(is_adze_attr(&v.attrs[1], "leaf"));
}

#[test]
fn cfg_plus_adze() {
    let s: ItemStruct = parse_quote! {
        #[cfg(feature = "full")]
        #[adze::language]
        struct Conditional;
    };
    assert_eq!(s.attrs.len(), 2);
    assert!(s.attrs[0].path().is_ident("cfg"));
}

#[test]
fn allow_plus_adze() {
    let s: ItemStruct = parse_quote! {
        #[allow(dead_code)]
        #[adze::language]
        struct Unused;
    };
    assert!(s.attrs[0].path().is_ident("allow"));
    assert!(is_adze_attr(&s.attrs[1], "language"));
}

#[test]
fn module_with_many_annotated_items() {
    let m = parse_mod(quote! {
        #[adze::grammar("complex")]
        mod grammar {
            /// Root.
            #[adze::language]
            enum Expr {
                #[adze::leaf(text = r"\d+")]
                Num(String),
                #[adze::prec(1)]
                Add(Box<Expr>),
            }

            struct Block {
                stmts: Vec<Stmt>,
            }

            /// Statement.
            enum Stmt {
                ExprStmt(Expr),
                LetStmt { name: String, value: Option<Expr> },
            }
        }
    });
    let items = module_items(&m);
    assert_eq!(items.len(), 3);
    // Verify the language root
    let root = items.iter().find_map(|i| match i {
        Item::Enum(e) if e.attrs.iter().any(|a| is_adze_attr(a, "language")) => Some(e),
        _ => None,
    });
    assert!(root.is_some());
    assert_eq!(root.unwrap().ident.to_string(), "Expr");
}

#[test]
fn prec_left_and_prec_right() {
    let e: ItemEnum = parse_quote! {
        enum BinOp {
            #[adze::prec_left(1)]
            Add(Box<Expr>),
            #[adze::prec_right(2)]
            Pow(Box<Expr>),
        }
    };
    let names0 = adze_attr_names(&e.variants[0].attrs);
    let names1 = adze_attr_names(&e.variants[1].attrs);
    assert_eq!(names0, vec!["prec_left"]);
    assert_eq!(names1, vec!["prec_right"]);
}

// =============================================================================
// Bonus: Additional edge-case tests (tests 98-101)
// =============================================================================

#[test]
fn derive_input_from_struct() {
    let di: DeriveInput = parse_quote! {
        #[adze::language]
        struct Root {
            child: Box<Node>,
        }
    };
    assert_eq!(di.ident.to_string(), "Root");
    assert!(di.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn derive_input_from_enum() {
    let di: DeriveInput = parse_quote! {
        enum Value {
            Int(i64),
            Float(f64),
            Str(String),
        }
    };
    assert_eq!(di.ident.to_string(), "Value");
    if let syn::Data::Enum(data) = &di.data {
        assert_eq!(data.variants.len(), 3);
    } else {
        panic!("expected enum data");
    }
}

#[test]
fn visibility_pub_crate() {
    let s: ItemStruct = parse_quote! {
        pub(crate) struct Internal { val: i32 }
    };
    assert!(matches!(s.vis, Visibility::Restricted(_)));
}

#[test]
fn where_clause_roundtrip() {
    let wc: WhereClause = parse_quote! { where T: Clone + Send, U: Sync };
    let ts = wc.to_token_stream();
    let wc2: WhereClause = parse2(ts).unwrap();
    assert_eq!(wc2.predicates.len(), 2);
}
