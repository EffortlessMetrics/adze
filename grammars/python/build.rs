use std::path::PathBuf;

fn main() {
    // Tell rustc this cfg is intentional so it doesn't warn
    println!("cargo::rustc-check-cfg=cfg(rust_sitter_unsafe_attrs)");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/scanner.rs");

    // Build the parser
    // Enable pure-rust parser generation
    // SAFETY: This is safe in a build script as it runs in a single-threaded context
    unsafe {
        std::env::set_var("RUST_SITTER_USE_PURE_RUST", "1");
        std::env::set_var("RUST_SITTER_EMIT_ARTIFACTS", "true");
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    eprintln!("DEBUG: Starting parser generation for Python grammar");
    eprintln!("DEBUG: OUT_DIR = {:?}", std::env::var("OUT_DIR"));

    // Set up panic hook to get more information
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("PANIC occurred: {}", panic_info);
        if let Some(location) = panic_info.location() {
            eprintln!(
                "  at {}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            );
        }
    }));

    rust_sitter_tool::build_parsers(&PathBuf::from("src/lib.rs"));

    // Register the external scanner
    println!("cargo:rustc-env=RUST_SITTER_EXTERNAL_SCANNER=python");
}
