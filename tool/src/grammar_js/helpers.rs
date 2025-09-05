use super::Rule;
use anyhow::{bail, Result};

/// Common helper functions used in Tree-sitter grammars
pub struct HelperFunctions;

impl HelperFunctions {
    /// Check if a function name is a known helper pattern
    pub fn is_helper_function(name: &str) -> bool {
        matches!(
            name,
            "commaSep"
                | "commaSep1"
                | "sep"
                | "sep1"
                | "sepBy"
                | "sepBy1"
                | "list"
                | "list1"
                | "delimited"
                | "parens"
                | "brackets"
                | "braces"
        )
    }

    /// Evaluate a helper function call
    pub fn evaluate_helper(name: &str, args: Vec<Rule>) -> Result<Rule> {
        match name {
            "commaSep" => {
                // commaSep(rule) => optional(seq(rule, repeat(seq(',', rule))))
                if args.len() != 1 {
                    bail!("commaSep expects 1 argument, got {}", args.len());
                }
                let rule = args.into_iter().next().unwrap();
                Ok(Rule::Optional {
                    value: Box::new(Rule::Seq {
                        members: vec![
                            rule.clone(),
                            Rule::Repeat {
                                content: Box::new(Rule::Seq {
                                    members: vec![
                                        Rule::String {
                                            value: ",".to_string(),
                                        },
                                        rule,
                                    ],
                                }),
                            },
                        ],
                    }),
                })
            }

            "commaSep1" => {
                // commaSep1(rule) => seq(rule, repeat(seq(',', rule)))
                if args.len() != 1 {
                    bail!("commaSep1 expects 1 argument, got {}", args.len());
                }
                let rule = args.into_iter().next().unwrap();
                Ok(Rule::Seq {
                    members: vec![
                        rule.clone(),
                        Rule::Repeat {
                            content: Box::new(Rule::Seq {
                                members: vec![
                                    Rule::String {
                                        value: ",".to_string(),
                                    },
                                    rule,
                                ],
                            }),
                        },
                    ],
                })
            }

            "sep" => {
                // sep(rule, separator) => optional(seq(rule, repeat(seq(separator, rule))))
                if args.len() != 2 {
                    bail!("sep expects 2 arguments, got {}", args.len());
                }
                let mut iter = args.into_iter();
                let rule = iter.next().unwrap();
                let separator = iter.next().unwrap();

                Ok(Rule::Optional {
                    value: Box::new(Rule::Seq {
                        members: vec![
                            rule.clone(),
                            Rule::Repeat {
                                content: Box::new(Rule::Seq {
                                    members: vec![separator, rule],
                                }),
                            },
                        ],
                    }),
                })
            }

            "sep1" => {
                // sep1(rule, separator) => seq(rule, repeat(seq(separator, rule)))
                if args.len() != 2 {
                    bail!("sep1 expects 2 arguments, got {}", args.len());
                }
                let mut iter = args.into_iter();
                let rule = iter.next().unwrap();
                let separator = iter.next().unwrap();

                Ok(Rule::Seq {
                    members: vec![
                        rule.clone(),
                        Rule::Repeat {
                            content: Box::new(Rule::Seq {
                                members: vec![separator, rule],
                            }),
                        },
                    ],
                })
            }

            "parens" => {
                // parens(rule) => seq('(', rule, ')')
                if args.len() != 1 {
                    bail!("parens expects 1 argument, got {}", args.len());
                }
                let rule = args.into_iter().next().unwrap();
                Ok(Rule::Seq {
                    members: vec![
                        Rule::String {
                            value: "(".to_string(),
                        },
                        rule,
                        Rule::String {
                            value: ")".to_string(),
                        },
                    ],
                })
            }

            "brackets" => {
                // brackets(rule) => seq('[', rule, ']')
                if args.len() != 1 {
                    bail!("brackets expects 1 argument, got {}", args.len());
                }
                let rule = args.into_iter().next().unwrap();
                Ok(Rule::Seq {
                    members: vec![
                        Rule::String {
                            value: "[".to_string(),
                        },
                        rule,
                        Rule::String {
                            value: "]".to_string(),
                        },
                    ],
                })
            }

            "braces" => {
                // braces(rule) => seq('{', rule, '}')
                if args.len() != 1 {
                    bail!("braces expects 1 argument, got {}", args.len());
                }
                let rule = args.into_iter().next().unwrap();
                Ok(Rule::Seq {
                    members: vec![
                        Rule::String {
                            value: "{".to_string(),
                        },
                        rule,
                        Rule::String {
                            value: "}".to_string(),
                        },
                    ],
                })
            }

            _ => bail!("Unknown helper function: {}", name),
        }
    }
}
