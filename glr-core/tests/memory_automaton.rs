// Only compile & run these memory/profiling tests if explicitly requested.
#![cfg(feature = "memory-tests")]

#[cfg(test)]
mod memory_tests {
    use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
    use rust_sitter_ir::builder::GrammarBuilder;

    #[test]
    fn dhat_heap_lr1_automaton() {
        let _profiler = dhat::Profiler::new_heap();

        let g = GrammarBuilder::new("memdemo")
            .token("IDENT", r"[a-zA-Z_][a-zA-Z0-9_]*")
            .rule("module", vec![])
            .rule("module", vec!["IDENT"])
            .start("module")
            .build();

        let ff = FirstFollowSets::compute(&g);
        let _pt = build_lr1_automaton(&g, &ff).expect("build");

        // Drop profiler to get stats
        drop(_profiler);
    }

    #[test]
    fn dhat_heap_larger_grammar() {
        let _profiler = dhat::Profiler::new_heap();

        // Build a larger grammar to better see memory patterns
        let g = GrammarBuilder::new("larger")
            .token("NUMBER", r"\d+")
            .token("IDENT", r"[a-zA-Z_][a-zA-Z0-9_]*")
            .token("PLUS", r"\+")
            .token("MINUS", r"-")
            .token("TIMES", r"\*")
            .token("DIV", r"/")
            .token("LPAREN", r"\(")
            .token("RPAREN", r"\)")
            .token("SEMICOLON", r";")
            .token("ASSIGN", r"=")
            // Expression rules
            .rule("expr", vec!["term"])
            .rule("expr", vec!["expr", "PLUS", "term"])
            .rule("expr", vec!["expr", "MINUS", "term"])
            .rule("term", vec!["factor"])
            .rule("term", vec!["term", "TIMES", "factor"])
            .rule("term", vec!["term", "DIV", "factor"])
            .rule("factor", vec!["NUMBER"])
            .rule("factor", vec!["IDENT"])
            .rule("factor", vec!["LPAREN", "expr", "RPAREN"])
            // Statement rules
            .rule("stmt", vec!["IDENT", "ASSIGN", "expr", "SEMICOLON"])
            .rule("stmt", vec!["expr", "SEMICOLON"])
            .rule("stmts", vec![])
            .rule("stmts", vec!["stmts", "stmt"])
            .start("stmts")
            .build();

        let ff = FirstFollowSets::compute(&g);
        let pt = build_lr1_automaton(&g, &ff).expect("build");

        println!("States generated: {}", pt.state_count);

        drop(_profiler);
    }
}
