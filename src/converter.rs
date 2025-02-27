use anyhow::{Context, Result};
use log::{debug, info};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::code_generator::ZorshGenerator;
use crate::dependency_resolver::DependencyResolver;
use crate::source_loader::SourceLoader;
use crate::type_parser::TypeParser;
use crate::OutputStructure;
use colored::Colorize;

pub struct ZorshConverter {
    source_loader: SourceLoader,
    output_dir: PathBuf,
    config: crate::Config,
}

impl ZorshConverter {
    pub fn new<P: AsRef<Path>>(input_path: P, output_path: P, config: crate::Config) -> Self {
        Self {
            source_loader: SourceLoader::new(input_path, config.ignored_patterns.clone()),
            output_dir: output_path.as_ref().to_path_buf(),
            config,
        }
    }

    fn get_output_path(&self, module_path: &str) -> PathBuf {
        match self.config.output_structure {
            OutputStructure::Nested => self
                .output_dir
                .join(format!("{}.ts", module_path.replace("::", "/")).to_lowercase()),
            OutputStructure::Flat => self
                .output_dir
                .join(format!("{}.ts", module_path.replace("::", "_")).to_lowercase()),
        }
    }

    pub fn convert(&self) -> Result<()> {
        // Find and load all Rust files
        info!("🔎 Discovering Rust files...");
        let source_files = self.source_loader.discover_rust_files()?;
        info!(
            "📂 Found {} Rust files",
            source_files.len().to_string().green()
        );

        // Parse types from each file
        info!("\n📝 Parsing types from files...");
        let mut all_structs = HashMap::new();
        let mut all_enums = HashMap::new();

        for source_file in &source_files {
            info!(
                "  Processing {}",
                source_file.path.display().to_string().cyan()
            );
            let mut parser =
                TypeParser::new(source_file.module_path.clone(), self.config.only_annotated);
            parser.parse_file(&source_file.content)?;

            debug!(
                "    Found {} structs and {} enums",
                parser.structs.len().to_string().yellow(),
                parser.enums.len().to_string().yellow()
            );

            all_structs.extend(parser.structs);
            all_enums.extend(parser.enums);
        }

        info!("\n🔄 Resolving type dependencies...");
        // Resolve dependencies
        let resolver = DependencyResolver::new(all_structs.clone(), all_enums.clone());
        let dependencies = resolver.resolve()?;
        info!(
            "✨ Found {} total types in dependency order",
            dependencies.ordered_types.len().to_string().green()
        );

        // Get unique set of modules
        let mut modules = HashSet::new();
        for type_path in &dependencies.ordered_types {
            if let Some(struct_info) = all_structs.get(type_path) {
                modules.insert(struct_info.module_path.clone());
            } else if let Some(enum_info) = all_enums.get(type_path) {
                modules.insert(enum_info.module_path.clone());
            }
        }

        // Generate code for each module
        info!("\n🏗️ Generating TypeScript code...");
        let generator = ZorshGenerator::new(all_structs, all_enums);

        for module in modules {
            let file_path = self.get_output_path(&module);
            info!("  Generating {}", file_path.display().to_string().cyan());

            // Create parent directories if they don't exist
            if let Some(parent) = file_path.parent() {
                debug!("    Creating directory: {}", parent.display());
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }

            // Generate and write code
            let code = generator.generate_module(&module, &dependencies)?;
            fs::write(&file_path, code)
                .with_context(|| format!("Failed to write file: {}", file_path.display()))?;
            info!("    ✅ {}", "Done".green());
        }

        Ok(())
    }
}
