#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for struct expansion and annotation processing.
//!
//! Covers: single/multi-field structs, Option/Vec/Box fields, leaf/skip/delimited/repeat
//! annotations, mixed attributes, visibility, generics, doc comments, derives, and
//! cross-reference patterns within grammar modules.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::{Attribute, Fields, Item, ItemMod, ItemStruct, Token, parse_quote};

use adze_common::NameValueExpr;

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

fn module_items(m: &ItemMod) -> &Vec<Item> {
    &m.content.as_ref().unwrap().1
}

fn leaf_params(attr: &Attribute) -> Punctuated<NameValueExpr, Token![,]> {
    attr.parse_args_with(Punctuated::<NameValueExpr, Token![,]>::parse_terminated)
        .unwrap()
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

fn field_names(s: &ItemStruct) -> Vec<String> {
    s.fields
        .iter()
        .filter_map(|f| f.ident.as_ref().map(|i| i.to_string()))
        .collect()
}

fn field_type_strings(s: &ItemStruct) -> Vec<String> {
    s.fields
        .iter()
        .map(|f| f.ty.to_token_stream().to_string())
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Simple struct with one field → grammar extraction
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t01_single_leaf_field_string() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Token {
            #[adze::leaf(pattern = r"\w+")]
            value: String,
        }
    };
    assert_eq!(s.fields.len(), 1);
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
    let f = s.fields.iter().next().unwrap();
    assert_eq!(f.ident.as_ref().unwrap(), "value");
    assert_eq!(f.ty.to_token_stream().to_string(), "String");
}

#[test]
fn t01_single_leaf_field_i32() {
    let s: ItemStruct = parse_quote! {
        pub struct Number {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: i32,
        }
    };
    assert_eq!(s.fields.len(), 1);
    let f = s.fields.iter().next().unwrap();
    assert_eq!(f.ty.to_token_stream().to_string(), "i32");
    let attr = f.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
    let params = leaf_params(attr);
    let names: Vec<_> = params.iter().map(|p| p.path.to_string()).collect();
    assert!(names.contains(&"pattern".to_string()));
    assert!(names.contains(&"transform".to_string()));
}

#[test]
fn t01_single_leaf_text_field() {
    let s: ItemStruct = parse_quote! {
        pub struct Plus {
            #[adze::leaf(text = "+")]
            _plus: (),
        }
    };
    let f = s.fields.iter().next().unwrap();
    assert_eq!(f.ty.to_token_stream().to_string(), "()");
    let attr = f.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
    let params = leaf_params(attr);
    assert_eq!(params[0].path.to_string(), "text");
}

#[test]
fn t01_single_field_in_grammar_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                word: String,
            }
        }
    });
    let root = find_struct_in_mod(&m, "Root").unwrap();
    assert_eq!(root.fields.len(), 1);
    assert!(root.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Struct with multiple fields → multiple rule elements
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t02_two_leaf_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct Pair {
            #[adze::leaf(pattern = r"[a-z]+")]
            key: String,
            #[adze::leaf(pattern = r"\d+")]
            value: String,
        }
    };
    assert_eq!(s.fields.len(), 2);
    assert_eq!(field_names(&s), vec!["key", "value"]);
}

#[test]
fn t02_three_fields_with_punctuation() {
    let s: ItemStruct = parse_quote! {
        pub struct Assignment {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
            #[adze::leaf(text = "=")]
            _eq: (),
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: i32,
        }
    };
    assert_eq!(s.fields.len(), 3);
    let types = field_type_strings(&s);
    assert_eq!(types, vec!["String", "()", "i32"]);
}

#[test]
fn t02_five_fields_ordering_preserved() {
    let s: ItemStruct = parse_quote! {
        pub struct Statement {
            #[adze::leaf(text = "let")]
            _kw: (),
            #[adze::leaf(pattern = r"[a-z]+")]
            name: String,
            #[adze::leaf(text = "=")]
            _eq: (),
            #[adze::leaf(pattern = r"\d+")]
            val: String,
            #[adze::leaf(text = ";")]
            _semi: (),
        }
    };
    assert_eq!(s.fields.len(), 5);
    assert_eq!(field_names(&s), vec!["_kw", "name", "_eq", "val", "_semi"]);
}

#[test]
fn t02_mixed_leaf_and_reference_fields() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                header: Header,
                #[adze::leaf(text = "{")]
                _open: (),
                body: Body,
                #[adze::leaf(text = "}")]
                _close: (),
            }

            pub struct Header {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }

            pub struct Body {
                #[adze::leaf(pattern = r"[^}]+")]
                content: String,
            }
        }
    });
    let prog = find_struct_in_mod(&m, "Program").unwrap();
    assert_eq!(prog.fields.len(), 4);
    let types = field_type_strings(prog);
    assert_eq!(types, vec!["Header", "()", "Body", "()"]);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Struct with Option fields → optional grammar elements
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t03_option_leaf_field() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct MaybeNum {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            v: Option<i32>,
        }
    };
    assert_eq!(field_type_strings(&s)[0], "Option < i32 >");
}

#[test]
fn t03_option_reference_field() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                child: Option<Child>,
            }

            pub struct Child {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    });
    let root = find_struct_in_mod(&m, "Root").unwrap();
    assert_eq!(field_type_strings(root)[0], "Option < Child >");
}

#[test]
fn t03_multiple_option_fields() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Flexible {
            #[adze::leaf(pattern = r"[a-z]+")]
            name: Option<String>,
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            age: Option<i32>,
            extra: Option<Other>,
        }
    };
    assert_eq!(s.fields.len(), 3);
    for ty in field_type_strings(&s) {
        assert!(ty.starts_with("Option"), "expected Option type, got {ty}");
    }
}

#[test]
fn t03_option_mixed_with_required() {
    let s: ItemStruct = parse_quote! {
        pub struct MixedReq {
            #[adze::leaf(pattern = r"\w+")]
            required_name: String,
            #[adze::leaf(pattern = r"\d+")]
            optional_num: Option<String>,
        }
    };
    let types = field_type_strings(&s);
    assert_eq!(types[0], "String");
    assert_eq!(types[1], "Option < String >");
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Struct with Vec fields → repetition grammar elements
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t04_vec_field_type() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct NumberList {
            numbers: Vec<Number>,
        }
    };
    assert_eq!(field_type_strings(&s)[0], "Vec < Number >");
}

#[test]
fn t04_vec_with_leaf_inner_type() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct TokenList {
                tokens: Vec<Tok>,
            }

            pub struct Tok {
                #[adze::leaf(pattern = r"\w+")]
                value: String,
            }
        }
    });
    let tl = find_struct_in_mod(&m, "TokenList").unwrap();
    assert_eq!(field_type_strings(tl)[0], "Vec < Tok >");
}

#[test]
fn t04_vec_field_alongside_other_fields() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Container {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
            items: Vec<Item>,
            #[adze::leaf(text = ";")]
            _end: (),
        }
    };
    let types = field_type_strings(&s);
    assert_eq!(types, vec!["String", "Vec < Item >", "()"]);
}

#[test]
fn t04_multiple_vec_fields() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct MultiList {
            items_a: Vec<TypeA>,
            items_b: Vec<TypeB>,
        }
    };
    assert_eq!(s.fields.len(), 2);
    for ty in field_type_strings(&s) {
        assert!(ty.starts_with("Vec"), "expected Vec type, got {ty}");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Struct with Box fields → indirection handling
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t05_box_field_type() {
    let s: ItemStruct = parse_quote! {
        pub struct Wrapper {
            inner: Box<Inner>,
        }
    };
    assert_eq!(field_type_strings(&s)[0], "Box < Inner >");
}

#[test]
fn t05_box_self_reference() {
    let s: ItemStruct = parse_quote! {
        pub struct Recursive {
            #[adze::leaf(pattern = r"\w+")]
            value: String,
            next: Option<Box<Recursive>>,
        }
    };
    let types = field_type_strings(&s);
    assert_eq!(types[1], "Option < Box < Recursive > >");
}

#[test]
fn t05_multiple_box_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct BinOp {
            left: Box<Expr>,
            #[adze::leaf(text = "+")]
            _op: (),
            right: Box<Expr>,
        }
    };
    let types = field_type_strings(&s);
    assert_eq!(types[0], "Box < Expr >");
    assert_eq!(types[1], "()");
    assert_eq!(types[2], "Box < Expr >");
}

#[test]
fn t05_box_in_grammar_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                child: Box<Child>,
            }

            pub struct Child {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    });
    let root = find_struct_in_mod(&m, "Root").unwrap();
    assert_eq!(field_type_strings(root)[0], "Box < Child >");
    assert!(find_struct_in_mod(&m, "Child").is_some());
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Struct with #[adze::leaf] annotated fields
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t06_leaf_pattern_param() {
    let s: ItemStruct = parse_quote! {
        pub struct Ident {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
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
    assert_eq!(params[0].path.to_string(), "pattern");
}

#[test]
fn t06_leaf_text_param() {
    let s: ItemStruct = parse_quote! {
        pub struct Semi {
            #[adze::leaf(text = ";")]
            _semi: (),
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
    assert_eq!(params[0].path.to_string(), "text");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(lit),
        ..
    }) = &params[0].expr
    {
        assert_eq!(lit.value(), ";");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn t06_leaf_with_transform_closure() {
    let s: ItemStruct = parse_quote! {
        pub struct Num {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            val: i32,
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
    let names: Vec<_> = params.iter().map(|p| p.path.to_string()).collect();
    assert!(names.contains(&"transform".to_string()));
}

#[test]
fn t06_leaf_with_typed_transform() {
    let s: ItemStruct = parse_quote! {
        pub struct TypedNum {
            #[adze::leaf(pattern = r"\d+", transform = |v: &str| v.parse::<u64>().unwrap())]
            val: u64,
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
fn t06_leaf_on_unit_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::leaf(text = "nil")]
        struct Nil;
    };
    assert!(matches!(s.fields, Fields::Unit));
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

#[test]
fn t06_leaf_complex_regex() {
    let s: ItemStruct = parse_quote! {
        pub struct StrLit {
            #[adze::leaf(pattern = r#""([^"\\]|\\.)*""#)]
            value: String,
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
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(lit),
        ..
    }) = &params[0].expr
    {
        assert!(lit.value().contains(r#"([^"\\]|\\.)*"#));
    } else {
        panic!("Expected string literal");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Struct with #[adze::skip] fields
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t07_skip_bool_default() {
    let s: ItemStruct = parse_quote! {
        pub struct Node {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
            #[adze::skip(false)]
            visited: bool,
        }
    };
    let skip_f = s
        .fields
        .iter()
        .find(|f| f.ident.as_ref().unwrap() == "visited")
        .unwrap();
    assert!(skip_f.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    let attr = skip_f
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "skip"))
        .unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    assert_eq!(expr.to_token_stream().to_string(), "false");
}

#[test]
fn t07_skip_integer_default() {
    let s: ItemStruct = parse_quote! {
        pub struct Counter {
            #[adze::leaf(pattern = r"\d+")]
            val: String,
            #[adze::skip(0)]
            count: i32,
        }
    };
    let skip_f = s
        .fields
        .iter()
        .find(|f| f.ident.as_ref().unwrap() == "count")
        .unwrap();
    let attr = skip_f
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "skip"))
        .unwrap();
    let expr: syn::Expr = attr.parse_args().unwrap();
    assert_eq!(expr.to_token_stream().to_string(), "0");
}

#[test]
fn t07_skip_string_default() {
    let s: ItemStruct = parse_quote! {
        pub struct Tagged {
            #[adze::leaf(pattern = r"\w+")]
            tag: String,
            #[adze::skip("default")]
            label: String,
        }
    };
    let skip_f = s
        .fields
        .iter()
        .find(|f| f.ident.as_ref().unwrap() == "label")
        .unwrap();
    assert!(skip_f.attrs.iter().any(|a| is_adze_attr(a, "skip")));
}

#[test]
fn t07_multiple_skip_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct MultiSkip {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
            #[adze::skip(false)]
            flag_a: bool,
            #[adze::skip(0)]
            counter: i32,
            #[adze::skip(true)]
            flag_b: bool,
        }
    };
    let skip_count = s
        .fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "skip")))
        .count();
    assert_eq!(skip_count, 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Struct with #[adze::delimited] fields
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t08_delimited_comma() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct CsvRow {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<Item>,
        }
    };
    let f = s.fields.iter().next().unwrap();
    assert!(f.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
    assert_eq!(f.ty.to_token_stream().to_string(), "Vec < Item >");
}

#[test]
fn t08_delimited_semicolon() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct StmtList {
            #[adze::delimited(
                #[adze::leaf(text = ";")]
                ()
            )]
            stmts: Vec<Stmt>,
        }
    };
    let f = s.fields.iter().next().unwrap();
    assert!(f.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
}

#[test]
fn t08_delimited_with_repeat() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct NonEmptyList {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            #[adze::repeat(non_empty = true)]
            items: Vec<Item>,
        }
    };
    let f = s.fields.iter().next().unwrap();
    let attr_names = adze_attr_names(&f.attrs);
    assert!(attr_names.contains(&"delimited".to_string()));
    assert!(attr_names.contains(&"repeat".to_string()));
}

#[test]
fn t08_delimited_in_grammar_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct NumberList {
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                numbers: Vec<Number>,
            }

            pub struct Number {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                v: i32,
            }
        }
    });
    let nl = find_struct_in_mod(&m, "NumberList").unwrap();
    let f = nl.fields.iter().next().unwrap();
    assert!(f.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Struct with #[adze::repeat] fields
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t09_repeat_non_empty_true() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Items {
            #[adze::repeat(non_empty = true)]
            things: Vec<Thing>,
        }
    };
    let f = s.fields.iter().next().unwrap();
    assert!(f.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
}

#[test]
fn t09_repeat_non_empty_false() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct Items {
            #[adze::repeat(non_empty = false)]
            things: Vec<Thing>,
        }
    };
    let f = s.fields.iter().next().unwrap();
    assert!(f.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
}

#[test]
fn t09_repeat_vec_field_only() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        pub struct OnlyRepeats {
            #[adze::repeat(non_empty = true)]
            items: Vec<MyItem>,
        }
    };
    assert_eq!(s.fields.len(), 1);
    assert_eq!(field_type_strings(&s)[0], "Vec < MyItem >");
}

#[test]
fn t09_repeat_in_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::repeat(non_empty = true)]
                children: Vec<Child>,
            }

            pub struct Child {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    });
    let root = find_struct_in_mod(&m, "Root").unwrap();
    let f = root.fields.iter().next().unwrap();
    assert!(f.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Struct with mixed attributes
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t10_leaf_and_skip_mixed() {
    let s: ItemStruct = parse_quote! {
        pub struct MixedNode {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            value: i32,
            #[adze::leaf(text = ";")]
            _semi: (),
            #[adze::skip(false)]
            processed: bool,
        }
    };
    let attrs: Vec<_> = s.fields.iter().map(|f| adze_attr_names(&f.attrs)).collect();
    assert_eq!(attrs[0], vec!["leaf"]);
    assert_eq!(attrs[1], vec!["leaf"]);
    assert_eq!(attrs[2], vec!["skip"]);
}

#[test]
fn t10_leaf_reference_and_skip() {
    let s: ItemStruct = parse_quote! {
        pub struct ComplexNode {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
            child: OtherNode,
            #[adze::skip(0)]
            depth: i32,
        }
    };
    assert_eq!(s.fields.len(), 3);
    let f2 = s.fields.iter().nth(1).unwrap();
    assert!(f2.attrs.is_empty());
}

#[test]
fn t10_delimited_and_repeat_on_same_field() {
    let s: ItemStruct = parse_quote! {
        pub struct DelimRep {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            #[adze::repeat(non_empty = true)]
            vals: Vec<Val>,
        }
    };
    let f = s.fields.iter().next().unwrap();
    let names = adze_attr_names(&f.attrs);
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"delimited".to_string()));
    assert!(names.contains(&"repeat".to_string()));
}

#[test]
fn t10_language_extra_word_external_in_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("combo")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                token: Ident,
            }

            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }

            #[adze::external]
            struct IndentToken;
        }
    });
    let items = module_items(&m);
    let struct_attrs: Vec<Vec<String>> = items
        .iter()
        .filter_map(|i| {
            if let Item::Struct(s) = i {
                Some(adze_attr_names(&s.attrs))
            } else {
                None
            }
        })
        .collect();
    let all: Vec<String> = struct_attrs.into_iter().flatten().collect();
    assert!(all.contains(&"language".to_string()));
    assert!(all.contains(&"word".to_string()));
    assert!(all.contains(&"extra".to_string()));
    assert!(all.contains(&"external".to_string()));
}

#[test]
fn t10_vec_option_box_fields_together() {
    let s: ItemStruct = parse_quote! {
        pub struct Everything {
            items: Vec<Item>,
            maybe: Option<Other>,
            indirect: Box<More>,
            #[adze::leaf(pattern = r"\w+")]
            direct: String,
        }
    };
    let types = field_type_strings(&s);
    assert_eq!(types[0], "Vec < Item >");
    assert_eq!(types[1], "Option < Other >");
    assert_eq!(types[2], "Box < More >");
    assert_eq!(types[3], "String");
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. Struct visibility (pub, pub(crate), private)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t11_pub_visibility() {
    let s: ItemStruct = parse_quote! {
        pub struct PubStruct {
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    assert!(matches!(s.vis, syn::Visibility::Public(_)));
}

#[test]
fn t11_private_visibility() {
    let s: ItemStruct = parse_quote! {
        struct PrivStruct {
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    assert!(matches!(s.vis, syn::Visibility::Inherited));
}

#[test]
fn t11_pub_crate_visibility() {
    let s: ItemStruct = parse_quote! {
        pub(crate) struct CrateStruct {
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    assert!(matches!(s.vis, syn::Visibility::Restricted(_)));
}

#[test]
fn t11_pub_super_visibility() {
    let s: ItemStruct = parse_quote! {
        pub(super) struct SuperStruct {
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    assert!(matches!(s.vis, syn::Visibility::Restricted(_)));
}

#[test]
fn t11_visibility_preserved_in_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }

            struct Private {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }

            pub(crate) struct PubCrate {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
        }
    });
    let root = find_struct_in_mod(&m, "Root").unwrap();
    assert!(matches!(root.vis, syn::Visibility::Public(_)));
    let priv_s = find_struct_in_mod(&m, "Private").unwrap();
    assert!(matches!(priv_s.vis, syn::Visibility::Inherited));
    let crate_s = find_struct_in_mod(&m, "PubCrate").unwrap();
    assert!(matches!(crate_s.vis, syn::Visibility::Restricted(_)));
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. Struct with generics
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t12_struct_with_lifetime_param() {
    let s: ItemStruct = parse_quote! {
        pub struct Borrowed<'a> {
            #[adze::leaf(pattern = r"\w+")]
            name: &'a str,
        }
    };
    assert_eq!(s.generics.lifetimes().count(), 1);
    assert_eq!(s.ident, "Borrowed");
}

#[test]
fn t12_struct_with_type_param() {
    let s: ItemStruct = parse_quote! {
        pub struct Wrapper<T> {
            inner: T,
        }
    };
    assert_eq!(s.generics.type_params().count(), 1);
}

#[test]
fn t12_struct_with_multiple_type_params() {
    let s: ItemStruct = parse_quote! {
        pub struct Pair<A, B> {
            first: A,
            second: B,
        }
    };
    assert_eq!(s.generics.type_params().count(), 2);
    assert_eq!(s.fields.len(), 2);
}

#[test]
fn t12_struct_with_where_clause() {
    let s: ItemStruct = parse_quote! {
        pub struct Bounded<T>
        where
            T: Clone + Send,
        {
            inner: T,
        }
    };
    assert!(s.generics.where_clause.is_some());
}

#[test]
fn t12_generic_params_preserved_ident() {
    let s: ItemStruct = parse_quote! {
        pub struct Container<T, U> {
            left: T,
            right: U,
        }
    };
    let param_names: Vec<_> = s
        .generics
        .type_params()
        .map(|p| p.ident.to_string())
        .collect();
    assert_eq!(param_names, vec!["T", "U"]);
}

// ═══════════════════════════════════════════════════════════════════════════
// 13. Struct documentation comments
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t13_doc_comment_on_struct() {
    let s: ItemStruct = parse_quote! {
        /// This is a root node.
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    let doc_attrs: Vec<_> = s
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .collect();
    assert_eq!(doc_attrs.len(), 1);
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn t13_multi_line_doc_comment() {
    let s: ItemStruct = parse_quote! {
        /// Line one.
        /// Line two.
        /// Line three.
        pub struct Documented {
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    let doc_count = s.attrs.iter().filter(|a| a.path().is_ident("doc")).count();
    assert_eq!(doc_count, 3);
}

#[test]
fn t13_doc_comment_on_field() {
    let s: ItemStruct = parse_quote! {
        pub struct WithFieldDocs {
            /// The token value.
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    let f = s.fields.iter().next().unwrap();
    let doc_count = f.attrs.iter().filter(|a| a.path().is_ident("doc")).count();
    assert_eq!(doc_count, 1);
    assert!(f.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

#[test]
fn t13_doc_comment_preserved_alongside_adze() {
    let s: ItemStruct = parse_quote! {
        /// Struct docs.
        #[derive(Debug)]
        #[adze::language]
        pub struct Root {
            /// Field docs.
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    assert_eq!(s.attrs.len(), 3); // doc, derive, language
    let f = s.fields.iter().next().unwrap();
    assert_eq!(f.attrs.len(), 2); // doc, leaf
}

// ═══════════════════════════════════════════════════════════════════════════
// 14. Struct derives
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t14_single_derive() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug)]
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    let has_derive = s.attrs.iter().any(|a| a.path().is_ident("derive"));
    assert!(has_derive);
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

#[test]
fn t14_multiple_derives() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug, Clone, PartialEq)]
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    let derive_count = s
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("derive"))
        .count();
    assert_eq!(derive_count, 1);
}

#[test]
fn t14_split_derives() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug)]
        #[derive(Clone)]
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    let derive_count = s
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("derive"))
        .count();
    assert_eq!(derive_count, 2);
}

#[test]
fn t14_derive_does_not_interfere_with_adze() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    let adze_names = adze_attr_names(&s.attrs);
    assert_eq!(adze_names, vec!["language"]);
}

// ═══════════════════════════════════════════════════════════════════════════
// 15. Tuple structs
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t15_tuple_struct_single_field() {
    let s: ItemStruct = parse_quote! {
        pub struct Wrapped(
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            i32
        );
    };
    assert!(matches!(s.fields, Fields::Unnamed(_)));
    assert_eq!(s.fields.len(), 1);
}

#[test]
fn t15_tuple_struct_multiple_fields() {
    let s: ItemStruct = parse_quote! {
        pub struct Triple(
            #[adze::leaf(pattern = r"\d+")]
            String,
            #[adze::leaf(text = ",")]
            (),
            #[adze::leaf(pattern = r"\d+")]
            String,
        );
    };
    assert_eq!(s.fields.len(), 3);
}

#[test]
fn t15_tuple_struct_no_field_names() {
    let s: ItemStruct = parse_quote! {
        pub struct NoNames(
            #[adze::leaf(pattern = r"\w+")]
            String,
        );
    };
    assert!(s.fields.iter().all(|f| f.ident.is_none()));
}

// ═══════════════════════════════════════════════════════════════════════════
// 16. Unit structs
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t16_unit_struct_leaf() {
    let s: ItemStruct = parse_quote! {
        #[adze::leaf(text = "true")]
        struct TrueKw;
    };
    assert!(matches!(s.fields, Fields::Unit));
    let attr = s.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
    let params = leaf_params(attr);
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(lit),
        ..
    }) = &params[0].expr
    {
        assert_eq!(lit.value(), "true");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn t16_unit_struct_external() {
    let s: ItemStruct = parse_quote! {
        #[adze::external]
        struct Indent;
    };
    assert!(matches!(s.fields, Fields::Unit));
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "external")));
}

// ═══════════════════════════════════════════════════════════════════════════
// 17. Extra annotation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t17_extra_whitespace() {
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
fn t17_extra_comment() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Comment {
            #[adze::leaf(pattern = r"//[^\n]*")]
            _comment: (),
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
}

#[test]
fn t17_multiple_extras_in_module() {
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
                _ws: (),
            }

            #[adze::extra]
            struct Comment {
                #[adze::leaf(pattern = r"//[^\n]*")]
                _c: (),
            }
        }
    });
    let extra_count = module_items(&m)
        .iter()
        .filter(|i| {
            if let Item::Struct(s) = i {
                s.attrs.iter().any(|a| is_adze_attr(a, "extra"))
            } else {
                false
            }
        })
        .count();
    assert_eq!(extra_count, 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 18. Word annotation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t18_word_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Keyword {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
}

#[test]
fn t18_word_in_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                ident: Ident,
            }

            #[adze::word]
            pub struct Ident {
                #[adze::leaf(pattern = r"[a-z_]\w*")]
                name: String,
            }
        }
    });
    let ident = find_struct_in_mod(&m, "Ident").unwrap();
    assert!(ident.attrs.iter().any(|a| is_adze_attr(a, "word")));
}

// ═══════════════════════════════════════════════════════════════════════════
// 19. External annotation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t19_external_unit_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::external]
        struct ExternalTok;
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "external")));
    assert!(matches!(s.fields, Fields::Unit));
}

#[test]
fn t19_external_with_leaf_field() {
    let s: ItemStruct = parse_quote! {
        #[adze::external]
        struct IndentToken {
            #[adze::leaf(pattern = r"\t+")]
            _indent: (),
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "external")));
    assert_eq!(s.fields.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 20. Cross-struct references in modules
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t20_struct_references_another_struct() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Language {
                e: Expression,
            }

            pub struct Expression {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                value: i32,
            }
        }
    });
    assert!(find_struct_in_mod(&m, "Language").is_some());
    assert!(find_struct_in_mod(&m, "Expression").is_some());
    let lang = find_struct_in_mod(&m, "Language").unwrap();
    assert_eq!(field_type_strings(lang)[0], "Expression");
}

#[test]
fn t20_three_struct_chain() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct A {
                b: B,
            }

            pub struct B {
                c: C,
            }

            pub struct C {
                #[adze::leaf(pattern = r"\d+")]
                val: String,
            }
        }
    });
    assert!(find_struct_in_mod(&m, "A").is_some());
    assert!(find_struct_in_mod(&m, "B").is_some());
    assert!(find_struct_in_mod(&m, "C").is_some());
}

// ═══════════════════════════════════════════════════════════════════════════
// 21. Non-adze attributes preserved
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t21_cfg_attr_preserved() {
    let s: ItemStruct = parse_quote! {
        #[cfg(test)]
        #[adze::language]
        pub struct TestOnly {
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    assert_eq!(s.attrs.len(), 2);
    let adze_names = adze_attr_names(&s.attrs);
    assert_eq!(adze_names, vec!["language"]);
}

#[test]
fn t21_allow_attr_preserved() {
    let s: ItemStruct = parse_quote! {
        #[allow(dead_code)]
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    let has_allow = s.attrs.iter().any(|a| a.path().is_ident("allow"));
    assert!(has_allow);
}

#[test]
fn t21_multiple_non_adze_attrs() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug, Clone)]
        #[cfg(feature = "test")]
        #[allow(unused)]
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    assert_eq!(s.attrs.len(), 4);
    let adze_names = adze_attr_names(&s.attrs);
    assert_eq!(adze_names, vec!["language"]);
}

// ═══════════════════════════════════════════════════════════════════════════
// 22. Field ordering and naming
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t22_field_ordering_matches_definition() {
    let s: ItemStruct = parse_quote! {
        pub struct Ordered {
            #[adze::leaf(text = "fn")]
            _kw: (),
            #[adze::leaf(pattern = r"[a-z]+")]
            name: String,
            #[adze::leaf(text = "(")]
            _open: (),
            #[adze::leaf(text = ")")]
            _close: (),
            body: Block,
        }
    };
    assert_eq!(
        field_names(&s),
        vec!["_kw", "name", "_open", "_close", "body"]
    );
}

#[test]
fn t22_underscore_prefix_naming() {
    let s: ItemStruct = parse_quote! {
        pub struct Bracketed {
            #[adze::leaf(text = "[")]
            _lbracket: (),
            #[adze::leaf(pattern = r"\w+")]
            content: String,
            #[adze::leaf(text = "]")]
            _rbracket: (),
        }
    };
    assert_eq!(field_names(&s), vec!["_lbracket", "content", "_rbracket"]);
}

// ═══════════════════════════════════════════════════════════════════════════
// 23. Grammar module structure
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t23_grammar_attr_on_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("my_lang")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
        }
    });
    assert!(m.attrs.iter().any(|a| is_adze_attr(a, "grammar")));
    assert_eq!(m.ident, "grammar");
}

#[test]
fn t23_module_content_present() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
        }
    });
    assert!(m.content.is_some());
    assert!(!module_items(&m).is_empty());
}

#[test]
fn t23_module_contains_language_struct() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod g {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
        }
    });
    let has_language = module_items(&m).iter().any(|i| {
        if let Item::Struct(s) = i {
            s.attrs.iter().any(|a| is_adze_attr(a, "language"))
        } else {
            false
        }
    });
    assert!(has_language);
}

// ═══════════════════════════════════════════════════════════════════════════
// 24. Large struct with many fields
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t24_eight_field_struct() {
    let s: ItemStruct = parse_quote! {
        pub struct Big {
            #[adze::leaf(text = "fn")]
            _kw: (),
            #[adze::leaf(pattern = r"[a-z]+")]
            name: String,
            #[adze::leaf(text = "(")]
            _open: (),
            params: Vec<Param>,
            #[adze::leaf(text = ")")]
            _close: (),
            #[adze::leaf(text = "->")]
            _arrow: (),
            ret: ReturnType,
            body: Block,
        }
    };
    assert_eq!(s.fields.len(), 8);
}

// ═══════════════════════════════════════════════════════════════════════════
// 25. Nested Option/Vec/Box
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t25_option_box() {
    let s: ItemStruct = parse_quote! {
        pub struct OptBox {
            child: Option<Box<Child>>,
        }
    };
    assert_eq!(field_type_strings(&s)[0], "Option < Box < Child > >");
}

#[test]
fn t25_vec_box() {
    let s: ItemStruct = parse_quote! {
        pub struct VecBox {
            items: Vec<Box<Item>>,
        }
    };
    assert_eq!(field_type_strings(&s)[0], "Vec < Box < Item > >");
}

// ═══════════════════════════════════════════════════════════════════════════
// 26. Spanned type support
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t26_spanned_in_vec_field() {
    let m = parse_mod(quote! {
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
    });
    let root = find_struct_in_mod(&m, "Root").unwrap();
    assert_eq!(field_type_strings(root)[0], "Vec < Spanned < Item > >");
}

#[test]
fn t26_spanned_in_option_field() {
    let s: ItemStruct = parse_quote! {
        pub struct MaybeSpanned {
            val: Option<Spanned<Child>>,
        }
    };
    assert_eq!(field_type_strings(&s)[0], "Option < Spanned < Child > >");
}

// ═══════════════════════════════════════════════════════════════════════════
// 27. Struct ident preservation
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t27_struct_ident_simple() {
    let s: ItemStruct = parse_quote! {
        pub struct MyParser {
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    assert_eq!(s.ident, "MyParser");
}

#[test]
fn t27_struct_ident_with_numbers() {
    let s: ItemStruct = parse_quote! {
        pub struct Token2 {
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    assert_eq!(s.ident, "Token2");
}

#[test]
fn t27_struct_ident_snake_case() {
    let s: ItemStruct = parse_quote! {
        pub struct my_node {
            #[adze::leaf(pattern = r"\w+")]
            tok: String,
        }
    };
    assert_eq!(s.ident, "my_node");
}

// ═══════════════════════════════════════════════════════════════════════════
// 28. Leaf parameter combinations
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t28_leaf_pattern_only() {
    let s: ItemStruct = parse_quote! {
        pub struct PatOnly {
            #[adze::leaf(pattern = r"\d+")]
            val: String,
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
    assert_eq!(params[0].path.to_string(), "pattern");
}

#[test]
fn t28_leaf_text_only() {
    let s: ItemStruct = parse_quote! {
        pub struct TextOnly {
            #[adze::leaf(text = "->")]
            _arrow: (),
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
fn t28_leaf_pattern_and_transform() {
    let s: ItemStruct = parse_quote! {
        pub struct PatTrans {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
            val: i32,
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
    let names: Vec<_> = params.iter().map(|p| p.path.to_string()).collect();
    assert_eq!(names, vec!["pattern", "transform"]);
}

// ═══════════════════════════════════════════════════════════════════════════
// 29. Empty / minimal structs
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t29_empty_named_fields_struct() {
    let s: ItemStruct = parse_quote! {
        pub struct Empty {}
    };
    assert!(matches!(s.fields, Fields::Named(_)));
    assert_eq!(s.fields.len(), 0);
}

#[test]
fn t29_unit_struct_no_attrs() {
    let s: ItemStruct = parse_quote! {
        struct Plain;
    };
    assert!(matches!(s.fields, Fields::Unit));
    assert!(s.attrs.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 30. Multiple structs in a grammar module
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t30_count_structs_in_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                item: Item,
            }

            pub struct Item {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }

            pub struct Sub {
                #[adze::leaf(pattern = r"\d+")]
                val: String,
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    let struct_count = module_items(&m)
        .iter()
        .filter(|i| matches!(i, Item::Struct(_)))
        .count();
    assert_eq!(struct_count, 4);
}

#[test]
fn t30_find_all_structs_by_name() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Alpha {
                b: Beta,
            }

            pub struct Beta {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }

            pub struct Gamma {
                #[adze::leaf(pattern = r"\d+")]
                val: String,
            }
        }
    });
    assert!(find_struct_in_mod(&m, "Alpha").is_some());
    assert!(find_struct_in_mod(&m, "Beta").is_some());
    assert!(find_struct_in_mod(&m, "Gamma").is_some());
    assert!(find_struct_in_mod(&m, "Delta").is_none());
}

// ═══════════════════════════════════════════════════════════════════════════
// 31. Module with use items
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t31_module_with_use_item() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            use adze::Spanned;

            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                tok: String,
            }
        }
    });
    let has_use = module_items(&m).iter().any(|i| matches!(i, Item::Use(_)));
    assert!(has_use);
}

// ═══════════════════════════════════════════════════════════════════════════
// 32. Struct field attr count
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t32_field_with_no_attrs() {
    let s: ItemStruct = parse_quote! {
        pub struct Plain {
            child: OtherType,
        }
    };
    let f = s.fields.iter().next().unwrap();
    assert!(f.attrs.is_empty());
}

#[test]
fn t32_field_with_one_attr() {
    let s: ItemStruct = parse_quote! {
        pub struct OneAttr {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    let f = s.fields.iter().next().unwrap();
    assert_eq!(f.attrs.len(), 1);
}

#[test]
fn t32_field_with_two_attrs() {
    let s: ItemStruct = parse_quote! {
        pub struct TwoAttr {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            #[adze::repeat(non_empty = true)]
            items: Vec<Item>,
        }
    };
    let f = s.fields.iter().next().unwrap();
    assert_eq!(adze_attr_names(&f.attrs).len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 33. Realistic grammar patterns
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn t33_arithmetic_like_grammar_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("arith")]
        mod grammar {
            #[adze::language]
            pub struct Calculation {
                expr: Expr,
            }

            pub struct Expr {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                value: i32,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    let calc = find_struct_in_mod(&m, "Calculation").unwrap();
    assert!(calc.attrs.iter().any(|a| is_adze_attr(a, "language")));
    let expr = find_struct_in_mod(&m, "Expr").unwrap();
    assert_eq!(expr.fields.len(), 1);
}

#[test]
fn t33_json_like_key_value_struct() {
    let s: ItemStruct = parse_quote! {
        pub struct KeyValue {
            #[adze::leaf(pattern = r#""[^"]*""#)]
            key: String,
            #[adze::leaf(text = ":")]
            _colon: (),
            value: Value,
        }
    };
    assert_eq!(s.fields.len(), 3);
    assert_eq!(field_names(&s), vec!["key", "_colon", "value"]);
}

#[test]
fn t33_list_with_brackets_pattern() {
    let s: ItemStruct = parse_quote! {
        pub struct BracketedList {
            #[adze::leaf(text = "[")]
            _open: (),
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<Item>,
            #[adze::leaf(text = "]")]
            _close: (),
        }
    };
    assert_eq!(s.fields.len(), 3);
    let middle = s.fields.iter().nth(1).unwrap();
    assert!(middle.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
}

#[test]
fn t33_function_definition_pattern() {
    let m = parse_mod(quote! {
        #[adze::grammar("fn_lang")]
        mod grammar {
            #[adze::language]
            pub struct FnDef {
                #[adze::leaf(text = "fn")]
                _kw: (),
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
                #[adze::leaf(text = "(")]
                _open: (),
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                params: Vec<Param>,
                #[adze::leaf(text = ")")]
                _close: (),
            }

            pub struct Param {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }

            #[adze::extra]
            struct Ws {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    let fndef = find_struct_in_mod(&m, "FnDef").unwrap();
    assert_eq!(fndef.fields.len(), 5);
    assert!(fndef.attrs.iter().any(|a| is_adze_attr(a, "language")));
}
