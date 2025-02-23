mod basic_types;
mod complex_types;
mod config_tests;
mod module_structure;
mod type_aliases;

// Shared test utilities
use std::fs;
use tempfile::TempDir;
use zorsh_gen_rs::{Config, ZorshGen};

// Helper to setup temporary test directories
pub(crate) fn setup_test_dir() -> TempDir {
    env_logger::try_init().ok();
    tempfile::Builder::new()
        .prefix("zorsh_test_") // Use zorsh_test_ instead of .tmp
        .tempdir()
        .expect("Failed to create temp directory")
}

// Helper to write test files and return output path
pub(crate) fn setup_test_files(temp_dir: &TempDir, files: &[(&str, &str)]) -> std::path::PathBuf {
    for (path, content) in files {
        let full_path = temp_dir.path().join(path);
        println!("Writing file: {}", full_path.display());
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create directories");
        }
        fs::write(&full_path, content).expect("Failed to write test file");
    }
    temp_dir.path().to_path_buf()
}
