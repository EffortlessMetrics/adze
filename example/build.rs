use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src");

    // Use pure-Rust parser generation if the feature is enabled
    // Note: build scripts can't directly check features, so we use an env var set by Cargo
    if std::env::var("CARGO_FEATURE_PURE_RUST").is_ok() {
        // SAFETY: This is safe in a build script as it runs in a single-threaded context
        unsafe {
            std::env::set_var("RUST_SITTER_USE_PURE_RUST", "1");
        }
    }

    // Use lib.rs as the root since that's where the grammar modules are defined
    rust_sitter_tool::build_parsers(&PathBuf::from("src/lib.rs"));
}
