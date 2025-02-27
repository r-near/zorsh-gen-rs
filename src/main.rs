use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;
use zorsh_gen_rs::{Config, OutputStructure, ZorshConverter};

/// Zorsh Generator for Rust - Convert Rust types to Zorsh TypeScript schemas
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

    /// Show detailed debug information during conversion
    #[arg(short, long)]
    verbose: bool,

    /// Show only errors (overrides --verbose)
    #[arg(short, long)]
    quiet: bool,
}

fn setup_logger(verbose: bool, quiet: bool) {
    let mut builder = Builder::new();

    // Set log level based on flags
    builder.filter_level(if quiet {
        LevelFilter::Error
    } else if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    });

    // Custom format with colors and emojis
    builder.format(|buf, record| {
        let level_emoji = match record.level() {
            log::Level::Error => "❌",
            log::Level::Warn => "⚠️ ",
            log::Level::Info => "ℹ️ ",
            log::Level::Debug => "🔍",
            log::Level::Trace => "📍",
        };

        let level_color = match record.level() {
            log::Level::Error => record.level().to_string().red(),
            log::Level::Warn => record.level().to_string().yellow(),
            log::Level::Info => record.level().to_string().cyan(),
            log::Level::Debug => record.level().to_string().purple(),
            log::Level::Trace => record.level().to_string().normal(),
        };

        if record.level() <= log::Level::Info {
            writeln!(buf, "{}", record.args())
        } else {
            writeln!(buf, "{} {} {}", level_emoji, level_color, record.args())
        }
    });

    builder.init();
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logger with custom format
    setup_logger(args.verbose, args.quiet);

    println!("\n{}\n", "🦘 Zorsh TypeScript Generator".bold());

    let config = Config {
        only_annotated: args.only_annotated,
        ignored_patterns: args.ignored_patterns,
        output_structure: args.output_structure,
    };

    let converter = ZorshConverter::new(&args.input_dir, &args.output_dir, config);
    converter.convert()?;

    println!(
        "\n{} Generated TypeScript schemas in: {}\n",
        "✨ Success!".green().bold(),
        args.output_dir.cyan()
    );

    Ok(())
}
