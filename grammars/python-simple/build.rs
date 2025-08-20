use std::path::Path;

fn main() {
    // Tell rustc this cfg is intentional so it doesn't warn
    println!("cargo::rustc-check-cfg=cfg(rust_sitter_unsafe_attrs)");

    // Only build if not in test mode, since we're manually including the generated parser
    if std::env::var("CARGO_CFG_TEST").is_err() {
        // SAFETY: This is safe in a build script as it runs in a single-threaded context
        unsafe {
            std::env::set_var("RUST_SITTER_USE_PURE_RUST", "1");
        }
        rust_sitter_tool::build_parsers(Path::new("src/lib.rs"));
    }
}
