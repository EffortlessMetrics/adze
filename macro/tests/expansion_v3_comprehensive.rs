#![allow(clippy::needless_range_loop)]

//! Comprehensive v3 tests for macro expansion building blocks.
//!
//! Covers: code generation patterns, quote interpolation, struct expansion
//! (named/unnamed/unit), enum expansion (named/unnamed/unit variants),
//! attribute-expansion relationship, grammar module expansion patterns,
//! error handling during expansion, and type generation patterns.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Expr, ExprLit, Fields, Item, ItemEnum, ItemMod, ItemStruct, Lit, Token, Type,
    Variant, parse_quote, parse_str,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

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

fn parse_mod(tokens: TokenStream) -> ItemMod {
    syn::parse2(tokens).expect("failed to parse module")
}

fn module_items(m: &ItemMod) -> &[Item] {
    &m.content.as_ref().expect("module has no content").1
}

fn leaf_params(attr: &Attribute) -> Punctuated<NameValueExpr, Token![,]> {
    attr.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap()
}

fn ty(s: &str) -> Type {
    parse_str::<Type>(s).unwrap()
}

fn ts(t: &Type) -> String {
    t.to_token_stream().to_string()
}

fn skip_set<'a>(names: &[&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn extract_grammar_name(m: &ItemMod) -> Option<String> {
    m.attrs.iter().find_map(|a| {
        if !is_adze_attr(a, "grammar") {
            return None;
        }
        let expr: Expr = a.parse_args().ok()?;
        if let Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) = expr
        {
            Some(s.value())
        } else {
            None
        }
    })
}

fn find_language_type(m: &ItemMod) -> Option<String> {
    module_items(m).iter().find_map(|item| match item {
        Item::Enum(e) if e.attrs.iter().any(|a| is_adze_attr(a, "language")) => {
            Some(e.ident.to_string())
        }
        Item::Struct(s) if s.attrs.iter().any(|a| is_adze_attr(a, "language")) => {
            Some(s.ident.to_string())
        }
        _ => None,
    })
}

fn find_struct_in_mod<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemStruct> {
    module_items(m).iter().find_map(|i| {
        if let Item::Struct(s) = i {
            if s.ident == name { Some(s) } else { None }
        } else {
            None
        }
    })
}

fn find_enum_in_mod<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemEnum> {
    module_items(m).iter().find_map(|i| {
        if let Item::Enum(e) = i {
            if e.ident == name { Some(e) } else { None }
        } else {
            None
        }
    })
}

fn variant_field_types(variant: &Variant) -> Vec<String> {
    match &variant.fields {
        Fields::Unnamed(u) => u
            .unnamed
            .iter()
            .map(|f| f.ty.to_token_stream().to_string())
            .collect(),
        Fields::Named(n) => n
            .named
            .iter()
            .map(|f| f.ty.to_token_stream().to_string())
            .collect(),
        Fields::Unit => vec![],
    }
}

fn prec_value(attr: &Attribute) -> i32 {
    let expr: Expr = attr.parse_args().unwrap();
    if let Expr::Lit(ExprLit {
        lit: Lit::Int(i), ..
    }) = expr
    {
        i.base10_parse::<i32>().unwrap()
    } else {
        panic!("Expected int literal in precedence attribute");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 1: Code generation patterns & quote interpolation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn quote_interpolates_ident() {
    let name = format_ident!("my_parser");
    let tokens = quote! { fn #name() {} };
    let s = tokens.to_string();
    assert!(s.contains("my_parser"), "Expected ident interpolation: {s}");
}

#[test]
fn quote_interpolates_type() {
    let field_ty: Type = parse_quote!(Vec<i32>);
    let tokens = quote! { let x: #field_ty = vec![]; };
    let s = tokens.to_string();
    assert!(s.contains("Vec"), "Expected type interpolation: {s}");
    assert!(s.contains("i32"), "Expected inner type: {s}");
}

#[test]
fn quote_interpolates_expr() {
    let val: Expr = parse_quote!(42);
    let tokens = quote! { let x = #val; };
    let s = tokens.to_string();
    assert!(s.contains("42"), "Expected expr interpolation: {s}");
}

#[test]
fn quote_repetition_pattern() {
    let names = vec![format_ident!("a"), format_ident!("b"), format_ident!("c")];
    let tokens = quote! { #(let #names = 0;)* };
    let s = tokens.to_string();
    assert!(s.contains("let a"), "Missing a: {s}");
    assert!(s.contains("let b"), "Missing b: {s}");
    assert!(s.contains("let c"), "Missing c: {s}");
}

#[test]
fn quote_nested_repetition_with_separator() {
    let items = ["x", "y", "z"];
    let idents: Vec<_> = items.iter().map(|i| format_ident!("{i}")).collect();
    let tokens = quote! { enum E { #(#idents),* } };
    let s = tokens.to_string();
    assert!(s.contains("x ,"), "Expected separated items: {s}");
}

#[test]
fn quote_conditional_tokens() {
    let include_extra = true;
    let extra = if include_extra {
        quote! { #[derive(Debug)] }
    } else {
        quote! {}
    };
    let tokens = quote! { #extra struct Foo; };
    let s = tokens.to_string();
    assert!(s.contains("derive"), "Expected conditional derive: {s}");
}

#[test]
fn format_ident_constructs_variant_symbol_name() {
    let enum_name = format_ident!("Expression");
    let variant_name = format_ident!("Number");
    let symbol = format!("{enum_name}_{variant_name}");
    assert_eq!(symbol, "Expression_Number");
}

#[test]
fn quote_generates_impl_block() {
    let struct_name = format_ident!("MyStruct");
    let tokens = quote! {
        impl #struct_name {
            fn extract() -> Self { todo!() }
        }
    };
    let s = tokens.to_string();
    assert!(s.contains("impl MyStruct"), "Expected impl block: {s}");
    assert!(s.contains("fn extract"), "Expected extract fn: {s}");
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 2: Struct expansion — named fields
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn struct_named_single_leaf_field() {
    let s: ItemStruct = parse_quote! {
        pub struct Identifier {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }
    };
    assert_eq!(s.fields.len(), 1);
    let f = s.fields.iter().next().unwrap();
    assert_eq!(f.ident.as_ref().unwrap(), "name");
    assert!(f.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

#[test]
fn struct_named_multiple_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct BinaryOp {
            left: Box<Expr>,
            #[adze::leaf(text = "+")]
            _op: (),
            right: Box<Expr>,
        }
    };
    assert_eq!(s.fields.len(), 3);
    let names: Vec<_> = s
        .fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect();
    assert_eq!(names, ["left", "_op", "right"]);
}

#[test]
fn struct_named_leaf_with_transform_has_both_params() {
    let s: ItemStruct = parse_quote! {
        pub struct Number {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: i32,
        }
    };
    let f = s.fields.iter().next().unwrap();
    let leaf_attr = f.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
    let params = leaf_params(leaf_attr);
    let param_names: Vec<_> = params.iter().map(|p| p.path.to_string()).collect();
    assert!(param_names.contains(&"pattern".to_string()));
    assert!(param_names.contains(&"transform".to_string()));
}

#[test]
fn struct_named_skip_field_preserves_type() {
    let s: ItemStruct = parse_quote! {
        pub struct MyNode {
            #[adze::leaf(pattern = r"\d+")]
            value: String,
            #[adze::skip(false)]
            visited: bool,
        }
    };
    let skip_field = s.fields.iter().nth(1).unwrap();
    assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    assert_eq!(skip_field.ty.to_token_stream().to_string(), "bool");
}

#[test]
fn struct_named_optional_field_type() {
    let s: ItemStruct = parse_quote! {
        pub struct MaybeNumber {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: Option<i32>,
        }
    };
    let (inner, found) = try_extract_inner_type(
        &s.fields.iter().next().unwrap().ty,
        "Option",
        &skip_set(&[]),
    );
    assert!(found);
    assert_eq!(ts(&inner), "i32");
}

#[test]
fn struct_named_vec_field_with_repeat() {
    let s: ItemStruct = parse_quote! {
        pub struct NumberList {
            #[adze::repeat(non_empty = true)]
            numbers: Vec<Number>,
        }
    };
    let f = s.fields.iter().next().unwrap();
    assert!(f.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
    let (inner, found) = try_extract_inner_type(&f.ty, "Vec", &skip_set(&[]));
    assert!(found);
    assert_eq!(ts(&inner), "Number");
}

#[test]
fn struct_named_delimited_field() {
    let s: ItemStruct = parse_quote! {
        pub struct CsvNumbers {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<Number>,
        }
    };
    let f = s.fields.iter().next().unwrap();
    assert!(f.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 3: Struct expansion — unnamed fields (tuple structs)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn struct_unnamed_single_field() {
    let s: ItemStruct = parse_quote! {
        pub struct Wrapper(
            #[adze::leaf(pattern = r"\d+")]
            String
        );
    };
    assert!(matches!(s.fields, Fields::Unnamed(_)));
    assert_eq!(s.fields.len(), 1);
}

#[test]
fn struct_unnamed_multiple_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct Pair(
            #[adze::leaf(pattern = r"\d+")]
            String,
            #[adze::leaf(text = ",")]
            (),
            #[adze::leaf(pattern = r"\d+")]
            String
        );
    };
    assert_eq!(s.fields.len(), 3);
    for f in s.fields.iter() {
        assert!(f.ident.is_none());
    }
}

#[test]
fn struct_unnamed_field_has_no_ident() {
    let s: ItemStruct = parse_quote! {
        pub struct Num(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            i32
        );
    };
    let f = s.fields.iter().next().unwrap();
    assert!(f.ident.is_none());
    assert_eq!(f.ty.to_token_stream().to_string(), "i32");
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 4: Struct expansion — unit structs
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn struct_unit_with_leaf_text() {
    let s: ItemStruct = parse_quote! {
        #[adze::leaf(text = ";")]
        pub struct Semicolon;
    };
    assert!(matches!(s.fields, Fields::Unit));
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

#[test]
fn struct_unit_extra() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Whitespace;
    };
    assert!(matches!(s.fields, Fields::Unit));
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
}

#[test]
fn struct_unit_external() {
    let s: ItemStruct = parse_quote! {
        #[adze::external]
        struct IndentToken;
    };
    assert!(matches!(s.fields, Fields::Unit));
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "external")));
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 5: Enum expansion — unnamed (tuple) variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn enum_unnamed_single_leaf_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                i32
            ),
        }
    };
    let v = e.variants.iter().next().unwrap();
    assert_eq!(v.ident, "Number");
    assert!(matches!(v.fields, Fields::Unnamed(_)));
    assert_eq!(variant_field_types(v), vec!["i32"]);
}

#[test]
fn enum_unnamed_recursive_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(
                #[adze::leaf(pattern = r"\d+")] String
            ),
            Neg(
                #[adze::leaf(text = "-")] (),
                Box<Expr>
            ),
        }
    };
    let neg = e.variants.iter().nth(1).unwrap();
    assert_eq!(neg.ident, "Neg");
    let types = variant_field_types(neg);
    assert_eq!(types.len(), 2);
    assert_eq!(types[0], "()");
    assert!(types[1].contains("Box"));
}

#[test]
fn enum_unnamed_binary_operator_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(#[adze::leaf(pattern = r"\d+")] String),
            #[adze::prec_left(1)]
            Add(
                Box<Expr>,
                #[adze::leaf(text = "+")] (),
                Box<Expr>
            ),
        }
    };
    let add = e.variants.iter().nth(1).unwrap();
    assert!(add.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
    let types = variant_field_types(add);
    assert_eq!(types.len(), 3);
}

#[test]
fn enum_unnamed_vec_field_in_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Numbers(
                #[adze::repeat(non_empty = true)]
                Vec<Number>
            ),
        }
    };
    let v = e.variants.iter().next().unwrap();
    let f = match &v.fields {
        Fields::Unnamed(u) => u.unnamed.iter().next().unwrap(),
        _ => panic!("expected unnamed"),
    };
    assert!(f.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 6: Enum expansion — named (struct) variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn enum_named_variant_single_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Neg {
                #[adze::leaf(text = "!")]
                _bang: (),
                value: Box<Expr>,
            },
        }
    };
    let v = e.variants.iter().next().unwrap();
    assert!(matches!(v.fields, Fields::Named(_)));
    let names: Vec<_> = v
        .fields
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
        .collect();
    assert_eq!(names, ["_bang", "value"]);
}

#[test]
fn enum_named_variant_with_skip() {
    let e: ItemEnum = parse_quote! {
        pub enum Node {
            Leaf {
                #[adze::leaf(pattern = r"\d+")]
                text: String,
                #[adze::skip(0usize)]
                depth: usize,
            },
        }
    };
    let v = e.variants.iter().next().unwrap();
    let skip_field = v.fields.iter().nth(1).unwrap();
    assert!(skip_field.attrs.iter().any(|a| is_adze_attr(a, "skip")));
}

#[test]
fn enum_named_variant_preserves_field_order() {
    let e: ItemEnum = parse_quote! {
        pub enum Stmt {
            Assign {
                target: Box<Expr>,
                #[adze::leaf(text = "=")]
                _eq: (),
                value: Box<Expr>,
            },
        }
    };
    let v = e.variants.iter().next().unwrap();
    let names: Vec<_> = v
        .fields
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
        .collect();
    assert_eq!(names, ["target", "_eq", "value"]);
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 7: Enum expansion — unit variants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn enum_unit_variant_with_leaf_text() {
    let e: ItemEnum = parse_quote! {
        pub enum Keyword {
            #[adze::leaf(text = "if")]
            If,
            #[adze::leaf(text = "else")]
            Else,
        }
    };
    for v in &e.variants {
        assert!(matches!(v.fields, Fields::Unit));
        assert!(v.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

#[test]
fn enum_unit_variant_leaf_text_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Op {
            #[adze::leaf(text = "+")]
            Plus,
            #[adze::leaf(text = "-")]
            Minus,
        }
    };
    let plus_attr = e
        .variants
        .iter()
        .next()
        .unwrap()
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(plus_attr);
    let text_param = params.iter().find(|p| p.path == "text").unwrap();
    let val = text_param.expr.to_token_stream().to_string();
    assert!(val.contains('+'), "Expected '+' in text param: {val}");
}

#[test]
fn enum_unit_variant_count() {
    let e: ItemEnum = parse_quote! {
        pub enum Digit {
            #[adze::leaf(text = "0")] Zero,
            #[adze::leaf(text = "1")] One,
            #[adze::leaf(text = "2")] Two,
            #[adze::leaf(text = "3")] Three,
            #[adze::leaf(text = "4")] Four,
        }
    };
    assert_eq!(e.variants.len(), 5);
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 8: Attribute-expansion relationship
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn attr_language_detected_on_enum() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Expr {
            Num(#[adze::leaf(pattern = r"\d+")] String),
        }
    };
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn attr_language_detected_on_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn attr_extra_detected_on_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
}

#[test]
fn attr_prec_left_has_correct_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(3)]
            Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
        }
    };
    let v = e.variants.iter().next().unwrap();
    let prec_attr = v
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_left"))
        .unwrap();
    assert_eq!(prec_value(prec_attr), 3);
}

#[test]
fn attr_prec_right_has_correct_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_right(5)]
            Cons(Box<Expr>, #[adze::leaf(text = "::")] (), Box<Expr>),
        }
    };
    let v = e.variants.iter().next().unwrap();
    let prec_attr = v
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_right"))
        .unwrap();
    assert_eq!(prec_value(prec_attr), 5);
}

#[test]
fn attr_prec_no_assoc_has_correct_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec(2)]
            Cmp(Box<Expr>, #[adze::leaf(text = "==")] (), Box<Expr>),
        }
    };
    let v = e.variants.iter().next().unwrap();
    let prec_attr = v.attrs.iter().find(|a| is_adze_attr(a, "prec")).unwrap();
    assert_eq!(prec_value(prec_attr), 2);
}

#[test]
fn attr_word_detected() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Identifier {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
}

#[test]
fn attr_external_detected() {
    let s: ItemStruct = parse_quote! {
        #[adze::external]
        struct IndentToken;
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "external")));
}

#[test]
fn multiple_attrs_on_single_item() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        #[adze::language]
        pub struct Ident {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    let names = adze_attr_names(&s.attrs);
    assert!(names.contains(&"word".to_string()));
    assert!(names.contains(&"language".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 9: Grammar module expansion patterns
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn grammar_module_name_extraction() {
    let m = parse_mod(quote! {
        #[adze::grammar("arithmetic")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    });
    assert_eq!(extract_grammar_name(&m), Some("arithmetic".to_string()));
}

#[test]
fn grammar_module_finds_language_enum() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    });
    assert_eq!(find_language_type(&m), Some("Expr".to_string()));
}

#[test]
fn grammar_module_finds_language_struct() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
        }
    });
    assert_eq!(find_language_type(&m), Some("Root".to_string()));
}

#[test]
fn grammar_module_contains_multiple_types() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                child: Child,
            }
            pub struct Child {
                #[adze::leaf(pattern = r"\w+")]
                val: String,
            }
            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    let structs: Vec<_> = module_items(&m)
        .iter()
        .filter_map(|i| {
            if let Item::Struct(s) = i {
                Some(s.ident.to_string())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(structs.len(), 3);
    assert!(structs.contains(&"Root".to_string()));
    assert!(structs.contains(&"Child".to_string()));
    assert!(structs.contains(&"Whitespace".to_string()));
}

#[test]
fn grammar_module_with_use_statement() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            use adze::Spanned;
            #[adze::language]
            pub struct Root {
                vals: Vec<Spanned<Number>>,
            }
            pub struct Number {
                #[adze::leaf(pattern = r"\d+")]
                v: String,
            }
        }
    });
    let uses = module_items(&m)
        .iter()
        .filter(|i| matches!(i, Item::Use(_)))
        .count();
    assert_eq!(uses, 1);
}

#[test]
fn grammar_module_enum_and_struct_mix() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
            }
            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    assert!(find_enum_in_mod(&m, "Expr").is_some());
    assert!(find_struct_in_mod(&m, "Whitespace").is_some());
}

#[test]
fn grammar_module_extra_types_identified() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _w: (),
            }
            #[adze::extra]
            struct Comment {
                #[adze::leaf(pattern = r"//[^\n]*")]
                _c: (),
            }
        }
    });
    let extras: Vec<_> = module_items(&m)
        .iter()
        .filter_map(|i| {
            if let Item::Struct(s) = i
                && s.attrs.iter().any(|a| is_adze_attr(a, "extra"))
            {
                return Some(s.ident.to_string());
            }
            None
        })
        .collect();
    assert_eq!(extras.len(), 2);
    assert!(extras.contains(&"Ws".to_string()));
    assert!(extras.contains(&"Comment".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 10: Error handling during expansion
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn error_missing_grammar_name_detected() {
    let m = parse_mod(quote! {
        #[adze::grammar]
        mod grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    // Without "(...)" args, parse_args returns Err
    let attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
    let result = attr.parse_args::<Expr>();
    assert!(result.is_err());
}

#[test]
fn error_non_string_grammar_name_detected() {
    let m = parse_mod(quote! {
        #[adze::grammar(42)]
        mod grammar {
            #[adze::language]
            pub struct Root {}
        }
    });
    let name = extract_grammar_name(&m);
    assert!(name.is_none(), "Integer should not parse as grammar name");
}

#[test]
fn error_no_language_type_detected() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
        }
    });
    assert!(find_language_type(&m).is_none());
}

#[test]
fn error_semicolon_module_has_no_content() {
    let m: ItemMod = parse_quote! {
        #[adze::grammar("test")]
        mod grammar;
    };
    assert!(m.content.is_none());
}

#[test]
fn error_leaf_missing_text_or_pattern() {
    let s: ItemStruct = parse_quote! {
        pub struct Bad {
            #[adze::leaf()]
            value: String,
        }
    };
    let f = s.fields.iter().next().unwrap();
    let leaf_attr = f.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
    let params = leaf_params(leaf_attr);
    // No text or pattern params
    let has_text = params.iter().any(|p| p.path == "text");
    let has_pattern = params.iter().any(|p| p.path == "pattern");
    assert!(!has_text);
    assert!(!has_pattern);
}

#[test]
fn error_invalid_attr_parse_returns_err() {
    // NameValueExpr requires `ident = expr`; a bare identifier is not valid
    let result = parse_str::<NameValueExpr>("no_equals_here");
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 11: Type generation patterns
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn wrap_leaf_type_simple_string() {
    let input = ty("String");
    let wrapped = wrap_leaf_type(&input, &skip_set(&["Box", "Option", "Vec", "Spanned"]));
    assert!(
        ts(&wrapped).contains("WithLeaf"),
        "Expected WithLeaf wrapper: {}",
        ts(&wrapped)
    );
}

#[test]
fn wrap_leaf_type_i32() {
    let input = ty("i32");
    let wrapped = wrap_leaf_type(&input, &skip_set(&["Box", "Option", "Vec", "Spanned"]));
    assert!(ts(&wrapped).contains("WithLeaf"));
    assert!(ts(&wrapped).contains("i32"));
}

#[test]
fn wrap_leaf_type_preserves_vec_wrapper() {
    let input = ty("Vec<String>");
    let wrapped = wrap_leaf_type(&input, &skip_set(&["Box", "Option", "Vec", "Spanned"]));
    let result = ts(&wrapped);
    assert!(result.contains("Vec"), "Vec should be preserved: {result}");
}

#[test]
fn wrap_leaf_type_preserves_option_wrapper() {
    let input = ty("Option<i32>");
    let wrapped = wrap_leaf_type(&input, &skip_set(&["Box", "Option", "Vec", "Spanned"]));
    let result = ts(&wrapped);
    assert!(
        result.contains("Option"),
        "Option should be preserved: {result}"
    );
}

#[test]
fn wrap_leaf_type_preserves_box_wrapper() {
    let input = ty("Box<String>");
    let wrapped = wrap_leaf_type(&input, &skip_set(&["Box", "Option", "Vec", "Spanned"]));
    let result = ts(&wrapped);
    assert!(result.contains("Box"), "Box should be preserved: {result}");
}

#[test]
fn try_extract_inner_type_vec() {
    let (inner, found) = try_extract_inner_type(&ty("Vec<Number>"), "Vec", &skip_set(&[]));
    assert!(found);
    assert_eq!(ts(&inner), "Number");
}

#[test]
fn try_extract_inner_type_option() {
    let (inner, found) = try_extract_inner_type(&ty("Option<bool>"), "Option", &skip_set(&[]));
    assert!(found);
    assert_eq!(ts(&inner), "bool");
}

#[test]
fn try_extract_inner_type_box() {
    let (inner, found) = try_extract_inner_type(&ty("Box<Expr>"), "Box", &skip_set(&[]));
    assert!(found);
    assert_eq!(ts(&inner), "Expr");
}

#[test]
fn try_extract_inner_type_no_match() {
    let (inner, found) = try_extract_inner_type(&ty("String"), "Vec", &skip_set(&[]));
    assert!(!found);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn try_extract_through_skip_over() {
    let (inner, found) = try_extract_inner_type(&ty("Box<Vec<i32>>"), "Vec", &skip_set(&["Box"]));
    assert!(found);
    assert_eq!(ts(&inner), "i32");
}

#[test]
fn filter_inner_type_removes_box() {
    let result = filter_inner_type(&ty("Box<String>"), &skip_set(&["Box"]));
    assert_eq!(ts(&result), "String");
}

#[test]
fn filter_inner_type_removes_nested_wrappers() {
    let result = filter_inner_type(&ty("Box<Option<i32>>"), &skip_set(&["Box", "Option"]));
    assert_eq!(ts(&result), "i32");
}

#[test]
fn filter_inner_type_preserves_non_skip() {
    let result = filter_inner_type(&ty("Vec<String>"), &skip_set(&["Box"]));
    // Vec is not in skip set, so it stays
    assert!(ts(&result).contains("Vec"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 12: Combined patterns — complex grammars
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn complex_grammar_module_full_structure() {
    let m = parse_mod(quote! {
        #[adze::grammar("calc")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
                #[adze::prec_left(1)]
                Add(
                    Box<Expr>,
                    #[adze::leaf(text = "+")] (),
                    Box<Expr>
                ),
                #[adze::prec_left(1)]
                Sub(
                    Box<Expr>,
                    #[adze::leaf(text = "-")] (),
                    Box<Expr>
                ),
                #[adze::prec_left(2)]
                Mul(
                    Box<Expr>,
                    #[adze::leaf(text = "*")] (),
                    Box<Expr>
                ),
            }
            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    assert_eq!(extract_grammar_name(&m), Some("calc".to_string()));
    assert_eq!(find_language_type(&m), Some("Expr".to_string()));
    let expr_enum = find_enum_in_mod(&m, "Expr").unwrap();
    assert_eq!(expr_enum.variants.len(), 4);
    assert!(find_struct_in_mod(&m, "Whitespace").is_some());
}

#[test]
fn variant_symbol_name_generation_pattern() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(#[adze::leaf(pattern = r"\d+")] String),
            Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
        }
    };
    let enum_name = e.ident.to_string();
    for v in &e.variants {
        let symbol = format!("{}_{}", enum_name, v.ident);
        assert!(
            symbol.starts_with("Expr_"),
            "Symbol should start with Expr_: {symbol}"
        );
    }
}

#[test]
fn mixed_variant_kinds_in_single_enum() {
    let e: ItemEnum = parse_quote! {
        pub enum Token {
            #[adze::leaf(text = "+")]
            Plus,
            Number(
                #[adze::leaf(pattern = r"\d+")] String
            ),
            Assign {
                target: Box<Expr>,
                #[adze::leaf(text = "=")]
                _eq: (),
                value: Box<Expr>,
            },
        }
    };
    let kinds: Vec<_> = e
        .variants
        .iter()
        .map(|v| match &v.fields {
            Fields::Unit => "unit",
            Fields::Unnamed(_) => "unnamed",
            Fields::Named(_) => "named",
        })
        .collect();
    assert_eq!(kinds, ["unit", "unnamed", "named"]);
}

#[test]
fn field_then_params_parsing() {
    let input: FieldThenParams = syn::parse_str(r#"#[adze::leaf(text = ",")] ()"#).unwrap();
    let field_ty = input.field.ty.to_token_stream().to_string();
    assert_eq!(field_ty, "()");
}

#[test]
fn name_value_expr_closure() {
    let nve: NameValueExpr = parse_str("transform = |v| v.parse().unwrap()").unwrap();
    assert_eq!(nve.path.to_string(), "transform");
}

#[test]
fn name_value_expr_string() {
    let nve: NameValueExpr = parse_str(r#"text = "+""#).unwrap();
    assert_eq!(nve.path.to_string(), "text");
}

#[test]
fn name_value_expr_raw_string_pattern() {
    let nve: NameValueExpr = parse_str(r#"pattern = r"\d+""#).unwrap();
    assert_eq!(nve.path.to_string(), "pattern");
}

#[test]
fn name_value_expr_boolean() {
    let nve: NameValueExpr = parse_str("non_empty = true").unwrap();
    assert_eq!(nve.path.to_string(), "non_empty");
}
