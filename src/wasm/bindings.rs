//! JavaScript bindings for Jigen WASM module
//!
//! Provides JavaScript API for loading scenes, controlling physics,
//! and interacting with the 3D environment.

use wasm_bindgen::prelude::*;
use crate::{dsl::SceneParser, graph::SceneGraph, physics::PhysicsWorld, renderer::SceneEntityMap};
use bevy::prelude::*;
use std::sync::{Arc, Mutex};

/// Main Jigen WASM application
#[wasm_bindgen]
pub struct JigenWasmApp {
    app: App,
    scene_parser: SceneParser,
    scene_graph: Arc<Mutex<SceneGraph>>,
    physics_world: Arc<Mutex<PhysicsWorld>>,
}

#[wasm_bindgen]
impl JigenWasmApp {
    /// Create a new Jigen WASM application
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<JigenWasmApp, JsValue> {
        console_error_panic_hook::set_once();

        // Create Bevy app with Jigen plugins
        let mut app = crate::init_jigen(crate::SceneConfig {
            title: "Jigen WASM".to_string(),
            dimensions: (800, 600),
            physics_enabled: true,
            gravity: [0.0, -9.81, 0.0],
            background_color: [0.1, 0.1, 0.1, 1.0],
        });

        let scene_graph = Arc::new(Mutex::new(SceneGraph::new()));
        let physics_world = Arc::new(Mutex::new(PhysicsWorld::new(nalgebra::Vector3::new(0.0, -9.81, 0.0))));

        // Insert resources
        app.insert_resource(scene_graph.clone());
        app.insert_resource(physics_world.clone());

        Ok(JigenWasmApp {
            app,
            scene_parser: SceneParser::new(),
            scene_graph,
            physics_world,
        })
    }

    /// Load a scene from JSON string
    #[wasm_bindgen]
    pub fn load_scene(&mut self, json_str: &str) -> Result<(), JsValue> {
        // Parse JSON
        let scene_def = self.scene_parser.parse_json(json_str)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

        // Load into scene graph
        if let Ok(mut sg) = self.scene_graph.lock() {
            sg.load_scene(&scene_def);

            // Create physics bodies for scene nodes
            if let Ok(mut pw) = self.physics_world.lock() {
                for node in &scene_def.nodes {
                    if let crate::dsl::scene::NodeProperties::Geometry(props) = &node.properties {
                        if let Some(physics) = &props.physics {
                            use rapier3d::prelude::*;

                            // Create collider shape based on geometry
                            let shape = match props.primitive {
                                crate::dsl::scene::GeometryPrimitive::Box => {
                                    ColliderBuilder::cuboid(0.5, 0.5, 0.5).build()
                                }
                                crate::dsl::scene::GeometryPrimitive::Sphere => {
                                    ColliderBuilder::ball(0.5).build()
                                }
                                _ => ColliderBuilder::cuboid(0.5, 0.5, 0.5).build(),
                            };

                            let collider_handle = pw.add_collider(node.id.clone(), shape);

                            // Create rigid body if dynamic
                            if matches!(physics.body_type, crate::dsl::scene::PhysicsBodyType::Dynamic) {
                                let rigid_body = RigidBodyBuilder::dynamic()
                                    .translation(node.transform.position.x, node.transform.position.y, node.transform.position.z)
                                    .rotation(node.transform.rotation.coords.x, node.transform.rotation.coords.y, node.transform.rotation.coords.z, node.transform.rotation.coords.w)
                                    .build();

                                let body_handle = pw.add_rigid_body(node.id.clone(), rigid_body);

                                // Attach collider to body
                                if let Some(body) = pw.rigid_body_set.get_mut(body_handle) {
                                    body.colliders().push(collider_handle);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Update the simulation (call this in requestAnimationFrame)
    #[wasm_bindgen]
    pub fn update(&mut self, delta_time: f32) {
        // Update physics
        if let Ok(mut pw) = self.physics_world.lock() {
            pw.step(delta_time);
        }

        // Update scene graph
        if let Ok(mut sg) = self.scene_graph.lock() {
            sg.update_topology();
            sg.process_incidents();
            sg.update_physics(delta_time);
        }

        // Update Bevy
        self.app.update();
    }

    /// Get scene statistics as JSON string
    #[wasm_bindgen]
    pub fn get_scene_stats(&self) -> Result<String, JsValue> {
        if let Ok(sg) = self.scene_graph.lock() {
            if let Ok(pw) = self.physics_world.lock() {
                let stats = serde_json::json!({
                    "scene_graph": sg.statistics(),
                    "physics_world": pw.statistics(),
                    "topology_version": sg.topology_version(),
                });
                Ok(stats.to_string())
            } else {
                Err(JsValue::from_str("Failed to lock physics world"))
            }
        } else {
            Err(JsValue::from_str("Failed to lock scene graph"))
        }
    }

    /// Apply force to an object
    #[wasm_bindgen]
    pub fn apply_force(&mut self, object_id: &str, force_x: f32, force_y: f32, force_z: f32) {
        if let Ok(mut pw) = self.physics_world.lock() {
            pw.apply_force(object_id, nalgebra::Vector3::new(force_x, force_y, force_z), None);
        }
    }

    /// Set object position
    #[wasm_bindgen]
    pub fn set_position(&mut self, object_id: &str, x: f32, y: f32, z: f32) {
        if let Ok(mut pw) = self.physics_world.lock() {
            pw.set_position(object_id, nalgebra::Vector3::new(x, y, z));
        }
    }

    /// Get object position as JSON string
    #[wasm_bindgen]
    pub fn get_position(&self, object_id: &str) -> Option<String> {
        if let Ok(pw) = self.physics_world.lock() {
            if let Some(pos) = pw.get_position(object_id) {
                let pos_json = serde_json::json!({
                    "x": pos.x,
                    "y": pos.y,
                    "z": pos.z
                });
                Some(pos_json.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Add a spring constraint between two objects
    #[wasm_bindgen]
    pub fn add_spring(&mut self, id: &str, object_a: &str, object_b: &str, rest_length: f32, stiffness: f32) {
        if let Ok(mut pw) = self.physics_world.lock() {
            let _ = pw.constraint_system.add_distance_constraint(
                id.to_string(),
                object_a.to_string(),
                object_b.to_string(),
                nalgebra::Vector3::zeros(), // anchor_a
                nalgebra::Vector3::zeros(), // anchor_b
                rest_length,
                stiffness,
            );
        }
    }

    /// Export current scene as JSON string
    #[wasm_bindgen]
    pub fn export_scene(&self) -> Result<String, JsValue> {
        if let Ok(sg) = self.scene_graph.lock() {
            Ok(sg.to_json().to_string())
        } else {
            Err(JsValue::from_str("Failed to lock scene graph"))
        }
    }

    /// Set gravity
    #[wasm_bindgen]
    pub fn set_gravity(&mut self, x: f32, y: f32, z: f32) {
        if let Ok(mut pw) = self.physics_world.lock() {
            pw.gravity = nalgebra::Vector3::new(x, y, z);
        }
    }

    /// Enable/disable physics
    #[wasm_bindgen]
    pub fn set_physics_enabled(&mut self, enabled: bool) {
        if let Ok(mut pw) = self.physics_world.lock() {
            pw.config.enabled = enabled;
        }
    }

    /// Get vertex count in scene graph
    #[wasm_bindgen]
    pub fn get_vertex_count(&self) -> usize {
        if let Ok(sg) = self.scene_graph.lock() {
            sg.statistics().vertex_count
        } else {
            0
        }
    }

    /// Get edge count in scene graph
    #[wasm_bindgen]
    pub fn get_edge_count(&self) -> usize {
        if let Ok(sg) = self.scene_graph.lock() {
            sg.statistics().edge_count
        } else {
            0
        }
    }

    /// Clear the scene
    #[wasm_bindgen]
    pub fn clear_scene(&mut self) {
        if let Ok(mut sg) = self.scene_graph.lock() {
            sg.clear();
        }
        if let Ok(mut pw) = self.physics_world.lock() {
            pw.clear();
        }
    }
}

/// Utility functions for JavaScript interop
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn create_minimal_scene() -> String {
    let scene = crate::dsl::parser::create_minimal_scene();
    serde_json::to_string_pretty(&scene).unwrap_or_default()
}

#[wasm_bindgen]
pub fn validate_scene_json(json_str: &str) -> Result<String, JsValue> {
    let mut parser = SceneParser::new();
    match parser.parse_json(json_str) {
        Ok(scene) => {
            let result = serde_json::json!({
                "valid": true,
                "metadata": scene.metadata,
                "node_count": scene.nodes.len(),
            });
            Ok(result.to_string())
        }
        Err(e) => {
            let result = serde_json::json!({
                "valid": false,
                "error": e.to_string(),
            });
            Ok(result.to_string())
        }
    }
}

/// Initialize WASM module
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    // Log initialization
    web_sys::console::log_1(&"Jigen WASM module initialized".into());
}

/// Performance monitoring functions
#[wasm_bindgen]
pub struct WasmPerformanceMonitor {
    monitor: crate::wasm::PerformanceMonitor,
}

#[wasm_bindgen]
impl WasmPerformanceMonitor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmPerformanceMonitor {
        WasmPerformanceMonitor {
            monitor: crate::wasm::PerformanceMonitor::new(),
        }
    }

    #[wasm_bindgen]
    pub fn update(&mut self) {
        self.monitor.update();
    }

    #[wasm_bindgen]
    pub fn get_fps(&self) -> f32 {
        self.monitor.fps()
    }

    #[wasm_bindgen]
    pub fn get_frame_time(&self) -> f32 {
        self.monitor.frame_time()
    }
}

/// Memory monitoring functions
#[wasm_bindgen]
pub struct WasmMemoryMonitor {
    monitor: crate::wasm::MemoryMonitor,
}

#[wasm_bindgen]
impl WasmMemoryMonitor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmMemoryMonitor {
        WasmMemoryMonitor {
            monitor: crate::wasm::MemoryMonitor::new(),
        }
    }

    #[wasm_bindgen]
    pub fn get_used_heap_mb(&self) -> f32 {
        (self.monitor.used_heap_size() / (1024.0 * 1024.0)) as f32
    }

    #[wasm_bindgen]
    pub fn log_memory_usage(&self) {
        self.monitor.log_memory_usage();
    }
}
