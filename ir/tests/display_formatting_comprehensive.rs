#![allow(clippy::needless_range_loop)]

use adze_ir::{
    Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, FieldId, Grammar,
    GrammarError, Precedence, PrecedenceKind, ProductionId, Rule, RuleId, StateId, Symbol,
    SymbolId, SymbolMetadata, Token, TokenPattern, ValidationError, ValidationWarning,
};

// ---------------------------------------------------------------------------
// SymbolId Display
// ---------------------------------------------------------------------------

#[test]
fn display_symbol_id_zero() {
    assert_eq!(format!("{}", SymbolId(0)), "Symbol(0)");
}

#[test]
fn display_symbol_id_large() {
    assert_eq!(format!("{}", SymbolId(u16::MAX)), "Symbol(65535)");
}

#[test]
fn debug_symbol_id() {
    let id = SymbolId(7);
    assert_eq!(format!("{:?}", id), "SymbolId(7)");
}

// ---------------------------------------------------------------------------
// RuleId Display
// ---------------------------------------------------------------------------

#[test]
fn display_rule_id() {
    assert_eq!(format!("{}", RuleId(10)), "Rule(10)");
}

#[test]
fn debug_rule_id() {
    assert_eq!(format!("{:?}", RuleId(10)), "RuleId(10)");
}

// ---------------------------------------------------------------------------
// StateId Display
// ---------------------------------------------------------------------------

#[test]
fn display_state_id() {
    assert_eq!(format!("{}", StateId(5)), "State(5)");
}

#[test]
fn debug_state_id() {
    assert_eq!(format!("{:?}", StateId(5)), "StateId(5)");
}

// ---------------------------------------------------------------------------
// FieldId Display
// ---------------------------------------------------------------------------

#[test]
fn display_field_id() {
    assert_eq!(format!("{}", FieldId(3)), "Field(3)");
}

#[test]
fn debug_field_id() {
    assert_eq!(format!("{:?}", FieldId(3)), "FieldId(3)");
}

// ---------------------------------------------------------------------------
// ProductionId Display
// ---------------------------------------------------------------------------

#[test]
fn display_production_id() {
    assert_eq!(format!("{}", ProductionId(42)), "Production(42)");
}

#[test]
fn debug_production_id() {
    assert_eq!(format!("{:?}", ProductionId(42)), "ProductionId(42)");
}

// ---------------------------------------------------------------------------
// Symbol Debug (all variants)
// ---------------------------------------------------------------------------

#[test]
fn debug_symbol_terminal() {
    let sym = Symbol::Terminal(SymbolId(1));
    assert_eq!(format!("{:?}", sym), "Terminal(SymbolId(1))");
}

#[test]
fn debug_symbol_nonterminal() {
    let sym = Symbol::NonTerminal(SymbolId(2));
    assert_eq!(format!("{:?}", sym), "NonTerminal(SymbolId(2))");
}

#[test]
fn debug_symbol_external() {
    let sym = Symbol::External(SymbolId(3));
    assert_eq!(format!("{:?}", sym), "External(SymbolId(3))");
}

#[test]
fn debug_symbol_optional() {
    let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(4))));
    assert_eq!(format!("{:?}", sym), "Optional(Terminal(SymbolId(4)))");
}

#[test]
fn debug_symbol_repeat() {
    let sym = Symbol::Repeat(Box::new(Symbol::NonTerminal(SymbolId(5))));
    assert_eq!(format!("{:?}", sym), "Repeat(NonTerminal(SymbolId(5)))");
}

#[test]
fn debug_symbol_repeat_one() {
    let sym = Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(6))));
    assert_eq!(format!("{:?}", sym), "RepeatOne(Terminal(SymbolId(6)))");
}

#[test]
fn debug_symbol_choice() {
    let sym = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(2)),
    ]);
    let dbg = format!("{:?}", sym);
    assert!(dbg.starts_with("Choice("));
    assert!(dbg.contains("Terminal(SymbolId(1))"));
    assert!(dbg.contains("NonTerminal(SymbolId(2))"));
}

#[test]
fn debug_symbol_sequence() {
    let sym = Symbol::Sequence(vec![
        Symbol::Terminal(SymbolId(10)),
        Symbol::Terminal(SymbolId(11)),
    ]);
    let dbg = format!("{:?}", sym);
    assert!(dbg.starts_with("Sequence("));
    assert!(dbg.contains("SymbolId(10)"));
    assert!(dbg.contains("SymbolId(11)"));
}

#[test]
fn debug_symbol_epsilon() {
    assert_eq!(format!("{:?}", Symbol::Epsilon), "Epsilon");
}

// ---------------------------------------------------------------------------
// Associativity Debug
// ---------------------------------------------------------------------------

#[test]
fn debug_associativity_all_variants() {
    assert_eq!(format!("{:?}", Associativity::Left), "Left");
    assert_eq!(format!("{:?}", Associativity::Right), "Right");
    assert_eq!(format!("{:?}", Associativity::None), "None");
}

// ---------------------------------------------------------------------------
// TokenPattern Debug
// ---------------------------------------------------------------------------

#[test]
fn debug_token_pattern_string() {
    let p = TokenPattern::String("hello".into());
    assert_eq!(format!("{:?}", p), r#"String("hello")"#);
}

#[test]
fn debug_token_pattern_regex() {
    let p = TokenPattern::Regex(r"\d+".into());
    assert_eq!(format!("{:?}", p), r#"Regex("\\d+")"#);
}

// ---------------------------------------------------------------------------
// PrecedenceKind Debug
// ---------------------------------------------------------------------------

#[test]
fn debug_precedence_kind_static() {
    assert_eq!(format!("{:?}", PrecedenceKind::Static(5)), "Static(5)");
}

#[test]
fn debug_precedence_kind_dynamic() {
    assert_eq!(format!("{:?}", PrecedenceKind::Dynamic(-3)), "Dynamic(-3)");
}

// ---------------------------------------------------------------------------
// ConflictResolution Debug
// ---------------------------------------------------------------------------

#[test]
fn debug_conflict_resolution_precedence() {
    let cr = ConflictResolution::Precedence(PrecedenceKind::Static(2));
    let dbg = format!("{:?}", cr);
    assert!(dbg.contains("Precedence"));
    assert!(dbg.contains("Static(2)"));
}

#[test]
fn debug_conflict_resolution_associativity() {
    let cr = ConflictResolution::Associativity(Associativity::Right);
    let dbg = format!("{:?}", cr);
    assert!(dbg.contains("Associativity"));
    assert!(dbg.contains("Right"));
}

#[test]
fn debug_conflict_resolution_glr() {
    assert_eq!(format!("{:?}", ConflictResolution::GLR), "GLR");
}

// ---------------------------------------------------------------------------
// GrammarError Display (thiserror)
// ---------------------------------------------------------------------------

#[test]
fn display_grammar_error_invalid_field_ordering() {
    let err = GrammarError::InvalidFieldOrdering;
    let msg = format!("{}", err);
    assert!(msg.contains("field ordering"));
}

#[test]
fn display_grammar_error_unresolved_symbol() {
    let err = GrammarError::UnresolvedSymbol(SymbolId(99));
    let msg = format!("{}", err);
    assert!(msg.contains("Unresolved symbol"));
    assert!(msg.contains("Symbol(99)"));
}

#[test]
fn display_grammar_error_unresolved_external() {
    let err = GrammarError::UnresolvedExternalSymbol(SymbolId(50));
    let msg = format!("{}", err);
    assert!(msg.contains("external symbol"));
    assert!(msg.contains("Symbol(50)"));
}

#[test]
fn display_grammar_error_conflict() {
    let err = GrammarError::ConflictError("shift-reduce".into());
    let msg = format!("{}", err);
    assert!(msg.contains("Conflict"));
    assert!(msg.contains("shift-reduce"));
}

#[test]
fn display_grammar_error_invalid_precedence() {
    let err = GrammarError::InvalidPrecedence("negative level".into());
    let msg = format!("{}", err);
    assert!(msg.contains("precedence"));
    assert!(msg.contains("negative level"));
}

// ---------------------------------------------------------------------------
// ValidationError Display
// ---------------------------------------------------------------------------

#[test]
fn display_validation_error_undefined_symbol() {
    let err = ValidationError::UndefinedSymbol {
        symbol: SymbolId(1),
        location: "rule 'expr'".into(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("Undefined symbol"));
    assert!(msg.contains("rule 'expr'"));
}

#[test]
fn display_validation_error_empty_grammar() {
    let err = ValidationError::EmptyGrammar;
    assert_eq!(format!("{}", err), "Grammar has no rules defined");
}

#[test]
fn display_validation_error_cyclic_rule() {
    let err = ValidationError::CyclicRule {
        symbols: vec![SymbolId(1), SymbolId(2)],
    };
    let msg = format!("{}", err);
    assert!(msg.contains("Cyclic"));
}

// ---------------------------------------------------------------------------
// ValidationWarning Display
// ---------------------------------------------------------------------------

#[test]
fn display_validation_warning_unused_token() {
    let w = ValidationWarning::UnusedToken {
        token: SymbolId(5),
        name: "SEMICOLON".into(),
    };
    let msg = format!("{}", w);
    assert!(msg.contains("SEMICOLON"));
    assert!(msg.contains("never used"));
}

#[test]
fn display_validation_warning_ambiguous_grammar() {
    let w = ValidationWarning::AmbiguousGrammar {
        message: "dangling else".into(),
    };
    let msg = format!("{}", w);
    assert!(msg.contains("ambiguity"));
    assert!(msg.contains("dangling else"));
}

// ---------------------------------------------------------------------------
// Compound struct Debug formatting
// ---------------------------------------------------------------------------

#[test]
fn debug_rule_struct() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(SymbolId(2)),
        ],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    };
    let dbg = format!("{:?}", rule);
    assert!(dbg.contains("SymbolId(0)"));
    assert!(dbg.contains("Terminal"));
    assert!(dbg.contains("NonTerminal"));
    assert!(dbg.contains("Static(1)"));
    assert!(dbg.contains("Left"));
}

#[test]
fn debug_token_struct() {
    let token = Token {
        name: "NUMBER".into(),
        pattern: TokenPattern::Regex(r"\d+".into()),
        fragile: true,
    };
    let dbg = format!("{:?}", token);
    assert!(dbg.contains("NUMBER"));
    assert!(dbg.contains("Regex"));
    assert!(dbg.contains("true"));
}

#[test]
fn debug_precedence_struct() {
    let prec = Precedence {
        level: 3,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(10), SymbolId(11)],
    };
    let dbg = format!("{:?}", prec);
    assert!(dbg.contains("level: 3"));
    assert!(dbg.contains("Right"));
    assert!(dbg.contains("SymbolId(10)"));
}

#[test]
fn debug_external_token() {
    let ext = ExternalToken {
        name: "INDENT".into(),
        symbol_id: SymbolId(100),
    };
    let dbg = format!("{:?}", ext);
    assert!(dbg.contains("INDENT"));
    assert!(dbg.contains("SymbolId(100)"));
}

#[test]
fn debug_conflict_declaration() {
    let cd = ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: ConflictResolution::GLR,
    };
    let dbg = format!("{:?}", cd);
    assert!(dbg.contains("SymbolId(1)"));
    assert!(dbg.contains("SymbolId(2)"));
    assert!(dbg.contains("GLR"));
}

#[test]
fn debug_symbol_metadata() {
    let meta = SymbolMetadata {
        visible: true,
        named: true,
        hidden: false,
        terminal: false,
    };
    let dbg = format!("{:?}", meta);
    assert!(dbg.contains("visible: true"));
    assert!(dbg.contains("named: true"));
    assert!(dbg.contains("hidden: false"));
    assert!(dbg.contains("terminal: false"));
}

// ---------------------------------------------------------------------------
// Grammar Debug
// ---------------------------------------------------------------------------

#[test]
fn debug_grammar_default() {
    let g = Grammar::default();
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("Grammar"));
    assert!(dbg.contains("name: \"\""));
}

#[test]
fn debug_grammar_named() {
    let g = Grammar::new("json".into());
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("json"));
}
