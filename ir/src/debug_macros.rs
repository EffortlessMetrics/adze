// Debug macros for IR development

//! Debug macros for inspecting grammar IR at build time.

/// Emit IR for debugging purposes
#[macro_export]
macro_rules! emit_ir {
    ($grammar:expr) => {
        if std::env::var("ADZE_DEBUG_IR").is_ok() {
            eprintln!("=== Grammar IR ===");
            eprintln!("{:#?}", $grammar);
            eprintln!("==================");
        }
    };
    ($label:expr, $grammar:expr) => {
        if std::env::var("ADZE_DEBUG_IR").is_ok() {
            eprintln!("=== {} ===", $label);
            eprintln!("{:#?}", $grammar);
            eprintln!("==================");
        }
    };
}
