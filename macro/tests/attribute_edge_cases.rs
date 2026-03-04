//! Comprehensive edge case tests for adze macro attributes.
//!
//! This test suite focuses on edge cases and boundary conditions for various
//! `#[adze::*]` attributes, including empty names, special characters, complex patterns,
//! unusual type combinations, and attribute interactions.
//!
//! These tests verify that:
//! - Attributes are properly recognized on various item types
//! - Grammar modules can contain diverse attribute combinations
//! - Edge cases in attribute arguments are handled correctly
//! - Complex type structures are properly annotated

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Item, ItemEnum, ItemMod, ItemStruct, parse_quote};

// ── Helper Functions ────────────────────────────────────────────────────────

/// Check whether an attribute path matches `adze::<name>`.
fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segs: Vec<_> = attr.path().segments.iter().collect();
    segs.len() == 2 && segs[0].ident == "adze" && segs[1].ident == name
}

/// Collect all `adze::*` attribute names from an attribute list.
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

/// Parse a token stream as an `ItemStruct`.
fn _parse_struct(tokens: TokenStream) -> ItemStruct {
    syn::parse2(tokens).expect("failed to parse struct")
}

/// Parse a token stream as an `ItemEnum`.
fn _parse_enum(tokens: TokenStream) -> ItemEnum {
    syn::parse2(tokens).expect("failed to parse enum")
}

/// Parse a token stream as an `ItemMod`.
fn parse_mod(tokens: TokenStream) -> ItemMod {
    syn::parse2(tokens).expect("failed to parse module")
}

/// Extract items from a module.
fn module_items(m: &ItemMod) -> &[Item] {
    &m.content.as_ref().expect("module has no content").1
}

/// Find the root type name (annotated with `#[adze::language]`) in a module.
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

/// Check if a module has a grammar attribute.
fn has_grammar_attr(m: &ItemMod) -> bool {
    m.attrs.iter().any(|a| is_adze_attr(a, "grammar"))
}

/// Extract the grammar name from a module's `#[adze::grammar("...")]` attribute.
fn extract_grammar_name(m: &ItemMod) -> Option<String> {
    m.attrs.iter().find_map(|a| {
        if !is_adze_attr(a, "grammar") {
            return None;
        }
        let expr: syn::Expr = a.parse_args().ok()?;
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
}

/// Count items of a specific type in a module.
fn count_items_by_type(m: &ItemMod, check: fn(&Item) -> bool) -> usize {
    module_items(m).iter().filter(|i| check(i)).count()
}

// ── 1. Grammar Attribute Edge Cases ─────────────────────────────────────────

/// Test 1: Grammar attribute with empty string name (should parse)
#[test]
fn grammar_with_empty_name() {
    let m = parse_mod(quote! {
        #[adze::grammar("")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    });
    assert!(has_grammar_attr(&m));
    assert_eq!(extract_grammar_name(&m), Some(String::from("")));
}

/// Test 2: Grammar attribute with special characters in name
#[test]
fn grammar_with_special_chars_in_name() {
    let m = parse_mod(quote! {
        #[adze::grammar("test_grammar-2024@v1")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    });
    assert!(has_grammar_attr(&m));
    assert_eq!(
        extract_grammar_name(&m),
        Some("test_grammar-2024@v1".to_string())
    );
}

/// Test 3: Grammar attribute with whitespace-only name
#[test]
fn grammar_with_whitespace_name() {
    let m = parse_mod(quote! {
        #[adze::grammar("   ")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    });
    assert!(has_grammar_attr(&m));
    assert_eq!(extract_grammar_name(&m), Some("   ".to_string()));
}

/// Test 4: Grammar attribute with very long name (>100 chars)
#[test]
fn grammar_with_very_long_name() {
    let long_name = "a".repeat(100);
    let m = parse_mod(quote! {
        #[adze::grammar(#long_name)]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    });
    assert!(has_grammar_attr(&m));
    let name = extract_grammar_name(&m);
    assert!(name.is_some());
    assert_eq!(name.unwrap().len(), 100);
}

/// Test 5: Grammar with unicode characters in name
#[test]
fn grammar_with_unicode_name() {
    let m = parse_mod(quote! {
        #[adze::grammar("gramma_文法")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    });
    assert!(has_grammar_attr(&m));
    assert_eq!(extract_grammar_name(&m), Some("gramma_文法".to_string()));
}

// ── 2. Language Attribute Edge Cases ────────────────────────────────────────

/// Test 6: Language attribute on struct (valid)
#[test]
fn language_attribute_on_struct() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    });
    assert_eq!(find_language_type(&m), Some("Root".to_string()));
}

/// Test 7: Language attribute on enum (valid)
#[test]
fn language_attribute_on_enum() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
    });
    assert_eq!(find_language_type(&m), Some("Expr".to_string()));
}

// ── 3. Leaf Attribute Edge Cases ────────────────────────────────────────────

/// Test 8: Leaf attribute with single-character pattern
#[test]
fn leaf_with_single_char_pattern() {
    let s: ItemStruct = parse_quote! {
        pub struct Token {
            #[adze::leaf(pattern = r"x")]
            value: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

/// Test 9: Leaf attribute with complex regex pattern
#[test]
fn leaf_with_complex_regex() {
    let s: ItemStruct = parse_quote! {
        pub struct Token {
            #[adze::leaf(pattern = r"(?:[0-9]+\.[0-9]*|\.[0-9]+)([eE][-+]?[0-9]+)?")]
            value: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

/// Test 10: Leaf attribute with unicode regex pattern
#[test]
fn leaf_with_unicode_pattern() {
    let s: ItemStruct = parse_quote! {
        pub struct Token {
            #[adze::leaf(pattern = r"[\p{Letter}][\p{Letter}\p{Number}]*")]
            value: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

/// Test 11: Leaf attribute with empty pattern string (edge case)
#[test]
fn leaf_with_empty_pattern() {
    let s: ItemStruct = parse_quote! {
        pub struct Token {
            #[adze::leaf(pattern = r"")]
            value: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

/// Test 12: Leaf attribute with special regex characters
#[test]
fn leaf_with_special_regex_chars() {
    let s: ItemStruct = parse_quote! {
        pub struct Token {
            #[adze::leaf(pattern = r"(\[\]|{|}|\(|\))")]
            value: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

/// Test 13: Leaf with text literal containing special chars
#[test]
fn leaf_with_special_text_literal() {
    let e: ItemEnum = parse_quote! {
        pub enum Op {
            #[adze::leaf(text = "=>")]
            Arrow,
            #[adze::leaf(text = "::")]
            DoubleColon,
        }
    };
    assert!(
        e.variants
            .iter()
            .any(|v| { v.attrs.iter().any(|a| is_adze_attr(a, "leaf")) })
    );
}

/// Test 14: Leaf with text literal containing single quote
#[test]
fn leaf_with_quote_in_text() {
    let e: ItemEnum = parse_quote! {
        pub enum Quote {
            #[adze::leaf(text = "\"")]
            DoubleQuote,
        }
    };
    assert!(
        e.variants
            .iter()
            .any(|v| { v.attrs.iter().any(|a| is_adze_attr(a, "leaf")) })
    );
}

// ── 4. Precedence Attribute Edge Cases ──────────────────────────────────────

/// Test 15: Precedence with positive value
#[test]
fn precedence_positive_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(#[adze::leaf(pattern = r"\d+")] i32),
            #[adze::prec(42)]
            Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
        }
    };
    assert!(
        e.variants
            .iter()
            .any(|v| { v.attrs.iter().any(|a| is_adze_attr(a, "prec")) })
    );
}

/// Test 16: Precedence with negative value
#[test]
fn precedence_negative_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(#[adze::leaf(pattern = r"\d+")] i32),
            #[adze::prec(-5)]
            LowOp(Box<Expr>, #[adze::leaf(text = "|")] (), Box<Expr>),
        }
    };
    assert!(
        e.variants
            .iter()
            .any(|v| { v.attrs.iter().any(|a| is_adze_attr(a, "prec")) })
    );
}

/// Test 17: Precedence with zero
#[test]
fn precedence_zero_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(#[adze::leaf(pattern = r"\d+")] i32),
            #[adze::prec(0)]
            Op(Box<Expr>, #[adze::leaf(text = "~")] (), Box<Expr>),
        }
    };
    assert!(
        e.variants
            .iter()
            .any(|v| { v.attrs.iter().any(|a| is_adze_attr(a, "prec")) })
    );
}

/// Test 18: Prec_left with various precedence levels
#[test]
fn prec_left_multiple_levels() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(#[adze::leaf(pattern = r"\d+")] i32),
            #[adze::prec_left(1)]
            Sub(Box<Expr>, #[adze::leaf(text = "-")] (), Box<Expr>),
            #[adze::prec_left(2)]
            Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
        }
    };
    let prec_left_count = e
        .variants
        .iter()
        .filter(|v| v.attrs.iter().any(|a| is_adze_attr(a, "prec_left")))
        .count();
    assert_eq!(prec_left_count, 2);
}

/// Test 19: Prec_right with various precedence levels
#[test]
fn prec_right_multiple_levels() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(#[adze::leaf(pattern = r"\d+")] i32),
            #[adze::prec_right(1)]
            Cons(Box<Expr>, #[adze::leaf(text = "::")] (), Box<Expr>),
            #[adze::prec_right(3)]
            Power(Box<Expr>, #[adze::leaf(text = "^")] (), Box<Expr>),
        }
    };
    let prec_right_count = e
        .variants
        .iter()
        .filter(|v| v.attrs.iter().any(|a| is_adze_attr(a, "prec_right")))
        .count();
    assert_eq!(prec_right_count, 2);
}

// ── 5. Multiple Attribute Combinations ──────────────────────────────────────

/// Test 20: Multiple attributes on same enum variant
#[test]
fn multiple_attributes_on_variant() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(#[adze::leaf(pattern = r"\d+")] i32),
            #[adze::prec_left(1)]
            Add(
                Box<Expr>,
                #[adze::leaf(text = "+")]
                (),
                Box<Expr>,
            ),
        }
    };
    let add_variant = e.variants.iter().find(|v| v.ident == "Add").unwrap();
    let attrs = adze_attr_names(&add_variant.attrs);
    assert!(attrs.contains(&"prec_left".to_string()));
}

/// Test 21: Skip attribute on field
#[test]
fn skip_attribute_on_field() {
    let s: ItemStruct = parse_quote! {
        pub struct MyNode {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
            #[adze::skip(false)]
            visited: bool,
            #[adze::skip(true)]
            processed: bool,
        }
    };
    let skip_count = s
        .fields
        .iter()
        .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "skip")))
        .count();
    assert_eq!(skip_count, 2);
}

/// Test 22: Extra attribute on struct
#[test]
fn extra_attribute_on_struct() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                Number(#[adze::leaf(pattern = r"\d+")] i32),
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                _ws: (),
            }
        }
    });
    let has_extra = module_items(&m).iter().any(|item| match item {
        Item::Struct(s) => s.attrs.iter().any(|a| is_adze_attr(a, "extra")),
        _ => false,
    });
    assert!(has_extra);
}

// ── 6. Unusual Type Combinations ────────────────────────────────────────────

/// Test 23: Unit struct with leaf attribute
#[test]
fn unit_struct_with_leaf() {
    let e: ItemEnum = parse_quote! {
        pub enum Keyword {
            #[adze::leaf(text = "if")]
            If,
            #[adze::leaf(text = "else")]
            Else,
        }
    };
    assert_eq!(e.variants.len(), 2);
    assert!(
        e.variants
            .iter()
            .all(|v| { v.attrs.iter().any(|a| is_adze_attr(a, "leaf")) })
    );
}

/// Test 24: Tuple struct with leaf field
#[test]
fn tuple_struct_with_leaf() {
    let s: ItemStruct = parse_quote! {
        pub struct Number(
            #[adze::leaf(pattern = r"\d+")]
            i32
        );
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

/// Test 25: Unit enum variant with leaf
#[test]
fn unit_enum_variant_with_leaf() {
    let e: ItemEnum = parse_quote! {
        pub enum Op {
            #[adze::leaf(text = "+")]
            Plus,
            #[adze::leaf(text = "-")]
            Minus,
        }
    };
    assert_eq!(e.variants.len(), 2);
    assert!(
        e.variants
            .iter()
            .all(|v| { v.attrs.iter().any(|a| is_adze_attr(a, "leaf")) })
    );
}

/// Test 26: Struct with delimited vector field
#[test]
fn delimited_vector_field() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "delimited")));
}

/// Test 27: Repeat field with non_empty configuration
#[test]
fn repeat_non_empty_field() {
    let s: ItemStruct = parse_quote! {
        pub struct List {
            #[adze::repeat(non_empty = true)]
            items: Vec<Item>,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
}

// ── 7. Complex Grammar Structures ───────────────────────────────────────────

/// Test 28: Grammar with multiple variants and different precedence levels
#[test]
fn complex_expression_grammar() {
    let m = parse_mod(quote! {
        #[adze::grammar("complex_expr")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Literal(#[adze::leaf(pattern = r"\d+")] i32),

                #[adze::prec_left(1)]
                Add(
                    Box<Expr>,
                    #[adze::leaf(text = "+")]
                    (),
                    Box<Expr>,
                ),

                #[adze::prec_left(2)]
                Mul(
                    Box<Expr>,
                    #[adze::leaf(text = "*")]
                    (),
                    Box<Expr>,
                ),

                #[adze::prec_right(3)]
                Power(
                    Box<Expr>,
                    #[adze::leaf(text = "^")]
                    (),
                    Box<Expr>,
                ),
            }
        }
    });
    assert!(has_grammar_attr(&m));
    assert_eq!(find_language_type(&m), Some("Expr".to_string()));
}

/// Test 29: Grammar with word attribute
#[test]
fn grammar_with_word_attribute() {
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
    let has_word = module_items(&m).iter().any(|item| match item {
        Item::Struct(s) => s.attrs.iter().any(|a| is_adze_attr(a, "word")),
        _ => false,
    });
    assert!(has_word);
}

/// Test 30: Grammar with external attribute
#[test]
fn grammar_with_external_attribute() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Code {
                indent: IndentToken,
            }

            #[adze::external]
            struct IndentToken {
                #[adze::leaf(pattern = r"\t+")]
                _indent: (),
            }
        }
    });
    let has_external = module_items(&m).iter().any(|item| match item {
        Item::Struct(s) => s.attrs.iter().any(|a| is_adze_attr(a, "external")),
        _ => false,
    });
    assert!(has_external);
}

/// Test 31: Grammar with multiple extra types
#[test]
fn grammar_with_multiple_extras() {
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
                #[adze::leaf(pattern = r" ")]
                _ws: (),
            }

            #[adze::extra]
            struct Newline {
                #[adze::leaf(pattern = r"\n")]
                _nl: (),
            }

            #[adze::extra]
            struct Comment {
                #[adze::leaf(pattern = r"//[^\n]*")]
                _comment: (),
            }
        }
    });
    let extra_count = count_items_by_type(&m, |item| match item {
        Item::Struct(s) => s.attrs.iter().any(|a| is_adze_attr(a, "extra")),
        _ => false,
    });
    assert_eq!(extra_count, 3);
}

// ── 8. Optional and Vector Fields ───────────────────────────────────────────

/// Test 32: Optional field in struct
#[test]
fn optional_field_in_struct() {
    let s: ItemStruct = parse_quote! {
        pub struct MaybeNum {
            #[adze::leaf(pattern = r"\d+")]
            value: Option<i32>,
        }
    };
    assert_eq!(s.fields.len(), 1);
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

/// Test 33: Vector field in struct
#[test]
fn vector_field_in_struct() {
    let s: ItemStruct = parse_quote! {
        pub struct NumList {
            #[adze::repeat(non_empty = false)]
            items: Vec<Item>,
        }
    };
    assert_eq!(s.fields.len(), 1);
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "repeat")));
}

/// Test 34: Boxed recursive type
#[test]
fn boxed_recursive_type() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(#[adze::leaf(pattern = r"\d+")] i32),
            Neg(
                #[adze::leaf(text = "-")]
                (),
                Box<Expr>,
            ),
        }
    };
    assert_eq!(e.variants.len(), 2);
}

// ── 9. Transform and Conversion Edge Cases ──────────────────────────────────

/// Test 35: Leaf with simple transform function
#[test]
fn leaf_with_simple_transform() {
    let s: ItemStruct = parse_quote! {
        pub struct Num {
            #[adze::leaf(pattern = r"\d+", transform = |s| s.len())]
            digit_count: usize,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

/// Test 36: Leaf with complex transform closure
#[test]
fn leaf_with_complex_transform() {
    let s: ItemStruct = parse_quote! {
        pub struct Num {
            #[adze::leaf(pattern = r"\d+", transform = |s: &str| s.parse::<i32>().unwrap_or(0))]
            value: i32,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

// ── 10. Visibility and Module Structure ─────────────────────────────────────

/// Test 37: Public visibility on language type
#[test]
fn public_visibility_language() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Number(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
    });
    assert_eq!(find_language_type(&m), Some("Expr".to_string()));
}

/// Test 38: Private visibility on language type
#[test]
fn private_visibility_language() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            enum Expr {
                Number(#[adze::leaf(pattern = r"\d+")] i32),
            }
        }
    });
    assert_eq!(find_language_type(&m), Some("Expr".to_string()));
}

/// Test 39: Pub(crate) visibility
#[test]
fn pub_crate_visibility() {
    let m = parse_mod(quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub(crate) struct Root {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }
        }
    });
    assert_eq!(find_language_type(&m), Some("Root".to_string()));
}

// ── 11. Numeric Boundaries and Limits ───────────────────────────────────────

/// Test 40: Very large precedence value
#[test]
fn large_precedence_value() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(#[adze::leaf(pattern = r"\d+")] i32),
            #[adze::prec(999999)]
            Op(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
        }
    };
    assert!(
        e.variants
            .iter()
            .any(|v| { v.attrs.iter().any(|a| is_adze_attr(a, "prec")) })
    );
}

/// Test 41: Very large negative precedence value
#[test]
fn large_negative_precedence() {
    let e: ItemEnum = parse_quote! {
        pub enum Expr {
            Number(#[adze::leaf(pattern = r"\d+")] i32),
            #[adze::prec(-999999)]
            Op(Box<Expr>, #[adze::leaf(text = "|")] (), Box<Expr>),
        }
    };
    assert!(
        e.variants
            .iter()
            .any(|v| { v.attrs.iter().any(|a| is_adze_attr(a, "prec")) })
    );
}

// ── 12. Special Pattern Cases ───────────────────────────────────────────────

/// Test 42: Leaf pattern with escape sequences
#[test]
fn leaf_pattern_with_escapes() {
    let s: ItemStruct = parse_quote! {
        pub struct Token {
            #[adze::leaf(pattern = r"\d+\.\d+")]
            value: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

/// Test 43: Leaf pattern with alternation
#[test]
fn leaf_pattern_with_alternation() {
    let e: ItemEnum = parse_quote! {
        pub enum Token {
            #[adze::leaf(pattern = r"true|false")]
            Bool(String),
            #[adze::leaf(pattern = r"null|nil|undefined")]
            Null(String),
        }
    };
    assert_eq!(e.variants.len(), 2);
    assert!(
        e.variants
            .iter()
            .all(|v| { v.attrs.iter().any(|a| is_adze_attr(a, "leaf")) })
    );
}

/// Test 44: Leaf pattern with character class
#[test]
fn leaf_pattern_with_char_class() {
    let s: ItemStruct = parse_quote! {
        pub struct HexToken {
            #[adze::leaf(pattern = r"[0-9a-fA-F]+")]
            value: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}

/// Test 45: Leaf pattern with negated character class
#[test]
fn leaf_pattern_with_negated_class() {
    let s: ItemStruct = parse_quote! {
        pub struct LineToken {
            #[adze::leaf(pattern = r"[^\n]+")]
            value: String,
        }
    };
    let field = s.fields.iter().next().unwrap();
    assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
}
