use anyhow::Result;

#[test]
fn test_nested_structs() -> Result<()> {
    let input = r#"
        #[derive(BorshSerialize)]
        struct Address {
            street: String,
            city: String,
            country: String,
        }

        #[derive(BorshSerialize)]
        struct User {
            name: String,
            address: Address,
        }
    "#;

    let output = zorsh_gen_rs::convert_str(input)?;
    insta::assert_snapshot!(output);
    Ok(())
}

#[test]
fn test_collections() -> Result<()> {
    let input = r#"
        #[derive(BorshSerialize)]
        struct Item {
            id: u32,
            name: String,
        }

        #[derive(BorshSerialize)]
        struct Inventory {
            items: Vec<Item>,
            counts: HashMap<String, u32>,
            fixed_data: [u8; 32],
        }
    "#;

    let output = zorsh_gen_rs::convert_str(input)?;
    insta::assert_snapshot!(output);
    Ok(())
}

#[test]
fn test_complex_enums() -> Result<()> {
    let input = r#"
        #[derive(BorshSerialize)]
        enum Event {
            Created { timestamp: u64 },
            Updated(String),
            Deleted,
        }

        #[derive(BorshSerialize)]
        struct EventLog {
            events: Vec<Event>,
        }
    "#;

    let output = zorsh_gen_rs::convert_str(input)?;
    insta::assert_snapshot!(output);
    Ok(())
}

#[test]
fn test_optional_and_nested_types() -> Result<()> {
    let input = r#"
        #[derive(BorshSerialize)]
        struct Metadata {
            tags: Vec<String>,
            extra: Option<HashMap<String, Vec<u32>>>,
        }

        #[derive(BorshSerialize)]
        struct Document {
            id: String,
            metadata: Option<Metadata>,
        }
    "#;

    let output = zorsh_gen_rs::convert_str(input)?;
    insta::assert_snapshot!(output);
    Ok(())
}
