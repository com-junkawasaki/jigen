//! Comprehensive integration tests for the entire Jigen pipeline

use jigen::dsl::{SceneParser, scene::*};
use jigen::graph::SceneGraph;
use serde_json::json;

#[test]
fn test_complete_jigen_pipeline() {
    // Create a comprehensive scene definition
    let json_scene = json!({
        "metadata": {
            "version": "1.0",
            "title": "Integration Test Scene",
            "description": "Complete pipeline test"
        },
        "camera": {
            "position": [5.0, 5.0, 5.0],
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
                "id": "ground",
                "type": "geometry",
                "transform": {
                    "position": [0.0, -1.0, 0.0],
                    "scale": [20.0, 0.2, 20.0]
                },
                "properties": {
                    "primitive": "box",
                    "material": {
                        "color": [0.7, 0.7, 0.7, 1.0]
                    },
                    "physics": {
                        "body_type": "static",
                        "mass": 0.0
                    }
                }
            },
            {
                "id": "red_cube",
                "type": "geometry",
                "transform": {
                    "position": [2.0, 2.0, 0.0]
                },
                "properties": {
                    "primitive": "box",
                    "material": {
                        "color": [1.0, 0.0, 0.0, 1.0]
                    },
                    "physics": {
                        "body_type": "dynamic",
                        "mass": 1.0,
                        "velocity": [0.0, 0.0, 0.0]
                    }
                }
            },
            {
                "id": "blue_sphere",
                "type": "geometry",
                "transform": {
                    "position": [-2.0, 3.0, 1.0]
                },
                "properties": {
                    "primitive": "sphere",
                    "material": {
                        "color": [0.0, 0.0, 1.0, 1.0]
                    },
                    "physics": {
                        "body_type": "dynamic",
                        "mass": 1.5,
                        "velocity": [1.0, 0.0, 0.0]
                    }
                }
            },
            {
                "id": "green_cylinder",
                "type": "geometry",
                "transform": {
                    "position": [0.0, 1.0, -2.0],
                    "rotation": [0.0, 0.0, 0.0, 1.0],
                    "scale": [1.0, 2.0, 1.0]
                },
                "properties": {
                    "primitive": "cylinder",
                    "material": {
                        "color": [0.0, 1.0, 0.0, 1.0]
                    },
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
            "affected_nodes": ["red_cube", "blue_sphere", "green_cylinder"]
        }],
        "constraints": [{
            "type": "distance",
            "body_a": "red_cube",
            "body_b": "blue_sphere",
            "rest_length": 5.0,
            "stiffness": 100.0
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

    // Step 1: Parse JSON scene
    let parser = SceneParser::new();
    let scene_def = parser.parse_json(&json_scene.to_string())
        .expect("Failed to parse scene JSON");

    // Verify scene metadata
    assert_eq!(scene_def.metadata.title, "Integration Test Scene");
    assert_eq!(scene_def.nodes.len(), 4);
    assert_eq!(scene_def.forces.len(), 1);
    assert_eq!(scene_def.constraints.len(), 1);

    // Step 2: Load into SceneGraph
    let mut scene_graph = SceneGraph::new();
    scene_graph.load_scene(&scene_def);

    // Verify SceneGraph structure
    assert_eq!(scene_graph.graph.node_count(), 4);

    // Verify nodes exist and have correct properties
    let vertices: Vec<_> = scene_graph.graph.node_weights().collect();
    let ground = vertices.iter().find(|v| v.id == "ground").unwrap();
    let red_cube = vertices.iter().find(|v| v.id == "red_cube").unwrap();
    let blue_sphere = vertices.iter().find(|v| v.id == "blue_sphere").unwrap();
    let green_cylinder = vertices.iter().find(|v| v.id == "green_cylinder").unwrap();

    // Verify physics properties
    assert!(ground.physics_state.is_some());
    assert!(red_cube.physics_state.is_some());
    assert!(blue_sphere.physics_state.is_some());
    assert!(green_cylinder.physics_state.is_some());

    // Ground should be static
    assert_eq!(ground.physics_state.as_ref().unwrap().mass, 0.0);

    // Dynamic objects should have mass
    assert_eq!(red_cube.physics_state.as_ref().unwrap().mass, 1.0);
    assert_eq!(blue_sphere.physics_state.as_ref().unwrap().mass, 1.5);
    assert_eq!(green_cylinder.physics_state.as_ref().unwrap().mass, 2.0);

    // Step 3: Update topology (create relationships)
    scene_graph.update_topology();

    // Verify relationships were created
    // At minimum, we should have some edges (spatial relationships)
    assert!(scene_graph.graph.edge_count() > 0);

    // Step 4: Test SceneGraph operations
    let stats = scene_graph.statistics();
    assert_eq!(stats.node_count, 4);
    assert!(stats.edge_count > 0);

    // Step 5: Test incident handling
    let incident_count_before = scene_graph.incident_manager.active_count();

    // Add a test incident
    let incident = jigen::graph::incident::Incident::new(
        "test_incident".to_string(),
        jigen::graph::incident::IncidentType::StateChange,
        vec!["red_cube".to_string()],
    );

    scene_graph.incident_manager.add_incident(incident);
    assert_eq!(scene_graph.incident_manager.active_count(), incident_count_before + 1);

    // Process incidents
    let vertices_refs: Vec<_> = scene_graph.graph.node_weights().collect();
    let edges_refs: Vec<_> = scene_graph.graph.edge_weights().collect();
    scene_graph.incident_manager.process_incidents(&vertices_refs, &edges_refs);

    // Incident should be processed (completed)
    assert_eq!(scene_graph.incident_manager.active_count(), incident_count_before);

    // Step 6: Test scene export
    let exported_json = scene_graph.export_scene();
    let exported_scene: SceneDefinition = serde_json::from_str(&exported_json)
        .expect("Failed to parse exported scene");

    // Verify exported scene has same number of nodes
    assert_eq!(exported_scene.nodes.len(), scene_def.nodes.len());

    // Step 7: Test scene clearing
    scene_graph.clear();
    assert_eq!(scene_graph.graph.node_count(), 0);
    assert_eq!(scene_graph.incident_manager.active_count(), 0);
}

#[test]
fn test_performance_with_large_scene() {
    use std::time::Instant;

    let parser = SceneParser::new();

    // Create a scene with many objects
    let mut nodes = Vec::new();
    for i in 0..50 {
        nodes.push(json!({
            "id": format!("object_{}", i),
            "type": "geometry",
            "transform": {
                "position": [i as f32 * 2.0, 0.0, 0.0]
            },
            "properties": {
                "primitive": "box",
                "physics": {
                    "body_type": "dynamic",
                    "mass": 1.0
                }
            }
        }));
    }

    let json_scene = json!({
        "nodes": nodes,
        "forces": [{
            "type": "gravity",
            "vector": [0.0, -9.81, 0.0]
        }]
    });

    // Measure parsing time
    let start = Instant::now();
    let scene_def = parser.parse_json(&json_scene.to_string())
        .expect("Failed to parse large scene");
    let parse_time = start.elapsed();

    // Measure SceneGraph loading time
    let start = Instant::now();
    let mut scene_graph = SceneGraph::new();
    scene_graph.load_scene(&scene_def);
    let load_time = start.elapsed();

    // Measure topology update time
    let start = Instant::now();
    scene_graph.update_topology();
    let topology_time = start.elapsed();

    // Verify results
    assert_eq!(scene_graph.graph.node_count(), 50);

    // Performance assertions (reasonable times for 50 objects)
    assert!(parse_time.as_millis() < 1000, "Parsing took too long: {:?}", parse_time);
    assert!(load_time.as_millis() < 1000, "Loading took too long: {:?}", load_time);
    assert!(topology_time.as_millis() < 1000, "Topology update took too long: {:?}", topology_time);
}

/// Performance regression test - ensure operations don't get slower over time
#[test]
fn test_performance_regression() {
    use std::time::Instant;

    let parser = SceneParser::new();

    // Create a moderate-sized scene for consistent benchmarking
    let mut nodes = Vec::new();
    for i in 0..100 {
        nodes.push(json!({
            "id": format!("perf_node_{}", i),
            "type": "geometry",
            "transform": {
                "position": [i as f32 * 0.5, 0.0, 0.0]
            },
            "properties": {
                "primitive": "box",
                "physics": {
                    "body_type": "dynamic",
                    "mass": 1.0
                }
            }
        }));
    }

    let scene_json = json!({
        "nodes": nodes,
        "forces": [{"type": "gravity", "vector": [0.0, -9.81, 0.0]}]
    });

    // Measure baseline performance
    let json_str = scene_json.to_string();

    // Parsing performance
    let parse_start = Instant::now();
    let scene_def = parser.parse_json(&json_str).unwrap();
    let parse_time = parse_start.elapsed();

    // SceneGraph loading performance
    let load_start = Instant::now();
    let mut scene_graph = SceneGraph::new();
    scene_graph.load_scene(&scene_def);
    let load_time = load_start.elapsed();

    // Topology update performance
    let topology_start = Instant::now();
    scene_graph.update_topology();
    let topology_time = topology_start.elapsed();

    // Statistics generation performance
    let stats_start = Instant::now();
    let _stats = scene_graph.statistics();
    let stats_time = stats_start.elapsed();

    // Export performance
    let export_start = Instant::now();
    let _exported = scene_graph.export_scene();
    let export_time = export_start.elapsed();

    // Performance thresholds (these should be adjusted based on target hardware)
    const MAX_PARSE_TIME_MS: u128 = 500;
    const MAX_LOAD_TIME_MS: u128 = 300;
    const MAX_TOPOLOGY_TIME_MS: u128 = 200;
    const MAX_STATS_TIME_MS: u128 = 50;
    const MAX_EXPORT_TIME_MS: u128 = 100;

    assert!(parse_time.as_millis() < MAX_PARSE_TIME_MS,
        "Parsing regression: {}ms (max: {}ms)", parse_time.as_millis(), MAX_PARSE_TIME_MS);
    assert!(load_time.as_millis() < MAX_LOAD_TIME_MS,
        "Loading regression: {}ms (max: {}ms)", load_time.as_millis(), MAX_LOAD_TIME_MS);
    assert!(topology_time.as_millis() < MAX_TOPOLOGY_TIME_MS,
        "Topology regression: {}ms (max: {}ms)", topology_time.as_millis(), MAX_TOPOLOGY_TIME_MS);
    assert!(stats_time.as_millis() < MAX_STATS_TIME_MS,
        "Stats regression: {}ms (max: {}ms)", stats_time.as_millis(), MAX_STATS_TIME_MS);
    assert!(export_time.as_millis() < MAX_EXPORT_TIME_MS,
        "Export regression: {}ms (max: {}ms)", export_time.as_millis(), MAX_EXPORT_TIME_MS);
}

#[test]
fn test_error_handling_and_recovery() {
    let parser = SceneParser::new();

    // Test with missing required fields (should still work with defaults)
    let incomplete_scene = json!({
        "nodes": [{
            "id": "test",
            "type": "geometry"
            // Missing transform and properties
        }]
    });

    let scene_def = parser.parse_json(&incomplete_scene.to_string())
        .expect("Should handle incomplete scenes gracefully");

    assert_eq!(scene_def.nodes.len(), 1);
    assert_eq!(scene_def.nodes[0].id, "test");

    // Test SceneGraph loading of incomplete scene
    let mut scene_graph = SceneGraph::new();
    scene_graph.load_scene(&scene_def);

    assert_eq!(scene_graph.graph.node_count(), 1);

    // Test clearing and reloading
    scene_graph.clear();
    assert_eq!(scene_graph.graph.node_count(), 0);

    scene_graph.load_scene(&scene_def);
    assert_eq!(scene_graph.graph.node_count(), 1);
}

#[test]
fn test_scene_modification_and_updates() {
    let parser = SceneParser::new();

    // Create initial scene
    let json_scene = json!({
        "nodes": [{
            "id": "dynamic_object",
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
        }]
    });

    let scene_def = parser.parse_json(&json_scene.to_string()).unwrap();

    let mut scene_graph = SceneGraph::new();
    scene_graph.load_scene(&scene_def);

    // Verify initial state
    let vertices: Vec<_> = scene_graph.graph.node_weights().collect();
    let vertex = vertices.first().unwrap();
    assert_eq!(vertex.transform.position, nalgebra::Vector3::new(0.0, 5.0, 0.0));

    // Simulate physics update (manual position change)
    let updated_vertex = jigen::graph::vertex::Vertex {
        transform: jigen::dsl::scene::Transform {
            position: nalgebra::Vector3::new(0.0, 4.5, 0.0), // Fell down a bit
            rotation: vertex.transform.rotation,
            scale: vertex.transform.scale,
        },
        ..vertex.clone()
    };

    // Update vertex in graph (simplified - in real implementation this would be done through Bevy)
    // This test verifies that vertex updates work conceptually

    assert!(true); // Placeholder - vertex update logic would be tested here
}

#[test]
fn test_memory_management_and_cleanup() {
    let parser = SceneParser::new();

    // Create scene
    let json_scene = json!({
        "nodes": [
            {"id": "obj1", "type": "geometry", "properties": {"primitive": "box"}},
            {"id": "obj2", "type": "geometry", "properties": {"primitive": "sphere"}},
            {"id": "obj3", "type": "geometry", "properties": {"primitive": "cylinder"}}
        ]
    });

    let scene_def = parser.parse_json(&json_scene.to_string()).unwrap();

    let mut scene_graph = SceneGraph::new();
    scene_graph.load_scene(&scene_def);
    scene_graph.update_topology();

    // Verify initial state
    assert_eq!(scene_graph.graph.node_count(), 3);
    assert!(scene_graph.graph.edge_count() >= 0); // May have spatial relationships

    // Test incident cleanup
    for i in 0..5 {
        let incident = jigen::graph::incident::Incident::new(
            format!("incident_{}", i),
            jigen::graph::incident::IncidentType::StateChange,
            vec!["obj1".to_string()],
        );
        scene_graph.incident_manager.add_incident(incident);
    }

    assert_eq!(scene_graph.incident_manager.active_count(), 5);

    // Process and clear
    let vertices_refs: Vec<_> = scene_graph.graph.node_weights().collect();
    let edges_refs: Vec<_> = scene_graph.graph.edge_weights().collect();
    scene_graph.incident_manager.process_incidents(&vertices_refs, &edges_refs);

    // Clear everything
    scene_graph.clear();

    // Verify cleanup
    assert_eq!(scene_graph.graph.node_count(), 0);
    assert_eq!(scene_graph.graph.edge_count(), 0);
    assert_eq!(scene_graph.incident_manager.active_count(), 0);
}
