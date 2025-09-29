//! Force system for physics simulation
//!
//! Manages various types of forces including gravity, springs, damping, etc.

use super::world::PhysicsWorld;
use nalgebra::Vector3;
use std::collections::HashMap;

/// Force system managing force calculations and application
pub struct ForceSystem {
    /// Gravity force generator
    gravity: GravityForce,
    /// Spring forces
    springs: HashMap<String, SpringForce>,
    /// Damping forces
    dampers: HashMap<String, DampingForce>,
    /// Wind forces
    winds: Vec<WindForce>,
    /// Custom force generators
    custom_forces: Vec<Box<dyn CustomForce>>,
}

/// Gravity force generator
pub struct GravityForce {
    /// Gravity acceleration vector
    acceleration: Vector3<f32>,
    /// Affected body types (empty means all)
    affected_types: Vec<String>,
}

/// Spring force between two bodies
pub struct SpringForce {
    /// Body A ID
    body_a: String,
    /// Body B ID
    body_b: String,
    /// Rest length
    rest_length: f32,
    /// Spring constant
    stiffness: f32,
    /// Damping coefficient
    damping: f32,
}

/// Damping force (velocity-dependent)
pub struct DampingForce {
    /// Affected body ID
    body_id: String,
    /// Linear damping coefficient
    linear_damping: f32,
    /// Angular damping coefficient
    angular_damping: f32,
}

/// Wind force field
pub struct WindForce {
    /// Force vector
    force: Vector3<f32>,
    /// Affected region (center, radius)
    region: Option<(Vector3<f32>, f32)>,
}

/// Custom force trait
pub trait CustomForce {
    /// Calculate force for a specific body
    fn calculate_force(&self, body_id: &str, world: &PhysicsWorld) -> Option<Vector3<f32>>;
}

impl ForceSystem {
    /// Create a new force system
    pub fn new() -> Self {
        Self {
            gravity: GravityForce::new(Vector3::new(0.0, -9.81, 0.0)),
            springs: HashMap::new(),
            dampers: HashMap::new(),
            winds: Vec::new(),
            custom_forces: Vec::new(),
        }
    }

    /// Apply all forces to the physics world
    pub fn apply_forces(&self, world: &mut PhysicsWorld) {
        // Apply gravity to all dynamic bodies
        self.gravity.apply_forces(world);

        // Apply spring forces
        for spring in self.springs.values() {
            spring.apply_forces(world);
        }

        // Apply damping forces
        for damper in self.dampers.values() {
            damper.apply_forces(world);
        }

        // Apply wind forces
        for wind in &self.winds {
            wind.apply_forces(world);
        }

        // Apply custom forces
        for custom in &self.custom_forces {
            self.apply_custom_force(custom, world);
        }
    }

    /// Apply custom force to all bodies
    fn apply_custom_force(&self, custom_force: &Box<dyn CustomForce>, world: &mut PhysicsWorld) {
        // Get all body IDs
        let body_ids: Vec<String> = world.bodies.values().cloned().collect();

        for body_id in body_ids {
            if let Some(force) = custom_force.calculate_force(&body_id, world) {
                world.apply_force(&body_id, force, None);
            }
        }
    }

    /// Set gravity
    pub fn set_gravity(&mut self, gravity: Vector3<f32>) {
        self.gravity.acceleration = gravity;
    }

    /// Add spring force
    pub fn add_spring(&mut self, id: String, body_a: String, body_b: String, rest_length: f32, stiffness: f32, damping: f32) {
        let spring = SpringForce {
            body_a,
            body_b,
            rest_length,
            stiffness,
            damping,
        };
        self.springs.insert(id, spring);
    }

    /// Remove spring force
    pub fn remove_spring(&mut self, id: &str) -> bool {
        self.springs.remove(id).is_some()
    }

    /// Add damping force
    pub fn add_damper(&mut self, id: String, body_id: String, linear_damping: f32, angular_damping: f32) {
        let damper = DampingForce {
            body_id,
            linear_damping,
            angular_damping,
        };
        self.dampers.insert(id, damper);
    }

    /// Remove damping force
    pub fn remove_damper(&mut self, id: &str) -> bool {
        self.dampers.remove(id).is_some()
    }

    /// Add wind force
    pub fn add_wind(&mut self, force: Vector3<f32>, region: Option<(Vector3<f32>, f32)>) {
        let wind = WindForce { force, region };
        self.winds.push(wind);
    }

    /// Clear wind forces
    pub fn clear_winds(&mut self) {
        self.winds.clear();
    }

    /// Add custom force generator
    pub fn add_custom_force(&mut self, force: Box<dyn CustomForce>) {
        self.custom_forces.push(force);
    }

    /// Clear all forces
    pub fn clear(&mut self) {
        self.springs.clear();
        self.dampers.clear();
        self.winds.clear();
        self.custom_forces.clear();
    }
}

impl GravityForce {
    /// Create new gravity force
    pub fn new(acceleration: Vector3<f32>) -> Self {
        Self {
            acceleration,
            affected_types: Vec::new(), // Empty means all types
        }
    }

    /// Apply gravity forces
    pub fn apply_forces(&self, world: &mut PhysicsWorld) {
        // Gravity is handled by the physics engine itself
        // This method is for additional gravity-like forces if needed
    }
}

impl SpringForce {
    /// Apply spring forces
    pub fn apply_forces(&self, world: &mut PhysicsWorld) {
        let pos_a = match world.get_position(&self.body_a) {
            Some(pos) => pos,
            None => return,
        };

        let pos_b = match world.get_position(&self.body_b) {
            Some(pos) => pos,
            None => return,
        };

        let vel_a = world.get_velocity(&self.body_a).unwrap_or(Vector3::zeros());
        let vel_b = world.get_velocity(&self.body_b).unwrap_or(Vector3::zeros());

        // Calculate spring direction and length
        let direction = (pos_b - pos_a).normalize();
        let current_length = (pos_b - pos_a).magnitude();

        // Hooke's law: F = -k * (current_length - rest_length)
        let spring_force_magnitude = -self.stiffness * (current_length - self.rest_length);
        let spring_force = direction * spring_force_magnitude;

        // Damping force: F_damping = -damping * relative_velocity
        let relative_velocity = vel_b - vel_a;
        let damping_force = -direction * self.damping * relative_velocity.dot(&direction);

        // Total force
        let total_force = spring_force + damping_force;

        // Apply equal and opposite forces
        world.apply_force(&self.body_a, total_force, None);
        world.apply_force(&self.body_b, -total_force, None);
    }
}

impl DampingForce {
    /// Apply damping forces
    pub fn apply_forces(&self, world: &mut PhysicsWorld) {
        if let Some(velocity) = world.get_velocity(&self.body_id) {
            let damping_force = -velocity * self.linear_damping;
            world.apply_force(&self.body_id, damping_force, None);
        }

        // Angular damping would require angular velocity access
        // This is simplified - full implementation would need angular velocity
    }
}

impl WindForce {
    /// Apply wind forces
    pub fn apply_forces(&self, world: &mut PhysicsWorld) {
        // Get all dynamic bodies
        let body_ids: Vec<String> = world.bodies.values()
            .filter(|&id| {
                // Check if body is in affected region
                if let Some((center, radius)) = self.region {
                    if let Some(pos) = world.get_position(id) {
                        (pos - center).magnitude() <= radius
                    } else {
                        false
                    }
                } else {
                    true // No region restriction
                }
            })
            .cloned()
            .collect();

        for body_id in body_ids {
            world.apply_force(&body_id, self.force, None);
        }
    }
}

/// Example custom force: Buoyancy
pub struct BuoyancyForce {
    /// Water surface height
    water_level: f32,
    /// Buoyancy strength
    strength: f32,
}

impl BuoyancyForce {
    pub fn new(water_level: f32, strength: f32) -> Self {
        Self { water_level, strength }
    }
}

impl CustomForce for BuoyancyForce {
    fn calculate_force(&self, body_id: &str, world: &PhysicsWorld) -> Option<Vector3<f32>> {
        let position = world.get_position(body_id)?;

        // Simple buoyancy model - force proportional to submerged volume
        if position.y < self.water_level {
            let submerged_fraction = (self.water_level - position.y).min(1.0).max(0.0);
            Some(Vector3::new(0.0, self.strength * submerged_fraction, 0.0))
        } else {
            None
        }
    }
}

/// Example custom force: Vortex
pub struct VortexForce {
    /// Vortex center
    center: Vector3<f32>,
    /// Vortex strength
    strength: f32,
    /// Vortex radius
    radius: f32,
}

impl VortexForce {
    pub fn new(center: Vector3<f32>, strength: f32, radius: f32) -> Self {
        Self { center, strength, radius }
    }
}

impl CustomForce for VortexForce {
    fn calculate_force(&self, body_id: &str, world: &PhysicsWorld) -> Option<Vector3<f32>> {
        let position = world.get_position(body_id)?;
        let to_center = self.center - position;
        let distance = to_center.magnitude();

        if distance < self.radius && distance > 0.0 {
            // Tangential force creating rotation around center
            let tangential_dir = Vector3::new(-to_center.z, 0.0, to_center.x).normalize();
            let force_magnitude = self.strength * (1.0 - distance / self.radius);
            Some(tangential_dir * force_magnitude)
        } else {
            None
        }
    }
}
