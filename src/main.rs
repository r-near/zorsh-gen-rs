use anyhow::Result;
use zorsh_gen_rs::{Config, ZorshConverter};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input_dir> <output_dir>", args[0]);
        std::process::exit(1);
    }

    let config = Config::default();
    let converter = ZorshConverter::new(&args[1], &args[2], config);
    converter.convert()?;

    Ok(())
}
