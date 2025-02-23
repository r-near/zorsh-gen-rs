use super::type_parser::{EnumInfo, StructInfo, TypeKind};
use anyhow::{anyhow, Result};
use log::debug;
use petgraph::algo::toposort;
use petgraph::prelude::*;
use std::collections::{HashMap, HashSet};

/// Represents all type dependencies across modules
#[derive(Debug)]
pub struct TypeDependencies {
    /// List of all types in dependency order
    pub ordered_types: Vec<String>,
    /// Map of module path -> set of types that need to be imported from it
    pub module_imports: HashMap<String, HashSet<String>>,
}

pub struct DependencyResolver {
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
}

impl DependencyResolver {
    pub fn new(structs: HashMap<String, StructInfo>, enums: HashMap<String, EnumInfo>) -> Self {
        Self { structs, enums }
    }

    pub fn resolve(&self) -> Result<TypeDependencies> {
        // Build dependency graph
        let mut graph = Graph::<String, ()>::new();
        let mut node_indices = HashMap::new();

        // Create nodes for all types
        for path in self.structs.keys().chain(self.enums.keys()) {
            let idx = graph.add_node(path.clone());
            node_indices.insert(path.clone(), idx);
        }

        // Add edges for all dependencies
        for (path, idx) in &node_indices {
            if let Some(deps) = self.get_type_dependencies(path) {
                for dep_path in deps {
                    if let Some(&dep_idx) = node_indices.get(&dep_path) {
                        // Add edge from dependency to dependent (reverse for topo sort)
                        graph.add_edge(dep_idx, *idx, ());
                    }
                }
            }
        }

        // Perform topological sort
        let sorted =
            toposort(&graph, None).map_err(|e| anyhow!("Dependency cycle detected: {:?}", e))?;

        // Convert node indices back to type paths in order
        let ordered_types = sorted
            .iter()
            .map(|&idx| graph[idx].clone())
            .collect::<Vec<_>>();

        // Collect required imports between modules
        let mut module_imports: HashMap<String, HashSet<String>> = HashMap::new();

        for type_path in &ordered_types {
            let current_module = self.get_module_path(type_path);

            // Get external dependencies for this type
            if let Some(deps) = self.get_type_dependencies(type_path) {
                for dep_path in deps {
                    let dep_module = self.get_module_path(&dep_path);

                    // Only add import if it's from a different module
                    if dep_module != current_module {
                        let type_name = dep_path
                            .split("::")
                            .last()
                            .ok_or_else(|| anyhow!("Invalid type path: {}", dep_path))?;

                        module_imports
                            .entry(dep_module)
                            .or_default()
                            .insert(type_name.to_string());
                    }
                }
            }
        }

        Ok(TypeDependencies {
            ordered_types,
            module_imports,
        })
    }

    fn get_type_dependencies(&self, type_path: &str) -> Option<HashSet<String>> {
        let mut deps = HashSet::new();

        debug!("Getting dependencies for type: {}", type_path);

        // Check structs
        if let Some(struct_info) = self.structs.get(type_path) {
            for field in &struct_info.fields {
                debug!("  Field type: {:?}", field.type_kind);
                self.collect_type_dependencies(&field.type_kind, &mut deps);
            }
            return Some(deps);
        }

        // Check enums
        if let Some(enum_info) = self.enums.get(type_path) {
            for variant in &enum_info.variants {
                if let Some(fields) = &variant.fields {
                    for field in fields {
                        debug!("  Variant field type: {:?}", field.type_kind);
                        self.collect_type_dependencies(&field.type_kind, &mut deps);
                    }
                }
            }
            return Some(deps);
        }

        None
    }

    #[allow(clippy::only_used_in_recursion)] // Parameter is essential for recursive calls
    fn collect_type_dependencies(&self, type_kind: &TypeKind, deps: &mut HashSet<String>) {
        match type_kind {
            TypeKind::Struct(name, path) | TypeKind::Enum(name, path) => {
                debug!("  Adding dependency: {} ({})", name, path);
                deps.insert(path.clone());
            }
            TypeKind::Vec(inner) | TypeKind::Option(inner) | TypeKind::Array(inner, _) => {
                self.collect_type_dependencies(inner, deps);
            }
            TypeKind::HashMap(key, value) => {
                self.collect_type_dependencies(key, deps);
                self.collect_type_dependencies(value, deps);
            }
            _ => {}
        }
    }

    fn get_module_path(&self, type_path: &str) -> String {
        // Get module path from either structs or enums
        if let Some(struct_info) = self.structs.get(type_path) {
            struct_info.module_path.clone()
        } else if let Some(enum_info) = self.enums.get(type_path) {
            enum_info.module_path.clone()
        } else {
            // If type not found, assume module path is everything before the last segment
            type_path
                .rsplit_once("::")
                .map(|(m, _)| m.to_string())
                .unwrap_or_else(|| type_path.to_string())
        }
    }
}
