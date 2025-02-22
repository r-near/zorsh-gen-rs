use super::*;
use anyhow::Result;
use zorsh_gen_rs::OutputStructure;

#[test]
fn test_only_annotated_config() -> Result<()> {
    let temp_dir = setup_test_dir();

    let files = vec![(
        "src/lib.rs",
        r#"
            // Has BorshSerialize
            #[derive(BorshSerialize)]
            pub struct Annotated {
                field: String,
            }

            // No BorshSerialize
            pub struct Unannotated {
                field: String,
            }
        "#,
    )];

    let input_dir = setup_test_files(&temp_dir, &files);
    let output_dir = temp_dir.path().join("generated");

    // Test with only_annotated = true (default)
    let default_config = Config::default();
    let generator = ZorshGen::new(default_config);
    generator.convert(&input_dir, &output_dir)?;

    let content = fs::read_to_string(output_dir.join("src/lib.ts"))?;
    assert!(content.contains("AnnotatedSchema"));
    assert!(!content.contains("UnannotatedSchema"));

    // Clean output directory
    fs::remove_dir_all(&output_dir)?;
    fs::create_dir(&output_dir)?;

    // Test with only_annotated = false
    let mut config = Config::default();
    config.only_annotated = false;
    let generator = ZorshGen::new(config);
    generator.convert(&input_dir, &output_dir)?;

    let content = fs::read_to_string(output_dir.join("src/lib.ts"))?;
    assert!(content.contains("AnnotatedSchema"));
    assert!(content.contains("UnannotatedSchema"));

    Ok(())
}

#[test]
fn test_ignored_patterns_config() -> Result<()> {
    let temp_dir = setup_test_dir();

    let files = vec![
        (
            "src/lib.rs",
            r#"
                #[derive(BorshSerialize)]
                pub struct MainStruct {
                    field: String,
                }
            "#,
        ),
        (
            "src/generated/types.rs",
            r#"
                #[derive(BorshSerialize)]
                pub struct GeneratedType {
                    field: String,
                }
            "#,
        ),
        (
            "src/test_utils/helpers.rs",
            r#"
                #[derive(BorshSerialize)]
                pub struct TestHelper {
                    field: String,
                }
            "#,
        ),
    ];

    let input_dir = setup_test_files(&temp_dir, &files);
    let output_dir = temp_dir.path().join("output");

    // Test with custom ignored patterns
    let mut config = Config::default();
    config.ignored_patterns = vec!["generated/".to_string(), "test_utils/".to_string()];

    let generator = ZorshGen::new(config);
    generator.convert(&input_dir, &output_dir)?;

    // MainStruct should be processed
    assert!(output_dir.join("src/lib.ts").exists());
    let main_content = fs::read_to_string(output_dir.join("src/lib.ts"))?;
    assert!(main_content.contains("MainStructSchema"));

    // GeneratedType and TestHelper should be ignored
    assert!(!output_dir.join("src/generated/types.ts").exists());
    assert!(!output_dir.join("src/test_utils/helpers.ts").exists());

    Ok(())
}

#[test]
fn test_output_structure_config() -> Result<()> {
    let temp_dir = setup_test_dir();

    let files = vec![
        (
            "src/models/user.rs",
            r#"
                #[derive(BorshSerialize)]
                pub struct User {
                    name: String,
                }
            "#,
        ),
        (
            "src/models/post.rs",
            r#"
                #[derive(BorshSerialize)]
                pub struct Post {
                    title: String,
                }
            "#,
        ),
    ];

    let input_dir = setup_test_files(&temp_dir, &files);

    // Test nested structure
    {
        let output_dir = temp_dir.path().join("output_nested");
        let config = Config {
            output_structure: OutputStructure::Nested,
            ..Config::default()
        };

        let generator = ZorshGen::new(config);
        generator.convert(&input_dir, &output_dir)?;

        // Should maintain directory structure
        assert!(output_dir.join("src/models/user.ts").exists());
        assert!(output_dir.join("src/models/post.ts").exists());
    }

    // Test flat structure
    {
        let output_dir = temp_dir.path().join("output_flat");
        let config = Config {
            output_structure: OutputStructure::Flat,
            ..Config::default()
        };

        let generator = ZorshGen::new(config);
        generator.convert(&input_dir, &output_dir)?;

        // Should flatten directory structure
        assert!(output_dir.join("src_models_user.ts").exists());
        assert!(output_dir.join("src_models_post.ts").exists());
    }

    Ok(())
}
