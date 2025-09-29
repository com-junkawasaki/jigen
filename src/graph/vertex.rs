//! Vertex definitions for scene graph
//!
//! Vertices represent scene nodes with their properties and state.

use crate::dsl::scene::{SceneNode, NodeType, Transform};
use nalgebra::{Vector3, Quaternion};
use petgraph::graph::NodeIndex;
use std::collections::HashMap;

/// Vertex representing a scene node in the graph
#[derive(Debug, Clone)]
pub struct Vertex {
    /// Unique identifier
    pub id: String,

    /// Node type
    pub node_type: NodeType,

    /// Current transform
    pub transform: Transform,

    /// Node properties (JSON value for flexibility)
    pub properties: serde_json::Value,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Physics state (if applicable)
    pub physics_state: Option<PhysicsState>,

    /// Graph node index
    pub graph_index: Option<NodeIndex>,

    /// Parent vertex ID
    pub parent_id: Option<String>,

    /// Child vertex IDs
    pub child_ids: Vec<String>,

    /// Metadata for graph algorithms
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Physics state for dynamic vertices
#[derive(Debug, Clone)]
pub struct PhysicsState {
    /// Current velocity
    pub velocity: Vector3<f32>,

    /// Current angular velocity
    pub angular_velocity: Vector3<f32>,

    /// Accumulated forces
    pub forces: Vec<Vector3<f32>>,

    /// Mass
    pub mass: f32,

    /// Whether the vertex is kinematic (moved by animation/physics)
    pub is_kinematic: bool,

    /// Collision shape bounding box (for broad-phase collision detection)
    pub bounding_box: BoundingBox,
}

/// Axis-aligned bounding box
#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

impl Vertex {
    /// Create a vertex from a scene node
    pub fn from_scene_node(node: &SceneNode, parent_id: Option<String>) -> Self {
        let physics_state = match &node.properties {
            crate::dsl::scene::NodeProperties::Geometry(props) => {
                if let Some(physics) = &props.physics {
                    Some(PhysicsState {
                        velocity: physics.velocity.unwrap_or(Vector3::zeros()),
                        angular_velocity: physics.angular_velocity.unwrap_or(Vector3::zeros()),
                        forces: physics.forces.iter().map(|f| f.vector).collect(),
                        mass: physics.mass,
                        is_kinematic: matches!(physics.body_type, crate::dsl::scene::PhysicsBodyType::Kinematic),
                        bounding_box: Self::calculate_bounding_box(&node.transform, props),
                    })
                } else {
                    None
                }
            }
            _ => None,
        };

        Self {
            id: node.id.clone(),
            node_type: node.node_type.clone(),
            transform: node.transform.clone(),
            properties: serde_json::to_value(&node.properties).unwrap_or(serde_json::Value::Null),
            tags: node.tags.clone(),
            physics_state,
            graph_index: None,
            parent_id,
            child_ids: node.children.iter().map(|c| c.id.clone()).collect(),
            metadata: HashMap::new(),
        }
    }

    /// Update vertex transform
    pub fn update_transform(&mut self, new_transform: Transform) {
        self.transform = new_transform;

        // Update bounding box if physics state exists
        if let Some(physics) = &mut self.physics_state {
            // Recalculate bounding box based on new transform
            // This is a simplified calculation - in practice you'd need geometry info
            physics.bounding_box = self.calculate_transformed_bbox(&physics.bounding_box);
        }
    }

    /// Apply physics forces
    pub fn apply_force(&mut self, force: Vector3<f32>) {
        if let Some(physics) = &mut self.physics_state {
            physics.forces.push(force);
        }
    }

    /// Update physics state
    pub fn update_physics(&mut self, delta_time: f32) {
        if let Some(physics) = &mut self.physics_state {
            if physics.mass > 0.0 && !physics.is_kinematic {
                // Calculate total force
                let total_force: Vector3<f32> = physics.forces.iter().sum();

                // F = ma -> a = F/m
                let acceleration = total_force / physics.mass;

                // Integrate velocity
                physics.velocity += acceleration * delta_time;

                // Integrate position
                self.transform.position += physics.velocity * delta_time;

                // Integrate angular velocity (simplified - no torque calculation)
                // In a full implementation, you'd calculate torque from forces and moments of inertia
                self.transform.rotation = self.integrate_angular_velocity(
                    &self.transform.rotation,
                    &physics.angular_velocity,
                    delta_time
                );

                // Clear forces for next frame
                physics.forces.clear();

                // Update bounding box
                physics.bounding_box = self.calculate_transformed_bbox(&physics.bounding_box);
            }
        }
    }

    /// Get world transform (accounting for parent hierarchy)
    pub fn world_transform(&self, parent_transform: Option<&Transform>) -> Transform {
        if let Some(parent) = parent_transform {
            // Combine transforms: child_world = parent_world * child_local
            Transform {
                position: parent.position + parent.rotation * self.transform.position,
                rotation: parent.rotation * self.transform.rotation,
                scale: parent.scale.component_mul(&self.transform.scale),
            }
        } else {
            self.transform.clone()
        }
    }

    /// Check if vertex has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }

    /// Add metadata
    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Calculate bounding box for geometry
    fn calculate_bounding_box(transform: &Transform, props: &crate::dsl::scene::GeometryProperties) -> BoundingBox {
        match &props.primitive {
            crate::dsl::scene::GeometryPrimitive::Box => {
                // Unit cube scaled by transform
                let half_size = transform.scale * 0.5;
                BoundingBox {
                    min: transform.position - half_size,
                    max: transform.position + half_size,
                }
            }
            crate::dsl::scene::GeometryPrimitive::Sphere => {
                // Simplified sphere as cube
                let radius = transform.scale.x.max(transform.scale.y).max(transform.scale.z);
                BoundingBox {
                    min: transform.position - Vector3::new(radius, radius, radius),
                    max: transform.position + Vector3::new(radius, radius, radius),
                }
            }
            _ => {
                // Default fallback
                BoundingBox {
                    min: transform.position - Vector3::new(0.5, 0.5, 0.5),
                    max: transform.position + Vector3::new(0.5, 0.5, 0.5),
                }
            }
        }
    }

    /// Calculate transformed bounding box
    fn calculate_transformed_bbox(&self, local_bbox: &BoundingBox) -> BoundingBox {
        // Transform the eight corners of the bounding box and find new min/max
        let corners = [
            Vector3::new(local_bbox.min.x, local_bbox.min.y, local_bbox.min.z),
            Vector3::new(local_bbox.min.x, local_bbox.min.y, local_bbox.max.z),
            Vector3::new(local_bbox.min.x, local_bbox.max.y, local_bbox.min.z),
            Vector3::new(local_bbox.min.x, local_bbox.max.y, local_bbox.max.z),
            Vector3::new(local_bbox.max.x, local_bbox.min.y, local_bbox.min.z),
            Vector3::new(local_bbox.max.x, local_bbox.min.y, local_bbox.max.z),
            Vector3::new(local_bbox.max.x, local_bbox.max.y, local_bbox.min.z),
            Vector3::new(local_bbox.max.x, local_bbox.max.y, local_bbox.max.z),
        ];

        let mut world_corners: Vec<Vector3<f32>> = corners
            .iter()
            .map(|corner| {
                // Apply rotation and scale, then translate
                let scaled = corner.component_mul(&self.transform.scale);
                let rotated = self.transform.rotation * scaled;
                rotated + self.transform.position
            })
            .collect();

        // Find min and max
        let mut min = world_corners[0];
        let mut max = world_corners[0];

        for corner in world_corners.iter().skip(1) {
            min = min.inf(corner);
            max = max.sup(corner);
        }

        BoundingBox { min, max }
    }

    /// Integrate angular velocity (simplified quaternion integration)
    fn integrate_angular_velocity(
        &self,
        current_rotation: &Quaternion<f32>,
        angular_velocity: &Vector3<f32>,
        delta_time: f32
    ) -> Quaternion<f32> {
        // Simplified angular integration - just add angular velocity to rotation
        // In a real implementation, you'd use proper quaternion integration
        let delta_rotation = Quaternion::new(0.0, angular_velocity.x * delta_time, angular_velocity.y * delta_time, angular_velocity.z * delta_time);
        let new_rotation = current_rotation * delta_rotation;

        // Normalize to prevent drift
        let norm = (new_rotation.w * new_rotation.w +
                   new_rotation.i * new_rotation.i +
                   new_rotation.j * new_rotation.j +
                   new_rotation.k * new_rotation.k).sqrt();
        Quaternion::new(
            new_rotation.w / norm,
            new_rotation.i / norm,
            new_rotation.j / norm,
            new_rotation.k / norm,
        )
    }
}

impl BoundingBox {
    /// Check if bounding boxes intersect
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        !(self.max.x < other.min.x || self.min.x > other.max.x ||
          self.max.y < other.min.y || self.min.y > other.max.y ||
          self.max.z < other.min.z || self.min.z > other.max.z)
    }

    /// Get center of bounding box
    pub fn center(&self) -> Vector3<f32> {
        (self.min + self.max) * 0.5
    }

    /// Get size/extents of bounding box
    pub fn size(&self) -> Vector3<f32> {
        self.max - self.min
    }
}
