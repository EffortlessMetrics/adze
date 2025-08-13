fn main() {
    // Build the tiny C shim against tree-sitter headers
    println!("cargo:rerun-if-changed=ffi/shim.c");
    println!("cargo:rerun-if-changed=ffi/shim.h");
    println!("cargo:rerun-if-changed=ffi/ts_stub.c");
    
    cc::Build::new()
        .file("ffi/shim.c")
        .file("ffi/ts_stub.c")  // Include stub implementations for now
        .include("ffi") // shim.h
        // The grammar crate (e.g., tree-sitter-json) must make TS headers visible in CI.
        // In local dev, rely on system include path or vendored headers.
        .warnings(false)
        .compile("tsb_shim");
}