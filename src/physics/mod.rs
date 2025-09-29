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
                ..Default::default()
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
