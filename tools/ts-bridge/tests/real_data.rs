#![cfg(feature = "with-grammars")]

use std::mem::transmute;
use ts_bridge::{
    extract,
    ffi::{SafeLang, TSLanguage},
};

fn get_json_language() -> *const TSLanguage {
    unsafe { tree_sitter_json::LANGUAGE.into_raw()() as *const TSLanguage }
}

fn tree_sitter_json_fn() -> unsafe extern "C" fn() -> *const TSLanguage {
    unsafe { transmute(tree_sitter_json::LANGUAGE.into_raw()) }
}

#[test]
fn counts_and_names_match_runtime() {
    let lang_fn = tree_sitter_json_fn();
    let data = extract(lang_fn).expect("extract failed");
    let lang = SafeLang(get_json_language());

    let (symc, stc, tokc, extc) = lang.counts();
    assert_eq!(data.symbol_count as u32, symc);
    assert_eq!(data.state_count as u32, stc);
    assert!(tokc > 0);
    assert!(extc >= 0);

    for (i, sym) in data.symbols.iter().enumerate().take(10) {
        let name = lang.symbol_name(i as u32);
        assert_eq!(sym.name, name, "symbol name mismatch at {i}");
        let meta = lang.symbol_metadata(i as u32);
        assert_eq!(sym.visible, meta.visible, "visible mismatch at {i}");
        assert_eq!(sym.named, meta.named, "named mismatch at {i}");
    }
}
