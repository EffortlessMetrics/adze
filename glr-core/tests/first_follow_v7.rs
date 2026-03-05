use adze_glr_core::FirstFollowSets;
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, SymbolId};

const EOF: SymbolId = SymbolId(0);

fn sym(g: &Grammar, name: &str) -> SymbolId {
    g.find_symbol_by_name(name).unwrap_or_else(|| panic!("symbol '{name}' not found"))
}

fn build_ff(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> (Grammar, FirstFollowSets) {
    let mut b = GrammarBuilder::new(name);
    for &(n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    b = b.start(start);
    let g = b.build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    (g, ff)
}

// ============================================================================
// Category 1: first_basic_* — basic FIRST set computation
// ============================================================================

#[test]
fn first_basic_1() {
    let (g, ff) = build_ff(
        "test1",
        &[("a", "[a]"), ("b", "[b]")],
        &[("S", vec!["a"])],
        "S",
    );
    let s = sym(&g, "S");
    let a = sym(&g, "a");
    let first_s = ff.first(s).expect("S has FIRST set");
    assert!(first_s.contains(a.0 as usize), "S FIRST contains a");
}

#[test]
fn first_basic_2() {
    let (g, ff) = build_ff(
        "test2",
        &[("x", "[x]"), ("y", "[y]")],
        &[("E", vec!["x"])],
        "E",
    );
    let x = sym(&g, "x");
    let e = sym(&g, "E");
    let first_e = ff.first(e).expect("E has FIRST set");
    assert!(first_e.contains(x.0 as usize));
}

#[test]
fn first_basic_3() {
    let (g, ff) = build_ff(
        "test3",
        &[("c", "[c]"), ("d", "[d]")],
        &[("T", vec!["c"]), ("U", vec!["d"])],
        "T",
    );
    let c = sym(&g, "c");
    let t = sym(&g, "T");
    let first_t = ff.first(t).expect("T has FIRST set");
    assert!(first_t.contains(c.0 as usize));
}

#[test]
fn first_basic_4() {
    let (g, ff) = build_ff(
        "test4",
        &[("p", "[p]"), ("q", "[q]")],
        &[("A", vec!["p"]), ("B", vec!["q"])],
        "A",
    );
    let p = sym(&g, "p");
    let a = sym(&g, "A");
    let first_a = ff.first(a).expect("A has FIRST set");
    assert!(first_a.contains(p.0 as usize));
}

#[test]
fn first_basic_5() {
    let (g, ff) = build_ff(
        "test5",
        &[("m", "[m]"), ("n", "[n]")],
        &[("X", vec!["m"]), ("Y", vec!["n"])],
        "X",
    );
    let m = sym(&g, "m");
    let x = sym(&g, "X");
    let first_x = ff.first(x).expect("X has FIRST set");
    assert!(first_x.contains(m.0 as usize));
}

#[test]
fn first_basic_6() {
    let (g, ff) = build_ff(
        "test6",
        &[("t1", "[t1]"), ("t2", "[t2]")],
        &[("Z", vec!["t1"])],
        "Z",
    );
    let t1 = sym(&g, "t1");
    let z = sym(&g, "Z");
    let first_z = ff.first(z).expect("Z has FIRST set");
    assert!(first_z.contains(t1.0 as usize));
}

#[test]
fn first_basic_7() {
    let (g, ff) = build_ff(
        "test7",
        &[("v", "[v]"), ("w", "[w]")],
        &[("P", vec!["v"]), ("Q", vec!["w"])],
        "P",
    );
    let v = sym(&g, "v");
    let p = sym(&g, "P");
    let first_p = ff.first(p).expect("P has FIRST set");
    assert!(first_p.contains(v.0 as usize));
}

#[test]
fn first_basic_8() {
    let (g, ff) = build_ff(
        "test8",
        &[("i", "[i]"), ("j", "[j]")],
        &[("K", vec!["i"]), ("L", vec!["j"])],
        "K",
    );
    let i = sym(&g, "i");
    let k = sym(&g, "K");
    let first_k = ff.first(k).expect("K has FIRST set");
    assert!(first_k.contains(i.0 as usize));
}

// ============================================================================
// Category 2: first_nullable_* — nullable symbol handling
// ============================================================================

#[test]
fn first_nullable_1() {
    let (g, ff) = build_ff(
        "test1",
        &[("tk", "[tk]")],
        &[("N", vec![]), ("M", vec!["tk", "N"])],
        "M",
    );
    let n = sym(&g, "N");
    assert!(ff.is_nullable(n), "N is nullable via empty production");
}

#[test]
fn first_nullable_2() {
    let (g, ff) = build_ff(
        "test2",
        &[("tok", "[tok]")],
        &[("A", vec![]), ("B", vec!["tok", "A"])],
        "B",
    );
    let a = sym(&g, "A");
    assert!(ff.is_nullable(a));
}

#[test]
fn first_nullable_3() {
    let (g, ff) = build_ff(
        "test3",
        &[("sym", "[sym]")],
        &[("X", vec![]), ("Y", vec!["X"]), ("Z", vec!["Y"])],
        "Z",
    );
    let x = sym(&g, "X");
    let y = sym(&g, "Y");
    let z = sym(&g, "Z");
    assert!(ff.is_nullable(x));
    assert!(ff.is_nullable(y));
    assert!(ff.is_nullable(z));
}

#[test]
fn first_nullable_4() {
    let (g, ff) = build_ff(
        "test4",
        &[("t", "[t]")],
        &[("C", vec![]), ("D", vec!["C"])],
        "D",
    );
    let c = sym(&g, "C");
    let d = sym(&g, "D");
    assert!(ff.is_nullable(c));
    assert!(ff.is_nullable(d));
}

#[test]
fn first_nullable_5() {
    let (g, ff) = build_ff(
        "test5",
        &[("s", "[s]")],
        &[("E", vec![]), ("F", vec!["E", "E"])],
        "F",
    );
    let e = sym(&g, "E");
    let f = sym(&g, "F");
    assert!(ff.is_nullable(e));
    assert!(ff.is_nullable(f));
}

#[test]
fn first_nullable_6() {
    let (g, ff) = build_ff(
        "test6",
        &[("u", "[u]")],
        &[("G", vec![]), ("H", vec!["G", "G"])],
        "H",
    );
    let g_nt = sym(&g, "G");
    assert!(ff.is_nullable(g_nt));
}

#[test]
fn first_nullable_7() {
    let (g, ff) = build_ff(
        "test7",
        &[("r", "[r]")],
        &[("I", vec![]), ("J", vec!["I"])],
        "J",
    );
    let i = sym(&g, "I");
    assert!(ff.is_nullable(i));
}

#[test]
fn first_nullable_8() {
    let (g, ff) = build_ff(
        "test8",
        &[("q", "[q]")],
        &[("O", vec![]), ("P", vec!["O"])],
        "P",
    );
    let o = sym(&g, "O");
    let p = sym(&g, "P");
    assert!(ff.is_nullable(o));
    assert!(ff.is_nullable(p));
}

// ============================================================================
// Category 3: first_multi_* — multiple alternatives FIRST
// ============================================================================

#[test]
fn first_multi_1() {
    let (g, ff) = build_ff(
        "test1",
        &[("a", "[a]"), ("b", "[b]")],
        &[("S", vec!["a"]), ("S", vec!["b"])],
        "S",
    );
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    let s = sym(&g, "S");
    let first_s = ff.first(s).expect("S has FIRST set");
    assert!(first_s.contains(a.0 as usize));
    assert!(first_s.contains(b.0 as usize));
}

#[test]
fn first_multi_2() {
    let (g, ff) = build_ff(
        "test2",
        &[("x", "[x]"), ("y", "[y]"), ("z", "[z]")],
        &[("E", vec!["x"]), ("E", vec!["y"]), ("E", vec!["z"])],
        "E",
    );
    let x = sym(&g, "x");
    let y = sym(&g, "y");
    let z = sym(&g, "z");
    let e = sym(&g, "E");
    let first_e = ff.first(e).expect("E has FIRST set");
    assert!(first_e.contains(x.0 as usize));
    assert!(first_e.contains(y.0 as usize));
    assert!(first_e.contains(z.0 as usize));
}

#[test]
fn first_multi_3() {
    let (g, ff) = build_ff(
        "test3",
        &[("p", "[p]"), ("q", "[q]")],
        &[("A", vec!["p"]), ("A", vec!["q"])],
        "A",
    );
    let p = sym(&g, "p");
    let q = sym(&g, "q");
    let a = sym(&g, "A");
    let first_a = ff.first(a).expect("A has FIRST set");
    assert!(first_a.contains(p.0 as usize));
    assert!(first_a.contains(q.0 as usize));
}

#[test]
fn first_multi_4() {
    let (g, ff) = build_ff(
        "test4",
        &[("m", "[m]"), ("n", "[n]"), ("o", "[o]")],
        &[("T", vec!["m"]), ("T", vec!["n"]), ("T", vec!["o"])],
        "T",
    );
    let m = sym(&g, "m");
    let n = sym(&g, "n");
    let o = sym(&g, "o");
    let t = sym(&g, "T");
    let first_t = ff.first(t).expect("T has FIRST set");
    assert!(first_t.contains(m.0 as usize));
    assert!(first_t.contains(n.0 as usize));
    assert!(first_t.contains(o.0 as usize));
}

#[test]
fn first_multi_5() {
    let (g, ff) = build_ff(
        "test5",
        &[("i", "[i]"), ("j", "[j]")],
        &[("U", vec!["i"]), ("U", vec!["j"])],
        "U",
    );
    let i = sym(&g, "i");
    let j = sym(&g, "j");
    let u = sym(&g, "U");
    let first_u = ff.first(u).expect("U has FIRST set");
    assert!(first_u.contains(i.0 as usize));
    assert!(first_u.contains(j.0 as usize));
}

#[test]
fn first_multi_6() {
    let (g, ff) = build_ff(
        "test6",
        &[("c", "[c]"), ("d", "[d]"), ("e", "[e]")],
        &[("V", vec!["c"]), ("V", vec!["d"]), ("V", vec!["e"])],
        "V",
    );
    let c = sym(&g, "c");
    let d = sym(&g, "d");
    let e = sym(&g, "e");
    let v = sym(&g, "V");
    let first_v = ff.first(v).expect("V has FIRST set");
    assert!(first_v.contains(c.0 as usize));
    assert!(first_v.contains(d.0 as usize));
    assert!(first_v.contains(e.0 as usize));
}

#[test]
fn first_multi_7() {
    let (g, ff) = build_ff(
        "test7",
        &[("f", "[f]"), ("g", "[g]")],
        &[("W", vec!["f"]), ("W", vec!["g"])],
        "W",
    );
    let f = sym(&g, "f");
    let g_tok = sym(&g, "g");
    let w = sym(&g, "W");
    let first_w = ff.first(w).expect("W has FIRST set");
    assert!(first_w.contains(f.0 as usize));
    assert!(first_w.contains(g_tok.0 as usize));
}

#[test]
fn first_multi_8() {
    let (g, ff) = build_ff(
        "test8",
        &[("h", "[h]"), ("k", "[k]"), ("l", "[l]")],
        &[("X", vec!["h"]), ("X", vec!["k"]), ("X", vec!["l"])],
        "X",
    );
    let h = sym(&g, "h");
    let k = sym(&g, "k");
    let l = sym(&g, "l");
    let x = sym(&g, "X");
    let first_x = ff.first(x).expect("X has FIRST set");
    assert!(first_x.contains(h.0 as usize));
    assert!(first_x.contains(k.0 as usize));
    assert!(first_x.contains(l.0 as usize));
}

// ============================================================================
// Category 4: follow_basic_* — basic FOLLOW set computation
// ============================================================================

#[test]
fn follow_basic_1() {
    let (g, ff) = build_ff(
        "test1",
        &[("a", "[a]"), ("b", "[b]")],
        &[("S", vec!["a", "b"])],
        "S",
    );
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    let follow_a = ff.follow(a).expect("a has FOLLOW set");
    assert!(follow_a.contains(b.0 as usize), "a FOLLOW contains b");
}

#[test]
fn follow_basic_2() {
    let (g, ff) = build_ff(
        "test2",
        &[("x", "[x]"), ("y", "[y]")],
        &[("E", vec!["x", "y"])],
        "E",
    );
    let x = sym(&g, "x");
    let y = sym(&g, "y");
    let follow_x = ff.follow(x).expect("x has FOLLOW set");
    assert!(follow_x.contains(y.0 as usize));
}

#[test]
fn follow_basic_3() {
    let (g, ff) = build_ff(
        "test3",
        &[("p", "[p]"), ("q", "[q]")],
        &[("A", vec!["p", "q"])],
        "A",
    );
    let p = sym(&g, "p");
    let q = sym(&g, "q");
    let follow_p = ff.follow(p).expect("p has FOLLOW set");
    assert!(follow_p.contains(q.0 as usize));
}

#[test]
fn follow_basic_4() {
    let (g, ff) = build_ff(
        "test4",
        &[("m", "[m]"), ("n", "[n]")],
        &[("T", vec!["m", "n"])],
        "T",
    );
    let m = sym(&g, "m");
    let n = sym(&g, "n");
    let follow_m = ff.follow(m).expect("m has FOLLOW set");
    assert!(follow_m.contains(n.0 as usize));
}

#[test]
fn follow_basic_5() {
    let (g, ff) = build_ff(
        "test5",
        &[("i", "[i]"), ("j", "[j]")],
        &[("U", vec!["i", "j"])],
        "U",
    );
    let i = sym(&g, "i");
    let j = sym(&g, "j");
    let follow_i = ff.follow(i).expect("i has FOLLOW set");
    assert!(follow_i.contains(j.0 as usize));
}

#[test]
fn follow_basic_6() {
    let (g, ff) = build_ff(
        "test6",
        &[("c", "[c]"), ("d", "[d]")],
        &[("V", vec!["c", "d"])],
        "V",
    );
    let c = sym(&g, "c");
    let d = sym(&g, "d");
    let follow_c = ff.follow(c).expect("c has FOLLOW set");
    assert!(follow_c.contains(d.0 as usize));
}

#[test]
fn follow_basic_7() {
    let (g, ff) = build_ff(
        "test7",
        &[("f", "[f]"), ("g", "[g]")],
        &[("W", vec!["f", "g"])],
        "W",
    );
    let f = sym(&g, "f");
    let g_tok = sym(&g, "g");
    let follow_f = ff.follow(f).expect("f has FOLLOW set");
    assert!(follow_f.contains(g_tok.0 as usize));
}

#[test]
fn follow_basic_8() {
    let (g, ff) = build_ff(
        "test8",
        &[("h", "[h]"), ("k", "[k]")],
        &[("X", vec!["h", "k"])],
        "X",
    );
    let h = sym(&g, "h");
    let k = sym(&g, "k");
    let follow_h = ff.follow(h).expect("h has FOLLOW set");
    assert!(follow_h.contains(k.0 as usize));
}

// ============================================================================
// Category 5: follow_eof_* — EOF in FOLLOW sets
// ============================================================================

#[test]
fn follow_eof_1() {
    let (g, ff) = build_ff(
        "test1",
        &[("a", "[a]")],
        &[("S", vec!["a"])],
        "S",
    );
    let s = sym(&g, "S");
    let follow_s = ff.follow(s).expect("S has FOLLOW set");
    assert!(follow_s.contains(EOF.0 as usize), "S FOLLOW contains EOF");
}

#[test]
fn follow_eof_2() {
    let (g, ff) = build_ff(
        "test2",
        &[("x", "[x]")],
        &[("E", vec!["x"])],
        "E",
    );
    let e = sym(&g, "E");
    let follow_e = ff.follow(e).expect("E has FOLLOW set");
    assert!(follow_e.contains(EOF.0 as usize));
}

#[test]
fn follow_eof_3() {
    let (g, ff) = build_ff(
        "test3",
        &[("p", "[p]")],
        &[("A", vec!["p"])],
        "A",
    );
    let a = sym(&g, "A");
    let follow_a = ff.follow(a).expect("A has FOLLOW set");
    assert!(follow_a.contains(EOF.0 as usize));
}

#[test]
fn follow_eof_4() {
    let (g, ff) = build_ff(
        "test4",
        &[("m", "[m]")],
        &[("T", vec!["m"])],
        "T",
    );
    let t = sym(&g, "T");
    let follow_t = ff.follow(t).expect("T has FOLLOW set");
    assert!(follow_t.contains(EOF.0 as usize));
}

#[test]
fn follow_eof_5() {
    let (g, ff) = build_ff(
        "test5",
        &[("i", "[i]")],
        &[("U", vec!["i"])],
        "U",
    );
    let u = sym(&g, "U");
    let follow_u = ff.follow(u).expect("U has FOLLOW set");
    assert!(follow_u.contains(EOF.0 as usize));
}

#[test]
fn follow_eof_6() {
    let (g, ff) = build_ff(
        "test6",
        &[("c", "[c]")],
        &[("V", vec!["c"])],
        "V",
    );
    let v = sym(&g, "V");
    let follow_v = ff.follow(v).expect("V has FOLLOW set");
    assert!(follow_v.contains(EOF.0 as usize));
}

#[test]
fn follow_eof_7() {
    let (g, ff) = build_ff(
        "test7",
        &[("f", "[f]")],
        &[("W", vec!["f"])],
        "W",
    );
    let w = sym(&g, "W");
    let follow_w = ff.follow(w).expect("W has FOLLOW set");
    assert!(follow_w.contains(EOF.0 as usize));
}

#[test]
fn follow_eof_8() {
    let (g, ff) = build_ff(
        "test8",
        &[("h", "[h]")],
        &[("X", vec!["h"])],
        "X",
    );
    let x = sym(&g, "X");
    let follow_x = ff.follow(x).expect("X has FOLLOW set");
    assert!(follow_x.contains(EOF.0 as usize));
}

// ============================================================================
// Category 6: follow_chain_* — FOLLOW propagation
// ============================================================================

#[test]
fn follow_chain_1() {
    let (g, ff) = build_ff(
        "test1",
        &[("a", "[a]"), ("b", "[b]"), ("c", "[c]")],
        &[("S", vec!["A", "b"]), ("A", vec!["a", "B"]), ("B", vec!["c"])],
        "S",
    );
    let b_nt = sym(&g, "B");
    let c = sym(&g, "c");
    let follow_b = ff.follow(b_nt).expect("B has FOLLOW set");
    assert!(follow_b.contains(c.0 as usize), "B FOLLOW includes c from second alternative");
}

#[test]
fn follow_chain_2() {
    let (g, ff) = build_ff(
        "test2",
        &[("x", "[x]"), ("y", "[y]"), ("z", "[z]")],
        &[("E", vec!["A", "y"]), ("A", vec!["x", "B"]), ("B", vec!["z"])],
        "E",
    );
    let b_nt = sym(&g, "B");
    let y = sym(&g, "y");
    let follow_b = ff.follow(b_nt).expect("B has FOLLOW set");
    assert!(follow_b.contains(y.0 as usize));
}

#[test]
fn follow_chain_3() {
    let (g, ff) = build_ff(
        "test3",
        &[("p", "[p]"), ("q", "[q]")],
        &[("S", vec!["A"]), ("A", vec!["p", "B"]), ("B", vec!["q"])],
        "S",
    );
    let b_nt = sym(&g, "B");
    let follow_b = ff.follow(b_nt).expect("B has FOLLOW set");
    assert!(follow_b.contains(EOF.0 as usize), "B FOLLOW contains EOF from start symbol");
}

#[test]
fn follow_chain_4() {
    let (g, ff) = build_ff(
        "test4",
        &[("m", "[m]"), ("n", "[n]")],
        &[("S", vec!["T", "n"]), ("T", vec!["m"])],
        "S",
    );
    let t = sym(&g, "T");
    let n = sym(&g, "n");
    let follow_t = ff.follow(t).expect("T has FOLLOW set");
    assert!(follow_t.contains(n.0 as usize));
}

#[test]
fn follow_chain_5() {
    let (g, ff) = build_ff(
        "test5",
        &[("i", "[i]"), ("j", "[j]")],
        &[("S", vec!["U", "j"]), ("U", vec!["i"])],
        "S",
    );
    let u = sym(&g, "U");
    let j = sym(&g, "j");
    let follow_u = ff.follow(u).expect("U has FOLLOW set");
    assert!(follow_u.contains(j.0 as usize));
}

#[test]
fn follow_chain_6() {
    let (g, ff) = build_ff(
        "test6",
        &[("c", "[c]"), ("d", "[d]")],
        &[("S", vec!["V", "d"]), ("V", vec!["c"])],
        "S",
    );
    let v = sym(&g, "V");
    let d = sym(&g, "d");
    let follow_v = ff.follow(v).expect("V has FOLLOW set");
    assert!(follow_v.contains(d.0 as usize));
}

#[test]
fn follow_chain_7() {
    let (g, ff) = build_ff(
        "test7",
        &[("f", "[f]"), ("g", "[g]")],
        &[("S", vec!["W", "g"]), ("W", vec!["f"])],
        "S",
    );
    let w = sym(&g, "W");
    let g_tok = sym(&g, "g");
    let follow_w = ff.follow(w).expect("W has FOLLOW set");
    assert!(follow_w.contains(g_tok.0 as usize));
}

#[test]
fn follow_chain_8() {
    let (g, ff) = build_ff(
        "test8",
        &[("h", "[h]"), ("k", "[k]")],
        &[("S", vec!["X", "k"]), ("X", vec!["h"])],
        "S",
    );
    let x = sym(&g, "X");
    let k = sym(&g, "k");
    let follow_x = ff.follow(x).expect("X has FOLLOW set");
    assert!(follow_x.contains(k.0 as usize));
}

// ============================================================================
// Category 7: nullable_* — nullability detection
// ============================================================================

#[test]
fn nullable_1() {
    let (g, ff) = build_ff(
        "test1",
        &[("a", "[a]")],
        &[("N", vec![])],
        "N",
    );
    let n = sym(&g, "N");
    assert!(ff.is_nullable(n), "empty production makes N nullable");
}

#[test]
fn nullable_2() {
    let (g, ff) = build_ff(
        "test2",
        &[("x", "[x]")],
        &[("N", vec![]), ("E", vec!["x"])],
        "E",
    );
    let n = sym(&g, "N");
    assert!(ff.is_nullable(n));
}

#[test]
fn nullable_3() {
    let (g, ff) = build_ff(
        "test3",
        &[("p", "[p]")],
        &[("A", vec![]), ("B", vec!["A"])],
        "B",
    );
    let a = sym(&g, "A");
    let b = sym(&g, "B");
    assert!(ff.is_nullable(a));
    assert!(ff.is_nullable(b), "B is nullable because A is nullable");
}

#[test]
fn nullable_4() {
    let (g, ff) = build_ff(
        "test4",
        &[("m", "[m]")],
        &[("T", vec![]), ("U", vec!["T"])],
        "U",
    );
    let t = sym(&g, "T");
    assert!(ff.is_nullable(t));
}

#[test]
fn nullable_5() {
    let (g, ff) = build_ff(
        "test5",
        &[("i", "[i]")],
        &[("N1", vec![]), ("N2", vec!["N1"]), ("N3", vec!["N2"])],
        "N3",
    );
    let n1 = sym(&g, "N1");
    let n2 = sym(&g, "N2");
    let n3 = sym(&g, "N3");
    assert!(ff.is_nullable(n1));
    assert!(ff.is_nullable(n2));
    assert!(ff.is_nullable(n3));
}

#[test]
fn nullable_6() {
    let (g, ff) = build_ff(
        "test6",
        &[("c", "[c]")],
        &[("N", vec![])],
        "N",
    );
    let n = sym(&g, "N");
    assert!(ff.is_nullable(n));
}

#[test]
fn nullable_7() {
    let (g, ff) = build_ff(
        "test7",
        &[("f", "[f]")],
        &[("A", vec![]), ("B", vec!["A"])],
        "B",
    );
    let a = sym(&g, "A");
    assert!(ff.is_nullable(a));
}

#[test]
fn nullable_8() {
    let (g, ff) = build_ff(
        "test8",
        &[("h", "[h]")],
        &[("N", vec![]), ("M", vec!["N"])],
        "M",
    );
    let n = sym(&g, "N");
    let m = sym(&g, "M");
    assert!(ff.is_nullable(n));
    assert!(ff.is_nullable(m));
}

// ============================================================================
// Category 8: combined_* — combined FIRST/FOLLOW properties
// ============================================================================

#[test]
fn combined_1() {
    let (g, ff) = build_ff(
        "test1",
        &[("a", "[a]"), ("b", "[b]")],
        &[("S", vec!["A", "b"]), ("A", vec!["a"])],
        "S",
    );
    let a = sym(&g, "A");
    let a_tok = sym(&g, "a");
    let first_a = ff.first(a).expect("A has FIRST");
    assert!(first_a.contains(a_tok.0 as usize), "FIRST(A) contains a");
}

#[test]
fn combined_2() {
    let (g, ff) = build_ff(
        "test2",
        &[("x", "[x]"), ("y", "[y]")],
        &[("E", vec!["A", "y"]), ("A", vec!["x"])],
        "E",
    );
    let a = sym(&g, "A");
    let y = sym(&g, "y");
    let follow_a = ff.follow(a).expect("A has FOLLOW");
    assert!(follow_a.contains(y.0 as usize), "FOLLOW(A) contains y");
}

#[test]
fn combined_3() {
    let (g, ff) = build_ff(
        "test3",
        &[("p", "[p]"), ("q", "[q]")],
        &[("S", vec!["A"]), ("A", vec![]), ("B", vec!["p"])],
        "S",
    );
    let a = sym(&g, "A");
    let b = sym(&g, "B");
    assert!(ff.is_nullable(a), "A is nullable");
    let b_follow = ff.follow(b);
    assert!(b_follow.is_some(), "B has FOLLOW set");
}

#[test]
fn combined_4() {
    let (g, ff) = build_ff(
        "test4",
        &[("m", "[m]"), ("n", "[n]")],
        &[("S", vec!["T"]), ("T", vec!["m"])],
        "S",
    );
    let t = sym(&g, "T");
    let m = sym(&g, "m");
    let first_t = ff.first(t).expect("T has FIRST");
    assert!(first_t.contains(m.0 as usize));
    let follow_t = ff.follow(t).expect("T has FOLLOW");
    assert!(follow_t.contains(EOF.0 as usize));
}

#[test]
fn combined_5() {
    let (g, ff) = build_ff(
        "test5",
        &[("i", "[i]"), ("j", "[j]")],
        &[("S", vec!["U"]), ("U", vec!["i", "V"]), ("V", vec!["j"])],
        "S",
    );
    let u = sym(&g, "U");
    let v = sym(&g, "V");
    let i = sym(&g, "i");
    let first_u = ff.first(u).expect("U has FIRST");
    assert!(first_u.contains(i.0 as usize));
    let follow_v = ff.follow(v).expect("V has FOLLOW");
    assert!(follow_v.contains(EOF.0 as usize));
}

#[test]
fn combined_6() {
    let (g, ff) = build_ff(
        "test6",
        &[("c", "[c]"), ("d", "[d]")],
        &[("S", vec!["V", "d"]), ("V", vec!["c"])],
        "S",
    );
    let v = sym(&g, "V");
    let c = sym(&g, "c");
    let d = sym(&g, "d");
    let first_v = ff.first(v).expect("V has FIRST");
    assert!(first_v.contains(c.0 as usize));
    let follow_v = ff.follow(v).expect("V has FOLLOW");
    assert!(follow_v.contains(d.0 as usize));
}

#[test]
fn combined_7() {
    let (g, ff) = build_ff(
        "test7",
        &[("f", "[f]"), ("g", "[g]")],
        &[("S", vec!["W"]), ("W", vec![]), ("X", vec!["f"])],
        "S",
    );
    let w = sym(&g, "W");
    assert!(ff.is_nullable(w), "W is nullable");
    let follow_w = ff.follow(w).expect("W has FOLLOW");
    assert!(follow_w.contains(EOF.0 as usize));
}

#[test]
fn combined_8() {
    let (g, ff) = build_ff(
        "test8",
        &[("h", "[h]"), ("k", "[k]"), ("l", "[l]")],
        &[
            ("S", vec!["X", "l"]),
            ("X", vec!["h"]),
            ("X", vec!["k"]),
        ],
        "S",
    );
    let x = sym(&g, "X");
    let h = sym(&g, "h");
    let k = sym(&g, "k");
    let l = sym(&g, "l");
    let first_x = ff.first(x).expect("X has FIRST");
    assert!(first_x.contains(h.0 as usize));
    assert!(first_x.contains(k.0 as usize));
    let follow_x = ff.follow(x).expect("X has FOLLOW");
    assert!(follow_x.contains(l.0 as usize));
}
