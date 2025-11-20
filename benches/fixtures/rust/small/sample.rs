// Small Rust fixture for benchmarking (~75 LOC)
// Part of v0.8.0 Performance Optimization

/// Calculate Fibonacci number recursively
fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

/// Calculate factorial iteratively
fn factorial(n: u64) -> u64 {
    (1..=n).product()
}

/// Simple Calculator struct
#[derive(Debug, Default)]
struct Calculator {
    result: f64,
}

impl Calculator {
    /// Create a new Calculator
    fn new() -> Self {
        Self::default()
    }

    /// Add two numbers
    fn add(&mut self, x: f64, y: f64) -> f64 {
        self.result = x + y;
        self.result
    }

    /// Subtract two numbers
    fn subtract(&mut self, x: f64, y: f64) -> f64 {
        self.result = x - y;
        self.result
    }

    /// Multiply two numbers
    fn multiply(&mut self, x: f64, y: f64) -> f64 {
        self.result = x * y;
        self.result
    }

    /// Divide two numbers
    fn divide(&mut self, x: f64, y: f64) -> Result<f64, String> {
        if y == 0.0 {
            Err("Division by zero".to_string())
        } else {
            self.result = x / y;
            Ok(self.result)
        }
    }
}

/// Array utility functions
mod array_utils {
    pub fn sum(arr: &[i32]) -> i32 {
        arr.iter().sum()
    }

    pub fn average(arr: &[i32]) -> f64 {
        sum(arr) as f64 / arr.len() as f64
    }

    pub fn max(arr: &[i32]) -> Option<&i32> {
        arr.iter().max()
    }

    pub fn min(arr: &[i32]) -> Option<&i32> {
        arr.iter().min()
    }
}

fn main() {
    println!("Fibonacci(10) = {}", fibonacci(10));
    println!("Factorial(5) = {}", factorial(5));

    let mut calc = Calculator::new();
    println!("10 + 5 = {}", calc.add(10.0, 5.0));
    println!("10 - 5 = {}", calc.subtract(10.0, 5.0));
    println!("10 * 5 = {}", calc.multiply(10.0, 5.0));
    println!("10 / 5 = {}", calc.divide(10.0, 5.0).unwrap());

    let numbers = [1, 2, 3, 4, 5];
    println!("Sum: {}", array_utils::sum(&numbers));
    println!("Average: {}", array_utils::average(&numbers));
}
