//! Edge definitions for scene graph relationships
//!
//! Edges represent connections between vertices with different relationship types.

use petgraph::graph::EdgeIndex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Edge representing a relationship between two vertices
#[derive(Debug, Clone)]
pub struct Edge {
    /// Source vertex ID
    pub source_id: String,

    /// Target vertex ID
    pub target_id: String,

    /// Edge type
    pub edge_type: EdgeType,

    /// Edge properties
    pub properties: EdgeProperties,

    /// Graph edge index
    pub graph_index: Option<EdgeIndex>,

    /// Edge weight/strength (for graph algorithms)
    pub weight: f32,

    /// Bidirectional flag
    pub bidirectional: bool,

    /// Metadata for graph algorithms
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of relationships between vertices
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeType {
    /// Hierarchical parent-child relationship
    ParentChild,
    /// Spatial proximity/adjacency
    Spatial,
    /// Physical constraint (joints, springs, etc.)
    Constraint,
    /// Force interaction (gravity, magnetism, etc.)
    Force,
    /// Visual relationship (lighting, shadowing)
    Visual,
    /// Collision relationship
    Collision,
    /// Custom user-defined relationship
    Custom(String),
}

/// Properties specific to different edge types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeProperties {
    /// Parent-child hierarchy properties
    ParentChild {
        /// Child index in parent's children list
        child_index: usize,
    },

    /// Spatial relationship properties
    Spatial {
        /// Distance between vertices
        distance: f32,
        /// Direction vector from source to target
        direction: [f32; 3],
    },

    /// Physical constraint properties
    Constraint {
        /// Constraint type
        constraint_type: ConstraintType,
        /// Constraint strength (0-1)
        strength: f32,
        /// Rest length/distance
        rest_length: Option<f32>,
        /// Damping factor
        damping: f32,
    },

    /// Force interaction properties
    Force {
        /// Force type
        force_type: ForceType,
        /// Force magnitude
        magnitude: f32,
        /// Force direction
        direction: [f32; 3],
        /// Force range (0 for infinite)
        range: f32,
        /// Force falloff function
        falloff: FalloffType,
    },

    /// Visual relationship properties
    Visual {
        /// Visual relationship type
        visual_type: VisualType,
        /// Influence strength
        influence: f32,
    },

    /// Collision relationship properties
    Collision {
        /// Collision normal
        normal: [f32; 3],
        /// Penetration depth
        penetration: f32,
        /// Contact points
        contacts: Vec<[f32; 3]>,
    },

    /// Custom properties
    Custom {
        /// Custom property data
        data: serde_json::Value,
    },
}

/// Constraint types for physics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Distance constraint (fixed distance between points)
    Distance,
    /// Hinge constraint (rotation around axis)
    Hinge,
    /// Ball-socket constraint (spherical joint)
    BallSocket,
    /// Fixed constraint (no relative motion)
    Fixed,
    /// Spring constraint (elastic connection)
    Spring,
}

/// Force types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForceType {
    /// Gravitational attraction
    Gravity,
    /// Electromagnetic force
    Electromagnetic,
    /// Spring force
    Spring,
    /// Damping force (velocity-dependent)
    Damping,
    /// Wind/air resistance
    Wind,
    /// Custom force
    Custom(String),
}

/// Force falloff functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FalloffType {
    /// No falloff (constant force)
    Constant,
    /// Linear falloff with distance
    Linear,
    /// Inverse square law (gravity, electromagnetism)
    InverseSquare,
    /// Exponential falloff
    Exponential,
}

/// Visual relationship types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualType {
    /// Lighting relationship (light affects object)
    Lighting,
    /// Shadowing relationship (object casts shadow on another)
    Shadowing,
    /// Reflection relationship
    Reflection,
    /// Refraction relationship
    Refraction,
    /// Occlusion relationship
    Occlusion,
}

impl Edge {
    /// Create a new edge
    pub fn new(source_id: String, target_id: String, edge_type: EdgeType) -> Self {
        Self {
            source_id,
            target_id,
            edge_type,
            properties: Self::default_properties(&edge_type),
            graph_index: None,
            weight: 1.0,
            bidirectional: Self::default_bidirectional(&edge_type),
            metadata: HashMap::new(),
        }
    }

    /// Create parent-child edge
    pub fn parent_child(parent_id: String, child_id: String, child_index: usize) -> Self {
        Self {
            source_id: parent_id,
            target_id: child_id,
            edge_type: EdgeType::ParentChild,
            properties: EdgeProperties::ParentChild { child_index },
            graph_index: None,
            weight: 1.0,
            bidirectional: false,
            metadata: HashMap::new(),
        }
    }

    /// Create spatial proximity edge
    pub fn spatial(source_id: String, target_id: String, distance: f32, direction: [f32; 3]) -> Self {
        Self {
            source_id,
            target_id,
            edge_type: EdgeType::Spatial,
            properties: EdgeProperties::Spatial { distance, direction },
            graph_index: None,
            weight: distance.recip(), // Closer objects have higher weight
            bidirectional: true,
            metadata: HashMap::new(),
        }
    }

    /// Create force interaction edge
    pub fn force(
        source_id: String,
        target_id: String,
        force_type: ForceType,
        magnitude: f32,
        direction: [f32; 3],
        range: f32,
        falloff: FalloffType,
    ) -> Self {
        Self {
            source_id,
            target_id,
            edge_type: EdgeType::Force,
            properties: EdgeProperties::Force {
                force_type,
                magnitude,
                direction,
                range,
                falloff,
            },
            graph_index: None,
            weight: magnitude,
            bidirectional: Self::is_force_bidirectional(&force_type),
            metadata: HashMap::new(),
        }
    }

    /// Create constraint edge
    pub fn constraint(
        source_id: String,
        target_id: String,
        constraint_type: ConstraintType,
        strength: f32,
        rest_length: Option<f32>,
        damping: f32,
    ) -> Self {
        Self {
            source_id,
            target_id,
            edge_type: EdgeType::Constraint,
            properties: EdgeProperties::Constraint {
                constraint_type,
                strength,
                rest_length,
                damping,
            },
            graph_index: None,
            weight: strength,
            bidirectional: true,
            metadata: HashMap::new(),
        }
    }

    /// Update edge properties
    pub fn update_properties(&mut self, properties: EdgeProperties) {
        self.properties = properties;
        self.update_weight();
    }

    /// Update edge weight based on properties
    pub fn update_weight(&mut self) {
        self.weight = match &self.properties {
            EdgeProperties::Spatial { distance, .. } => {
                if *distance > 0.0 { distance.recip() } else { 1000.0 }
            }
            EdgeProperties::Force { magnitude, .. } => *magnitude,
            EdgeProperties::Constraint { strength, .. } => *strength,
            EdgeProperties::Collision { penetration, .. } => penetration.recip(),
            _ => 1.0,
        };
    }

    /// Check if edge is active (not broken/disabled)
    pub fn is_active(&self) -> bool {
        match &self.properties {
            EdgeProperties::Constraint { strength, .. } => *strength > 0.0,
            EdgeProperties::Force { magnitude, .. } => *magnitude > 0.0,
            _ => true,
        }
    }

    /// Get default properties for edge type
    fn default_properties(edge_type: &EdgeType) -> EdgeProperties {
        match edge_type {
            EdgeType::ParentChild => EdgeProperties::ParentChild { child_index: 0 },
            EdgeType::Spatial => EdgeProperties::Spatial {
                distance: 1.0,
                direction: [0.0, 0.0, 1.0],
            },
            EdgeType::Constraint => EdgeProperties::Constraint {
                constraint_type: ConstraintType::Distance,
                strength: 1.0,
                rest_length: Some(1.0),
                damping: 0.1,
            },
            EdgeType::Force => EdgeProperties::Force {
                force_type: ForceType::Gravity,
                magnitude: 1.0,
                direction: [0.0, -1.0, 0.0],
                range: 0.0,
                falloff: FalloffType::InverseSquare,
            },
            EdgeType::Visual => EdgeProperties::Visual {
                visual_type: VisualType::Lighting,
                influence: 1.0,
            },
            EdgeType::Collision => EdgeProperties::Collision {
                normal: [0.0, 1.0, 0.0],
                penetration: 0.0,
                contacts: Vec::new(),
            },
            EdgeType::Custom(_) => EdgeProperties::Custom {
                data: serde_json::Value::Null,
            },
        }
    }

    /// Get default bidirectional flag for edge type
    fn default_bidirectional(edge_type: &EdgeType) -> bool {
        match edge_type {
            EdgeType::ParentChild => false,
            EdgeType::Spatial => true,
            EdgeType::Constraint => true,
            EdgeType::Force => Self::is_force_bidirectional(&ForceType::Gravity),
            EdgeType::Visual => false,
            EdgeType::Collision => true,
            EdgeType::Custom(_) => true,
        }
    }

    /// Check if force type is bidirectional
    fn is_force_bidirectional(force_type: &ForceType) -> bool {
        match force_type {
            ForceType::Gravity => true,
            ForceType::Electromagnetic => true,
            ForceType::Spring => true,
            ForceType::Damping => true,
            ForceType::Wind => false,
            ForceType::Custom(_) => true,
        }
    }

    /// Add metadata
    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Reverse edge direction
    pub fn reverse(&mut self) {
        std::mem::swap(&mut self.source_id, &mut self.target_id);

        // Update properties that depend on direction
        match &mut self.properties {
            EdgeProperties::Spatial { direction, .. } => {
                *direction = [-direction[0], -direction[1], -direction[2]];
            }
            EdgeProperties::Force { direction, .. } => {
                *direction = [-direction[0], -direction[1], -direction[2]];
            }
            EdgeProperties::ParentChild { .. } => {
                // Parent-child relationships shouldn't be reversed
                // This would be invalid
            }
            _ => {}
        }
    }
}

impl Default for EdgeType {
    fn default() -> Self {
        EdgeType::Spatial
    }
}

impl Default for EdgeProperties {
    fn default() -> Self {
        EdgeProperties::Spatial {
            distance: 1.0,
            direction: [0.0, 0.0, 1.0],
        }
    }
}
