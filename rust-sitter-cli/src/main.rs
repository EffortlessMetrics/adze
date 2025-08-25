use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Parse an input file; in dynamic mode, load grammar dylib at runtime
    Parse {
        /// Path to grammar (crate name or dylib when --dynamic)
        #[arg(long)]
        grammar: String,
        /// Input file path
        #[arg(long)]
        input: String,
        /// Use dynamic loader (requires --features dynamic)
        #[arg(long)]
        dynamic: bool,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.cmd {
        Cmd::Parse {
            grammar,
            input,
            dynamic,
        } => {
            let text = fs::read_to_string(&input)?;
            if dynamic {
                #[cfg(feature = "dynamic")]
                unsafe {
                    use libloading::Library;
                    // NOTE: adjust symbol type to your TSLanguage ABI bridge
                    let lib = Library::new(&grammar)?;
                    let sym: libloading::Symbol<unsafe extern "C" fn() -> *const u8> =
                        lib.get(b"language")?;
                    let _lang_ptr = sym();
                    // TODO: feed into pure parser bridge and print a JSON-ish summary
                    println!("(dynamic) loaded language from {}", grammar);
                    println!("input bytes = {}", text.len());
                }
                #[cfg(not(feature = "dynamic"))]
                {
                    eprintln!("binary not built with --features dynamic");
                    std::process::exit(2);
                }
            } else {
                // Static path placeholder — wire to your existing parser entry
                println!("(static) grammar = {grammar}, input bytes = {}", text.len());
                // TODO: run pure parser and print a tiny tree summary
            }
        }
    }
    Ok(())
}