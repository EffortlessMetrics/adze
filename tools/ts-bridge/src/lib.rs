pub mod extract;
pub mod ffi;
pub mod schema;

pub use extract::extract;
pub use schema::*;

// Compile-time guard to prevent conflicting features
#[cfg(all(feature = "stub-ts", feature = "with-grammars"))]
compile_error!("features `stub-ts` and `with-grammars` cannot be enabled together.");
