//! Constraint system for physics simulation
//!
//! Manages various types of constraints including joints, motors, and limits.

use super::world::PhysicsWorld;
use nalgebra::Vector3;
use rapier3d::prelude::*;
use std::collections::HashMap;

/// Constraint system managing joints and limits
pub struct ConstraintSystem {
    /// Ball joints
    ball_joints: HashMap<String, BallJointConstraint>,
    /// Fixed joints
    fixed_joints: HashMap<String, FixedJointConstraint>,
    /// Distance constraints
    distance_constraints: HashMap<String, DistanceConstraint>,
    /// Hinge joints
    hinge_joints: HashMap<String, HingeJointConstraint>,
    /// Prismatic joints (sliding)
    prismatic_joints: HashMap<String, PrismaticJointConstraint>,
}

/// Ball joint constraint (spherical joint)
pub struct BallJointConstraint {
    /// Body A ID
    body_a: String,
    /// Body B ID
    body_b: String,
    /// Anchor point in body A local space
    anchor_a: Vector3<f32>,
    /// Anchor point in body B local space
    anchor_b: Vector3<f32>,
    /// Joint handle in Rapier
    joint_handle: Option<JointHandle>,
}

/// Fixed joint constraint (rigid connection)
pub struct FixedJointConstraint {
    /// Body A ID
    body_a: String,
    /// Body B ID
    body_b: String,
    /// Anchor point in body A local space
    anchor_a: Vector3<f32>,
    /// Anchor point in body B local space
    anchor_b: Vector3<f32>,
    /// Joint handle in Rapier
    joint_handle: Option<JointHandle>,
}

/// Distance constraint
pub struct DistanceConstraint {
    /// Body A ID
    body_a: String,
    /// Body B ID
    body_b: String,
    /// Anchor point in body A local space
    anchor_a: Vector3<f32>,
    /// Anchor point in body B local space
    anchor_b: Vector3<f32>,
    /// Rest length
    rest_length: f32,
    /// Stiffness (0 = soft, 1 = rigid)
    stiffness: f32,
    /// Joint handle in Rapier
    joint_handle: Option<JointHandle>,
}

/// Hinge joint constraint (rotates around axis)
pub struct HingeJointConstraint {
    /// Body A ID
    body_a: String,
    /// Body B ID
    body_b: String,
    /// Anchor point in body A local space
    anchor_a: Vector3<f32>,
    /// Anchor point in body B local space
    anchor_b: Vector3<f32>,
    /// Axis of rotation in body A local space
    axis: Vector3<f32>,
    /// Joint handle in Rapier
    joint_handle: Option<JointHandle>,
}

/// Prismatic joint constraint (linear motion along axis)
pub struct PrismaticJointConstraint {
    /// Body A ID
    body_a: String,
    /// Body B ID
    body_b: String,
    /// Anchor point in body A local space
    anchor_a: Vector3<f32>,
    /// Anchor point in body B local space
    anchor_b: Vector3<f32>,
    /// Axis of motion in body A local space
    axis: Vector3<f32>,
    /// Joint handle in Rapier
    joint_handle: Option<JointHandle>,
}

impl ConstraintSystem {
    /// Create a new constraint system
    pub fn new() -> Self {
        Self {
            ball_joints: HashMap::new(),
            fixed_joints: HashMap::new(),
            distance_constraints: HashMap::new(),
            hinge_joints: HashMap::new(),
            prismatic_joints: HashMap::new(),
        }
    }

    /// Solve constraints (called after physics step)
    pub fn solve_constraints(&mut self, world: &mut PhysicsWorld) {
        // Rapier handles constraint solving internally
        // This method is for post-processing or custom constraints
    }

    /// Add ball joint constraint
    pub fn add_ball_joint(&mut self, id: String, body_a: String, body_b: String, anchor_a: Vector3<f32>, anchor_b: Vector3<f32>) -> Result<(), String> {
        // Get rigid body handles
        let handle_a = world.get_rigid_body(&body_a).ok_or("Body A not found")?.0;
        let handle_b = world.get_rigid_body(&body_b).ok_or("Body B not found")?.0;

        // Create Rapier joint
        let joint = BallJoint::new(anchor_a.into(), anchor_b.into());
        let joint_handle = world.joint_set.insert(handle_a, handle_b, joint, true);

        let constraint = BallJointConstraint {
            body_a,
            body_b,
            anchor_a,
            anchor_b,
            joint_handle: Some(joint_handle),
        };

        self.ball_joints.insert(id, constraint);
        Ok(())
    }

    /// Add fixed joint constraint
    pub fn add_fixed_joint(&mut self, id: String, body_a: String, body_b: String, anchor_a: Vector3<f32>, anchor_b: Vector3<f32>) -> Result<(), String> {
        let handle_a = world.get_rigid_body(&body_a).ok_or("Body A not found")?.0;
        let handle_b = world.get_rigid_body(&body_b).ok_or("Body B not found")?.0;

        let joint = FixedJoint::new(anchor_a.into(), anchor_b.into());
        let joint_handle = world.joint_set.insert(handle_a, handle_b, joint, true);

        let constraint = FixedJointConstraint {
            body_a,
            body_b,
            anchor_a,
            anchor_b,
            joint_handle: Some(joint_handle),
        };

        self.fixed_joints.insert(id, constraint);
        Ok(())
    }

    /// Add distance constraint
    pub fn add_distance_constraint(&mut self, id: String, body_a: String, body_b: String, anchor_a: Vector3<f32>, anchor_b: Vector3<f32>, rest_length: f32, stiffness: f32) -> Result<(), String> {
        let handle_a = world.get_rigid_body(&body_a).ok_or("Body A not found")?.0;
        let handle_b = world.get_rigid_body(&body_b).ok_or("Body B not found")?.0;

        // Calculate current distance as default rest length if not specified
        let current_rest_length = if rest_length > 0.0 {
            rest_length
        } else {
            let pos_a = world.get_position(&body_a).unwrap_or(anchor_a);
            let pos_b = world.get_position(&body_b).unwrap_or(anchor_b);
            (pos_b - pos_a).magnitude()
        };

        let joint = PrismaticJoint::new(
            anchor_a.into(),
            anchor_b.into(),
            Vector::x_axis(), // Distance constraint axis doesn't matter for distance
            0.0, // limits
            0.0,
            0.0, // motor
            stiffness,
            0.0,
        );

        // Actually use a spring joint for distance constraints
        let joint = SpringJoint::new(
            anchor_a.into(),
            anchor_b.into(),
            current_rest_length,
            stiffness,
        );

        let joint_handle = world.joint_set.insert(handle_a, handle_b, joint, true);

        let constraint = DistanceConstraint {
            body_a,
            body_b,
            anchor_a,
            anchor_b,
            rest_length: current_rest_length,
            stiffness,
            joint_handle: Some(joint_handle),
        };

        self.distance_constraints.insert(id, constraint);
        Ok(())
    }

    /// Add hinge joint constraint
    pub fn add_hinge_joint(&mut self, id: String, body_a: String, body_b: String, anchor_a: Vector3<f32>, anchor_b: Vector3<f32>, axis: Vector3<f32>) -> Result<(), String> {
        let handle_a = world.get_rigid_body(&body_a).ok_or("Body A not found")?.0;
        let handle_b = world.get_rigid_body(&body_b).ok_or("Body B not found")?.0;

        let joint = RevoluteJoint::new(anchor_a.into(), anchor_b.into(), axis.into());
        let joint_handle = world.joint_set.insert(handle_a, handle_b, joint, true);

        let constraint = HingeJointConstraint {
            body_a,
            body_b,
            anchor_a,
            anchor_b,
            axis,
            joint_handle: Some(joint_handle),
        };

        self.hinge_joints.insert(id, constraint);
        Ok(())
    }

    /// Add prismatic joint constraint
    pub fn add_prismatic_joint(&mut self, id: String, body_a: String, body_b: String, anchor_a: Vector3<f32>, anchor_b: Vector3<f32>, axis: Vector3<f32>) -> Result<(), String> {
        let handle_a = world.get_rigid_body(&body_a).ok_or("Body A not found")?.0;
        let handle_b = world.get_rigid_body(&body_b).ok_or("Body B not found")?.0;

        let joint = PrismaticJoint::new(anchor_a.into(), anchor_b.into(), axis.into(), 0.0, 0.0, 0.0, 1.0, 0.0);
        let joint_handle = world.joint_set.insert(handle_a, handle_b, joint, true);

        let constraint = PrismaticJointConstraint {
            body_a,
            body_b,
            anchor_a,
            anchor_b,
            axis,
            joint_handle: Some(joint_handle),
        };

        self.prismatic_joints.insert(id, constraint);
        Ok(())
    }

    /// Remove constraint by ID
    pub fn remove_constraint(&mut self, id: &str, world: &mut PhysicsWorld) -> bool {
        // Check all constraint types
        if let Some(constraint) = self.ball_joints.remove(id) {
            if let Some(handle) = constraint.joint_handle {
                world.joint_set.remove(handle, true);
            }
            return true;
        }

        if let Some(constraint) = self.fixed_joints.remove(id) {
            if let Some(handle) = constraint.joint_handle {
                world.joint_set.remove(handle, true);
            }
            return true;
        }

        if let Some(constraint) = self.distance_constraints.remove(id) {
            if let Some(handle) = constraint.joint_handle {
                world.joint_set.remove(handle, true);
            }
            return true;
        }

        if let Some(constraint) = self.hinge_joints.remove(id) {
            if let Some(handle) = constraint.joint_handle {
                world.joint_set.remove(handle, true);
            }
            return true;
        }

        if let Some(constraint) = self.prismatic_joints.remove(id) {
            if let Some(handle) = constraint.joint_handle {
                world.joint_set.remove(handle, true);
            }
            return true;
        }

        false
    }

    /// Get constraint statistics
    pub fn statistics(&self) -> ConstraintStatistics {
        ConstraintStatistics {
            ball_joints: self.ball_joints.len(),
            fixed_joints: self.fixed_joints.len(),
            distance_constraints: self.distance_constraints.len(),
            hinge_joints: self.hinge_joints.len(),
            prismatic_joints: self.prismatic_joints.len(),
            total_constraints: self.total_count(),
        }
    }

    /// Get total constraint count
    pub fn total_count(&self) -> usize {
        self.ball_joints.len() +
        self.fixed_joints.len() +
        self.distance_constraints.len() +
        self.hinge_joints.len() +
        self.prismatic_joints.len()
    }

    /// Clear all constraints
    pub fn clear(&mut self) {
        self.ball_joints.clear();
        self.fixed_joints.clear();
        self.distance_constraints.clear();
        self.hinge_joints.clear();
        self.prismatic_joints.clear();
    }

    /// Get ball joint by ID
    pub fn get_ball_joint(&self, id: &str) -> Option<&BallJointConstraint> {
        self.ball_joints.get(id)
    }

    /// Get fixed joint by ID
    pub fn get_fixed_joint(&self, id: &str) -> Option<&FixedJointConstraint> {
        self.fixed_joints.get(id)
    }

    /// Get distance constraint by ID
    pub fn get_distance_constraint(&self, id: &str) -> Option<&DistanceConstraint> {
        self.distance_constraints.get(id)
    }

    /// Get hinge joint by ID
    pub fn get_hinge_joint(&self, id: &str) -> Option<&HingeJointConstraint> {
        self.hinge_joints.get(id)
    }

    /// Get prismatic joint by ID
    pub fn get_prismatic_joint(&self, id: &str) -> Option<&PrismaticJointConstraint> {
        self.prismatic_joints.get(id)
    }
}

/// Constraint statistics
#[derive(Debug, Clone)]
pub struct ConstraintStatistics {
    pub ball_joints: usize,
    pub fixed_joints: usize,
    pub distance_constraints: usize,
    pub hinge_joints: usize,
    pub prismatic_joints: usize,
    pub total_constraints: usize,
}

/// Motor for joints (can be added to joints for powered motion)
pub struct JointMotor {
    /// Target velocity
    pub target_velocity: f32,
    /// Maximum force/torque
    pub max_force: f32,
    /// Motor type
    pub motor_type: MotorType,
}

/// Motor types
pub enum MotorType {
    /// Velocity motor (maintains target velocity)
    Velocity,
    /// Position motor (moves to target position)
    Position,
}

impl JointMotor {
    /// Create velocity motor
    pub fn velocity(target_velocity: f32, max_force: f32) -> Self {
        Self {
            target_velocity,
            max_force,
            motor_type: MotorType::Velocity,
        }
    }

    /// Create position motor
    pub fn position(target_position: f32, max_force: f32) -> Self {
        Self {
            target_velocity: target_position,
            max_force,
            motor_type: MotorType::Position,
        }
    }
}
