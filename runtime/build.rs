fn main() {
    // Tell rustc this cfg is allowed across the crate (tests included)
    println!("cargo::rustc-check-cfg=cfg(skip_integration_tests)");
}
