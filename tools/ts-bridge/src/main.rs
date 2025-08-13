use libloading::{Library, Symbol};
use ts_bridge::extract;

type LangFn = unsafe extern "C" fn() -> *const ts_bridge::ffi::TSLanguage;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <path/to/libtree-sitter-*.so> [output.json] [symbol_name]", args[0]);
        eprintln!("Example: {} libtree-sitter-json.so parse_table.json tree_sitter_json", args[0]);
        std::process::exit(1);
    }
    
    let path = &args[1];
    let out = args.get(2)
        .map(|s| s.as_str())
        .unwrap_or("parse_table.json");
    let sym_name = args.get(3)
        .map(|s| s.as_str())
        .unwrap_or("tree_sitter_json");

    eprintln!("Loading library: {}", path);
    eprintln!("Looking for symbol: {}", sym_name);
    
    let lib = unsafe { 
        Library::new(path)
            .map_err(|e| anyhow::anyhow!("Failed to load library {}: {}", path, e))?
    };
    
    let lang_fn: Symbol<LangFn> = unsafe { 
        lib.get(sym_name.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to find symbol '{}': {}", sym_name, e))?
    };

    eprintln!("Extracting parse tables...");
    let data = extract(*lang_fn)?;
    
    eprintln!("Extracted data:");
    eprintln!("  - {} symbols", data.symbol_count);
    eprintln!("  - {} states", data.state_count);
    eprintln!("  - {} rules", data.rules.len());
    eprintln!("  - {} action cells", data.actions.len());
    eprintln!("  - {} goto cells", data.gotos.len());
    eprintln!("  - start symbol: {} ('{}')", data.start_symbol, &data.symbol_names[data.start_symbol as usize]);
    
    let json = serde_json::to_vec_pretty(&data)?;
    std::fs::write(out, &json)?;
    eprintln!("Wrote {} bytes to {}", json.len(), out);
    
    Ok(())
}