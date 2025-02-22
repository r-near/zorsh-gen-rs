pub mod code_generator;
pub mod converter;
pub mod dependency_resolver;
pub mod source_loader;
pub mod type_parser;

use anyhow::Result;
use std::path::Path;

// Re-export main types for easier usage
pub use code_generator::ZorshGenerator;
pub use converter::ZorshConverter;
pub use dependency_resolver::DependencyResolver;
pub use source_loader::SourceLoader;
pub use type_parser::TypeParser;

/// Configuration options for the Zorsh generator
#[derive(Debug, Clone)]
pub struct Config {
    /// Only process structs with #[derive(BorshSerialize)]
    pub only_annotated: bool,
    /// Skip files and directories matching these patterns
    pub ignored_patterns: Vec<String>,
    /// Output directory structure (flat or nested)
    pub output_structure: OutputStructure,
}

#[derive(Debug, Clone)]
pub enum OutputStructure {
    /// Maintain the same directory structure as input
    Nested,
    /// Put all files in a single directory
    Flat,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            only_annotated: true,
            ignored_patterns: vec![
                "tests/".to_string(),
                "examples/".to_string(),
                "target/".to_string(),
            ],
            output_structure: OutputStructure::Nested,
        }
    }
}

/// Main entry point for the library
pub struct ZorshGen {
    config: Config,
}

impl ZorshGen {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Convert Rust files in input_path to Zorsh TypeScript files in output_path
    pub fn convert<P: AsRef<Path>>(&self, input_path: P, output_path: P) -> Result<()> {
        let converter = ZorshConverter::new(input_path, output_path, self.config.clone());
        converter.convert()
    }

    /// Process a single Rust file and return the generated Zorsh code as a string
    pub fn convert_str(&self, rust_code: &str) -> Result<String> {
        let mut parser = TypeParser::new("root".to_string(), self.config.only_annotated.clone());
        parser.parse_file(rust_code)?;

        let resolver = DependencyResolver::new(parser.structs.clone(), parser.enums.clone());
        let dependencies = resolver.resolve()?;

        let generator = ZorshGenerator::new(parser.structs, parser.enums);

        // Since we're processing a single string, treat it as a single module
        generator.generate_module("root", &dependencies)
    }
}

// Convenience function for quick conversions
pub fn convert_str(rust_code: &str) -> Result<String> {
    ZorshGen::new(Config::default()).convert_str(rust_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_struct() -> Result<()> {
        let input = r#"
            #[derive(BorshSerialize)]
            struct User {
                name: String,
                age: u32,
            }
        "#;

        let output = convert_str(input)?;
        assert!(output.contains("export const UserSchema"));
        assert!(output.contains("b.string()"));
        assert!(output.contains("b.u32()"));
        Ok(())
    }

    #[test]
    fn test_complex_types() -> Result<()> {
        let input = r#"
            #[derive(BorshSerialize)]
            struct Player {
                inventory: HashMap<String, Vec<Item>>,
                status: Option<PlayerStatus>,
            }

            #[derive(BorshSerialize)]
            struct Item {
                name: String,
                quantity: u32,
            }

            #[derive(BorshSerialize)]
            enum PlayerStatus {
                Idle,
                Fighting,
            }
        "#;

        let output = convert_str(input)?;
        assert!(output.contains("b.hashMap("));
        assert!(output.contains("b.vec("));
        assert!(output.contains("b.option("));
        Ok(())
    }
}
