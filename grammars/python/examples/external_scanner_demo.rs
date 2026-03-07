use std::panic::{self, AssertUnwindSafe};

use adze::pure_external_scanner::{ExternalScanner, ExternalScannerRegistry, Lexer};
use adze_python::grammar_python::LANGUAGE as PY_LANGUAGE;

#[derive(Default)]
struct PythonStringsScanner;

impl ExternalScanner for PythonStringsScanner {
    fn scan(&mut self, _lexer: &mut Lexer, _valid_symbols: &[bool]) -> bool {
        false
    }

    fn serialize(&self, _buf: &mut [u8]) -> usize {
        0
    }

    fn deserialize(&mut self, _buf: &[u8]) {}
}

fn main() {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    println!("A: about to take &PY_LANGUAGE");
    let res_a = panic::catch_unwind(|| {
        let _ = &PY_LANGUAGE;
    });
    if let Err(e) = res_a {
        eprintln!("PANIC at A (&PY_LANGUAGE): {e:?}");
        return;
    }
    println!("A✓");

    println!("B: creating registry");
    let mut registry = match panic::catch_unwind(ExternalScannerRegistry::default) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("PANIC at B (registry::default): {e:?}");
            return;
        }
    };
    println!("B✓");

    println!("C: registering scanner as \"python_strings\"");
    let res_c = panic::catch_unwind(AssertUnwindSafe(|| {
        registry.register("python_strings".to_string(), Box::new(PythonStringsScanner));
    }));
    if let Err(e) = res_c {
        eprintln!("PANIC at C (registry::register): {e:?}");
        return;
    }
    println!("C✓");

    println!("D: done. exiting normally");
}
