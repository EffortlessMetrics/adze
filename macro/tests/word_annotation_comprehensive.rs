#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for `#[adze::word]` annotation handling in the adze macro crate.
//!
//! Tests cover word annotation parsing, word patterns with special characters,
//! word with regex patterns, multiple words in grammar, word as language root,
//! word combined with other annotations, and edge cases.

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

// ── 1. Basic word annotation on struct ──────────────────────────────────────

#[test]
fn word_attr_recognized_on_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Identifier {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
}

// ── 2. Word annotation with leaf text field ─────────────────────────────────

#[test]
fn word_with_leaf_text_field() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Keyword {
            #[adze::leaf(text = "let")]
            _kw: (),
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
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
        assert_eq!(s.value(), "let");
    } else {
        panic!("Expected string literal");
    }
}

// ── 3. Word annotation with leaf pattern (identifier regex) ─────────────────

#[test]
fn word_with_identifier_regex_pattern() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Ident {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
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
        assert_eq!(s.value(), r"[a-zA-Z_]\w*");
    } else {
        panic!("Expected string literal");
    }
}

// ── 4. Word pattern with Unicode character classes ──────────────────────────

#[test]
fn word_pattern_unicode_character_class() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct UnicodeIdent {
            #[adze::leaf(pattern = r"[\p{L}_][\p{L}\p{N}_]*")]
            name: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
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
        assert_eq!(s.value(), r"[\p{L}_][\p{L}\p{N}_]*");
    } else {
        panic!("Expected string literal");
    }
}

// ── 5. Word pattern with special regex characters ───────────────────────────

#[test]
fn word_pattern_special_regex_chars() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct SpecialToken {
            #[adze::leaf(pattern = r"[a-zA-Z$_][a-zA-Z0-9$_]*")]
            name: String,
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
        assert_eq!(s.value(), r"[a-zA-Z$_][a-zA-Z0-9$_]*");
    } else {
        panic!("Expected string literal");
    }
}

// ── 6. Word pattern with hyphen-allowed identifiers ─────────────────────────

#[test]
fn word_pattern_hyphen_identifiers() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct LispIdent {
            #[adze::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_\-]*")]
            name: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
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
        assert!(s.value().contains(r"\-"));
    } else {
        panic!("Expected string literal");
    }
}

// ── 7. Word with transform parameter ────────────────────────────────────────

#[test]
fn word_with_transform_closure() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Identifier {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*", transform = |v| v.to_uppercase())]
            name: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
    let field = s.fields.iter().next().unwrap();
    let attr = field
        .attrs
        .iter()
        .find(|a| is_adze_attr(a, "leaf"))
        .unwrap();
    let params = leaf_params(attr);
    let names: Vec<_> = params.iter().map(|p| p.path.to_string()).collect();
    assert!(names.contains(&"pattern".to_owned()));
    assert!(names.contains(&"transform".to_owned()));
}

// ── 8. Word in grammar module ───────────────────────────────────────────────

#[test]
fn word_in_grammar_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Code {
                ident: Identifier,
            }

            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }
        }
    });
    let items = module_items(&m);
    let has_word = items.iter().any(|i| {
        if let Item::Struct(s) = i {
            s.attrs.iter().any(|a| is_adze_attr(a, "word"))
        } else {
            false
        }
    });
    assert!(has_word);
}

// ── 9. Word combined with language annotation ───────────────────────────────

#[test]
fn word_combined_with_language() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        #[adze::language]
        pub struct Root {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }
    };
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"word".to_string()));
    assert!(names.contains(&"language".to_string()));
}

// ── 10. Word as language root in grammar module ─────────────────────────────

#[test]
fn word_as_language_root_in_module() {
    let m = parse_mod(quote! {
        #[adze::grammar("word_root")]
        mod grammar {
            #[adze::word]
            #[adze::language]
            pub struct Token {
                #[adze::leaf(pattern = r"\w+")]
                value: String,
            }
        }
    });
    let items = module_items(&m);
    let root = items.iter().find_map(|i| {
        if let Item::Struct(s) = i
            && s.attrs.iter().any(|a| is_adze_attr(a, "language"))
        {
            return Some(s);
        }
        None
    });
    assert!(root.is_some());
    let root = root.unwrap();
    assert!(root.attrs.iter().any(|a| is_adze_attr(a, "word")));
}

// ── 11. Multiple words in grammar (only one should be word) ─────────────────

#[test]
fn multiple_structs_one_word() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Code {
                ident: Identifier,
                kw: Keyword,
            }

            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }

            pub struct Keyword {
                #[adze::leaf(text = "let")]
                _kw: (),
            }
        }
    });
    let items = module_items(&m);
    let word_count = items
        .iter()
        .filter(|i| {
            if let Item::Struct(s) = i {
                s.attrs.iter().any(|a| is_adze_attr(a, "word"))
            } else {
                false
            }
        })
        .count();
    assert_eq!(word_count, 1);
}

// ── 12. Word attribute ordering among multiple attributes ───────────────────

#[test]
fn word_attr_ordering_preserved() {
    let s: ItemStruct = parse_quote! {
        #[adze::language]
        #[adze::word]
        pub struct Ident {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    let names = adze_attr_names(&s.attrs);
    assert_eq!(names[0], "language");
    assert_eq!(names[1], "word");
}

// ── 13. Word struct with no fields (unit struct) ────────────────────────────

#[test]
fn word_on_unit_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        #[adze::leaf(text = "identifier")]
        pub struct WordToken;
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
    assert!(matches!(s.fields, Fields::Unit));
}

// ── 14. Word struct with tuple fields ───────────────────────────────────────

#[test]
fn word_on_tuple_struct() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Identifier(
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            String
        );
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
    assert!(matches!(s.fields, Fields::Unnamed(_)));
    if let Fields::Unnamed(ref u) = s.fields {
        assert_eq!(u.unnamed.len(), 1);
    }
}

// ── 15. Word with extra and external in same module ─────────────────────────

#[test]
fn word_with_extra_and_external() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
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

            #[adze::external]
            struct IndentToken;
        }
    });
    let items = module_items(&m);
    let mut found_word = false;
    let mut found_extra = false;
    let mut found_external = false;
    for item in items {
        if let Item::Struct(s) = item {
            if s.attrs.iter().any(|a| is_adze_attr(a, "word")) {
                found_word = true;
            }
            if s.attrs.iter().any(|a| is_adze_attr(a, "extra")) {
                found_extra = true;
            }
            if s.attrs.iter().any(|a| is_adze_attr(a, "external")) {
                found_external = true;
            }
        }
    }
    assert!(found_word);
    assert!(found_extra);
    assert!(found_external);
}

// ── 16. Word struct name preserved correctly ────────────────────────────────

#[test]
fn word_struct_name_preserved() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct MyCustomIdentifier {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }
    };
    assert_eq!(s.ident.to_string(), "MyCustomIdentifier");
}

// ── 17. Word attribute has no arguments ─────────────────────────────────────

#[test]
fn word_attr_has_no_arguments() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Ident {
            name: String,
        }
    };
    let word_attr = s.attrs.iter().find(|a| is_adze_attr(a, "word")).unwrap();
    // word attribute uses path-style (no parentheses/arguments)
    assert!(
        matches!(word_attr.meta, syn::Meta::Path(_)),
        "Expected path-style attribute with no arguments"
    );
}

// ── 18. Word pattern with digit-starting regex ──────────────────────────────

#[test]
fn word_pattern_digit_start_allowed() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct NumericIdent {
            #[adze::leaf(pattern = r"[0-9a-zA-Z_]+")]
            name: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
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
        assert_eq!(s.value(), r"[0-9a-zA-Z_]+");
    } else {
        panic!("Expected string literal");
    }
}

// ── 19. Word with complex regex quantifiers ─────────────────────────────────

#[test]
fn word_pattern_complex_quantifiers() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Token {
            #[adze::leaf(pattern = r"[a-zA-Z_]{1,}[a-zA-Z0-9_]{0,255}")]
            name: String,
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
        assert!(s.value().contains("{1,}"));
        assert!(s.value().contains("{0,255}"));
    } else {
        panic!("Expected string literal");
    }
}

// ── 20. Word struct visibility variants ─────────────────────────────────────

#[test]
fn word_struct_private_visibility() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        struct PrivateIdent {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
    assert!(matches!(s.vis, syn::Visibility::Inherited));
}

// ── 21. Word struct crate visibility ────────────────────────────────────────

#[test]
fn word_struct_crate_visibility() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub(crate) struct CrateIdent {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
    assert!(matches!(s.vis, syn::Visibility::Restricted(_)));
}

// ── 22. Word combined with enum using keywords ──────────────────────────────

#[test]
fn word_alongside_keyword_enum() {
    let m = parse_mod(quote! {
        #[adze::grammar("kw_test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Ident(Identifier),
                #[adze::leaf(text = "true")]
                True,
                #[adze::leaf(text = "false")]
                False,
            }

            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }
        }
    });
    let items = module_items(&m);
    let has_word = items.iter().any(|i| {
        if let Item::Struct(s) = i {
            s.attrs.iter().any(|a| is_adze_attr(a, "word"))
        } else {
            false
        }
    });
    let has_enum = items.iter().any(|i| matches!(i, Item::Enum(_)));
    assert!(has_word);
    assert!(has_enum);
}

// ── 23. Word struct field count ─────────────────────────────────────────────

#[test]
fn word_struct_single_leaf_field() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Ident {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }
    };
    assert_eq!(s.fields.iter().count(), 1);
}

// ── 24. Word struct with multiple fields ────────────────────────────────────

#[test]
fn word_struct_multiple_fields() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct RichIdent {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
            #[adze::skip(false)]
            is_keyword: bool,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
    assert_eq!(s.fields.iter().count(), 2);
    let has_leaf = s
        .fields
        .iter()
        .any(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    let has_skip = s
        .fields
        .iter()
        .any(|f| f.attrs.iter().any(|a| is_adze_attr(a, "skip")));
    assert!(has_leaf);
    assert!(has_skip);
}

// ── 25. Word in grammar with precedence operators ───────────────────────────

#[test]
fn word_in_grammar_with_precedence() {
    let m = parse_mod(quote! {
        #[adze::grammar("expr_lang")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Ident(Identifier),
                Number(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                #[adze::prec_left(2)]
                Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
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
    let has_word = items.iter().any(|i| {
        if let Item::Struct(s) = i {
            s.attrs.iter().any(|a| is_adze_attr(a, "word"))
        } else {
            false
        }
    });
    assert!(has_word);

    // Verify the enum still has its variants
    let expr_enum = items.iter().find_map(|i| {
        if let Item::Enum(e) = i {
            if e.ident == "Expr" { Some(e) } else { None }
        } else {
            None
        }
    });
    assert!(expr_enum.is_some());
    assert_eq!(expr_enum.unwrap().variants.len(), 4);
}

// ── 26. Word annotation does not appear on enum ─────────────────────────────

#[test]
fn word_attr_on_enum_parses() {
    // While word is typically used on structs, the attribute itself is
    // syntactically valid on enums too — it's a pass-through macro
    let e: ItemEnum = parse_quote! {
        #[adze::word]
        pub enum TokenType {
            #[adze::leaf(text = "if")]
            If,
            #[adze::leaf(text = "else")]
            Else,
        }
    };
    assert!(e.attrs.iter().any(|a| is_adze_attr(a, "word")));
    assert_eq!(e.variants.len(), 2);
}

// ── 27. Word struct leaf pattern value preserved exactly ────────────────────

#[test]
fn word_leaf_pattern_exact_preservation() {
    let pattern = r"[a-zA-Z\u{00C0}-\u{024F}_][a-zA-Z\u{00C0}-\u{024F}0-9_]*";
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct ExtendedIdent {
            #[adze::leaf(pattern = r"[a-zA-Z\u{00C0}-\u{024F}_][a-zA-Z\u{00C0}-\u{024F}0-9_]*")]
            name: String,
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
        assert_eq!(s.value(), pattern);
    } else {
        panic!("Expected string literal");
    }
}

// ── 28. Word struct field type is String ────────────────────────────────────

#[test]
fn word_field_type_is_string() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Ident {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert_eq!(field.ty.to_token_stream().to_string(), "String");
}

// ── 29. Word attr path segments correct ─────────────────────────────────────

#[test]
fn word_attr_path_has_two_segments() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Ident {
            name: String,
        }
    };
    let word_attr = s.attrs.iter().find(|a| is_adze_attr(a, "word")).unwrap();
    let segs: Vec<_> = word_attr.path().segments.iter().collect();
    assert_eq!(segs.len(), 2);
    assert_eq!(segs[0].ident.to_string(), "adze");
    assert_eq!(segs[1].ident.to_string(), "word");
}

// ── 30. Word does not interfere with non-adze attributes ────────────────────

#[test]
fn word_preserves_non_adze_attrs() {
    let s: ItemStruct = parse_quote! {
        #[derive(Debug, Clone)]
        #[adze::word]
        pub struct Identifier {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    // Should have derive + word
    assert_eq!(s.attrs.len(), 2);
    let adze_names = adze_attr_names(&s.attrs);
    assert_eq!(adze_names, vec!["word"]);
    // The derive attr should still be there
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

// ── 31. Word struct in module is distinct from language struct ───────────────

#[test]
fn word_struct_distinct_from_language() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                name: Identifier,
            }

            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }
        }
    });
    let items = module_items(&m);
    let language_structs: Vec<_> = items
        .iter()
        .filter_map(|i| {
            if let Item::Struct(s) = i
                && s.attrs.iter().any(|a| is_adze_attr(a, "language"))
            {
                return Some(s.ident.to_string());
            }
            None
        })
        .collect();
    let word_structs: Vec<_> = items
        .iter()
        .filter_map(|i| {
            if let Item::Struct(s) = i
                && s.attrs.iter().any(|a| is_adze_attr(a, "word"))
            {
                return Some(s.ident.to_string());
            }
            None
        })
        .collect();
    assert_eq!(language_structs, vec!["Program"]);
    assert_eq!(word_structs, vec!["Identifier"]);
}

// ── 32. Word pattern with alternation ───────────────────────────────────────

#[test]
fn word_pattern_with_alternation() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct MixedToken {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*|@[a-zA-Z_]\w*")]
            name: String,
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
    } else {
        panic!("Expected string literal");
    }
}

// ── 33. Word in module with delimited list ──────────────────────────────────

#[test]
fn word_in_module_with_delimited_list() {
    let m = parse_mod(quote! {
        #[adze::grammar("list_lang")]
        mod grammar {
            #[adze::language]
            pub struct IdentList {
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                idents: Vec<Identifier>,
            }

            #[adze::word]
            pub struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }
        }
    });
    let items = module_items(&m);
    let has_word = items.iter().any(|i| {
        if let Item::Struct(s) = i {
            s.attrs.iter().any(|a| is_adze_attr(a, "word"))
        } else {
            false
        }
    });
    assert!(has_word);
    // Verify the language struct has a delimited field
    let lang = items.iter().find_map(|i| {
        if let Item::Struct(s) = i
            && s.attrs.iter().any(|a| is_adze_attr(a, "language"))
        {
            return Some(s);
        }
        None
    });
    assert!(lang.is_some());
    let field = lang.unwrap().fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
}

// ── 34. Word annotation count in attribute list ─────────────────────────────

#[test]
fn word_appears_exactly_once_in_attrs() {
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct Ident {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    let word_count = s.attrs.iter().filter(|a| is_adze_attr(a, "word")).count();
    assert_eq!(word_count, 1);
}

// ── 35. Word with empty pattern ─────────────────────────────────────────────

#[test]
fn word_with_empty_string_pattern() {
    // Edge case: empty pattern string is syntactically valid at macro level
    let s: ItemStruct = parse_quote! {
        #[adze::word]
        pub struct EmptyWord {
            #[adze::leaf(pattern = "")]
            name: String,
        }
    };
    assert!(s.attrs.iter().any(|a| is_adze_attr(a, "word")));
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
