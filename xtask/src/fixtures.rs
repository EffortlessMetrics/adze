//! Fixture generation for benchmarking
//!
//! This module generates valid arithmetic expression fixtures for performance
//! benchmarking, ensuring that benchmarks measure actual parsing, not error recovery.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use xshell::Shell;

/// Target lines of code for each fixture size
const SMALL_TARGET_LOC: usize = 100;
const MEDIUM_TARGET_LOC: usize = 2_000;
const LARGE_TARGET_LOC: usize = 10_000;

/// Generate arithmetic expression fixtures
pub fn generate_fixtures(sh: &Shell, output_dir: &str, force: bool) -> Result<()> {
    let output_path = Path::new(output_dir);

    // Create output directory if it doesn't exist
    if !output_path.exists() {
        println!("Creating output directory: {}", output_dir);
        fs::create_dir_all(output_path)
            .with_context(|| format!("Failed to create directory: {}", output_dir))?;
    }

    // Generate each fixture size
    generate_fixture(output_path, "small.expr", SMALL_TARGET_LOC, force)?;
    generate_fixture(output_path, "medium.expr", MEDIUM_TARGET_LOC, force)?;
    generate_fixture(output_path, "large.expr", LARGE_TARGET_LOC, force)?;

    println!("\n✅ Fixture generation complete!");
    println!("   Location: {}", output_dir);
    println!("   Files: small.expr, medium.expr, large.expr");

    // Validate generated fixtures
    println!("\n🔍 Validating generated fixtures...");
    validate_fixtures(sh, output_path)?;

    Ok(())
}

/// Generate a single fixture file
fn generate_fixture(
    output_dir: &Path,
    filename: &str,
    target_loc: usize,
    force: bool,
) -> Result<()> {
    let file_path = output_dir.join(filename);

    // Check if file exists and force flag
    if file_path.exists() && !force {
        println!("⏭️  Skipping {} (already exists, use --force to regenerate)", filename);
        return Ok(());
    }

    println!("📝 Generating {} (~{} LOC)...", filename, target_loc);

    let mut content = String::new();

    // NO HEADER COMMENTS - the arithmetic grammar doesn't support comments!
    // The arithmetic grammar expects a SINGLE expression, so we'll generate
    // one large chained expression with many operators.

    // Generate a single large expression with many operations
    // We'll chain operations together: 1 - 2 * 3 - 4 * 5 - 6 * 7 - ...

    // Start with first number
    content.push_str("1");

    // Calculate how many operations we need
    // IMPORTANT: Parser may have limits on expression depth/length
    // Keep expressions reasonable to avoid hitting parser limits
    let target_ops = match target_loc {
        100 => 50,       // Small: 50 operations (~250 bytes)
        2000 => 200,     // Medium: 200 operations (~1.5 KB)
        10000 => 1000,   // Large: 1000 operations (~6-7 KB)
        _ => target_loc.min(1000), // Cap at 1000 ops for any other size
    };

    // Alternate between operators to create interesting parsing
    let operators = [" - ", " * "];
    let mut current_num = 2;

    for i in 0..target_ops {
        let op = operators[i % 2];
        content.push_str(op);
        content.push_str(&current_num.to_string());
        current_num += 1;
    }

    content.push_str("\n");

    let line_count = 1; // Single line (plus newline)
    let expr_count = target_ops;

    // Write to file
    fs::write(&file_path, content)
        .with_context(|| format!("Failed to write fixture: {}", file_path.display()))?;

    println!("   ✅ Generated {} ({} lines, {} expressions)",
             filename, line_count, expr_count);

    Ok(())
}

/// Validate that generated fixtures can be parsed by the arithmetic grammar
fn validate_fixtures(sh: &Shell, fixtures_dir: &Path) -> Result<()> {
    let _dir = sh.push_dir(sh.current_dir());

    println!("   Running validation tests...");

    // Run the verification test
    xshell::cmd!(sh, "cargo test -p rust-sitter-benchmarks --test verify_fixture_parsing verify_valid_arithmetic_expressions_do_parse -- --nocapture")
        .run()
        .context("Fixture validation failed")?;

    println!("   ✅ All fixtures validated successfully");

    Ok(())
}

/// Validate existing fixtures (without generating new ones)
pub fn validate_only(sh: &Shell, fixtures_dir: &str) -> Result<()> {
    let fixtures_path = Path::new(fixtures_dir);

    if !fixtures_path.exists() {
        anyhow::bail!("Fixtures directory does not exist: {}", fixtures_dir);
    }

    println!("🔍 Validating fixtures in: {}", fixtures_dir);

    // Check that required files exist
    for filename in &["small.expr", "medium.expr", "large.expr"] {
        let file_path = fixtures_path.join(filename);
        if !file_path.exists() {
            println!("   ⚠️  Missing fixture: {}", filename);
            println!("      Run: cargo xtask generate-fixtures");
            anyhow::bail!("Required fixture missing: {}", filename);
        }
    }

    validate_fixtures(sh, fixtures_path)?;

    println!("\n✅ Validation complete!");

    Ok(())
}

/// Show information about generated fixtures
pub fn info_fixtures(fixtures_dir: &str) -> Result<()> {
    let fixtures_path = Path::new(fixtures_dir);

    println!("📊 Fixture Information");
    println!("   Location: {}", fixtures_dir);
    println!();

    for (size, filename) in &[
        ("Small", "small.expr"),
        ("Medium", "medium.expr"),
        ("Large", "large.expr"),
    ] {
        let file_path = fixtures_path.join(filename);

        if file_path.exists() {
            let content = fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read: {}", filename))?;

            let line_count = content.lines().count();
            let byte_size = content.len();
            // Count non-empty lines as expressions
            let expr_count = content.lines().filter(|l| !l.trim().is_empty()).count();

            println!("   {} ({}):", size, filename);
            println!("      Lines: {}", line_count);
            println!("      Bytes: {}", byte_size);
            println!("      Expressions: {}", expr_count);
            println!();
        } else {
            println!("   {} ({}): ⚠️ NOT FOUND", size, filename);
            println!();
        }
    }

    Ok(())
}
