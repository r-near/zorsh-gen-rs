use anyhow::Result;
use log::debug;
use quote::ToTokens;
use std::collections::HashMap;
use syn::{
    visit::{self, Visit},
    Fields, File, GenericArgument, ItemEnum, ItemStruct, PathArguments, Type, TypePath,
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
    only_annotated: bool,
    pub structs: HashMap<String, StructInfo>,
    pub enums: HashMap<String, EnumInfo>,
    type_aliases: HashMap<String, Type>,
}

impl TypeParser {
    pub fn new(module_path: String, only_annotated: bool) -> Self {
        Self {
            module_path,
            only_annotated,
            structs: HashMap::new(),
            enums: HashMap::new(),
            type_aliases: HashMap::new(),
        }
    }

    fn should_process_item(&self, attrs: &[syn::Attribute]) -> bool {
        !self.only_annotated || has_borsh_derive(attrs)
    }

    pub fn parse_file(&mut self, content: &str) -> Result<()> {
        let syntax: File = syn::parse_str(content)?;

        // First pass: collect all type aliases
        self.collect_type_aliases(&syntax);

        // Second pass: process structs and enums
        self.visit_file(&syntax);

        Ok(())
    }

    fn collect_type_aliases(&mut self, file: &File) {
        use syn::Item;

        for item in &file.items {
            if let Item::Type(type_item) = item {
                let type_name = type_item.ident.to_string();
                // Store the original type for later resolution
                self.type_aliases.insert(type_name, *type_item.ty.clone());
            }
        }
    }

    fn parse_type(&self, ty: &Type) -> TypeKind {
        debug!("Parsing type: {}", ty.to_token_stream());

        match ty {
            Type::Path(TypePath { path, .. }) => {
                if let Some(segment) = path.segments.last() {
                    let type_name = segment.ident.to_string();

                    // Debug: Print when we're checking for an alias
                    debug!("  Checking for alias: {}", type_name);

                    // Try to resolve type alias before other type matching
                    if let Some(aliased_type) = self.type_aliases.get(&type_name) {
                        debug!(
                            "  Found alias! Resolving to: {:?}",
                            aliased_type.to_token_stream()
                        );
                        return self.parse_type(aliased_type);
                    }

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
                            // If path has multiple segments, it's a cross-module reference
                            let module_path = if path.segments.len() > 1 {
                                let mut segments: Vec<_> = path
                                    .segments
                                    .iter()
                                    .take(path.segments.len() - 1)
                                    .map(|s| s.ident.to_string())
                                    .collect();
                                // Handle super:: by using the module directly
                                if segments[0] == "super" {
                                    segments.remove(0);
                                }
                                segments.join("::")
                            } else {
                                self.module_path.clone()
                            };

                            let full_path = if module_path.is_empty() {
                                type_name.clone()
                            } else {
                                format!("{}::{}", module_path, type_name)
                            };

                            if self.structs.contains_key(&full_path) {
                                TypeKind::Struct(type_name.clone(), full_path)
                            } else if self.enums.contains_key(&full_path) {
                                TypeKind::Enum(type_name.clone(), full_path)
                            } else {
                                // Even for unknown types, use the full module path
                                TypeKind::Struct(type_name.clone(), full_path)
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
        // Only process if it matches our annotation requirements
        if !self.should_process_item(&node.attrs) {
            return;
        }

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
        // Only process if it matches our annotation requirements
        if !self.should_process_item(&node.attrs) {
            return;
        }

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

// Helper function to check for Borsh derives
fn has_borsh_derive(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("derive") {
            return false;
        }

        match attr.meta {
            syn::Meta::List(ref list) => {
                list.tokens.to_string().contains("BorshSerialize")
                    || list.tokens.to_string().contains("BorshDeserialize")
            }
            _ => false,
        }
    })
}
