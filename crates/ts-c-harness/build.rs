//! Build script for `ts-c-harness`.
//! Compiles small C helpers when the `ts-ffi-raw` feature is enabled.
#![allow(missing_docs)]

fn main() {
    #[cfg(feature = "ts-ffi-raw")]
    {
        cc::Build::new()
            .file("tests/ts_c_shim.c")
            .compile("ts_shim");
        println!("cargo:rerun-if-changed=tests/ts_c_shim.c");
    }
}
