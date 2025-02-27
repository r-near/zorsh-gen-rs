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
# Using npx (Node.js)
npx @zorsh/cli ./src/models ./generated

# Or using pnpm
pnpm dlx @zorsh/cli ./src/models ./generated
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
import { b } from "@zorsh/zorsh";

export const PlayerSchema = b.struct({
  name: b.string(),
  score: b.u32(),
  inventory: b.hashMap(b.string(), b.vec(ItemSchema)),
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

Choose the installation method that works best for your workflow:

### Node.js / npm (Recommended for TypeScript projects)

```bash
# Install globally with npm
npm install -g @zorsh/cli

# Install as a dev dependency in your project
npm install --save-dev @zorsh/cli

# Or use without installing
npx @zorsh/cli
pnpm dlx @zorsh/cli
```

### Cargo (Recommended for Rust projects)

```bash
# Install from crates.io
cargo install zorsh-gen-rs

# Or build from source
git clone https://github.com/r-near/zorsh-gen-rs
cd zorsh-gen-rs
cargo install --path .
```

### Script Installers

```bash
# Unix-like systems (Linux, macOS)
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/r-near/zorsh-gen-rs/releases/latest/download/zorsh-gen-rs-installer.sh | sh

# Windows PowerShell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/r-near/zorsh-gen-rs/releases/latest/download/zorsh-gen-rs-installer.ps1 | iex"
```

### Package Managers

```bash
# Homebrew (macOS and Linux)
brew install r-near/tap/zorsh-gen-rs
```

### Manual Installation

Pre-built binaries are available for the following platforms:

- macOS (Apple Silicon, Intel)
- Windows (x64)
- Linux (x64, ARM64)

Download the appropriate binary for your platform from the [releases page](https://github.com/r-near/zorsh-gen-rs/releases).

## Usage

### Command Line

```bash
# Basic usage
zorsh-gen-rs <input-dir> <output-dir>

# With options
zorsh-gen-rs --flat-output --only-annotated ./src/models ./generated

# Debug output
zorsh-gen-rs --verbose ./src/models ./generated  # Show detailed debug information
zorsh-gen-rs --quiet ./src/models ./generated    # Show only errors
```

Options:

- `--verbose, -v`: Show detailed debug information during conversion
- `--quiet, -q`: Show only errors (overrides --verbose)
- `--flat-output`: Output all files in a flat directory structure
- `--only-annotated`: Only process structs with Borsh derive macros (default: true)
- `--ignored-patterns`: Comma-separated patterns to ignore (e.g., "tests/,examples/")

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

Contributions are welcome! Before you start:

- Check out our [Technical Documentation](docs/technical.md) for a deep dive into the codebase
- Feel free to:
  - Report bugs
  - Suggest features
  - Submit pull requests

## Related Projects

- [Zorsh](https://github.com/r-near/zorsh) - The TypeScript implementation of Borsh
- [Borsh](https://borsh.io) - The original Rust binary serialization format

## License

MIT
