// Ensure only one backend is enabled
#[cfg(all(feature = "pure-rust", feature = "c-backend"))]
compile_error!("Enable exactly one backend: 'pure-rust' OR 'c-backend'.");

// Re-export modules that contain grammars
pub mod ambiguous;
pub mod arithmetic;
pub mod external_word_example;
pub mod optionals;
pub mod performance_test;
pub mod repetitions;
pub mod test_precedence;
pub mod test_whitespace;
pub mod words;

// Tree-sitter compatibility language helpers
#[cfg(all(feature = "ts-compat", feature = "pure-rust"))]
pub mod ts_langs;
