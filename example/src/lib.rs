// Re-export modules that contain grammars
pub mod arithmetic;

// Re-export the get_language function for arithmetic grammar
pub use arithmetic::get_language as get_arithmetic_language;
