pub mod extract;
pub mod ffi;
pub mod schema;

pub use extract::extract;
pub use schema::*;

// Export at least one symbol for proper linking
#[no_mangle]
pub extern "C" fn rs_ts_bridge_version() -> u32 {
    1
}

// Compile-time guard to prevent conflicting features
#[cfg(all(feature = "stub-ts", feature = "with-grammars"))]
compile_error!("features `stub-ts` and `with-grammars` cannot be enabled together.");
