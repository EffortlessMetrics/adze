//! Tree-sitter baseline comparison tools
//!
//! Compares rust-sitter performance against Tree-sitter C implementation.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use xshell::Shell;

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkResult {
    language: String,
    size: String,
    parse_time_ms: Option<f64>,
    memory_mb: Option<f64>,
    note: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BaselineData {
    comment: String,
    date: String,
    benchmarks: Vec<BenchmarkResult>,
    methodology: String,
}

/// Output format for comparison report
pub enum OutputFormat {
    Table,
    Json,
    Markdown,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Self {
        match s {
            "json" => Self::Json,
            "markdown" | "md" => Self::Markdown,
            _ => Self::Table,
        }
    }
}

/// Run Tree-sitter comparison and generate report
pub fn compare_baseline(
    _sh: &Shell,
    format: OutputFormat,
    output_file: Option<&str>,
) -> Result<()> {
    eprintln!("🌲 Tree-sitter Baseline Comparison");
    eprintln!();

    // Check if Tree-sitter CLI is available
    let has_tree_sitter = which::which("tree-sitter").is_ok();

    if !has_tree_sitter {
        eprintln!("⚠️  tree-sitter CLI not found");
        eprintln!("Install with: npm install -g tree-sitter-cli");
        eprintln!();
        eprintln!("Generating placeholder comparison data...");
    }

    // Create output directory
    let output_dir = "target/benchmarks";
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create directory: {}", output_dir))?;

    // Generate baseline data (placeholder for now)
    let baseline = generate_baseline_data(has_tree_sitter)?;

    // Save JSON baseline
    let baseline_json_path = format!("{}/tree_sitter_baseline.json", output_dir);
    fs::write(
        &baseline_json_path,
        serde_json::to_string_pretty(&baseline)?
    )?;

    eprintln!("✅ Baseline data saved: {}", baseline_json_path);

    // Generate comparison report
    let report = generate_report(&baseline, &format)?;

    // Output report
    if let Some(output_file) = output_file {
        fs::write(output_file, &report)
            .with_context(|| format!("Failed to write report: {}", output_file))?;
        eprintln!("✅ Report saved: {}", output_file);
    } else {
        println!();
        println!("{}", "=".repeat(80));
        println!("{}", report);
        println!("{}", "=".repeat(80));
    }

    Ok(())
}

fn generate_baseline_data(has_tree_sitter: bool) -> Result<BaselineData> {
    let status = if has_tree_sitter {
        "Ready for measurement"
    } else {
        "Pending tree-sitter CLI installation"
    };

    let benchmarks = vec![
        BenchmarkResult {
            language: "python".to_string(),
            size: "small".to_string(),
            parse_time_ms: None,
            memory_mb: None,
            note: format!("Status: {}", status),
        },
        BenchmarkResult {
            language: "python".to_string(),
            size: "medium".to_string(),
            parse_time_ms: None,
            memory_mb: None,
            note: format!("Status: {} - pending medium fixture creation", status),
        },
        BenchmarkResult {
            language: "python".to_string(),
            size: "large".to_string(),
            parse_time_ms: None,
            memory_mb: None,
            note: format!("Status: {} - pending large fixture creation", status),
        },
        BenchmarkResult {
            language: "javascript".to_string(),
            size: "small".to_string(),
            parse_time_ms: None,
            memory_mb: None,
            note: format!("Status: {}", status),
        },
        BenchmarkResult {
            language: "rust".to_string(),
            size: "small".to_string(),
            parse_time_ms: None,
            memory_mb: None,
            note: format!("Status: {}", status),
        },
    ];

    Ok(BaselineData {
        comment: "Tree-sitter C baseline measurements".to_string(),
        date: chrono::Utc::now().to_rfc3339(),
        benchmarks,
        methodology: "hyperfine --warmup 10 --runs 100 'tree-sitter parse <fixture>'".to_string(),
    })
}

fn generate_report(baseline: &BaselineData, format: &OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Json => Ok(serde_json::to_string_pretty(baseline)?),
        OutputFormat::Markdown => generate_markdown_report(baseline),
        OutputFormat::Table => generate_table_report(baseline),
    }
}

fn generate_table_report(baseline: &BaselineData) -> Result<String> {
    let mut report = String::new();

    report.push_str("Tree-sitter C vs. rust-sitter Performance Comparison\n");
    report.push_str(&format!("Generated: {}\n\n", baseline.date));

    report.push_str("Performance Goal:\n");
    report.push_str("  rust-sitter parsing time ≤ 2x Tree-sitter C (all benchmarks)\n\n");

    report.push_str(&format!("{:-<80}\n", ""));
    report.push_str(&format!("{:<12} {:<8} {:<16} {:<16} {:<8} {:<12}\n",
        "Language", "Size", "Tree-sitter (ms)", "rust-sitter (ms)", "Ratio", "Goal Met?"));
    report.push_str(&format!("{:-<80}\n", ""));

    for bench in &baseline.benchmarks {
        let ts_time = bench.parse_time_ms.map(|t| format!("{:.2}", t))
            .unwrap_or_else(|| "TBD".to_string());
        let rs_time = "TBD";
        let ratio = "TBD";
        let goal_met = "⏳";

        report.push_str(&format!("{:<12} {:<8} {:<16} {:<16} {:<8} {:<12}\n",
            bench.language, bench.size, ts_time, rs_time, ratio, goal_met));
    }

    report.push_str(&format!("{:-<80}\n", ""));
    report.push_str("\nStatus: Benchmarking infrastructure ready, measurements pending Week 3 Day 2.\n");
    report.push_str("\nNext Steps:\n");
    report.push_str("  1. Install tree-sitter CLI: npm install -g tree-sitter-cli\n");
    report.push_str("  2. Create medium/large fixtures (Week 3 Day 2)\n");
    report.push_str("  3. Integrate GLR parsing into benchmarks\n");
    report.push_str("  4. Run actual measurements: cargo xtask compare-baseline\n");

    Ok(report)
}

fn generate_markdown_report(baseline: &BaselineData) -> Result<String> {
    let mut report = String::new();

    report.push_str("# Tree-sitter C vs. rust-sitter Performance Comparison\n\n");
    report.push_str(&format!("**Generated**: {}\n\n", baseline.date));

    report.push_str("## Performance Goal\n\n");
    report.push_str("- rust-sitter parsing time ≤ 2x Tree-sitter C (all benchmarks)\n\n");

    report.push_str("## Results\n\n");
    report.push_str("| Language   | Size   | Tree-sitter (ms) | rust-sitter (ms) | Ratio | Goal Met? |\n");
    report.push_str("|------------|--------|------------------|------------------|-------|-----------|\n");

    for bench in &baseline.benchmarks {
        let ts_time = bench.parse_time_ms.map(|t| format!("{:.2}", t))
            .unwrap_or_else(|| "TBD".to_string());
        let rs_time = "TBD";
        let ratio = "TBD";
        let goal_met = "⏳";

        report.push_str(&format!("| {:<10} | {:<6} | {:<16} | {:<16} | {:<5} | {:<9} |\n",
            bench.language, bench.size, ts_time, rs_time, ratio, goal_met));
    }

    report.push_str("\n## Status\n\n");
    report.push_str("Benchmarking infrastructure ready, measurements pending Week 3 Day 2.\n\n");

    report.push_str("## Next Steps\n\n");
    report.push_str("1. Install tree-sitter CLI: `npm install -g tree-sitter-cli`\n");
    report.push_str("2. Create medium/large fixtures (Week 3 Day 2)\n");
    report.push_str("3. Integrate GLR parsing into benchmarks\n");
    report.push_str("4. Run actual measurements: `cargo xtask compare-baseline`\n");

    Ok(report)
}
