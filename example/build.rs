use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src");
    
    // Use pure-Rust parser generation if the feature is enabled
    #[cfg(feature = "pure-rust")]
    unsafe {
        std::env::set_var("RUST_SITTER_USE_PURE_RUST", "1");
    }
    
    rust_sitter_tool::build_parsers(&PathBuf::from("src/main.rs"));
}
