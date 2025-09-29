//! Integration tests for Physics functionality

use jigen::dsl::{SceneParser, scene::*};
use jigen::graph::SceneGraph;
use jigen::physics::{PhysicsPlugin, PhysicsWorld};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde_json::json;

#[test]
fn test_physics_world_creation() {
    let gravity = Vec3::new(0.0, -9.81, 0.0);
    let physics_world = PhysicsWorld::new(gravity);

    assert_eq!(physics_world.gravity, [0.0, -9.81, 0.0]);
    assert_eq!(physics_world.config.enabled, true);
}

#[test]
fn test_physics_world_with_config() {
    let config = PhysicsConfig {
        enabled: false,
        gravity: [0.0, -20.0, 0.0],
        time_step: 1.0 / 120.0,
        max_substeps: 5,
    };

    let physics_world = PhysicsWorld::with_config(config.clone());

    assert_eq!(physics_world.gravity, [0.0, -20.0, 0.0]);
    assert_eq!(physics_world.config.enabled, false);
    assert_eq!(physics_world.config.time_step, 1.0 / 120.0);
    assert_eq!(physics_world.config.max_substeps, 5);
}

#[test]
fn test_physics_world_step() {
    let gravity = Vec3::new(0.0, -9.81, 0.0);
    let mut physics_world = PhysicsWorld::new(gravity);

    // Step physics (placeholder implementation)
    physics_world.step(1.0 / 60.0);

    // With current implementation, this should not panic
    assert_eq!(physics_world.gravity, [0.0, -9.81, 0.0]);
}

#[test]
fn test_physics_world_statistics() {
    let gravity = Vec3::new(0.0, -9.81, 0.0);
    let physics_world = PhysicsWorld::new(gravity);

    let stats = physics_world.statistics();

    assert_eq!(stats.gravity, [0.0, -9.81, 0.0]);
    // Other stats are placeholder values in current implementation
    assert_eq!(stats.body_count, 0);
    assert_eq!(stats.collider_count, 0);
    assert_eq!(stats.joint_count, 0);
}

#[test]
fn test_physics_plugin_integration() {
    // Test that PhysicsPlugin can be created without panicking
    let plugin = PhysicsPlugin;
    // Plugin creation should not panic
    assert!(true);
}

#[test]
fn test_scene_with_physics_properties() {
    let parser = SceneParser::new();

    let json_scene = json!({
        "nodes": [{
            "id": "physics_cube",
            "type": "geometry",
            "transform": {
                "position": [0.0, 5.0, 0.0]
            },
            "properties": {
                "primitive": "box",
                "physics": {
                    "body_type": "dynamic",
                    "mass": 2.0,
                    "velocity": [1.0, 0.0, 0.0],
                    "angular_velocity": [0.0, 1.0, 0.0]
                }
            }
        }]
    });

    let scene_def = parser.parse_json(&json_scene.to_string()).unwrap();

    let mut scene_graph = SceneGraph::new();
    scene_graph.load_scene(&scene_def);

    // Check that vertex has physics state
    let vertices: Vec<_> = scene_graph.graph.node_weights().collect();
    let vertex = vertices.first().unwrap();

    assert_eq!(vertex.id, "physics_cube");
    assert!(vertex.physics_state.is_some());

    let physics = vertex.physics_state.as_ref().unwrap();
    assert_eq!(physics.mass, 2.0);
    assert_eq!(physics.velocity, nalgebra::Vector3::new(1.0, 0.0, 0.0));
    assert_eq!(physics.angular_velocity, nalgebra::Vector3::new(0.0, 1.0, 0.0));
}

#[test]
fn test_force_system_basic_operations() {
    use jigen::physics::{ForceSystem, forces::GravityForce};
    use nalgebra::Vector3;

    let mut force_system = ForceSystem::new();

    // Test clearing (should not panic)
    force_system.clear();
    assert!(true);
}

#[test]
fn test_constraint_system_basic_operations() {
    use jigen::physics::{ConstraintSystem, constraints::ConstraintSystem as CS};
    use jigen::physics::world::PhysicsWorld;

    let mut constraint_system = ConstraintSystem::new();
    let mut physics_world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    // Test constraint solving (placeholder implementation)
    constraint_system.solve_constraints(&mut physics_world);

    // Test clearing
    constraint_system.clear();

    assert!(true);
}

#[test]
fn test_collision_system_basic_operations() {
    use jigen::physics::{CollisionSystem, collisions::CollisionSystem as CS};
    use jigen::physics::world::PhysicsWorld;

    let mut collision_system = CollisionSystem::new();
    let mut physics_world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    // Test collision processing (placeholder implementation)
    collision_system.process_collisions(&mut physics_world);

    // Test clearing
    collision_system.clear();

    assert!(true);
}

#[test]
fn test_physics_scene_loading() {
    let parser = SceneParser::new();

    let json_scene = json!({
        "nodes": [
            {
                "id": "ground",
                "type": "geometry",
                "transform": {
                    "position": [0.0, -1.0, 0.0],
                    "scale": [10.0, 0.1, 10.0]
                },
                "properties": {
                    "primitive": "box",
                    "physics": {
                        "body_type": "static",
                        "mass": 0.0
                    }
                }
            },
            {
                "id": "falling_cube",
                "type": "geometry",
                "transform": {
                    "position": [0.0, 5.0, 0.0]
                },
                "properties": {
                    "primitive": "box",
                    "physics": {
                        "body_type": "dynamic",
                        "mass": 1.0
                    }
                }
            }
        ],
        "forces": [{
            "type": "gravity",
            "vector": [0.0, -9.81, 0.0]
        }]
    });

    let scene_def = parser.parse_json(&json_scene.to_string()).unwrap();

    let mut scene_graph = SceneGraph::new();
    scene_graph.load_scene(&scene_def);

    assert_eq!(scene_graph.graph.node_count(), 2);

    // Check physics properties
    let vertices: Vec<_> = scene_graph.graph.node_weights().collect();
    let ground = vertices.iter().find(|v| v.id == "ground").unwrap();
    let cube = vertices.iter().find(|v| v.id == "falling_cube").unwrap();

    // Ground should be static
    let ground_physics = ground.physics_state.as_ref().unwrap();
    assert_eq!(ground_physics.mass, 0.0);

    // Cube should be dynamic
    let cube_physics = cube.physics_state.as_ref().unwrap();
    assert_eq!(cube_physics.mass, 1.0);
}
