//! Integration tests for DSL parser functionality

use jigen::dsl::{SceneParser, scene::*};
use serde_json::json;

#[test]
fn test_basic_scene_parsing() {
    let parser = SceneParser::new();

    let json_scene = json!({
        "metadata": {
            "version": "1.0",
            "title": "Test Scene"
        },
        "nodes": [{
            "id": "cube",
            "type": "geometry",
            "transform": {
                "position": [0.0, 1.0, 0.0],
                "rotation": [0.0, 0.0, 0.0, 1.0],
                "scale": [1.0, 1.0, 1.0]
            },
            "properties": {
                "primitive": "box",
                "material": {
                    "color": [1.0, 0.0, 0.0, 1.0]
                },
                "physics": {
                    "body_type": "dynamic",
                    "mass": 1.0
                }
            }
        }]
    });

    let scene_def = parser.parse_json(&json_scene.to_string()).unwrap();

    assert_eq!(scene_def.metadata.title, "Test Scene");
    assert_eq!(scene_def.nodes.len(), 1);

    let node = &scene_def.nodes[0];
    assert_eq!(node.id, "cube");
    assert_eq!(node.node_type, NodeType::Geometry);
    assert_eq!(node.transform.position, [0.0, 1.0, 0.0]);

    match &node.properties {
        NodeProperties::Geometry(props) => {
            assert_eq!(props.primitive, GeometryPrimitive::Box);
            if let Some(physics) = &props.physics {
                assert_eq!(physics.body_type, PhysicsBodyType::Dynamic);
                assert_eq!(physics.mass, 1.0);
            } else {
                panic!("Expected physics properties");
            }
        }
        _ => panic!("Expected geometry properties"),
    }
}

#[test]
fn test_scene_with_parent_child_relationships() {
    let parser = SceneParser::new();

    let json_scene = json!({
        "nodes": [
            {
                "id": "parent",
                "type": "group",
                "transform": {
                    "position": [0.0, 0.0, 0.0]
                },
                "children": ["child1", "child2"]
            },
            {
                "id": "child1",
                "type": "geometry",
                "transform": {
                    "position": [1.0, 0.0, 0.0]
                },
                "properties": {
                    "primitive": "sphere"
                }
            },
            {
                "id": "child2",
                "type": "geometry",
                "transform": {
                    "position": [-1.0, 0.0, 0.0]
                },
                "properties": {
                    "primitive": "box"
                }
            }
        ]
    });

    let scene_def = parser.parse_json(&json_scene.to_string()).unwrap();

    assert_eq!(scene_def.nodes.len(), 3);

    // Find parent node
    let parent = scene_def.nodes.iter().find(|n| n.id == "parent").unwrap();
    assert_eq!(parent.children, vec!["child1", "child2"]);

    // Find child nodes
    let child1 = scene_def.nodes.iter().find(|n| n.id == "child1").unwrap();
    let child2 = scene_def.nodes.iter().find(|n| n.id == "child2").unwrap();

    assert_eq!(child1.transform.position, [1.0, 0.0, 0.0]);
    assert_eq!(child2.transform.position, [-1.0, 0.0, 0.0]);
}

#[test]
fn test_scene_with_forces_and_constraints() {
    let parser = SceneParser::new();

    let json_scene = json!({
        "nodes": [
            {
                "id": "body1",
                "type": "geometry",
                "properties": {
                    "primitive": "box",
                    "physics": {
                        "body_type": "dynamic",
                        "mass": 1.0
                    }
                }
            },
            {
                "id": "body2",
                "type": "geometry",
                "properties": {
                    "primitive": "sphere",
                    "physics": {
                        "body_type": "dynamic",
                        "mass": 2.0
                    }
                }
            }
        ],
        "forces": [{
            "type": "gravity",
            "vector": [0.0, -9.81, 0.0],
            "affected_nodes": ["body1", "body2"]
        }],
        "constraints": [{
            "type": "distance",
            "body_a": "body1",
            "body_b": "body2",
            "rest_length": 2.0,
            "stiffness": 100.0
        }]
    });

    let scene_def = parser.parse_json(&json_scene.to_string()).unwrap();

    assert_eq!(scene_def.forces.len(), 1);
    assert_eq!(scene_def.constraints.len(), 1);

    let force = &scene_def.forces[0];
    assert_eq!(force.vector, [0.0, -9.81, 0.0]);
    assert_eq!(force.affected_nodes, vec!["body1", "body2"]);

    let constraint = &scene_def.constraints[0];
    assert_eq!(constraint.body_a, "body1");
    assert_eq!(constraint.body_b, "body2");
    assert_eq!(constraint.rest_length, Some(2.0));
    assert_eq!(constraint.stiffness, Some(100.0));
}

#[test]
fn test_invalid_json_handling() {
    let parser = SceneParser::new();

    // Test invalid JSON
    let result = parser.parse_json("invalid json");
    assert!(result.is_err());

    // Test missing required fields
    let invalid_scene = json!({
        "nodes": [{
            // Missing id
            "type": "geometry"
        }]
    });

    let result = parser.parse_json(&invalid_scene.to_string());
    assert!(result.is_err());
}

/// Test edge cases in DSL parsing
#[test]
fn test_dsl_parsing_edge_cases() {
    let parser = SceneParser::new();

    // Test empty scene
    let empty_scene = json!({
        "nodes": []
    });
    let result = parser.parse_json(&empty_scene.to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().nodes.len(), 0);

    // Test single node scene
    let single_node = json!({
        "nodes": [{
            "id": "single",
            "type": "geometry"
        }]
    });
    let result = parser.parse_json(&single_node.to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().nodes.len(), 1);

    // Test malformed JSON
    let malformed_cases = vec![
        "{invalid json",
        "",
        "{}",
        "[]",
        "null",
        "{\"nodes\": invalid}",
    ];

    for malformed in malformed_cases {
        let result = parser.parse_json(malformed);
        // Should either succeed with graceful degradation or fail safely
        let _ = result; // Just ensure no panics
    }

    // Test extreme values
    let extreme_scene = json!({
        "nodes": [{
            "id": "extreme",
            "type": "geometry",
            "transform": {
                "position": [f32::MAX, f32::MIN, f32::INFINITY],
                "rotation": [0.0, 0.0, 0.0, 1.0],
                "scale": [f32::NAN, 0.0, -f32::INFINITY]
            }
        }]
    });
    let result = parser.parse_json(&extreme_scene.to_string());
    assert!(result.is_ok()); // Should handle gracefully

    // Test very large scene (1000 nodes)
    let mut large_nodes = Vec::new();
    for i in 0..1000 {
        large_nodes.push(json!({
            "id": format!("node_{}", i),
            "type": "geometry",
            "properties": {"primitive": "box"}
        }));
    }
    let large_scene = json!({
        "nodes": large_nodes
    });
    let result = parser.parse_json(&large_scene.to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().nodes.len(), 1000);
}

/// Test concurrent parsing scenarios
#[test]
fn test_concurrent_parsing() {
    use std::sync::Arc;
    use std::thread;

    let parser = Arc::new(SceneParser::new());
    let scene_json = Arc::new(json!({
        "nodes": [{
            "id": "concurrent_test",
            "type": "geometry",
            "properties": {"primitive": "box"}
        }]
    }).to_string());

    let mut handles = vec![];

    // Spawn multiple threads parsing the same scene
    for _ in 0..10 {
        let parser_clone = Arc::clone(&parser);
        let json_clone = Arc::clone(&scene_json);

        let handle = thread::spawn(move || {
            let result = parser_clone.parse_json(&json_clone);
            assert!(result.is_ok());
            let scene = result.unwrap();
            assert_eq!(scene.nodes.len(), 1);
            assert_eq!(scene.nodes[0].id, "concurrent_test");
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
}

/// Test parsing performance with different scene sizes
#[test]
fn test_parsing_performance_scaling() {
    use std::time::Instant;

    let parser = SceneParser::new();

    // Test parsing time scaling with scene size
    let sizes = vec![10, 50, 100, 200];

    let mut previous_time = 0u128;

    for &size in &sizes {
        let mut nodes = Vec::new();
        for i in 0..size {
            nodes.push(json!({
                "id": format!("perf_node_{}", i),
                "type": "geometry",
                "properties": {"primitive": "box"},
                "transform": {
                    "position": [i as f32, 0.0, 0.0]
                }
            }));
        }

        let scene = json!({
            "nodes": nodes
        });

        let json_str = scene.to_string();
        let start = Instant::now();
        let result = parser.parse_json(&json_str);
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert_eq!(result.unwrap().nodes.len(), size);

        // Basic scaling check - parsing should scale reasonably with size
        // (allowing some overhead for JSON parsing itself)
        if previous_time > 0 {
            let ratio = elapsed.as_micros() as f64 / previous_time as f64;
            let size_ratio = size as f64 / (size - sizes[0]) as f64;

            // Should scale roughly linearly (with some tolerance)
            assert!(ratio < size_ratio * 2.0,
                "Performance scaling issue: size {}, time ratio {:.2}, size ratio {:.2}",
                size, ratio, size_ratio);
        }

        previous_time = elapsed.as_micros();
    }
}

#[test]
fn test_complex_scene_with_all_features() {
    let parser = SceneParser::new();

    let json_scene = json!({
        "metadata": {
            "version": "1.0",
            "title": "Complex Test Scene",
            "description": "A comprehensive test scene"
        },
        "camera": {
            "position": [0.0, 5.0, 10.0],
            "target": [0.0, 0.0, 0.0],
            "up": [0.0, 1.0, 0.0],
            "fov": 60.0
        },
        "lights": [{
            "type": "directional",
            "direction": [1.0, -1.0, 1.0],
            "color": [1.0, 1.0, 0.9],
            "intensity": 1000.0
        }],
        "nodes": [
            {
                "id": "floor",
                "type": "geometry",
                "transform": {
                    "position": [0.0, -1.0, 0.0],
                    "scale": [10.0, 0.1, 10.0]
                },
                "properties": {
                    "primitive": "box",
                    "material": {
                        "color": [0.8, 0.8, 0.8, 1.0]
                    },
                    "physics": {
                        "body_type": "static",
                        "mass": 0.0
                    }
                }
            },
            {
                "id": "ball",
                "type": "geometry",
                "transform": {
                    "position": [0.0, 5.0, 0.0]
                },
                "properties": {
                    "primitive": "sphere",
                    "material": {
                        "color": [1.0, 0.0, 0.0, 1.0]
                    },
                    "physics": {
                        "body_type": "dynamic",
                        "mass": 1.0,
                        "velocity": [2.0, 0.0, 0.0]
                    }
                }
            }
        ],
        "forces": [{
            "type": "gravity",
            "vector": [0.0, -9.81, 0.0]
        }],
        "settings": {
            "background_color": [0.1, 0.1, 0.2, 1.0],
            "shadows": true,
            "physics": {
                "enabled": true,
                "gravity": [0.0, -9.81, 0.0],
                "time_step": 0.016
            }
        }
    });

    let scene_def = parser.parse_json(&json_scene.to_string()).unwrap();

    // Test metadata
    assert_eq!(scene_def.metadata.title, "Complex Test Scene");

    // Test camera
    assert_eq!(scene_def.camera.as_ref().unwrap().position, [0.0, 5.0, 10.0]);

    // Test lights
    assert_eq!(scene_def.lights.len(), 1);

    // Test nodes
    assert_eq!(scene_def.nodes.len(), 2);

    // Test forces
    assert_eq!(scene_def.forces.len(), 1);

    // Test settings
    assert_eq!(scene_def.settings.physics.enabled, true);
}
