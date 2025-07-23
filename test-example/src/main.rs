// Simple arithmetic grammar test

#[rust_sitter::grammar("arithmetic")]
mod grammar {
    #[rust_sitter::language]
    pub struct Expr {
        #[rust_sitter::leaf(pattern = r"\d+", transform = |s| s.parse().unwrap())]
        pub value: i32,
    }
}

fn main() {
    println!("Test example for rust-sitter");
    
    // Try to parse a simple number
    match grammar::parse("42") {
        Ok(expr) => println!("Parsed: {:?}", expr.value),
        Err(e) => println!("Parse error: {:?}", e),
    }
}