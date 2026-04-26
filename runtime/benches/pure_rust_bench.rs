// Placeholder benchmark entrypoint.
// This binary exists so `cargo bench` can enumerate the target while
// pure-Rust benchmark APIs are stabilized. It does NOT run runtime benchmarks.

fn main() {
    println!(
        "placeholder: pure_rust_bench is disabled while unstable benchmark APIs are stabilized"
    );
}
