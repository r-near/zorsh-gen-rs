// tests/integration_tests/type_aliases.rs
use anyhow::Result;

#[test]
fn test_primitive_aliases() -> Result<()> {
    let input = r#"
        #[derive(BorshSerialize)]
        pub struct Data {
            value: Value,
            count: Count,
        }

        type Value = u64;
        type Count = u32;
    "#;

    let output = zorsh_gen_rs::convert_str(input)?;
    insta::assert_snapshot!(output);
    Ok(())
}

#[test]
fn test_array_aliases() -> Result<()> {
    let input = r#"
        #[derive(BorshSerialize)]
        pub struct Crypto {
            hash: Hash,
            address: Address,
        }

        type Hash = [u8; 32];
        type Address = [u8; 20];
    "#;

    let output = zorsh_gen_rs::convert_str(input)?;
    insta::assert_snapshot!(output);
    Ok(())
}

#[test]
fn test_nested_type_aliases() -> Result<()> {
    let input = r#"
        #[derive(BorshSerialize)]
        pub struct Transaction {
            sender: AccountId,
            receiver: AccountId,
            amount: Balance,
            data: TxData,
        }

        type AccountId = [u8; 32];
        type Balance = u64;
        type TxData = Vec<u8>;
    "#;

    let output = zorsh_gen_rs::convert_str(input)?;
    insta::assert_snapshot!(output);
    Ok(())
}

#[test]
fn test_enum_with_type_aliases() -> Result<()> {
    let input = r#"
        #[derive(BorshSerialize)]
        pub enum Event {
            Transfer {
                from: AccountId,
                to: AccountId,
                amount: Balance,
            },
            Mint {
                to: AccountId,
                token_id: TokenId,
            },
        }

        type AccountId = [u8; 32];
        type Balance = u64;
        type TokenId = u32;
    "#;

    let output = zorsh_gen_rs::convert_str(input)?;
    insta::assert_snapshot!(output);
    Ok(())
}

#[test]
fn test_complex_nested_aliases() -> Result<()> {
    let input = r#"
        #[derive(BorshSerialize)]
        pub struct State {
            accounts: Registry,
            balances: Balances,
            metadata: Metadata,
        }

        type AccountId = [u8; 32];
        type Registry = Vec<AccountId>;
        type Balance = u64;
        type Balances = HashMap<AccountId, Balance>;
        type Metadata = HashMap<String, Vec<u8>>;
    "#;

    let output = zorsh_gen_rs::convert_str(input)?;
    insta::assert_snapshot!(output);
    Ok(())
}
