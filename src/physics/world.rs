//! Physics world implementation
//!
//! Manages the physics simulation state and coordinates
//! force calculation, constraint solving, and collision detection.

use super::{forces::ForceSystem, constraints::ConstraintSystem, collisions::CollisionSystem};
use crate::dsl::scene::PhysicsConfig;
use nalgebra::{Vector3, Quaternion};
use rapier3d::prelude::*;
use std::collections::HashMap;

/// Physics world managing simulation state
pub struct PhysicsWorld {
    /// Rapier physics pipeline
    pub pipeline: PhysicsPipeline,

    /// Gravity vector
    pub gravity: Vector3<f32>,

    /// Physics bodies (RigidBodyHandle -> vertex ID mapping)
    pub bodies: HashMap<RigidBodyHandle, String>,

    /// Colliders (ColliderHandle -> vertex ID mapping)
    pub colliders: HashMap<ColliderHandle, String>,

    /// Rigid body set
    pub rigid_body_set: RigidBodySet,

    /// Collider set
    pub collider_set: ColliderSet,

    /// Force generators
    pub force_generators: Vec<Box<dyn ForceGenerator>>,

    /// Constraint system
    pub constraint_system: ConstraintSystem,

    /// Force system
    pub force_system: ForceSystem,

    /// Collision system
    pub collision_system: CollisionSystem,

    /// Integration parameters
    pub integration_parameters: IntegrationParameters,

    /// Island manager
    pub island_manager: IslandManager,

    /// Broad phase
    pub broad_phase: BroadPhase,

    /// Narrow phase
    pub narrow_phase: NarrowPhase,

    /// Joint set
    pub joint_set: JointSet,

    /// CCD solver
    pub ccd_solver: CCDSolver,

    /// Physics hooks
    pub physics_hooks: Box<dyn PhysicsHooks>,

    /// Event handler
    pub event_handler: Box<dyn EventHandler>,

    /// Simulation time accumulator
    pub time_accumulator: f32,

    /// Configuration
    pub config: PhysicsConfig,
}

/// Force generator trait for custom forces
pub trait ForceGenerator {
    /// Apply forces to the physics world
    fn apply_forces(&self, physics_world: &mut PhysicsWorld);
}

/// Physics event handler
pub struct JigenEventHandler {
    /// Collision events
    pub collision_events: Vec<CollisionEvent>,
    /// Contact force events
    pub contact_force_events: Vec<ContactForceEvent>,
}

impl PhysicsWorld {
    /// Create a new physics world
    pub fn new(gravity: Vector3<f32>) -> Self {
        let config = PhysicsConfig {
            enabled: true,
            gravity,
            time_step: 1.0 / 60.0,
            max_substeps: 10,
        };

        Self::with_config(config)
    }

    /// Create physics world with custom configuration
    pub fn with_config(config: PhysicsConfig) -> Self {
        let integration_parameters = IntegrationParameters {
            dt: config.time_step,
            min_ccd_dt: 0.0,
            max_ccd_substeps: 10,
            ..Default::default()
        };

        let event_handler = Box::new(JigenEventHandler {
            collision_events: Vec::new(),
            contact_force_events: Vec::new(),
        });

        Self {
            pipeline: PhysicsPipeline::new(),
            gravity: config.gravity,
            bodies: HashMap::new(),
            colliders: HashMap::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            force_generators: Vec::new(),
            constraint_system: ConstraintSystem::new(),
            force_system: ForceSystem::new(),
            collision_system: CollisionSystem::new(),
            integration_parameters,
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            joint_set: JointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_hooks: Box::new(()),
            event_handler,
            time_accumulator: 0.0,
            config,
        }
    }

    /// Step the physics simulation
    pub fn step(&mut self, delta_time: f32) {
        if !self.config.enabled {
            return;
        }

        self.time_accumulator += delta_time;

        // Fixed time stepping
        let time_step = self.config.time_step;
        let mut steps = 0;

        while self.time_accumulator >= time_step && steps < self.config.max_substeps {
            // Apply custom forces
            for force_gen in &self.force_generators {
                force_gen.apply_forces(self);
            }

            // Apply force system
            self.force_system.apply_forces(self);

            // Step the simulation
            self.pipeline.step(
                &self.gravity.into(),
                &self.integration_parameters,
                &mut self.island_manager,
                &mut self.broad_phase,
                &mut self.narrow_phase,
                &mut self.rigid_body_set,
                &mut self.collider_set,
                &mut self.joint_set,
                &mut self.ccd_solver,
                &self.physics_hooks,
                self.event_handler.as_ref(),
            );

            // Process constraints
            self.constraint_system.solve_constraints(self);

            // Process collisions
            self.collision_system.process_collisions(self);

            self.time_accumulator -= time_step;
            steps += 1;
        }

        // Clear events for next frame
        if let Some(handler) = self.event_handler.downcast_ref::<JigenEventHandler>() {
            // Note: We can't modify the handler directly since it's borrowed
            // Events are processed in the collision system
        }
    }

    /// Add a rigid body to the physics world
    pub fn add_rigid_body(&mut self, vertex_id: String, rigid_body: RigidBody) -> RigidBodyHandle {
        let handle = self.rigid_body_set.insert(rigid_body);
        self.bodies.insert(handle, vertex_id);
        handle
    }

    /// Add a collider to the physics world
    pub fn add_collider(&mut self, vertex_id: String, collider: Collider) -> ColliderHandle {
        let handle = self.collider_set.insert(collider);
        self.colliders.insert(handle, vertex_id);
        handle
    }

    /// Get rigid body by vertex ID
    pub fn get_rigid_body(&self, vertex_id: &str) -> Option<(RigidBodyHandle, &RigidBody)> {
        for (handle, id) in &self.bodies {
            if id == vertex_id {
                return self.rigid_body_set.get(*handle).map(|body| (*handle, body));
            }
        }
        None
    }

    /// Get rigid body mutably by vertex ID
    pub fn get_rigid_body_mut(&mut self, vertex_id: &str) -> Option<(RigidBodyHandle, &mut RigidBody)> {
        for (handle, id) in &self.bodies {
            if id == vertex_id {
                return self.rigid_body_set.get_mut(*handle).map(|body| (*handle, body));
            }
        }
        None
    }

    /// Remove rigid body by vertex ID
    pub fn remove_rigid_body(&mut self, vertex_id: &str) -> bool {
        if let Some((handle, _)) = self.get_rigid_body(vertex_id) {
            self.bodies.remove(&handle);
            self.rigid_body_set.remove(handle, &mut self.island_manager, &mut self.collider_set, &mut self.joint_set);
            true
        } else {
            false
        }
    }

    /// Apply force to a rigid body
    pub fn apply_force(&mut self, vertex_id: &str, force: Vector3<f32>, point: Option<Vector3<f32>>) {
        if let Some((_, body)) = self.get_rigid_body_mut(vertex_id) {
            if let Some(point) = point {
                body.add_force_at_point(force.into(), point.into(), true);
            } else {
                body.add_force(force.into(), true);
            }
        }
    }

    /// Apply impulse to a rigid body
    pub fn apply_impulse(&mut self, vertex_id: &str, impulse: Vector3<f32>, point: Option<Vector3<f32>>) {
        if let Some((_, body)) = self.get_rigid_body_mut(vertex_id) {
            if let Some(point) = point {
                body.apply_impulse_at_point(impulse.into(), point.into(), true);
            } else {
                body.apply_impulse(impulse.into(), true);
            }
        }
    }

    /// Set rigid body position
    pub fn set_position(&mut self, vertex_id: &str, position: Vector3<f32>) {
        if let Some((_, body)) = self.get_rigid_body_mut(vertex_id) {
            body.set_position(Isometry::translation(position.x, position.y, position.z), true);
        }
    }

    /// Set rigid body rotation
    pub fn set_rotation(&mut self, vertex_id: &str, rotation: Quaternion<f32>) {
        if let Some((_, body)) = self.get_rigid_body_mut(vertex_id) {
            let iso = body.position();
            let new_iso = Isometry::from_parts(
                iso.translation,
                UnitQuaternion::from_quaternion(rotation).into(),
            );
            body.set_position(new_iso, true);
        }
    }

    /// Set rigid body velocity
    pub fn set_velocity(&mut self, vertex_id: &str, velocity: Vector3<f32>) {
        if let Some((_, body)) = self.get_rigid_body_mut(vertex_id) {
            body.set_linvel(velocity.into(), true);
        }
    }

    /// Set rigid body angular velocity
    pub fn set_angular_velocity(&mut self, vertex_id: &str, angular_velocity: Vector3<f32>) {
        if let Some((_, body)) = self.get_rigid_body_mut(vertex_id) {
            body.set_angvel(angular_velocity.into(), true);
        }
    }

    /// Get rigid body position
    pub fn get_position(&self, vertex_id: &str) -> Option<Vector3<f32>> {
        self.get_rigid_body(vertex_id)
            .map(|(_, body)| body.position().translation.vector.into())
    }

    /// Get rigid body velocity
    pub fn get_velocity(&self, vertex_id: &str) -> Option<Vector3<f32>> {
        self.get_rigid_body(vertex_id)
            .map(|(_, body)| body.linvel().into())
    }

    /// Add force generator
    pub fn add_force_generator(&mut self, generator: Box<dyn ForceGenerator>) {
        self.force_generators.push(generator);
    }

    /// Get physics statistics
    pub fn statistics(&self) -> PhysicsStatistics {
        PhysicsStatistics {
            body_count: self.rigid_body_set.len(),
            collider_count: self.collider_set.len(),
            joint_count: self.joint_set.len(),
            island_count: self.island_manager.active_islands().count(),
            time_accumulator: self.time_accumulator,
            gravity: self.gravity,
        }
    }

    /// Clear the physics world
    pub fn clear(&mut self) {
        self.bodies.clear();
        self.colliders.clear();
        self.rigid_body_set.clear();
        self.collider_set.clear();
        self.joint_set.clear();
        self.force_generators.clear();
        self.constraint_system.clear();
        self.force_system.clear();
        self.collision_system.clear();
        self.time_accumulator = 0.0;
    }
}

/// Physics statistics
#[derive(Debug, Clone)]
pub struct PhysicsStatistics {
    pub body_count: usize,
    pub collider_count: usize,
    pub joint_count: usize,
    pub island_count: usize,
    pub time_accumulator: f32,
    pub gravity: Vector3<f32>,
}

impl EventHandler for JigenEventHandler {
    fn handle_collision_event(
        &self,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        event: CollisionEvent,
        _contact_pair: Option<&ContactPair>,
    ) {
        // Store collision events for processing
        // Note: We can't modify self here due to borrowing rules
        // Events are handled in the collision system
    }

    fn handle_contact_force_event(
        &self,
        _dt: Real,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        contact_pair: &ContactPair,
        total_force_magnitude: Real,
    ) {
        // Handle contact force events
        // Similar to collision events, processed in collision system
    }
}

impl PhysicsHooks for () {}
