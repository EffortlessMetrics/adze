use std::path::Path;

fn main() {
    // Tell rustc this cfg is intentional so it doesn't warn
    println!("cargo::rustc-check-cfg=cfg(adze_unsafe_attrs)");

    // Use pure Rust parser generation
    // SAFETY: This is safe in a build script as it runs in a single-threaded context
    unsafe {
        std::env::set_var("ADZE_USE_PURE_RUST", "1");
    }

    // Enable debug output
    // SAFETY: This is safe in a build script as it runs in a single-threaded context
    unsafe {
        std::env::set_var("ADZE_EMIT_ARTIFACTS", "true");
    }

    // Generate grammars first to see what's being generated
    let grammars =
        adze_tool::generate_grammars(Path::new("src/lib.rs")).expect("failed to generate grammars");

    // Print the generated grammar for debugging
    for grammar in &grammars {
        eprintln!("Generated adze grammar JSON:");
        eprintln!("{}", serde_json::to_string_pretty(&grammar).unwrap());

        // Also write to a file for easier debugging
        use std::io::Write;
        let mut file = std::fs::File::create("/tmp/test-vec-wrapper-grammar.json").unwrap();
        writeln!(file, "{}", serde_json::to_string_pretty(&grammar).unwrap()).unwrap();
    }

    // Now build the parsers
    adze_tool::build_parsers(Path::new("src/lib.rs"));
}
