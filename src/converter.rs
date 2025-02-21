use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::code_generator::ZorshGenerator;
use crate::dependency_resolver::DependencyResolver;
use crate::source_loader::SourceLoader;
use crate::type_parser::TypeParser;

pub struct ZorshConverter {
    source_loader: SourceLoader,
    output_dir: PathBuf,
    config: crate::Config,
}

impl ZorshConverter {
    pub fn new<P: AsRef<Path>>(input_path: P, output_path: P, config: crate::Config) -> Self {
        Self {
            source_loader: SourceLoader::new(input_path),
            output_dir: output_path.as_ref().to_path_buf(),
            config,
        }
    }

    pub fn convert(&self) -> Result<()> {
        // Find and load all Rust files
        let source_files = self.source_loader.discover_rust_files()?;

        // Parse types from each file
        let mut all_structs = HashMap::new();
        let mut all_enums = HashMap::new();

        for source_file in &source_files {
            let mut parser = TypeParser::new(source_file.module_path.clone());
            parser.parse_file(&source_file.content)?;

            all_structs.extend(parser.structs);
            all_enums.extend(parser.enums);
        }

        // Resolve dependencies and group by module
        let resolver = DependencyResolver::new(all_structs.clone(), all_enums.clone());
        let module_types = resolver.resolve()?;

        // Generate Zorsh code for each module
        let generator = ZorshGenerator::new(all_structs, all_enums);

        for module in module_types {
            // Create module directory path
            let module_dir = self.output_dir.join(module.module_path.replace("::", "/"));
            fs::create_dir_all(&module_dir)
                .with_context(|| format!("Failed to create directory: {}", module_dir.display()))?;

            // Generate and write code
            let code = generator.generate_module(&module)?;
            fs::write(module_dir.join("index.ts"), code).with_context(|| {
                format!(
                    "Failed to write file: {}",
                    module_dir.join("index.ts").display()
                )
            })?;
        }

        Ok(())
    }
}
