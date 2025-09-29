//! Collision system for physics simulation
//!
//! Handles collision detection, contact resolution, and collision events.

use super::world::PhysicsWorld;
use crate::graph::{incident::{Incident, IncidentType}, vertex::Vertex, edge::Edge};
use nalgebra::Vector3;
use rapier3d::prelude::*;
use std::collections::HashMap;

/// Collision system managing collision detection and response
pub struct CollisionSystem {
    /// Collision event handlers
    event_handlers: Vec<Box<dyn CollisionHandler>>,
    /// Collision filters (body_id -> filter function)
    filters: HashMap<String, Box<dyn CollisionFilter>>,
    /// Collision groups
    groups: HashMap<String, CollisionGroup>,
    /// Active collisions (for continuous collision detection)
    active_collisions: HashMap<(String, String), CollisionInfo>,
}

/// Collision event handler trait
pub trait CollisionHandler {
    /// Handle collision start event
    fn on_collision_start(&mut self, collision: &CollisionInfo, world: &PhysicsWorld);

    /// Handle collision end event
    fn on_collision_end(&mut self, collision: &CollisionInfo, world: &PhysicsWorld);

    /// Handle collision update (continuous contact)
    fn on_collision_update(&mut self, collision: &CollisionInfo, world: &PhysicsWorld);
}

/// Collision filter trait
pub trait CollisionFilter {
    /// Return true if collision between these bodies should be allowed
    fn should_collide(&self, body_a: &str, body_b: &str, world: &PhysicsWorld) -> bool;
}

/// Collision group for managing collision layers
#[derive(Debug, Clone)]
pub struct CollisionGroup {
    /// Group name
    name: String,
    /// Collision mask (bitmask of groups this group collides with)
    mask: u32,
    /// Group membership bit
    membership: u32,
}

/// Collision information
#[derive(Debug, Clone)]
pub struct CollisionInfo {
    /// Body A ID
    pub body_a: String,
    /// Body B ID
    pub body_b: String,
    /// Contact points
    pub contacts: Vec<ContactPoint>,
    /// Total impulse
    pub total_impulse: Vector3<f32>,
    /// Relative velocity at contact
    pub relative_velocity: Vector3<f32>,
    /// Collision normal
    pub normal: Vector3<f32>,
    /// Penetration depth
    pub penetration: f32,
    /// Timestamp
    pub timestamp: std::time::Instant,
}

/// Contact point information
#[derive(Debug, Clone)]
pub struct ContactPoint {
    /// World position
    pub position: Vector3<f32>,
    /// Contact normal
    pub normal: Vector3<f32>,
    /// Penetration depth
    pub penetration: f32,
    /// Contact impulse
    pub impulse: f32,
}

impl CollisionSystem {
    /// Create a new collision system
    pub fn new() -> Self {
        Self {
            event_handlers: Vec::new(),
            filters: HashMap::new(),
            groups: HashMap::new(),
            active_collisions: HashMap::new(),
        }
    }

    /// Process collisions from the physics world
    pub fn process_collisions(&mut self, world: &mut PhysicsWorld) {
        // Process collision events from Rapier
        if let Some(event_handler) = world.event_handler.downcast_ref::<super::world::JigenEventHandler>() {
            // In a real implementation, we'd need to collect events during the physics step
            // For now, we'll process broad phase collisions
            self.process_broad_phase_collisions(world);
        }
    }

    /// Process broad phase collisions
    fn process_broad_phase_collisions(&mut self, world: &PhysicsWorld) {
        let mut new_collisions = Vec::new();

        // Check all potential collisions from narrow phase
        for (collider1, collider2, intersecting) in world.narrow_phase.intersections_with(world.collider_set.as_ref()) {
            if intersecting {
                if let (Some(body_a), Some(body_b)) = (
                    world.colliders.get(&collider1),
                    world.colliders.get(&collider2),
                ) {
                    // Check collision filter
                    if !self.should_collide(body_a, body_b, world) {
                        continue;
                    }

                    // Get contact information
                    if let Some(contact_pair) = world.narrow_phase.contact_pair(collider1, collider2) {
                        let collision_info = self.extract_collision_info(body_a, body_b, &contact_pair, world);

                        // Check if this is a new collision
                        let key = Self::collision_key(body_a, body_b);
                        if !self.active_collisions.contains_key(&key) {
                            // New collision
                            new_collisions.push((key.clone(), collision_info));
                        } else {
                            // Update existing collision
                            if let Some(existing) = self.active_collisions.get_mut(&key) {
                                *existing = collision_info;
                            }
                        }
                    }
                }
            }
        }

        // Process new collisions
        for (key, collision_info) in new_collisions {
            // Notify event handlers
            for handler in &mut self.event_handlers {
                handler.on_collision_start(&collision_info, world);
            }

            self.active_collisions.insert(key, collision_info);
        }

        // Process active collisions
        for collision_info in self.active_collisions.values() {
            for handler in &mut self.event_handlers {
                handler.on_collision_update(collision_info, world);
            }
        }
    }

    /// Extract collision information from contact pair
    fn extract_collision_info(&self, body_a: &str, body_b: &str, contact_pair: &ContactPair, world: &PhysicsWorld) -> CollisionInfo {
        let mut contacts = Vec::new();
        let mut total_impulse = Vector3::zeros();
        let mut total_normal = Vector3::zeros();

        for manifold in &contact_pair.manifolds {
            for contact in &manifold.contacts {
                let contact_point = ContactPoint {
                    position: contact.local_p1.into(), // World space position
                    normal: manifold.normal.into(),
                    penetration: contact.dist, // Negative distance = penetration
                    impulse: 0.0, // Would need to calculate from solver
                };
                contacts.push(contact_point);
                total_normal += contact_point.normal;
            }
        }

        // Average normal
        let normal = if contacts.is_empty() {
            Vector3::y() // Default up
        } else {
            (total_normal / contacts.len() as f32).normalize()
        };

        // Calculate relative velocity (simplified)
        let vel_a = world.get_velocity(body_a).unwrap_or(Vector3::zeros());
        let vel_b = world.get_velocity(body_b).unwrap_or(Vector3::zeros());
        let relative_velocity = vel_b - vel_a;

        // Calculate penetration
        let penetration = contacts.iter()
            .map(|c| c.penetration.abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        CollisionInfo {
            body_a: body_a.to_string(),
            body_b: body_b.to_string(),
            contacts,
            total_impulse,
            relative_velocity,
            normal,
            penetration,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Check if collision should be allowed
    fn should_collide(&self, body_a: &str, body_b: &str, world: &PhysicsWorld) -> bool {
        // Check custom filters first
        for filter in self.filters.values() {
            if !filter.should_collide(body_a, body_b, world) {
                return false;
            }
        }

        // Check collision groups
        if let (Some(group_a), Some(group_b)) = (
            self.get_body_group(body_a),
            self.get_body_group(body_b),
        ) {
            return (group_a.mask & group_b.membership) != 0;
        }

        // Default: allow all collisions
        true
    }

    /// Get collision group for body
    fn get_body_group(&self, body_id: &str) -> Option<&CollisionGroup> {
        // In a real implementation, you'd have a mapping from body ID to group
        // For now, return default group
        self.groups.get("default")
    }

    /// Create collision key (sorted to ensure consistency)
    fn collision_key(body_a: &str, body_b: &str) -> (String, String) {
        if body_a <= body_b {
            (body_a.to_string(), body_b.to_string())
        } else {
            (body_b.to_string(), body_a.to_string())
        }
    }

    /// Add collision event handler
    pub fn add_event_handler(&mut self, handler: Box<dyn CollisionHandler>) {
        self.event_handlers.push(handler);
    }

    /// Add collision filter
    pub fn add_filter(&mut self, body_id: String, filter: Box<dyn CollisionFilter>) {
        self.filters.insert(body_id, filter);
    }

    /// Add collision group
    pub fn add_group(&mut self, name: String, mask: u32, membership: u32) {
        let group = CollisionGroup { name: name.clone(), mask, membership };
        self.groups.insert(name, group);
    }

    /// Assign body to collision group
    pub fn assign_to_group(&mut self, body_id: String, group_name: &str) {
        // In a real implementation, you'd maintain a body -> group mapping
        // For now, this is a placeholder
    }

    /// Get active collision count
    pub fn active_collision_count(&self) -> usize {
        self.active_collisions.len()
    }

    /// Clear collision system
    pub fn clear(&mut self) {
        self.event_handlers.clear();
        self.filters.clear();
        self.groups.clear();
        self.active_collisions.clear();
    }

    /// Get collision statistics
    pub fn statistics(&self) -> CollisionStatistics {
        CollisionStatistics {
            active_collisions: self.active_collisions.len(),
            event_handlers: self.event_handlers.len(),
            filters: self.filters.len(),
            groups: self.groups.len(),
        }
    }
}

/// Collision statistics
#[derive(Debug, Clone)]
pub struct CollisionStatistics {
    pub active_collisions: usize,
    pub event_handlers: usize,
    pub filters: usize,
    pub groups: usize,
}

/// Default collision handler that creates incidents
pub struct IncidentCollisionHandler {
    /// Scene graph for incident creation
    scene_graph: Option<std::sync::Arc<std::sync::Mutex<crate::graph::SceneGraph>>>,
}

impl IncidentCollisionHandler {
    pub fn new() -> Self {
        Self { scene_graph: None }
    }

    pub fn with_scene_graph(scene_graph: std::sync::Arc<std::sync::Mutex<crate::graph::SceneGraph>>) -> Self {
        Self { scene_graph: Some(scene_graph) }
    }
}

impl CollisionHandler for IncidentCollisionHandler {
    fn on_collision_start(&mut self, collision: &CollisionInfo, _world: &PhysicsWorld) {
        if let Some(scene_graph) = &self.scene_graph {
            let incident = Incident::collision(
                format!("collision_{}_{}", collision.body_a, collision.body_b),
                vec![collision.body_a.clone(), collision.body_b.clone()],
                [collision.normal.x, collision.normal.y, collision.normal.z],
                [collision.normal.x, collision.normal.y, collision.normal.z],
                collision.total_impulse.magnitude(),
                [collision.relative_velocity.x, collision.relative_velocity.y, collision.relative_velocity.z],
            );

            if let Ok(mut sg) = scene_graph.lock() {
                sg.add_incident(incident);
            }
        }
    }

    fn on_collision_end(&mut self, _collision: &CollisionInfo, _world: &PhysicsWorld) {
        // Handle collision end if needed
    }

    fn on_collision_update(&mut self, _collision: &CollisionInfo, _world: &PhysicsWorld) {
        // Handle continuous collision updates if needed
    }
}

/// Example collision filter: Same group filter
pub struct SameGroupFilter {
    /// Allowed group
    allowed_group: String,
}

impl SameGroupFilter {
    pub fn new(group: String) -> Self {
        Self { allowed_group: group }
    }
}

impl CollisionFilter for SameGroupFilter {
    fn should_collide(&self, body_a: &str, body_b: &str, world: &PhysicsWorld) -> bool {
        // In a real implementation, you'd check group membership
        // For now, allow all collisions
        true
    }
}

/// Example collision filter: Distance-based filter
pub struct DistanceFilter {
    /// Maximum collision distance
    max_distance: f32,
}

impl DistanceFilter {
    pub fn new(max_distance: f32) -> Self {
        Self { max_distance }
    }
}

impl CollisionFilter for DistanceFilter {
    fn should_collide(&self, body_a: &str, body_b: &str, world: &PhysicsWorld) -> bool {
        let pos_a = world.get_position(body_a);
        let pos_b = world.get_position(body_b);

        if let (Some(pos_a), Some(pos_b)) = (pos_a, pos_b) {
            (pos_a - pos_b).magnitude() <= self.max_distance
        } else {
            false
        }
    }
}
