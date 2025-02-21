use super::type_parser::{EnumInfo, StructInfo, TypeKind};
use anyhow::Result;
use petgraph::prelude::*;
use petgraph::{algo::toposort, Graph};
use std::collections::{HashMap, HashSet};

pub struct DependencyResolver {
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
}

#[derive(Debug)]
pub struct ModuleTypes {
    pub module_path: String,
    pub type_paths: Vec<String>,
    pub dependencies: HashSet<String>, // Other modules this one depends on
}

impl DependencyResolver {
    pub fn new(structs: HashMap<String, StructInfo>, enums: HashMap<String, EnumInfo>) -> Self {
        Self { structs, enums }
    }

    pub fn resolve(&self) -> Result<Vec<ModuleTypes>> {
        let mut graph = Graph::<String, ()>::new();
        let mut node_indices = HashMap::new();

        // Create nodes for all types
        for (path, _) in &self.structs {
            let idx = graph.add_node(path.clone());
            node_indices.insert(path.clone(), idx);
        }
        for (path, _) in &self.enums {
            let idx = graph.add_node(path.clone());
            node_indices.insert(path.clone(), idx);
        }

        // Add edges for dependencies
        self.add_struct_dependencies(&mut graph, &node_indices);
        self.add_enum_dependencies(&mut graph, &node_indices);

        // Perform topological sort with proper error handling
        let sorted = toposort(&graph, None)
            .map_err(|e| anyhow::anyhow!("Dependency cycle detected: {:?}", e))?;

        // Group by module and collect dependencies
        let mut module_groups: HashMap<String, ModuleTypes> = HashMap::new();

        for idx in sorted {
            let type_path = graph[idx].clone();
            let (module_path, dependencies) =
                if let Some(struct_info) = self.structs.get(&type_path) {
                    (
                        struct_info.module_path.clone(),
                        self.get_struct_dependencies(&type_path),
                    )
                } else if let Some(enum_info) = self.enums.get(&type_path) {
                    (
                        enum_info.module_path.clone(),
                        self.get_enum_dependencies(&type_path),
                    )
                } else {
                    continue;
                };

            module_groups
                .entry(module_path.clone())
                .or_insert_with(|| ModuleTypes {
                    module_path: module_path.clone(),
                    type_paths: Vec::new(),
                    dependencies: HashSet::new(),
                })
                .type_paths
                .push(type_path);

            // Add dependencies
            if let Some(module_types) = module_groups.get_mut(&module_path) {
                module_types.dependencies.extend(dependencies);
            }
        }

        Ok(module_groups.into_values().collect())
    }

    fn add_struct_dependencies(
        &self,
        graph: &mut Graph<String, ()>,
        node_indices: &HashMap<String, NodeIndex>,
    ) {
        for (path, struct_info) in &self.structs {
            let from_idx = node_indices[path];

            for field in &struct_info.fields {
                self.add_type_dependencies(&field.type_kind, from_idx, graph, node_indices);
            }
        }
    }

    fn add_enum_dependencies(
        &self,
        graph: &mut Graph<String, ()>,
        node_indices: &HashMap<String, NodeIndex>,
    ) {
        for (path, enum_info) in &self.enums {
            let from_idx = node_indices[path];

            for variant in &enum_info.variants {
                if let Some(fields) = &variant.fields {
                    for field in fields {
                        self.add_type_dependencies(&field.type_kind, from_idx, graph, node_indices);
                    }
                }
            }
        }
    }

    fn add_type_dependencies(
        &self,
        type_kind: &TypeKind,
        from_idx: NodeIndex,
        graph: &mut Graph<String, ()>,
        node_indices: &HashMap<String, NodeIndex>,
    ) {
        match type_kind {
            TypeKind::Struct(_, path) | TypeKind::Enum(_, path) => {
                if let Some(&to_idx) = node_indices.get(path) {
                    graph.add_edge(from_idx, to_idx, ());
                }
            }
            TypeKind::Vec(inner) | TypeKind::Option(inner) | TypeKind::Array(inner, _) => {
                self.add_type_dependencies(inner, from_idx, graph, node_indices);
            }
            TypeKind::HashMap(key, value) => {
                self.add_type_dependencies(key, from_idx, graph, node_indices);
                self.add_type_dependencies(value, from_idx, graph, node_indices);
            }
            _ => {}
        }
    }

    fn get_struct_dependencies(&self, path: &str) -> HashSet<String> {
        let mut deps = HashSet::new();
        if let Some(struct_info) = self.structs.get(path) {
            for field in &struct_info.fields {
                self.collect_type_dependencies(&field.type_kind, &mut deps);
            }
        }
        deps
    }

    fn get_enum_dependencies(&self, path: &str) -> HashSet<String> {
        let mut deps = HashSet::new();
        if let Some(enum_info) = self.enums.get(path) {
            for variant in &enum_info.variants {
                if let Some(fields) = &variant.fields {
                    for field in fields {
                        self.collect_type_dependencies(&field.type_kind, &mut deps);
                    }
                }
            }
        }
        deps
    }

    fn collect_type_dependencies(&self, type_kind: &TypeKind, deps: &mut HashSet<String>) {
        match type_kind {
            TypeKind::Struct(_, path) | TypeKind::Enum(_, path) => {
                if let Some(struct_info) = self.structs.get(path) {
                    deps.insert(struct_info.module_path.clone());
                } else if let Some(enum_info) = self.enums.get(path) {
                    deps.insert(enum_info.module_path.clone());
                }
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
}
