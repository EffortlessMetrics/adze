#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for token patterns (string and regex) in adze-ir.

use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

// ---------------------------------------------------------------------------
// String token patterns
// ---------------------------------------------------------------------------

#[test]
fn string_pattern_keyword() {
    let pat = TokenPattern::String("if".into());
    match &pat {
        TokenPattern::String(s) => assert_eq!(s, "if"),
        _ => panic!("expected String variant"),
    }
}

#[test]
fn string_pattern_operator() {
    let pat = TokenPattern::String("+=".into());
    if let TokenPattern::String(s) = &pat {
        assert_eq!(s.len(), 2);
        assert_eq!(s, "+=");
    } else {
        panic!("expected String");
    }
}

#[test]
fn string_pattern_single_char() {
    let pat = TokenPattern::String(";".into());
    assert_eq!(pat, TokenPattern::String(";".into()));
}

// ---------------------------------------------------------------------------
// Regex token patterns
// ---------------------------------------------------------------------------

#[test]
fn regex_pattern_digits() {
    let pat = TokenPattern::Regex(r"\d+".into());
    match &pat {
        TokenPattern::Regex(r) => assert_eq!(r, r"\d+"),
        _ => panic!("expected Regex variant"),
    }
}

#[test]
fn regex_pattern_identifier() {
    let pat = TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".into());
    if let TokenPattern::Regex(r) = &pat {
        assert!(r.starts_with('['));
        assert!(r.contains("a-z"));
    } else {
        panic!("expected Regex");
    }
}

#[test]
fn regex_pattern_float() {
    let pat = TokenPattern::Regex(r"[0-9]+(\.[0-9]+)?".into());
    if let TokenPattern::Regex(r) = &pat {
        assert!(r.contains(r"\."));
        assert!(r.contains('?'));
    } else {
        panic!("expected Regex");
    }
}

// ---------------------------------------------------------------------------
// Special regex characters
// ---------------------------------------------------------------------------

#[test]
fn regex_special_chars_anchors() {
    let pat = TokenPattern::Regex(r"^start$".into());
    if let TokenPattern::Regex(r) = &pat {
        assert!(r.starts_with('^'));
        assert!(r.ends_with('$'));
    } else {
        panic!("expected Regex");
    }
}

#[test]
fn regex_special_chars_quantifiers() {
    let pat = TokenPattern::Regex(r"a+b*c?d{2,5}".into());
    if let TokenPattern::Regex(r) = &pat {
        assert!(r.contains('+'));
        assert!(r.contains('*'));
        assert!(r.contains('?'));
        assert!(r.contains("{2,5}"));
    } else {
        panic!("expected Regex");
    }
}

#[test]
fn regex_special_chars_alternation_and_groups() {
    let pat = TokenPattern::Regex(r"(true|false|null)".into());
    if let TokenPattern::Regex(r) = &pat {
        assert!(r.contains('|'));
        assert!(r.starts_with('('));
        assert!(r.ends_with(')'));
    } else {
        panic!("expected Regex");
    }
}

// ---------------------------------------------------------------------------
// Unicode in token patterns
// ---------------------------------------------------------------------------

#[test]
fn string_pattern_unicode_keyword() {
    let pat = TokenPattern::String("λ".into());
    if let TokenPattern::String(s) = &pat {
        assert_eq!(s, "λ");
        assert_eq!(s.len(), 2); // λ is 2 bytes in UTF-8
    } else {
        panic!("expected String");
    }
}

#[test]
fn string_pattern_unicode_cjk() {
    let pat = TokenPattern::String("変数".into());
    if let TokenPattern::String(s) = &pat {
        assert_eq!(s.chars().count(), 2);
    } else {
        panic!("expected String");
    }
}

#[test]
fn regex_pattern_unicode_class() {
    let pat = TokenPattern::Regex(r"\p{L}+".into());
    if let TokenPattern::Regex(r) = &pat {
        assert!(r.contains(r"\p{L}"));
    } else {
        panic!("expected Regex");
    }
}

#[test]
fn string_pattern_emoji() {
    let pat = TokenPattern::String("🚀".into());
    let cloned = pat.clone();
    assert_eq!(pat, cloned);
    if let TokenPattern::String(s) = &pat {
        assert_eq!(s, "🚀");
    } else {
        panic!("expected String");
    }
}

// ---------------------------------------------------------------------------
// Empty patterns
// ---------------------------------------------------------------------------

#[test]
fn empty_string_pattern() {
    let pat = TokenPattern::String(String::new());
    if let TokenPattern::String(s) = &pat {
        assert!(s.is_empty());
    } else {
        panic!("expected String");
    }
}

#[test]
fn empty_regex_pattern() {
    let pat = TokenPattern::Regex(String::new());
    if let TokenPattern::Regex(r) = &pat {
        assert!(r.is_empty());
    } else {
        panic!("expected Regex");
    }
}

#[test]
fn grammar_detects_empty_string_token() {
    let mut g = Grammar::new("test".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "BAD".into(),
            pattern: TokenPattern::String(String::new()),
            fragile: false,
        },
    );
    let result = g.check_empty_terminals();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors[0].contains("empty string pattern"));
}

#[test]
fn grammar_detects_empty_regex_token() {
    let mut g = Grammar::new("test".into());
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: "BAD_RE".into(),
            pattern: TokenPattern::Regex(String::new()),
            fragile: false,
        },
    );
    let result = g.check_empty_terminals();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors[0].contains("empty regex pattern"));
}

// ---------------------------------------------------------------------------
// Token ID assignment
// ---------------------------------------------------------------------------

#[test]
fn token_id_assignment_sequential() {
    let mut g = Grammar::new("arith".into());
    for i in 0..5 {
        g.tokens.insert(
            SymbolId(i),
            Token {
                name: format!("T{i}"),
                pattern: TokenPattern::String(format!("t{i}")),
                fragile: false,
            },
        );
    }
    assert_eq!(g.tokens.len(), 5);
    for i in 0..5 {
        assert!(g.tokens.contains_key(&SymbolId(i)));
    }
}

#[test]
fn token_id_assignment_sparse() {
    let mut g = Grammar::new("sparse".into());
    let ids = [SymbolId(0), SymbolId(10), SymbolId(100)];
    for (idx, id) in ids.iter().enumerate() {
        g.tokens.insert(
            *id,
            Token {
                name: format!("T{idx}"),
                pattern: TokenPattern::String(format!("{idx}")),
                fragile: false,
            },
        );
    }
    assert_eq!(g.tokens.len(), 3);
    assert!(g.tokens.contains_key(&SymbolId(100)));
    assert!(!g.tokens.contains_key(&SymbolId(50)));
}

#[test]
fn token_id_overwrite_replaces() {
    let mut g = Grammar::new("test".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "OLD".into(),
            pattern: TokenPattern::String("old".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "NEW".into(),
            pattern: TokenPattern::String("new".into()),
            fragile: false,
        },
    );
    assert_eq!(g.tokens.len(), 1);
    assert_eq!(g.tokens[&SymbolId(1)].name, "NEW");
}

// ---------------------------------------------------------------------------
// Token visibility (named vs anonymous)
// ---------------------------------------------------------------------------

#[test]
fn hidden_token_underscore_prefix() {
    // Tokens whose name starts with '_' are not visible per build_registry
    let mut g = Grammar::new("test".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "_ws".into(),
            pattern: TokenPattern::Regex(r"\s+".into()),
            fragile: false,
        },
    );
    let registry = g.build_registry();
    let id = registry.get_id("_ws");
    assert!(id.is_some());
    let meta = registry.get_metadata(id.unwrap()).unwrap();
    assert!(
        !meta.visible,
        "underscore-prefixed token should not be visible"
    );
}

#[test]
fn visible_token_normal_name() {
    let mut g = Grammar::new("test".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "PLUS".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    let registry = g.build_registry();
    let id = registry.get_id("PLUS").expect("PLUS should be registered");
    let meta = registry.get_metadata(id).unwrap();
    assert!(meta.visible);
}

#[test]
fn extra_token_hidden_in_registry() {
    let mut g = Grammar::new("test".into());
    let ws_id = SymbolId(1);
    g.tokens.insert(
        ws_id,
        Token {
            name: "WS".into(),
            pattern: TokenPattern::Regex(r"\s+".into()),
            fragile: false,
        },
    );
    g.extras.push(ws_id);
    let registry = g.build_registry();
    let id = registry.get_id("WS").expect("WS should be registered");
    let meta = registry.get_metadata(id).unwrap();
    assert!(meta.hidden, "extra token should be hidden");
}

// ---------------------------------------------------------------------------
// Token in grammar rules
// ---------------------------------------------------------------------------

#[test]
fn token_referenced_in_rule_rhs() {
    let mut g = Grammar::new("calc".into());
    let plus_id = SymbolId(1);
    let expr_id = SymbolId(10);
    g.tokens.insert(
        plus_id,
        Token {
            name: "PLUS".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(expr_id, "expr".into());
    let rule = Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(plus_id),
            Symbol::NonTerminal(expr_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    g.add_rule(rule);
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    assert_eq!(rules.len(), 1);
    assert!(rules[0].rhs.contains(&Symbol::Terminal(plus_id)));
}

#[test]
fn multiple_tokens_in_grammar() {
    let mut g = Grammar::new("json".into());
    let tokens = [
        (SymbolId(1), "LBRACE", "{"),
        (SymbolId(2), "RBRACE", "}"),
        (SymbolId(3), "COLON", ":"),
        (SymbolId(4), "COMMA", ","),
    ];
    for (id, name, pat) in &tokens {
        g.tokens.insert(
            *id,
            Token {
                name: (*name).into(),
                pattern: TokenPattern::String((*pat).into()),
                fragile: false,
            },
        );
    }
    assert_eq!(g.tokens.len(), 4);
    for (id, _name, pat) in &tokens {
        let tok = &g.tokens[id];
        assert_eq!(tok.pattern, TokenPattern::String((*pat).into()));
    }
}

// ---------------------------------------------------------------------------
// Token serde roundtrip
// ---------------------------------------------------------------------------

#[test]
fn serde_string_token_roundtrip() {
    let tok = Token {
        name: "SEMICOLON".into(),
        pattern: TokenPattern::String(";".into()),
        fragile: true,
    };
    let json = serde_json::to_string(&tok).unwrap();
    let back: Token = serde_json::from_str(&json).unwrap();
    assert_eq!(tok, back);
}

#[test]
fn serde_regex_token_roundtrip() {
    let tok = Token {
        name: "IDENT".into(),
        pattern: TokenPattern::Regex(r"[a-zA-Z_]\w*".into()),
        fragile: false,
    };
    let json = serde_json::to_string(&tok).unwrap();
    let back: Token = serde_json::from_str(&json).unwrap();
    assert_eq!(tok, back);
}

#[test]
fn serde_token_pattern_string_vs_regex_distinct() {
    let s = TokenPattern::String("x".into());
    let r = TokenPattern::Regex("x".into());
    let js = serde_json::to_string(&s).unwrap();
    let jr = serde_json::to_string(&r).unwrap();
    // Serialized forms must differ so deserialization recovers the correct variant
    assert_ne!(js, jr);
}

#[test]
fn serde_unicode_token_roundtrip() {
    let tok = Token {
        name: "ARROW".into(),
        pattern: TokenPattern::String("→".into()),
        fragile: false,
    };
    let json = serde_json::to_string(&tok).unwrap();
    let back: Token = serde_json::from_str(&json).unwrap();
    assert_eq!(tok, back);
    if let TokenPattern::String(s) = &back.pattern {
        assert_eq!(s, "→");
    } else {
        panic!("expected String pattern after roundtrip");
    }
}

#[test]
fn serde_special_regex_roundtrip() {
    let pat = TokenPattern::Regex(r#"\\[nrt"\\]"#.into());
    let json = serde_json::to_string(&pat).unwrap();
    let back: TokenPattern = serde_json::from_str(&json).unwrap();
    assert_eq!(pat, back);
}

// ---------------------------------------------------------------------------
// Token comparison
// ---------------------------------------------------------------------------

#[test]
fn token_equality_same_fields() {
    let a = Token {
        name: "X".into(),
        pattern: TokenPattern::String("x".into()),
        fragile: false,
    };
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn token_ne_different_pattern_variant() {
    let a = Token {
        name: "X".into(),
        pattern: TokenPattern::String("x".into()),
        fragile: false,
    };
    let b = Token {
        name: "X".into(),
        pattern: TokenPattern::Regex("x".into()),
        fragile: false,
    };
    assert_ne!(a, b);
}

#[test]
fn token_ne_different_name() {
    let a = Token {
        name: "A".into(),
        pattern: TokenPattern::String("+".into()),
        fragile: false,
    };
    let b = Token {
        name: "B".into(),
        pattern: TokenPattern::String("+".into()),
        fragile: false,
    };
    assert_ne!(a, b);
}

#[test]
fn token_ne_different_fragile() {
    let a = Token {
        name: "X".into(),
        pattern: TokenPattern::String("x".into()),
        fragile: false,
    };
    let b = Token {
        name: "X".into(),
        pattern: TokenPattern::String("x".into()),
        fragile: true,
    };
    assert_ne!(a, b);
}

#[test]
fn token_pattern_ne_string_vs_regex_same_content() {
    let s = TokenPattern::String(r"\d+".into());
    let r = TokenPattern::Regex(r"\d+".into());
    assert_ne!(s, r, "String and Regex with identical content must differ");
}
