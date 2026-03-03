#![allow(clippy::needless_range_loop)]
//! Property-based tests for NODE_TYPES JSON generation in adze-tablegen.
//!
//! Focuses on JSON validity, named type classification, structural invariants,
//! children/fields info, determinism, various grammar shapes, and internal-rule
//! exclusion — all through the public `NodeTypesGenerator` API.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use adze_tablegen::NodeTypesGenerator;
use proptest::prelude::*;
use serde_json::Value;

// ===========================================================================
// Helpers
// ===========================================================================

/// Parse generated JSON and return the array of node type objects.
fn generate_types(grammar: &Grammar) -> Vec<Value> {
    let g = NodeTypesGenerator::new(grammar);
    let json = g.generate().expect("generate must succeed");
    let v: Value = serde_json::from_str(&json).expect("not valid JSON");
    v.as_array().expect("not an array").clone()
}

/// Collect all `"type"` strings from node-type entries.
fn type_names(entries: &[Value]) -> Vec<String> {
    entries
        .iter()
        .filter_map(|e| e.get("type").and_then(Value::as_str).map(String::from))
        .collect()
}

/// Build a grammar with N visible rules, M hidden (_-prefixed) rules,
/// some string tokens, and some regex tokens.
fn make_grammar(
    visible: &[String],
    hidden: &[String],
    str_toks: &[(String, String)],
    regex_toks: &[(String, String)],
    fields: &[String],
) -> Grammar {
    let mut g = Grammar::new("proptest".to_string());
    let mut next: u16 = 0;

    // Regex tokens
    let mut regex_ids = Vec::new();
    for (name, pat) in regex_toks {
        let id = SymbolId(next);
        next += 1;
        g.tokens.insert(
            id,
            Token {
                name: name.clone(),
                pattern: TokenPattern::Regex(pat.clone()),
                fragile: false,
            },
        );
        regex_ids.push(id);
    }

    // String tokens
    let mut str_ids = Vec::new();
    for (name, pat) in str_toks {
        let id = SymbolId(next);
        next += 1;
        g.tokens.insert(
            id,
            Token {
                name: name.clone(),
                pattern: TokenPattern::String(pat.clone()),
                fragile: false,
            },
        );
        str_ids.push(id);
    }

    // Fields
    let fids: Vec<FieldId> = fields
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let fid = FieldId(i as u16);
            g.fields.insert(fid, name.clone());
            fid
        })
        .collect();

    let default_term = regex_ids.first().or(str_ids.first()).copied();
    let mut pid: u16 = 0;

    // Visible rules
    for name in visible {
        let id = SymbolId(next);
        next += 1;
        g.rule_names.insert(id, name.clone());

        let (rule_fields, rhs) = match (default_term, fids.is_empty()) {
            (Some(tid), false) => {
                let mut syms = Vec::new();
                let mut fps = Vec::new();
                for (pos, fid) in fids.iter().enumerate() {
                    syms.push(Symbol::Terminal(tid));
                    fps.push((*fid, pos));
                }
                (fps, syms)
            }
            (Some(tid), true) => (vec![], vec![Symbol::Terminal(tid)]),
            _ => (vec![], vec![Symbol::Epsilon]),
        };

        g.add_rule(Rule {
            lhs: id,
            rhs,
            precedence: None,
            associativity: None,
            fields: rule_fields,
            production_id: ProductionId(pid),
        });
        pid += 1;
    }

    // Hidden rules
    for name in hidden {
        let id = SymbolId(next);
        next += 1;
        g.rule_names.insert(id, name.clone());
        let rhs = default_term
            .map(|tid| vec![Symbol::Terminal(tid)])
            .unwrap_or_else(|| vec![Symbol::Epsilon]);
        g.add_rule(Rule {
            lhs: id,
            rhs,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(pid),
        });
        pid += 1;
    }

    g
}

fn dedup(mut v: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    v.retain(|s| seen.insert(s.clone()));
    v
}

fn dedup_pairs(mut v: Vec<(String, String)>) -> Vec<(String, String)> {
    let mut seen = std::collections::HashSet::new();
    v.retain(|(_, p)| seen.insert(p.clone()));
    v
}

// ===========================================================================
// Strategies
// ===========================================================================

fn vis_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9]{0,8}".prop_filter("nonempty", |s| !s.is_empty())
}

fn hid_name() -> impl Strategy<Value = String> {
    "_[a-z][a-z0-9]{0,6}".prop_filter("len>1", |s| s.len() > 1)
}

fn field_name() -> impl Strategy<Value = String> {
    "[a-z][a-z_]{0,6}".prop_filter("nonempty", |s| !s.is_empty())
}

fn str_tok() -> impl Strategy<Value = (String, String)> {
    prop_oneof![
        Just(("plus".into(), "+".into())),
        Just(("minus".into(), "-".into())),
        Just(("star".into(), "*".into())),
        Just(("semi".into(), ";".into())),
        Just(("eq".into(), "=".into())),
        Just(("dot".into(), ".".into())),
    ]
}

fn re_tok() -> impl Strategy<Value = (String, String)> {
    prop_oneof![
        Just(("number".into(), r"\d+".into())),
        Just(("ident".into(), r"[a-z]+".into())),
    ]
}

type GMeta = (
    Grammar,
    Vec<String>,           // visible
    Vec<String>,           // hidden
    Vec<(String, String)>, // str_toks
    Vec<(String, String)>, // re_toks
    Vec<String>,           // fields
);

fn grammar_strat() -> impl Strategy<Value = GMeta> {
    (
        prop::collection::vec(vis_name(), 0..5),
        prop::collection::vec(hid_name(), 0..3),
        prop::collection::vec(str_tok(), 0..4),
        prop::collection::vec(re_tok(), 0..3),
        prop::collection::vec(field_name(), 0..3),
    )
        .prop_map(|(v, h, st, rt, f)| {
            let v = dedup(v);
            let h = dedup(h);
            let st = dedup_pairs(st);
            let rt = dedup_pairs(rt);
            let f = dedup(f);
            let g = make_grammar(&v, &h, &st, &rt, &f);
            (g, v, h, st, rt, f)
        })
}

// ===========================================================================
// Property tests — 25 proptest + 5 deterministic = 30 tests
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // -----------------------------------------------------------------------
    // 1. NODE_TYPES JSON is valid JSON
    // -----------------------------------------------------------------------
    #[test]
    fn json_always_valid((grammar, ..) in grammar_strat()) {
        let g = NodeTypesGenerator::new(&grammar);
        let json = g.generate().expect("generate ok");
        let _: Value = serde_json::from_str(&json).expect("valid JSON");
    }

    // -----------------------------------------------------------------------
    // 2. NODE_TYPES contains named types for every visible rule
    // -----------------------------------------------------------------------
    #[test]
    fn named_types_present_for_visible_rules((grammar, visible, ..) in grammar_strat()) {
        let entries = generate_types(&grammar);
        let names = type_names(&entries);
        for v in &visible {
            prop_assert!(
                names.contains(v),
                "visible rule '{}' missing; got {:?}", v, names
            );
        }
    }

    // -----------------------------------------------------------------------
    // 3. NODE_TYPES type structure: every entry has `type` (string) + `named` (bool)
    // -----------------------------------------------------------------------
    #[test]
    fn every_entry_has_type_and_named((grammar, ..) in grammar_strat()) {
        for e in generate_types(&grammar) {
            prop_assert!(e.get("type").and_then(Value::as_str).is_some());
            prop_assert!(e.get("named").and_then(Value::as_bool).is_some());
        }
    }

    // -----------------------------------------------------------------------
    // 4. NODE_TYPES children info: `children` is an object with expected keys
    // -----------------------------------------------------------------------
    #[test]
    fn children_is_object_with_correct_keys((grammar, ..) in grammar_strat()) {
        for e in generate_types(&grammar) {
            if let Some(ch) = e.get("children") {
                prop_assert!(ch.is_object(), "children must be object");
                if let Some(m) = ch.get("multiple") {
                    prop_assert!(m.is_boolean());
                }
                if let Some(r) = ch.get("required") {
                    prop_assert!(r.is_boolean());
                }
                if let Some(ts) = ch.get("types") {
                    prop_assert!(ts.is_array());
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 5. NODE_TYPES fields info: `fields` is an object mapping field names
    // -----------------------------------------------------------------------
    #[test]
    fn fields_is_object_when_present((grammar, ..) in grammar_strat()) {
        for e in generate_types(&grammar) {
            if let Some(f) = e.get("fields") {
                prop_assert!(f.is_object(), "fields must be an object");
                for (_, fv) in f.as_object().unwrap() {
                    prop_assert!(fv.get("types").and_then(Value::as_array).is_some());
                    prop_assert!(fv.get("multiple").and_then(Value::as_bool).is_some());
                    prop_assert!(fv.get("required").and_then(Value::as_bool).is_some());
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 6. NODE_TYPES determinism: same grammar → semantically identical JSON
    // -----------------------------------------------------------------------
    #[test]
    fn deterministic_output((grammar, ..) in grammar_strat()) {
        let g = NodeTypesGenerator::new(&grammar);
        let a: Value = serde_json::from_str(&g.generate().unwrap()).unwrap();
        let b: Value = serde_json::from_str(&g.generate().unwrap()).unwrap();
        prop_assert_eq!(a, b, "output must be semantically identical");
    }

    // -----------------------------------------------------------------------
    // 7. NODE_TYPES excludes internal (_-prefixed) rules
    // -----------------------------------------------------------------------
    #[test]
    fn internal_rules_excluded((grammar, _, hidden, ..) in grammar_strat()) {
        let names = type_names(&generate_types(&grammar));
        for h in &hidden {
            prop_assert!(
                !names.iter().any(|n| n == h),
                "internal rule '{}' must not appear", h
            );
        }
    }

    // -----------------------------------------------------------------------
    // 8. Output is sorted alphabetically by type name
    // -----------------------------------------------------------------------
    #[test]
    fn output_sorted((grammar, ..) in grammar_strat()) {
        let names = type_names(&generate_types(&grammar));
        for w in names.windows(2) {
            prop_assert!(w[0] <= w[1], "not sorted: '{}' > '{}'", w[0], w[1]);
        }
    }

    // -----------------------------------------------------------------------
    // 9. No duplicate type names
    // -----------------------------------------------------------------------
    #[test]
    fn no_duplicate_types((grammar, ..) in grammar_strat()) {
        let names = type_names(&generate_types(&grammar));
        let set: std::collections::HashSet<_> = names.iter().collect();
        prop_assert_eq!(names.len(), set.len());
    }

    // -----------------------------------------------------------------------
    // 10. String tokens are unnamed (named=false)
    // -----------------------------------------------------------------------
    #[test]
    fn string_tokens_unnamed((grammar, _, _, str_toks, ..) in grammar_strat()) {
        let entries = generate_types(&grammar);
        for (_, pat) in &str_toks {
            if let Some(e) = entries.iter().find(|e| e["type"].as_str() == Some(pat)) {
                prop_assert_eq!(e["named"].as_bool(), Some(false));
            }
        }
    }

    // -----------------------------------------------------------------------
    // 11. Visible rules are named (named=true)
    // -----------------------------------------------------------------------
    #[test]
    fn visible_rules_named_true((grammar, visible, ..) in grammar_strat()) {
        let entries = generate_types(&grammar);
        for v in &visible {
            if let Some(e) = entries.iter().find(|e| e["type"].as_str() == Some(v.as_str())) {
                prop_assert_eq!(e["named"].as_bool(), Some(true));
            }
        }
    }

    // -----------------------------------------------------------------------
    // 12. Top-level value is always an array
    // -----------------------------------------------------------------------
    #[test]
    fn top_level_is_array((grammar, ..) in grammar_strat()) {
        let g = NodeTypesGenerator::new(&grammar);
        let json = g.generate().unwrap();
        let v: Value = serde_json::from_str(&json).unwrap();
        prop_assert!(v.is_array());
    }

    // -----------------------------------------------------------------------
    // 13. Only allowed keys in entries
    // -----------------------------------------------------------------------
    #[test]
    fn only_known_keys((grammar, ..) in grammar_strat()) {
        let allowed: std::collections::HashSet<&str> =
            ["type", "named", "fields", "children", "subtypes"].into_iter().collect();
        for e in generate_types(&grammar) {
            if let Some(obj) = e.as_object() {
                for k in obj.keys() {
                    prop_assert!(allowed.contains(k.as_str()), "unexpected key '{}'", k);
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 14. `type` field is always a non-empty string
    // -----------------------------------------------------------------------
    #[test]
    fn type_field_nonempty((grammar, ..) in grammar_strat()) {
        for e in generate_types(&grammar) {
            let t = e["type"].as_str().unwrap_or("");
            prop_assert!(!t.is_empty());
        }
    }

    // -----------------------------------------------------------------------
    // 15. Field names in output come from grammar field definitions
    // -----------------------------------------------------------------------
    #[test]
    fn field_names_subset_of_grammar((grammar, _, _, _, _, fields) in grammar_strat()) {
        let gfields: std::collections::HashSet<&str> = fields.iter().map(String::as_str).collect();
        for e in generate_types(&grammar) {
            if let Some(fo) = e.get("fields").and_then(Value::as_object) {
                for k in fo.keys() {
                    prop_assert!(gfields.contains(k.as_str()),
                        "field '{}' not in grammar", k);
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 16. Multiple rules for one symbol → single entry
    // -----------------------------------------------------------------------
    #[test]
    fn multiple_rules_single_entry(name in vis_name()) {
        let mut g = Grammar::new("multi".into());
        let tid = SymbolId(0);
        g.tokens.insert(tid, Token {
            name: "t".into(),
            pattern: TokenPattern::Regex(r"\w+".into()),
            fragile: false,
        });
        let rid = SymbolId(10);
        g.rule_names.insert(rid, name.clone());
        for p in 0..3u16 {
            g.add_rule(Rule {
                lhs: rid,
                rhs: vec![Symbol::Terminal(tid)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(p),
            });
        }
        let entries = generate_types(&g);
        let cnt = entries.iter().filter(|e| e["type"].as_str() == Some(&name)).count();
        prop_assert_eq!(cnt, 1);
    }

    // -----------------------------------------------------------------------
    // 17. Adding tokens preserves existing rule entries
    // -----------------------------------------------------------------------
    #[test]
    fn adding_tokens_preserves_rules(
        vis in prop::collection::vec(vis_name(), 1..4),
        extra in prop::collection::vec(str_tok(), 0..4),
    ) {
        let vis = dedup(vis);
        let extra = dedup_pairs(extra);
        let base = make_grammar(&vis, &[], &[], &[("tok".into(), r"\w+".into())], &[]);
        let ext = make_grammar(&vis, &[], &extra, &[("tok".into(), r"\w+".into())], &[]);
        let base_named: std::collections::HashSet<_> = generate_types(&base)
            .iter()
            .filter(|e| e["named"].as_bool() == Some(true))
            .filter_map(|e| e["type"].as_str().map(String::from))
            .collect();
        let ext_named: std::collections::HashSet<_> = generate_types(&ext)
            .iter()
            .filter(|e| e["named"].as_bool() == Some(true))
            .filter_map(|e| e["type"].as_str().map(String::from))
            .collect();
        for n in &base_named {
            prop_assert!(ext_named.contains(n), "rule '{}' disappeared", n);
        }
    }

    // -----------------------------------------------------------------------
    // 18. GrammarBuilder grammars yield valid JSON
    // -----------------------------------------------------------------------
    #[test]
    fn builder_grammar_valid_json(count in 1usize..6) {
        let mut b = GrammarBuilder::new("btest").token("NUM", r"\d+");
        for i in 0..count {
            b = b.rule(&format!("r{}", i), vec!["NUM"]);
        }
        let g = b.start("r0").build();
        let ntg = NodeTypesGenerator::new(&g);
        let json = ntg.generate().expect("ok");
        let _: Value = serde_json::from_str(&json).expect("valid JSON");
    }

    // -----------------------------------------------------------------------
    // 19. Children types entries have `type` and `named` keys
    // -----------------------------------------------------------------------
    #[test]
    fn children_types_have_type_and_named((grammar, ..) in grammar_strat()) {
        for e in generate_types(&grammar) {
            if let Some(ch) = e.get("children") {
                if let Some(ts) = ch.get("types").and_then(Value::as_array) {
                    for t in ts {
                        prop_assert!(t.get("type").is_some());
                        prop_assert!(t.get("named").is_some());
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 20. Subtypes is always an array when present
    // -----------------------------------------------------------------------
    #[test]
    fn subtypes_array_when_present((grammar, ..) in grammar_strat()) {
        for e in generate_types(&grammar) {
            if let Some(st) = e.get("subtypes") {
                prop_assert!(st.is_array());
            }
        }
    }

    // -----------------------------------------------------------------------
    // 21. Grammar with externals produces valid JSON containing rules
    // -----------------------------------------------------------------------
    #[test]
    fn externals_dont_break_generation(count in 1usize..4) {
        let mut b = GrammarBuilder::new("ext")
            .token("NUM", r"\d+")
            .external("INDENT")
            .external("DEDENT");
        for i in 0..count {
            b = b.rule(&format!("e{}", i), vec!["NUM"]);
        }
        let g = b.start("e0").build();
        let entries = generate_types(&g);
        for i in 0..count {
            let name = format!("e{}", i);
            prop_assert!(
                entries.iter().any(|e| e["type"].as_str() == Some(&name)),
                "rule '{}' missing", name
            );
        }
    }

    // -----------------------------------------------------------------------
    // 22. Named count >= visible rules count
    // -----------------------------------------------------------------------
    #[test]
    fn named_count_ge_visible((grammar, visible, ..) in grammar_strat()) {
        let named_cnt = generate_types(&grammar)
            .iter()
            .filter(|e| e["named"].as_bool() == Some(true))
            .count();
        prop_assert!(named_cnt >= visible.len());
    }

    // -----------------------------------------------------------------------
    // 23. Large grammar is valid
    // -----------------------------------------------------------------------
    #[test]
    fn large_grammar_valid(
        rc in 10usize..30,
        tc in 3usize..10,
    ) {
        let mut b = GrammarBuilder::new("big").token("NUM", r"\d+");
        for i in 0..tc {
            b = b.token(&format!("t{}", i), &format!("t{}", i));
        }
        for i in 0..rc {
            b = b.rule(&format!("rule{}", i), vec!["NUM"]);
        }
        let g = b.start("rule0").build();
        let entries = generate_types(&g);
        let named = entries.iter().filter(|e| e["named"].as_bool() == Some(true)).count();
        prop_assert!(named >= rc);
    }

    // -----------------------------------------------------------------------
    // 24. Determinism across cloned grammar
    // -----------------------------------------------------------------------
    #[test]
    fn determinism_across_clone((grammar, ..) in grammar_strat()) {
        let g1 = NodeTypesGenerator::new(&grammar);
        let cloned = grammar.clone();
        let g2 = NodeTypesGenerator::new(&cloned);
        let a: Value = serde_json::from_str(&g1.generate().unwrap()).unwrap();
        let b: Value = serde_json::from_str(&g2.generate().unwrap()).unwrap();
        prop_assert_eq!(a, b);
    }

    // -----------------------------------------------------------------------
    // 25. Fields with data preserve field names
    // -----------------------------------------------------------------------
    #[test]
    fn fields_preserve_names(
        vis in prop::collection::vec(vis_name(), 1..3),
        fields in prop::collection::vec(field_name(), 1..4),
    ) {
        let vis = dedup(vis);
        let fields = dedup(fields);
        let g = make_grammar(&vis, &[], &[], &[("tok".into(), r"\w+".into())], &fields);
        let entries = generate_types(&g);
        let mut found = std::collections::HashSet::new();
        for e in &entries {
            if let Some(fo) = e.get("fields").and_then(Value::as_object) {
                for k in fo.keys() { found.insert(k.clone()); }
            }
        }
        for f in &fields {
            prop_assert!(found.contains(f), "field '{}' not in output", f);
        }
    }
}

// ===========================================================================
// Deterministic (non-proptest) tests for specific grammar shapes
// ===========================================================================

// -----------------------------------------------------------------------
// 26. Empty grammar → valid empty JSON array
// -----------------------------------------------------------------------
#[test]
fn empty_grammar_valid_empty_array() {
    let g = Grammar::new("empty".into());
    let entries = generate_types(&g);
    assert!(entries.is_empty(), "empty grammar should yield []");
}

// -----------------------------------------------------------------------
// 27. Token-only grammar → only unnamed entries
// -----------------------------------------------------------------------
#[test]
fn token_only_all_unnamed() {
    let mut g = Grammar::new("toks".into());
    g.tokens.insert(
        SymbolId(0),
        Token {
            name: "plus".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "minus".into(),
            pattern: TokenPattern::String("-".into()),
            fragile: false,
        },
    );
    let entries = generate_types(&g);
    for e in &entries {
        assert_eq!(e["named"].as_bool(), Some(false));
    }
}

// -----------------------------------------------------------------------
// 28. Python-like grammar produces expected named rules
// -----------------------------------------------------------------------
#[test]
fn python_like_produces_expected_rules() {
    let g = GrammarBuilder::python_like();
    let entries = generate_types(&g);
    let names = type_names(&entries);
    assert!(names.contains(&"module".to_string()));
    assert!(names.contains(&"statement".to_string()));
}

// -----------------------------------------------------------------------
// 29. JavaScript-like grammar produces expected named rules
// -----------------------------------------------------------------------
#[test]
fn javascript_like_produces_expected_rules() {
    let g = GrammarBuilder::javascript_like();
    let entries = generate_types(&g);
    let names = type_names(&entries);
    assert!(names.contains(&"program".to_string()));
    assert!(names.contains(&"expression".to_string()));
}

// -----------------------------------------------------------------------
// 30. Mixed grammar (rules + tokens + externals) all valid
// -----------------------------------------------------------------------
#[test]
fn mixed_grammar_all_valid() {
    let g = GrammarBuilder::new("mix")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("(", "(")
        .token(")", ")")
        .external("INDENT")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("paren", vec!["(", "expr", ")"])
        .start("expr")
        .build();
    let entries = generate_types(&g);
    assert!(!entries.is_empty());
    for e in &entries {
        assert!(e.get("type").and_then(Value::as_str).is_some());
        assert!(e.get("named").and_then(Value::as_bool).is_some());
    }
    let names = type_names(&entries);
    assert!(names.contains(&"expr".to_string()));
    assert!(names.contains(&"paren".to_string()));
}
