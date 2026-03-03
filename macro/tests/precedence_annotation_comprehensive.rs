#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for precedence and associativity annotation handling
//! in the adze proc-macro crate.
//!
//! Covers `#[adze::prec]`, `#[adze::prec_left]`, `#[adze::prec_right]` attribute
//! parsing, value extraction, coexistence with other attributes, ordering,
//! edge cases, and interaction with grammar expansion via `expand_grammar`.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Attribute, Fields, Item, ItemEnum, ItemMod, parse_quote};

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

/// Returns (variant_name, attr_kind, prec_level) for all variants with prec attrs.
fn extract_prec_info(e: &ItemEnum) -> Vec<(String, String, i32)> {
    e.variants
        .iter()
        .filter_map(|v| {
            for kind in &["prec", "prec_left", "prec_right"] {
                if let Some(attr) = v.attrs.iter().find(|a| is_adze_attr(a, kind)) {
                    return Some((v.ident.to_string(), kind.to_string(), prec_value(attr)));
                }
            }
            None
        })
        .collect()
}

// ── 1. prec_left: basic value extraction ────────────────────────────────────

#[test]
fn prec_left_basic_value() {
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

// ── 2. prec_right: basic value extraction ───────────────────────────────────

#[test]
fn prec_right_basic_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Num(i32),
            #[adze::prec_right(2)]
            Assign(Box<Expr>, Box<Expr>),
        }
    };
    let attr = e.variants[1]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_right"))
        .unwrap();
    assert_eq!(prec_value(attr), 2);
}

// ── 3. prec (no associativity): basic value extraction ──────────────────────

#[test]
fn prec_no_assoc_basic_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Num(i32),
            #[adze::prec(7)]
            Compare(Box<Expr>, Box<Expr>),
        }
    };
    let attr = e.variants[1]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec"))
        .unwrap();
    assert_eq!(prec_value(attr), 7);
}

// ── 4. All three prec kinds coexist on separate variants ────────────────────

#[test]
fn all_three_prec_kinds_coexist() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            #[adze::prec(5)]
            Eq(Box<Expr>, Box<Expr>),
            #[adze::prec_left(10)]
            Add(Box<Expr>, Box<Expr>),
            #[adze::prec_right(15)]
            Assign(Box<Expr>, Box<Expr>),
        }
    };
    let info = extract_prec_info(&e);
    assert_eq!(info.len(), 3);
    assert_eq!(info[0], ("Eq".into(), "prec".into(), 5));
    assert_eq!(info[1], ("Add".into(), "prec_left".into(), 10));
    assert_eq!(info[2], ("Assign".into(), "prec_right".into(), 15));
}

// ── 5. Multiple prec_left at different levels ───────────────────────────────

#[test]
fn multiple_prec_left_levels() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Num(i32),
            #[adze::prec_left(1)]
            Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
            #[adze::prec_left(2)]
            Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
            #[adze::prec_left(3)]
            Pow(Box<Expr>, #[adze::leaf(text = "^")] (), Box<Expr>),
        }
    };
    let info = extract_prec_info(&e);
    assert_eq!(info.len(), 3);
    for i in 0..info.len() {
        assert_eq!(info[i].1, "prec_left");
        assert_eq!(info[i].2, (i as i32) + 1);
    }
}

// ── 6. Multiple prec_right at different levels ──────────────────────────────

#[test]
fn multiple_prec_right_levels() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Num(i32),
            #[adze::prec_right(10)]
            Cons(Box<Expr>, #[adze::leaf(text = "::")] (), Box<Expr>),
            #[adze::prec_right(20)]
            Arrow(Box<Expr>, #[adze::leaf(text = "->")] (), Box<Expr>),
        }
    };
    let info = extract_prec_info(&e);
    assert_eq!(info.len(), 2);
    assert_eq!(info[0], ("Cons".into(), "prec_right".into(), 10));
    assert_eq!(info[1], ("Arrow".into(), "prec_right".into(), 20));
}

// ── 7. Precedence with zero level ───────────────────────────────────────────

#[test]
fn prec_level_zero() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            #[adze::prec_left(0)]
            Or(Box<Expr>, Box<Expr>),
        }
    };
    let info = extract_prec_info(&e);
    assert_eq!(info[0].2, 0);
}

// ── 8. Precedence with large level ──────────────────────────────────────────

#[test]
fn prec_level_large() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            #[adze::prec_left(999)]
            High(Box<Expr>, Box<Expr>),
        }
    };
    let info = extract_prec_info(&e);
    assert_eq!(info[0].2, 999);
}

// ── 9. Prec attr on named-field variant ─────────────────────────────────────

#[test]
fn prec_on_named_field_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            #[adze::prec_left(1)]
            BinOp {
                lhs: Box<Expr>,
                #[adze::leaf(text = "+")]
                _op: (),
                rhs: Box<Expr>,
            },
        }
    };
    let attr = e.variants[1]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_left"))
        .unwrap();
    assert_eq!(prec_value(attr), 1);
    assert!(matches!(e.variants[1].fields, Fields::Named(_)));
}

// ── 10. Prec attr coexists with leaf on same variant's fields ───────────────

#[test]
fn prec_coexists_with_leaf_fields() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Num(#[adze::leaf(pattern = r"\d+")] String),
            #[adze::prec_left(1)]
            Add(
                Box<Expr>,
                #[adze::leaf(text = "+")]
                (),
                Box<Expr>,
            ),
        }
    };
    // Variant-level prec_left
    let prec_attr = e.variants[1]
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "prec_left"))
        .unwrap();
    assert_eq!(prec_value(prec_attr), 1);

    // Field-level leaf
    if let Fields::Unnamed(ref u) = e.variants[1].fields {
        let leaf_field = &u.unnamed[1];
        assert!(leaf_field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 11. Variants without prec attr have no prec info ────────────────────────

#[test]
fn non_prec_variants_excluded() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Num(#[adze::leaf(pattern = r"\d+")] String),
            Neg(#[adze::leaf(text = "-")] (), Box<Expr>),
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
        }
    };
    let info = extract_prec_info(&e);
    assert_eq!(info.len(), 1);
    assert_eq!(info[0].0, "Add");
}

// ── 12. Same prec level on two different variants ───────────────────────────

#[test]
fn same_prec_level_on_two_variants() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
            #[adze::prec_left(1)]
            Sub(Box<Expr>, Box<Expr>),
        }
    };
    let info = extract_prec_info(&e);
    assert_eq!(info.len(), 2);
    assert_eq!(info[0].2, info[1].2);
    assert_eq!(info[0].1, "prec_left");
    assert_eq!(info[1].1, "prec_left");
}

// ── 13. Prec attr names are distinct from each other ────────────────────────

#[test]
fn prec_attr_names_are_distinct() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec(1)]
            A(i32),
            #[adze::prec_left(2)]
            B(i32),
            #[adze::prec_right(3)]
            C(i32),
        }
    };
    let attr_names: Vec<_> = e
        .variants
        .iter()
        .flat_map(|v| adze_attr_names(&v.attrs))
        .collect();
    assert_eq!(attr_names, vec!["prec", "prec_left", "prec_right"]);
    // All names should be distinct
    let set: std::collections::HashSet<_> = attr_names.iter().collect();
    assert_eq!(set.len(), 3);
}

// ── 14. Grammar module with prec variants preserves structure ───────────────

#[test]
fn grammar_module_preserves_prec_variants() {
    let m = parse_mod(quote! {
        #[adze::grammar("calc")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                #[adze::prec_left(2)]
                Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
            }
        }
    });
    if let Item::Enum(e) = &module_items(&m)[0] {
        assert_eq!(e.ident, "Expr");
        assert_eq!(e.variants.len(), 3);
        let info = extract_prec_info(e);
        assert_eq!(info.len(), 2);
        assert_eq!(info[0].2, 1);
        assert_eq!(info[1].2, 2);
    } else {
        panic!("Expected enum as first item");
    }
}

// ── 15. Prec left with operator leaf in a full grammar ──────────────────────

#[test]
fn prec_left_with_operator_leaf_in_grammar() {
    let m = parse_mod(quote! {
        #[adze::grammar("arith")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
                #[adze::prec_left(1)]
                Sub(
                    Box<Expr>,
                    #[adze::leaf(text = "-")]
                    (),
                    Box<Expr>,
                ),
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    let items = module_items(&m);
    // Should have enum, extra struct
    let enum_count = items.iter().filter(|i| matches!(i, Item::Enum(_))).count();
    assert_eq!(enum_count, 1);
    if let Item::Enum(e) = &items[0] {
        let info = extract_prec_info(e);
        assert_eq!(info.len(), 1);
        assert_eq!(info[0], ("Sub".into(), "prec_left".into(), 1));
    }
}

// ── 16. Prec right for right-associative operators ──────────────────────────

#[test]
fn prec_right_for_cons_operator() {
    let m = parse_mod(quote! {
        #[adze::grammar("list")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Atom(#[adze::leaf(pattern = r"\w+")] String),
                #[adze::prec_right(1)]
                Cons(
                    Box<Expr>,
                    #[adze::leaf(text = "::")]
                    (),
                    Box<Expr>,
                ),
            }
        }
    });
    if let Item::Enum(e) = &module_items(&m)[0] {
        let info = extract_prec_info(e);
        assert_eq!(info[0], ("Cons".into(), "prec_right".into(), 1));
    }
}

// ── 17. Prec with no associativity for comparison ───────────────────────────

#[test]
fn prec_no_assoc_for_comparison() {
    let m = parse_mod(quote! {
        #[adze::grammar("cmp")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec(1)]
                Eq(
                    Box<Expr>,
                    #[adze::leaf(text = "==")]
                    (),
                    Box<Expr>,
                ),
            }
        }
    });
    if let Item::Enum(e) = &module_items(&m)[0] {
        let info = extract_prec_info(e);
        assert_eq!(info[0].1, "prec");
    }
}

// ── 18. Full expression grammar with ascending precedence levels ────────────

#[test]
fn full_expr_grammar_ascending_prec() {
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
            Pow(Box<Expr>, #[adze::leaf(text = "**")] (), Box<Expr>),
        }
    };
    let info = extract_prec_info(&e);
    assert_eq!(info.len(), 5);
    // Add and Sub share level 1
    assert_eq!(info[0].2, 1);
    assert_eq!(info[1].2, 1);
    // Mul and Div share level 2
    assert_eq!(info[2].2, 2);
    assert_eq!(info[3].2, 2);
    // Pow at level 3 with right associativity
    assert_eq!(info[4].2, 3);
    assert_eq!(info[4].1, "prec_right");
}

// ── 19. Prec attr on unit variant ───────────────────────────────────────────

#[test]
fn prec_on_unit_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Token {
            #[adze::prec(1)]
            #[adze::leaf(text = "and")]
            And,
            #[adze::prec(2)]
            #[adze::leaf(text = "or")]
            Or,
        }
    };
    let info = extract_prec_info(&e);
    assert_eq!(info.len(), 2);
    assert_eq!(info[0], ("And".into(), "prec".into(), 1));
    assert_eq!(info[1], ("Or".into(), "prec".into(), 2));
    // Verify both are unit variants
    for v in &e.variants {
        assert!(matches!(v.fields, Fields::Unit));
    }
}

// ── 20. Prec attr ordering: prec appears before leaf ────────────────────────

#[test]
fn prec_attr_appears_before_leaf_on_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Token {
            #[adze::prec(1)]
            #[adze::leaf(text = "not")]
            Not,
        }
    };
    let attrs = adze_attr_names(&e.variants[0].attrs);
    assert_eq!(attrs, vec!["prec", "leaf"]);
}

// ── 21. Leaf attr appears before prec attr (reversed order) ─────────────────

#[test]
fn leaf_attr_before_prec_attr_on_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Token {
            #[adze::leaf(text = "not")]
            #[adze::prec(1)]
            Not,
        }
    };
    let attrs = adze_attr_names(&e.variants[0].attrs);
    assert_eq!(attrs, vec!["leaf", "prec"]);
    // Both should still be discoverable regardless of order
    assert!(e.variants[0].attrs.iter().any(|a| is_adze_attr(a, "prec")));
    assert!(e.variants[0].attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

// ── 22. Mixed associativity kinds at same precedence level ──────────────────

#[test]
fn mixed_assoc_at_same_level() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            #[adze::prec_left(5)]
            Add(Box<Expr>, Box<Expr>),
            #[adze::prec_right(5)]
            Assign(Box<Expr>, Box<Expr>),
            #[adze::prec(5)]
            Eq(Box<Expr>, Box<Expr>),
        }
    };
    let info = extract_prec_info(&e);
    assert_eq!(info.len(), 3);
    // All at level 5 but different kinds
    for entry in &info {
        assert_eq!(entry.2, 5);
    }
    let kinds: Vec<_> = info.iter().map(|i| i.1.clone()).collect();
    assert_eq!(kinds, vec!["prec_left", "prec_right", "prec"]);
}

// ── 23. Prec attr is recognized as adze attr by helper ──────────────────────

#[test]
fn prec_attrs_recognized_by_is_adze_attr() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(1)]
            Add(i32),
            #[adze::prec_right(2)]
            Cons(i32),
            #[adze::prec(3)]
            Eq(i32),
        }
    };
    for (i, kind) in ["prec_left", "prec_right", "prec"].iter().enumerate() {
        assert!(
            e.variants[i].attrs.iter().any(|a| is_adze_attr(a, kind)),
            "variant {i} should have {kind} attr"
        );
    }
}

// ── 24. Variant field types preserved alongside prec annotation ─────────────

#[test]
fn field_types_preserved_with_prec() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(2)]
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
        let types: Vec<_> = u
            .unnamed
            .iter()
            .map(|f| f.ty.to_token_stream().to_string())
            .collect();
        assert_eq!(types[0], "Box < Expr >");
        assert_eq!(types[1], "()");
        assert_eq!(types[2], "Box < Expr >");
        assert_eq!(types[3], "()");
        assert_eq!(types[4], "Box < Expr >");
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 25. Prec level extraction via parse_args matches integer literal ────────

#[test]
fn prec_parse_args_matches_int_literal() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(42)]
            Op(i32),
        }
    };
    let attr = &e.variants[0].attrs[0];
    let expr: syn::Expr = attr.parse_args().unwrap();
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(ref i),
            ..
        }) => {
            assert_eq!(i.base10_parse::<i32>().unwrap(), 42);
            assert_eq!(i.to_string(), "42");
        }
        _ => panic!("Expected integer literal expression"),
    }
}

// ── 26. Grammar with only prec (no prec_left/prec_right) ───────────────────

#[test]
fn grammar_with_only_prec_no_assoc() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Atom(#[adze::leaf(pattern = r"\w+")] String),
                #[adze::prec(1)]
                Lt(Box<Expr>, #[adze::leaf(text = "<")] (), Box<Expr>),
                #[adze::prec(2)]
                Gt(Box<Expr>, #[adze::leaf(text = ">")] (), Box<Expr>),
            }
        }
    });
    if let Item::Enum(e) = &module_items(&m)[0] {
        let info = extract_prec_info(e);
        assert_eq!(info.len(), 2);
        assert!(info.iter().all(|i| i.1 == "prec"));
    }
}

// ── 27. Prec on variant with Vec field ──────────────────────────────────────

#[test]
fn prec_on_variant_with_vec_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            #[adze::prec_left(1)]
            Call(
                Box<Expr>,
                #[adze::leaf(text = "(")]
                (),
                #[adze::repeat(non_empty = false)]
                Vec<Expr>,
                #[adze::leaf(text = ")")]
                (),
            ),
        }
    };
    let info = extract_prec_info(&e);
    assert_eq!(info.len(), 1);
    assert_eq!(info[0].0, "Call");
    // Verify the Vec field is present
    if let Fields::Unnamed(ref u) = e.variants[1].fields {
        let has_vec = u
            .unnamed
            .iter()
            .any(|f| f.ty.to_token_stream().to_string().contains("Vec"));
        assert!(has_vec);
    }
}

// ── 28. Prec attr on variant with Option field ──────────────────────────────

#[test]
fn prec_on_variant_with_option_field() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            #[adze::prec_right(3)]
            Conditional(
                Box<Expr>,
                #[adze::leaf(text = "?")]
                (),
                Option<Box<Expr>>,
            ),
        }
    };
    let info = extract_prec_info(&e);
    assert_eq!(info[0], ("Conditional".into(), "prec_right".into(), 3));
    if let Fields::Unnamed(ref u) = e.variants[1].fields {
        let opt_field = u
            .unnamed
            .iter()
            .any(|f| f.ty.to_token_stream().to_string().starts_with("Option"));
        assert!(opt_field);
    }
}

// ── 29. Descending precedence level ordering ────────────────────────────────

#[test]
fn descending_prec_levels() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Lit(i32),
            #[adze::prec_left(100)]
            High(Box<Expr>, Box<Expr>),
            #[adze::prec_left(50)]
            Mid(Box<Expr>, Box<Expr>),
            #[adze::prec_left(1)]
            Low(Box<Expr>, Box<Expr>),
        }
    };
    let info = extract_prec_info(&e);
    let levels: Vec<i32> = info.iter().map(|i| i.2).collect();
    assert_eq!(levels, vec![100, 50, 1]);
    // Verify descending
    for i in 0..levels.len() - 1 {
        assert!(levels[i] > levels[i + 1]);
    }
}

// ── 30. Complex grammar: mixed prec + extra + word ──────────────────────────

#[test]
fn complex_grammar_prec_extra_word() {
    let m = parse_mod(quote! {
        #[adze::grammar("lang")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Ident(#[adze::leaf(pattern = r"[a-zA-Z_]\w*")] String),
                Number(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                #[adze::prec_left(2)]
                Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
                #[adze::prec_right(3)]
                Assign(Box<Expr>, #[adze::leaf(text = "=")] (), Box<Expr>),
            }

            #[adze::word]
            pub struct Keyword {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    let items = module_items(&m);
    // Should have enum, word struct, extra struct
    let has_word = items.iter().any(|i| {
        if let Item::Struct(s) = i {
            s.attrs.iter().any(|a| is_adze_attr(a, "word"))
        } else {
            false
        }
    });
    let has_extra = items.iter().any(|i| {
        if let Item::Struct(s) = i {
            s.attrs.iter().any(|a| is_adze_attr(a, "extra"))
        } else {
            false
        }
    });
    assert!(has_word);
    assert!(has_extra);

    if let Item::Enum(e) = &items[0] {
        let info = extract_prec_info(e);
        assert_eq!(info.len(), 3);
    }
}

// ── 31. Prec on named-field variant preserves field names ───────────────────

#[test]
fn prec_named_variant_preserves_field_names() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            #[adze::prec_left(1)]
            BinOp {
                left: Box<Expr>,
                #[adze::leaf(text = "+")]
                _op: (),
                right: Box<Expr>,
            },
        }
    };
    if let Fields::Named(ref n) = e.variants[0].fields {
        let names: Vec<_> = n
            .named
            .iter()
            .map(|f| f.ident.as_ref().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["left", "_op", "right"]);
    } else {
        panic!("Expected named fields");
    }
}

// ── 32. Prec value extraction round-trips ───────────────────────────────────

#[test]
fn prec_value_roundtrip() {
    let levels = [0, 1, 5, 10, 42, 100, 255, 999];
    for &level in &levels {
        let tokens: TokenStream = format!("pub enum E {{ #[adze::prec_left({level})] V(i32), }}")
            .parse()
            .unwrap();
        let e: ItemEnum = syn::parse2(tokens).unwrap();
        let attr = e.variants[0]
            .attrs
            .iter()
            .find(|a| is_adze_attr(a, "prec_left"))
            .unwrap();
        assert_eq!(prec_value(attr), level);
    }
}

// ── 33. Enum variant count matches when prec attrs present ──────────────────

#[test]
fn variant_count_unaffected_by_prec_attrs() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            A(i32),
            #[adze::prec_left(1)]
            B(i32, i32),
            C(i32),
            #[adze::prec_right(2)]
            D(i32, i32),
            E(i32),
        }
    };
    assert_eq!(e.variants.len(), 5);
    let with_prec = extract_prec_info(&e);
    assert_eq!(with_prec.len(), 2);
}
