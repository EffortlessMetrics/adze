//! Comprehensive tests for building multiple different grammars through the
//! adze-tool pipeline.
//!
//! 80+ tests building DIFFERENT grammar patterns — each grammar has a unique
//! name prefixed with "mg_v8_" and exercises a distinct language shape.

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, build_parser};
use tempfile::TempDir;

// ── Helpers ──────────────────────────────────────────────────────────────

fn opts() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().expect("tmpdir");
    let o = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: true,
    };
    (dir, o)
}

fn build(g: adze_ir::Grammar) -> BuildResult {
    let (_dir, o) = opts();
    build_parser(g, o).expect("build should succeed")
}

fn parse_node_types(json: &str) -> serde_json::Value {
    serde_json::from_str(json).expect("node_types_json should be valid JSON")
}

// ── Grammar constructors ─────────────────────────────────────────────────

/// 1. Single token, single rule — minimal grammar
fn minimal_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build()
}

/// 2. Two tokens, single rule — pair grammar
fn pair_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build()
}

/// 3. Keyword + identifier grammar
fn keyword_ident_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("kw_let", "let")
        .token("ident", r"[a-z_][a-z0-9_]*")
        .rule("start", vec!["kw_let", "ident"])
        .start("start")
        .build()
}

/// 4. Number + operator grammar
fn num_op_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("plus", r"\+")
        .rule("start", vec!["num", "plus", "num"])
        .start("start")
        .build()
}

/// 5. If-then-else grammar
fn if_then_else_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("kw_if", "if")
        .token("kw_then", "then")
        .token("kw_else", "else")
        .token("cond", r"[a-z]+")
        .token("body", r"\d+")
        .rule("start", vec!["kw_if", "cond", "kw_then", "body", "kw_else", "body"])
        .rule("start", vec!["kw_if", "cond", "kw_then", "body"])
        .start("start")
        .build()
}

/// 6. Function call grammar (name + parens + args)
fn fn_call_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("fname", r"[a-z]+")
        .token("lparen", r"\(")
        .token("rparen", r"\)")
        .token("arg", r"\d+")
        .rule("start", vec!["fname", "lparen", "arg", "rparen"])
        .rule("start", vec!["fname", "lparen", "rparen"])
        .start("start")
        .build()
}

/// 7. List grammar (item, comma, item, ...)
fn list_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("item", r"[a-z]+")
        .token("comma", ",")
        .rule("list", vec!["item"])
        .rule("list", vec!["list", "comma", "item"])
        .rule("start", vec!["list"])
        .start("start")
        .build()
}

/// 8. Binary expression grammar (left-recursive)
fn binexpr_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("plus", r"\+")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

/// 9. Unary expression grammar
fn unary_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("minus", "-")
        .rule("expr", vec!["minus", "expr"])
        .rule("expr", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

/// 10. Statement grammar (assignment)
fn assign_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("ident", r"[a-z]+")
        .token("eq", "=")
        .token("num", r"\d+")
        .token("semi", ";")
        .rule("start", vec!["ident", "eq", "num", "semi"])
        .start("start")
        .build()
}

/// 11. Block grammar (braces + statements)
fn block_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("lbrace", r"\{")
        .token("rbrace", r"\}")
        .token("word", r"[a-z]+")
        .rule("stmt", vec!["word"])
        .rule("stmts", vec!["stmt"])
        .rule("stmts", vec!["stmts", "stmt"])
        .rule("start", vec!["lbrace", "stmts", "rbrace"])
        .start("start")
        .build()
}

/// 12. Grammar with whitespace extra
fn ws_extra_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("word", r"[a-z]+")
        .token("ws", r"[ \t]+")
        .rule("start", vec!["word"])
        .extra("ws")
        .start("start")
        .build()
}

/// 13. Grammar with comment token
fn comment_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("word", r"[a-z]+")
        .token("comment", r"//[^\n]*")
        .rule("start", vec!["word"])
        .extra("comment")
        .start("start")
        .build()
}

/// 14. Grammar with string literal token
fn string_lit_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("string", r#""[^"]*""#)
        .rule("start", vec!["string"])
        .start("start")
        .build()
}

/// 15. Grammar with number literal token
fn number_lit_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("number", r"[0-9]+(\.[0-9]+)?")
        .rule("start", vec!["number"])
        .start("start")
        .build()
}

/// 16. Arithmetic with +-*/
fn arith_four_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("minus", "-")
        .token("star", r"\*")
        .token("slash", r"\/")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "minus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "slash", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

/// 17. Comparison operators grammar
fn comparison_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("lt", "<")
        .token("gt", ">")
        .token("le", "<=")
        .token("ge", ">=")
        .rule_with_precedence("cmp", vec!["num", "lt", "num"], 1, Associativity::None)
        .rule_with_precedence("cmp", vec!["num", "gt", "num"], 1, Associativity::None)
        .rule_with_precedence("cmp", vec!["num", "le", "num"], 1, Associativity::None)
        .rule_with_precedence("cmp", vec!["num", "ge", "num"], 1, Associativity::None)
        .rule("start", vec!["cmp"])
        .start("start")
        .build()
}

/// 18. Logical operators grammar
fn logical_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("val", r"[a-z]+")
        .token("and", "&&")
        .token("or", "||")
        .rule_with_precedence("expr", vec!["expr", "and", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "or", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["val"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

/// 19. Grammar with optional elements (via alternatives)
fn optional_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("kw_return", "return")
        .token("val", r"\d+")
        .token("semi", ";")
        .rule("start", vec!["kw_return", "val", "semi"])
        .rule("start", vec!["kw_return", "semi"])
        .start("start")
        .build()
}

/// 20. Grammar with repeated elements (via recursion)
fn repeat_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("item", r"[a-z]+")
        .rule("items", vec!["item"])
        .rule("items", vec!["items", "item"])
        .rule("start", vec!["items"])
        .start("start")
        .build()
}

/// 21. Triple token sequence
fn triple_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build()
}

/// 22. Chain of non-terminals
fn chain3_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("x", "x")
        .rule("c", vec!["x"])
        .rule("b", vec!["c"])
        .rule("a", vec!["b"])
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// 23. Grammar with right associativity
fn right_assoc_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("pow", r"\^")
        .rule_with_precedence("expr", vec!["expr", "pow", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

/// 24. Parenthesized expression grammar
fn paren_expr_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("lparen", r"\(")
        .token("rparen", r"\)")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["lparen", "expr", "rparen"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

/// 25. Let binding grammar
fn let_binding_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("kw_let", "let")
        .token("ident", r"[a-z]+")
        .token("eq", "=")
        .token("num", r"\d+")
        .token("semi", ";")
        .rule("start", vec!["kw_let", "ident", "eq", "num", "semi"])
        .start("start")
        .build()
}

/// 26. While loop grammar
fn while_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("kw_while", "while")
        .token("cond", r"[a-z]+")
        .token("lbrace", r"\{")
        .token("rbrace", r"\}")
        .token("body", r"\d+")
        .rule("start", vec!["kw_while", "cond", "lbrace", "body", "rbrace"])
        .start("start")
        .build()
}

/// 27. Dot access grammar
fn dot_access_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("ident", r"[a-z]+")
        .token("dot", r"\.")
        .rule("access", vec!["ident"])
        .rule("access", vec!["access", "dot", "ident"])
        .rule("start", vec!["access"])
        .start("start")
        .build()
}

/// 28. Array index grammar
fn array_index_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("ident", r"[a-z]+")
        .token("lbracket", r"\[")
        .token("rbracket", r"\]")
        .token("num", r"\d+")
        .rule("start", vec!["ident", "lbracket", "num", "rbracket"])
        .start("start")
        .build()
}

/// 29. Ternary conditional grammar
fn ternary_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("val", r"[a-z]+")
        .token("qmark", r"\?")
        .token("colon", ":")
        .rule("start", vec!["val", "qmark", "val", "colon", "val"])
        .start("start")
        .build()
}

/// 30. Pipeline operator grammar
fn pipe_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("val", r"[a-z]+")
        .token("pipe", r"\|>")
        .rule_with_precedence("expr", vec!["expr", "pipe", "val"], 1, Associativity::Left)
        .rule("expr", vec!["val"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

/// 31. Tag grammar (XML-like open/close)
fn tag_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("lt", "<")
        .token("gt", ">")
        .token("ltslash", "</")
        .token("name", r"[a-z]+")
        .rule("start", vec!["lt", "name", "gt", "ltslash", "name", "gt"])
        .start("start")
        .build()
}

/// 32. Comma separated pair
fn comma_pair_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("comma", ",")
        .rule("start", vec!["num", "comma", "num"])
        .start("start")
        .build()
}

/// 33. Semicolon-terminated list
fn semi_list_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("word", r"[a-z]+")
        .token("semi", ";")
        .rule("stmt", vec!["word", "semi"])
        .rule("stmts", vec!["stmt"])
        .rule("stmts", vec!["stmts", "stmt"])
        .rule("start", vec!["stmts"])
        .start("start")
        .build()
}

/// 34. Grammar with inline rules
fn inline_alt_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("x", "x")
        .token("y", "y")
        .rule("choice", vec!["x"])
        .rule("choice", vec!["y"])
        .rule("start", vec!["choice"])
        .inline("choice")
        .start("start")
        .build()
}

/// 35. Grammar with supertype node
fn supertype_expr_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("str", r#""[^"]*""#)
        .rule("number", vec!["num"])
        .rule("string", vec!["str"])
        .rule("literal", vec!["number"])
        .rule("literal", vec!["string"])
        .supertype("literal")
        .rule("start", vec!["literal"])
        .start("start")
        .build()
}

/// 36. External token grammar
fn external_tok_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("word", r"[a-z]+")
        .token("indent", "INDENT")
        .external("indent")
        .rule("start", vec!["word"])
        .start("start")
        .build()
}

/// 37. Boolean literal grammar
fn bool_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("kw_true", "true")
        .token("kw_false", "false")
        .rule("start", vec!["kw_true"])
        .rule("start", vec!["kw_false"])
        .start("start")
        .build()
}

/// 38. Type annotation grammar
fn type_annot_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("ident", r"[a-z]+")
        .token("colon", ":")
        .token("ty", r"[A-Z][a-z]+")
        .rule("start", vec!["ident", "colon", "ty"])
        .start("start")
        .build()
}

/// 39. Import grammar
fn import_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("kw_import", "import")
        .token("path", r"[a-z./]+")
        .token("semi", ";")
        .rule("start", vec!["kw_import", "path", "semi"])
        .start("start")
        .build()
}

/// 40. Arrow function grammar
fn arrow_fn_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("param", r"[a-z]+")
        .token("arrow", "=>")
        .token("body", r"\d+")
        .rule("start", vec!["param", "arrow", "body"])
        .start("start")
        .build()
}

/// 41. Multiline block with newlines as extras
fn newline_extra_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("word", r"[a-z]+")
        .token("nl", r"\n")
        .rule("start", vec!["word"])
        .extra("nl")
        .start("start")
        .build()
}

/// 42. Two extras grammar (ws + comments)
fn two_extras_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("word", r"[a-z]+")
        .token("ws", r"[ \t]+")
        .token("comment", r"//[^\n]*")
        .rule("start", vec!["word"])
        .extra("ws")
        .extra("comment")
        .start("start")
        .build()
}

/// 43. Mixed precedence grammar
fn mixed_prec_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .token("pow", r"\^")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "pow", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

/// 44. None associativity comparison
fn none_assoc_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("eqeq", "==")
        .rule_with_precedence("expr", vec!["num", "eqeq", "num"], 1, Associativity::None)
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

/// 45. Deep chain grammar (5 levels)
fn deep_chain_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("x", "x")
        .rule("e", vec!["x"])
        .rule("d", vec!["e"])
        .rule("c", vec!["d"])
        .rule("b", vec!["c"])
        .rule("a", vec!["b"])
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// 46. Wide alternatives (4 alternatives)
fn wide_alt_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .rule("start", vec!["d"])
        .start("start")
        .build()
}

/// 47. Key-value grammar
fn kv_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("key", r"[a-z]+")
        .token("colon", ":")
        .token("val", r"\d+")
        .rule("pair", vec!["key", "colon", "val"])
        .rule("start", vec!["pair"])
        .start("start")
        .build()
}

/// 48. Nested parentheses grammar
fn nested_paren_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("lp", r"\(")
        .token("rp", r"\)")
        .token("val", r"[a-z]+")
        .rule("inner", vec!["val"])
        .rule("inner", vec!["lp", "inner", "rp"])
        .rule("start", vec!["inner"])
        .start("start")
        .build()
}

/// 49. Labeled statement grammar
fn label_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("label", r"[a-z]+")
        .token("colon", ":")
        .token("stmt", r"\d+")
        .rule("start", vec!["label", "colon", "stmt"])
        .start("start")
        .build()
}

/// 50. Spread operator grammar
fn spread_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("dots", r"\.\.\.")
        .token("ident", r"[a-z]+")
        .rule("start", vec!["dots", "ident"])
        .start("start")
        .build()
}

/// 51. Regex token with complex pattern
fn complex_regex_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("hex", r"0x[0-9a-fA-F]+")
        .rule("start", vec!["hex"])
        .start("start")
        .build()
}

/// 52. Two-rule alternatives with different lengths
fn mixed_len_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build()
}

/// 53. Grammar with fragile token
fn fragile_tok_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .fragile_token("ws", r"[ \t]+")
        .token("word", r"[a-z]+")
        .rule("start", vec!["word"])
        .extra("ws")
        .start("start")
        .build()
}

/// 54. Switch/match grammar (multiple case arms)
fn switch_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("kw_match", "match")
        .token("val", r"[a-z]+")
        .token("lbrace", r"\{")
        .token("rbrace", r"\}")
        .token("num", r"\d+")
        .rule("arm", vec!["num"])
        .rule("arms", vec!["arm"])
        .rule("arms", vec!["arms", "arm"])
        .rule("start", vec!["kw_match", "val", "lbrace", "arms", "rbrace"])
        .start("start")
        .build()
}

/// 55. For-in loop grammar
fn for_in_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("kw_for", "for")
        .token("kw_in", "in")
        .token("ident", r"[a-z]+")
        .token("range", r"\d+")
        .rule("start", vec!["kw_for", "ident", "kw_in", "range"])
        .start("start")
        .build()
}

/// 56. Decorator / annotation grammar
fn decorator_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("at", "@")
        .token("name", r"[a-z]+")
        .token("item", r"\d+")
        .rule("start", vec!["at", "name", "item"])
        .start("start")
        .build()
}

/// 57. Concatenation operator grammar
fn concat_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("str", r#""[^"]*""#)
        .token("plusplus", "++")
        .rule_with_precedence("expr", vec!["expr", "plusplus", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["str"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

/// 58. Range expression grammar
fn range_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("dotdot", r"\.\.")
        .rule("start", vec!["num", "dotdot", "num"])
        .start("start")
        .build()
}

/// 59. Cast expression grammar
fn cast_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("val", r"\d+")
        .token("kw_as", "as")
        .token("ty", r"[A-Z][a-z]+")
        .rule("start", vec!["val", "kw_as", "ty"])
        .start("start")
        .build()
}

/// 60. Enum variant grammar
fn enum_variant_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("name", r"[A-Z][a-z]+")
        .token("pipe", r"\|")
        .rule("variant", vec!["name"])
        .rule("variants", vec!["variant"])
        .rule("variants", vec!["variants", "pipe", "variant"])
        .rule("start", vec!["variants"])
        .start("start")
        .build()
}

/// 61. Tuple grammar
fn tuple_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("lp", r"\(")
        .token("rp", r"\)")
        .token("num", r"\d+")
        .token("comma", ",")
        .rule("start", vec!["lp", "num", "comma", "num", "rp"])
        .start("start")
        .build()
}

/// 62. Negation grammar
fn negation_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("bang", "!")
        .token("val", r"[a-z]+")
        .rule("expr", vec!["bang", "expr"])
        .rule("expr", vec!["val"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

/// 63. Ternary with nesting
fn nested_ternary_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("val", r"[a-z]+")
        .token("qmark", r"\?")
        .token("colon", ":")
        .rule("expr", vec!["val"])
        .rule("expr", vec!["expr", "qmark", "expr", "colon", "expr"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

/// 64. Multi-keyword grammar
fn multi_kw_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("kw_pub", "pub")
        .token("kw_fn", "fn")
        .token("name", r"[a-z]+")
        .token("lp", r"\(")
        .token("rp", r"\)")
        .rule("start", vec!["kw_pub", "kw_fn", "name", "lp", "rp"])
        .start("start")
        .build()
}

/// 65. Index assignment grammar
fn index_assign_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("ident", r"[a-z]+")
        .token("lb", r"\[")
        .token("rb", r"\]")
        .token("eq", "=")
        .token("num", r"\d+")
        .rule("start", vec!["ident", "lb", "num", "rb", "eq", "num"])
        .start("start")
        .build()
}

/// 66. Bitwise operators grammar
fn bitwise_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("band", "&")
        .token("bor", r"\|")
        .rule_with_precedence("expr", vec!["expr", "band", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "bor", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

/// 67. Lambda grammar
fn lambda_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("kw_fn", "fn")
        .token("lp", r"\(")
        .token("rp", r"\)")
        .token("arrow", "->")
        .token("body", r"\d+")
        .rule("start", vec!["kw_fn", "lp", "rp", "arrow", "body"])
        .start("start")
        .build()
}

/// 68. Try-catch grammar
fn try_catch_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("kw_try", "try")
        .token("kw_catch", "catch")
        .token("lb", r"\{")
        .token("rb", r"\}")
        .token("body", r"[a-z]+")
        .rule("block", vec!["lb", "body", "rb"])
        .rule("start", vec!["kw_try", "block", "kw_catch", "block"])
        .start("start")
        .build()
}

/// 69. Pattern match arm grammar
fn match_arm_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("pat", r"[a-z]+")
        .token("arrow", "=>")
        .token("body", r"\d+")
        .token("comma", ",")
        .rule("arm", vec!["pat", "arrow", "body"])
        .rule("arms", vec!["arm"])
        .rule("arms", vec!["arms", "comma", "arm"])
        .rule("start", vec!["arms"])
        .start("start")
        .build()
}

/// 70. Namespace/module path grammar
fn namespace_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("ident", r"[a-z]+")
        .token("coloncolon", "::")
        .rule("path", vec!["ident"])
        .rule("path", vec!["path", "coloncolon", "ident"])
        .rule("start", vec!["path"])
        .start("start")
        .build()
}

/// 71. Prefix increment grammar
fn prefix_inc_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("plusplus", r"\+\+")
        .token("ident", r"[a-z]+")
        .rule("start", vec!["plusplus", "ident"])
        .start("start")
        .build()
}

/// 72. Postfix question mark (option chaining)
fn postfix_qmark_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("ident", r"[a-z]+")
        .token("qmark", r"\?")
        .rule("start", vec!["ident", "qmark"])
        .start("start")
        .build()
}

/// 73. Const declaration grammar
fn const_decl_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("kw_const", "const")
        .token("name", r"[A-Z_]+")
        .token("eq", "=")
        .token("num", r"\d+")
        .token("semi", ";")
        .rule("start", vec!["kw_const", "name", "eq", "num", "semi"])
        .start("start")
        .build()
}

/// 74. Attribute bracket grammar
fn attr_bracket_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("hash", "#")
        .token("lb", r"\[")
        .token("rb", r"\]")
        .token("name", r"[a-z]+")
        .rule("start", vec!["hash", "lb", "name", "rb"])
        .start("start")
        .build()
}

/// 75. Variadic args grammar
fn variadic_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("name", r"[a-z]+")
        .token("lp", r"\(")
        .token("rp", r"\)")
        .token("dots", r"\.\.\.")
        .rule("start", vec!["name", "lp", "dots", "rp"])
        .start("start")
        .build()
}

/// 76. Slice syntax grammar
fn slice_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("ident", r"[a-z]+")
        .token("lb", r"\[")
        .token("rb", r"\]")
        .token("num", r"\d+")
        .token("dotdot", r"\.\.")
        .rule("start", vec!["ident", "lb", "num", "dotdot", "num", "rb"])
        .start("start")
        .build()
}

/// 77. Chained method calls grammar
fn method_chain_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("ident", r"[a-z]+")
        .token("dot", r"\.")
        .token("lp", r"\(")
        .token("rp", r"\)")
        .rule("call", vec!["ident", "lp", "rp"])
        .rule("chain", vec!["call"])
        .rule("chain", vec!["chain", "dot", "call"])
        .rule("start", vec!["chain"])
        .start("start")
        .build()
}

/// 78. Generic type grammar
fn generic_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("name", r"[A-Z][a-z]+")
        .token("lt", "<")
        .token("gt", ">")
        .rule("start", vec!["name", "lt", "name", "gt"])
        .start("start")
        .build()
}

/// 79. Compound assignment grammar
fn compound_assign_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("ident", r"[a-z]+")
        .token("pluseq", r"\+=")
        .token("num", r"\d+")
        .rule("start", vec!["ident", "pluseq", "num"])
        .start("start")
        .build()
}

/// 80. Empty-body alternative grammar
fn empty_body_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("lb", r"\{")
        .token("rb", r"\}")
        .token("word", r"[a-z]+")
        .rule("start", vec!["lb", "rb"])
        .rule("start", vec!["lb", "word", "rb"])
        .start("start")
        .build()
}

/// 81. Six-token sequence grammar
fn six_seq_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .token("f", "f")
        .rule("start", vec!["a", "b", "c", "d", "e", "f"])
        .start("start")
        .build()
}

/// 82. Deeply nested alternatives
fn deep_alt_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("inner", vec!["x"])
        .rule("inner", vec!["y"])
        .rule("outer", vec!["inner"])
        .rule("outer", vec!["z"])
        .rule("start", vec!["outer"])
        .start("start")
        .build()
}

/// 83. Multi-level precedence (4 levels)
fn four_prec_grammar(name: &str) -> adze_ir::Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .token("pow", r"\^")
        .token("eq", "==")
        .rule_with_precedence("expr", vec!["expr", "eq", "expr"], 1, Associativity::None)
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 3, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "pow", "expr"], 4, Associativity::Right)
        .rule("expr", vec!["num"])
        .rule("start", vec!["expr"])
        .start("start")
        .build()
}

// ═════════════════════════════════════════════════════════════════════════
// 1. Minimal grammar — single token, single rule
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_minimal_builds_ok() {
    let r = build(minimal_grammar("mg_v8_min"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_minimal_state_positive() {
    let r = build(minimal_grammar("mg_v8_min_st"));
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_minimal_name() {
    let r = build(minimal_grammar("mg_v8_min_nm"));
    assert_eq!(r.grammar_name, "mg_v8_min_nm");
}

// ═════════════════════════════════════════════════════════════════════════
// 2. Pair grammar — two tokens, single rule
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_pair_builds_ok() {
    let r = build(pair_grammar("mg_v8_pair"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_pair_symbols_gte_two() {
    let r = build(pair_grammar("mg_v8_pair_sy"));
    assert!(r.build_stats.symbol_count >= 2);
}

// ═════════════════════════════════════════════════════════════════════════
// 3. Keyword + identifier grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_keyword_ident_builds_ok() {
    let r = build(keyword_ident_grammar("mg_v8_kwid"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_keyword_ident_node_types_valid() {
    let r = build(keyword_ident_grammar("mg_v8_kwid_nt"));
    let v = parse_node_types(&r.node_types_json);
    assert!(v.is_array());
}

// ═════════════════════════════════════════════════════════════════════════
// 4. Number + operator grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_num_op_builds_ok() {
    let r = build(num_op_grammar("mg_v8_numop"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_num_op_symbols_gte_two() {
    let r = build(num_op_grammar("mg_v8_numop_sy"));
    assert!(r.build_stats.symbol_count >= 2);
}

// ═════════════════════════════════════════════════════════════════════════
// 5. If-then-else grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_if_then_else_builds_ok() {
    let r = build(if_then_else_grammar("mg_v8_ite"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_if_then_else_state_positive() {
    let r = build(if_then_else_grammar("mg_v8_ite_st"));
    assert!(r.build_stats.state_count > 0);
}

// ═════════════════════════════════════════════════════════════════════════
// 6. Function call grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_fn_call_builds_ok() {
    let r = build(fn_call_grammar("mg_v8_fncall"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_fn_call_symbols_gte_four() {
    let r = build(fn_call_grammar("mg_v8_fncall_sy"));
    assert!(r.build_stats.symbol_count >= 4);
}

// ═════════════════════════════════════════════════════════════════════════
// 7. List grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_list_builds_ok() {
    let r = build(list_grammar("mg_v8_list"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_list_node_types_valid() {
    let r = build(list_grammar("mg_v8_list_nt"));
    let v = parse_node_types(&r.node_types_json);
    assert!(v.is_array());
}

// ═════════════════════════════════════════════════════════════════════════
// 8. Binary expression grammar (left-recursive)
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_binexpr_builds_ok() {
    let r = build(binexpr_grammar("mg_v8_binex"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_binexpr_state_positive() {
    let r = build(binexpr_grammar("mg_v8_binex_st"));
    assert!(r.build_stats.state_count > 0);
}

// ═════════════════════════════════════════════════════════════════════════
// 9. Unary expression grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_unary_builds_ok() {
    let r = build(unary_grammar("mg_v8_unary"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_unary_symbols_positive() {
    let r = build(unary_grammar("mg_v8_unary_sy"));
    assert!(r.build_stats.symbol_count > 0);
}

// ═════════════════════════════════════════════════════════════════════════
// 10. Statement grammar (assignment)
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_assign_builds_ok() {
    let r = build(assign_grammar("mg_v8_asgn"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_assign_name() {
    let r = build(assign_grammar("mg_v8_asgn_nm"));
    assert_eq!(r.grammar_name, "mg_v8_asgn_nm");
}

// ═════════════════════════════════════════════════════════════════════════
// 11. Block grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_block_builds_ok() {
    let r = build(block_grammar("mg_v8_blk"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_block_state_positive() {
    let r = build(block_grammar("mg_v8_blk_st"));
    assert!(r.build_stats.state_count > 0);
}

// ═════════════════════════════════════════════════════════════════════════
// 12. Grammar with whitespace extra
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_ws_extra_builds_ok() {
    let r = build(ws_extra_grammar("mg_v8_wsx"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_ws_extra_node_types_valid() {
    let r = build(ws_extra_grammar("mg_v8_wsx_nt"));
    let v = parse_node_types(&r.node_types_json);
    assert!(v.is_array());
}

// ═════════════════════════════════════════════════════════════════════════
// 13. Grammar with comment token
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_comment_builds_ok() {
    let r = build(comment_grammar("mg_v8_cmt"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_comment_symbols_positive() {
    let r = build(comment_grammar("mg_v8_cmt_sy"));
    assert!(r.build_stats.symbol_count > 0);
}

// ═════════════════════════════════════════════════════════════════════════
// 14. Grammar with string literal token
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_string_lit_builds_ok() {
    let r = build(string_lit_grammar("mg_v8_strlit"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 15. Grammar with number literal token
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_number_lit_builds_ok() {
    let r = build(number_lit_grammar("mg_v8_numlit"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 16. Arithmetic with +-*/
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_arith_four_builds_ok() {
    let r = build(arith_four_grammar("mg_v8_a4"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_arith_four_symbols_gte_five() {
    let r = build(arith_four_grammar("mg_v8_a4_sy"));
    assert!(r.build_stats.symbol_count >= 5);
}

#[test]
fn test_arith_four_state_positive() {
    let r = build(arith_four_grammar("mg_v8_a4_st"));
    assert!(r.build_stats.state_count > 0);
}

// ═════════════════════════════════════════════════════════════════════════
// 17. Comparison operators grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_comparison_builds_ok() {
    let r = build(comparison_grammar("mg_v8_cmp"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_comparison_node_types_valid() {
    let r = build(comparison_grammar("mg_v8_cmp_nt"));
    let v = parse_node_types(&r.node_types_json);
    assert!(v.is_array());
}

// ═════════════════════════════════════════════════════════════════════════
// 18. Logical operators grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_logical_builds_ok() {
    let r = build(logical_grammar("mg_v8_log"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_logical_state_positive() {
    let r = build(logical_grammar("mg_v8_log_st"));
    assert!(r.build_stats.state_count > 0);
}

// ═════════════════════════════════════════════════════════════════════════
// 19. Grammar with optional elements (via alternatives)
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_optional_builds_ok() {
    let r = build(optional_grammar("mg_v8_opt"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_optional_name() {
    let r = build(optional_grammar("mg_v8_opt_nm"));
    assert_eq!(r.grammar_name, "mg_v8_opt_nm");
}

// ═════════════════════════════════════════════════════════════════════════
// 20. Grammar with repeated elements (via recursion)
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_repeat_builds_ok() {
    let r = build(repeat_grammar("mg_v8_rep"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_repeat_state_positive() {
    let r = build(repeat_grammar("mg_v8_rep_st"));
    assert!(r.build_stats.state_count > 0);
}

// ═════════════════════════════════════════════════════════════════════════
// 21. Triple token sequence
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_triple_builds_ok() {
    let r = build(triple_grammar("mg_v8_tri"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 22. Chain of non-terminals (3 levels)
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_chain3_builds_ok() {
    let r = build(chain3_grammar("mg_v8_ch3"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_chain3_symbols_gte_four() {
    let r = build(chain3_grammar("mg_v8_ch3_sy"));
    assert!(r.build_stats.symbol_count >= 4);
}

// ═════════════════════════════════════════════════════════════════════════
// 23. Right associativity
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_right_assoc_builds_ok() {
    let r = build(right_assoc_grammar("mg_v8_ras"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 24. Parenthesized expression
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_paren_expr_builds_ok() {
    let r = build(paren_expr_grammar("mg_v8_pex"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_paren_expr_state_positive() {
    let r = build(paren_expr_grammar("mg_v8_pex_st"));
    assert!(r.build_stats.state_count > 0);
}

// ═════════════════════════════════════════════════════════════════════════
// 25. Let binding
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_let_binding_builds_ok() {
    let r = build(let_binding_grammar("mg_v8_let"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_let_binding_symbols_gte_five() {
    let r = build(let_binding_grammar("mg_v8_let_sy"));
    assert!(r.build_stats.symbol_count >= 5);
}

// ═════════════════════════════════════════════════════════════════════════
// 26. While loop
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_while_builds_ok() {
    let r = build(while_grammar("mg_v8_whi"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 27. Dot access
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_dot_access_builds_ok() {
    let r = build(dot_access_grammar("mg_v8_dot"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_dot_access_node_types_valid() {
    let r = build(dot_access_grammar("mg_v8_dot_nt"));
    let v = parse_node_types(&r.node_types_json);
    assert!(v.is_array());
}

// ═════════════════════════════════════════════════════════════════════════
// 28. Array index
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_array_index_builds_ok() {
    let r = build(array_index_grammar("mg_v8_arr"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 29. Ternary conditional
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_ternary_builds_ok() {
    let r = build(ternary_grammar("mg_v8_tern"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 30. Pipeline operator
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_pipe_builds_ok() {
    let r = build(pipe_grammar("mg_v8_pipe"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 31. Tag grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_tag_builds_ok() {
    let r = build(tag_grammar("mg_v8_tag"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 32. Comma separated pair
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_comma_pair_builds_ok() {
    let r = build(comma_pair_grammar("mg_v8_cpair"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 33. Semicolon-terminated list
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_semi_list_builds_ok() {
    let r = build(semi_list_grammar("mg_v8_slist"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_semi_list_state_positive() {
    let r = build(semi_list_grammar("mg_v8_slist_st"));
    assert!(r.build_stats.state_count > 0);
}

// ═════════════════════════════════════════════════════════════════════════
// 34. Inline rules
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_inline_alt_builds_ok() {
    let r = build(inline_alt_grammar("mg_v8_inl"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 35. Supertype node
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_supertype_expr_builds_ok() {
    let r = build(supertype_expr_grammar("mg_v8_sup"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 36. External token
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_external_tok_builds_ok() {
    let r = build(external_tok_grammar("mg_v8_ext"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 37. Boolean literal grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_bool_builds_ok() {
    let r = build(bool_grammar("mg_v8_bool"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 38. Type annotation grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_type_annot_builds_ok() {
    let r = build(type_annot_grammar("mg_v8_tyann"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 39. Import grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_import_builds_ok() {
    let r = build(import_grammar("mg_v8_imp"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 40. Arrow function grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_arrow_fn_builds_ok() {
    let r = build(arrow_fn_grammar("mg_v8_arfn"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 41. Newline extra
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_newline_extra_builds_ok() {
    let r = build(newline_extra_grammar("mg_v8_nlx"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 42. Two extras (ws + comments)
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_two_extras_builds_ok() {
    let r = build(two_extras_grammar("mg_v8_2x"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 43. Mixed precedence
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_mixed_prec_builds_ok() {
    let r = build(mixed_prec_grammar("mg_v8_mixp"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_mixed_prec_state_positive() {
    let r = build(mixed_prec_grammar("mg_v8_mixp_st"));
    assert!(r.build_stats.state_count > 0);
}

// ═════════════════════════════════════════════════════════════════════════
// 44. None associativity comparison
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_none_assoc_builds_ok() {
    let r = build(none_assoc_grammar("mg_v8_noas"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 45. Deep chain (5 levels)
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_deep_chain_builds_ok() {
    let r = build(deep_chain_grammar("mg_v8_dch"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_deep_chain_symbols_gte_six() {
    let r = build(deep_chain_grammar("mg_v8_dch_sy"));
    // start + a + b + c + d + e + x = 7 symbols minimum
    assert!(r.build_stats.symbol_count >= 6);
}

// ═════════════════════════════════════════════════════════════════════════
// 46. Wide alternatives (4 choices)
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_wide_alt_builds_ok() {
    let r = build(wide_alt_grammar("mg_v8_walt"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 47. Key-value grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_kv_builds_ok() {
    let r = build(kv_grammar("mg_v8_kv"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 48. Nested parentheses
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_nested_paren_builds_ok() {
    let r = build(nested_paren_grammar("mg_v8_nparen"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 49. Labeled statement
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_label_builds_ok() {
    let r = build(label_grammar("mg_v8_lbl"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 50. Spread operator
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_spread_builds_ok() {
    let r = build(spread_grammar("mg_v8_sprd"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 51. Complex regex token
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_complex_regex_builds_ok() {
    let r = build(complex_regex_grammar("mg_v8_crx"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 52. Mixed-length alternatives
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_mixed_len_builds_ok() {
    let r = build(mixed_len_grammar("mg_v8_mxln"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 53. Fragile token
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_fragile_tok_builds_ok() {
    let r = build(fragile_tok_grammar("mg_v8_frag"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 54. Switch/match grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_switch_builds_ok() {
    let r = build(switch_grammar("mg_v8_swt"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 55. For-in loop grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_for_in_builds_ok() {
    let r = build(for_in_grammar("mg_v8_forin"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 56. Decorator grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_decorator_builds_ok() {
    let r = build(decorator_grammar("mg_v8_deco"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 57. Concatenation operator grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_concat_builds_ok() {
    let r = build(concat_grammar("mg_v8_cat"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 58. Range expression grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_range_builds_ok() {
    let r = build(range_grammar("mg_v8_rng"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 59. Cast expression grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_cast_builds_ok() {
    let r = build(cast_grammar("mg_v8_cast"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 60. Enum variant grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_enum_variant_builds_ok() {
    let r = build(enum_variant_grammar("mg_v8_evar"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 61. Tuple grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_tuple_builds_ok() {
    let r = build(tuple_grammar("mg_v8_tup"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 62. Negation grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_negation_builds_ok() {
    let r = build(negation_grammar("mg_v8_neg"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 63. Nested ternary
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_nested_ternary_builds_ok() {
    let r = build(nested_ternary_grammar("mg_v8_ntern"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 64. Multi-keyword grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_multi_kw_builds_ok() {
    let r = build(multi_kw_grammar("mg_v8_mkw"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 65. Index assignment grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_index_assign_builds_ok() {
    let r = build(index_assign_grammar("mg_v8_ixas"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 66. Bitwise operators grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_bitwise_builds_ok() {
    let r = build(bitwise_grammar("mg_v8_bw"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 67. Lambda grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_lambda_builds_ok() {
    let r = build(lambda_grammar("mg_v8_lam"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 68. Try-catch grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_try_catch_builds_ok() {
    let r = build(try_catch_grammar("mg_v8_try"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 69. Pattern match arm grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_match_arm_builds_ok() {
    let r = build(match_arm_grammar("mg_v8_marm"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 70. Namespace path grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_namespace_builds_ok() {
    let r = build(namespace_grammar("mg_v8_ns"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 71. Prefix increment grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_prefix_inc_builds_ok() {
    let r = build(prefix_inc_grammar("mg_v8_pinc"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 72. Postfix question mark
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_postfix_qmark_builds_ok() {
    let r = build(postfix_qmark_grammar("mg_v8_pqm"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 73. Const declaration grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_const_decl_builds_ok() {
    let r = build(const_decl_grammar("mg_v8_cnst"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 74. Attribute bracket grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_attr_bracket_builds_ok() {
    let r = build(attr_bracket_grammar("mg_v8_attr"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 75. Variadic args grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_variadic_builds_ok() {
    let r = build(variadic_grammar("mg_v8_var"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 76. Slice syntax grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_slice_builds_ok() {
    let r = build(slice_grammar("mg_v8_slc"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 77. Method chain grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_method_chain_builds_ok() {
    let r = build(method_chain_grammar("mg_v8_mch"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 78. Generic type grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_generic_builds_ok() {
    let r = build(generic_grammar("mg_v8_gen"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 79. Compound assignment grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_compound_assign_builds_ok() {
    let r = build(compound_assign_grammar("mg_v8_cpas"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 80. Empty-body alternative grammar
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_body_builds_ok() {
    let r = build(empty_body_grammar("mg_v8_ebody"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 81. Six-token sequence
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_six_seq_builds_ok() {
    let r = build(six_seq_grammar("mg_v8_6seq"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_six_seq_symbols_gte_six() {
    let r = build(six_seq_grammar("mg_v8_6seq_sy"));
    assert!(r.build_stats.symbol_count >= 6);
}

// ═════════════════════════════════════════════════════════════════════════
// 82. Deeply nested alternatives
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_deep_alt_builds_ok() {
    let r = build(deep_alt_grammar("mg_v8_dalt"));
    assert!(!r.parser_code.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════
// 83. Multi-level precedence (4 levels)
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_four_prec_builds_ok() {
    let r = build(four_prec_grammar("mg_v8_4prec"));
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_four_prec_symbols_gte_five() {
    let r = build(four_prec_grammar("mg_v8_4prec_sy"));
    assert!(r.build_stats.symbol_count >= 5);
}

// ═════════════════════════════════════════════════════════════════════════
// Cross-grammar validation tests
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_all_grammars_produce_nonempty_parser_code() {
    let grammars: Vec<adze_ir::Grammar> = vec![
        minimal_grammar("mg_v8_all_1"),
        pair_grammar("mg_v8_all_2"),
        keyword_ident_grammar("mg_v8_all_3"),
        num_op_grammar("mg_v8_all_4"),
        if_then_else_grammar("mg_v8_all_5"),
        fn_call_grammar("mg_v8_all_6"),
        list_grammar("mg_v8_all_7"),
        binexpr_grammar("mg_v8_all_8"),
        unary_grammar("mg_v8_all_9"),
        assign_grammar("mg_v8_all_10"),
    ];
    for g in grammars {
        let r = build(g);
        assert!(!r.parser_code.is_empty());
    }
}

#[test]
fn test_all_grammars_produce_valid_node_types_json() {
    let grammars: Vec<adze_ir::Grammar> = vec![
        block_grammar("mg_v8_json_1"),
        ws_extra_grammar("mg_v8_json_2"),
        comment_grammar("mg_v8_json_3"),
        string_lit_grammar("mg_v8_json_4"),
        number_lit_grammar("mg_v8_json_5"),
        arith_four_grammar("mg_v8_json_6"),
        comparison_grammar("mg_v8_json_7"),
        logical_grammar("mg_v8_json_8"),
        optional_grammar("mg_v8_json_9"),
        repeat_grammar("mg_v8_json_10"),
    ];
    for g in grammars {
        let r = build(g);
        let v = parse_node_types(&r.node_types_json);
        assert!(v.is_array());
    }
}

#[test]
fn test_all_grammars_have_positive_state_count() {
    let grammars: Vec<adze_ir::Grammar> = vec![
        triple_grammar("mg_v8_stc_1"),
        chain3_grammar("mg_v8_stc_2"),
        right_assoc_grammar("mg_v8_stc_3"),
        paren_expr_grammar("mg_v8_stc_4"),
        let_binding_grammar("mg_v8_stc_5"),
        while_grammar("mg_v8_stc_6"),
        dot_access_grammar("mg_v8_stc_7"),
        array_index_grammar("mg_v8_stc_8"),
        ternary_grammar("mg_v8_stc_9"),
        pipe_grammar("mg_v8_stc_10"),
    ];
    for g in grammars {
        let r = build(g);
        assert!(r.build_stats.state_count > 0);
    }
}

#[test]
fn test_complex_grammars_have_more_states_than_minimal() {
    let min = build(minimal_grammar("mg_v8_cplx_min"));
    let complex = build(arith_four_grammar("mg_v8_cplx_a4"));
    assert!(complex.build_stats.state_count >= min.build_stats.state_count);
}

#[test]
fn test_complex_grammars_have_more_symbols_than_minimal() {
    let min = build(minimal_grammar("mg_v8_sym_min"));
    let complex = build(four_prec_grammar("mg_v8_sym_4p"));
    assert!(complex.build_stats.symbol_count > min.build_stats.symbol_count);
}

#[test]
fn test_different_grammars_produce_different_code() {
    let r1 = build(minimal_grammar("mg_v8_diff_1"));
    let r2 = build(arith_four_grammar("mg_v8_diff_2"));
    assert_ne!(r1.parser_code, r2.parser_code);
}

#[test]
fn test_parser_path_nonempty_for_all_shapes() {
    let grammars: Vec<adze_ir::Grammar> = vec![
        tag_grammar("mg_v8_pp_1"),
        comma_pair_grammar("mg_v8_pp_2"),
        semi_list_grammar("mg_v8_pp_3"),
        inline_alt_grammar("mg_v8_pp_4"),
        supertype_expr_grammar("mg_v8_pp_5"),
        external_tok_grammar("mg_v8_pp_6"),
        bool_grammar("mg_v8_pp_7"),
        type_annot_grammar("mg_v8_pp_8"),
        import_grammar("mg_v8_pp_9"),
        arrow_fn_grammar("mg_v8_pp_10"),
    ];
    for g in grammars {
        let r = build(g);
        assert!(!r.parser_path.is_empty());
    }
}

#[test]
fn test_grammar_name_preserved_in_result() {
    let names = [
        "mg_v8_nm_a", "mg_v8_nm_b", "mg_v8_nm_c", "mg_v8_nm_d", "mg_v8_nm_e",
    ];
    for nm in names {
        let r = build(minimal_grammar(nm));
        assert_eq!(r.grammar_name, nm);
    }
}

#[test]
fn test_parser_code_contains_grammar_name() {
    let r = build(let_binding_grammar("mg_v8_pcnm"));
    assert!(r.parser_code.contains("mg_v8_pcnm"));
}

#[test]
fn test_node_types_is_json_array_for_remaining_grammars() {
    let grammars: Vec<adze_ir::Grammar> = vec![
        newline_extra_grammar("mg_v8_ntj_1"),
        two_extras_grammar("mg_v8_ntj_2"),
        mixed_prec_grammar("mg_v8_ntj_3"),
        none_assoc_grammar("mg_v8_ntj_4"),
        deep_chain_grammar("mg_v8_ntj_5"),
        wide_alt_grammar("mg_v8_ntj_6"),
        kv_grammar("mg_v8_ntj_7"),
        nested_paren_grammar("mg_v8_ntj_8"),
        label_grammar("mg_v8_ntj_9"),
        spread_grammar("mg_v8_ntj_10"),
    ];
    for g in grammars {
        let r = build(g);
        let v = parse_node_types(&r.node_types_json);
        assert!(v.is_array());
    }
}
