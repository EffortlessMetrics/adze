#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for `#[adze::extra]` annotation handling in the adze macro crate.
//!
//! Tests cover extra annotation parsing, multiple extras in grammar, extra with regex
//! patterns, extra combined with other annotations, whitespace patterns, comment patterns,
//! extra on struct vs enum, and edge cases.

use adze_common::NameValueExpr;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::{Attribute, Fields, Item, ItemEnum, ItemMod, ItemStruct, Token, parse_quote};

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

// ── 1. Basic extra annotation on struct ─────────────────────────────────────

#[test]
fn extra_attr_recognized_on_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
}

// ── 2. Extra attribute has no arguments ─────────────────────────────────────

#[test]
fn extra_attr_has_no_arguments() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    let extra_attr = s.attrs.iter().find(|a| is_adze_attr(a, "extra")).unwrap();
    assert!(
        matches!(extra_attr.meta, syn::Meta::Path(_)),
        "Expected path-style attribute with no arguments"
    );
}

// ── 3. Extra attr path has two segments ─────────────────────────────────────

#[test]
fn extra_attr_path_has_two_segments() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Ws {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    let extra_attr = s.attrs.iter().find(|a| is_adze_attr(a, "extra")).unwrap();
    let segs: Vec<_> = extra_attr.path().segments.iter().collect();
    assert_eq!(segs.len(), 2);
    assert_eq!(segs[0].ident.to_string(), "adze");
    assert_eq!(segs[1].ident.to_string(), "extra");
}

// ── 4. Extra with simple whitespace regex ───────────────────────────────────

#[test]
fn extra_whitespace_regex_pattern() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    assert_eq!(params[0].path.to_string(), "pattern");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &params[0].expr
    {
        assert_eq!(s.value(), r"\s");
    } else {
        panic!("Expected string literal");
    }
}

// ── 5. Extra with multi-character whitespace pattern ────────────────────────

#[test]
fn extra_multichar_whitespace_pattern() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s+")]
            _ws: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
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
        assert_eq!(s.value(), r"\s+");
    } else {
        panic!("Expected string literal");
    }
}

// ── 6. Extra with newline pattern ───────────────────────────────────────────

#[test]
fn extra_newline_pattern() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Newline {
            #[adze::leaf(pattern = r"\n")]
            _nl: (),
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
    let field = s.fields.iter().next().unwrap();
    let attr = field
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
        assert_eq!(s.value(), r"\n");
    } else {
        panic!("Expected string literal");
    }
}

// ── 7. Extra for single-line comment ────────────────────────────────────────

#[test]
fn extra_single_line_comment_pattern() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct LineComment {
            #[adze::leaf(pattern = r"//[^\n]*")]
            _comment: (),
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
    let field = s.fields.iter().next().unwrap();
    let attr = field
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
        assert_eq!(s.value(), r"//[^\n]*");
    } else {
        panic!("Expected string literal");
    }
}

// ── 8. Extra for hash-style comment ─────────────────────────────────────────

#[test]
fn extra_hash_comment_pattern() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct HashComment {
            #[adze::leaf(pattern = r"#[^\n]*")]
            _comment: (),
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
    let field = s.fields.iter().next().unwrap();
    let attr = field
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
        assert_eq!(s.value(), r"#[^\n]*");
    } else {
        panic!("Expected string literal");
    }
}

// ── 9. Extra for semicolon-style comment ────────────────────────────────────

#[test]
fn extra_semicolon_comment_pattern() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct SemiComment {
            #[adze::leaf(pattern = r";[^\n]*")]
            _comment: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
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
        assert_eq!(s.value(), r";[^\n]*");
    } else {
        panic!("Expected string literal");
    }
}

// ── 10. Multiple extras in grammar module ───────────────────────────────────

#[test]
fn multiple_extras_in_grammar() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Code {
                #[adze::leaf(pattern = r"\w+")]
                token: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }

            #[adze::extra]
            struct Comment {
                #[adze::leaf(pattern = r"//[^\n]*")]
                _comment: (),
            }
        }
    });
    let items = module_items(&m);
    let extra_count = items
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

// ── 11. Three extras in grammar ─────────────────────────────────────────────

#[test]
fn three_extras_whitespace_line_comment_block_comment() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Code {
                #[adze::leaf(pattern = r"\w+")]
                token: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }

            #[adze::extra]
            struct LineComment {
                #[adze::leaf(pattern = r"//[^\n]*")]
                _lc: (),
            }

            #[adze::extra]
            struct BlockComment {
                #[adze::leaf(pattern = r"/\*[^*]*\*/")]
                _bc: (),
            }
        }
    });
    let items = module_items(&m);
    let extra_names: Vec<_> = items
        .iter()
        .filter_map(|i| {
            if let Item::Struct(s) = i {
                if s.attrs.iter().any(|a| is_adze_attr(a, "extra")) {
                    return Some(s.ident.to_string());
                }
            }
            None
        })
        .collect();
    assert_eq!(extra_names.len(), 3);
    assert!(extra_names.contains(&"Whitespace".to_string()));
    assert!(extra_names.contains(&"LineComment".to_string()));
    assert!(extra_names.contains(&"BlockComment".to_string()));
}

// ── 12. Extra combined with word and language in module ──────────────────────

#[test]
fn extra_with_word_and_language() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                ident: Identifier,
            }

            #[adze::word]
            pub struct Identifier {
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
    let mut found_language = false;
    let mut found_word = false;
    let mut found_extra = false;
    for item in items {
        if let Item::Struct(s) = item {
            if s.attrs.iter().any(|a| is_adze_attr(a, "language")) {
                found_language = true;
            }
            if s.attrs.iter().any(|a| is_adze_attr(a, "word")) {
                found_word = true;
            }
            if s.attrs.iter().any(|a| is_adze_attr(a, "extra")) {
                found_extra = true;
            }
        }
    }
    assert!(found_language);
    assert!(found_word);
    assert!(found_extra);
}

// ── 13. Extra combined with external in module ──────────────────────────────

#[test]
fn extra_with_external() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Code {
                #[adze::leaf(pattern = r"\w+")]
                token: String,
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
    let has_extra = items.iter().any(|i| {
        if let Item::Struct(s) = i {
            s.attrs.iter().any(|a| is_adze_attr(a, "extra"))
        } else {
            false
        }
    });
    let has_external = items.iter().any(|i| {
        if let Item::Struct(s) = i {
            s.attrs.iter().any(|a| is_adze_attr(a, "external"))
        } else {
            false
        }
    });
    assert!(has_extra);
    assert!(has_external);
}

// ── 14. Extra on enum (syntactically valid) ─────────────────────────────────

#[test]
fn extra_attr_on_enum_parses() {
    // extra is a pass-through macro, syntactically valid on enums
    let e: ItemEnum = parse_quote! {
        #[adze::extra]
        pub enum SkipToken {
            #[adze::leaf(text = " ")]
            Space,
            #[adze::leaf(text = "\t")]
            Tab,
        }
    };
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "extra")));
    assert_eq!(e.variants.len(), 2);
}

// ── 15. Extra struct name preserved correctly ───────────────────────────────

#[test]
fn extra_struct_name_preserved() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct MyCustomWhitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    assert_eq!(s.ident.to_string(), "MyCustomWhitespace");
}

// ── 16. Extra struct private visibility ─────────────────────────────────────

#[test]
fn extra_struct_private_visibility() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    assert!(matches!(s.vis, syn::Visibility::Inherited));
}

// ── 17. Extra struct public visibility ──────────────────────────────────────

#[test]
fn extra_struct_public_visibility() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        pub struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    assert!(matches!(s.vis, syn::Visibility::Public(_)));
}

// ── 18. Extra struct crate visibility ───────────────────────────────────────

#[test]
fn extra_struct_crate_visibility() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        pub(crate) struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    assert!(matches!(s.vis, syn::Visibility::Restricted(_)));
}

// ── 19. Extra on unit struct ────────────────────────────────────────────────

#[test]
fn extra_on_unit_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        #[adze::leaf(pattern = r"\s")]
        struct Whitespace;
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
    assert!(matches!(s.fields, Fields::Unit));
}

// ── 20. Extra on tuple struct ───────────────────────────────────────────────

#[test]
fn extra_on_tuple_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Whitespace(
            #[adze::leaf(pattern = r"\s")]
            ()
        );
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
    assert!(matches!(s.fields, Fields::Unnamed(_)));
    if let Fields::Unnamed(ref u) = s.fields {
        assert_eq!(u.unnamed.len(), 1);
    }
}

// ── 21. Extra field type is unit ────────────────────────────────────────────

#[test]
fn extra_field_type_is_unit() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert_eq!(field.ty.to_token_stream().to_string(), "()");
}

// ── 22. Extra preserves non-adze attributes ─────────────────────────────────

#[test]
fn extra_preserves_non_adze_attrs() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug)]
        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    assert_eq!(s.attrs.len(), 2);
    let adze_names = adze_attr_names(&s.attrs);
    assert_eq!(adze_names, vec!["extra"]);
    let derive_attr = s.attrs.iter().find(|a| {
        a.path()
            .segments
            .iter()
            .next()
            .map(|s| s.ident == "derive")
            .unwrap_or(false)
    });
    assert!(derive_attr.is_some());
}

// ── 23. Extra with tab and space character class ────────────────────────────

#[test]
fn extra_tab_space_character_class() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct HorizontalWs {
            #[adze::leaf(pattern = r"[ \t]+")]
            _ws: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
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
        assert_eq!(s.value(), r"[ \t]+");
    } else {
        panic!("Expected string literal");
    }
}

// ── 24. Extra with carriage-return/newline pattern ──────────────────────────

#[test]
fn extra_crlf_pattern() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct LineEnd {
            #[adze::leaf(pattern = r"\r?\n")]
            _le: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
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
        assert_eq!(s.value(), r"\r?\n");
    } else {
        panic!("Expected string literal");
    }
}

// ── 25. Extra with empty pattern (edge case) ────────────────────────────────

#[test]
fn extra_with_empty_pattern() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct EmptyExtra {
            #[adze::leaf(pattern = "")]
            _empty: (),
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
    let field = s.fields.iter().next().unwrap();
    let attr = field
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

// ── 26. Extra with leaf text (literal match) ────────────────────────────────

#[test]
fn extra_with_leaf_text() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct SpaceToken {
            #[adze::leaf(text = " ")]
            _space: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
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
        assert_eq!(s.value(), " ");
    } else {
        panic!("Expected string literal");
    }
}

// ── 27. Extra appears exactly once in attrs ─────────────────────────────────

#[test]
fn extra_appears_exactly_once_in_attrs() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Ws {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    let extra_count = s.attrs.iter().filter(|a| is_adze_attr(a, "extra")).count();
    assert_eq!(extra_count, 1);
}

// ── 28. Extra struct single field count ─────────────────────────────────────

#[test]
fn extra_struct_single_leaf_field() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    assert_eq!(s.fields.iter().count(), 1);
}

// ── 29. Extra in grammar with precedence operators ──────────────────────────

#[test]
fn extra_in_grammar_with_precedence() {
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
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                #[adze::prec_left(2)]
                Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    let items = module_items(&m);
    let has_extra = items.iter().any(|i| {
        if let Item::Struct(s) = i {
            s.attrs.iter().any(|a| is_adze_attr(a, "extra"))
        } else {
            false
        }
    });
    assert!(has_extra);
    let expr_enum = items.iter().find_map(|i| {
        if let Item::Enum(e) = i {
            if e.ident == "Expr" { Some(e) } else { None }
        } else {
            None
        }
    });
    assert!(expr_enum.is_some());
    assert_eq!(expr_enum.unwrap().variants.len(), 3);
}

// ── 30. Extra distinct from language struct ──────────────────────────────────

#[test]
fn extra_struct_distinct_from_language() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                #[adze::leaf(pattern = r"\w+")]
                token: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    let items = module_items(&m);
    let language_names: Vec<_> = items
        .iter()
        .filter_map(|i| {
            if let Item::Struct(s) = i {
                if s.attrs.iter().any(|a| is_adze_attr(a, "language")) {
                    return Some(s.ident.to_string());
                }
            }
            None
        })
        .collect();
    let extra_names: Vec<_> = items
        .iter()
        .filter_map(|i| {
            if let Item::Struct(s) = i {
                if s.attrs.iter().any(|a| is_adze_attr(a, "extra")) {
                    return Some(s.ident.to_string());
                }
            }
            None
        })
        .collect();
    assert_eq!(language_names, vec!["Program"]);
    assert_eq!(extra_names, vec!["Whitespace"]);
}

// ── 31. Extra with complex regex alternation ────────────────────────────────

#[test]
fn extra_complex_regex_alternation() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct SkipToken {
            #[adze::leaf(pattern = r"\s+|//[^\n]*|/\*[^*]*\*/")]
            _skip: (),
        }
    };
    let field = s.fields.iter().next().unwrap();
    let attr = field
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
        assert!(s.value().contains('|'));
        assert!(s.value().contains(r"\s+"));
        assert!(s.value().contains(r"//[^\n]*"));
    } else {
        panic!("Expected string literal");
    }
}

// ── 32. Extra in module with repeat and delimited ───────────────────────────

#[test]
fn extra_in_module_with_repeat_delimited() {
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

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    let items = module_items(&m);
    let has_extra = items.iter().any(|i| {
        if let Item::Struct(s) = i {
            s.attrs.iter().any(|a| is_adze_attr(a, "extra"))
        } else {
            false
        }
    });
    assert!(has_extra);
    let lang = items.iter().find_map(|i| {
        if let Item::Struct(s) = i {
            if s.attrs.iter().any(|a| is_adze_attr(a, "language")) {
                return Some(s);
            }
        }
        None
    });
    assert!(lang.is_some());
    let field = lang.unwrap().fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
}

// ── 33. Extra attr ordering among multiple attributes ───────────────────────

#[test]
fn extra_attr_ordering_preserved() {
    let s: ItemStruct = parse_quote! {
        #[derive(Clone)]
        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    // derive comes first, then extra
    let adze_names = adze_attr_names(&s.attrs);
    assert_eq!(adze_names, vec!["extra"]);
    assert_eq!(s.attrs.len(), 2);
    // Verify derive is first
    let first_attr = &s.attrs[0];
    assert!(
        first_attr
            .path()
            .segments
            .iter()
            .next()
            .map(|s| s.ident == "derive")
            .unwrap_or(false)
    );
}

// ── 34. Extra with Unicode whitespace pattern ───────────────────────────────

#[test]
fn extra_unicode_whitespace_pattern() {
    let s: ItemStruct = parse_quote! {
        #[adze::extra]
        struct UnicodeWs {
            #[adze::leaf(pattern = r"[\s\u{00A0}\u{2003}]+")]
            _ws: (),
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "extra")));
    let field = s.fields.iter().next().unwrap();
    let attr = field
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
        assert!(s.value().contains(r"\u{00A0}"));
    } else {
        panic!("Expected string literal");
    }
}

// ── 35. Extra with all annotation types in grammar ──────────────────────────

#[test]
fn extra_with_all_annotation_types() {
    let m = parse_mod(quote! {
        #[adze::grammar("full")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Ident(Identifier),
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                #[adze::prec_right(2)]
                Assign(Box<Expr>, #[adze::leaf(text = "=")] (), Box<Expr>),
            }

            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }

            #[adze::extra]
            struct Comment {
                #[adze::leaf(pattern = r"//[^\n]*")]
                _comment: (),
            }

            #[adze::external]
            struct IndentToken;
        }
    });
    let items = module_items(&m);
    let mut found = std::collections::HashMap::new();
    for item in items {
        if let Item::Struct(s) = item {
            for attr_name in &["language", "word", "extra", "external"] {
                if s.attrs.iter().any(|a| is_adze_attr(a, attr_name)) {
                    *found.entry(attr_name.to_string()).or_insert(0usize) += 1;
                }
            }
        }
        if let Item::Enum(e) = item {
            if e.attrs.iter().any(|a| is_adze_attr(a, "language")) {
                *found.entry("language".to_string()).or_insert(0usize) += 1;
            }
        }
    }
    assert_eq!(found.get("language"), Some(&1));
    assert_eq!(found.get("word"), Some(&1));
    assert_eq!(found.get("extra"), Some(&2));
    assert_eq!(found.get("external"), Some(&1));
}
