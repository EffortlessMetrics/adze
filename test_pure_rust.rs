// Simple test for pure-Rust parser
use std::process::Command;

fn main() {
    println!("Testing pure-Rust parser...");
    
    // Build with pure-rust feature
    let output = Command::new("cargo")
        .args(&["build", "-p", "rust-sitter-example", "--features", "pure-rust"])
        .output()
        .expect("Failed to execute cargo build");
    
    if !output.status.success() {
        eprintln!("Build failed!");
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
    
    println!("Build successful!");
    
    // Try to run a simple parse test
    let test_code = r#"
#[cfg(test)]
mod test {
    use super::arithmetic::grammar;
    
    #[test] 
    fn test_parse() {
        let result = grammar::parse("1 + 2");
        println!("Parse result: {:?}", result);
    }
}
"#;
    
    std::fs::write("test_arithmetic.rs", test_code).expect("Failed to write test file");
    
    println!("Test file created. Run with: cargo test --features pure-rust");
}