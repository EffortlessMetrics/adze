use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=src");

    // Use pure-Rust parser generation if the feature is enabled
    // Note: build scripts can't directly check features, so we use an env var set by Cargo
    if env::var("CARGO_FEATURE_PURE_RUST").is_ok() {
        // SAFETY: This is safe in a build script as it runs in a single-threaded context
        unsafe {
            env::set_var("ADZE_USE_PURE_RUST", "1");
        }
    }

    // Edition-aware attribute toggle for generated code
    // Use proper TOML parsing for robust edition detection
    let manifest_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("Cargo.toml");
    let manifest_str = fs::read_to_string(&manifest_path).unwrap();
    let manifest: toml::Value = toml::from_str(&manifest_str).expect("Failed to parse Cargo.toml");

    let edition = manifest
        .get("package")
        .and_then(|p| p.get("edition"))
        .and_then(|e| e.as_str());

    if edition == Some("2024") {
        println!("cargo:rustc-cfg=adze_unsafe_attrs");
    }

    // Always tell rustc this cfg is intentional
    println!("cargo:rustc-check-cfg=cfg(adze_unsafe_attrs)");

    // Use lib.rs as the root since that's where the grammar modules are defined
    eprintln!("DEBUG: Building parsers...");
    adze_tool::build_parsers(&PathBuf::from("src/lib.rs"));
}
