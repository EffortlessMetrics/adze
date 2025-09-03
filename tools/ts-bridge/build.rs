fn main() {
    println!("cargo:rerun-if-changed=ffi/shim.c");
    println!("cargo:rerun-if-changed=ffi/shim.h");

    // Feature flags (exposed to build.rs via env)
    let vendored_rt = std::env::var_os("CARGO_FEATURE_VENDORED_TS_RUNTIME").is_some();
    let link_system = std::env::var_os("CARGO_FEATURE_LINK_SYSTEM_TS").is_some();
    let ts_ffi_raw = std::env::var_os("CARGO_FEATURE_TS_FFI_RAW").is_some();

    let mut b = cc::Build::new();
    b.file("ffi/shim.c").include("ffi");

    if link_system {
        // Use system libtree-sitter when requested
        let lib = pkg_config::Config::new()
            .atleast_version("0.22")
            .probe("tree-sitter")
            .expect("pkg-config could not find system libtree-sitter; disable 'link-system-ts' or install the library");
        for p in lib.include_paths {
            b.include(p);
        }
        // Still allow building against vendored headers if desired:
        println!("cargo:rerun-if-changed=ci/vendor/tree_sitter/api.h");
        println!("cargo:rerun-if-changed=ci/vendor/tree_sitter/parser.h");
        b.include("ci/vendor");
    } else if vendored_rt || ts_ffi_raw {
        // Use upstream headers + sources
        println!("cargo:rerun-if-changed=ci/vendor/tree_sitter/lib/include/tree_sitter/api.h");
        println!("cargo:rerun-if-changed=ci/vendor/tree_sitter/lib/include/tree_sitter/parser.h");
        b.include("ci/vendor/tree_sitter/lib/include");
        b.include("ci/vendor/tree_sitter/lib/src"); // internal headers

        // Official runtime sources - need more complete set for linking
        println!("cargo:rerun-if-changed=ci/vendor/tree_sitter/lib/src/language.c");
        println!("cargo:rerun-if-changed=ci/vendor/tree_sitter/lib/src/alloc.c");
        println!("cargo:rerun-if-changed=ci/vendor/tree_sitter/lib/src/lookup.c");
        b.file("ci/vendor/tree_sitter/lib/src/language.c");
        b.file("ci/vendor/tree_sitter/lib/src/alloc.c");
        b.file("ci/vendor/tree_sitter/lib/src/lookup.c"); // This has ts_language_lookup
    } else {
        // Fallback: headers only (will fail to link). Nudge the user.
        println!("cargo:warning=No runtime selected. Enable 'vendored-ts-runtime' (default) or 'link-system-ts'.");
        println!("cargo:rerun-if-changed=ci/vendor/tree_sitter/api.h");
        println!("cargo:rerun-if-changed=ci/vendor/tree_sitter/parser.h");
        b.include("ci/vendor");
        b.include("ci/vendor/tree_sitter");
    }

    b.warnings(false).compile("tsb_shim");
}
