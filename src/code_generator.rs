use super::dependency_resolver::ModuleTypes;
use super::type_parser::{EnumInfo, EnumVariant, FieldInfo, StructInfo, TypeKind};
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

    pub fn generate_module(&self, module_types: &ModuleTypes) -> Result<String> {
        let mut output = String::new();

        // Add imports
        output.push_str("import { b } from '@zorsh/zorsh';\n");

        // Add imports from other modules
        for dep in &module_types.dependencies {
            if dep != &module_types.module_path {
                output.push_str(&format!(
                    "import {{ {} }} from './{}';\n",
                    self.get_module_exports(dep),
                    dep.replace("::", "/")
                ));
            }
        }
        output.push('\n');

        // Generate type definitions in dependency order
        for type_path in &module_types.type_paths {
            if let Some(struct_info) = self.structs.get(type_path) {
                output.push_str(&self.generate_struct(struct_info));
                output.push_str("\n\n");
            } else if let Some(enum_info) = self.enums.get(type_path) {
                output.push_str(&self.generate_enum(enum_info));
                output.push_str("\n\n");
            }
        }

        // Add exports
        output.push_str("export {\n");
        for type_path in &module_types.type_paths {
            if let Some(struct_info) = self.structs.get(type_path) {
                output.push_str(&format!("    {}Schema,\n", struct_info.name));
            } else if let Some(enum_info) = self.enums.get(type_path) {
                output.push_str(&format!("    {}Schema,\n", enum_info.name));
            }
        }
        output.push_str("};\n");

        Ok(output)
    }

    fn get_module_exports(&self, module_path: &str) -> String {
        let mut exports = Vec::new();

        for (path, struct_info) in &self.structs {
            if struct_info.module_path == module_path {
                exports.push(format!("{}Schema", struct_info.name));
            }
        }

        for (path, enum_info) in &self.enums {
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
