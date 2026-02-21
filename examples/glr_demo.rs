//! Demonstrates GLR parsing capabilities with ambiguous grammars
//!
//! This example shows how adze handles ambiguous parse cases
//! using GLR (Generalized LR) parsing with SPPF (Shared Packed Parse Forest).

use adze::*;

// Example 1: Classic dangling-else ambiguity
#[adze::grammar("dangling_else")]
pub struct DanglingElseGrammar;

pub enum Statement {
    If {
        condition: String,
        then_stmt: Box<Statement>,
        else_stmt: Option<Box<Statement>>,
    },
    Block(String),
}

// Example 2: Arithmetic with ambiguous precedence
#[adze::grammar("ambiguous_math")]
pub struct AmbiguousMathGrammar;

pub enum Expr {
    Binary {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
    Number(i32),
}

fn main() {
    println!("GLR Parsing Demo");
    println!("================\n");

    // Example of dangling else
    println!("Dangling Else Example:");
    println!("Input: if a then if b then c else d");
    println!("Possible parses:");
    println!("1. if a then (if b then c else d)");
    println!("2. if a then (if b then c) else d\n");

    // Example of arithmetic ambiguity
    println!("Arithmetic Ambiguity Example:");
    println!("Input: 1 + 2 * 3");
    println!("Without precedence, this could be:");
    println!("1. (1 + 2) * 3 = 9");
    println!("2. 1 + (2 * 3) = 7\n");

    println!("GLR parsing creates a forest representing all valid parses.");
    println!("The application can then choose the intended interpretation.");
}
