//! V4 expansion-pattern tests for adze-macro.
//!
//! Covers: macro expansion helpers, type manipulation (`try_extract_inner_type`,
//! `filter_inner_type`, `wrap_leaf_type`), attribute processing, token stream
//! operations, `NameValueExpr` / `FieldThenParams` parsing, and grammar module
//! structure validation.

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

fn find_struct_in_mod<'a>(m: &'a ItemMod, name: &str) -> Option<&'a ItemStruct> {
    module_items(m).iter().find_map(|i| {
        if let Item::Struct(s) = i {
            if s.ident == name { Some(s) } else { None }
        } else {
            None
        }
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 1 – try_extract_inner_type
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn extract_vec_string() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<String>"), "Vec", &skip_set(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn extract_option_i32() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<i32>"), "Option", &skip_set(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "i32");
}

#[test]
fn extract_vec_not_found_in_option() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<i32>"), "Vec", &skip_set(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "Option < i32 >");
}

#[test]
fn extract_through_box_skip() {
    let skip = skip_set(&["Box"]);
    let (inner, ok) = try_extract_inner_type(&ty("Box<Vec<u8>>"), "Vec", &skip);
    assert!(ok);
    assert_eq!(ts(&inner), "u8");
}

#[test]
fn extract_through_arc_skip() {
    let skip = skip_set(&["Arc"]);
    let (inner, ok) = try_extract_inner_type(&ty("Arc<Option<bool>>"), "Option", &skip);
    assert!(ok);
    assert_eq!(ts(&inner), "bool");
}

#[test]
fn extract_through_nested_box_arc() {
    let skip = skip_set(&["Box", "Arc"]);
    let (inner, ok) = try_extract_inner_type(&ty("Box<Arc<Vec<f64>>>"), "Vec", &skip);
    assert!(ok);
    assert_eq!(ts(&inner), "f64");
}

#[test]
fn extract_skip_type_without_target_returns_original() {
    let skip = skip_set(&["Box"]);
    let (inner, ok) = try_extract_inner_type(&ty("Box<String>"), "Vec", &skip);
    assert!(!ok);
    assert_eq!(ts(&inner), "Box < String >");
}

#[test]
fn extract_plain_type_returns_unchanged() {
    let (inner, ok) = try_extract_inner_type(&ty("String"), "Vec", &skip_set(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn extract_reference_type_returns_unchanged() {
    let (inner, ok) = try_extract_inner_type(&ty("&str"), "Option", &skip_set(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "& str");
}

#[test]
fn extract_tuple_type_returns_unchanged() {
    let (inner, ok) = try_extract_inner_type(&ty("(i32, u64)"), "Vec", &skip_set(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "(i32 , u64)");
}

#[test]
fn extract_vec_of_unit() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<()>"), "Vec", &skip_set(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "()");
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 2 – filter_inner_type
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn filter_box_string() {
    let filtered = filter_inner_type(&ty("Box<String>"), &skip_set(&["Box"]));
    assert_eq!(ts(&filtered), "String");
}

#[test]
fn filter_arc_u32() {
    let filtered = filter_inner_type(&ty("Arc<u32>"), &skip_set(&["Arc"]));
    assert_eq!(ts(&filtered), "u32");
}

#[test]
fn filter_nested_box_arc() {
    let filtered = filter_inner_type(&ty("Box<Arc<i64>>"), &skip_set(&["Box", "Arc"]));
    assert_eq!(ts(&filtered), "i64");
}

#[test]
fn filter_non_skip_type_unchanged() {
    let filtered = filter_inner_type(&ty("Vec<String>"), &skip_set(&["Box"]));
    assert_eq!(ts(&filtered), "Vec < String >");
}

#[test]
fn filter_empty_skip_returns_original() {
    let filtered = filter_inner_type(&ty("Box<u8>"), &skip_set(&[]));
    assert_eq!(ts(&filtered), "Box < u8 >");
}

#[test]
fn filter_reference_type_unchanged() {
    let filtered = filter_inner_type(&ty("&mut i32"), &skip_set(&["Box"]));
    assert_eq!(ts(&filtered), "& mut i32");
}

#[test]
fn filter_plain_ident_unchanged() {
    let filtered = filter_inner_type(&ty("MyStruct"), &skip_set(&["Box", "Arc"]));
    assert_eq!(ts(&filtered), "MyStruct");
}

#[test]
fn filter_triple_nested() {
    let filtered = filter_inner_type(&ty("Box<Arc<Box<bool>>>"), &skip_set(&["Box", "Arc"]));
    assert_eq!(ts(&filtered), "bool");
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 3 – wrap_leaf_type
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn wrap_plain_string() {
    let wrapped = wrap_leaf_type(&ty("String"), &skip_set(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_i32() {
    let wrapped = wrap_leaf_type(&ty("i32"), &skip_set(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_vec_skips_vec_wraps_inner() {
    let wrapped = wrap_leaf_type(&ty("Vec<String>"), &skip_set(&["Vec"]));
    assert_eq!(ts(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_option_skips_option_wraps_inner() {
    let wrapped = wrap_leaf_type(&ty("Option<u64>"), &skip_set(&["Option"]));
    assert_eq!(ts(&wrapped), "Option < adze :: WithLeaf < u64 > >");
}

#[test]
fn wrap_nested_vec_option() {
    let wrapped = wrap_leaf_type(&ty("Vec<Option<bool>>"), &skip_set(&["Vec", "Option"]));
    assert_eq!(ts(&wrapped), "Vec < Option < adze :: WithLeaf < bool > > >");
}

#[test]
fn wrap_non_skip_container_wraps_whole() {
    let wrapped = wrap_leaf_type(&ty("Box<i32>"), &skip_set(&["Vec"]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < Box < i32 > >");
}

#[test]
fn wrap_reference_type() {
    let wrapped = wrap_leaf_type(&ty("&str"), &skip_set(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn wrap_array_type() {
    let wrapped = wrap_leaf_type(&ty("[u8; 4]"), &skip_set(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn wrap_result_skips_wraps_both_args() {
    let wrapped = wrap_leaf_type(&ty("Result<String, i32>"), &skip_set(&["Result"]));
    assert_eq!(
        ts(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 4 – NameValueExpr parsing
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn nve_string_literal() {
    let nve: NameValueExpr = parse_str("text = \"hello\"").unwrap();
    assert_eq!(nve.path.to_string(), "text");
}

#[test]
fn nve_integer_literal() {
    let nve: NameValueExpr = parse_str("level = 5").unwrap();
    assert_eq!(nve.path.to_string(), "level");
}

#[test]
fn nve_bool_literal() {
    let nve: NameValueExpr = parse_str("non_empty = true").unwrap();
    assert_eq!(nve.path.to_string(), "non_empty");
}

#[test]
fn nve_regex_pattern() {
    let nve: NameValueExpr = parse_str(r#"pattern = r"\d+""#).unwrap();
    assert_eq!(nve.path.to_string(), "pattern");
}

#[test]
fn nve_closure_expr() {
    let nve: NameValueExpr = parse_str("transform = |v| v.parse().unwrap()").unwrap();
    assert_eq!(nve.path.to_string(), "transform");
}

#[test]
fn nve_path_expr() {
    let nve: NameValueExpr = parse_str("kind = MyEnum::Variant").unwrap();
    assert_eq!(nve.path.to_string(), "kind");
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 5 – FieldThenParams parsing
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ftp_bare_type() {
    let ftp: FieldThenParams = parse_str("i32").unwrap();
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_type_with_one_param() {
    let ftp: FieldThenParams = parse_str("String, name = \"x\"").unwrap();
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "name");
}

#[test]
fn ftp_type_with_two_params() {
    let ftp: FieldThenParams = parse_str("u32, min = 0, max = 100").unwrap();
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "min");
    assert_eq!(ftp.params[1].path.to_string(), "max");
}

#[test]
fn ftp_unit_type() {
    let ftp: FieldThenParams = parse_str("()").unwrap();
    assert!(ftp.params.is_empty());
    assert_eq!(ftp.field.ty.to_token_stream().to_string(), "()");
}

#[test]
fn ftp_generic_type() {
    let ftp: FieldThenParams = parse_str("Vec<i32>").unwrap();
    assert!(ftp.params.is_empty());
    assert!(ftp.field.ty.to_token_stream().to_string().contains("Vec"));
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 6 – Token stream / quote patterns
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn quote_ident_interpolation() {
    let name = format_ident!("parser_fn");
    let tokens = quote! { fn #name() {} };
    let s = tokens.to_string();
    assert!(s.contains("parser_fn"));
}

#[test]
fn quote_type_interpolation() {
    let field_ty: Type = parse_quote!(Option<String>);
    let tokens = quote! { let _x: #field_ty = None; };
    let s = tokens.to_string();
    assert!(s.contains("Option"));
    assert!(s.contains("String"));
}

#[test]
fn quote_repetition_expands_all() {
    let ids = vec![format_ident!("a"), format_ident!("b")];
    let tokens = quote! { #(let #ids = 0;)* };
    let s = tokens.to_string();
    assert!(s.contains("let a"));
    assert!(s.contains("let b"));
}

#[test]
fn quote_separated_repetition() {
    let variants = ["X", "Y", "Z"];
    let idents: Vec<_> = variants.iter().map(|v| format_ident!("{v}")).collect();
    let tokens = quote! { enum E { #(#idents),* } };
    let s = tokens.to_string();
    for v in &variants {
        assert!(s.contains(v), "Missing variant {v} in: {s}");
    }
}

#[test]
fn quote_conditional_derive() {
    let add_derive = true;
    let derive_attr = if add_derive {
        quote! { #[derive(Clone)] }
    } else {
        quote! {}
    };
    let tokens = quote! { #derive_attr struct Foo; };
    assert!(tokens.to_string().contains("Clone"));
}

#[test]
fn quote_impl_block_generation() {
    let name = format_ident!("MyNode");
    let tokens = quote! {
        impl #name {
            fn new() -> Self { todo!() }
        }
    };
    let s = tokens.to_string();
    assert!(s.contains("impl MyNode"));
    assert!(s.contains("fn new"));
}

#[test]
fn format_ident_symbol_name() {
    let enum_name = format_ident!("Expr");
    let variant = format_ident!("Add");
    let symbol = format!("{enum_name}_{variant}");
    assert_eq!(symbol, "Expr_Add");
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 7 – Struct attribute processing
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn struct_language_attr_detected() {
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
fn struct_extra_attr_detected() {
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
fn struct_word_attr_detected() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        struct Ident {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
}

#[test]
fn struct_external_attr_detected() {
    let s: ItemStruct = parse_quote! {
        #[adze::external]
        struct IndentToken;
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "external")));
}

#[test]
fn struct_multiple_attrs() {
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

#[test]
fn struct_leaf_field_params_text() {
    let s: ItemStruct = parse_quote! {
        pub struct Semi {
            #[adze::leaf(text = ";")]
            _tok: (),
        }
    };
    let f = s.fields.iter().next().unwrap();
    let attr = f.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
    let params = leaf_params(attr);
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].path.to_string(), "text");
}

#[test]
fn struct_leaf_field_params_pattern_and_transform() {
    let s: ItemStruct = parse_quote! {
        pub struct Num {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: i32,
        }
    };
    let f = s.fields.iter().next().unwrap();
    let attr = f.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
    let params = leaf_params(attr);
    let names: Vec<_> = params.iter().map(|p| p.path.to_string()).collect();
    assert!(names.contains(&"pattern".to_string()));
    assert!(names.contains(&"transform".to_string()));
}

#[test]
fn struct_skip_field_attr() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            #[adze::leaf(pattern = r"\w+")]
            text: String,
            #[adze::skip(false)]
            visited: bool,
        }
    };
    let skip_f = s.fields.iter().nth(1).unwrap();
    assert!(skip_f.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    assert_eq!(ts(&skip_f.ty), "bool");
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 8 – Enum attribute processing & variant shapes
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn enum_language_attr() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Expr {
            Num(#[adze::leaf(pattern = r"\d+")] String),
        }
    };
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn enum_unit_variant_leaf() {
    let e: ItemEnum = parse_quote! {
        pub enum Kw {
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
fn enum_unnamed_variant_fields() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Neg(
                #[adze::leaf(text = "-")] (),
                Box<Expr>
            ),
        }
    };
    let v = e.variants.iter().next().unwrap();
    let types = variant_field_types(v);
    assert_eq!(types.len(), 2);
    assert_eq!(types[0], "()");
    assert!(types[1].contains("Box"));
}

#[test]
fn enum_named_variant_fields() {
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
    assert!(matches!(v.fields, Fields::Named(_)));
    let names: Vec<_> = v
        .fields
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
        .collect();
    assert_eq!(names, ["target", "_eq", "value"]);
}

#[test]
fn enum_prec_left_attr_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(2)]
            Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
        }
    };
    let v = e.variants.iter().next().unwrap();
    let attr = v
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_left"))
        .unwrap();
    let expr: Expr = attr.parse_args().unwrap();
    if let Expr::Lit(ExprLit {
        lit: Lit::Int(i), ..
    }) = expr
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 2);
    } else {
        panic!("Expected integer literal");
    }
}

#[test]
fn enum_prec_right_attr_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_right(7)]
            Cons(Box<Expr>, #[adze::leaf(text = "::")] (), Box<Expr>),
        }
    };
    let v = e.variants.iter().next().unwrap();
    let attr = v
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_right"))
        .unwrap();
    let expr: Expr = attr.parse_args().unwrap();
    if let Expr::Lit(ExprLit {
        lit: Lit::Int(i), ..
    }) = expr
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 7);
    } else {
        panic!("Expected integer literal");
    }
}

#[test]
fn enum_prec_no_assoc_attr_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec(4)]
            Eq(Box<Expr>, #[adze::leaf(text = "==")] (), Box<Expr>),
        }
    };
    let v = e.variants.iter().next().unwrap();
    let attr = v.attrs.iter().find(|a| is_adze_attr(a, "prec")).unwrap();
    let expr: Expr = attr.parse_args().unwrap();
    if let Expr::Lit(ExprLit {
        lit: Lit::Int(i), ..
    }) = expr
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 4);
    } else {
        panic!("Expected integer literal");
    }
}

#[test]
fn enum_repeat_attr_on_variant_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            List(
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

#[test]
fn enum_delimited_attr_on_variant_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Csv(
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                Vec<Number>
            ),
        }
    };
    let v = e.variants.iter().next().unwrap();
    let f = match &v.fields {
        Fields::Unnamed(u) => u.unnamed.iter().next().unwrap(),
        _ => panic!("expected unnamed"),
    };
    assert!(f.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 9 – Grammar module structure
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn grammar_module_extracts_name() {
    let m = parse_mod(quote! {
        #[adze::grammar("calc")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    });
    let name_attr = m.attrs.iter().find(|a| is_adze_attr(a, "grammar")).unwrap();
    let expr: Expr = name_attr.parse_args().unwrap();
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(s), ..
    }) = expr
    {
        assert_eq!(s.value(), "calc");
    } else {
        panic!("Expected string literal");
    }
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
    let found = module_items(&m).iter().any(|item| {
        if let Item::Enum(e) = item {
            e.attrs.iter().any(|a| is_adze_attr(a, "language"))
        } else {
            false
        }
    });
    assert!(found);
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
    let found = module_items(&m).iter().any(|item| {
        if let Item::Struct(s) = item {
            s.attrs.iter().any(|a| is_adze_attr(a, "language"))
        } else {
            false
        }
    });
    assert!(found);
}

#[test]
fn grammar_module_contains_extra_struct() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
            }
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    let ws = find_struct_in_mod(&m, "Ws").unwrap();
    assert!(ws.attrs.iter().any(|a| is_adze_attr(a, "extra")));
}

#[test]
fn grammar_module_multi_type_count() {
    let m = parse_mod(quote! {
        #[adze::grammar("lang")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
            }
            struct Number {
                #[adze::leaf(pattern = r"\d+")]
                value: String,
            }
            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    let items = module_items(&m);
    let struct_count = items
        .iter()
        .filter(|i| matches!(i, Item::Struct(_)))
        .count();
    let enum_count = items.iter().filter(|i| matches!(i, Item::Enum(_))).count();
    assert_eq!(struct_count, 2);
    assert_eq!(enum_count, 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 10 – Type manipulation integration
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn extract_then_filter_option_box() {
    let skip = skip_set(&["Box"]);
    let orig = ty("Option<Box<String>>");
    let (inner, ok) = try_extract_inner_type(&orig, "Option", &skip);
    assert!(ok);
    let filtered = filter_inner_type(&inner, &skip);
    assert_eq!(ts(&filtered), "String");
}

#[test]
fn extract_then_wrap_vec_content() {
    let orig = ty("Vec<MyNode>");
    let (inner, ok) = try_extract_inner_type(&orig, "Vec", &skip_set(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip_set(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < MyNode >");
}

#[test]
fn filter_then_wrap() {
    let orig = ty("Box<SomeType>");
    let filtered = filter_inner_type(&orig, &skip_set(&["Box"]));
    assert_eq!(ts(&filtered), "SomeType");
    let wrapped = wrap_leaf_type(&filtered, &skip_set(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < SomeType >");
}

#[test]
fn wrap_preserves_vec_option_nesting() {
    let skip = skip_set(&["Vec", "Option"]);
    let wrapped = wrap_leaf_type(&ty("Vec<Option<i32>>"), &skip);
    assert_eq!(ts(&wrapped), "Vec < Option < adze :: WithLeaf < i32 > > >");
}

#[test]
fn extract_vec_from_struct_field() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::repeat(non_empty = true)]
            items: Vec<Number>,
        }
    };
    let f = s.fields.iter().next().unwrap();
    let (inner, ok) = try_extract_inner_type(&f.ty, "Vec", &skip_set(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Number");
}

#[test]
fn extract_option_from_struct_field() {
    let s: ItemStruct = parse_quote! {
        pub struct Maybe {
            value: Option<i32>,
        }
    };
    let f = s.fields.iter().next().unwrap();
    let (inner, ok) = try_extract_inner_type(&f.ty, "Option", &skip_set(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "i32");
}

#[test]
fn filter_box_from_enum_variant_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Neg(#[adze::leaf(text = "-")] (), Box<Expr>),
        }
    };
    let v = e.variants.iter().next().unwrap();
    let box_field = match &v.fields {
        Fields::Unnamed(u) => u.unnamed.iter().nth(1).unwrap(),
        _ => panic!("expected unnamed"),
    };
    let filtered = filter_inner_type(&box_field.ty, &skip_set(&["Box"]));
    assert_eq!(ts(&filtered), "Expr");
}
