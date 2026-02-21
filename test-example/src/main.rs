// Simple arithmetic grammar test

#[adze::grammar("arithmetic")]
mod grammar {
    #[adze::language]
    pub struct Expr {
        #[adze::leaf(pattern = r"\d+", transform = |s| s.parse().unwrap())]
        pub value: i32,
    }
}

fn main() {
    println!("Test example for adze");

    // Try to parse a simple number
    match grammar::parse("42") {
        Ok(expr) => println!("Parsed: {:?}", expr.value),
        Err(e) => println!("Parse error: {:?}", e),
    }
}
