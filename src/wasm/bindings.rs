//! JavaScript bindings for Jigen WASM module
//!
//! Provides JavaScript API for loading scenes, controlling physics,
//! and interacting with the 3D environment.

use wasm_bindgen::prelude::*;
use crate::{dsl::SceneParser, graph::SceneGraph, renderer::SceneEntityMap};
use bevy::prelude::*;
use std::sync::{Arc, Mutex};

/// Main Jigen WASM application
#[wasm_bindgen]
pub struct JigenWasmApp {
    app: App,
    scene_parser: SceneParser,
    scene_graph: Arc<Mutex<SceneGraph>>,
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

        // Insert resources
        app.insert_resource(scene_graph.clone());

        Ok(JigenWasmApp {
            app,
            scene_parser: SceneParser::new(),
            scene_graph,
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
            let stats = serde_json::json!({
                "scene_graph": sg.statistics(),
                "topology_version": sg.topology_version(),
            });
            Ok(stats.to_string())
        } else {
            Err(JsValue::from_str("Failed to lock scene graph"))
        }
    }

    /// Apply force to an object (placeholder - physics handled by Bevy Rapier3D)
    #[wasm_bindgen]
    pub fn apply_force(&mut self, _object_id: &str, _force_x: f32, _force_y: f32, _force_z: f32) {
        // Physics forces are handled by Bevy Rapier3D directly
        // This method is a placeholder for future implementation
    }

    /// Set object position (placeholder - use Bevy Rapier3D directly)
    #[wasm_bindgen]
    pub fn set_position(&mut self, _object_id: &str, _x: f32, _y: f32, _z: f32) {
        // Position setting is handled by Bevy Rapier3D directly
        // This method is a placeholder for future implementation
    }

    /// Get object position as JSON string (placeholder - use Bevy Rapier3D directly)
    #[wasm_bindgen]
    pub fn get_position(&self, _object_id: &str) -> Option<String> {
        // Position querying is handled by Bevy Rapier3D directly
        // This method is a placeholder for future implementation
        None
    }

    /// Add a spring constraint between two objects (placeholder - use Bevy Rapier3D directly)
    #[wasm_bindgen]
    pub fn add_spring(&mut self, _id: &str, _object_a: &str, _object_b: &str, _rest_length: f32, _stiffness: f32) {
        // Joint constraints are handled by Bevy Rapier3D directly
        // This method is a placeholder for future implementation
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

    /// Set gravity (placeholder - use Bevy Rapier3D configuration)
    #[wasm_bindgen]
    pub fn set_gravity(&mut self, _x: f32, _y: f32, _z: f32) {
        // Gravity is configured via RapierConfiguration resource
        // This method is a placeholder for future implementation
    }

    /// Enable/disable physics (placeholder - use Bevy Rapier3D configuration)
    #[wasm_bindgen]
    pub fn set_physics_enabled(&mut self, _enabled: bool) {
        // Physics enabling/disabling is handled by RapierConfiguration
        // This method is a placeholder for future implementation
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
        // Physics world clearing is handled by Bevy Rapier3D
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
