//! Edge case and boundary condition tests
//!
//! Tests for:
//! - Empty and minimal scenes
//! - Extreme values and edge conditions
//! - Error recovery and resilience
//! - Invalid inputs and malformed data
//! - Concurrent access scenarios
//! - Resource limits and memory constraints

use jigen::dsl::{SceneParser, scene::*};
use jigen::graph::{SceneGraph, vertex::Vertex, edge::Edge, incident::Incident};
use serde_json::json;
use nalgebra::{Vector3, Quaternion};

/// Test empty scene handling
#[test]
fn test_empty_scene() {
    let parser = SceneParser::new();

    let empty_scene = json!({
        "metadata": {
            "version": "1.0",
            "title": "Empty Scene"
        },
        "nodes": []
    });

    let scene_def = parser.parse_json(&empty_scene.to_string()).unwrap();

    assert_eq!(scene_def.nodes.len(), 0);
    assert_eq!(scene_def.metadata.title, "Empty Scene");

    // Load into SceneGraph
    let mut scene_graph = SceneGraph::new();
    scene_graph.load_scene(&scene_def);

    assert_eq!(scene_graph.graph.node_count(), 0);
    assert_eq!(scene_graph.graph.edge_count(), 0);

    let stats = scene_graph.statistics();
    assert_eq!(stats.node_count, 0);
    assert_eq!(stats.edge_count, 0);
}

/// Test minimal valid scene
#[test]
fn test_minimal_scene() {
    let parser = SceneParser::new();

    let minimal_scene = json!({
        "nodes": [{
            "id": "single_node",
            "type": "geometry"
        }]
    });

    let scene_def = parser.parse_json(&minimal_scene.to_string()).unwrap();

    assert_eq!(scene_def.nodes.len(), 1);
    assert_eq!(scene_def.nodes[0].id, "single_node");

    // Load into SceneGraph
    let mut scene_graph = SceneGraph::new();
    scene_graph.load_scene(&scene_def);

    assert_eq!(scene_graph.graph.node_count(), 1);
}

/// Test extreme values
#[test]
fn test_extreme_values() {
    let parser = SceneParser::new();

    let extreme_scene = json!({
        "nodes": [{
            "id": "extreme_node",
            "type": "geometry",
            "transform": {
                "position": [f32::MAX, f32::MIN, 0.0],
                "rotation": [0.0, 0.0, 0.0, 1.0],
                "scale": [f32::INFINITY, f32::NEG_INFINITY, f32::NAN]
            },
            "properties": {
                "primitive": "box",
                "physics": {
                    "body_type": "dynamic",
                    "mass": f32::MAX
                }
            }
        }]
    });

    // This should parse without panicking, though values may be sanitized
    let result = parser.parse_json(&extreme_scene.to_string());
    assert!(result.is_ok());
}

/// Test very large scenes
#[test]
fn test_large_scene() {
    let parser = SceneParser::new();

    // Create a scene with 1000 nodes
    let mut nodes = Vec::new();
    for i in 0..1000 {
        nodes.push(json!({
            "id": format!("node_{}", i),
            "type": "geometry",
            "properties": {"primitive": "box"}
        }));
    }

    let large_scene = json!({
        "nodes": nodes
    });

    let scene_def = parser.parse_json(&large_scene.to_string()).unwrap();
    assert_eq!(scene_def.nodes.len(), 1000);

    // Load into SceneGraph
    let mut scene_graph = SceneGraph::new();
    scene_graph.load_scene(&scene_def);

    assert_eq!(scene_graph.graph.node_count(), 1000);

    // Test topology update with large scene
    scene_graph.update_topology();

    let stats = scene_graph.statistics();
    assert_eq!(stats.node_count, 1000);
}

/// Test very deep hierarchy
#[test]
fn test_deep_hierarchy() {
    let parser = SceneParser::new();

    // Create a deep hierarchy (10 levels)
    let mut nodes = Vec::new();
    let mut parent_id = None;

    for level in 0..10 {
        let node_id = format!("level_{}", level);
        let mut node = json!({
            "id": node_id,
            "type": if level == 0 { "group" } else { "geometry" },
            "properties": if level > 0 { json!({"primitive": "box"}) } else { json!(null) }
        });

        if let Some(parent) = parent_id {
            node["children"] = json!([format!("level_{}", level)]);
        }

        nodes.push(node);
        parent_id = Some(node_id);
    }

    let deep_scene = json!({
        "nodes": nodes
    });

    let scene_def = parser.parse_json(&deep_scene.to_string()).unwrap();
    assert_eq!(scene_def.nodes.len(), 10);

    // Load into SceneGraph
    let mut scene_graph = SceneGraph::new();
    scene_graph.load_scene(&scene_def);

    assert_eq!(scene_graph.graph.node_count(), 10);
}

/// Test circular references (should be handled gracefully)
#[test]
fn test_circular_references() {
    let parser = SceneParser::new();

    // Create nodes with circular parent-child references
    let circular_scene = json!({
        "nodes": [
            {
                "id": "node_a",
                "type": "group",
                "children": ["node_b"]
            },
            {
                "id": "node_b",
                "type": "group",
                "children": ["node_a"]  // Circular reference
            }
        ]
    });

    let scene_def = parser.parse_json(&circular_scene.to_string()).unwrap();

    // Load into SceneGraph (should handle circular references gracefully)
    let mut scene_graph = SceneGraph::new();
    scene_graph.load_scene(&scene_def);

    assert_eq!(scene_graph.graph.node_count(), 2);
}

/// Test duplicate IDs
#[test]
fn test_duplicate_ids() {
    let parser = SceneParser::new();

    let duplicate_scene = json!({
        "nodes": [
            {
                "id": "duplicate",
                "type": "geometry",
                "properties": {"primitive": "box"}
            },
            {
                "id": "duplicate",  // Same ID
                "type": "geometry",
                "properties": {"primitive": "sphere"}
            }
        ]
    });

    let scene_def = parser.parse_json(&duplicate_scene.to_string()).unwrap();

    // Should parse successfully, but SceneGraph may handle duplicates
    let mut scene_graph = SceneGraph::new();
    scene_graph.load_scene(&scene_def);

    // SceneGraph should handle this gracefully (may overwrite or skip)
    assert!(scene_graph.graph.node_count() >= 1);
}

/// Test invalid JSON structures
#[test]
fn test_invalid_json_structures() {
    let parser = SceneParser::new();

    // Test various invalid structures
    let invalid_cases = vec![
        json!(null),
        json!([]),
        json!("string"),
        json!(42),
        json!({}),  // Empty object
        json!({"nodes": null}),
        json!({"nodes": "invalid"}),
        json!({"nodes": [null]}),
        json!({"nodes": [{"id": null}]}),
        json!({"nodes": [{"type": null}]}),
    ];

    for invalid_json in invalid_cases {
        let result = parser.parse_json(&invalid_json.to_string());
        // Some may succeed (graceful degradation), some may fail
        // The important thing is no panics
        let _ = result; // Just ensure it doesn't panic
    }
}

/// Test malformed physics properties
#[test]
fn test_malformed_physics_properties() {
    let parser = SceneParser::new();

    let malformed_physics = json!({
        "nodes": [{
            "id": "test",
            "type": "geometry",
            "properties": {
                "primitive": "box",
                "physics": {
                    "body_type": "invalid_type",
                    "mass": "not_a_number",
                    "velocity": "also_not_a_number"
                }
            }
        }]
    });

    // Should parse without panicking, though physics may be ignored
    let result = parser.parse_json(&malformed_physics.to_string());
    assert!(result.is_ok());
}

/// Test concurrent access scenarios (basic)
#[test]
fn test_concurrent_scene_access() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let parser = SceneParser::new();
    let scene_json = json!({
        "nodes": [{
            "id": "shared_node",
            "type": "geometry",
            "properties": {"primitive": "box"}
        }]
    });

    let scene_def = Arc::new(Mutex::new(
        parser.parse_json(&scene_json.to_string()).unwrap()
    ));

    let mut handles = vec![];

    // Spawn multiple threads trying to access the scene
    for i in 0..5 {
        let scene_clone = Arc::clone(&scene_def);
        let handle = thread::spawn(move || {
            let scene = scene_clone.lock().unwrap();
            assert_eq!(scene.nodes.len(), 1);
            assert_eq!(scene.nodes[0].id, "shared_node");
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
}

/// Test memory exhaustion scenarios (basic)
#[test]
fn test_memory_limits() {
    let parser = SceneParser::new();

    // Create a scene with very large strings to test memory handling
    let large_string = "x".repeat(100000); // 100KB string

    let memory_scene = json!({
        "metadata": {
            "title": large_string
        },
        "nodes": [{
            "id": "test",
            "type": "geometry",
            "properties": {"primitive": "box"}
        }]
    });

    // Should handle large strings without issues
    let result = parser.parse_json(&memory_scene.to_string());
    assert!(result.is_ok());
}

/// Test boundary conditions for numerical values
#[test]
fn test_numerical_boundaries() {
    let parser = SceneParser::new();

    let boundary_scene = json!({
        "nodes": [{
            "id": "boundary_test",
            "type": "geometry",
            "transform": {
                "position": [0.0, 0.0, 0.0],
                "rotation": [0.0, 0.0, 0.0, 1.0],
                "scale": [0.0, 0.0, 0.0]  // Zero scale
            },
            "properties": {
                "primitive": "box",
                "physics": {
                    "mass": 0.0  // Zero mass
                }
            }
        }]
    });

    let scene_def = parser.parse_json(&boundary_scene.to_string()).unwrap();
    assert_eq!(scene_def.nodes.len(), 1);

    // Load into SceneGraph
    let mut scene_graph = SceneGraph::new();
    scene_graph.load_scene(&scene_def);

    assert_eq!(scene_graph.graph.node_count(), 1);
}

/// Test rapid scene loading/unloading
#[test]
fn test_rapid_scene_operations() {
    let parser = SceneParser::new();

    for i in 0..10 {
        let scene_json = json!({
            "metadata": {
                "title": format!("Rapid Test {}", i)
            },
            "nodes": [{
                "id": format!("rapid_node_{}", i),
                "type": "geometry",
                "properties": {"primitive": "box"}
            }]
        });

        let scene_def = parser.parse_json(&scene_json.to_string()).unwrap();

        let mut scene_graph = SceneGraph::new();
        scene_graph.load_scene(&scene_def);

        assert_eq!(scene_graph.graph.node_count(), 1);

        // Immediately clear
        scene_graph.clear();
        assert_eq!(scene_graph.graph.node_count(), 0);
    }
}

/// Test incident system edge cases
#[test]
fn test_incident_edge_cases() {
    let mut scene_graph = SceneGraph::new();

    // Test with empty incident manager
    assert_eq!(scene_graph.incident_manager.active_count(), 0);
    assert_eq!(scene_graph.incident_manager.completed_count(), 0);

    // Add incident with empty affected objects
    let empty_incident = Incident::new(
        "empty_incident".to_string(),
        jigen::graph::incident::IncidentType::StateChange,
        vec![],
    );

    scene_graph.incident_manager.add_incident(empty_incident);
    assert_eq!(scene_graph.incident_manager.active_count(), 1);

    // Process with empty vertex/edge lists
    let empty_vertices: Vec<&Vertex> = vec![];
    let empty_edges: Vec<&Edge> = vec![];
    scene_graph.incident_manager.process_incidents(&empty_vertices, &empty_edges);

    assert_eq!(scene_graph.incident_manager.active_count(), 0);
}

/// Test SceneGraph operations on empty graph
#[test]
fn test_empty_scene_graph_operations() {
    let mut scene_graph = SceneGraph::new();

    // Test operations on empty graph
    scene_graph.update_topology();
    let stats = scene_graph.statistics();
    let exported = scene_graph.export_scene();

    assert_eq!(stats.node_count, 0);
    assert_eq!(stats.edge_count, 0);
    assert!(!exported.is_empty()); // Should still produce valid JSON

    // Clear empty graph
    scene_graph.clear();
    assert_eq!(scene_graph.graph.node_count(), 0);
}

/// Test very long identifiers
#[test]
fn test_long_identifiers() {
    let parser = SceneParser::new();

    let long_id = "a".repeat(1000); // Very long ID

    let long_id_scene = json!({
        "nodes": [{
            "id": long_id,
            "type": "geometry",
            "properties": {"primitive": "box"}
        }]
    });

    let scene_def = parser.parse_json(&long_id_scene.to_string()).unwrap();
    assert_eq!(scene_def.nodes.len(), 1);
    assert_eq!(scene_def.nodes[0].id, long_id);
}

/// Test special characters in identifiers
#[test]
fn test_special_characters_in_ids() {
    let parser = SceneParser::new();

    let special_id_scene = json!({
        "nodes": [{
            "id": "special_chars_!@#$%^&*()",
            "type": "geometry",
            "properties": {"primitive": "box"}
        }]
    });

    let scene_def = parser.parse_json(&special_id_scene.to_string()).unwrap();
    assert_eq!(scene_def.nodes.len(), 1);
    assert_eq!(scene_def.nodes[0].id, "special_chars_!@#$%^&*()");
}

/// Test Unicode identifiers
#[test]
fn test_unicode_identifiers() {
    let parser = SceneParser::new();

    let unicode_scene = json!({
        "nodes": [{
            "id": "テスト_シーン_オブジェクト",
            "type": "geometry",
            "properties": {"primitive": "box"}
        }]
    });

    let scene_def = parser.parse_json(&unicode_scene.to_string()).unwrap();
    assert_eq!(scene_def.nodes.len(), 1);
    assert_eq!(scene_def.nodes[0].id, "テスト_シーン_オブジェクト");
}
