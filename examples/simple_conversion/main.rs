use anyhow::Result;
use std::path::PathBuf;
use zorsh_gen_rs::{Config, OutputStructure, ZorshGen};

fn main() -> Result<()> {
    // Set up paths relative to the example directory
    let example_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let input_dir = example_dir.join("input");
    let output_dir = example_dir.join("output");

    // Create custom configuration
    let config = Config {
        only_annotated: true,     // Only process types with BorshSerialize
        ignored_patterns: vec![], // Don't ignore any files
        output_structure: OutputStructure::Nested, // Maintain directory structure
    };

    // Initialize the generator
    let generator = ZorshGen::new(config);

    // Convert Rust models to Zorsh TypeScript
    println!(
        "Converting Rust models from {:?} to Zorsh TypeScript in {:?}",
        input_dir, output_dir
    );
    generator.convert(input_dir, output_dir)?;

    // You can also convert a string directly
    let rust_code = r#"
        #[derive(BorshSerialize)]
        struct SimpleExample {
            name: String,
            count: u32,
        }
    "#;

    let zorsh_code = zorsh_gen_rs::convert_str(rust_code)?;
    println!("\nDirect string conversion example:");
    println!("{}", zorsh_code);

    Ok(())
}
