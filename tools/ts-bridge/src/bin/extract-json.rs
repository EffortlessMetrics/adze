use ts_bridge::{extract, ffi::TSLanguage};

type LangFn = unsafe extern "C" fn() -> *const TSLanguage;

fn tree_sitter_json_fn() -> LangFn {
    unsafe { std::mem::transmute(tree_sitter_json::LANGUAGE.into_raw()) }
}

fn main() {
    let lang_fn = tree_sitter_json_fn();
    let data = extract(lang_fn).expect("Failed to extract tables");

    let json = serde_json::to_string_pretty(&data).expect("Failed to serialize");
    println!("{}", json);
}
