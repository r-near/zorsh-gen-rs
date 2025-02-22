use anyhow::Result;

#[test]
fn test_primitive_types() -> Result<()> {
    let input = r#"
        #[derive(BorshSerialize)]
        struct Numbers {
            u8_val: u8,
            u16_val: u16,
            u32_val: u32,
            u64_val: u64,
            i8_val: i8,
            i16_val: i16,
            i32_val: i32,
            i64_val: i64,
            f32_val: f32,
            f64_val: f64,
        }
    "#;

    let output = zorsh_gen_rs::convert_str(input)?;
    insta::assert_snapshot!(output);
    Ok(())
}

#[test]
fn test_string_types() -> Result<()> {
    let input = r#"
        #[derive(BorshSerialize)]
        struct User {
            name: String,
            email: String,
        }
    "#;

    let output = zorsh_gen_rs::convert_str(input)?;
    insta::assert_snapshot!(output);
    Ok(())
}

#[test]
fn test_unit_structs() -> Result<()> {
    let input = r#"
        #[derive(BorshSerialize)]
        struct Empty;

        #[derive(BorshSerialize)]
        struct EmptyBraces {}
    "#;

    let output = zorsh_gen_rs::convert_str(input)?;
    insta::assert_snapshot!(output);
    Ok(())
}

#[test]
fn test_basic_enums() -> Result<()> {
    let input = r#"
        #[derive(BorshSerialize)]
        enum Status {
            Active,
            Inactive,
            Pending,
        }
    "#;

    let output = zorsh_gen_rs::convert_str(input)?;
    insta::assert_snapshot!(output);
    Ok(())
}
