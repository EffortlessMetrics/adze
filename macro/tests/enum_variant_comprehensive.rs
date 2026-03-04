#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for enum variant handling in the adze proc-macro crate.
//!
//! Covers enum variant kinds (unit, tuple, struct), attribute interactions on
//! variants (leaf, prec, prec_left, prec_right), data-carrying vs unit variants,
//! nested enum references, variant field type extraction, and edge cases in
//! enum-based grammar definitions.

use std::collections::HashSet;

use adze_common::{FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::{Attribute, Fields, Item, ItemEnum, ItemMod, Token, parse_quote};

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

fn prec_value(attr: &Attribute) -> i32 {
    let expr: syn::Expr = attr.parse_args().unwrap();
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(i),
        ..
    }) = expr
    {
        i.base10_parse::<i32>().unwrap()
    } else {
        panic!("Expected int literal in precedence attribute");
    }
}

fn variant_field_types(variant: &syn::Variant) -> Vec<String> {
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

// ── 1. Enum with multiple unit variants ─────────────────────────────────────

#[test]
fn enum_multiple_unit_variants() {
    let e: ItemEnum = parse_quote! {
        pub enum Keyword {
            #[adze::leaf(text = "if")]
            If,
            #[adze::leaf(text = "else")]
            Else,
            #[adze::leaf(text = "while")]
            While,
            #[adze::leaf(text = "for")]
            For,
            #[adze::leaf(text = "return")]
            Return,
        }
    };
    assert_eq!(e.variants.len(), 5);
    for variant in &e.variants {
        assert!(matches!(variant.fields, Fields::Unit));
        assert!(variant.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ── 2. Enum with multiple tuple variants ────────────────────────────────────

#[test]
fn enum_multiple_tuple_variants() {
    let e: ItemEnum = parse_quote! {
        pub enum Literal {
            Int(#[adze::leaf(pattern = r"\d+")] String),
            Float(#[adze::leaf(pattern = r"\d+\.\d+")] String),
            Str(#[adze::leaf(pattern = r#""[^"]*""#)] String),
        }
    };
    assert_eq!(e.variants.len(), 3);
    for variant in &e.variants {
        assert!(matches!(variant.fields, Fields::Unnamed(_)));
    }
}

// ── 3. Enum mixing unit and tuple variants ──────────────────────────────────

#[test]
fn enum_mixed_unit_and_tuple() {
    let e: ItemEnum = parse_quote! {
        pub enum Token {
            #[adze::leaf(text = "+")]
            Plus,
            Number(#[adze::leaf(pattern = r"\d+")] String),
            #[adze::leaf(text = "-")]
            Minus,
            Ident(#[adze::leaf(pattern = r"[a-z]+")] String),
        }
    };
    assert!(matches!(e.variants[0].fields, Fields::Unit));
    assert!(matches!(e.variants[1].fields, Fields::Unnamed(_)));
    assert!(matches!(e.variants[2].fields, Fields::Unit));
    assert!(matches!(e.variants[3].fields, Fields::Unnamed(_)));
}

// ── 4. Enum with all three variant kinds ────────────────────────────────────

#[test]
fn enum_all_three_variant_kinds() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Expr {
            #[adze::leaf(text = "nil")]
            Nil,
            Number(#[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
            BinOp {
                lhs: Box<Expr>,
                #[adze::leaf(text = "+")]
                _op: (),
                rhs: Box<Expr>,
            },
        }
    };
    assert!(matches!(e.variants[0].fields, Fields::Unit));
    assert!(matches!(e.variants[1].fields, Fields::Unnamed(_)));
    assert!(matches!(e.variants[2].fields, Fields::Named(_)));
}

// ── 5. Unit variant leaf text values ────────────────────────────────────────

#[test]
fn unit_variant_leaf_text_values() {
    let e: ItemEnum = parse_quote! {
        pub enum Operator {
            #[adze::leaf(text = "+")]
            Add,
            #[adze::leaf(text = "-")]
            Sub,
            #[adze::leaf(text = "*")]
            Mul,
            #[adze::leaf(text = "/")]
            Div,
        }
    };
    let expected = ["+", "-", "*", "/"];
    for i in 0..e.variants.len() {
        let attr = e.variants[i]
            .attrs
            .iter()
            .find(|a| is_adze_attr(a, "leaf"))
            .unwrap();
        let params = leaf_params(attr);
        assert_eq!(params[0].path.to_string(), "text");
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &params[0].expr
        {
            assert_eq!(s.value(), expected[i]);
        } else {
            panic!("Expected string literal for variant {i}");
        }
    }
}

// ── 6. Tuple variant with transform closure ─────────────────────────────────

#[test]
fn tuple_variant_with_transform() {
    let e: ItemEnum = parse_quote! {
        pub enum Value {
            Int(#[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<i64>().unwrap())] i64),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        let attr = u.unnamed[0]
            .attrs
            .iter()
            .find(|a| is_adze_attr(a, "leaf"))
            .unwrap();
        let params = leaf_params(attr);
        let names: Vec<_> = params.iter().map(|p| p.path.to_string()).collect();
        assert_eq!(names, vec!["pattern", "transform"]);
        assert!(matches!(params[1].expr, syn::Expr::Closure(_)));
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 7. Named-field variant preserves field idents and types ─────────────────

#[test]
fn named_variant_field_idents_and_types() {
    let e: ItemEnum = parse_quote! {
        pub enum Stmt {
            Assign {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
                #[adze::leaf(text = "=")]
                _eq: (),
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                value: i32,
            },
        }
    };
    if let Fields::Named(ref n) = e.variants[0].fields {
        let idents: Vec<_> = n
            .named
            .iter()
            .map(|f| f.ident.as_ref().unwrap().to_string())
            .collect();
        assert_eq!(idents, vec!["name", "_eq", "value"]);

        let types: Vec<_> = n
            .named
            .iter()
            .map(|f| f.ty.to_token_stream().to_string())
            .collect();
        assert_eq!(types, vec!["String", "()", "i32"]);
    } else {
        panic!("Expected named fields");
    }
}

// ── 8. Variant precedence: prec_left ────────────────────────────────────────

#[test]
fn variant_prec_left_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Num(i32),
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
        }
    };
    let attr = e.variants[1]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_left"))
        .unwrap();
    assert_eq!(prec_value(attr), 1);
}

// ── 9. Variant precedence: prec_right ───────────────────────────────────────

#[test]
fn variant_prec_right_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Num(i32),
            #[adze::prec_right(5)]
            Assign(Box<Expr>, Box<Expr>),
        }
    };
    let attr = e.variants[1]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_right"))
        .unwrap();
    assert_eq!(prec_value(attr), 5);
}

// ── 10. Variant precedence: prec (no associativity) ─────────────────────────

#[test]
fn variant_prec_no_assoc_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Num(i32),
            #[adze::prec(3)]
            Compare(Box<Expr>, Box<Expr>),
        }
    };
    let attr = e.variants[1]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec"))
        .unwrap();
    assert_eq!(prec_value(attr), 3);
}

// ── 11. Multiple precedence levels on different variants ────────────────────

#[test]
fn multiple_precedence_levels_extracted() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Num(i32),
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
            #[adze::prec_left(2)]
            Mul(Box<Expr>, Box<Expr>),
            #[adze::prec_right(3)]
            Pow(Box<Expr>, Box<Expr>),
            #[adze::prec(4)]
            Eq(Box<Expr>, Box<Expr>),
        }
    };
    let prec_info: Vec<(String, String, i32)> = e
        .variants
        .iter()
        .filter_map(|v| {
            for kind in &["prec", "prec_left", "prec_right"] {
                if let Some(attr) = v.attrs.iter().find(|a| is_adze_attr(a, kind)) {
                    return Some((v.ident.to_string(), kind.to_string(), prec_value(attr)));
                }
            }
            None
        })
        .collect();

    assert_eq!(prec_info.len(), 4);
    assert_eq!(prec_info[0], ("Add".into(), "prec_left".into(), 1));
    assert_eq!(prec_info[1], ("Mul".into(), "prec_left".into(), 2));
    assert_eq!(prec_info[2], ("Pow".into(), "prec_right".into(), 3));
    assert_eq!(prec_info[3], ("Eq".into(), "prec".into(), 4));
}

// ── 12. Enum variant referencing another type via Box ────────────────────────

#[test]
fn variant_recursive_box_reference() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            Neg(#[adze::leaf(text = "-")] (), Box<Expr>),
            Group(#[adze::leaf(text = "(")] (), Box<Expr>, #[adze::leaf(text = ")")] ()),
        }
    };
    let neg_types = variant_field_types(&e.variants[1]);
    assert_eq!(neg_types.len(), 2);
    assert_eq!(neg_types[0], "()");
    assert_eq!(neg_types[1], "Box < Expr >");

    let group_types = variant_field_types(&e.variants[2]);
    assert_eq!(group_types.len(), 3);
    assert_eq!(group_types[1], "Box < Expr >");
}

// ── 13. Enum variant referencing another enum type ──────────────────────────

#[test]
fn variant_references_other_enum() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Lit(#[adze::leaf(pattern = r"\d+")] String),
                BinOp(Box<Expr>, Operator, Box<Expr>),
            }

            pub enum Operator {
                #[adze::leaf(text = "+")]
                Plus,
                #[adze::leaf(text = "-")]
                Minus,
            }
        }
    });
    let items = module_items(&m);
    if let Item::Enum(expr) = &items[0] {
        let binop_types = variant_field_types(&expr.variants[1]);
        assert!(binop_types.iter().any(|t| t == "Operator"));
    } else {
        panic!("Expected enum");
    }
    if let Item::Enum(op) = &items[1] {
        assert_eq!(op.ident, "Operator");
        assert_eq!(op.variants.len(), 2);
    } else {
        panic!("Expected second enum");
    }
}

// ── 14. Enum variant referencing a struct type ──────────────────────────────

#[test]
fn variant_references_struct() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Statement {
                VarDecl(VarDecl),
                ExprStmt(ExprNode),
            }

            pub struct VarDecl {
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
            }

            pub struct ExprNode {
                #[adze::leaf(pattern = r"\d+")]
                value: String,
            }
        }
    });
    if let Item::Enum(e) = &module_items(&m)[0] {
        let types0 = variant_field_types(&e.variants[0]);
        let types1 = variant_field_types(&e.variants[1]);
        assert_eq!(types0, vec!["VarDecl"]);
        assert_eq!(types1, vec!["ExprNode"]);
    } else {
        panic!("Expected enum");
    }
}

// ── 15. Enum variant with Vec field ─────────────────────────────────────────

#[test]
fn variant_with_vec_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Numbers(#[adze::repeat(non_empty = true)] Vec<Number>),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        let ty_str = u.unnamed[0].ty.to_token_stream().to_string();
        assert!(ty_str.contains("Vec"));
        assert!(ty_str.contains("Number"));

        let repeat_attr = u.unnamed[0]
            .attrs
            .iter()
            .find(|a| is_adze_attr(a, "repeat"))
            .unwrap();
        let params = leaf_params(repeat_attr);
        assert_eq!(params[0].path.to_string(), "non_empty");
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 16. Enum variant with Option field ──────────────────────────────────────

#[test]
fn variant_with_option_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            MaybeValue(Option<Inner>),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&u.unnamed[0].ty, "Option", &skip);
        assert!(extracted);
        assert_eq!(inner.to_token_stream().to_string(), "Inner");
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 17. Enum as language root in grammar module ─────────────────────────────

#[test]
fn enum_as_language_root() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Root {
                A(#[adze::leaf(text = "a")] ()),
                B(#[adze::leaf(text = "b")] ()),
            }
        }
    });
    if let Item::Enum(e) = &module_items(&m)[0] {
        assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        assert_eq!(e.ident, "Root");
    } else {
        panic!("Expected enum");
    }
}

// ── 18. Enum without language attribute in grammar module ───────────────────

#[test]
fn enum_non_language_in_grammar() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                op: Operator,
            }

            pub enum Operator {
                #[adze::leaf(text = "+")]
                Plus,
                #[adze::leaf(text = "-")]
                Minus,
            }
        }
    });
    if let Item::Enum(e) = &module_items(&m)[1] {
        assert!(!e.attrs.iter().any(|a| is_adze_attr(a, "language")));
        assert_eq!(e.ident, "Operator");
    } else {
        panic!("Expected enum");
    }
}

// ── 19. Enum variant field types through filter_inner_type ──────────────────

#[test]
fn variant_box_field_filter_inner_type() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Neg(Box<Expr>),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let filtered = filter_inner_type(&u.unnamed[0].ty, &skip);
        assert_eq!(filtered.to_token_stream().to_string(), "Expr");
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 20. Enum variant with multiple leaf-annotated fields ────────────────────

#[test]
fn variant_multiple_leaf_fields() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            BinOp(
                Box<Expr>,
                #[adze::leaf(text = "+")]
                (),
                Box<Expr>,
            ),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        assert_eq!(u.unnamed.len(), 3);
        // Only the middle field has a leaf attribute
        let leaf_count = u
            .unnamed
            .iter()
            .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
            .count();
        assert_eq!(leaf_count, 1);
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 21. Enum variant count in large enum ────────────────────────────────────

#[test]
fn large_enum_variant_count() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Expr {
            Number(#[adze::leaf(pattern = r"\d+")] String),
            #[adze::prec_left(1)]
            Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
            #[adze::prec_left(1)]
            Sub(Box<Expr>, #[adze::leaf(text = "-")] (), Box<Expr>),
            #[adze::prec_left(2)]
            Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
            #[adze::prec_left(2)]
            Div(Box<Expr>, #[adze::leaf(text = "/")] (), Box<Expr>),
            #[adze::prec_right(3)]
            Pow(Box<Expr>, #[adze::leaf(text = "^")] (), Box<Expr>),
            Neg(#[adze::leaf(text = "-")] (), Box<Expr>),
            Group(#[adze::leaf(text = "(")] (), Box<Expr>, #[adze::leaf(text = ")")] ()),
        }
    };
    assert_eq!(e.variants.len(), 8);

    let prec_variants: Vec<_> = e
        .variants
        .iter()
        .filter(|v| {
            v.attrs
                .iter()
                .any(|a| is_adze_attr(a, "prec_left") || is_adze_attr(a, "prec_right"))
        })
        .collect();
    assert_eq!(prec_variants.len(), 5);
}

// ── 22. Enum variant names preserved ────────────────────────────────────────

#[test]
fn enum_variant_names_preserved() {
    let e: ItemEnum = parse_quote! {
        pub enum Statement {
            LetBinding(String),
            IfElse(String),
            WhileLoop(String),
            FnDef(String),
        }
    };
    let names: Vec<_> = e.variants.iter().map(|v| v.ident.to_string()).collect();
    assert_eq!(names, vec!["LetBinding", "IfElse", "WhileLoop", "FnDef"]);
}

// ── 23. Enum variant with leaf + prec on same variant ───────────────────────

#[test]
fn variant_leaf_and_prec_combined() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            #[adze::prec_left(1)]
            Add(
                Box<Expr>,
                #[adze::leaf(text = "+")]
                (),
                Box<Expr>,
            ),
        }
    };
    let add = &e.variants[1];
    let attr_names = adze_attr_names(&add.attrs);
    assert_eq!(attr_names, vec!["prec_left"]);

    // The leaf attr is on the field, not the variant
    if let Fields::Unnamed(ref u) = add.fields {
        let field_leaf = u.unnamed[1].attrs.iter().find(|a| is_adze_attr(a, "leaf"));
        assert!(field_leaf.is_some());
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 24. Enum variant with Box<Option<T>> nested type ────────────────────────

#[test]
fn variant_nested_box_option_type() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            MaybeChild(Box<Option<Expr>>),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let filtered = filter_inner_type(&u.unnamed[0].ty, &skip);
        assert_eq!(filtered.to_token_stream().to_string(), "Option < Expr >");
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 25. Enum with single variant ────────────────────────────────────────────

#[test]
fn enum_single_variant() {
    let e: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Wrapper {
            Value(#[adze::leaf(pattern = r"\w+")] String),
        }
    };
    assert_eq!(e.variants.len(), 1);
    assert_eq!(e.variants[0].ident, "Value");
    assert!(matches!(e.variants[0].fields, Fields::Unnamed(_)));
}

// ── 26. Enum with derive attributes alongside adze attributes ───────────────

#[test]
fn enum_derive_and_adze_attrs_coexist() {
    let e: ItemEnum = parse_quote! {
        #[derive(Debug, Clone, PartialEq)]
        #[adze::language]
        pub enum Token {
            #[adze::leaf(text = "x")]
            X,
        }
    };
    let derive_count = e
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("derive"))
        .count();
    let lang_count = e
        .attrs
        .iter()
        .filter(|a| is_adze_attr(a, "language"))
        .count();
    assert_eq!(derive_count, 1);
    assert_eq!(lang_count, 1);
}

// ── 27. Enum visibility preserved ───────────────────────────────────────────

#[test]
fn enum_pub_visibility() {
    let e: ItemEnum = parse_quote! {
        pub enum MyEnum {
            A,
            B,
        }
    };
    assert!(matches!(e.vis, syn::Visibility::Public(_)));
    assert_eq!(e.ident, "MyEnum");
}

// ── 28. Named-field variant with leaf on every field ────────────────────────

#[test]
fn named_variant_all_fields_have_leaf() {
    let e: ItemEnum = parse_quote! {
        pub enum Pair {
            KeyValue {
                #[adze::leaf(pattern = r"[a-z]+")]
                key: String,
                #[adze::leaf(text = ":")]
                _sep: (),
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                value: i32,
            },
        }
    };
    if let Fields::Named(ref n) = e.variants[0].fields {
        let leaf_count = n
            .named
            .iter()
            .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
            .count();
        assert_eq!(leaf_count, 3);
    } else {
        panic!("Expected named fields");
    }
}

// ── 29. Two enums in grammar: language + helper ─────────────────────────────

#[test]
fn two_enums_one_language_one_helper() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Lit(#[adze::leaf(pattern = r"\d+")] String),
                BinOp(Box<Expr>, Op, Box<Expr>),
            }

            pub enum Op {
                #[adze::leaf(text = "+")]
                Plus,
                #[adze::leaf(text = "*")]
                Star,
            }
        }
    });
    let items = module_items(&m);
    let enums: Vec<_> = items
        .iter()
        .filter_map(|i| if let Item::Enum(e) = i { Some(e) } else { None })
        .collect();
    assert_eq!(enums.len(), 2);

    // Only the first is marked language
    assert!(enums[0].attrs.iter().any(|a| is_adze_attr(a, "language")));
    assert!(!enums[1].attrs.iter().any(|a| is_adze_attr(a, "language")));
}

// ── 30. Variant with leaf pattern containing special regex ──────────────────

#[test]
fn variant_leaf_pattern_special_regex() {
    let e: ItemEnum = parse_quote! {
        pub enum Token {
            Float(#[adze::leaf(pattern = r"-?\d+(\.\d+)?([eE][+-]?\d+)?")] String),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        let attr = u.unnamed[0]
            .attrs
            .iter()
            .find(|a| is_adze_attr(a, "leaf"))
            .unwrap();
        let params = leaf_params(attr);
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &params[0].expr
        {
            assert_eq!(s.value(), r"-?\d+(\.\d+)?([eE][+-]?\d+)?");
        } else {
            panic!("Expected string literal");
        }
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 31. Variant field count per variant kind ────────────────────────────────

#[test]
fn variant_field_counts() {
    let e: ItemEnum = parse_quote! {
        pub enum Node {
            #[adze::leaf(text = "nil")]
            Nil,
            Unary(Box<Node>),
            Binary(Box<Node>, #[adze::leaf(text = ",")] (), Box<Node>),
            Triple {
                a: Box<Node>,
                #[adze::leaf(text = ",")]
                _sep1: (),
                b: Box<Node>,
                #[adze::leaf(text = ",")]
                _sep2: (),
                c: Box<Node>,
            },
        }
    };
    assert_eq!(variant_field_types(&e.variants[0]).len(), 0); // Unit
    assert_eq!(variant_field_types(&e.variants[1]).len(), 1); // Unary
    assert_eq!(variant_field_types(&e.variants[2]).len(), 3); // Binary
    assert_eq!(variant_field_types(&e.variants[3]).len(), 5); // Triple
}

// ── 32. Enum variant with delimited field ───────────────────────────────────

#[test]
fn variant_with_delimited_vec() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            List(
                #[adze::leaf(text = "[")]
                (),
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                Vec<Item>,
                #[adze::leaf(text = "]")]
                (),
            ),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        assert_eq!(u.unnamed.len(), 3);
        let delim_attr = u.unnamed[1]
            .attrs
            .iter()
            .find(|a| is_adze_attr(a, "delimited"))
            .unwrap();
        let ftp: FieldThenParams = delim_attr.parse_args().unwrap();
        let inner_leaf = ftp
            .field
            .attrs
            .iter()
            .find(|a| is_adze_attr(a, "leaf"))
            .unwrap();
        let params = leaf_params(inner_leaf);
        assert_eq!(params[0].path.to_string(), "text");
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &params[0].expr
        {
            assert_eq!(s.value(), ",");
        } else {
            panic!("Expected string literal");
        }
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 33. Same prec level on multiple variants ────────────────────────────────

#[test]
fn same_prec_level_multiple_variants() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
            #[adze::prec_left(1)]
            Sub(Box<Expr>, Box<Expr>),
        }
    };
    let add_attr = e.variants[1]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_left"))
        .unwrap();
    let sub_attr = e.variants[2]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_left"))
        .unwrap();
    assert_eq!(prec_value(add_attr), prec_value(sub_attr));
    assert_eq!(prec_value(add_attr), 1);
}

// ── 34. Variant with no adze attributes ─────────────────────────────────────

#[test]
fn variant_no_adze_attributes() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            Ref(Box<Expr>),
        }
    };
    for variant in &e.variants {
        let adze_attrs = adze_attr_names(&variant.attrs);
        assert!(adze_attrs.is_empty());
    }
    // But the fields themselves also have no adze attrs here
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        assert!(u.unnamed[0].attrs.is_empty());
    }
}

// ── 35. Enum variant with leaf text multi-char keyword ──────────────────────

#[test]
fn variant_leaf_multichar_keywords() {
    let e: ItemEnum = parse_quote! {
        pub enum Keyword {
            #[adze::leaf(text = "function")]
            Function,
            #[adze::leaf(text = "return")]
            Return,
            #[adze::leaf(text = "const")]
            Const,
            #[adze::leaf(text = "let")]
            Let,
        }
    };
    let keywords: Vec<String> = e
        .variants
        .iter()
        .map(|v| {
            let attr = v.attrs.iter().find(|a| is_adze_attr(a, "leaf")).unwrap();
            let params = leaf_params(attr);
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = &params[0].expr
            {
                s.value()
            } else {
                panic!("Expected string literal");
            }
        })
        .collect();
    assert_eq!(keywords, vec!["function", "return", "const", "let"]);
}

// ── 36. Enum private (inherited) visibility ─────────────────────────────────

#[test]
fn enum_private_visibility() {
    let e: ItemEnum = parse_quote! {
        enum InternalExpr {
            A,
            B,
        }
    };
    assert!(matches!(e.vis, syn::Visibility::Inherited));
}

// ── 37. Enum pub(crate) visibility ──────────────────────────────────────────

#[test]
fn enum_pub_crate_visibility() {
    let e: ItemEnum = parse_quote! {
        pub(crate) enum CrateExpr {
            A,
            B,
        }
    };
    assert!(matches!(e.vis, syn::Visibility::Restricted(_)));
}

// ── 38. Enum pub(super) visibility ──────────────────────────────────────────

#[test]
fn enum_pub_super_visibility() {
    let e: ItemEnum = parse_quote! {
        pub(super) enum ParentExpr {
            A,
            B,
        }
    };
    assert!(matches!(e.vis, syn::Visibility::Restricted(_)));
}

// ── 39. Doc comments on enum preserved ──────────────────────────────────────

#[test]
fn enum_doc_comments_preserved() {
    let e: ItemEnum = parse_quote! {
        /// Top-level expression
        #[adze::language]
        pub enum Expr {
            Lit(i32),
        }
    };
    let doc_count = e.attrs.iter().filter(|a| a.path().is_ident("doc")).count();
    assert_eq!(doc_count, 1);
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

// ── 40. Doc comments on variants preserved ──────────────────────────────────

#[test]
fn variant_doc_comments_preserved() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            /// A number literal
            Number(#[adze::leaf(pattern = r"\d+")] String),
            /// An addition expression
            #[adze::prec_left(1)]
            Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
        }
    };
    for variant in &e.variants {
        let doc_attrs: Vec<_> = variant
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("doc"))
            .collect();
        assert_eq!(
            doc_attrs.len(),
            1,
            "Variant {} should have 1 doc comment",
            variant.ident
        );
    }
}

// ── 41. Multiple doc comments on same variant ───────────────────────────────

#[test]
fn variant_multiple_doc_comments() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            /// First line
            /// Second line
            #[adze::leaf(text = "nil")]
            Nil,
        }
    };
    let doc_count = e.variants[0]
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .count();
    assert_eq!(doc_count, 2);
}

// ── 42. Enum with external attribute on variant struct context ───────────────

#[test]
fn enum_with_external_struct_in_grammar() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Lit(#[adze::leaf(pattern = r"\d+")] String),
            }

            #[adze::external]
            struct IndentToken;
        }
    });
    let items = module_items(&m);
    let external_struct = items.iter().find_map(|i| {
        if let Item::Struct(s) = i {
            if s.attrs.iter().any(|a| is_adze_attr(a, "external")) {
                Some(s)
            } else {
                None
            }
        } else {
            None
        }
    });
    assert!(external_struct.is_some());
    assert_eq!(external_struct.unwrap().ident, "IndentToken");
}

// ── 43. Enum with skip field inside named variant ───────────────────────────

#[test]
fn named_variant_with_skip_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Node {
            Data {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
                #[adze::skip(0)]
                count: i32,
            },
        }
    };
    if let Fields::Named(ref n) = e.variants[0].fields {
        let skip_field = n
            .named
            .iter()
            .find(|f| f.attrs.iter().any(|a| is_adze_attr(a, "skip")))
            .unwrap();
        assert_eq!(skip_field.ident.as_ref().unwrap(), "count");
    } else {
        panic!("Expected named fields");
    }
}

// ── 44. Enum with skip default expression ───────────────────────────────────

#[test]
fn named_variant_skip_default_expr() {
    let e: ItemEnum = parse_quote! {
        pub enum Node {
            Info {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
                #[adze::skip(Vec::new())]
                children: Vec<String>,
            },
        }
    };
    if let Fields::Named(ref n) = e.variants[0].fields {
        let skip_field = n
            .named
            .iter()
            .find(|f| f.attrs.iter().any(|a| is_adze_attr(a, "skip")))
            .unwrap();
        let attr = skip_field
            .attrs
            .iter()
            .find(|a| is_adze_attr(a, "skip"))
            .unwrap();
        let expr: syn::Expr = attr.parse_args().unwrap();
        let expr_str = expr.to_token_stream().to_string();
        assert!(expr_str.contains("Vec"));
    } else {
        panic!("Expected named fields");
    }
}

// ── 45. Enum with derive Debug ──────────────────────────────────────────────

#[test]
fn enum_derive_debug() {
    let e: ItemEnum = parse_quote! {
        #[derive(Debug)]
        pub enum Token {
            #[adze::leaf(text = "x")]
            X,
        }
    };
    let derive = e
        .attrs
        .iter()
        .find(|a| a.path().is_ident("derive"))
        .unwrap();
    let tokens = derive.to_token_stream().to_string();
    assert!(tokens.contains("Debug"));
}

// ── 46. Enum with derive Clone ──────────────────────────────────────────────

#[test]
fn enum_derive_clone() {
    let e: ItemEnum = parse_quote! {
        #[derive(Clone)]
        pub enum Token {
            #[adze::leaf(text = "y")]
            Y,
        }
    };
    let derive = e
        .attrs
        .iter()
        .find(|a| a.path().is_ident("derive"))
        .unwrap();
    let tokens = derive.to_token_stream().to_string();
    assert!(tokens.contains("Clone"));
}

// ── 47. Enum with multiple derives in one attribute ─────────────────────────

#[test]
fn enum_multiple_derives_single_attr() {
    let e: ItemEnum = parse_quote! {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[adze::language]
        pub enum Expr {
            Lit(i32),
        }
    };
    let derive = e
        .attrs
        .iter()
        .find(|a| a.path().is_ident("derive"))
        .unwrap();
    let tokens = derive.to_token_stream().to_string();
    assert!(tokens.contains("Debug"));
    assert!(tokens.contains("Clone"));
    assert!(tokens.contains("PartialEq"));
    assert!(tokens.contains("Eq"));
    assert!(tokens.contains("Hash"));
}

// ── 48. Enum with separate derive attributes ────────────────────────────────

#[test]
fn enum_separate_derive_attrs() {
    let e: ItemEnum = parse_quote! {
        #[derive(Debug)]
        #[derive(Clone)]
        #[derive(PartialEq)]
        pub enum Token {
            A,
        }
    };
    let derive_count = e
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("derive"))
        .count();
    assert_eq!(derive_count, 3);
}

// ── 49. Variant with prec_left and leaf in named struct fields ──────────────

#[test]
fn variant_prec_left_with_named_fields() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            #[adze::prec_left(2)]
            BinOp {
                lhs: Box<Expr>,
                #[adze::leaf(text = "+")]
                _op: (),
                rhs: Box<Expr>,
            },
        }
    };
    let binop = &e.variants[1];
    assert!(binop.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
    assert!(matches!(binop.fields, Fields::Named(_)));
    if let Fields::Named(ref n) = binop.fields {
        assert_eq!(n.named.len(), 3);
        let op_field = &n.named[1];
        assert!(op_field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ── 50. Variant with prec_right and named fields ────────────────────────────

#[test]
fn variant_prec_right_with_named_fields() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            #[adze::prec_right(1)]
            Assign {
                target: Box<Expr>,
                #[adze::leaf(text = "=")]
                _eq: (),
                value: Box<Expr>,
            },
        }
    };
    let assign = &e.variants[1];
    assert!(assign.attrs.iter().any(|a| is_adze_attr(a, "prec_right")));
    let attr = assign
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_right"))
        .unwrap();
    assert_eq!(prec_value(attr), 1);
}

// ── 51. Variant with zero precedence level ──────────────────────────────────

#[test]
fn variant_prec_zero_level() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            #[adze::prec_left(0)]
            Low(Box<Expr>, Box<Expr>),
        }
    };
    let attr = e.variants[1]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_left"))
        .unwrap();
    assert_eq!(prec_value(attr), 0);
}

// ── 52. Variant with high precedence level ──────────────────────────────────

#[test]
fn variant_prec_high_level() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            #[adze::prec_left(100)]
            VeryHigh(Box<Expr>, Box<Expr>),
        }
    };
    let attr = e.variants[1]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_left"))
        .unwrap();
    assert_eq!(prec_value(attr), 100);
}

// ── 53. Enum ident preserved ────────────────────────────────────────────────

#[test]
fn enum_ident_preserved() {
    let e: ItemEnum = parse_quote! {
        pub enum MyCustomExpression {
            A,
        }
    };
    assert_eq!(e.ident, "MyCustomExpression");
}

// ── 54. Empty variant list parses (edge case) ───────────────────────────────

#[test]
fn enum_empty_variants() {
    let e: ItemEnum = parse_quote! {
        pub enum Empty {}
    };
    assert_eq!(e.variants.len(), 0);
}

// ── 55. Variant with tuple field containing Vec ─────────────────────────────

#[test]
fn variant_tuple_with_vec_and_repeat() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Args(
                #[adze::leaf(text = "(")]
                (),
                #[adze::repeat(non_empty = true)]
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                Vec<Arg>,
                #[adze::leaf(text = ")")]
                (),
            ),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        assert_eq!(u.unnamed.len(), 3);
        let mid = &u.unnamed[1];
        assert!(mid.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
        assert!(mid.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 56. Variant leaf text empty string ──────────────────────────────────────

#[test]
fn variant_leaf_text_empty_string() {
    let e: ItemEnum = parse_quote! {
        pub enum Token {
            #[adze::leaf(text = "")]
            Empty,
        }
    };
    let attr = e.variants[0]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert_eq!(s.value(), "");
    } else {
        panic!("Expected string literal");
    }
}

// ── 57. Variant leaf text with special characters ───────────────────────────

#[test]
fn variant_leaf_text_special_chars() {
    let e: ItemEnum = parse_quote! {
        pub enum Op {
            #[adze::leaf(text = "&&")]
            And,
            #[adze::leaf(text = "||")]
            Or,
            #[adze::leaf(text = "!=")]
            NotEq,
            #[adze::leaf(text = ">=")]
            GtEq,
            #[adze::leaf(text = "<=")]
            LtEq,
            #[adze::leaf(text = "==")]
            EqEq,
        }
    };
    let expected_texts = ["&&", "||", "!=", ">=", "<=", "=="];
    for (i, variant) in e.variants.iter().enumerate() {
        let attr = variant
            .attrs
            .iter()
            .find(|a| is_adze_attr(a, "leaf"))
            .unwrap();
        let params = leaf_params(attr);
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &params[0].expr
        {
            assert_eq!(s.value(), expected_texts[i]);
        }
    }
}

// ── 58. Variant leaf pattern with unicode character class ───────────────────

#[test]
fn variant_leaf_pattern_unicode() {
    let e: ItemEnum = parse_quote! {
        pub enum Token {
            Ident(#[adze::leaf(pattern = r"[\p{L}_][\p{L}\p{N}_]*")] String),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        let attr = u.unnamed[0]
            .attrs
            .iter()
            .find(|a| is_adze_attr(a, "leaf"))
            .unwrap();
        let params = leaf_params(attr);
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &params[0].expr
        {
            assert!(s.value().contains(r"\p{L}"));
        }
    }
}

// ── 59. Variant with only Box<Self> pattern ─────────────────────────────────

#[test]
fn variant_box_self_reference() {
    let e: ItemEnum = parse_quote! {
        pub enum Tree {
            Leaf(#[adze::leaf(pattern = r"\d+")] String),
            Node(Box<Tree>, Box<Tree>),
        }
    };
    let node_types = variant_field_types(&e.variants[1]);
    assert_eq!(node_types, vec!["Box < Tree >", "Box < Tree >"]);
}

// ── 60. Enum variant with many tuple fields ─────────────────────────────────

#[test]
fn variant_many_tuple_fields() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Ternary(
                Box<Expr>,
                #[adze::leaf(text = "?")]
                (),
                Box<Expr>,
                #[adze::leaf(text = ":")]
                (),
                Box<Expr>,
            ),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        assert_eq!(u.unnamed.len(), 5);
        let leaf_indices: Vec<usize> = u
            .unnamed
            .iter()
            .enumerate()
            .filter(|(_, f)| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
            .map(|(i, _)| i)
            .collect();
        assert_eq!(leaf_indices, vec![1, 3]);
    }
}

// ── 61. Named variant with underscore-prefixed punct fields ─────────────────

#[test]
fn named_variant_underscore_punct_fields() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Grouped {
                #[adze::leaf(text = "(")]
                _open: (),
                inner: Box<Expr>,
                #[adze::leaf(text = ")")]
                _close: (),
            },
        }
    };
    if let Fields::Named(ref n) = e.variants[0].fields {
        let names: Vec<_> = n
            .named
            .iter()
            .map(|f| f.ident.as_ref().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["_open", "inner", "_close"]);
        let unit_fields: Vec<_> = n
            .named
            .iter()
            .filter(|f| f.ty.to_token_stream().to_string() == "()")
            .map(|f| f.ident.as_ref().unwrap().to_string())
            .collect();
        assert_eq!(unit_fields, vec!["_open", "_close"]);
    }
}

// ── 62. Enum with cfg attribute alongside adze ──────────────────────────────

#[test]
fn enum_cfg_attr_alongside_adze() {
    let e: ItemEnum = parse_quote! {
        #[cfg(feature = "full")]
        #[adze::language]
        pub enum Expr {
            Lit(i32),
        }
    };
    assert_eq!(e.attrs.len(), 2);
    let adze_names = adze_attr_names(&e.attrs);
    assert_eq!(adze_names, vec!["language"]);
}

// ── 63. Variant with cfg attribute ──────────────────────────────────────────

#[test]
fn variant_with_cfg_attribute() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[cfg(feature = "full")]
            Extended(Box<Expr>),
            Basic(#[adze::leaf(pattern = r"\d+")] String),
        }
    };
    let extended = &e.variants[0];
    assert!(extended.attrs.iter().any(|a| a.path().is_ident("cfg")));
    assert_eq!(e.variants.len(), 2);
}

// ── 64. Variant with doc + prec + leaf on fields ────────────────────────────

#[test]
fn variant_doc_prec_leaf_combined() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            /// Addition
            #[adze::prec_left(1)]
            Add(
                Box<Expr>,
                #[adze::leaf(text = "+")]
                (),
                Box<Expr>,
            ),
        }
    };
    let add = &e.variants[1];
    let doc_count = add
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .count();
    assert_eq!(doc_count, 1);
    assert!(add.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
    if let Fields::Unnamed(ref u) = add.fields {
        assert!(u.unnamed[1].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ── 65. Enum variant referencing Spanned<T> ─────────────────────────────────

#[test]
fn variant_with_spanned_wrapper() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(Spanned<Number>),
        }
    };
    let types = variant_field_types(&e.variants[0]);
    assert_eq!(types, vec!["Spanned < Number >"]);
}

// ── 66. Variant with multiple Box<T> fields ─────────────────────────────────

#[test]
fn variant_multiple_box_fields() {
    let e: ItemEnum = parse_quote! {
        pub enum Stmt {
            IfElse(
                #[adze::leaf(text = "if")]
                (),
                Box<Expr>,
                #[adze::leaf(text = "then")]
                (),
                Box<Stmt>,
                #[adze::leaf(text = "else")]
                (),
                Box<Stmt>,
            ),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        assert_eq!(u.unnamed.len(), 6);
        let box_count = u
            .unnamed
            .iter()
            .filter(|f| f.ty.to_token_stream().to_string().starts_with("Box"))
            .count();
        assert_eq!(box_count, 3);
    }
}

// ── 67. Enum with all prec kinds on different variants ──────────────────────

#[test]
fn enum_all_prec_kinds() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            #[adze::prec(1)]
            NoAssoc(Box<Expr>, Box<Expr>),
            #[adze::prec_left(2)]
            LeftAssoc(Box<Expr>, Box<Expr>),
            #[adze::prec_right(3)]
            RightAssoc(Box<Expr>, Box<Expr>),
        }
    };
    let prec_kinds: Vec<(String, String)> = e
        .variants
        .iter()
        .filter_map(|v| {
            for kind in &["prec", "prec_left", "prec_right"] {
                if v.attrs.iter().any(|a| is_adze_attr(a, kind)) {
                    return Some((v.ident.to_string(), kind.to_string()));
                }
            }
            None
        })
        .collect();
    assert_eq!(prec_kinds.len(), 3);
    assert_eq!(prec_kinds[0], ("NoAssoc".into(), "prec".into()));
    assert_eq!(prec_kinds[1], ("LeftAssoc".into(), "prec_left".into()));
    assert_eq!(prec_kinds[2], ("RightAssoc".into(), "prec_right".into()));
}

// ── 68. Variant with try_extract_inner_type on Vec ──────────────────────────

#[test]
fn variant_extract_vec_inner_type() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            List(Vec<Item>),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&u.unnamed[0].ty, "Vec", &skip);
        assert!(extracted);
        assert_eq!(inner.to_token_stream().to_string(), "Item");
    }
}

// ── 69. Variant with try_extract_inner_type on Box<Vec<T>> ──────────────────

#[test]
fn variant_extract_vec_through_box() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Items(Box<Vec<Item>>),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let (inner, extracted) = try_extract_inner_type(&u.unnamed[0].ty, "Vec", &skip);
        assert!(extracted);
        assert_eq!(inner.to_token_stream().to_string(), "Item");
    }
}

// ── 70. Enum with generic type parameter ────────────────────────────────────

#[test]
fn enum_with_generic_type_param() {
    let e: ItemEnum = parse_quote! {
        pub enum Tree<T> {
            Leaf(T),
            Node(Box<Tree<T>>, Box<Tree<T>>),
        }
    };
    assert_eq!(e.generics.params.len(), 1);
    assert_eq!(e.variants.len(), 2);
    let node_types = variant_field_types(&e.variants[1]);
    assert_eq!(node_types, vec!["Box < Tree < T > >", "Box < Tree < T > >"]);
}

// ── 71. Enum with multiple generic type parameters ──────────────────────────

#[test]
fn enum_with_multiple_generic_params() {
    let e: ItemEnum = parse_quote! {
        pub enum Either<L, R> {
            Left(L),
            Right(R),
        }
    };
    assert_eq!(e.generics.params.len(), 2);
    let left_types = variant_field_types(&e.variants[0]);
    let right_types = variant_field_types(&e.variants[1]);
    assert_eq!(left_types, vec!["L"]);
    assert_eq!(right_types, vec!["R"]);
}

// ── 72. Enum with lifetime parameter ────────────────────────────────────────

#[test]
fn enum_with_lifetime_param() {
    let e: ItemEnum = parse_quote! {
        pub enum Ref<'a> {
            Borrowed(&'a str),
            Owned(String),
        }
    };
    assert_eq!(e.generics.params.len(), 1);
    assert_eq!(e.variants.len(), 2);
}

// ── 73. Enum with where clause ──────────────────────────────────────────────

#[test]
fn enum_with_where_clause() {
    let e: ItemEnum = parse_quote! {
        pub enum Container<T> where T: Clone {
            Single(T),
            Multiple(Vec<T>),
        }
    };
    assert!(e.generics.where_clause.is_some());
    assert_eq!(e.variants.len(), 2);
}

// ── 74. Variant with leaf pattern on String type ────────────────────────────

#[test]
fn variant_leaf_pattern_string_type() {
    let e: ItemEnum = parse_quote! {
        pub enum Token {
            Ident(#[adze::leaf(pattern = r"[a-zA-Z_]\w*")] String),
            Number(#[adze::leaf(pattern = r"\d+")] String),
        }
    };
    for variant in &e.variants {
        if let Fields::Unnamed(ref u) = variant.fields {
            assert_eq!(u.unnamed[0].ty.to_token_stream().to_string(), "String");
            assert!(u.unnamed[0].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
        }
    }
}

// ── 75. Variant field type is () for punctuation ────────────────────────────

#[test]
fn variant_unit_typed_punctuation_fields() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Group(
                #[adze::leaf(text = "(")] (),
                Box<Expr>,
                #[adze::leaf(text = ")")] (),
            ),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        let unit_count = u
            .unnamed
            .iter()
            .filter(|f| f.ty.to_token_stream().to_string() == "()")
            .count();
        assert_eq!(unit_count, 2);
    }
}

// ── 76. Enum in grammar with use statement ──────────────────────────────────

#[test]
fn enum_grammar_with_use_statement() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            use adze::Spanned;

            #[adze::language]
            pub enum Expr {
                Lit(Spanned<Number>),
            }

            pub struct Number {
                #[adze::leaf(pattern = r"\d+")]
                value: String,
            }
        }
    });
    let items = module_items(&m);
    let has_use = items.iter().any(|i| matches!(i, Item::Use(_)));
    assert!(has_use);
}

// ── 77. Variant with leaf text single char ──────────────────────────────────

#[test]
fn variant_leaf_text_single_char() {
    let chars = [
        "+", "-", "*", "/", "(", ")", "{", "}", "[", "]", ";", ",", ".",
    ];
    for c in &chars {
        let text = *c;
        let tokens = quote! {
            pub enum T {
                #[adze::leaf(text = #text)]
                V,
            }
        };
        let e: ItemEnum = syn::parse2(tokens).unwrap();
        let attr = e.variants[0]
            .attrs
            .iter()
            .find(|a| is_adze_attr(a, "leaf"))
            .unwrap();
        let params = leaf_params(attr);
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &params[0].expr
        {
            assert_eq!(s.value(), *c);
        }
    }
}

// ── 78. Variant with filter_inner_type stripping Box and Option ─────────────

#[test]
fn variant_filter_inner_strips_multiple_wrappers() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            A(Box<Inner>),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        let skip: HashSet<&str> = ["Box", "Option"].into_iter().collect();
        let filtered = filter_inner_type(&u.unnamed[0].ty, &skip);
        assert_eq!(filtered.to_token_stream().to_string(), "Inner");
    }
}

// ── 79. Large enum variant names are unique ─────────────────────────────────

#[test]
fn enum_variant_names_unique() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(i32),
            String(String),
            Bool(bool),
            Nil,
            Add(Box<Expr>, Box<Expr>),
            Sub(Box<Expr>, Box<Expr>),
            Mul(Box<Expr>, Box<Expr>),
            Div(Box<Expr>, Box<Expr>),
            Neg(Box<Expr>),
            Group(Box<Expr>),
        }
    };
    let names: Vec<String> = e.variants.iter().map(|v| v.ident.to_string()).collect();
    let unique: HashSet<String> = names.iter().cloned().collect();
    assert_eq!(names.len(), unique.len());
    assert_eq!(names.len(), 10);
}

// ── 80. Enum prec values are monotonically increasing ───────────────────────

#[test]
fn enum_prec_values_increasing() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
            #[adze::prec_left(2)]
            Mul(Box<Expr>, Box<Expr>),
            #[adze::prec_right(3)]
            Pow(Box<Expr>, Box<Expr>),
        }
    };
    let prec_values: Vec<i32> = e
        .variants
        .iter()
        .filter_map(|v| {
            for kind in &["prec", "prec_left", "prec_right"] {
                if let Some(attr) = v.attrs.iter().find(|a| is_adze_attr(a, kind)) {
                    return Some(prec_value(attr));
                }
            }
            None
        })
        .collect();
    assert_eq!(prec_values, vec![1, 2, 3]);
    for i in 1..prec_values.len() {
        assert!(prec_values[i] > prec_values[i - 1]);
    }
}

// ── 81. Variant with Option<Box<T>> nested type ─────────────────────────────

#[test]
fn variant_option_box_nested_type() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            MaybeChild(Option<Box<Expr>>),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&u.unnamed[0].ty, "Option", &skip);
        assert!(extracted);
        assert_eq!(inner.to_token_stream().to_string(), "Box < Expr >");
    }
}

// ── 82. Enum with only unit variants all having leaf ────────────────────────

#[test]
fn enum_all_unit_variants_with_leaf() {
    let e: ItemEnum = parse_quote! {
        pub enum BoolLit {
            #[adze::leaf(text = "true")]
            True,
            #[adze::leaf(text = "false")]
            False,
        }
    };
    assert_eq!(e.variants.len(), 2);
    for variant in &e.variants {
        assert!(matches!(variant.fields, Fields::Unit));
        assert!(variant.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

// ── 83. Enum variant attribute ordering preserved ───────────────────────────

#[test]
fn variant_attribute_ordering_preserved() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            /// docs first
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
        }
    };
    let attrs = &e.variants[0].attrs;
    assert!(attrs[0].path().is_ident("doc"));
    assert!(is_adze_attr(&attrs[1], "prec_left"));
}

// ── 84. Variant with no fields and no attributes ────────────────────────────

#[test]
fn variant_bare_unit_no_attrs() {
    let e: ItemEnum = parse_quote! {
        pub enum Direction {
            North,
            South,
            East,
            West,
        }
    };
    for variant in &e.variants {
        assert!(matches!(variant.fields, Fields::Unit));
        assert!(variant.attrs.is_empty());
    }
}

// ── 85. Enum with mixed visibility variants in grammar ──────────────────────

#[test]
fn enum_in_grammar_module_with_struct_helper() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Lit(Number),
                Neg(#[adze::leaf(text = "-")] (), Box<Expr>),
            }

            pub struct Number {
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
    let items = module_items(&m);
    let enum_count = items.iter().filter(|i| matches!(i, Item::Enum(_))).count();
    let struct_count = items
        .iter()
        .filter(|i| matches!(i, Item::Struct(_)))
        .count();
    assert_eq!(enum_count, 1);
    assert_eq!(struct_count, 2);
}

// ── 86. Variant leaf pattern with anchors ───────────────────────────────────

#[test]
fn variant_leaf_pattern_with_anchors() {
    let e: ItemEnum = parse_quote! {
        pub enum Token {
            Line(#[adze::leaf(pattern = r"^[^\n]*$")] String),
        }
    };
    if let Fields::Unnamed(ref u) = e.variants[0].fields {
        let attr = u.unnamed[0]
            .attrs
            .iter()
            .find(|a| is_adze_attr(a, "leaf"))
            .unwrap();
        let params = leaf_params(attr);
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &params[0].expr
        {
            assert_eq!(s.value(), r"^[^\n]*$");
        }
    }
}

// ── 87. Named variant field with Vec and repeat ─────────────────────────────

#[test]
fn named_variant_vec_with_repeat() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Block {
                #[adze::leaf(text = "{")]
                _open: (),
                #[adze::repeat(non_empty = true)]
                stmts: Vec<Stmt>,
                #[adze::leaf(text = "}")]
                _close: (),
            },
        }
    };
    if let Fields::Named(ref n) = e.variants[0].fields {
        let stmts_field = n
            .named
            .iter()
            .find(|f| f.ident.as_ref().unwrap() == "stmts")
            .unwrap();
        assert!(stmts_field.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
        assert_eq!(stmts_field.ty.to_token_stream().to_string(), "Vec < Stmt >");
    }
}

// ── 88. Enum variant roundtrip through quote ────────────────────────────────

#[test]
fn variant_roundtrip_through_quote() {
    let original: ItemEnum = parse_quote! {
        #[adze::language]
        pub enum Expr {
            #[adze::leaf(text = "nil")]
            Nil,
            #[adze::prec_left(1)]
            Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
        }
    };
    let tokens = original.to_token_stream();
    let reparsed: ItemEnum = syn::parse2(tokens).unwrap();
    assert_eq!(original.variants.len(), reparsed.variants.len());
    assert_eq!(original.ident, reparsed.ident);
    for (o, r) in original.variants.iter().zip(reparsed.variants.iter()) {
        assert_eq!(o.ident, r.ident);
    }
}

// ── 89. Enum variant discriminant is None ───────────────────────────────────

#[test]
fn variant_no_discriminant() {
    let e: ItemEnum = parse_quote! {
        pub enum Token {
            #[adze::leaf(text = "a")]
            A,
            #[adze::leaf(text = "b")]
            B,
        }
    };
    for variant in &e.variants {
        assert!(variant.discriminant.is_none());
    }
}

// ── 90. Enum with three enums in grammar ────────────────────────────────────

#[test]
fn three_enums_in_grammar() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Lit(Literal),
                BinOp(Box<Expr>, Op, Box<Expr>),
                UnaryOp(UnaryOp, Box<Expr>),
            }

            pub enum Literal {
                #[adze::leaf(text = "true")]
                True,
                #[adze::leaf(text = "false")]
                False,
            }

            pub enum Op {
                #[adze::leaf(text = "+")]
                Plus,
                #[adze::leaf(text = "-")]
                Minus,
            }

            pub enum UnaryOp {
                #[adze::leaf(text = "!")]
                Not,
                #[adze::leaf(text = "-")]
                Neg,
            }
        }
    });
    let items = module_items(&m);
    let enums: Vec<_> = items
        .iter()
        .filter_map(|i| if let Item::Enum(e) = i { Some(e) } else { None })
        .collect();
    assert_eq!(enums.len(), 4);
    let names: Vec<_> = enums.iter().map(|e| e.ident.to_string()).collect();
    assert!(names.contains(&"Expr".to_string()));
    assert!(names.contains(&"Literal".to_string()));
    assert!(names.contains(&"Op".to_string()));
    assert!(names.contains(&"UnaryOp".to_string()));
}
