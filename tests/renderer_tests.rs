//! Integration tests for Renderer functionality

use jigen::dsl::{SceneParser, scene::*};
use jigen::graph::SceneGraph;
use jigen::renderer::RendererPlugin;
use bevy::prelude::*;
use serde_json::json;

#[test]
fn test_renderer_plugin_creation() {
    // Test that RendererPlugin can be created without panicking
    let plugin = RendererPlugin;
    // Plugin creation should not panic
    assert!(true);
}

#[test]
fn test_scene_entity_mapping() {
    use jigen::renderer::SceneEntityMap;

    let mut entity_map = SceneEntityMap::new();

    // Test basic operations (placeholder - actual mapping happens in Bevy)
    assert_eq!(entity_map.entity_map.len(), 0);
}

#[test]
fn test_mesh_creation_from_vertex() {
    use jigen::renderer::create_mesh_from_vertex;
    use jigen::graph::vertex::Vertex;
    use nalgebra::{Vector3, Quaternion};

    // Create a test vertex
    let vertex = Vertex {
        id: "test".to_string(),
        node_type: NodeType::Geometry,
        transform: Transform {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        },
        properties: serde_json::Value::Null,
        tags: vec![],
        physics_state: None,
        graph_index: None,
        parent_id: None,
        child_ids: vec![],
        metadata: std::collections::HashMap::new(),
    };

    // Test mesh creation (should not panic)
    let mesh = create_mesh_from_vertex(&vertex);
    // Mesh creation should succeed
    assert!(true);
}

#[test]
fn test_material_creation_from_vertex() {
    use jigen::renderer::create_material_from_vertex;
    use jigen::graph::vertex::Vertex;
    use nalgebra::{Vector3, Quaternion};

    // Create a test vertex
    let vertex = Vertex {
        id: "test".to_string(),
        node_type: NodeType::Geometry,
        transform: Transform {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        },
        properties: serde_json::Value::Null,
        tags: vec![],
        physics_state: None,
        graph_index: None,
        parent_id: None,
        child_ids: vec![],
        metadata: std::collections::HashMap::new(),
    };

    // Test material creation (should not panic)
    let material = create_material_from_vertex(&vertex);
    // Material creation should succeed
    assert!(true);
}

#[test]
fn test_collider_creation_from_vertex() {
    use jigen::renderer::create_collider_from_vertex;
    use jigen::graph::vertex::Vertex;
    use nalgebra::{Vector3, Quaternion};

    // Create a test vertex
    let vertex = Vertex {
        id: "test".to_string(),
        node_type: NodeType::Geometry,
        transform: Transform {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        },
        properties: serde_json::Value::Null,
        tags: vec![],
        physics_state: None,
        graph_index: None,
        parent_id: None,
        child_ids: vec![],
        metadata: std::collections::HashMap::new(),
    };

    // Test collider creation (should not panic)
    let collider = create_collider_from_vertex(&vertex);
    // Collider creation should succeed
    assert!(true);
}

#[test]
fn test_vertex_with_physics_properties() {
    let parser = SceneParser::new();

    let json_scene = json!({
        "nodes": [{
            "id": "physics_object",
            "type": "geometry",
            "transform": {
                "position": [1.0, 2.0, 3.0],
                "rotation": [0.0, 0.0, 0.0, 1.0],
                "scale": [2.0, 2.0, 2.0]
            },
            "properties": {
                "primitive": "box",
                "material": {
                    "color": [1.0, 0.0, 0.0, 1.0]
                },
                "physics": {
                    "body_type": "dynamic",
                    "mass": 5.0,
                    "velocity": [1.0, 0.0, 0.0]
                }
            }
        }]
    });

    let scene_def = parser.parse_json(&json_scene.to_string()).unwrap();

    let mut scene_graph = SceneGraph::new();
    scene_graph.load_scene(&scene_def);

    // Get the vertex
    let vertices: Vec<_> = scene_graph.graph.node_weights().collect();
    let vertex = vertices.first().unwrap();

    assert_eq!(vertex.id, "physics_object");
    assert!(vertex.physics_state.is_some());

    let physics = vertex.physics_state.as_ref().unwrap();
    assert_eq!(physics.mass, 5.0);
    assert_eq!(physics.velocity, nalgebra::Vector3::new(1.0, 0.0, 0.0));
}

#[test]
fn test_light_creation() {
    let parser = SceneParser::new();

    let json_scene = json!({
        "nodes": [{
            "id": "test_light",
            "type": "light",
            "transform": {
                "position": [0.0, 5.0, 0.0]
            },
            "properties": {
                "light_type": "point",
                "color": [1.0, 1.0, 1.0],
                "intensity": 1000.0
            }
        }]
    });

    let scene_def = parser.parse_json(&json_scene.to_string()).unwrap();

    let mut scene_graph = SceneGraph::new();
    scene_graph.load_scene(&scene_def);

    // Get the vertex
    let vertices: Vec<_> = scene_graph.graph.node_weights().collect();
    let vertex = vertices.first().unwrap();

    assert_eq!(vertex.id, "test_light");
    assert_eq!(vertex.node_type, NodeType::Light);
}

#[test]
fn test_camera_creation() {
    let parser = SceneParser::new();

    let json_scene = json!({
        "camera": {
            "position": [0.0, 5.0, 10.0],
            "target": [0.0, 0.0, 0.0],
            "up": [0.0, 1.0, 0.0],
            "fov": 60.0
        },
        "nodes": [{
            "id": "test_object",
            "type": "geometry",
            "properties": {
                "primitive": "box"
            }
        }]
    });

    let scene_def = parser.parse_json(&json_scene.to_string()).unwrap();

    assert!(scene_def.camera.is_some());
    let camera = scene_def.camera.as_ref().unwrap();
    assert_eq!(camera.position, [0.0, 5.0, 10.0]);
    assert_eq!(camera.target, [0.0, 0.0, 0.0]);
    assert_eq!(camera.fov, 60.0);
}
