use clap::Parser;
use rust_sitter_tool::cli::{Cli, Commands, run_generate, run_info, run_init, run_parse, run_test};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate {
            grammar,
            output,
            debug,
            pure_rust,
        } => run_generate(grammar, output, *debug, *pure_rust),
        Commands::Parse {
            file,
            parser,
            format,
            fields,
            stats,
        } => run_parse(file, parser, format, *fields, *stats),
        Commands::Test {
            path,
            filter,
            update,
        } => run_test(path, filter, *update),
        Commands::Init { name, in_place } => run_init(name, *in_place),
        Commands::Info {
            path,
            node_types,
            rules,
        } => run_info(path, *node_types, *rules),
    }
}
