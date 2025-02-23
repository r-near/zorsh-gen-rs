use anyhow::Result;
use clap::Parser;
use zorsh_gen_rs::{Config, OutputStructure, ZorshConverter};

/// Zorsh Generator for Rust
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input directory containing Rust files
    #[arg(value_name = "INPUT_DIR")]
    input_dir: String,

    /// Output directory for generated TypeScript files
    #[arg(value_name = "OUTPUT_DIR")]
    output_dir: String,

    /// Output structure: nested or flat
    #[arg(long, value_enum, default_value_t = OutputStructure::Nested)]
    output_structure: OutputStructure,

    /// Only process structs and enums with #[derive(BorshSerialize)] or #[derive(BorshDeserialize)]
    #[arg(long, default_value_t = true)]
    only_annotated: bool,

    /// Ignore files and directories matching these comma-separated patterns (e.g., "tests/,examples/,target/")
    #[arg(long, value_delimiter = ',')]
    ignored_patterns: Vec<String>,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    let config = Config {
        only_annotated: args.only_annotated,
        ignored_patterns: args.ignored_patterns,
        output_structure: args.output_structure,
    };

    let converter = ZorshConverter::new(&args.input_dir, &args.output_dir, config);
    converter.convert()?;

    println!(
        "🎉 Zorsh TypeScript schemas generated successfully in: {}",
        args.output_dir
    );

    Ok(())
}
