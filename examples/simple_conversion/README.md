# Simple Conversion Example

This example demonstrates how to use the zorsh-gen library to convert Rust types to Zorsh TypeScript schemas.

## Structure

```
simple_conversion/
├── input/
│   └── models.rs        # Example Rust models with Borsh serialization
├── main.rs             # Example usage of the zorsh-gen library
└── README.md           # This file
```

## Running the Example

From the root of the zorsh-gen project:

```bash
cargo run --example simple_conversion
```

This will:
1. Read the Rust models from `input/models.rs`
2. Generate Zorsh TypeScript schemas in the `output/` directory
3. Show an example of direct string conversion

## Generated Output

The example will generate files in `output/` with this structure:

```
output/
└── models/
    └── index.ts       # Generated Zorsh TypeScript schemas
```

The generated TypeScript file will contain Zorsh schemas for:
- Item
- Inventory
- PlayerStatus (enum)
- Player
- GameState

Each type will be properly mapped to its Zorsh equivalent, with all dependencies handled correctly.

## Examining the Output

After running the example, you can check the generated TypeScript file to see how Rust types are mapped to Zorsh schemas. The output will preserve:

- Type relationships and dependencies
- Enum variants (unit, tuple, and struct variants)
- Complex types (Vec, HashMap, Option)
- Nested structures

## Note on Borsh Compatibility

This example demonstrates the conversion of Rust types that use Borsh serialization. The generated Zorsh schemas will be compatible with the binary format produced by Borsh, ensuring seamless interoperability between Rust and TypeScript/JavaScript applications.