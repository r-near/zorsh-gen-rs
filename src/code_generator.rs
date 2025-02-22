use super::dependency_resolver::TypeDependencies;
use super::type_parser::{EnumInfo, StructInfo, TypeKind};
use anyhow::Result;
use std::collections::HashMap;

pub struct ZorshGenerator {
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
}

impl ZorshGenerator {
    pub fn new(structs: HashMap<String, StructInfo>, enums: HashMap<String, EnumInfo>) -> Self {
        Self { structs, enums }
    }
    pub fn generate_module(
        &self,
        current_module: &str,
        dependencies: &TypeDependencies,
    ) -> Result<String> {
        let mut output = String::new();

        // Add base import
        output.push_str("import { b } from '@zorsh/zorsh';\n");

        // Add imports from other modules
        for (module_path, type_names) in &dependencies.module_imports {
            if module_path != current_module {
                let schema_names: Vec<_> = type_names
                    .iter()
                    .map(|name| format!("{}Schema", name))
                    .collect();

                output.push_str(&format!(
                    "import {{ {} }} from './{}';\n",
                    schema_names.join(", "),
                    module_path.replace("::", "/").to_lowercase()
                ));
            }
        }
        output.push('\n');

        // Generate type definitions in dependency order
        for type_path in &dependencies.ordered_types {
            // Only generate types that belong to the current module
            let type_module = self.get_type_module(type_path);
            if type_module == current_module {
                if let Some(struct_info) = self.structs.get(type_path) {
                    output.push_str(&self.generate_struct(struct_info));
                    output.push_str("\n\n");
                } else if let Some(enum_info) = self.enums.get(type_path) {
                    output.push_str(&self.generate_enum(enum_info));
                    output.push_str("\n\n");
                }
            }
        }

        // Add exports
        let exports: Vec<_> = dependencies
            .ordered_types
            .iter()
            .filter(|type_path| self.get_type_module(type_path) == current_module)
            .map(|type_path| {
                let name = type_path.split("::").last().unwrap();
                format!("    {}Schema", name)
            })
            .collect();

        if !exports.is_empty() {
            output.push_str("export {\n");
            output.push_str(&exports.join(",\n"));
            output.push_str("\n};\n");
        }

        Ok(output)
    }

    fn get_type_module(&self, type_path: &str) -> String {
        if let Some(struct_info) = self.structs.get(type_path) {
            struct_info.module_path.to_string()
        } else if let Some(enum_info) = self.enums.get(type_path) {
            enum_info.module_path.to_string()
        } else {
            // Fallback to everything before the last "::"
            type_path
                .rsplit_once("::")
                .map(|(m, _)| m.to_string())
                .unwrap_or_else(|| type_path.to_string())
        }
    }

    fn get_module_exports(&self, module_path: &str) -> String {
        let mut exports = Vec::new();

        for (_path, struct_info) in &self.structs {
            if struct_info.module_path == module_path {
                exports.push(format!("{}Schema", struct_info.name));
            }
        }

        for (_path, enum_info) in &self.enums {
            if enum_info.module_path == module_path {
                exports.push(format!("{}Schema", enum_info.name));
            }
        }

        exports.join(", ")
    }

    fn generate_struct(&self, struct_info: &StructInfo) -> String {
        let mut fields = Vec::new();

        for field in &struct_info.fields {
            fields.push(format!(
                "    {}: {}",
                field.name,
                self.type_to_zorsh(&field.type_kind)
            ));
        }

        format!(
            "export const {}Schema = b.struct({{\n{}\n}});",
            struct_info.name,
            fields.join(",\n")
        )
    }

    fn generate_enum(&self, enum_info: &EnumInfo) -> String {
        let mut variants = Vec::new();

        for variant in &enum_info.variants {
            let variant_schema = match &variant.fields {
                None => "b.unit()".to_string(),
                Some(fields) if fields.len() == 1 && fields[0].name.is_empty() => {
                    // Tuple variant with single field
                    self.type_to_zorsh(&fields[0].type_kind)
                }
                Some(fields) => {
                    // Struct variant
                    let mut struct_fields = Vec::new();
                    for field in fields {
                        struct_fields.push(format!(
                            "        {}: {}",
                            field.name,
                            self.type_to_zorsh(&field.type_kind)
                        ));
                    }
                    format!("b.struct({{\n{}\n    }})", struct_fields.join(",\n"))
                }
            };

            variants.push(format!("    {}: {}", variant.name, variant_schema));
        }

        format!(
            "export const {}Schema = b.enum({{\n{}\n}});",
            enum_info.name,
            variants.join(",\n")
        )
    }

    fn type_to_zorsh(&self, type_kind: &TypeKind) -> String {
        match type_kind {
            TypeKind::Primitive(name) => format!("b.{}()", name),
            TypeKind::String => "b.string()".to_string(),
            TypeKind::Struct(name, _) => format!("{}Schema", name),
            TypeKind::Enum(name, _) => format!("{}Schema", name),
            TypeKind::Vec(inner) => format!("b.vec({})", self.type_to_zorsh(inner)),
            TypeKind::HashMap(key, value) => format!(
                "b.hashMap({}, {})",
                self.type_to_zorsh(key),
                self.type_to_zorsh(value)
            ),
            TypeKind::Option(inner) => format!("b.option({})", self.type_to_zorsh(inner)),
            TypeKind::Array(inner, size) => {
                format!("b.array({}, {})", self.type_to_zorsh(inner), size)
            }
        }
    }
}
