use super::*;
use anyhow::Result;

#[test]
fn test_multiple_files_same_module() -> Result<()> {
    let temp_dir = setup_test_dir();

    let files = vec![
        (
            "src/models/mod.rs",
            r#"
            pub mod types;
            pub mod data;
        "#,
        ),
        (
            "src/models/types.rs",
            r#"
            #[derive(BorshSerialize)]
            pub struct Type1 {
                field: String,
            }
        "#,
        ),
        (
            "src/models/data.rs",
            r#"
            #[derive(BorshSerialize)]
            pub struct Type2 {
                field: String,
            }
        "#,
        ),
    ];

    let input_dir = setup_test_files(&temp_dir, &files);
    let output_dir = temp_dir.path().join("generated");

    let generator = ZorshGen::new(Config::default());
    let result = generator.convert(&input_dir, &output_dir);

    // Both types should be in their respective module files
    assert!(output_dir.join("src/models/types.ts").exists());
    assert!(output_dir.join("src/models/data.ts").exists());

    // Print any error details
    if let Err(ref e) = result {
        println!("Conversion error: {:?}", e);
    }

    // Keep temp_dir in scope until here
    result?;

    Ok(())
}

#[test]
fn test_visibility_and_derives() -> Result<()> {
    let temp_dir = setup_test_dir();

    let files = vec![(
        "src/lib.rs",
        r#"
            // Should be included (has BorshSerialize)
            #[derive(BorshSerialize)]
            pub struct Public {
                field: String,
            }

            // Should be ignored (private and no BorshSerialize)
            struct Private {
                field: String,
            }

            // Should be ignored (no BorshSerialize)
            pub struct NoDerive {
                field: String,
            }
        "#,
    )];

    let input_dir = setup_test_files(&temp_dir, &files);
    let output_dir = temp_dir.path().join("generated");

    let generator = ZorshGen::new(Config::default());
    generator.convert(&input_dir, &output_dir)?;

    let content = fs::read_to_string(output_dir.join("src/lib.ts"))?;

    assert!(content.contains("PublicSchema"));
    assert!(!content.contains("PrivateSchema"));
    assert!(!content.contains("NoDeriveSchema"));

    Ok(())
}

#[test]
fn test_module_imports() -> Result<()> {
    let temp_dir = setup_test_dir();

    let files = vec![
        (
            "src/a.rs",
            r#"
            #[derive(BorshSerialize)]
            pub struct A {
                b_field: super::b::B,
                c_field: super::c::C,
            }
        "#,
        ),
        (
            "src/b.rs",
            r#"
            #[derive(BorshSerialize)]
            pub struct B {
                field: String,
            }
        "#,
        ),
        (
            "src/c.rs",
            r#"
            #[derive(BorshSerialize)]
            pub struct C {
                field: String,
            }
        "#,
        ),
    ];

    let input_dir = setup_test_files(&temp_dir, &files);
    let output_dir = temp_dir.path().join("generated");

    let generator = ZorshGen::new(Config::default());
    generator.convert(&input_dir, &output_dir)?;

    let a_content = fs::read_to_string(output_dir.join("src/a.ts"))?;
    println!("{}", a_content);

    assert!(a_content.contains("import { BSchema } from './b'"));
    assert!(a_content.contains("import { CSchema } from './c'"));

    Ok(())
}
