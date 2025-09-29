//! Integration tests for SceneGraph functionality

use jigen::dsl::{SceneParser, scene::*};
use jigen::graph::{SceneGraph, vertex::Vertex, edge::Edge, incident::Incident};
use nalgebra::{Vector3, Quaternion};
use petgraph::graph::NodeIndex;
use serde_json::json;

#[test]
fn test_scene_graph_creation_and_basic_operations() {
    let mut scene_graph = SceneGraph::new();

    // Create a simple scene
    let scene_json = json!({
        "nodes": [{
            "id": "test_cube",
            "type": "geometry",
            "transform": {
                "position": [1.0, 2.0, 3.0],
                "rotation": [0.0, 0.0, 0.0, 1.0],
                "scale": [1.0, 1.0, 1.0]
            },
            "properties": {
                "primitive": "box"
            }
        }]
    });

    let parser = SceneParser::new();
    let scene_def = parser.parse_json(&scene_json.to_string()).unwrap();

    scene_graph.load_scene(&scene_def);

    // Check that vertex was added
    assert_eq!(scene_graph.graph.node_count(), 1);

    // Find the vertex
    let node_index = scene_graph.graph.node_indices().next().unwrap();
    let vertex = scene_graph.graph.node_weight(node_index).unwrap();

    assert_eq!(vertex.id, "test_cube");
    assert_eq!(vertex.transform.position, Vector3::new(1.0, 2.0, 3.0));
}

#[test]
fn test_scene_graph_hierarchy() {
    let mut scene_graph = SceneGraph::new();

    // Create hierarchical scene
    let scene_json = json!({
        "nodes": [
            {
                "id": "parent",
                "type": "group",
                "transform": {
                    "position": [0.0, 0.0, 0.0]
                },
                "children": ["child"]
            },
            {
                "id": "child",
                "type": "geometry",
                "transform": {
                    "position": [1.0, 0.0, 0.0]
                },
                "properties": {
                    "primitive": "box"
                }
            }
        ]
    });

    let parser = SceneParser::new();
    let scene_def = parser.parse_json(&scene_json.to_string()).unwrap();

    scene_graph.load_scene(&scene_def);

    // Check hierarchy
    assert_eq!(scene_graph.graph.node_count(), 2);

    // Find vertices
    let vertices: Vec<_> = scene_graph.graph.node_weights().collect();
    let parent = vertices.iter().find(|v| v.id == "parent").unwrap();
    let child = vertices.iter().find(|v| v.id == "child").unwrap();

    // Check parent-child relationship
    assert!(parent.child_ids.contains(&"child".to_string()));
    assert_eq!(child.parent_id, Some("parent".to_string()));
}

#[test]
fn test_scene_graph_spatial_relationships() {
    let mut scene_graph = SceneGraph::new();

    // Create scene with multiple objects
    let scene_json = json!({
        "nodes": [
            {
                "id": "obj1",
                "type": "geometry",
                "transform": {"position": [0.0, 0.0, 0.0]},
                "properties": {"primitive": "box"}
            },
            {
                "id": "obj2",
                "type": "geometry",
                "transform": {"position": [3.0, 0.0, 0.0]},
                "properties": {"primitive": "box"}
            },
            {
                "id": "obj3",
                "type": "geometry",
                "transform": {"position": [10.0, 0.0, 0.0]},
                "properties": {"primitive": "box"}
            }
        ]
    });

    let parser = SceneParser::new();
    let scene_def = parser.parse_json(&scene_json.to_string()).unwrap();

    scene_graph.load_scene(&scene_def);
    scene_graph.update_topology();

    // Check spatial edges were created
    // Objects within 50 units should be connected
    let spatial_edges: Vec<_> = scene_graph.graph.edge_weights()
        .filter(|e| matches!(e.edge_type, jigen::graph::edge::EdgeType::Spatial))
        .collect();

    // obj1 and obj2 should be connected (distance = 3.0 < 50.0)
    // obj1 and obj3 should not be connected (distance = 10.0 > 50.0)
    // obj2 and obj3 should not be connected (distance = 7.0 < 50.0)
    assert!(!spatial_edges.is_empty());
}

#[test]
fn test_scene_graph_physics_relationships() {
    let mut scene_graph = SceneGraph::new();

    // Create scene with physics objects and forces
    let scene_json = json!({
        "nodes": [
            {
                "id": "body1",
                "type": "geometry",
                "transform": {"position": [0.0, 0.0, 0.0]},
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
                "transform": {"position": [2.0, 0.0, 0.0]},
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
        }],
        "constraints": [{
            "type": "distance",
            "body_a": "body1",
            "body_b": "body2",
            "rest_length": 2.0
        }]
    });

    let parser = SceneParser::new();
    let scene_def = parser.parse_json(&scene_json.to_string()).unwrap();

    scene_graph.load_scene(&scene_def);
    scene_graph.update_topology();

    // Check force edges were created
    let force_edges: Vec<_> = scene_graph.graph.edge_weights()
        .filter(|e| matches!(e.edge_type, jigen::graph::edge::EdgeType::Force))
        .collect();

    assert!(!force_edges.is_empty());
}

#[test]
fn test_scene_graph_vertex_operations() {
    let mut scene_graph = SceneGraph::new();

    // Add vertices manually
    let vertex1 = Vertex::from_scene_node(
        &SceneNode {
            id: "test1".to_string(),
            node_type: NodeType::Geometry,
            transform: Transform {
                position: Vector3::new(0.0, 0.0, 0.0),
                rotation: Quaternion::identity(),
                scale: Vector3::new(1.0, 1.0, 1.0),
            },
            properties: NodeProperties::Geometry(GeometryProperties {
                primitive: GeometryPrimitive::Box,
                material: Some(MaterialProperties {
                    color: [1.0, 0.0, 0.0, 1.0],
                    ..Default::default()
                }),
                physics: None,
            }),
            tags: vec![],
            children: vec![],
        },
        None
    );

    let vertex2 = Vertex::from_scene_node(
        &SceneNode {
            id: "test2".to_string(),
            node_type: NodeType::Geometry,
            transform: Transform {
                position: Vector3::new(1.0, 0.0, 0.0),
                rotation: Quaternion::identity(),
                scale: Vector3::new(1.0, 1.0, 1.0),
            },
            properties: NodeProperties::Geometry(GeometryProperties {
                primitive: GeometryPrimitive::Sphere,
                material: Some(MaterialProperties {
                    color: [0.0, 1.0, 0.0, 1.0],
                    ..Default::default()
                }),
                physics: None,
            }),
            tags: vec![],
            children: vec![],
        },
        None
    );

    let idx1 = scene_graph.graph.add_node(vertex1);
    let idx2 = scene_graph.graph.add_node(vertex2);

    // Add edge between vertices
    let edge = Edge::spatial("test1".to_string(), "test2".to_string(), 1.0, [1.0, 0.0, 0.0]);
    scene_graph.graph.add_edge(idx1, idx2, edge);

    assert_eq!(scene_graph.graph.node_count(), 2);
    assert_eq!(scene_graph.graph.edge_count(), 1);
}

#[test]
fn test_scene_graph_statistics() {
    let mut scene_graph = SceneGraph::new();

    // Create a scene
    let scene_json = json!({
        "nodes": [{
            "id": "test",
            "type": "geometry",
            "properties": {"primitive": "box"}
        }]
    });

    let parser = SceneParser::new();
    let scene_def = parser.parse_json(&scene_json.to_string()).unwrap();

    scene_graph.load_scene(&scene_def);

    let stats = scene_graph.statistics();
    assert_eq!(stats.node_count, 1);
    assert_eq!(stats.edge_count, 0); // No relationships created yet
    assert_eq!(stats.topology_version, 0);
}

#[test]
fn test_scene_graph_clear() {
    let mut scene_graph = SceneGraph::new();

    // Create a scene
    let scene_json = json!({
        "nodes": [{
            "id": "test",
            "type": "geometry",
            "properties": {"primitive": "box"}
        }]
    });

    let parser = SceneParser::new();
    let scene_def = parser.parse_json(&scene_json.to_string()).unwrap();

    scene_graph.load_scene(&scene_def);
    assert_eq!(scene_graph.graph.node_count(), 1);

    scene_graph.clear();
    assert_eq!(scene_graph.graph.node_count(), 0);
}

#[test]
fn test_scene_graph_incident_handling() {
    let mut scene_graph = SceneGraph::new();

    // Create a scene
    let scene_json = json!({
        "nodes": [{
            "id": "test",
            "type": "geometry",
            "properties": {"primitive": "box"}
        }]
    });

    let parser = SceneParser::new();
    let scene_def = parser.parse_json(&scene_json.to_string()).unwrap();

    scene_graph.load_scene(&scene_def);

    // Test incident manager
    let incident = Incident::new(
        "test_collision".to_string(),
        jigen::graph::incident::IncidentType::Collision,
        vec!["test".to_string()],
    );

    scene_graph.incident_manager.add_incident(incident);
    assert_eq!(scene_graph.incident_manager.active_count(), 1);

    // Process incidents (should complete immediately with current implementation)
    let vertices: Vec<_> = scene_graph.graph.node_weights().collect();
    let edges: Vec<_> = scene_graph.graph.edge_weights().collect();
    scene_graph.incident_manager.process_incidents(&vertices, &edges);

    assert_eq!(scene_graph.incident_manager.active_count(), 0);
}
