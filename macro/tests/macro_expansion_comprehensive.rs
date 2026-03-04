#![allow(clippy::needless_range_loop)]

//! Comprehensive macro expansion tests for adze-macro attribute parsing.
//!
//! These tests verify that attribute parsing works correctly using only quote!,
//! syn, and proc_macro2. No expansion functions are imported from the proc-macro crate.
//! Tests focus on TokenStream construction and attribute parsing verification.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Error, Fields, ItemEnum, ItemStruct, parse2};

// ── Helper Functions ────────────────────────────────────────────────────────

/// Parse a TokenStream as an ItemStruct
fn parse_struct(tokens: TokenStream) -> syn::Result<ItemStruct> {
    parse2(tokens)
}

/// Parse a TokenStream as an ItemEnum
fn parse_enum(tokens: TokenStream) -> syn::Result<ItemEnum> {
    parse2(tokens)
}

/// Extract attribute names from a list of attributes
fn attr_names(attrs: &[Attribute]) -> Vec<String> {
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

/// Check if an attribute matches an adze attribute name
fn has_adze_attr(attrs: &[Attribute], name: &str) -> bool {
    attr_names(attrs).iter().any(|n| n == name)
}

// ── 1. Parse #[adze::grammar] on struct ─────────────────────────────────────

#[test]
fn test_parse_grammar_attribute_on_struct() {
    let tokens = quote! {
        #[adze::grammar("test_lang")]
        struct TestGrammar {
            value: i32,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();
    assert_eq!(item.ident, "TestGrammar");
    assert!(has_adze_attr(&item.attrs, "grammar"));
}

// ── 2. Parse #[adze::grammar] on enum ────────────────────────────────────────

#[test]
fn test_parse_grammar_attribute_on_enum() {
    let tokens = quote! {
        #[adze::grammar("expr")]
        enum Expr {
            Number(i32),
            Add(Box<Expr>, Box<Expr>),
        }
    };

    let result = parse_enum(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();
    assert_eq!(item.ident, "Expr");
    assert!(has_adze_attr(&item.attrs, "grammar"));
}

// ── 3. Parse #[adze::language] attribute ─────────────────────────────────────

#[test]
fn test_parse_language_attribute() {
    let tokens = quote! {
        #[adze::language]
        struct Program {
            statements: Vec<Statement>,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();
    assert!(has_adze_attr(&item.attrs, "language"));
}

// ── 4. Parse #[adze::leaf] with text pattern ────────────────────────────────

#[test]
fn test_parse_leaf_attribute_with_text() {
    let tokens = quote! {
        struct Plus {
            #[adze::leaf(text = "+")]
            op: (),
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    if let Fields::Named(ref fields) = item.fields {
        let field = fields.named.iter().next().unwrap();
        assert!(has_adze_attr(&field.attrs, "leaf"));
    } else {
        panic!("Expected named fields");
    }
}

// ── 5. Parse #[adze::leaf] with regex pattern ───────────────────────────────

#[test]
fn test_parse_leaf_attribute_with_pattern() {
    let tokens = quote! {
        struct Number {
            #[adze::leaf(pattern = r"\d+")]
            num: u32,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    if let Fields::Named(ref fields) = item.fields {
        let field = fields.named.iter().next().unwrap();
        assert!(has_adze_attr(&field.attrs, "leaf"));
    } else {
        panic!("Expected named fields");
    }
}

// ── 6. Parse #[adze::leaf] with transform closure ────────────────────────────

#[test]
fn test_parse_leaf_attribute_with_transform() {
    let tokens = quote! {
        struct IntValue {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<i32>().unwrap())]
            value: i32,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    if let Fields::Named(ref fields) = item.fields {
        let field = fields.named.iter().next().unwrap();
        assert!(has_adze_attr(&field.attrs, "leaf"));
    } else {
        panic!("Expected named fields");
    }
}

// ── 7. Parse #[adze::word] attribute ─────────────────────────────────────────

#[test]
fn test_parse_word_attribute() {
    let tokens = quote! {
        #[adze::word]
        struct Identifier {
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            name: String,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();
    assert!(has_adze_attr(&item.attrs, "word"));
}

// ── 8. Parse #[adze::skip] with value ────────────────────────────────────────

#[test]
fn test_parse_skip_attribute_with_value() {
    let tokens = quote! {
        struct Node {
            #[adze::skip(false)]
            visited: bool,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    if let Fields::Named(ref fields) = item.fields {
        let field = fields.named.iter().next().unwrap();
        assert!(has_adze_attr(&field.attrs, "skip"));
    } else {
        panic!("Expected named fields");
    }
}

// ── 9. Parse #[adze::skip] with different value ──────────────────────────────

#[test]
fn test_parse_skip_attribute_with_expr() {
    let tokens = quote! {
        struct Metadata {
            #[adze::skip(42)]
            count: i32,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    if let Fields::Named(ref fields) = item.fields {
        let field = fields.named.iter().next().unwrap();
        assert!(has_adze_attr(&field.attrs, "skip"));
    } else {
        panic!("Expected named fields");
    }
}

// ── 10. Parse #[adze::extra] attribute ───────────────────────────────────────

#[test]
fn test_parse_extra_attribute() {
    let tokens = quote! {
        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            ws: (),
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();
    assert!(has_adze_attr(&item.attrs, "extra"));
}

// ── 11. Parse #[adze::prec_left] with precedence level ────────────────────────

#[test]
fn test_parse_prec_left_attribute() {
    let tokens = quote! {
        enum Expr {
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
        }
    };

    let result = parse_enum(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    if let Fields::Unnamed(_) = &item.variants.iter().next().unwrap().fields {
        let variant = item.variants.iter().next().unwrap();
        assert!(has_adze_attr(&variant.attrs, "prec_left"));
    } else {
        panic!("Expected unnamed fields");
    }
}

// ── 12. Parse #[adze::prec_right] with precedence level ───────────────────────

#[test]
fn test_parse_prec_right_attribute() {
    let tokens = quote! {
        enum Expr {
            #[adze::prec_right(2)]
            Cons(Box<Expr>, Box<Expr>),
        }
    };

    let result = parse_enum(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    let variant = item.variants.iter().next().unwrap();
    assert!(has_adze_attr(&variant.attrs, "prec_right"));
}

// ── 13. Parse #[adze::prec] (non-associative) ────────────────────────────────

#[test]
fn test_parse_prec_attribute() {
    let tokens = quote! {
        enum Expr {
            #[adze::prec(3)]
            Compare(Box<Expr>, Box<Expr>),
        }
    };

    let result = parse_enum(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    let variant = item.variants.iter().next().unwrap();
    assert!(has_adze_attr(&variant.attrs, "prec"));
}

// ── 14. Parse #[adze::repeat] attribute ──────────────────────────────────────

#[test]
fn test_parse_repeat_attribute() {
    let tokens = quote! {
        struct Statement {
            #[adze::repeat(non_empty = true)]
            items: Vec<Item>,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    if let Fields::Named(ref fields) = item.fields {
        let field = fields.named.iter().next().unwrap();
        assert!(has_adze_attr(&field.attrs, "repeat"));
    } else {
        panic!("Expected named fields");
    }
}

// ── 15. Parse #[adze::repeat] with non_empty = false ─────────────────────────

#[test]
fn test_parse_repeat_attribute_non_empty_false() {
    let tokens = quote! {
        struct Items {
            #[adze::repeat(non_empty = false)]
            values: Vec<Value>,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    if let Fields::Named(ref fields) = item.fields {
        let field = fields.named.iter().next().unwrap();
        assert!(has_adze_attr(&field.attrs, "repeat"));
    } else {
        panic!("Expected named fields");
    }
}

// ── 16. Parse #[adze::delimited] with separator ──────────────────────────────

#[test]
fn test_parse_delimited_attribute() {
    let tokens = quote! {
        struct List {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<Item>,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    if let Fields::Named(ref fields) = item.fields {
        let field = fields.named.iter().next().unwrap();
        assert!(has_adze_attr(&field.attrs, "delimited"));
    } else {
        panic!("Expected named fields");
    }
}

// ── 17. Parse #[adze::external] attribute ────────────────────────────────────

#[test]
fn test_parse_external_attribute() {
    let tokens = quote! {
        #[adze::external]
        struct IndentToken;
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();
    assert!(has_adze_attr(&item.attrs, "external"));
}

// ── 18. Multiple attributes on same struct ───────────────────────────────────

#[test]
fn test_multiple_attributes_on_struct() {
    let tokens = quote! {
        #[adze::external]
        #[adze::word]
        struct SpecialToken {
            value: String,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();
    let names = attr_names(&item.attrs);
    assert!(names.contains(&"external".to_string()));
    assert!(names.contains(&"word".to_string()));
}

// ── 19. Multiple attributes on enum variant ──────────────────────────────────

#[test]
fn test_multiple_attributes_on_enum_variant() {
    let tokens = quote! {
        enum Expr {
            #[adze::prec_left(1)]
            #[adze::repeat(non_empty = true)]
            Add(Box<Expr>, Box<Expr>),
        }
    };

    let result = parse_enum(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();
    let variant = item.variants.iter().next().unwrap();
    let names = attr_names(&variant.attrs);
    assert!(names.contains(&"prec_left".to_string()));
}

// ── 20. Attributes on multiple fields within struct ──────────────────────────

#[test]
fn test_attributes_on_multiple_fields() {
    let tokens = quote! {
        struct Binary {
            #[adze::leaf(text = "+")]
            op: (),
            #[adze::leaf(pattern = r"\d+")]
            left: i32,
            #[adze::leaf(pattern = r"\d+")]
            right: i32,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    if let Fields::Named(ref fields) = item.fields {
        assert_eq!(fields.named.len(), 3);
        let all_leaf = fields.named.iter().all(|f| has_adze_attr(&f.attrs, "leaf"));
        assert!(all_leaf);
    } else {
        panic!("Expected named fields");
    }
}

// ── 21. Attributes on multiple enum variants ─────────────────────────────────

#[test]
fn test_attributes_on_multiple_enum_variants() {
    let tokens = quote! {
        enum Expr {
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
            #[adze::prec_left(1)]
            Sub(Box<Expr>, Box<Expr>),
            #[adze::prec_right(2)]
            Pow(Box<Expr>, Box<Expr>),
        }
    };

    let result = parse_enum(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();
    assert_eq!(item.variants.len(), 3);

    let variant_attrs: Vec<Vec<String>> =
        item.variants.iter().map(|v| attr_names(&v.attrs)).collect();

    assert!(variant_attrs[0].contains(&"prec_left".to_string()));
    assert!(variant_attrs[1].contains(&"prec_left".to_string()));
    assert!(variant_attrs[2].contains(&"prec_right".to_string()));
}

// ── 22. Complex nested attribute: leaf with all parameters ──────────────────

#[test]
fn test_leaf_with_all_parameters() {
    let tokens = quote! {
        struct FloatLit {
            #[adze::leaf(pattern = r"\d+\.\d+", transform = |v| v.parse::<f64>().unwrap())]
            value: f64,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    if let Fields::Named(ref fields) = item.fields {
        let field = fields.named.iter().next().unwrap();
        assert!(has_adze_attr(&field.attrs, "leaf"));
    } else {
        panic!("Expected named fields");
    }
}

// ── 23. Struct with grammar + language attributes ──────────────────────────

#[test]
fn test_struct_with_grammar_and_language() {
    let tokens = quote! {
        #[adze::grammar("calc")]
        #[adze::language]
        struct Calculator {
            expr: Box<Expr>,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();
    let names = attr_names(&item.attrs);
    assert!(names.contains(&"grammar".to_string()));
    assert!(names.contains(&"language".to_string()));
}

// ── 24. Enum variant with leaf field attributes ──────────────────────────────

#[test]
fn test_enum_variant_with_leaf_fields() {
    let tokens = quote! {
        enum Literal {
            Number(
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<i32>().unwrap())]
                i32
            ),
            String(
                #[adze::leaf(pattern = r#""[^"]*""#)]
                String
            ),
        }
    };

    let result = parse_enum(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    for variant in item.variants.iter() {
        if let Fields::Unnamed(ref fields) = variant.fields {
            let field = fields.unnamed.iter().next().unwrap();
            assert!(has_adze_attr(&field.attrs, "leaf"));
        }
    }
}

// ── 25. Roundtrip: TokenStream → ItemStruct → TokenStream ───────────────────

#[test]
fn test_roundtrip_struct_tokenstream() {
    let original = quote! {
        #[adze::language]
        struct Program {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };

    let result = parse_struct(original.clone());
    assert!(result.is_ok());
    let item = result.unwrap();
    let roundtrip = quote! { #item };

    // The roundtrip should parse successfully
    let reparsed = parse_struct(roundtrip);
    assert!(reparsed.is_ok());
}

// ── 26. Roundtrip: TokenStream → ItemEnum → TokenStream ─────────────────────

#[test]
fn test_roundtrip_enum_tokenstream() {
    let original = quote! {
        #[adze::language]
        enum Expr {
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
        }
    };

    let result = parse_enum(original.clone());
    assert!(result.is_ok());
    let item = result.unwrap();
    let roundtrip = quote! { #item };

    // The roundtrip should parse successfully
    let reparsed = parse_enum(roundtrip);
    assert!(reparsed.is_ok());
}

// ── 27. Attribute with empty parentheses ─────────────────────────────────────

#[test]
fn test_attribute_with_empty_parens() {
    let tokens = quote! {
        #[adze::language]
        struct Start {
            value: String,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();
    assert!(has_adze_attr(&item.attrs, "language"));
}

// ── 28. Multiple leaf attributes with different patterns on fields ──────────

#[test]
fn test_multiple_leaf_patterns() {
    let tokens = quote! {
        struct Tokens {
            #[adze::leaf(pattern = r"[a-z]+")]
            word: String,
            #[adze::leaf(pattern = r"[0-9]+")]
            number: String,
            #[adze::leaf(pattern = r"[+\-*/]")]
            op: String,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    if let Fields::Named(ref fields) = item.fields {
        assert_eq!(fields.named.len(), 3);
        for field in fields.named.iter() {
            assert!(has_adze_attr(&field.attrs, "leaf"));
        }
    } else {
        panic!("Expected named fields");
    }
}

// ── 29. Enum with mixed precedence attributes ────────────────────────────────

#[test]
fn test_enum_mixed_precedence() {
    let tokens = quote! {
        enum Expr {
            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),
            #[adze::prec_left(2)]
            Mul(Box<Expr>, Box<Expr>),
            #[adze::prec_right(3)]
            Power(Box<Expr>, Box<Expr>),
            #[adze::prec(4)]
            Compare(Box<Expr>, Box<Expr>),
        }
    };

    let result = parse_enum(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();
    assert_eq!(item.variants.len(), 4);
}

// ── 30. Struct with repeat and delimited on same field ──────────────────────

#[test]
fn test_repeat_and_delimited_attributes() {
    let tokens = quote! {
        struct List {
            #[adze::repeat(non_empty = true)]
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<Item>,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    if let Fields::Named(ref fields) = item.fields {
        let field = fields.named.iter().next().unwrap();
        let names = attr_names(&field.attrs);
        assert!(names.contains(&"repeat".to_string()));
        assert!(names.contains(&"delimited".to_string()));
    } else {
        panic!("Expected named fields");
    }
}

// ── 31. Leaf attribute with closure transform using method calls ─────────────

#[test]
fn test_leaf_with_complex_transform() {
    let tokens = quote! {
        struct SpecialNumber {
            #[adze::leaf(
                pattern = r"\d+",
                transform = |v| v.parse::<i32>().expect("parse failed").abs()
            )]
            value: i32,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
}

// ── 32. Unit struct with external attribute ──────────────────────────────────

#[test]
fn test_unit_struct_with_external() {
    let tokens = quote! {
        #[adze::external]
        struct Dedent;
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();
    assert_eq!(item.ident, "Dedent");
    assert!(has_adze_attr(&item.attrs, "external"));
}

// ── 33. Enum unit variant with leaf ──────────────────────────────────────────

#[test]
fn test_enum_unit_variant_with_leaf() {
    let tokens = quote! {
        enum Token {
            #[adze::leaf(text = "true")]
            True,
            #[adze::leaf(text = "false")]
            False,
        }
    };

    let result = parse_enum(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    for variant in item.variants.iter() {
        assert!(has_adze_attr(&variant.attrs, "leaf"));
    }
}

// ── 34. Struct with skip on multiple fields ──────────────────────────────────

#[test]
fn test_multiple_skip_attributes() {
    let tokens = quote! {
        struct Node {
            #[adze::skip(false)]
            visited: bool,
            #[adze::skip(0)]
            id: usize,
            #[adze::skip(None)]
            parent: Option<Box<Node>>,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    if let Fields::Named(ref fields) = item.fields {
        let skip_count = fields
            .named
            .iter()
            .filter(|f| has_adze_attr(&f.attrs, "skip"))
            .count();
        assert_eq!(skip_count, 3);
    } else {
        panic!("Expected named fields");
    }
}

// ── 35. Enum variant with multiple skip fields ───────────────────────────────

#[test]
fn test_enum_variant_with_skip_fields() {
    let tokens = quote! {
        enum Node {
            Internal(
                Box<Node>,
                #[adze::skip(false)]
                bool,
                Box<Node>,
            ),
        }
    };

    let result = parse_enum(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();
    let variant = item.variants.iter().next().unwrap();

    if let Fields::Unnamed(ref fields) = variant.fields {
        let skip_count = fields
            .unnamed
            .iter()
            .filter(|f| has_adze_attr(&f.attrs, "skip"))
            .count();
        assert_eq!(skip_count, 1);
    }
}

// ── 36. Grammar attribute with different name values ─────────────────────────

#[test]
fn test_grammar_attribute_different_names() {
    let tokens1 = quote! {
        #[adze::grammar("simple_lang")]
        struct Grammar1 {}
    };

    let tokens2 = quote! {
        #[adze::grammar("complex_lang")]
        struct Grammar2 {}
    };

    let result1 = parse_struct(tokens1);
    let result2 = parse_struct(tokens2);

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

// ── 37. Leaf with raw string pattern containing escapes ──────────────────────

#[test]
fn test_leaf_raw_string_pattern() {
    let tokens = quote! {
        struct StringLit {
            #[adze::leaf(pattern = r#""([^"\\]|\\.)*""#)]
            value: String,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
}

// ── 38. Complex enum with heterogeneous variants ──────────────────────────────

#[test]
fn test_complex_enum_structure() {
    let tokens = quote! {
        enum Statement {
            #[adze::prec_left(1)]
            If(
                #[adze::leaf(text = "if")]
                (),
                Box<Expr>,
                Box<Statement>,
            ),
            #[adze::prec_left(2)]
            While(
                #[adze::leaf(text = "while")]
                (),
                Box<Expr>,
                Box<Statement>,
            ),
            #[adze::prec_right(3)]
            Block(Vec<Box<Statement>>),
        }
    };

    let result = parse_enum(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();
    assert_eq!(item.variants.len(), 3);
}

// ── 39. Verify attribute path structure ──────────────────────────────────────

#[test]
fn test_attribute_path_structure() {
    let tokens = quote! {
        #[adze::language]
        struct Test {}
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    assert_eq!(item.attrs.len(), 1);
    let attr = &item.attrs[0];

    let segs: Vec<_> = attr.path().segments.iter().collect();
    assert_eq!(segs.len(), 2);
    assert_eq!(segs[0].ident, "adze");
    assert_eq!(segs[1].ident, "language");
}

// ── 40. Verify field attribute path structure ────────────────────────────────

#[test]
fn test_field_attribute_path_structure() {
    let tokens = quote! {
        struct Test {
            #[adze::leaf(text = "x")]
            field: (),
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    if let Fields::Named(ref fields) = item.fields {
        let field = fields.named.iter().next().unwrap();
        assert_eq!(field.attrs.len(), 1);

        let attr = &field.attrs[0];
        let segs: Vec<_> = attr.path().segments.iter().collect();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].ident, "adze");
        assert_eq!(segs[1].ident, "leaf");
    }
}

// ── 41. Struct with extra and language combined ───────────────────────────────

#[test]
fn test_extra_and_language_combination() {
    let tokens = quote! {
        #[adze::extra]
        #[adze::language]
        struct CommentOrWhitespace {
            #[adze::leaf(pattern = r"//.*")]
            content: String,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();
    let names = attr_names(&item.attrs);
    assert!(names.contains(&"extra".to_string()));
    assert!(names.contains(&"language".to_string()));
}

// ── 42. Enum with all precedence variants ────────────────────────────────────

#[test]
fn test_enum_all_precedence_types() {
    let tokens = quote! {
        enum Expr {
            #[adze::prec(0)]
            Type0(Box<Expr>),
            #[adze::prec_left(1)]
            TypeLeft1(Box<Expr>),
            #[adze::prec_right(1)]
            TypeRight1(Box<Expr>),
            #[adze::prec_left(2)]
            TypeLeft2(Box<Expr>),
            #[adze::prec_right(2)]
            TypeRight2(Box<Expr>),
        }
    };

    let result = parse_enum(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();
    assert_eq!(item.variants.len(), 5);
}

// ── 43. Struct field visibility with attributes ──────────────────────────────

#[test]
fn test_field_visibility_with_attributes() {
    let tokens = quote! {
        struct Public {
            pub field1: i32,
            pub field2: String,
            #[adze::leaf(text = "x")]
            pub field3: (),
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    if let Fields::Named(ref fields) = item.fields {
        let leaf_field = fields
            .named
            .iter()
            .find(|f| has_adze_attr(&f.attrs, "leaf"))
            .unwrap();
        assert!(!matches!(leaf_field.vis, syn::Visibility::Inherited));
    }
}

// ── 44. Enum variant field with nested Box types ──────────────────────────────

#[test]
fn test_enum_variant_nested_box_types() {
    let tokens = quote! {
        enum Expr {
            Deeply(
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<i64>().unwrap())]
                Box<Box<i64>>,
            ),
        }
    };

    let result = parse_enum(tokens);
    assert!(result.is_ok());
}

// ── 45. Verify struct ident is preserved through roundtrip ────────────────────

#[test]
fn test_struct_ident_preserved() {
    let tokens = quote! {
        #[adze::language]
        struct MySpecialGrammar {
            field: String,
        }
    };

    let parsed = parse_struct(tokens).unwrap();
    assert_eq!(parsed.ident, "MySpecialGrammar");

    let roundtrip = quote! { #parsed };
    let reparsed = parse_struct(roundtrip).unwrap();
    assert_eq!(reparsed.ident, "MySpecialGrammar");
}

// ── 46. Verify enum ident is preserved through roundtrip ──────────────────────

#[test]
fn test_enum_ident_preserved() {
    let tokens = quote! {
        #[adze::language]
        enum Expression {
            Add(Box<Expression>, Box<Expression>),
        }
    };

    let parsed = parse_enum(tokens).unwrap();
    assert_eq!(parsed.ident, "Expression");

    let roundtrip = quote! { #parsed };
    let reparsed = parse_enum(roundtrip).unwrap();
    assert_eq!(reparsed.ident, "Expression");
}

// ── 47. Struct with generic parameters and attributes ──────────────────────

#[test]
fn test_generic_struct_with_attributes() {
    let tokens = quote! {
        #[adze::language]
        struct Generic<T> {
            value: T,
        }
    };

    let result = parse_struct(tokens);
    // Note: syn should parse this successfully
    assert!(result.is_ok());
}

// ── 48. Complex leaf pattern with word boundaries ────────────────────────────

#[test]
fn test_leaf_pattern_word_boundaries() {
    let tokens = quote! {
        struct Identifier {
            #[adze::leaf(pattern = r"\b[a-zA-Z_]\w*\b")]
            name: String,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
}

// ── 49. Enum with unit variants mixed with tuple variants ──────────────────

#[test]
fn test_enum_mixed_variant_types() {
    let tokens = quote! {
        enum Token {
            #[adze::leaf(text = "true")]
            True,
            #[adze::leaf(text = "false")]
            False,
            Number(
                #[adze::leaf(pattern = r"\d+")]
                i32
            ),
            Identifier(
                #[adze::leaf(pattern = r"[a-z]+")]
                String
            ),
        }
    };

    let result = parse_enum(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();
    assert_eq!(item.variants.len(), 4);
}

// ── 50. Verify leaf fields count in struct ───────────────────────────────────

#[test]
fn test_leaf_fields_count() {
    let tokens = quote! {
        struct Arithmetic {
            #[adze::leaf(pattern = r"\d+")]
            left: i32,
            #[adze::leaf(text = "+")]
            op: (),
            #[adze::leaf(pattern = r"\d+")]
            right: i32,
        }
    };

    let result = parse_struct(tokens);
    assert!(result.is_ok());
    let item = result.unwrap();

    if let Fields::Named(ref fields) = item.fields {
        let leaf_count = fields
            .named
            .iter()
            .filter(|f| has_adze_attr(&f.attrs, "leaf"))
            .count();
        assert_eq!(leaf_count, 3);
    }
}
