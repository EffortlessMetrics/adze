//! Fixture generator binary
//!
//! Generates synthetic Python and JavaScript code for benchmarking.
//!
//! Usage:
//!   cargo run --bin generate-fixtures
//!
//! Fixtures are written to benchmarks/fixtures/ and validated with
//! reference parsers (python3, node).

use rust_sitter_benchmarks::fixtures::FixtureGenerator;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("rust-sitter Fixture Generator");
    println!("==============================\n");

    // Determine output directory (benchmarks/fixtures/)
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output_dir = PathBuf::from(manifest_dir).join("fixtures");

    println!("Output directory: {}\n", output_dir.display());

    // Generate all fixtures
    let generator = FixtureGenerator::new(&output_dir);
    generator.generate_all()?;

    println!("\n✨ Fixture generation complete!");
    println!("Fixtures are ready for benchmarking at:");
    println!("  {}", output_dir.display());

    Ok(())
}
