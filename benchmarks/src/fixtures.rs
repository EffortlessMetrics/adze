//! Fixture generator and validator for performance benchmarks
//!
//! This module provides:
//! - Synthetic code generation for Python and JavaScript
//! - Validation against reference parsers
//! - Metadata management

use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Fixture generator for benchmarking
pub struct FixtureGenerator {
    output_dir: PathBuf,
}

impl FixtureGenerator {
    pub fn new<P: AsRef<Path>>(output_dir: P) -> Self {
        FixtureGenerator {
            output_dir: output_dir.as_ref().to_path_buf(),
        }
    }

    /// Generate all fixtures
    pub fn generate_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Generating benchmark fixtures...");

        // Python fixtures
        self.generate_python_fixture("small.py", 100)?;
        self.generate_python_fixture("medium.py", 2000)?;
        self.generate_python_fixture("large.py", 10000)?;

        // JavaScript fixtures
        self.generate_javascript_fixture("small.js", 100)?;
        self.generate_javascript_fixture("medium.js", 1000)?;
        self.generate_javascript_fixture("large.js", 5000)?;

        println!("✅ All fixtures generated successfully!");
        Ok(())
    }

    /// Generate a Python fixture with approximately `target_loc` lines
    fn generate_python_fixture(&self, filename: &str, target_loc: usize) -> Result<(), Box<dyn std::error::Error>> {
        println!("Generating {filename} (target: {target_loc} LOC)...");

        let mut code = String::new();

        // Add header comment
        writeln!(
            &mut code,
            "# Generated Python fixture for benchmarking\n\
             # Target LOC: {target_loc}\n\
             # License: MIT (generated code)\n\
             # DO NOT EDIT MANUALLY - Regenerate with: cargo run -p benchmarks --bin generate-fixtures\n"
        )?;

        // Generate diverse Python constructs
        let mut current_loc = 5; // Header is ~5 lines

        // Module-level imports
        writeln!(&mut code, "import sys")?;
        writeln!(&mut code, "import os")?;
        writeln!(&mut code, "from typing import List, Dict, Optional, Any\n")?;
        current_loc += 4;

        // Generate functions, classes, and realistic Python patterns
        let mut func_counter = 0;
        let mut class_counter = 0;

        while current_loc < target_loc {
            let remaining = target_loc - current_loc;

            if remaining > 50 && current_loc % 100 < 60 {
                // Generate a class (takes ~30-50 lines)
                let class_lines = self.generate_python_class(&mut code, class_counter)?;
                current_loc += class_lines;
                class_counter += 1;
            } else if remaining > 15 {
                // Generate a function (takes ~10-20 lines)
                let func_lines = self.generate_python_function(&mut code, func_counter)?;
                current_loc += func_lines;
                func_counter += 1;
            } else {
                // Fill remaining with simple statements
                writeln!(&mut code, "\n# Module-level configuration")?;
                writeln!(&mut code, "DEBUG = True")?;
                writeln!(&mut code, "VERSION = '1.0.0'")?;
                current_loc += 3;
                break;
            }
        }

        // Add main guard
        writeln!(&mut code, "\nif __name__ == '__main__':")?;
        writeln!(&mut code, "    print('Benchmark fixture')")?;
        current_loc += 2;

        // Write to file
        let output_path = self.output_dir.join("python").join(filename);
        fs::create_dir_all(output_path.parent().unwrap())?;
        fs::write(&output_path, &code)?;

        // Validate with Python compiler
        self.validate_python(&output_path)?;

        let actual_loc = code.lines().count();
        println!("  ✅ Generated: {actual_loc} LOC (target: {target_loc})");

        Ok(())
    }

    /// Generate a Python class (returns lines generated)
    fn generate_python_class(&self, code: &mut String, idx: usize) -> Result<usize, std::fmt::Error> {
        let class_name = format!("DataProcessor{idx}");
        let mut lines = 0;

        writeln!(code, "\nclass {class_name}:")?;
        lines += 1;

        // Docstring
        writeln!(code, "    \"\"\"Process data with various transformations.\"\"\"")?;
        lines += 1;

        // __init__
        writeln!(code, "\n    def __init__(self, config: Optional[Dict[str, Any]] = None):")?;
        writeln!(code, "        self.config = config or {{}}")?;
        writeln!(code, "        self.data: List[int] = []")?;
        writeln!(code, "        self.processed = False")?;
        lines += 4;

        // add method
        writeln!(code, "\n    def add(self, value: int) -> None:")?;
        writeln!(code, "        \"\"\"Add a value to the dataset.\"\"\"")?;
        writeln!(code, "        if value is not None:")?;
        writeln!(code, "            self.data.append(value)")?;
        lines += 4;

        // process method
        writeln!(code, "\n    def process(self) -> List[int]:")?;
        writeln!(code, "        \"\"\"Process all data with transformation.\"\"\"")?;
        writeln!(code, "        result = []")?;
        writeln!(code, "        for item in self.data:")?;
        writeln!(code, "            if item > 0:")?;
        writeln!(code, "                transformed = item * 2")?;
        writeln!(code, "                result.append(transformed)")?;
        writeln!(code, "            elif item < 0:")?;
        writeln!(code, "                result.append(abs(item))")?;
        writeln!(code, "        self.processed = True")?;
        writeln!(code, "        return result")?;
        lines += 11;

        // reset method
        writeln!(code, "\n    def reset(self) -> None:")?;
        writeln!(code, "        \"\"\"Reset processor state.\"\"\"")?;
        writeln!(code, "        self.data.clear()")?;
        writeln!(code, "        self.processed = False")?;
        lines += 4;

        // property
        writeln!(code, "\n    @property")?;
        writeln!(code, "    def size(self) -> int:")?;
        writeln!(code, "        \"\"\"Get current dataset size.\"\"\"")?;
        writeln!(code, "        return len(self.data)")?;
        lines += 4;

        Ok(lines)
    }

    /// Generate a Python function (returns lines generated)
    fn generate_python_function(&self, code: &mut String, idx: usize) -> Result<usize, std::fmt::Error> {
        let func_name = format!("process_items_{idx}");
        let mut lines = 0;

        writeln!(code, "\ndef {func_name}(items: List[int], threshold: int = 0) -> Dict[str, Any]:")?;
        lines += 1;

        // Docstring
        writeln!(code, "    \"\"\"Process items and return statistics.\n    \n    Args:\n        items: List of integers to process\n        threshold: Minimum value to include\n        \n    Returns:\n        Dictionary with processing results\n    \"\"\"")?;
        lines += 1;

        // Function body
        writeln!(code, "    if not items:")?;
        writeln!(code, "        return {{'count': 0, 'sum': 0, 'average': 0.0}}")?;
        writeln!(code, "    ")?;
        writeln!(code, "    filtered = [x for x in items if x > threshold]")?;
        writeln!(code, "    count = len(filtered)")?;
        writeln!(code, "    total = sum(filtered)")?;
        writeln!(code, "    average = total / count if count > 0 else 0.0")?;
        writeln!(code, "    ")?;
        writeln!(code, "    return {{")?;
        writeln!(code, "        'count': count,")?;
        writeln!(code, "        'sum': total,")?;
        writeln!(code, "        'average': average,")?;
        writeln!(code, "        'min': min(filtered) if filtered else None,")?;
        writeln!(code, "        'max': max(filtered) if filtered else None,")?;
        writeln!(code, "    }}")?;
        lines += 14;

        Ok(lines)
    }

    /// Generate a JavaScript fixture
    fn generate_javascript_fixture(&self, filename: &str, target_loc: usize) -> Result<(), Box<dyn std::error::Error>> {
        println!("Generating {filename} (target: {target_loc} LOC)...");

        let mut code = String::new();

        // Add header comment
        writeln!(
            &mut code,
            "// Generated JavaScript fixture for benchmarking\n\
             // Target LOC: {target_loc}\n\
             // License: MIT (generated code)\n\
             // DO NOT EDIT MANUALLY - Regenerate with: cargo run -p benchmarks --bin generate-fixtures\n"
        )?;

        let mut current_loc = 5;

        // Generate ES6 classes and functions
        let mut class_counter = 0;
        let mut func_counter = 0;

        while current_loc < target_loc {
            let remaining = target_loc - current_loc;

            if remaining > 40 && current_loc % 80 < 50 {
                let class_lines = self.generate_javascript_class(&mut code, class_counter)?;
                current_loc += class_lines;
                class_counter += 1;
            } else if remaining > 15 {
                let func_lines = self.generate_javascript_function(&mut code, func_counter)?;
                current_loc += func_lines;
                func_counter += 1;
            } else {
                writeln!(&mut code, "\n// Module exports")?;
                writeln!(&mut code, "module.exports = {{ DataProcessor0 }};")?;
                current_loc += 2;
                break;
            }
        }

        // Write to file
        let output_path = self.output_dir.join("javascript").join(filename);
        fs::create_dir_all(output_path.parent().unwrap())?;
        fs::write(&output_path, &code)?;

        // Validate with Node.js
        self.validate_javascript(&output_path)?;

        let actual_loc = code.lines().count();
        println!("  ✅ Generated: {actual_loc} LOC (target: {target_loc})");

        Ok(())
    }

    /// Generate a JavaScript class
    fn generate_javascript_class(&self, code: &mut String, idx: usize) -> Result<usize, std::fmt::Error> {
        let class_name = format!("DataProcessor{idx}");
        let mut lines = 0;

        writeln!(code, "\nclass {class_name} {{")?;
        lines += 1;

        // Constructor
        writeln!(code, "  constructor(config = {{}}) {{")?;
        writeln!(code, "    this.config = config;")?;
        writeln!(code, "    this.data = [];")?;
        writeln!(code, "    this.processed = false;")?;
        writeln!(code, "  }}")?;
        lines += 5;

        // add method
        writeln!(code, "\n  add(value) {{")?;
        writeln!(code, "    if (value !== null && value !== undefined) {{")?;
        writeln!(code, "      this.data.push(value);")?;
        writeln!(code, "    }}")?;
        writeln!(code, "  }}")?;
        lines += 5;

        // process method
        writeln!(code, "\n  process() {{")?;
        writeln!(code, "    const result = [];")?;
        writeln!(code, "    for (const item of this.data) {{")?;
        writeln!(code, "      if (item > 0) {{")?;
        writeln!(code, "        result.push(item * 2);")?;
        writeln!(code, "      }} else if (item < 0) {{")?;
        writeln!(code, "        result.push(Math.abs(item));")?;
        writeln!(code, "      }}")?;
        writeln!(code, "    }}")?;
        writeln!(code, "    this.processed = true;")?;
        writeln!(code, "    return result;")?;
        writeln!(code, "  }}")?;
        lines += 12;

        // reset method
        writeln!(code, "\n  reset() {{")?;
        writeln!(code, "    this.data = [];")?;
        writeln!(code, "    this.processed = false;")?;
        writeln!(code, "  }}")?;
        lines += 4;

        // getter
        writeln!(code, "\n  get size() {{")?;
        writeln!(code, "    return this.data.length;")?;
        writeln!(code, "  }}")?;
        writeln!(code, "}}")?;
        lines += 4;

        Ok(lines)
    }

    /// Generate a JavaScript function
    fn generate_javascript_function(&self, code: &mut String, idx: usize) -> Result<usize, std::fmt::Error> {
        let func_name = format!("processItems{idx}");
        let mut lines = 0;

        writeln!(code, "\nfunction {func_name}(items, threshold = 0) {{")?;
        lines += 1;

        writeln!(code, "  if (!items || items.length === 0) {{")?;
        writeln!(code, "    return {{ count: 0, sum: 0, average: 0 }};")?;
        writeln!(code, "  }}")?;
        writeln!(code, "  ")?;
        writeln!(code, "  const filtered = items.filter(x => x > threshold);")?;
        writeln!(code, "  const count = filtered.length;")?;
        writeln!(code, "  const sum = filtered.reduce((a, b) => a + b, 0);")?;
        writeln!(code, "  const average = count > 0 ? sum / count : 0;")?;
        writeln!(code, "  ")?;
        writeln!(code, "  return {{")?;
        writeln!(code, "    count,")?;
        writeln!(code, "    sum,")?;
        writeln!(code, "    average,")?;
        writeln!(code, "    min: filtered.length > 0 ? Math.min(...filtered) : null,")?;
        writeln!(code, "    max: filtered.length > 0 ? Math.max(...filtered) : null")?;
        writeln!(code, "  }};")?;
        writeln!(code, "}}")?;
        lines += 16;

        Ok(lines)
    }

    /// Validate Python file with python -m py_compile
    fn validate_python(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("python3")
            .args(["-m", "py_compile", path.to_str().unwrap()])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                println!("    Validated with python3 -m py_compile ✓");
                Ok(())
            }
            Ok(output) => {
                eprintln!("Python validation failed:");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                Err("Python syntax error".into())
            }
            Err(_) => {
                println!("    ⚠️  python3 not available, skipping validation");
                Ok(())
            }
        }
    }

    /// Validate JavaScript file with node --check
    fn validate_javascript(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("node")
            .args(["--check", path.to_str().unwrap()])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                println!("    Validated with node --check ✓");
                Ok(())
            }
            Ok(output) => {
                eprintln!("JavaScript validation failed:");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                Err("JavaScript syntax error".into())
            }
            Err(_) => {
                println!("    ⚠️  node not available, skipping validation");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_generation() {
        let temp_dir = std::env::temp_dir().join("rust-sitter-fixtures-test");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let generator = FixtureGenerator::new(&temp_dir);
        generator.generate_all().unwrap();

        // Verify files exist
        assert!(temp_dir.join("python/small.py").exists());
        assert!(temp_dir.join("python/medium.py").exists());
        assert!(temp_dir.join("python/large.py").exists());
        assert!(temp_dir.join("javascript/small.js").exists());
        assert!(temp_dir.join("javascript/medium.js").exists());
        assert!(temp_dir.join("javascript/large.js").exists());

        // Verify LOC roughly match targets (within 20%)
        let python_small = std::fs::read_to_string(temp_dir.join("python/small.py")).unwrap();
        let small_loc = python_small.lines().count();
        assert!(
            small_loc >= 80 && small_loc <= 120,
            "Python small LOC {} not in range [80, 120]",
            small_loc
        );

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
