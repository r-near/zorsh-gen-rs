# zorsh-gen-rs 🦀 → 📜

![Status](https://img.shields.io/badge/Status-Beta-blue)
![Stability](https://img.shields.io/badge/Stability-Pre--Release-yellow)

A code generator that turns Rust's Borsh structs into Zorsh TypeScript schemas, using Rust itself as the schema language.

zorsh-gen-rs takes your existing Rust types and automatically generates [Zorsh](https://github.com/r-near/zorsh) schemas for TypeScript. One command, zero maintenance, complete type safety.

## Features

- 🎯 **Use Rust as Your Schema** - No separate schema language needed. Your Rust types are your source of truth
- 🔄 **Automatic Conversion** - Directly generate Zorsh schemas from your Rust structs and enums
- 🌳 **Preserves Structure** - Maintains your Rust module hierarchy in the generated TypeScript
- 🔍 **Deep Type Analysis** - Handles complex nested types, generics, and cross-module dependencies
- ⚡ **Zero Runtime Overhead** - Follows Borsh's philosophy of high performance serialization
- 🛡️ **Type Safe** - If it compiles in Rust, it works in TypeScript

## Quick Start

```bash
# Install the generator
cargo install zorsh-gen-rs

# Run it on your Rust project
zorsh-gen-rs ./src/models ./generated
```

Your Rust types:
```rust
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Player {
    name: String,
    score: u32,
    inventory: HashMap<String, Vec<Item>>,
}
```

Automatically become Zorsh schemas:
```typescript
import { b } from '@zorsh/zorsh';

export const PlayerSchema = b.struct({
    name: b.string(),
    score: b.u32(),
    inventory: b.hashMap(b.string(), b.vec(ItemSchema))
});
```

## Why zorsh-gen-rs?

Most serialization formats force you to define your data structures twice: once in your schema language (like `.proto` files for Protocol Buffers or SDL for GraphQL), and again in your programming language. With Borsh, your data structures are already defined in Rust - that's where they live. zorsh-gen-rs recognizes this and lets your existing Rust types act as the schema language for [Zorsh](https://github.com/r-near/zorsh), the TypeScript implementation of Borsh.

No duplicate definitions. No schema syncing. No extra maintenance. Just:

1. **No Schema Maintenance** - Your Rust types are your schema. No need to maintain separate schema files
2. **Compiler-Verified** - If your Rust types compile, your schemas are valid
3. **IDE Support** - Get full IDE support, type checking, and refactoring tools for your schemas
4. **Ecosystem Support** - Use the full power of Rust's type system and module system

## Installation

```bash
# From crates.io
cargo install zorsh-gen-rs

# From source
git clone https://github.com/r-near/zorsh-gen-rs
cd zorsh-gen-rs
cargo install --path .
```

## Usage

### Command Line

```bash
# Basic usage
zorsh-gen-rs <input-dir> <output-dir>

# With options
zorsh-gen-rs --flat-output --only-annotated ./src/models ./generated
```

### As a Library

```rust
use zorsh_gen_rs::{ZorshGen, Config};

let config = Config::default();
let generator = ZorshGen::new(config);

// Convert a directory
generator.convert("./src/models", "./generated")?;

// Or convert a string
let zorsh_code = zorsh_gen_rs::convert_str(rust_code)?;
```

## Supported Types

- **Primitives**: All Rust numeric types (`u8` through `u128`, `i8` through `i128`, `f32`, `f64`)
- **Strings**: `String` and `&str`
- **Collections**: `Vec<T>`, `[T; N]`, `HashMap<K, V>`
- **Options**: `Option<T>`
- **Custom Types**: Structs and Enums (including complex nested types)

## Module Structure

zorsh-gen-rs preserves your Rust module structure in the generated TypeScript:

```
src/
  models/
    player.rs
    items/
      mod.rs
      weapon.rs
```

Becomes:

```
generated/
  models/
    player.ts
    items/
      index.ts
      weapon.ts
```

## Configuration

```rust
let config = Config {
    // Only process structs with #[derive(BorshSerialize)]
    only_annotated: true,
    
    // Skip certain paths
    ignored_patterns: vec!["tests/", "examples/"],
    
    // Output structure (nested or flat)
    output_structure: OutputStructure::Nested,
};
```

## Contributing

Contributions are welcome! Feel free to:

- Report bugs
- Suggest features
- Submit pull requests


## Related Projects

- [Zorsh](https://github.com/r-near/zorsh) - The TypeScript implementation of Borsh
- [Borsh](https://borsh.io) - The original Rust binary serialization format

## License

MIT