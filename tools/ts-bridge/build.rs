fn main() {
    println!("cargo:rerun-if-changed=ffi/shim.c");
    println!("cargo:rerun-if-changed=ffi/shim.h");

    let mut b = cc::Build::new();
    b.file("ffi/shim.c").include("ffi");

    if cfg!(feature = "stub-ts") {
        println!("cargo:rerun-if-changed=ffi/ts_stub.c");
        println!("cargo:rustc-cfg=tsb_stub");
        b.define("tsb_stub", None);  // Define for C preprocessor
        b.file("ffi/ts_stub.c").include("ffi");
    } else {
        println!("cargo:rerun-if-changed=ci/vendor/tree_sitter/api.h");
        println!("cargo:rerun-if-changed=ci/vendor/tree_sitter/parser.h");
        b.include("ci/vendor");                 // has tree_sitter/
        b.include("ci/vendor/tree_sitter");     // so #include "tree_sitter/api.h" resolves
    }

    b.warnings(false).compile("tsb_shim");
}