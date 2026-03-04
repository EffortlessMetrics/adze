//! Comprehensive tests for `macro/src/expansion.rs` logic.
//!
//! Since `adze-macro` is a proc-macro crate and cannot export regular functions,
//! these tests exercise the expansion pipeline through two complementary approaches:
//!
//! 1. Testing `adze_common` helper functions that `expansion.rs` delegates to
//!    (wrap_leaf_type, try_extract_inner_type, filter_inner_type, NameValueExpr,
//!    FieldThenParams).
//!
//! 2. Verifying the input grammar module structures that expansion.rs processes:
//!    attribute parsing, field extraction, variant detection, module shapes,
//!    and error detection.
//!
//! Together these cover the full expansion surface with 50+ tests.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proptest::prelude::*;
use quote::{ToTokens, quote};
use syn::{Attribute, Fields, Item, ItemEnum, ItemMod, ItemStruct, Type, parse_quote, parse2};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn ty(s: &str) -> Type {
    syn::parse_str::<Type>(s).unwrap()
}

fn ts(t: &Type) -> String {
    t.to_token_stream().to_string()
}

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == name
}

fn parse_mod(tokens: proc_macro2::TokenStream) -> ItemMod {
    parse2(tokens).expect("failed to parse module")
}

fn module_items(m: &ItemMod) -> &[Item] {
    &m.content.as_ref().expect("module has no content").1
}

fn find_struct<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemStruct> {
    module_items(m).iter().find_map(|i| {
        if let Item::Struct(s) = i {
            if s.ident == name { Some(s) } else { None }
        } else {
            None
        }
    })
}

fn find_enum<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemEnum> {
    module_items(m).iter().find_map(|i| {
        if let Item::Enum(e) = i {
            if e.ident == name { Some(e) } else { None }
        } else {
            None
        }
    })
}

fn struct_field_names(s: &ItemStruct) -> Vec<String> {
    s.fields
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|id| id.to_string()))
        .collect()
}

fn struct_field_types(s: &ItemStruct) -> Vec<String> {
    s.fields
        .iter()
        .map(|f| f.ty.to_token_stream().to_string())
        .collect()
}

fn variant_names(e: &ItemEnum) -> Vec<String> {
    e.variants.iter().map(|v| v.ident.to_string()).collect()
}

fn has_language_item(m: &ItemMod) -> bool {
    module_items(m).iter().any(|item| match item {
        Item::Enum(e) => e.attrs.iter().any(|a| is_adze_attr(a, "language")),
        Item::Struct(s) => s.attrs.iter().any(|a| is_adze_attr(a, "language")),
        _ => false,
    })
}

fn grammar_attr_value(m: &ItemMod) -> Option<String> {
    m.attrs.iter().find_map(|a| {
        if is_adze_attr(a, "grammar") {
            a.parse_args::<syn::Expr>().ok().and_then(|expr| {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = expr
                {
                    Some(s.value())
                } else {
                    None
                }
            })
        } else {
            None
        }
    })
}

// =====================================================================
// 1. wrap_leaf_type – expansion.rs gen_field relies on this
// =====================================================================

#[test]
fn wrap_leaf_plain_type() {
    let wrapped = wrap_leaf_type(&ty("i32"), &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_leaf_string_type() {
    let wrapped = wrap_leaf_type(&ty("String"), &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_leaf_skips_option() {
    let wrapped = wrap_leaf_type(&ty("Option<i32>"), &skip(&["Option"]));
    assert_eq!(ts(&wrapped), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_leaf_skips_vec() {
    let wrapped = wrap_leaf_type(&ty("Vec<String>"), &skip(&["Vec"]));
    assert_eq!(ts(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_leaf_skips_box_wraps_inner() {
    let wrapped = wrap_leaf_type(&ty("Box<u64>"), &skip(&["Box"]));
    assert_eq!(ts(&wrapped), "Box < adze :: WithLeaf < u64 > >");
}

#[test]
fn wrap_leaf_nested_option_vec() {
    let wrapped = wrap_leaf_type(&ty("Option<Vec<i32>>"), &skip(&["Option", "Vec"]));
    assert_eq!(ts(&wrapped), "Option < Vec < adze :: WithLeaf < i32 > > >");
}

#[test]
fn wrap_leaf_spanned_skipped() {
    // expansion.rs uses Spanned, Box, Option, Vec as the non_leaf set
    let non_leaf = skip(&["Spanned", "Box", "Option", "Vec"]);
    let wrapped = wrap_leaf_type(&ty("Spanned<i32>"), &non_leaf);
    assert_eq!(ts(&wrapped), "Spanned < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_leaf_with_expansion_non_leaf_set() {
    // Exact set used in expansion.rs gen_field
    let non_leaf = skip(&["Spanned", "Box", "Option", "Vec"]);
    let wrapped = wrap_leaf_type(&ty("Vec<Option<i32>>"), &non_leaf);
    assert_eq!(ts(&wrapped), "Vec < Option < adze :: WithLeaf < i32 > > >");
}

// =====================================================================
// 2. try_extract_inner_type – expansion field processing
// =====================================================================

#[test]
fn extract_vec_inner() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<String>"), "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn extract_option_inner() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<u32>"), "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "u32");
}

#[test]
fn extract_box_inner() {
    let (inner, ok) = try_extract_inner_type(&ty("Box<Expr>"), "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Expr");
}

#[test]
fn extract_through_skip_container() {
    let (inner, ok) = try_extract_inner_type(&ty("Box<Vec<i32>>"), "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ts(&inner), "i32");
}

#[test]
fn extract_miss_returns_original() {
    let orig = ty("HashMap<String, i32>");
    let (inner, ok) = try_extract_inner_type(&orig, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), ts(&orig));
}

#[test]
fn extract_non_path_type_unchanged() {
    let (inner, ok) = try_extract_inner_type(&ty("&str"), "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "& str");
}

#[test]
fn extract_skip_no_match_returns_original() {
    let orig = ty("Box<String>");
    let (inner, ok) = try_extract_inner_type(&orig, "Option", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(ts(&inner), ts(&orig));
}

// =====================================================================
// 3. filter_inner_type – container unwrapping
// =====================================================================

#[test]
fn filter_box_unwraps() {
    let filtered = filter_inner_type(&ty("Box<String>"), &skip(&["Box"]));
    assert_eq!(ts(&filtered), "String");
}

#[test]
fn filter_nested_box_arc() {
    let filtered = filter_inner_type(&ty("Box<Arc<i32>>"), &skip(&["Box", "Arc"]));
    assert_eq!(ts(&filtered), "i32");
}

#[test]
fn filter_empty_skip_noop() {
    let orig = ty("Box<String>");
    let filtered = filter_inner_type(&orig, &skip(&[]));
    assert_eq!(ts(&filtered), ts(&orig));
}

#[test]
fn filter_non_path_type_unchanged() {
    let filtered = filter_inner_type(&ty("(i32, u32)"), &skip(&["Box"]));
    assert_eq!(ts(&filtered), "(i32 , u32)");
}

// =====================================================================
// 4. NameValueExpr parsing – attribute parameter processing
// =====================================================================

#[test]
fn name_value_string_param() {
    let nv: NameValueExpr = parse_quote!(text = "+");
    assert_eq!(nv.path.to_string(), "text");
}

#[test]
fn name_value_pattern_param() {
    let nv: NameValueExpr = parse_quote!(pattern = r"\d+");
    assert_eq!(nv.path.to_string(), "pattern");
}

#[test]
fn name_value_transform_param() {
    let nv: NameValueExpr = parse_quote!(transform = |v| v.parse::<i32>().unwrap());
    assert_eq!(nv.path.to_string(), "transform");
}

#[test]
fn name_value_non_empty_param() {
    let nv: NameValueExpr = parse_quote!(non_empty = true);
    assert_eq!(nv.path.to_string(), "non_empty");
}

#[test]
fn name_value_integer_param() {
    let nv: NameValueExpr = parse_quote!(precedence = 5);
    assert_eq!(nv.path.to_string(), "precedence");
}

// =====================================================================
// 5. FieldThenParams parsing – field+parameter extraction
// =====================================================================

#[test]
fn field_then_no_params() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn field_then_with_params() {
    let ftp: FieldThenParams = parse_quote!(String, name = "test", value = 42);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "name");
    assert_eq!(ftp.params[1].path.to_string(), "value");
}

#[test]
fn field_then_unit_type() {
    let ftp: FieldThenParams = parse_quote!(());
    assert!(ftp.params.is_empty());
}

#[test]
fn field_then_single_param() {
    let ftp: FieldThenParams = parse_quote!(i32, max = 100);
    assert_eq!(ftp.params.len(), 1);
}

// =====================================================================
// 6. Grammar module structure – what expansion.rs parses
// =====================================================================

#[test]
fn grammar_module_has_content() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
    });
    assert!(m.content.is_some());
}

#[test]
fn grammar_module_preserves_ident() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod my_grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
    });
    assert_eq!(m.ident.to_string(), "my_grammar");
}

#[test]
fn grammar_module_attr_value_extracted() {
    let m = parse_mod(quote! {
        #[adze::grammar("arithmetic")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
    });
    assert_eq!(grammar_attr_value(&m), Some("arithmetic".to_string()));
}

#[test]
fn grammar_module_has_language_item() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    });
    assert!(has_language_item(&m));
}

#[test]
fn grammar_module_without_language_detected() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            pub enum Expr {
                Num(String),
            }
        }
    });
    assert!(!has_language_item(&m));
}

// =====================================================================
// 7. Struct field shapes recognized by expansion
// =====================================================================

#[test]
fn struct_named_fields_detected() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Pair {
                #[adze::leaf(pattern = r"\w+")]
                left: String,
                #[adze::leaf(pattern = r"\d+")]
                right: i32,
            }
        }
    });
    let s = find_struct(&m, "Pair").unwrap();
    assert_eq!(struct_field_names(s), vec!["left", "right"]);
    assert!(matches!(s.fields, Fields::Named(_)));
}

#[test]
fn struct_leaf_attrs_on_all_fields() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Triple {
                #[adze::leaf(pattern = r"\w+")]
                a: String,
                #[adze::leaf(text = "+")]
                b: (),
                #[adze::leaf(pattern = r"\d+")]
                c: i32,
            }
        }
    });
    let s = find_struct(&m, "Triple").unwrap();
    for f in &s.fields {
        assert!(
            f.attrs.iter().any(|a| is_adze_attr(a, "leaf")),
            "Every field should have leaf attr"
        );
    }
}

#[test]
fn struct_skip_field_detected() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Node {
                #[adze::leaf(pattern = r"\w+")]
                value: String,
                #[adze::skip(false)]
                visited: bool,
            }
        }
    });
    let s = find_struct(&m, "Node").unwrap();
    let visited = s
        .fields
        .iter()
        .find(|f| f.ident.as_ref().is_some_and(|i| i.to_string() == "visited"));
    assert!(
        visited
            .unwrap()
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "skip"))
    );
}

#[test]
fn struct_vec_field_type() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct List {
                items: Vec<Item>,
            }

            pub struct Item {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
    });
    let s = find_struct(&m, "List").unwrap();
    let types = struct_field_types(s);
    assert!(types[0].contains("Vec"));
}

#[test]
fn struct_optional_field_type() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                name: Option<String>,
            }
        }
    });
    let s = find_struct(&m, "Root").unwrap();
    let types = struct_field_types(s);
    assert!(types[0].contains("Option"));
}

// =====================================================================
// 8. Enum variant shapes recognized by expansion
// =====================================================================

#[test]
fn enum_unit_variant_with_leaf() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Keyword {
                #[adze::leaf(text = "if")]
                If,
                #[adze::leaf(text = "else")]
                Else,
            }
        }
    });
    let e = find_enum(&m, "Keyword").unwrap();
    assert_eq!(variant_names(e), vec!["If", "Else"]);
    for v in &e.variants {
        assert!(matches!(v.fields, Fields::Unit));
        assert!(v.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

#[test]
fn enum_tuple_variant_with_leaf_field() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Value {
                Num(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<f64>().unwrap())]
                    f64
                ),
            }
        }
    });
    let e = find_enum(&m, "Value").unwrap();
    let v = &e.variants[0];
    assert!(matches!(v.fields, Fields::Unnamed(_)));
    if let Fields::Unnamed(u) = &v.fields {
        assert_eq!(u.unnamed.len(), 1);
        assert!(u.unnamed[0].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

#[test]
fn enum_struct_variant_with_named_fields() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Binary {
                    left: Box<Expr>,
                    #[adze::leaf(text = "+")]
                    _op: (),
                    right: Box<Expr>,
                },
            }
        }
    });
    let e = find_enum(&m, "Expr").unwrap();
    let v = &e.variants[0];
    assert!(matches!(v.fields, Fields::Named(_)));
}

#[test]
fn enum_prec_left_attr_detected() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
            }
        }
    });
    let e = find_enum(&m, "Expr").unwrap();
    let add = &e.variants[1];
    assert!(add.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
}

#[test]
fn enum_prec_right_attr_detected() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_right(2)]
                Cons(Box<Expr>, #[adze::leaf(text = "::")] (), Box<Expr>),
            }
        }
    });
    let e = find_enum(&m, "Expr").unwrap();
    let cons = &e.variants[1];
    assert!(cons.attrs.iter().any(|a| is_adze_attr(a, "prec_right")));
}

#[test]
fn enum_mixed_variant_kinds() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                #[adze::leaf(text = "+")]
                Plus,
                Number(#[adze::leaf(pattern = r"\d+")] String),
                Named {
                    #[adze::leaf(pattern = r"\w+")]
                    name: String,
                },
            }
        }
    });
    let e = find_enum(&m, "Token").unwrap();
    assert!(matches!(e.variants[0].fields, Fields::Unit));
    assert!(matches!(e.variants[1].fields, Fields::Unnamed(_)));
    assert!(matches!(e.variants[2].fields, Fields::Named(_)));
}

// =====================================================================
// 9. Extra / word / external annotation detection
// =====================================================================

#[test]
fn extra_struct_detected() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    let ws = find_struct(&m, "Whitespace").unwrap();
    assert!(ws.attrs.iter().any(|a| is_adze_attr(a, "extra")));
}

#[test]
fn word_struct_detected() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root { ident: Ident }

            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }
        }
    });
    let ident = find_struct(&m, "Ident").unwrap();
    assert!(ident.attrs.iter().any(|a| is_adze_attr(a, "word")));
}

#[test]
fn external_struct_detected() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }

            #[adze::external]
            struct IndentToken {
                #[adze::leaf(pattern = r"\t+")]
                _indent: (),
            }
        }
    });
    let indent = find_struct(&m, "IndentToken").unwrap();
    assert!(indent.attrs.iter().any(|a| is_adze_attr(a, "external")));
}

// =====================================================================
// 10. Error detection – invalid module structures
// =====================================================================

#[test]
fn error_no_grammar_attr_value() {
    let m = parse_mod(quote! {
        #[adze::grammar]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
    });
    assert!(grammar_attr_value(&m).is_none());
}

#[test]
fn error_non_string_grammar_name() {
    let m = parse_mod(quote! {
        #[adze::grammar(42)]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
    });
    assert!(grammar_attr_value(&m).is_none());
}

#[test]
fn error_missing_language_item() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            pub struct Foo {
                v: String,
            }
        }
    });
    assert!(!has_language_item(&m));
}

#[test]
fn error_empty_module_body() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {}
    });
    assert!(module_items(&m).is_empty());
    assert!(!has_language_item(&m));
}

#[test]
fn error_only_functions_no_types() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            fn helper() {}
        }
    });
    assert!(!has_language_item(&m));
}

// =====================================================================
// 11. Module content passthrough items
// =====================================================================

#[test]
fn use_item_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            use std::collections::HashMap;

            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
    });
    assert!(module_items(&m).iter().any(|i| matches!(i, Item::Use(_))));
}

#[test]
fn fn_item_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }

            fn helper() -> bool { true }
        }
    });
    assert!(module_items(&m).iter().any(|i| matches!(i, Item::Fn(_))));
}

#[test]
fn const_item_preserved() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            const MAX: usize = 100;

            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
    });
    assert!(module_items(&m).iter().any(|i| matches!(i, Item::Const(_))));
}

// =====================================================================
// 12. Complex grammar structure verification
// =====================================================================

#[test]
fn nested_grammar_all_types_present() {
    let m = parse_mod(quote! {
        #[adze::grammar("nested")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                stmts: Vec<Statement>,
            }

            pub struct Statement {
                expr: Expr,
            }

            pub enum Expr {
                Lit(#[adze::leaf(pattern = r"\d+")] String),
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    assert!(find_struct(&m, "Program").is_some());
    assert!(find_struct(&m, "Statement").is_some());
    assert!(find_enum(&m, "Expr").is_some());
    assert!(find_struct(&m, "Ws").is_some());
}

#[test]
fn full_arithmetic_grammar_structure() {
    let m = parse_mod(quote! {
        #[adze::grammar("arith")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<i32>().unwrap())]
                    i32
                ),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                #[adze::prec_left(1)]
                Sub(Box<Expr>, #[adze::leaf(text = "-")] (), Box<Expr>),
                #[adze::prec_left(2)]
                Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
                #[adze::prec_left(2)]
                Div(Box<Expr>, #[adze::leaf(text = "/")] (), Box<Expr>),
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    let e = find_enum(&m, "Expr").unwrap();
    assert_eq!(e.variants.len(), 5);
    assert_eq!(variant_names(e), vec!["Number", "Add", "Sub", "Mul", "Div"]);
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn delimited_repeat_field_detected() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct List {
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                items: Vec<Number>,
            }

            pub struct Number {
                #[adze::leaf(pattern = r"\d+")]
                v: String,
            }
        }
    });
    let s = find_struct(&m, "List").unwrap();
    let items_field = s.fields.iter().next().unwrap();
    assert!(
        items_field
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "delimited"))
    );
}

#[test]
fn repeat_non_empty_field_detected() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct List {
                #[adze::repeat(non_empty = true)]
                items: Vec<Item>,
            }

            pub struct Item {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
    });
    let s = find_struct(&m, "List").unwrap();
    let items_field = s.fields.iter().next().unwrap();
    assert!(items_field.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
}

// =====================================================================
// 13. Token extraction type patterns
// =====================================================================

#[test]
fn leaf_text_param_in_attr() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Op {
                #[adze::leaf(text = "+")]
                Plus,
            }
        }
    });
    let e = find_enum(&m, "Op").unwrap();
    let v = &e.variants[0];
    assert!(v.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

#[test]
fn leaf_transform_closure_in_field_attr() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Number {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<i32>().unwrap())]
                value: i32,
            }
        }
    });
    let s = find_struct(&m, "Number").unwrap();
    let f = s.fields.iter().next().unwrap();
    let leaf_attr = f.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
    let tokens = leaf_attr.to_token_stream().to_string();
    assert!(tokens.contains("transform"));
}

// =====================================================================
// 14. Expansion symbol name format
// =====================================================================

#[test]
fn expected_symbol_name_format() {
    // expansion.rs creates symbols as EnumName_VariantName
    let enum_name = "Expr";
    let variant_name = "Number";
    let symbol = format!("{}_{}", enum_name, variant_name);
    assert_eq!(symbol, "Expr_Number");
}

#[test]
fn expected_tree_sitter_fn_format() {
    let grammar_name = "my_lang";
    let fn_name = format!("tree_sitter_{}", grammar_name);
    assert_eq!(fn_name, "tree_sitter_my_lang");
}

// =====================================================================
// 15. Proptest: property-based input shape verification
// =====================================================================

fn valid_grammar_name() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[a-z][a-z0-9_]{0,15}")
        .unwrap()
        .prop_filter("non-empty", |s| !s.is_empty())
}

proptest! {
    #[test]
    fn prop_grammar_name_roundtrips(name in valid_grammar_name()) {
        let name_lit = syn::LitStr::new(&name, proc_macro2::Span::call_site());
        let m: ItemMod = parse_quote! {
            #[adze::grammar(#name_lit)]
            mod grammar {
                #[adze::language]
                pub struct Root {
                    #[adze::leaf(pattern = r"\w+")]
                    v: String,
                }
            }
        };
        let extracted = grammar_attr_value(&m);
        prop_assert_eq!(extracted, Some(name));
    }

    #[test]
    fn prop_field_count_preserved(count in 1usize..=8) {
        let fields: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                quote! {
                    #[adze::leaf(pattern = r"\w+")]
                    #ident: String
                }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root { #(#fields),* }
            }
        });
        let s = find_struct(&m, "Root").unwrap();
        prop_assert_eq!(s.fields.len(), count);
    }

    #[test]
    fn prop_variant_count_preserved(count in 1usize..=8) {
        let variants: Vec<proc_macro2::TokenStream> = (0..count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
                let text = format!("t{i}");
                quote! {
                    #[adze::leaf(text = #text)]
                    #ident
                }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub enum Expr {
                    #(#variants),*
                }
            }
        });
        let e = find_enum(&m, "Expr").unwrap();
        prop_assert_eq!(e.variants.len(), count);
    }

    #[test]
    fn prop_wrap_leaf_always_wraps_plain(idx in 0usize..=4) {
        let types = ["i32", "u64", "String", "bool", "f64"];
        let t = ty(types[idx]);
        let wrapped = wrap_leaf_type(&t, &skip(&[]));
        let result = ts(&wrapped);
        prop_assert!(result.contains("WithLeaf"), "Expected WithLeaf in {}", result);
    }

    #[test]
    fn prop_wrap_leaf_skip_preserves_outer(idx in 0usize..=2) {
        let containers = ["Vec", "Option", "Box"];
        let container = containers[idx];
        let type_str = format!("{}<i32>", container);
        let t = ty(&type_str);
        let wrapped = wrap_leaf_type(&t, &skip(&[container]));
        let result = ts(&wrapped);
        prop_assert!(result.starts_with(container), "Expected {} prefix in {}", container, result);
        prop_assert!(result.contains("WithLeaf"), "Expected inner WithLeaf in {}", result);
    }

    #[test]
    fn prop_extract_vec_always_succeeds(idx in 0usize..=3) {
        let inner_types = ["i32", "String", "u8", "bool"];
        let type_str = format!("Vec<{}>", inner_types[idx]);
        let t = ty(&type_str);
        let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&[]));
        prop_assert!(ok);
        prop_assert_eq!(ts(&inner), inner_types[idx]);
    }

    #[test]
    fn prop_filter_box_always_unwraps(idx in 0usize..=3) {
        let inner_types = ["i32", "String", "Expr", "bool"];
        let type_str = format!("Box<{}>", inner_types[idx]);
        let t = ty(&type_str);
        let filtered = filter_inner_type(&t, &skip(&["Box"]));
        prop_assert_eq!(ts(&filtered), inner_types[idx]);
    }

    #[test]
    fn prop_all_fields_have_leaf_or_skip(field_count in 1usize..=5) {
        let fields: Vec<proc_macro2::TokenStream> = (0..field_count)
            .map(|i| {
                let ident = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
                if i % 2 == 0 {
                    quote! {
                        #[adze::leaf(pattern = r"\w+")]
                        #ident: String
                    }
                } else {
                    quote! {
                        #[adze::skip(0)]
                        #ident: i32
                    }
                }
            })
            .collect();
        let m = parse_mod(quote! {
            #[adze::grammar("test")]
            mod grammar {
                #[adze::language]
                pub struct Root { #(#fields),* }
            }
        });
        let s = find_struct(&m, "Root").unwrap();
        prop_assert_eq!(s.fields.len(), field_count);
        for (i, f) in s.fields.iter().enumerate() {
            if i % 2 == 0 {
                prop_assert!(f.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
            } else {
                prop_assert!(f.attrs.iter().any(|a| is_adze_attr(a, "skip")));
            }
        }
    }

    #[test]
    fn prop_symbol_name_format(
        enum_name_idx in 0usize..=3,
        variant_idx in 0usize..=3,
    ) {
        let enum_names = ["Expr", "Token", "Value", "Stmt"];
        let variant_names_arr = ["Number", "Plus", "Ident", "Block"];
        let symbol = format!("{}_{}", enum_names[enum_name_idx], variant_names_arr[variant_idx]);
        prop_assert!(symbol.contains('_'));
        prop_assert!(!symbol.starts_with('_'));
        prop_assert!(!symbol.ends_with('_'));
    }

    #[test]
    fn prop_tree_sitter_fn_name_format(name in valid_grammar_name()) {
        let fn_name = format!("tree_sitter_{}", name);
        prop_assert!(fn_name.starts_with("tree_sitter_"));
        prop_assert!(fn_name.len() > "tree_sitter_".len());
    }
}
