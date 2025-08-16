fn main() {
    ts_bridge::ffi::assert_abi_compatible();
    println!("Tree-sitter ABI check passed (v15)");
}
