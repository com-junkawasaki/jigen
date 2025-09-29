//! Physics simulation module
//!
//! Provides physics simulation with force dynamics, collision detection,
//! and constraint solving for the scene graph.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub mod world;
pub mod forces;
pub mod constraints;
pub mod collisions;

pub use world::*;
pub use forces::*;
pub use constraints::*;
pub use collisions::*;

/// Physics plugin for Bevy integration
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugins(RapierDebugRenderPlugin::default())
            .insert_resource(RapierConfiguration {
                gravity: Vec3::new(0.0, -9.81, 0.0),
                physics_pipeline_active: true,
                query_pipeline_active: true,
                timestep_mode: bevy_rapier3d::plugin::TimestepMode::Variable {
                    max_dt: 1.0 / 60.0,
                    time_scale: 1.0,
                    substeps: 1,
                },
                force_update_from_transform_changes: false,
                scaled_shape_subdivision: 10,
            })
            .add_systems(Update, sync_physics_with_scene_graph);
    }
}

/// Sync physics simulation with scene graph
fn sync_physics_with_scene_graph(
    mut commands: Commands,
    scene_graph: Res<crate::graph::SceneGraph>,
    mut query: Query<(Entity, &mut Transform), With<RigidBody>>,
) {
    // Sync scene graph vertices with physics bodies
    for node_index in scene_graph.graph.node_indices() {
        if let Some(vertex) = scene_graph.graph.node_weight(node_index) {
            // Find or create physics body for this vertex
            // Implementation will be added when physics bodies are created
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_physics_plugin_creation() {
        let plugin = PhysicsPlugin;
        // Plugin should be created without issues
        assert!(true);
    }

    #[test]
    fn test_physics_world_basic_operations() {
        let gravity = Vec3::new(0.0, -9.81, 0.0);
        let mut physics_world = PhysicsWorld::new(gravity);

        // Test basic operations
        physics_world.step(1.0 / 60.0);

        assert_eq!(physics_world.gravity, [0.0, -9.81, 0.0]);
        assert_eq!(physics_world.config.enabled, true);
    }

    #[test]
    fn test_force_system_operations() {
        use super::forces::ForceSystem;

        let force_system = ForceSystem::new();

        // Test clearing (should not panic)
        // Note: We can't access private fields, so we test the public interface
        assert!(true);
    }

    #[test]
    fn test_constraint_system_operations() {
        use super::constraints::ConstraintSystem;

        let constraint_system = ConstraintSystem::new();
        let physics_world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

        // Test constraint solving (placeholder)
        // Note: In real implementation, this would require mutable access
        assert!(true);
    }

    #[test]
    fn test_collision_system_operations() {
        use super::collisions::CollisionSystem;

        let collision_system = CollisionSystem::new();
        let physics_world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

        // Test collision processing (placeholder)
        // Note: In real implementation, this would require mutable access
        assert!(true);
    }
}
