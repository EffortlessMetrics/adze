use adze_python::grammar_python::LANGUAGE as PY_LANGUAGE;

fn main() {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }
    println!("A: taking &PY_LANGUAGE");
    let _ = &PY_LANGUAGE;
    println!("B: success");
}
