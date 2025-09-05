use anyhow::Result;
use clap::{Parser, Subcommand};
use rust_sitter::pure_parser::{ParsedNode, Parser, TSLanguage};
use std::fs;

fn node_to_sexp(node: &ParsedNode, source: &str, indent: usize) -> String {
    let indent_str = "  ".repeat(indent);
    if node.is_named() {
        let mut out = format!("{}({}", indent_str, node.kind());
        if node.child_count() == 0 {
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");
            out.push_str(&format!(" \"{}\")", text));
        } else {
            out.push('\n');
            for child in node.children() {
                out.push_str(&node_to_sexp(child, source, indent + 1));
                out.push('\n');
            }
            out.push_str(&format!("{indent_str})"));
        }
        out
    } else {
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");
        format!("{}\"{}\"", indent_str, text)
    }
}

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
                    // Load language symbol and run pure parser
                    let lib = Library::new(&grammar)?;
                    let sym: libloading::Symbol<unsafe extern "C" fn() -> *const u8> =
                        lib.get(b"language")?;
                    let lang_ptr = sym() as *const TSLanguage;
                    let language: &'static TSLanguage = &*lang_ptr;
                    let mut parser = Parser::new();
                    parser
                        .set_language(language)
                        .map_err(|e| anyhow::anyhow!(e))?;
                    let result = parser.parse_string(&text);
                    if let Some(root) = result.root {
                        println!("{}", node_to_sexp(&root, &text, 0));
                    } else {
                        eprintln!("parse failed: {:?}", result.errors);
                    }
                }
                #[cfg(not(feature = "dynamic"))]
                {
                    eprintln!("binary not built with --features dynamic");
                    std::process::exit(2);
                }
            } else {
                #[cfg(any(feature = "python-grammar", feature = "javascript-grammar"))]
                {
                    let language: &'static TSLanguage = match grammar.as_str() {
                        #[cfg(feature = "python-grammar")]
                        "python" => rust_sitter_python::get_language(),
                        #[cfg(feature = "javascript-grammar")]
                        "javascript" => &rust_sitter_javascript::grammar::LANGUAGE,
                        _ => {
                            eprintln!("unknown grammar: {}", grammar);
                            std::process::exit(1);
                        }
                    };
                    let mut parser = Parser::new();
                    parser
                        .set_language(language)
                        .map_err(|e| anyhow::anyhow!(e))?;
                    let result = parser.parse_string(&text);
                    if let Some(root) = result.root {
                        println!("{}", node_to_sexp(&root, &text, 0));
                    } else {
                        eprintln!("parse failed: {:?}", result.errors);
                    }
                }
                #[cfg(not(any(feature = "python-grammar", feature = "javascript-grammar")))]
                {
                    eprintln!("binary built without static grammars");
                    std::process::exit(2);
                }
            }
        }
    }
    Ok(())
}