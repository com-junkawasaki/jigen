//! Scene graph implementation using v(vertex), e(edge), i(incident) model
//!
//! Integrates vertices, edges, and incidents into a cohesive graph structure
//! for declarative 3D scene management with physics simulation.

use super::{vertex::Vertex, edge::Edge, incident::{Incident, IncidentManager}};
use crate::dsl::scene::{SceneDefinition, SceneNode};
use bevy::prelude::Resource;
use petgraph::{Graph, Directed};
use petgraph::graph::{NodeIndex, EdgeIndex};
use std::collections::{HashMap, HashSet};
use nalgebra::Vector3;

/// Scene graph using v-e-i model
#[derive(Debug, Resource)]
pub struct SceneGraph {
    /// Graph structure (vertices and edges)
    pub graph: Graph<Vertex, Edge, Directed>,

    /// Incident manager for events and interactions
    incident_manager: IncidentManager,

    /// Vertex lookup by ID
    vertex_lookup: HashMap<String, NodeIndex>,

    /// Edge lookup by ID pair (source_id, target_id, edge_type)
    edge_lookup: HashMap<(String, String, String), EdgeIndex>,

    /// Root vertices (no parent)
    root_vertices: Vec<NodeIndex>,

    /// Spatial partitioning for efficient queries
    spatial_index: SpatialIndex,

    /// Graph metadata
    metadata: HashMap<String, serde_json::Value>,

    /// Topology version (incremented on changes)
    topology_version: u64,
}

/// Spatial index for efficient proximity queries
#[derive(Debug)]
pub struct SpatialIndex {
    /// Grid cell size
    cell_size: f32,
    /// Grid cells containing vertex indices
    cells: HashMap<(i32, i32, i32), Vec<NodeIndex>>,
}

impl SceneGraph {
    /// Create a new empty scene graph
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            incident_manager: IncidentManager::default(),
            vertex_lookup: HashMap::new(),
            edge_lookup: HashMap::new(),
            root_vertices: Vec::new(),
            spatial_index: SpatialIndex::new(10.0), // 10 unit cells
            metadata: HashMap::new(),
            topology_version: 0,
        }
    }

    /// Load scene from SceneDefinition
    pub fn load_scene(&mut self, scene_def: &SceneDefinition) {
        // Clear existing graph
        self.clear();

        // Add metadata
        self.metadata.insert("title".to_string(), serde_json::Value::String(scene_def.metadata.title.clone()));
        self.metadata.insert("version".to_string(), serde_json::Value::String(scene_def.metadata.version.clone()));

        // Build hierarchy from scene nodes
        self.build_hierarchy(&scene_def.nodes, None);

        // Build spatial relationships
        self.build_spatial_relationships();

        // Build physics constraints and forces
        self.build_physics_relationships(scene_def);

        // Update spatial index
        self.update_spatial_index();

        self.topology_version += 1;
    }

    /// Recursively build node hierarchy
    fn build_hierarchy(&mut self, nodes: &[SceneNode], parent_id: Option<String>) -> Vec<NodeIndex> {
        let mut node_indices = Vec::new();

        for (child_index, node) in nodes.iter().enumerate() {
            // Create vertex from scene node
            let vertex = Vertex::from_scene_node(node, parent_id.clone());
            let node_index = self.graph.add_node(vertex);

            // Update lookups
            self.vertex_lookup.insert(node.id.clone(), node_index);

            // Store graph index in vertex
            if let Some(vertex_data) = self.graph.node_weight_mut(node_index) {
                vertex_data.graph_index = Some(node_index);
            }

            // Create parent-child edge if not root
            if let Some(parent_id) = &parent_id {
                if let Some(&parent_index) = self.vertex_lookup.get(parent_id) {
                    let edge = Edge::parent_child(parent_id.clone(), node.id.clone(), child_index);
                    let edge_index = self.graph.add_edge(parent_index, node_index, edge);

                    // Update edge lookup
                    let edge_key = (parent_id.clone(), node.id.clone(), "ParentChild".to_string());
                    self.edge_lookup.insert(edge_key, edge_index);

                    // Store graph index in edge
                    if let Some(edge_data) = self.graph.edge_weight_mut(edge_index) {
                        edge_data.graph_index = Some(edge_index);
                    }
                }
            } else {
                // Root vertex
                self.root_vertices.push(node_index);
            }

            // Recursively process children
            let child_indices = self.build_hierarchy(&node.children, Some(node.id.clone()));

            // Update vertex children
            if let Some(vertex_data) = self.graph.node_weight_mut(node_index) {
                vertex_data.child_ids = child_indices.iter()
                    .filter_map(|&idx| self.graph.node_weight(idx).map(|v| v.id.clone()))
                    .collect();
            }

            node_indices.push(node_index);
        }

        node_indices
    }

    /// Build spatial proximity relationships
    fn build_spatial_relationships(&mut self) {
        let vertices: Vec<_> = self.graph.node_indices().collect();

        for &i in &vertices {
            for &j in &vertices {
                if i == j { continue; }

                let vertex_i = &self.graph[i];
                let vertex_j = &self.graph[j];

                // Calculate distance
                let pos_i = vertex_i.transform.position;
                let pos_j = vertex_j.transform.position;
                let distance = (pos_i - pos_j).magnitude();

                // Create spatial edge if close enough (arbitrary threshold)
                if distance < 50.0 {
                    let direction = (pos_j - pos_i).normalize();
                    let edge = Edge::spatial(
                        vertex_i.id.clone(),
                        vertex_j.id.clone(),
                        distance,
                        [direction.x, direction.y, direction.z],
                    );

                    let edge_index = self.graph.add_edge(i, j, edge);

                    let edge_key = (
                        vertex_i.id.clone(),
                        vertex_j.id.clone(),
                        "Spatial".to_string()
                    );
                    self.edge_lookup.insert(edge_key, edge_index);

                    if let Some(edge_data) = self.graph.edge_weight_mut(edge_index) {
                        edge_data.graph_index = Some(edge_index);
                    }
                }
            }
        }
    }

    /// Build physics constraints and force relationships
    fn build_physics_relationships(&mut self, scene_def: &SceneDefinition) {
        // Process each vertex for physics relationships
        for node_index in self.graph.node_indices() {
            let vertex = &self.graph[node_index];

            // Find physics properties in scene definition
            if let Some(scene_node) = self.find_scene_node(&scene_def.nodes, &vertex.id) {
                if let crate::dsl::scene::NodeProperties::Geometry(props) = &scene_node.properties {
                    if let Some(physics) = &props.physics {
                        // Create force relationships based on physics forces
                        for force in &physics.forces {
                            match force.force_type {
                                crate::dsl::scene::ForceType::Gravity => {
                                    // Gravity affects all dynamic objects
                                    self.create_gravity_relationships(node_index, force);
                                }
                                crate::dsl::scene::ForceType::Spring => {
                                    // Spring forces need target objects
                                    if let Some(target_id) = force.point {
                                        // For simplicity, assume target is another vertex
                                        // In practice, you'd resolve this properly
                                    }
                                }
                                _ => {
                                    // Other force types can be added here
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Create gravity relationships
    fn create_gravity_relationships(&mut self, source_index: NodeIndex, force: &crate::dsl::scene::Force) {
        for target_index in self.graph.node_indices() {
            if source_index == target_index { continue; }

            let source_vertex = &self.graph[source_index];
            let target_vertex = &self.graph[target_index];

            // Skip if target is not dynamic
            if target_vertex.physics_state.as_ref()
                .map_or(true, |p| !matches!(p.is_kinematic, false)) {
                continue;
            }

            let direction = [force.vector.x, force.vector.y, force.vector.z];
            let edge = Edge::force(
                source_vertex.id.clone(),
                target_vertex.id.clone(),
                super::edge::ForceType::Gravity,
                force.magnitude.unwrap_or(force.vector.magnitude()),
                direction,
                0.0, // Default range
                super::edge::FalloffType::InverseSquare,
            );

            let edge_index = self.graph.add_edge(source_index, target_index, edge);

            let edge_key = (
                source_vertex.id.clone(),
                target_vertex.id.clone(),
                "Force".to_string()
            );
            self.edge_lookup.insert(edge_key, edge_index);

            if let Some(edge_data) = self.graph.edge_weight_mut(edge_index) {
                edge_data.graph_index = Some(edge_index);
            }
        }
    }

    /// Find scene node by ID (recursive)
    fn find_scene_node<'a>(&self, nodes: &'a [SceneNode], target_id: &str) -> Option<&'a SceneNode> {
        for node in nodes {
            if node.id == target_id {
                return Some(node);
            }
            if let Some(found) = self.find_scene_node(&node.children, target_id) {
                return Some(found);
            }
        }
        None
    }

    /// Update scene graph topology
    pub fn update_topology(&mut self) {
        // Update spatial relationships
        self.update_spatial_relationships();

        // Update physics relationships
        self.update_physics_relationships();

        // Update spatial index
        self.update_spatial_index();

        self.topology_version += 1;
    }

    /// Update spatial relationships (distances, directions)
    fn update_spatial_relationships(&mut self) {
        let mut edges_to_update = Vec::new();

        // Collect spatial edges that need updating
        for edge_index in self.graph.edge_indices() {
            if let Some(edge) = self.graph.edge_weight(edge_index) {
                if matches!(edge.edge_type, super::edge::EdgeType::Spatial) {
                    edges_to_update.push(edge_index);
                }
            }
        }

        // Update spatial edges
        for edge_index in edges_to_update {
            // First get the endpoints and vertex positions
            let (source_idx, target_idx) = self.graph.edge_endpoints(edge_index).unwrap();
            let source_pos = self.graph[source_idx].transform.position;
            let target_pos = self.graph[target_idx].transform.position;
            let distance = (source_pos - target_pos).magnitude();
            let dir = (target_pos - source_pos).normalize();

            // Then update the edge
            if let Some(edge) = self.graph.edge_weight_mut(edge_index) {
                if let super::edge::EdgeProperties::Spatial { distance: dist, direction } = &mut edge.properties {
                    *dist = distance;
                    *direction = [dir.x, dir.y, dir.z];
                }
                edge.update_weight();
            }
        }
    }

    /// Update physics relationships
    fn update_physics_relationships(&mut self) {
        // Update force magnitudes based on current distances
        let mut edges_to_update = Vec::new();

        for edge_index in self.graph.edge_indices() {
            if let Some(edge) = self.graph.edge_weight(edge_index) {
                if matches!(edge.edge_type, super::edge::EdgeType::Force) {
                    edges_to_update.push(edge_index);
                }
            }
        }

        for edge_index in edges_to_update {
            // First get the endpoints and vertex positions
            let (source_idx, target_idx) = self.graph.edge_endpoints(edge_index).unwrap();
            let source_pos = self.graph[source_idx].transform.position;
            let target_pos = self.graph[target_idx].transform.position;
            let distance = (source_pos - target_pos).magnitude();

            // Then update the edge
            if let Some(edge) = self.graph.edge_weight_mut(edge_index) {
                if let super::edge::EdgeProperties::Force { magnitude, falloff, range, .. } = &mut edge.properties {
                    // Apply falloff
                    let falloff_factor = match falloff {
                        super::edge::FalloffType::Constant => 1.0,
                        super::edge::FalloffType::Linear => {
                            if *range > 0.0 { (*range - distance).max(0.0) / *range } else { 1.0 }
                        }
                        super::edge::FalloffType::InverseSquare => {
                            if distance > 0.0 { 1.0 / (distance * distance) } else { 1.0 }
                        }
                        super::edge::FalloffType::Exponential => {
                            (-distance * 0.1).exp()
                        }
                    };

                    *magnitude *= falloff_factor;
                }

                edge.update_weight();
            }
        }
    }

    /// Update spatial index for efficient queries
    fn update_spatial_index(&mut self) {
        self.spatial_index.clear();

        for node_index in self.graph.node_indices() {
            let vertex = &self.graph[node_index];
            self.spatial_index.insert(vertex.transform.position, node_index);
        }
    }

    /// Query vertices within radius
    pub fn query_radius(&self, center: Vector3<f32>, radius: f32) -> Vec<NodeIndex> {
        self.spatial_index.query_radius(center, radius)
    }

    /// Add incident to the graph
    pub fn add_incident(&mut self, incident: Incident) {
        self.incident_manager.add_incident(incident);
    }

    /// Process pending incidents
    pub fn process_incidents(&mut self) {
        let vertices: Vec<&Vertex> = self.graph.node_weights().collect();
        let edges: Vec<&Edge> = self.graph.edge_weights().collect();

        self.incident_manager.process_incidents(&vertices, &edges);
    }

    /// Get vertex by ID
    pub fn get_vertex(&self, id: &str) -> Option<&Vertex> {
        self.vertex_lookup.get(id)
            .and_then(|&idx| self.graph.node_weight(idx))
    }

    /// Get vertex mutably by ID
    pub fn get_vertex_mut(&mut self, id: &str) -> Option<&mut Vertex> {
        if let Some(&idx) = self.vertex_lookup.get(id) {
            self.graph.node_weight_mut(idx)
        } else {
            None
        }
    }

    /// Get edge between vertices
    pub fn get_edge(&self, source_id: &str, target_id: &str, edge_type: &str) -> Option<&Edge> {
        let key = (source_id.to_string(), target_id.to_string(), edge_type.to_string());
        self.edge_lookup.get(&key)
            .and_then(|&idx| self.graph.edge_weight(idx))
    }

    /// Update vertex transform
    pub fn update_vertex_transform(&mut self, vertex_id: &str, transform: crate::dsl::scene::Transform) {
        if let Some(vertex) = self.get_vertex_mut(vertex_id) {
            vertex.update_transform(transform);
            self.topology_version += 1;
        }
    }

    /// Apply force to vertex
    pub fn apply_force(&mut self, vertex_id: &str, force: Vector3<f32>) {
        if let Some(vertex) = self.get_vertex_mut(vertex_id) {
            vertex.apply_force(force);
        }
    }

    /// Update physics for all vertices
    pub fn update_physics(&mut self, delta_time: f32) {
        for node_index in self.graph.node_indices() {
            if let Some(vertex) = self.graph.node_weight_mut(node_index) {
                vertex.update_physics(delta_time);
            }
        }
    }

    /// Get topology version
    pub fn topology_version(&self) -> u64 {
        self.topology_version
    }

    /// Get graph statistics
    pub fn statistics(&self) -> GraphStatistics {
        GraphStatistics {
            vertex_count: self.graph.node_count(),
            edge_count: self.graph.edge_count(),
            root_count: self.root_vertices.len(),
            active_incidents: self.incident_manager.active_count(),
            topology_version: self.topology_version,
        }
    }

    /// Clear the entire graph
    pub fn clear(&mut self) {
        self.graph.clear();
        self.vertex_lookup.clear();
        self.edge_lookup.clear();
        self.root_vertices.clear();
        self.spatial_index.clear();
        self.incident_manager.clear();
        self.topology_version = 0;
    }

    /// Export graph to JSON (for debugging)
    pub fn to_json(&self) -> serde_json::Value {
        let vertices: Vec<_> = self.graph.node_weights()
            .map(|v| serde_json::json!({
                "id": v.id,
                "type": format!("{:?}", v.node_type),
                "position": [v.transform.position.x, v.transform.position.y, v.transform.position.z],
                "physics": v.physics_state.is_some()
            }))
            .collect();

        let edges: Vec<_> = self.graph.edge_weights()
            .map(|e| serde_json::json!({
                "source": e.source_id,
                "target": e.target_id,
                "type": format!("{:?}", e.edge_type),
                "weight": e.weight
            }))
            .collect();

        serde_json::json!({
            "vertices": vertices,
            "edges": edges,
            "topology_version": self.topology_version,
            "statistics": self.statistics()
        })
    }
}

/// Graph statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphStatistics {
    pub vertex_count: usize,
    pub edge_count: usize,
    pub root_count: usize,
    pub active_incidents: usize,
    pub topology_version: u64,
}

impl SpatialIndex {
    /// Create new spatial index
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    /// Insert vertex into spatial index
    pub fn insert(&mut self, position: Vector3<f32>, node_index: NodeIndex) {
        let cell = self.position_to_cell(position);
        self.cells.entry(cell).or_insert_with(Vec::new).push(node_index);
    }

    /// Query vertices within radius
    pub fn query_radius(&self, center: Vector3<f32>, radius: f32) -> Vec<NodeIndex> {
        let mut result = Vec::new();
        let min_cell = self.position_to_cell(center - Vector3::new(radius, radius, radius));
        let max_cell = self.position_to_cell(center + Vector3::new(radius, radius, radius));

        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                for z in min_cell.2..=max_cell.2 {
                    if let Some(vertices) = self.cells.get(&(x, y, z)) {
                        result.extend(vertices.iter().cloned());
                    }
                }
            }
        }

        result
    }

    /// Convert position to cell coordinates
    fn position_to_cell(&self, position: Vector3<f32>) -> (i32, i32, i32) {
        (
            (position.x / self.cell_size).floor() as i32,
            (position.y / self.cell_size).floor() as i32,
            (position.z / self.cell_size).floor() as i32,
        )
    }

    /// Clear spatial index
    pub fn clear(&mut self) {
        self.cells.clear();
    }
}

impl Default for SceneGraph {
    fn default() -> Self {
        Self::new()
    }
}
