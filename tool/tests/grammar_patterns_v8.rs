//! Comprehensive tests for programming language grammar pattern builds in adze-tool.
//!
//! 80+ tests covering these language patterns:
//!   1.  calc_*       — Calculator: expr + number + operators
//!   2.  bool_*       — Boolean expressions: true/false/and/or/not
//!   3.  strcat_*     — String concatenation: string + "+" + string
//!   4.  varlet_*     — Variable declaration: let + name + = + value
//!   5.  fndef_*      — Function definition: fn + name + () + block
//!   6.  ifstmt_*     — If statement: if + condition + then + else
//!   7.  whloop_*     — While loop: while + condition + block
//!   8.  forloop_*    — For loop: for + var + in + range + block
//!   9.  arrlit_*     — Array literal: [ + items + ]
//!  10.  objlit_*     — Object literal: { + key: value pairs + }
//!  11.  imprt_*      — Import statement: import + path
//!  12.  retstmt_*    — Return statement: return + expr
//!  13.  cmt_*        — Comment grammar (line comment)
//!  14.  ws_*         — Whitespace handling (extra)
//!  15.  semi_*       — Semicolon-separated statements
//!  16.  comma_*      — Comma-separated lists
//!  17.  dotacc_*     — Dot-access: expr.name
//!  18.  idxacc_*     — Index access: expr[index]
//!  19.  ternary_*    — Ternary expression: cond ? true : false
//!  20.  pmatch_*     — Pattern matching: match + cases

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, BuildStats, build_parser};
use tempfile::TempDir;

// ── Helpers ──────────────────────────────────────────────────────────────

fn tmp_opts() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().expect("tmpdir");
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    (dir, opts)
}

fn build_ok(g: Grammar) -> BuildResult {
    let (_dir, opts) = tmp_opts();
    build_parser(g, opts).expect("build should succeed")
}

fn stats_for(g: Grammar) -> BuildStats {
    build_ok(g).build_stats.clone()
}

// ── Grammar constructors ─────────────────────────────────────────────────

fn calc_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("minus", r"\-")
        .token("star", r"\*")
        .token("slash", r"\/")
        .token("lparen", r"\(")
        .token("rparen", r"\)")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "minus", "expr"],
            1,
            Associativity::Left,
        )
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "slash", "expr"],
            2,
            Associativity::Left,
        )
        .rule("expr", vec!["lparen", "expr", "rparen"])
        .rule("expr", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

fn bool_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("true_kw", "true")
        .token("false_kw", "false")
        .token("and_kw", "and")
        .token("or_kw", "or")
        .token("not_kw", "not")
        .rule_with_precedence(
            "bexpr",
            vec!["bexpr", "and_kw", "bexpr"],
            2,
            Associativity::Left,
        )
        .rule_with_precedence(
            "bexpr",
            vec!["bexpr", "or_kw", "bexpr"],
            1,
            Associativity::Left,
        )
        .rule("bexpr", vec!["not_kw", "bexpr"])
        .rule("bexpr", vec!["true_kw"])
        .rule("bexpr", vec!["false_kw"])
        .rule("start", vec!["bexpr"])
        .start("start")
        .build()
}

fn strcat_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("str_lit", r#""[^"]*""#)
        .token("concat", r"\+")
        .rule_with_precedence(
            "strexpr",
            vec!["strexpr", "concat", "strexpr"],
            1,
            Associativity::Left,
        )
        .rule("strexpr", vec!["str_lit"])
        .rule("start", vec!["strexpr"])
        .start("start")
        .build()
}

fn varlet_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("let_kw", "let")
        .token("ident", r"[a-z_][a-z0-9_]*")
        .token("eq", "=")
        .token("num", r"\d+")
        .rule("vardecl", vec!["let_kw", "ident", "eq", "num"])
        .rule("start", vec!["vardecl"])
        .start("start")
        .build()
}

fn fndef_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("fn_kw", "fn")
        .token("ident", r"[a-z_][a-z0-9_]*")
        .token("lparen", r"\(")
        .token("rparen", r"\)")
        .token("lbrace", r"\{")
        .token("rbrace", r"\}")
        .token("num", r"\d+")
        .rule("body", vec!["num"])
        .rule("block", vec!["lbrace", "body", "rbrace"])
        .rule("fndef", vec!["fn_kw", "ident", "lparen", "rparen", "block"])
        .rule("start", vec!["fndef"])
        .start("start")
        .build()
}

fn ifstmt_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("if_kw", "if")
        .token("then_kw", "then")
        .token("else_kw", "else")
        .token("end_kw", "end")
        .token("ident", r"[a-z_][a-z0-9_]*")
        .token("num", r"\d+")
        .rule("cond", vec!["ident"])
        .rule("val", vec!["num"])
        .rule(
            "ifstmt",
            vec![
                "if_kw", "cond", "then_kw", "val", "else_kw", "val", "end_kw",
            ],
        )
        .rule("start", vec!["ifstmt"])
        .start("start")
        .build()
}

fn whloop_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("while_kw", "while")
        .token("do_kw", "do")
        .token("end_kw", "end")
        .token("ident", r"[a-z_][a-z0-9_]*")
        .token("num", r"\d+")
        .rule("cond", vec!["ident"])
        .rule("body", vec!["num"])
        .rule(
            "whloop",
            vec!["while_kw", "cond", "do_kw", "body", "end_kw"],
        )
        .rule("start", vec!["whloop"])
        .start("start")
        .build()
}

fn forloop_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("for_kw", "for")
        .token("in_kw", "in")
        .token("do_kw", "do")
        .token("end_kw", "end")
        .token("ident", r"[a-z_][a-z0-9_]*")
        .token("num", r"\d+")
        .rule("range", vec!["num"])
        .rule("body", vec!["num"])
        .rule(
            "forloop",
            vec![
                "for_kw", "ident", "in_kw", "range", "do_kw", "body", "end_kw",
            ],
        )
        .rule("start", vec!["forloop"])
        .start("start")
        .build()
}

fn arrlit_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("lbracket", r"\[")
        .token("rbracket", r"\]")
        .token("comma", ",")
        .token("num", r"\d+")
        .rule("item", vec!["num"])
        .rule("items", vec!["item"])
        .rule("items", vec!["items", "comma", "item"])
        .rule("arr", vec!["lbracket", "items", "rbracket"])
        .rule("start", vec!["arr"])
        .start("start")
        .build()
}

fn objlit_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("lbrace", r"\{")
        .token("rbrace", r"\}")
        .token("colon", ":")
        .token("comma", ",")
        .token("ident", r"[a-z_][a-z0-9_]*")
        .token("num", r"\d+")
        .rule("value", vec!["num"])
        .rule("pair", vec!["ident", "colon", "value"])
        .rule("pairs", vec!["pair"])
        .rule("pairs", vec!["pairs", "comma", "pair"])
        .rule("obj", vec!["lbrace", "pairs", "rbrace"])
        .rule("start", vec!["obj"])
        .start("start")
        .build()
}

fn imprt_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("import_kw", "import")
        .token("path", r"[a-z_][a-z0-9_/]*")
        .rule("imprt", vec!["import_kw", "path"])
        .rule("start", vec!["imprt"])
        .start("start")
        .build()
}

fn retstmt_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("return_kw", "return")
        .token("num", r"\d+")
        .rule("expr", vec!["num"])
        .rule("retstmt", vec!["return_kw", "expr"])
        .rule("start", vec!["retstmt"])
        .start("start")
        .build()
}

fn cmt_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("comment", r"//[^\n]*")
        .token("ident", r"[a-z]+")
        .rule("start", vec!["ident"])
        .extra("comment")
        .start("start")
        .build()
}

fn ws_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("ws", r"[ \t\n]+")
        .token("word", r"[a-z]+")
        .rule("start", vec!["word"])
        .extra("ws")
        .start("start")
        .build()
}

fn semi_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("semi", ";")
        .token("ident", r"[a-z]+")
        .rule("stmt", vec!["ident"])
        .rule("stmts", vec!["stmt"])
        .rule("stmts", vec!["stmts", "semi", "stmt"])
        .rule("start", vec!["stmts"])
        .start("start")
        .build()
}

fn comma_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("comma", ",")
        .token("num", r"\d+")
        .rule("item", vec!["num"])
        .rule("list", vec!["item"])
        .rule("list", vec!["list", "comma", "item"])
        .rule("start", vec!["list"])
        .start("start")
        .build()
}

fn dotacc_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("dot", r"\.")
        .token("ident", r"[a-z_][a-z0-9_]*")
        .rule_with_precedence("expr", vec!["expr", "dot", "ident"], 1, Associativity::Left)
        .rule("expr", vec!["ident"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

fn idxacc_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("lbracket", r"\[")
        .token("rbracket", r"\]")
        .token("ident", r"[a-z_][a-z0-9_]*")
        .token("num", r"\d+")
        .rule("index", vec!["num"])
        .rule_with_precedence(
            "expr",
            vec!["expr", "lbracket", "index", "rbracket"],
            1,
            Associativity::Left,
        )
        .rule("expr", vec!["ident"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

fn ternary_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("ident", r"[a-z_][a-z0-9_]*")
        .token("num", r"\d+")
        .token("qmark", r"\?")
        .token("colon", ":")
        .rule("cond", vec!["ident"])
        .rule("val", vec!["num"])
        .rule_with_precedence(
            "texpr",
            vec!["cond", "qmark", "val", "colon", "val"],
            1,
            Associativity::Right,
        )
        .rule("texpr", vec!["val"])
        .rule("start", vec!["texpr"])
        .start("start")
        .build()
}

fn pmatch_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("match_kw", "match")
        .token("arrow", "=>")
        .token("pipe", r"\|")
        .token("end_kw", "end")
        .token("ident", r"[a-z_][a-z0-9_]*")
        .token("num", r"\d+")
        .rule("pattern", vec!["ident"])
        .rule("result", vec!["num"])
        .rule("arm", vec!["pattern", "arrow", "result"])
        .rule("arms", vec!["arm"])
        .rule("arms", vec!["arms", "pipe", "arm"])
        .rule("pmatch", vec!["match_kw", "ident", "arms", "end_kw"])
        .rule("start", vec!["pmatch"])
        .start("start")
        .build()
}

// ═════════════════════════════════════════════════════════════════════════
// 1. calc — Calculator: expr + number + operators
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn calc_build_ok() {
    build_ok(calc_grammar("gp_v8_calc01"));
}

#[test]
fn calc_state_count_positive() {
    let s = stats_for(calc_grammar("gp_v8_calc02"));
    assert!(s.state_count > 0);
}

#[test]
fn calc_symbol_count_positive() {
    let s = stats_for(calc_grammar("gp_v8_calc03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn calc_parser_code_nonempty() {
    let r = build_ok(calc_grammar("gp_v8_calc04"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 2. bool — Boolean expressions: true/false/and/or/not
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn bool_build_ok() {
    build_ok(bool_grammar("gp_v8_bool01"));
}

#[test]
fn bool_state_count_positive() {
    let s = stats_for(bool_grammar("gp_v8_bool02"));
    assert!(s.state_count > 0);
}

#[test]
fn bool_symbol_count_positive() {
    let s = stats_for(bool_grammar("gp_v8_bool03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn bool_parser_code_nonempty() {
    let r = build_ok(bool_grammar("gp_v8_bool04"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 3. strcat — String concatenation
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn strcat_build_ok() {
    build_ok(strcat_grammar("gp_v8_strcat01"));
}

#[test]
fn strcat_state_count_positive() {
    let s = stats_for(strcat_grammar("gp_v8_strcat02"));
    assert!(s.state_count > 0);
}

#[test]
fn strcat_symbol_count_positive() {
    let s = stats_for(strcat_grammar("gp_v8_strcat03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn strcat_node_types_nonempty() {
    let r = build_ok(strcat_grammar("gp_v8_strcat04"));
    assert!(!r.node_types_json.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 4. varlet — Variable declaration: let + name + = + value
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn varlet_build_ok() {
    build_ok(varlet_grammar("gp_v8_varlet01"));
}

#[test]
fn varlet_state_count_positive() {
    let s = stats_for(varlet_grammar("gp_v8_varlet02"));
    assert!(s.state_count > 0);
}

#[test]
fn varlet_symbol_count_positive() {
    let s = stats_for(varlet_grammar("gp_v8_varlet03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn varlet_parser_code_nonempty() {
    let r = build_ok(varlet_grammar("gp_v8_varlet04"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 5. fndef — Function definition: fn + name + () + block
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn fndef_build_ok() {
    build_ok(fndef_grammar("gp_v8_fndef01"));
}

#[test]
fn fndef_state_count_positive() {
    let s = stats_for(fndef_grammar("gp_v8_fndef02"));
    assert!(s.state_count > 0);
}

#[test]
fn fndef_symbol_count_positive() {
    let s = stats_for(fndef_grammar("gp_v8_fndef03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn fndef_parser_code_nonempty() {
    let r = build_ok(fndef_grammar("gp_v8_fndef04"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 6. ifstmt — If statement: if + condition + then + else
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn ifstmt_build_ok() {
    build_ok(ifstmt_grammar("gp_v8_ifstmt01"));
}

#[test]
fn ifstmt_state_count_positive() {
    let s = stats_for(ifstmt_grammar("gp_v8_ifstmt02"));
    assert!(s.state_count > 0);
}

#[test]
fn ifstmt_symbol_count_positive() {
    let s = stats_for(ifstmt_grammar("gp_v8_ifstmt03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn ifstmt_parser_code_nonempty() {
    let r = build_ok(ifstmt_grammar("gp_v8_ifstmt04"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 7. whloop — While loop: while + condition + block
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn whloop_build_ok() {
    build_ok(whloop_grammar("gp_v8_whloop01"));
}

#[test]
fn whloop_state_count_positive() {
    let s = stats_for(whloop_grammar("gp_v8_whloop02"));
    assert!(s.state_count > 0);
}

#[test]
fn whloop_symbol_count_positive() {
    let s = stats_for(whloop_grammar("gp_v8_whloop03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn whloop_parser_code_nonempty() {
    let r = build_ok(whloop_grammar("gp_v8_whloop04"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 8. forloop — For loop: for + var + in + range + block
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn forloop_build_ok() {
    build_ok(forloop_grammar("gp_v8_forloop01"));
}

#[test]
fn forloop_state_count_positive() {
    let s = stats_for(forloop_grammar("gp_v8_forloop02"));
    assert!(s.state_count > 0);
}

#[test]
fn forloop_symbol_count_positive() {
    let s = stats_for(forloop_grammar("gp_v8_forloop03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn forloop_parser_code_nonempty() {
    let r = build_ok(forloop_grammar("gp_v8_forloop04"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 9. arrlit — Array literal: [ + items + ]
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn arrlit_build_ok() {
    build_ok(arrlit_grammar("gp_v8_arrlit01"));
}

#[test]
fn arrlit_state_count_positive() {
    let s = stats_for(arrlit_grammar("gp_v8_arrlit02"));
    assert!(s.state_count > 0);
}

#[test]
fn arrlit_symbol_count_positive() {
    let s = stats_for(arrlit_grammar("gp_v8_arrlit03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn arrlit_node_types_nonempty() {
    let r = build_ok(arrlit_grammar("gp_v8_arrlit04"));
    assert!(!r.node_types_json.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 10. objlit — Object literal: { + key: value pairs + }
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn objlit_build_ok() {
    build_ok(objlit_grammar("gp_v8_objlit01"));
}

#[test]
fn objlit_state_count_positive() {
    let s = stats_for(objlit_grammar("gp_v8_objlit02"));
    assert!(s.state_count > 0);
}

#[test]
fn objlit_symbol_count_positive() {
    let s = stats_for(objlit_grammar("gp_v8_objlit03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn objlit_parser_code_nonempty() {
    let r = build_ok(objlit_grammar("gp_v8_objlit04"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 11. imprt — Import statement: import + path
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn imprt_build_ok() {
    build_ok(imprt_grammar("gp_v8_imprt01"));
}

#[test]
fn imprt_state_count_positive() {
    let s = stats_for(imprt_grammar("gp_v8_imprt02"));
    assert!(s.state_count > 0);
}

#[test]
fn imprt_symbol_count_positive() {
    let s = stats_for(imprt_grammar("gp_v8_imprt03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn imprt_parser_code_nonempty() {
    let r = build_ok(imprt_grammar("gp_v8_imprt04"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 12. retstmt — Return statement: return + expr
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn retstmt_build_ok() {
    build_ok(retstmt_grammar("gp_v8_retstmt01"));
}

#[test]
fn retstmt_state_count_positive() {
    let s = stats_for(retstmt_grammar("gp_v8_retstmt02"));
    assert!(s.state_count > 0);
}

#[test]
fn retstmt_symbol_count_positive() {
    let s = stats_for(retstmt_grammar("gp_v8_retstmt03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn retstmt_parser_code_nonempty() {
    let r = build_ok(retstmt_grammar("gp_v8_retstmt04"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 13. cmt — Comment grammar (line comment as extra)
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cmt_build_ok() {
    build_ok(cmt_grammar("gp_v8_cmt01"));
}

#[test]
fn cmt_state_count_positive() {
    let s = stats_for(cmt_grammar("gp_v8_cmt02"));
    assert!(s.state_count > 0);
}

#[test]
fn cmt_symbol_count_positive() {
    let s = stats_for(cmt_grammar("gp_v8_cmt03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn cmt_parser_code_nonempty() {
    let r = build_ok(cmt_grammar("gp_v8_cmt04"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 14. ws — Whitespace handling (extra)
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn ws_build_ok() {
    build_ok(ws_grammar("gp_v8_ws01"));
}

#[test]
fn ws_state_count_positive() {
    let s = stats_for(ws_grammar("gp_v8_ws02"));
    assert!(s.state_count > 0);
}

#[test]
fn ws_symbol_count_positive() {
    let s = stats_for(ws_grammar("gp_v8_ws03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn ws_parser_code_nonempty() {
    let r = build_ok(ws_grammar("gp_v8_ws04"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 15. semi — Semicolon-separated statements
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn semi_build_ok() {
    build_ok(semi_grammar("gp_v8_semi01"));
}

#[test]
fn semi_state_count_positive() {
    let s = stats_for(semi_grammar("gp_v8_semi02"));
    assert!(s.state_count > 0);
}

#[test]
fn semi_symbol_count_positive() {
    let s = stats_for(semi_grammar("gp_v8_semi03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn semi_node_types_nonempty() {
    let r = build_ok(semi_grammar("gp_v8_semi04"));
    assert!(!r.node_types_json.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 16. comma — Comma-separated lists
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn comma_build_ok() {
    build_ok(comma_grammar("gp_v8_comma01"));
}

#[test]
fn comma_state_count_positive() {
    let s = stats_for(comma_grammar("gp_v8_comma02"));
    assert!(s.state_count > 0);
}

#[test]
fn comma_symbol_count_positive() {
    let s = stats_for(comma_grammar("gp_v8_comma03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn comma_node_types_nonempty() {
    let r = build_ok(comma_grammar("gp_v8_comma04"));
    assert!(!r.node_types_json.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 17. dotacc — Dot-access: expr.name
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn dotacc_build_ok() {
    build_ok(dotacc_grammar("gp_v8_dotacc01"));
}

#[test]
fn dotacc_state_count_positive() {
    let s = stats_for(dotacc_grammar("gp_v8_dotacc02"));
    assert!(s.state_count > 0);
}

#[test]
fn dotacc_symbol_count_positive() {
    let s = stats_for(dotacc_grammar("gp_v8_dotacc03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn dotacc_parser_code_nonempty() {
    let r = build_ok(dotacc_grammar("gp_v8_dotacc04"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 18. idxacc — Index access: expr[index]
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn idxacc_build_ok() {
    build_ok(idxacc_grammar("gp_v8_idxacc01"));
}

#[test]
fn idxacc_state_count_positive() {
    let s = stats_for(idxacc_grammar("gp_v8_idxacc02"));
    assert!(s.state_count > 0);
}

#[test]
fn idxacc_symbol_count_positive() {
    let s = stats_for(idxacc_grammar("gp_v8_idxacc03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn idxacc_parser_code_nonempty() {
    let r = build_ok(idxacc_grammar("gp_v8_idxacc04"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 19. ternary — Ternary expression: cond ? true : false
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn ternary_build_ok() {
    build_ok(ternary_grammar("gp_v8_ternary01"));
}

#[test]
fn ternary_state_count_positive() {
    let s = stats_for(ternary_grammar("gp_v8_ternary02"));
    assert!(s.state_count > 0);
}

#[test]
fn ternary_symbol_count_positive() {
    let s = stats_for(ternary_grammar("gp_v8_ternary03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn ternary_parser_code_nonempty() {
    let r = build_ok(ternary_grammar("gp_v8_ternary04"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 20. pmatch — Pattern matching: match + cases
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn pmatch_build_ok() {
    build_ok(pmatch_grammar("gp_v8_pmatch01"));
}

#[test]
fn pmatch_state_count_positive() {
    let s = stats_for(pmatch_grammar("gp_v8_pmatch02"));
    assert!(s.state_count > 0);
}

#[test]
fn pmatch_symbol_count_positive() {
    let s = stats_for(pmatch_grammar("gp_v8_pmatch03"));
    assert!(s.symbol_count > 0);
}

#[test]
fn pmatch_parser_code_nonempty() {
    let r = build_ok(pmatch_grammar("gp_v8_pmatch04"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// Cross-pattern: complexity comparison
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn cross_calc_more_symbols_than_imprt() {
    let calc = stats_for(calc_grammar("gp_v8_cross01a"));
    let imp = stats_for(imprt_grammar("gp_v8_cross01b"));
    assert!(calc.symbol_count > imp.symbol_count);
}

#[test]
fn cross_objlit_more_symbols_than_retstmt() {
    let obj = stats_for(objlit_grammar("gp_v8_cross02a"));
    let ret = stats_for(retstmt_grammar("gp_v8_cross02b"));
    assert!(obj.symbol_count > ret.symbol_count);
}

#[test]
fn cross_fndef_more_states_than_imprt() {
    let fnd = stats_for(fndef_grammar("gp_v8_cross03a"));
    let imp = stats_for(imprt_grammar("gp_v8_cross03b"));
    assert!(fnd.state_count > imp.state_count);
}

#[test]
fn cross_pmatch_more_symbols_than_ws() {
    let pm = stats_for(pmatch_grammar("gp_v8_cross04a"));
    let wsg = stats_for(ws_grammar("gp_v8_cross04b"));
    assert!(pm.symbol_count > wsg.symbol_count);
}

// ═════════════════════════════════════════════════════════════════════════
// Cross-pattern: deterministic builds
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn deterministic_calc_stats() {
    let s1 = stats_for(calc_grammar("gp_v8_det01a"));
    let s2 = stats_for(calc_grammar("gp_v8_det01b"));
    assert_eq!(s1.state_count, s2.state_count);
    assert_eq!(s1.symbol_count, s2.symbol_count);
}

#[test]
fn deterministic_bool_stats() {
    let s1 = stats_for(bool_grammar("gp_v8_det02a"));
    let s2 = stats_for(bool_grammar("gp_v8_det02b"));
    assert_eq!(s1.state_count, s2.state_count);
    assert_eq!(s1.symbol_count, s2.symbol_count);
}

#[test]
fn deterministic_fndef_stats() {
    let s1 = stats_for(fndef_grammar("gp_v8_det03a"));
    let s2 = stats_for(fndef_grammar("gp_v8_det03b"));
    assert_eq!(s1.state_count, s2.state_count);
    assert_eq!(s1.symbol_count, s2.symbol_count);
}

#[test]
fn deterministic_pmatch_stats() {
    let s1 = stats_for(pmatch_grammar("gp_v8_det04a"));
    let s2 = stats_for(pmatch_grammar("gp_v8_det04b"));
    assert_eq!(s1.state_count, s2.state_count);
    assert_eq!(s1.symbol_count, s2.symbol_count);
}

// ═════════════════════════════════════════════════════════════════════════
// Cross-pattern: grammar name in output
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn grammar_name_calc_in_output() {
    let r = build_ok(calc_grammar("gp_v8_name01"));
    assert!(r.parser_code.contains("gp_v8_name01"));
}

#[test]
fn grammar_name_fndef_in_output() {
    let r = build_ok(fndef_grammar("gp_v8_name02"));
    assert!(r.parser_code.contains("gp_v8_name02"));
}

#[test]
fn grammar_name_pmatch_in_output() {
    let r = build_ok(pmatch_grammar("gp_v8_name03"));
    assert!(r.parser_code.contains("gp_v8_name03"));
}

#[test]
fn grammar_name_objlit_in_output() {
    let r = build_ok(objlit_grammar("gp_v8_name04"));
    assert!(r.parser_code.contains("gp_v8_name04"));
}

// ═════════════════════════════════════════════════════════════════════════
// Cross-pattern: node_types_json valid JSON
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn node_json_valid_calc() {
    let r = build_ok(calc_grammar("gp_v8_json01"));
    let parsed: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid json");
    assert!(parsed.is_array());
}

#[test]
fn node_json_valid_bool() {
    let r = build_ok(bool_grammar("gp_v8_json02"));
    let parsed: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid json");
    assert!(parsed.is_array());
}

#[test]
fn node_json_valid_fndef() {
    let r = build_ok(fndef_grammar("gp_v8_json03"));
    let parsed: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid json");
    assert!(parsed.is_array());
}

#[test]
fn node_json_valid_pmatch() {
    let r = build_ok(pmatch_grammar("gp_v8_json04"));
    let parsed: serde_json::Value = serde_json::from_str(&r.node_types_json).expect("valid json");
    assert!(parsed.is_array());
}
