use std::fs;
use tempfile::tempdir;

#[test]
fn test_external_attribute() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(
        &grammar_path,
        r#"
        #[adze::grammar("test_external")]
        mod grammar {
            #[adze::language]
            pub enum Expression {
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
            }

            #[adze::external]
            struct IndentToken;

            #[adze::external]
            struct DedentToken;
        }
    "#,
    )
    .unwrap();

    // Generate grammar JSON
    let grammars =
        adze_tool::generate_grammars(&grammar_path).expect("Failed to generate grammars");
    assert!(!grammars.is_empty(), "No grammars generated");

    let grammar_json = &grammars[0];

    // Check that externals are included
    assert!(grammar_json.get("externals").is_some());
    let externals = grammar_json["externals"].as_array().unwrap();
    assert_eq!(externals.len(), 2);

    // Check that external symbols are present
    let external_names: Vec<String> = externals
        .iter()
        .map(|e| e["name"].as_str().unwrap().to_string())
        .collect();
    assert!(external_names.contains(&"IndentToken".to_string()));
    assert!(external_names.contains(&"DedentToken".to_string()));
}

#[test]
fn test_word_attribute() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(
        &grammar_path,
        r#"
        #[adze::grammar("test_word")]
        mod grammar {
            #[adze::language]
            pub enum Expression {
                Identifier(Identifier),
                Keyword(
                    #[adze::leaf(text = "if")]
                    ()
                ),
            }

            #[adze::word]
            struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }
        }
    "#,
    )
    .unwrap();

    // Generate grammar JSON
    let grammars =
        adze_tool::generate_grammars(&grammar_path).expect("Failed to generate grammars");
    assert!(!grammars.is_empty(), "No grammars generated");

    let grammar_json = &grammars[0];

    // Check that word is set
    eprintln!(
        "Grammar JSON: {}",
        serde_json::to_string_pretty(&grammar_json).unwrap()
    );
    assert!(
        grammar_json.get("word").is_some(),
        "Word field not found in grammar"
    );
    let word = grammar_json["word"].as_str().unwrap();
    assert_eq!(word, "Identifier");
}

#[test]
fn test_combined_attributes() {
    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.rs");

    fs::write(
        &grammar_path,
        r#"
        #[adze::grammar("test_combined")]
        mod grammar {
            #[adze::language]
            pub struct Program {
                statements: Vec<Statement>,
            }

            pub enum Statement {
                Expression(Expression),
            }

            pub enum Expression {
                Identifier(Identifier),
                Number(
                    #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                    i32
                ),
            }

            #[adze::word]
            struct Identifier {
                #[adze::leaf(pattern = r"[a-zA-Z_]\w*")]
                name: String,
            }

            #[adze::extra]
            struct Whitespace {
                #[adze::leaf(pattern = r"\s+")]
                _ws: (),
            }

            #[adze::external]
            struct Comment;
        }
    "#,
    )
    .unwrap();

    // Generate grammar JSON
    let grammars =
        adze_tool::generate_grammars(&grammar_path).expect("Failed to generate grammars");
    assert!(!grammars.is_empty(), "No grammars generated");

    let grammar_json = &grammars[0];

    // Check word
    assert_eq!(grammar_json["word"].as_str().unwrap(), "Identifier");

    // Check extras
    let extras = grammar_json["extras"].as_array().unwrap();
    assert!(
        extras
            .iter()
            .any(|e| e["name"].as_str() == Some("Whitespace"))
    );

    // Check externals
    let externals = grammar_json["externals"].as_array().unwrap();
    assert!(
        externals
            .iter()
            .any(|e| e["name"].as_str() == Some("Comment"))
    );
}
