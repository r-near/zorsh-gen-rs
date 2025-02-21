use anyhow::Result;
use std::collections::HashMap;
use syn::{
    visit::{self, Visit},
    Fields, File, GenericArgument, Ident, ItemEnum, ItemStruct, PathArguments, Type, TypePath,
};

#[derive(Debug, Clone)]
pub enum TypeKind {
    Primitive(String),
    Struct(String, String), // (name, module_path)
    Enum(String, String),   // (name, module_path)
    Vec(Box<TypeKind>),
    HashMap(Box<TypeKind>, Box<TypeKind>),
    Option(Box<TypeKind>),
    Array(Box<TypeKind>, usize),
    String,
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub module_path: String,
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub type_kind: TypeKind,
}

#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub module_path: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Option<Vec<FieldInfo>>,
}

pub struct TypeParser {
    module_path: String,
    pub structs: HashMap<String, StructInfo>,
    pub enums: HashMap<String, EnumInfo>,
}

impl TypeParser {
    pub fn new(module_path: String) -> Self {
        Self {
            module_path,
            structs: HashMap::new(),
            enums: HashMap::new(),
        }
    }

    pub fn parse_file(&mut self, content: &str) -> Result<()> {
        let syntax: File = syn::parse_str(content)?;
        self.visit_file(&syntax);
        Ok(())
    }

    fn parse_type(&self, ty: &Type) -> TypeKind {
        match ty {
            Type::Path(TypePath { path, .. }) => {
                if let Some(segment) = path.segments.last() {
                    let type_name = segment.ident.to_string();

                    match type_name.as_str() {
                        // Primitive types
                        "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" | "f32"
                        | "f64" => TypeKind::Primitive(type_name),
                        "String" => TypeKind::String,
                        "Vec" => {
                            if let PathArguments::AngleBracketed(args) = &segment.arguments {
                                if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                                    return TypeKind::Vec(Box::new(self.parse_type(inner_type)));
                                }
                            }
                            panic!("Invalid Vec type")
                        }
                        "HashMap" => {
                            if let PathArguments::AngleBracketed(args) = &segment.arguments {
                                let mut types = args.args.iter().filter_map(|arg| {
                                    if let GenericArgument::Type(ty) = arg {
                                        Some(self.parse_type(ty))
                                    } else {
                                        None
                                    }
                                });

                                if let (Some(key_type), Some(value_type)) =
                                    (types.next(), types.next())
                                {
                                    return TypeKind::HashMap(
                                        Box::new(key_type),
                                        Box::new(value_type),
                                    );
                                }
                            }
                            panic!("Invalid HashMap type")
                        }
                        "Option" => {
                            if let PathArguments::AngleBracketed(args) = &segment.arguments {
                                if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                                    return TypeKind::Option(Box::new(self.parse_type(inner_type)));
                                }
                            }
                            panic!("Invalid Option type")
                        }
                        _ => {
                            // Check if it's a known struct or enum
                            let full_path = format!("{}::{}", self.module_path, type_name);
                            if self.structs.contains_key(&full_path) {
                                TypeKind::Struct(type_name.clone(), full_path)
                            } else if self.enums.contains_key(&full_path) {
                                TypeKind::Enum(type_name.clone(), full_path)
                            } else {
                                // Assume it's a struct/enum from another module
                                TypeKind::Struct(type_name.clone(), type_name)
                            }
                        }
                    }
                } else {
                    panic!("Invalid type path")
                }
            }
            Type::Array(array) => {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(size),
                    ..
                }) = &array.len
                {
                    TypeKind::Array(
                        Box::new(self.parse_type(&array.elem)),
                        size.base10_parse().unwrap(),
                    )
                } else {
                    panic!("Invalid array size")
                }
            }
            _ => panic!("Unsupported type"),
        }
    }
}

impl<'ast> Visit<'ast> for TypeParser {
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        let struct_name = node.ident.to_string();
        let full_path = format!("{}::{}", self.module_path, struct_name);
        let mut fields = Vec::new();

        if let Fields::Named(named_fields) = &node.fields {
            for field in &named_fields.named {
                if let Some(ident) = &field.ident {
                    fields.push(FieldInfo {
                        name: ident.to_string(),
                        type_kind: self.parse_type(&field.ty),
                    });
                }
            }
        }

        self.structs.insert(
            full_path.clone(),
            StructInfo {
                name: struct_name,
                module_path: self.module_path.clone(),
                fields,
            },
        );

        visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        let enum_name = node.ident.to_string();
        let full_path = format!("{}::{}", self.module_path, enum_name);
        let mut variants = Vec::new();

        for variant in &node.variants {
            let variant_name = variant.ident.to_string();
            let fields = match &variant.fields {
                Fields::Named(named_fields) => Some(
                    named_fields
                        .named
                        .iter()
                        .map(|field| FieldInfo {
                            name: field.ident.as_ref().unwrap().to_string(),
                            type_kind: self.parse_type(&field.ty),
                        })
                        .collect(),
                ),
                Fields::Unnamed(unnamed_fields) => Some(
                    unnamed_fields
                        .unnamed
                        .iter()
                        .map(|field| FieldInfo {
                            name: String::new(),
                            type_kind: self.parse_type(&field.ty),
                        })
                        .collect(),
                ),
                Fields::Unit => None,
            };

            variants.push(EnumVariant {
                name: variant_name,
                fields,
            });
        }

        self.enums.insert(
            full_path.clone(),
            EnumInfo {
                name: enum_name,
                module_path: self.module_path.clone(),
                variants,
            },
        );

        visit::visit_item_enum(self, node);
    }
}
