pub mod extract;
pub mod ffi;
pub mod schema;

pub use extract::extract;
pub use schema::*;

// Export at least one symbol for proper linking
#[cfg(adze_unsafe_attrs)]
#[unsafe(no_mangle)]
#[cfg(not(adze_unsafe_attrs))]
#[no_mangle]
pub extern "C" fn rs_ts_bridge_version() -> u32 {
    1
}
