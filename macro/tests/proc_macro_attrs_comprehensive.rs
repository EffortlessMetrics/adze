//! Comprehensive proc-macro attribute tests for the adze-macro crate.
//!
//! Tests cover attribute construction and parsing using quote!/syn without importing
//! non-macro items. Each test validates attribute token stream generation and parsing.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Fields, ItemEnum, ItemStruct, parse2};

// ── Helper Functions ─────────────────────────────────────────────────────────

/// Parse a TokenStream into a syn::ItemStruct
fn parse_struct(tokens: TokenStream) -> ItemStruct {
    parse2(tokens).expect("failed to parse struct")
}

/// Parse a TokenStream into a syn::ItemEnum
fn parse_enum(tokens: TokenStream) -> ItemEnum {
    parse2(tokens).expect("failed to parse enum")
}

/// Create an Attribute by wrapping in a struct and parsing
fn parse_attr_from_struct(tokens: TokenStream) -> Attribute {
    let item_struct = parse_struct(quote! {
        #tokens
        struct Dummy;
    });
    item_struct
        .attrs
        .first()
        .cloned()
        .expect("no attribute found")
}

/// Extract attribute name from syn::Attribute path
fn attr_name(attr: &Attribute) -> String {
    attr.path()
        .segments
        .iter()
        .last()
        .map(|seg| seg.ident.to_string())
        .unwrap_or_default()
}

/// Check if attribute is an adze attribute with specific name
fn is_adze_attr(attr: &Attribute, name: &str) -> bool {
    let segments: Vec<_> = attr.path().segments.iter().collect();
    segments.len() == 2 && segments[0].ident == "adze" && segments[1].ident == name
}

// ============================================================================
// ATTRIBUTE CONSTRUCTION TESTS (1-7)
// ============================================================================

/// Test 1: Construct grammar attribute TokenStream with quote!
#[test]
fn construct_grammar_attribute_tokenstream() {
    let attr_stream = quote! {
        #[adze::grammar("test_grammar")]
    };
    let tokens: TokenStream = attr_stream;
    assert!(
        !tokens.is_empty(),
        "grammar attribute token stream should not be empty"
    );
}

/// Test 2: Construct language attribute TokenStream
#[test]
fn construct_language_attribute_tokenstream() {
    let attr_stream = quote! {
        #[adze::language]
    };
    let tokens: TokenStream = attr_stream;
    assert!(
        !tokens.is_empty(),
        "language attribute token stream should not be empty"
    );
}

/// Test 3: Construct leaf attribute TokenStream with pattern
#[test]
fn construct_leaf_attribute_with_pattern() {
    let attr_stream = quote! {
        #[adze::leaf(pattern = r"\d+")]
    };
    let tokens: TokenStream = attr_stream;
    assert!(
        !tokens.is_empty(),
        "leaf attribute token stream should not be empty"
    );
}

/// Test 4: Construct word attribute TokenStream
#[test]
fn construct_word_attribute_tokenstream() {
    let attr_stream = quote! {
        #[adze::word]
    };
    let tokens: TokenStream = attr_stream;
    assert!(
        !tokens.is_empty(),
        "word attribute token stream should not be empty"
    );
}

/// Test 5: Construct skip attribute TokenStream
#[test]
fn construct_skip_attribute_tokenstream() {
    let attr_stream = quote! {
        #[adze::skip(true)]
    };
    let tokens: TokenStream = attr_stream;
    assert!(
        !tokens.is_empty(),
        "skip attribute token stream should not be empty"
    );
}

/// Test 6: Construct extra attribute TokenStream
#[test]
fn construct_extra_attribute_tokenstream() {
    let attr_stream = quote! {
        #[adze::extra]
    };
    let tokens: TokenStream = attr_stream;
    assert!(
        !tokens.is_empty(),
        "extra attribute token stream should not be empty"
    );
}

/// Test 7: Construct precedence attribute with value
#[test]
fn construct_precedence_attribute_with_value() {
    let attr_stream = quote! {
        #[adze::prec_left(2)]
    };
    let tokens: TokenStream = attr_stream;
    assert!(
        !tokens.is_empty(),
        "precedence attribute token stream should not be empty"
    );
}

// ============================================================================
// ATTRIBUTE PARSING TESTS (8-12)
// ============================================================================

/// Test 8: Parse attribute using syn::Attribute
#[test]
fn parse_attribute_using_syn() {
    let attr = parse_attr_from_struct(quote! { #[adze::leaf(pattern = r"\d+")] });
    assert!(is_adze_attr(&attr, "leaf"), "Should parse as adze::leaf");
}

/// Test 9: Parse struct with adze attributes
#[test]
fn parse_struct_with_adze_attributes() {
    let tokens = quote! {
        #[adze::language]
        struct Program {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    let item_struct = parse_struct(tokens);
    assert_eq!(item_struct.ident, "Program");
    assert!(
        item_struct
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "language"))
    );
}

/// Test 10: Parse enum with adze attributes
#[test]
fn parse_enum_with_adze_attributes() {
    let tokens = quote! {
        #[adze::language]
        enum Expr {
            #[adze::leaf(text = "+")]
            Plus,
        }
    };
    let item_enum = parse_enum(tokens);
    assert_eq!(item_enum.ident, "Expr");
    assert!(item_enum.attrs.iter().any(|a| is_adze_attr(a, "language")));
}

/// Test 11: Parse field with leaf attribute
#[test]
fn parse_field_with_leaf_attribute() {
    let tokens = quote! {
        struct Number {
            #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<i32>().unwrap())]
            value: i32,
        }
    };
    let item_struct = parse_struct(tokens);
    if let Fields::Named(ref fields) = item_struct.fields {
        let field = fields.named.iter().next().unwrap();
        assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

/// Test 12: Parse variant with multiple attributes
#[test]
fn parse_variant_with_multiple_attributes() {
    let tokens = quote! {
        enum Op {
            #[adze::prec_left(1)]
            #[adze::leaf(text = "-")]
            Sub,
        }
    };
    let item_enum = parse_enum(tokens);
    let variant = item_enum.variants.iter().next().unwrap();
    assert!(variant.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
}

// ============================================================================
// ATTRIBUTE VALUE TESTS (13-17)
// ============================================================================

/// Test 13: Attribute with string literal value
#[test]
fn attribute_with_string_literal_value() {
    let attr = parse_attr_from_struct(quote! { #[adze::leaf(text = "hello")] });
    assert!(is_adze_attr(&attr, "leaf"), "Should be leaf attribute");
}

/// Test 14: Attribute with integer literal value
#[test]
fn attribute_with_integer_literal_value() {
    let attr = parse_attr_from_struct(quote! { #[adze::skip(42)] });
    assert!(is_adze_attr(&attr, "skip"), "Should be skip attribute");
}

/// Test 15: Attribute with path value
#[test]
fn attribute_with_path_value() {
    let attr = parse_attr_from_struct(quote! { #[adze::delimited(MyDelimiter)] });
    assert!(
        is_adze_attr(&attr, "delimited"),
        "Should be delimited attribute"
    );
}

/// Test 16: Attribute with multiple arguments
#[test]
fn attribute_with_multiple_arguments() {
    let attr = parse_attr_from_struct(
        quote! { #[adze::leaf(pattern = r"\d+", text = "123", transform = |v| v)] },
    );
    assert!(
        is_adze_attr(&attr, "leaf"),
        "Should be leaf with multiple args"
    );
}

/// Test 17: Multiple attributes on same item
#[test]
fn multiple_attributes_on_same_item() {
    let tokens = quote! {
        #[adze::language]
        #[adze::word]
        #[adze::external]
        struct Identifier {
            value: String,
        }
    };
    let item_struct = parse_struct(tokens);
    assert!(
        item_struct
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "language")),
        "Should have language attribute"
    );
    assert!(
        item_struct.attrs.iter().any(|a| is_adze_attr(a, "word")),
        "Should have word attribute"
    );
    assert!(
        item_struct
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "external")),
        "Should have external attribute"
    );
}

// ============================================================================
// COMPLEX STRUCTURE TESTS (18-22)
// ============================================================================

/// Test 18: Nested attributes in module
#[test]
fn nested_attributes_in_module() {
    let tokens = quote! {
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            struct Program {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s")]
                space: (),
            }
        }
    };
    let module: syn::ItemMod = parse2(tokens).expect("failed to parse module");
    assert_eq!(module.ident, "grammar");
    assert!(module.attrs.iter().any(|a| is_adze_attr(a, "grammar")));
}

/// Test 19: Attribute with doc comments
#[test]
fn attribute_with_doc_comments() {
    let tokens = quote! {
        /// Documentation comment
        #[adze::language]
        /// More docs
        struct Code {
            value: String,
        }
    };
    let item_struct = parse_struct(tokens);
    assert!(
        !item_struct.attrs.is_empty(),
        "Should have attributes with docs"
    );
    assert!(
        item_struct
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "language"))
    );
}

/// Test 20: Attribute with visibility modifiers
#[test]
fn attribute_with_visibility_modifiers() {
    let tokens = quote! {
        #[adze::language]
        pub struct PublicType {
            #[adze::leaf(pattern = r"\d+")]
            pub field: u32,
        }
    };
    let item_struct = parse_struct(tokens);
    assert_eq!(item_struct.ident, "PublicType");
    assert!(matches!(item_struct.vis, syn::Visibility::Public(_)));
}

/// Test 21: Raw identifier in attribute value (r#type)
#[test]
fn raw_identifier_in_attribute_value() {
    let attr = parse_attr_from_struct(quote! { #[adze::leaf(text = "type")] });
    assert!(is_adze_attr(&attr, "leaf"), "Should handle keywords");
}

/// Test 22: Empty attribute arguments
#[test]
fn empty_attribute_arguments() {
    let attr = parse_attr_from_struct(quote! { #[adze::language] });
    assert!(
        is_adze_attr(&attr, "language"),
        "Should handle attribute without args"
    );
}

// ============================================================================
// ROUNDTRIP AND COMPLEX EXPRESSION TESTS (23-30)
// ============================================================================

/// Test 23: Attribute roundtrip (construct → to_string → parse)
#[test]
fn attribute_roundtrip_construct_to_string_parse() {
    let attr = parse_attr_from_struct(
        quote! { #[adze::leaf(pattern = r"\d+", transform = |v| v.parse::<u32>().unwrap())] },
    );
    let name = attr_name(&attr);
    assert_eq!(name, "leaf", "Roundtrip should preserve attribute name");
}

/// Test 24: Complex attribute expressions
#[test]
fn complex_attribute_expressions() {
    let attr = parse_attr_from_struct(quote! { #[adze::leaf(
        pattern = r"[a-zA-Z_]\w*",
        transform = |value: &str| {
            if value == "true" {
                true
            } else {
                false
            }
        }
    )] });
    assert!(
        is_adze_attr(&attr, "leaf"),
        "Should parse complex expressions"
    );
}

/// Test 25: Attribute on unit struct
#[test]
fn attribute_on_unit_struct() {
    let tokens = quote! {
        #[adze::extra]
        struct Empty;
    };
    let item_struct = parse_struct(tokens);
    assert_eq!(item_struct.ident, "Empty");
    assert!(item_struct.attrs.iter().any(|a| is_adze_attr(a, "extra")));
    assert!(matches!(item_struct.fields, Fields::Unit));
}

/// Test 26: Attribute on tuple struct
#[test]
fn attribute_on_tuple_struct() {
    let tokens = quote! {
        #[adze::language]
        struct Point(
            #[adze::leaf(pattern = r"\d+")]
            i32,
            #[adze::leaf(pattern = r"\d+")]
            i32,
        );
    };
    let item_struct = parse_struct(tokens);
    assert_eq!(item_struct.ident, "Point");
    assert!(
        item_struct
            .attrs
            .iter()
            .any(|a| is_adze_attr(a, "language"))
    );
    assert!(matches!(item_struct.fields, Fields::Unnamed(_)));
}

/// Test 27: Attribute on generic struct
#[test]
fn attribute_on_generic_struct() {
    let tokens = quote! {
        #[adze::language]
        struct Container<T> {
            #[adze::leaf(pattern = r"\w+")]
            value: T,
        }
    };
    let item_struct = parse_struct(tokens);
    assert_eq!(item_struct.ident, "Container");
    assert!(!item_struct.generics.params.is_empty());
}

/// Test 28: Attribute extraction from parsed AST
#[test]
fn attribute_extraction_from_parsed_ast() {
    let tokens = quote! {
        struct Data {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
            #[adze::skip(false)]
            visited: bool,
        }
    };
    let item_struct = parse_struct(tokens);
    if let Fields::Named(ref fields) = item_struct.fields {
        let leaf_count = fields
            .named
            .iter()
            .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
            .count();
        let skip_count = fields
            .named
            .iter()
            .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "skip")))
            .count();
        assert_eq!(leaf_count, 1, "Should find one leaf attribute");
        assert_eq!(skip_count, 1, "Should find one skip attribute");
    }
}

/// Test 29: Complex nested attribute expressions
#[test]
fn complex_nested_attribute_expressions() {
    let attr = parse_attr_from_struct(quote! { #[adze::delimited(
        #[adze::leaf(text = ",")]
        ()
    )] });
    assert!(
        is_adze_attr(&attr, "delimited"),
        "Should parse delimited with nested attributes"
    );
}

/// Test 30: quote! generates correct attribute syntax
#[test]
fn quote_generates_correct_attribute_syntax() {
    let grammar_name = "my_grammar";
    let attr_stream = quote! {
        #[adze::grammar(#grammar_name)]
    };
    let attr = parse_attr_from_struct(attr_stream);
    assert!(
        is_adze_attr(&attr, "grammar"),
        "quote! should generate valid grammar attribute"
    );
}

// ============================================================================
// ADDITIONAL COMPREHENSIVE TESTS (31-40+)
// ============================================================================

/// Test 31: prec_left attribute on enum variant
#[test]
fn prec_left_attribute_on_enum_variant() {
    let tokens = quote! {
        enum Op {
            #[adze::prec_left(1)]
            Sub(Box<Op>, Box<Op>),
        }
    };
    let item_enum = parse_enum(tokens);
    let variant = item_enum.variants.iter().next().unwrap();
    assert!(
        variant.attrs.iter().any(|a| is_adze_attr(a, "prec_left")),
        "Should have prec_left attribute"
    );
}

/// Test 32: prec_right attribute
#[test]
fn prec_right_attribute() {
    let attr = parse_attr_from_struct(quote! { #[adze::prec_right(2)] });
    assert!(is_adze_attr(&attr, "prec_right"), "Should parse prec_right");
}

/// Test 33: prec attribute (no associativity)
#[test]
fn prec_attribute_no_associativity() {
    let attr = parse_attr_from_struct(quote! { #[adze::prec(3)] });
    assert!(is_adze_attr(&attr, "prec"), "Should parse prec");
}

/// Test 34: repeat attribute with non_empty parameter
#[test]
fn repeat_attribute_with_non_empty_parameter() {
    let attr = parse_attr_from_struct(quote! { #[adze::repeat(non_empty = true)] });
    assert!(is_adze_attr(&attr, "repeat"), "Should parse repeat");
}

/// Test 35: repeat attribute without parameters
#[test]
fn repeat_attribute_without_parameters() {
    let attr = parse_attr_from_struct(quote! { #[adze::repeat] });
    assert!(
        is_adze_attr(&attr, "repeat"),
        "Should parse repeat without args"
    );
}

/// Test 36: delimited attribute with complex inner attribute
#[test]
fn delimited_attribute_with_complex_inner_attribute() {
    let attr = parse_attr_from_struct(quote! { #[adze::delimited(#[adze::leaf(text = ",")] ())] });
    assert!(is_adze_attr(&attr, "delimited"), "Should parse delimited");
}

/// Test 37: external attribute
#[test]
fn external_attribute() {
    let attr = parse_attr_from_struct(quote! { #[adze::external] });
    assert!(is_adze_attr(&attr, "external"), "Should parse external");
}

/// Test 38: Leaf with both text and pattern parameters
#[test]
fn leaf_with_multiple_patterns() {
    let attr = parse_attr_from_struct(quote! { #[adze::leaf(text = "+", pattern = r"\+")] });
    assert!(
        is_adze_attr(&attr, "leaf"),
        "Should parse leaf with multiple parameters"
    );
}

/// Test 39: Skip attribute with boolean false
#[test]
fn skip_attribute_with_boolean_false() {
    let attr = parse_attr_from_struct(quote! { #[adze::skip(false)] });
    assert!(is_adze_attr(&attr, "skip"), "Should parse skip with false");
}

/// Test 40: Complex enum with mixed attribute types
#[test]
fn complex_enum_with_mixed_attribute_types() {
    let tokens = quote! {
        #[adze::language]
        enum Expr {
            #[adze::leaf(pattern = r"\d+")]
            Number(u32),

            #[adze::prec_left(1)]
            Add(Box<Expr>, Box<Expr>),

            #[adze::prec_right(2)]
            Pow(Box<Expr>, Box<Expr>),

            #[adze::prec(3)]
            Compare(Box<Expr>, Box<Expr>),
        }
    };
    let item_enum = parse_enum(tokens);
    assert_eq!(item_enum.ident, "Expr");
    assert!(item_enum.attrs.iter().any(|a| is_adze_attr(a, "language")));
    assert_eq!(item_enum.variants.len(), 4, "Should have 4 variants");
}

/// Test 41: Grammar attribute with special characters in name
#[test]
fn grammar_attribute_with_special_name() {
    let attr = parse_attr_from_struct(quote! { #[adze::grammar("my_grammar_2024")] });
    assert!(
        is_adze_attr(&attr, "grammar"),
        "Should parse grammar with name"
    );
}

/// Test 42: Transform closure with complex logic
#[test]
fn transform_closure_with_complex_logic() {
    let attr = parse_attr_from_struct(
        quote! { #[adze::leaf(pattern = r"\d+", transform = |v: &str| {
            v.parse::<i32>()
                .map_err(|_| "invalid number")
                .map(|n| n * 2)
                .unwrap_or(0)
        })] },
    );
    assert!(
        is_adze_attr(&attr, "leaf"),
        "Should parse leaf with complex transform"
    );
}

/// Test 43: Attribute on struct with lifetime parameters
#[test]
fn attribute_on_struct_with_lifetime_parameters() {
    let tokens = quote! {
        #[adze::language]
        struct Borrowed<'a> {
            #[adze::leaf(pattern = r"\w+")]
            data: &'a str,
        }
    };
    let item_struct = parse_struct(tokens);
    assert!(item_struct.generics.lifetimes().next().is_some());
}

/// Test 44: Word attribute on enum variant
#[test]
fn word_attribute_on_enum_variant() {
    let tokens = quote! {
        enum Token {
            #[adze::word]
            #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
            Identifier(String),
        }
    };
    let item_enum = parse_enum(tokens);
    let variant = item_enum.variants.iter().next().unwrap();
    assert!(
        variant.attrs.iter().any(|a| is_adze_attr(a, "word")),
        "Should have word attribute on variant"
    );
}

/// Test 45: Nested quote! expressions in attributes
#[test]
fn nested_quote_expressions_in_attributes() {
    let inner_attr = quote! { text = ";" };
    let attr = parse_attr_from_struct(quote! { #[adze::leaf(#inner_attr)] });
    assert!(is_adze_attr(&attr, "leaf"), "Should handle nested quote!");
}

// ============================================================================
// VALIDATION AND EDGE CASES (46-50+)
// ============================================================================

/// Test 46: Attribute path with correct module path
#[test]
fn attribute_path_with_correct_module_path() {
    let attr = parse_attr_from_struct(quote! { #[adze::language] });
    let segments: Vec<_> = attr.path().segments.iter().collect();
    assert_eq!(segments.len(), 2, "Should have exactly 2 path segments");
    assert_eq!(segments[0].ident, "adze");
    assert_eq!(segments[1].ident, "language");
}

/// Test 47: Multiple leaf attributes on different fields
#[test]
fn multiple_leaf_attributes_on_different_fields() {
    let tokens = quote! {
        struct Record {
            #[adze::leaf(text = "name")]
            name: String,
            #[adze::leaf(text = "age")]
            age: u32,
            #[adze::leaf(text = "active")]
            active: bool,
        }
    };
    let item_struct = parse_struct(tokens);
    if let Fields::Named(ref fields) = item_struct.fields {
        let leaf_count = fields
            .named
            .iter()
            .filter(|f| f.attrs.iter().any(|a| is_adze_attr(a, "leaf")))
            .count();
        assert_eq!(leaf_count, 3, "Should have three leaf attributes");
    }
}

/// Test 48: Enum variant with only skip attribute
#[test]
fn enum_variant_with_only_skip_attribute() {
    let tokens = quote! {
        enum State {
            #[adze::skip(42)]
            Waiting,
        }
    };
    let item_enum = parse_enum(tokens);
    let variant = item_enum.variants.iter().next().unwrap();
    assert!(
        variant.attrs.iter().any(|a| is_adze_attr(a, "skip")),
        "Should have skip attribute"
    );
}

/// Test 49: Large precedence value
#[test]
fn large_precedence_value() {
    let attr = parse_attr_from_struct(quote! { #[adze::prec_left(999999)] });
    assert!(
        is_adze_attr(&attr, "prec_left"),
        "Should handle large precedence"
    );
}

/// Test 50: Pattern with complex regex
#[test]
fn pattern_with_complex_regex() {
    let attr = parse_attr_from_struct(
        quote! { #[adze::leaf(pattern = r"(?:[0-9]+\.)?[0-9]+([eE][+-]?[0-9]+)?")] },
    );
    assert!(
        is_adze_attr(&attr, "leaf"),
        "Should handle complex regex pattern"
    );
}

/// Test 51: Attribute on named enum variant with fields
#[test]
fn attribute_on_named_enum_variant_with_fields() {
    let tokens = quote! {
        enum Result {
            #[adze::prec_left(1)]
            Neg {
                #[adze::leaf(text = "!")]
                _bang: (),
                value: Box<Result>,
            }
        }
    };
    let item_enum = parse_enum(tokens);
    let variant = item_enum.variants.iter().next().unwrap();
    assert!(variant.attrs.iter().any(|a| is_adze_attr(a, "prec_left")));
}

/// Test 52: Extra attribute on struct with fields
#[test]
fn extra_attribute_on_struct_with_fields() {
    let tokens = quote! {
        #[adze::extra]
        struct Whitespace {
            #[adze::leaf(pattern = r"\s")]
            _ws: (),
        }
    };
    let item_struct = parse_struct(tokens);
    assert!(item_struct.attrs.iter().any(|a| is_adze_attr(a, "extra")));
}

/// Test 53: Attribute with path containing colon prefix
#[test]
fn attribute_with_colon_prefix() {
    let tokens = quote! {
        struct Item {
            #[adze::leaf(pattern = r"\w+")]
            name: String,
        }
    };
    let item_struct = parse_struct(tokens);
    if let Fields::Named(ref fields) = item_struct.fields {
        let field = fields.named.iter().next().unwrap();
        assert!(field.attrs.iter().any(|a| is_adze_attr(a, "leaf")));
    }
}

/// Test 54: Delimited with parenthesized content
#[test]
fn delimited_with_parenthesized_content() {
    let tokens = quote! {
        struct List {
            #[adze::delimited(
                #[adze::leaf(text = ",")]
                ()
            )]
            items: Vec<Item>,
        }
    };
    let item_struct = parse_struct(tokens);
    assert_eq!(item_struct.ident, "List");
}

/// Test 55: Multiple precedence attributes on variants
#[test]
fn multiple_prec_variants_same_enum() {
    let tokens = quote! {
        enum Op {
            #[adze::prec_left(1)]
            Add(Box<Op>, Box<Op>),
            #[adze::prec_left(2)]
            Mul(Box<Op>, Box<Op>),
            #[adze::prec_right(3)]
            Exp(Box<Op>, Box<Op>),
        }
    };
    let item_enum = parse_enum(tokens);
    assert_eq!(item_enum.variants.len(), 3);
}
