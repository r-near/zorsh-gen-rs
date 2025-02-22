# Welcome to zorsh-gen-rs! 🦀 → 📜

## Introduction for Contributors

Welcome! We're excited to have you contribute to zorsh-gen-rs. This document will help you understand how the codebase works, both at a high level and in technical detail. Whether you're fixing bugs, adding features, or just exploring, this guide will help you get oriented.

## What Does zorsh-gen-rs Do?

At its core, zorsh-gen-rs is a bridge between Rust and TypeScript. It takes Rust code that uses Borsh serialization and automatically generates equivalent TypeScript schemas using the Zorsh library. This means developers can write their data structures once in Rust and automatically get type-safe TypeScript code.

For example, when a developer writes this Rust code:

```rust
#[derive(BorshSerialize)]
struct Player {
    name: String,
    score: u32,
    inventory: Vec<Item>,
}
```

zorsh-gen-rs automatically generates this TypeScript:

```typescript
export const PlayerSchema = b.struct({
    name: b.string(),
    score: b.u32(),
    inventory: b.vec(ItemSchema)
});
export type Player = b.infer<typeof PlayerSchema>;
```

## Technical Deep Dive

### 1. The Processing Pipeline

zorsh-gen-rs processes code in four main phases. Let's explore each in detail:

#### Discovery Phase (source_loader.rs)
```rust
pub struct SourceLoader {
    root_path: PathBuf,
    ignored_patterns: Vec<String>,
}
```

The SourceLoader is your first point of contact with the codebase. It:
1. Walks through the directory tree looking for Rust files
2. Maintains the module hierarchy (which is crucial for proper TypeScript module generation)
3. Handles pattern-based exclusions (like ignoring test files)

Key methods to understand:
```rust
// Main entry point for file discovery
pub fn discover_rust_files(&self) -> Result<Vec<SourceFile>>

// Determines module path from file location
fn calculate_module_path(root: &Path, file_path: &Path) -> Result<String>
```

The module path calculation is particularly important as it affects how imports are generated in the final TypeScript code.

#### Parsing Phase (type_parser.rs)

The parser uses the `syn` crate to understand Rust code. Here's where it gets interesting:

```rust
pub struct TypeParser {
    module_path: String,
    only_annotated: bool,
    pub structs: HashMap<String, StructInfo>,
    pub enums: HashMap<String, EnumInfo>,
}
```

The parser implements the `Visit` trait from `syn` to walk the AST (Abstract Syntax Tree). Here's what it looks for:

1. **Struct Definitions**:
```rust
impl<'ast> Visit<'ast> for TypeParser {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        // Only process if it has BorshSerialize when only_annotated is true
        if !self.should_process_item(&node.attrs) {
            return;
        }
        // Process the struct...
    }
}
```

2. **Type Resolution**:
```rust
fn parse_type(&self, ty: &Type) -> TypeKind {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            // Handle different type patterns
            match type_name.as_str() {
                "String" => TypeKind::String,
                "Vec" => {
                    // Extract generic type parameter
                    TypeKind::Vec(Box::new(self.parse_type(inner_type)))
                }
                // ... other types
            }
        }
        // ... other patterns
    }
}
```

#### Type System

The type system is the heart of zorsh-gen-rs. It needs to handle everything from simple primitives to complex nested generic types:

```rust
pub enum TypeKind {
    Primitive(String),        // e.g., u8, i32
    String,                   // Rust's String type
    Struct(String, String),   // (name, full_path)
    Enum(String, String),     // (name, full_path)
    Vec(Box<TypeKind>),      // Vec<T>
    HashMap(Box<TypeKind>, Box<TypeKind>), // HashMap<K,V>
    Option(Box<TypeKind>),   // Option<T>
    Array(Box<TypeKind>, usize), // [T; N]
}
```

Each variant handles different aspects of Rust's type system:
- `Primitive`: Maps directly to Zorsh primitives
- `Struct/Enum`: Handles custom types, maintaining their module paths
- `Vec/HashMap/Option`: Deals with generic types
- `Array`: Handles fixed-size arrays

#### Resolution Phase (dependency_resolver.rs)

The dependency resolver is crucial for handling complex type relationships:

```rust
pub struct DependencyResolver {
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
}
```

It uses `petgraph` to build a directed acyclic graph (DAG) of type dependencies:

```rust
pub fn resolve(&self) -> Result<TypeDependencies> {
    // Build dependency graph
    let mut graph = Graph::<String, ()>::new();
    
    // Add nodes for all types
    for path in self.structs.keys().chain(self.enums.keys()) {
        let idx = graph.add_node(path.clone());
        node_indices.insert(path.clone(), idx);
    }
    
    // Add edges for dependencies
    // ...
    
    // Perform topological sort
    let sorted = toposort(&graph, None)?;
}
```

This ensures that:
1. Types are generated in the correct order
2. Circular dependencies are detected
3. Module imports are properly tracked

#### Generation Phase (code_generator.rs)

The code generator produces clean, well-formatted TypeScript code:

```rust
pub struct ZorshGenerator {
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
}
```

Key generation methods:

```rust
// Generates a complete module
pub fn generate_module(
    &self,
    current_module: &str,
    dependencies: &TypeDependencies,
) -> Result<String>

// Generates struct schemas
fn generate_struct(&self, struct_info: &StructInfo) -> String

// Generates enum schemas
fn generate_enum(&self, enum_info: &EnumInfo) -> String
```

The generator handles:
1. Import statements
2. Type declarations
3. Nested type references
4. Module organization

### Important Implementation Details

#### Error Handling Strategy

We use `anyhow` for error handling, which provides rich context:

```rust
pub fn convert(&self) -> Result<()> {
    let source_files = self.source_loader.discover_rust_files()
        .context("Failed to discover Rust files")?;
        
    // More operations...
}
```

#### Module Resolution

Module resolution is complex because it needs to:
1. Handle both `mod.rs` and directory/file modules
2. Maintain proper import paths
3. Support nested modules

Here's how it works:
```rust
fn calculate_module_path(root: &Path, file_path: &Path) -> Result<String> {
    let rel_path = file_path.strip_prefix(root)?;
    let mut module_parts: Vec<String> = rel_path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
        
    // Special handling for mod.rs
    if let Some(last) = module_parts.last_mut() {
        *last = last.trim_end_matches(".rs").to_string();
    }
    module_parts.retain(|part| part != "mod");
    
    Ok(module_parts.join("::"))
}
```

### Working with the Code

#### Adding a New Type Mapping

Let's say you want to add support for a new Rust type. Here's the process:

1. Add the type to `TypeKind`:
```rust
pub enum TypeKind {
    // ... existing types ...
    NewType(Box<TypeKind>),
}
```

2. Add parsing support:
```rust
fn parse_type(&self, ty: &Type) -> TypeKind {
    match ty {
        // ... existing matches ...
        Type::Path(TypePath { path, .. }) => {
            if is_new_type(path) {
                TypeKind::NewType(Box::new(self.parse_inner_type(ty)))
            }
        }
    }
}
```

3. Add generation support:
```rust
fn type_to_zorsh(&self, type_kind: &TypeKind) -> String {
    match type_kind {
        // ... existing matches ...
        TypeKind::NewType(inner) => {
            format!("b.newType({})", self.type_to_zorsh(inner))
        }
    }
}
```

4. Add tests:
```rust
#[test]
fn test_new_type_conversion() {
    let input = r#"
        #[derive(BorshSerialize)]
        struct Test {
            field: NewType<String>,
        }
    "#;
    
    let output = convert_str(input)?;
    assert!(output.contains("b.newType("));
}
```

### Common Development Tasks

#### Running Tests

The project uses several test types:

```bash
# Run all tests
cargo test

# Run specific test categories
cargo test basic_types
cargo test complex_types

# Update snapshots
cargo test -- --accept
```

#### Debugging Tips

1. Enable debug logging:
```rust
println!("Module path: {}", module_path);
println!("Parsed type: {:?}", type_kind);
```

2. Use the `syn` parser directly:
```rust
let syntax: File = syn::parse_str(input)?;
println!("AST: {:#?}", syntax);
```

3. Check generated code:
```rust
let code = generator.generate_module(&module, &deps)?;
println!("Generated code:\n{}", code);
```

### Best Practices

1. **Type Safety**
   - Always use `Result` for fallible operations
   - Provide meaningful error contexts
   - Handle edge cases explicitly

2. **Code Generation**
   - Generate clean, formatted code
   - Maintain consistent spacing
   - Use meaningful variable names
   - Add type annotations

3. **Testing**
   - Write tests for new features
   - Update snapshots when needed
   - Test edge cases
   - Use integration tests for complex scenarios

## Getting Help

If you're stuck or have questions:
1. Check the test files for examples
2. Look at the snapshots for expected output
3. Ask in GitHub issues
4. Add debug prints to understand the flow

Happy coding! 🚀