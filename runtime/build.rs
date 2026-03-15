fn main() {
    // Tell rustc these cfgs are allowed across the crate (tests included)
    println!("cargo::rustc-check-cfg=cfg(skip_integration_tests)");
    println!("cargo::rustc-check-cfg=cfg(skip_outdated_tests)");
}
