#[cfg(feature = "glr-core")]
use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
#[cfg(feature = "glr-core")]
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token as IrToken, TokenPattern};
use adze_runtime::{Language, Parser, Token, language::SymbolMetadata};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[cfg(feature = "glr-core")]
fn make_language(counter: Arc<AtomicUsize>) -> Language {
    let mut grammar = Grammar::new("test".to_string());
    let a_id = SymbolId(1);
    grammar.tokens.insert(
        a_id,
        IrToken {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    let start_id = SymbolId(2);
    grammar.rule_names.insert(start_id, "start".to_string());
    grammar.rules.insert(
        start_id,
        vec![Rule {
            lhs: start_id,
            rhs: vec![Symbol::Terminal(a_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).expect("table");
    let table: &'static _ = Box::leak(Box::new(table));

    let t_counter = counter.clone();

    #[allow(clippy::type_complexity)]
    let tokenize_fn: Box<dyn for<'a> Fn(&'a [u8]) -> Box<dyn Iterator<Item = Token> + 'a>> =
        Box::new(
            move |input: &[u8]| -> Box<dyn Iterator<Item = Token> + '_> {
                t_counter.fetch_add(1, Ordering::SeqCst);
                let mut toks = Vec::new();
                if input == b"a" {
                    toks.push(Token {
                        kind: 1,
                        start: 0,
                        end: 1,
                    });
                }
                toks.push(Token {
                    kind: 0,
                    start: input.len() as u32,
                    end: input.len() as u32,
                });
                Box::new(toks.into_iter())
            },
        );

    Language::builder()
        .parse_table(table)
        .symbol_names(vec!["EOF".into(), "a".into(), "start".into()])
        .symbol_metadata(vec![
            SymbolMetadata {
                is_terminal: true,
                is_visible: false,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
        ])
        .field_names(vec![])
        .tokenizer(tokenize_fn)
        .build()
        .unwrap()
}

#[test]
#[cfg(feature = "glr-core")]
fn glr_parse_simple() {
    let counter = Arc::new(AtomicUsize::new(0));
    let lang = make_language(counter.clone());
    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse_utf8("a", None).unwrap();
    assert_eq!(tree.root_kind(), 2); // start symbol
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[cfg(all(feature = "glr-core", feature = "incremental"))]
#[test]
fn glr_incremental_reuse() {
    let counter = Arc::new(AtomicUsize::new(0));
    let lang = make_language(counter.clone());
    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();

    let tree1 = parser.parse_utf8("a", None).unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    let tree2 = parser.parse_utf8("a", Some(&tree1)).unwrap();
    assert_eq!(tree2.root_kind(), 2);
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}
