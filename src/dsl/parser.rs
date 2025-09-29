//! JSON DSL Parser for scene definitions
//!
//! Parses JSON strings into SceneDefinition structures
//! with validation and error handling.

use super::scene::*;
use serde_json::{self, Value};
use std::collections::HashMap;

/// Parser for JSON DSL scene definitions
pub struct SceneParser {
    /// Cached parsed scenes
    cache: HashMap<String, SceneDefinition>,
}

impl SceneParser {
    /// Create a new scene parser
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Parse JSON string into SceneDefinition
    pub fn parse_json(&mut self, json_str: &str) -> Result<SceneDefinition, ParseError> {
        // Check cache first
        if let Some(scene) = self.cache.get(json_str) {
            return Ok(scene.clone());
        }

        // Parse JSON
        let value: Value = serde_json::from_str(json_str)
            .map_err(|e| ParseError::InvalidJson(e.to_string()))?;

        // Validate structure
        self.validate_scene_structure(&value)?;

        // Deserialize into SceneDefinition
        let scene: SceneDefinition = serde_json::from_value(value)
            .map_err(|e| ParseError::DeserializationError(e.to_string()))?;

        // Post-process and validate
        self.post_process_scene(&scene)?;

        // Cache the result
        self.cache.insert(json_str.to_string(), scene.clone());

        Ok(scene)
    }

    /// Parse JSON from file
    pub fn parse_file(&mut self, file_path: &str) -> Result<SceneDefinition, ParseError> {
        let json_str = std::fs::read_to_string(file_path)
            .map_err(|e| ParseError::FileReadError(e.to_string()))?;

        self.parse_json(&json_str)
    }

    /// Validate basic scene structure
    fn validate_scene_structure(&self, value: &Value) -> Result<(), ParseError> {
        let obj = value.as_object()
            .ok_or_else(|| ParseError::InvalidStructure("Root must be an object".to_string()))?;

        // Check required fields
        if !obj.contains_key("metadata") {
            return Err(ParseError::MissingField("metadata".to_string()));
        }
        if !obj.contains_key("nodes") {
            return Err(ParseError::MissingField("nodes".to_string()));
        }
        if !obj.contains_key("globals") {
            return Err(ParseError::MissingField("globals".to_string()));
        }

        // Validate nodes array
        if let Some(nodes) = obj.get("nodes") {
            if !nodes.is_array() {
                return Err(ParseError::InvalidStructure("nodes must be an array".to_string()));
            }
        }

        Ok(())
    }

    /// Post-process and validate parsed scene
    fn post_process_scene(&self, scene: &SceneDefinition) -> Result<(), ParseError> {
        // Validate node IDs are unique
        let mut ids = std::collections::HashSet::new();
        self.validate_node_ids(&scene.nodes, &mut ids)?;

        // Validate node relationships
        self.validate_node_relationships(&scene.nodes)?;

        // Validate physics configuration
        self.validate_physics_config(&scene.globals.physics)?;

        Ok(())
    }

    /// Recursively validate node IDs are unique
    fn validate_node_ids(&self, nodes: &[SceneNode], ids: &mut std::collections::HashSet<String>) -> Result<(), ParseError> {
        for node in nodes {
            if !ids.insert(node.id.clone()) {
                return Err(ParseError::DuplicateId(node.id.clone()));
            }

            // Validate children recursively
            self.validate_node_ids(&node.children, ids)?;
        }
        Ok(())
    }

    /// Validate node relationships and references
    fn validate_node_relationships(&self, nodes: &[SceneNode]) -> Result<(), ParseError> {
        for node in nodes {
            match &node.properties {
                NodeProperties::Geometry(props) => {
                    self.validate_geometry_properties(props)?;
                }
                NodeProperties::Light(props) => {
                    self.validate_light_properties(props)?;
                }
                NodeProperties::Camera(props) => {
                    self.validate_camera_properties(props)?;
                }
                NodeProperties::Particles(props) => {
                    self.validate_particle_properties(props)?;
                }
                NodeProperties::Empty(_) => {
                    // Empty nodes have no specific validation
                }
            }

            // Validate children recursively
            self.validate_node_relationships(&node.children)?;
        }
        Ok(())
    }

    /// Validate geometry properties
    fn validate_geometry_properties(&self, props: &GeometryProperties) -> Result<(), ParseError> {
        // Validate material
        if let Some(physics) = &props.physics {
            // Validate physics body properties
            if physics.mass < 0.0 {
                return Err(ParseError::InvalidValue("Mass cannot be negative".to_string()));
            }
            if physics.friction < 0.0 {
                return Err(ParseError::InvalidValue("Friction cannot be negative".to_string()));
            }
            if physics.restitution < 0.0 || physics.restitution > 1.0 {
                return Err(ParseError::InvalidValue("Restitution must be between 0 and 1".to_string()));
            }
        }

        Ok(())
    }

    /// Validate light properties
    fn validate_light_properties(&self, props: &LightProperties) -> Result<(), ParseError> {
        if props.intensity < 0.0 {
            return Err(ParseError::InvalidValue("Light intensity cannot be negative".to_string()));
        }

        if let Some(range) = props.range {
            if range <= 0.0 {
                return Err(ParseError::InvalidValue("Light range must be positive".to_string()));
            }
        }

        Ok(())
    }

    /// Validate camera properties
    fn validate_camera_properties(&self, props: &CameraProperties) -> Result<(), ParseError> {
        if props.near <= 0.0 {
            return Err(ParseError::InvalidValue("Camera near plane must be positive".to_string()));
        }
        if props.far <= props.near {
            return Err(ParseError::InvalidValue("Camera far plane must be greater than near plane".to_string()));
        }

        if let Some(fov) = props.fov {
            if fov <= 0.0 || fov >= 180.0 {
                return Err(ParseError::InvalidValue("Camera FOV must be between 0 and 180 degrees".to_string()));
            }
        }

        Ok(())
    }

    /// Validate particle properties
    fn validate_particle_properties(&self, props: &ParticleProperties) -> Result<(), ParseError> {
        if props.max_particles == 0 {
            return Err(ParseError::InvalidValue("Max particles must be greater than 0".to_string()));
        }
        if props.lifetime <= 0.0 {
            return Err(ParseError::InvalidValue("Particle lifetime must be positive".to_string()));
        }
        if props.emission_rate <= 0.0 {
            return Err(ParseError::InvalidValue("Emission rate must be positive".to_string()));
        }
        if props.size <= 0.0 {
            return Err(ParseError::InvalidValue("Particle size must be positive".to_string()));
        }

        Ok(())
    }

    /// Validate physics configuration
    fn validate_physics_config(&self, physics: &PhysicsConfig) -> Result<(), ParseError> {
        if physics.time_step <= 0.0 {
            return Err(ParseError::InvalidValue("Physics time step must be positive".to_string()));
        }
        if physics.max_substeps == 0 {
            return Err(ParseError::InvalidValue("Max physics substeps must be greater than 0".to_string()));
        }

        Ok(())
    }

    /// Clear parser cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for SceneParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parser error types
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    InvalidJson(String),
    InvalidStructure(String),
    MissingField(String),
    DuplicateId(String),
    InvalidValue(String),
    DeserializationError(String),
    FileReadError(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidJson(msg) => write!(f, "Invalid JSON: {}", msg),
            ParseError::InvalidStructure(msg) => write!(f, "Invalid structure: {}", msg),
            ParseError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ParseError::DuplicateId(id) => write!(f, "Duplicate node ID: {}", id),
            ParseError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            ParseError::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            ParseError::FileReadError(msg) => write!(f, "File read error: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

/// Helper function to create a minimal valid scene
pub fn create_minimal_scene() -> SceneDefinition {
    SceneDefinition {
        metadata: SceneMetadata {
            version: "1.0".to_string(),
            title: "Minimal Scene".to_string(),
            description: Some("A minimal valid Jigen scene".to_string()),
            author: None,
        },
        nodes: vec![
            SceneNode {
                id: "camera".to_string(),
                node_type: NodeType::Camera,
                name: Some("Main Camera".to_string()),
                transform: Transform {
                    position: nalgebra::Vector3::new(0.0, 5.0, 10.0),
                    rotation: nalgebra::Quaternion::identity(),
                    scale: nalgebra::Vector3::new(1.0, 1.0, 1.0),
                },
                properties: NodeProperties::Camera(CameraProperties {
                    camera_type: CameraType::Perspective,
                    fov: Some(60.0),
                    near: 0.1,
                    far: 1000.0,
                    orthographic_size: None,
                }),
                children: Vec::new(),
                tags: vec!["camera".to_string(), "main".to_string()],
            },
            SceneNode {
                id: "cube".to_string(),
                node_type: NodeType::Geometry,
                name: Some("Red Cube".to_string()),
                transform: Transform {
                    position: nalgebra::Vector3::new(0.0, 1.0, 0.0),
                    rotation: nalgebra::Quaternion::identity(),
                    scale: nalgebra::Vector3::new(1.0, 1.0, 1.0),
                },
                properties: NodeProperties::Geometry(GeometryProperties {
                    primitive: GeometryPrimitive::Box,
                    material: Material {
                        material_type: MaterialType::Standard,
                        color: Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
                        metallic: Some(0.0),
                        roughness: Some(0.5),
                        emissive: None,
                        textures: HashMap::new(),
                    },
                    physics: Some(PhysicsBody {
                        body_type: PhysicsBodyType::Dynamic,
                        mass: 1.0,
                        friction: 0.5,
                        restitution: 0.3,
                        shape: CollisionShape::Box {
                            size: nalgebra::Vector3::new(1.0, 1.0, 1.0),
                        },
                        velocity: None,
                        angular_velocity: None,
                        forces: Vec::new(),
                    }),
                }),
                children: Vec::new(),
                tags: vec!["geometry".to_string(), "cube".to_string()],
            },
        ],
        globals: SceneGlobals {
            background: Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 },
            fog: None,
            shadows: true,
            physics: PhysicsConfig {
                enabled: true,
                gravity: nalgebra::Vector3::new(0.0, -9.81, 0.0),
                time_step: 1.0 / 60.0,
                max_substeps: 10,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_scene_creation() {
        let scene = create_minimal_scene();
        assert_eq!(scene.nodes.len(), 2);
        assert_eq!(scene.metadata.title, "Minimal Scene");
    }

    #[test]
    fn test_parser_validation() {
        let mut parser = SceneParser::new();

        // Test invalid JSON
        let result = parser.parse_json("{invalid json");
        assert!(matches!(result, Err(ParseError::InvalidJson(_))));

        // Test missing required fields
        let result = parser.parse_json(r#"{"metadata": {}}"#);
        assert!(matches!(result, Err(ParseError::MissingField(_))));
    }

    #[test]
    fn test_duplicate_id_validation() {
        let mut parser = SceneParser::new();

        let json = r#"
        {
            "metadata": {"version": "1.0", "title": "Test"},
            "nodes": [
                {"id": "node1", "type": "empty", "transform": {"position": [0,0,0], "rotation": [0,0,0,1], "scale": [1,1,1]}, "properties": {}},
                {"id": "node1", "type": "empty", "transform": {"position": [0,0,0], "rotation": [0,0,0,1], "scale": [1,1,1]}, "properties": {}}
            ],
            "globals": {"background": {"r": 0, "g": 0, "b": 0, "a": 1}, "fog": null, "shadows": true, "physics": {"enabled": true, "gravity": [0,-9.81,0], "time_step": 0.016, "max_substeps": 10}}
        }
        "#;

        let result = parser.parse_json(json);
        assert!(matches!(result, Err(ParseError::DuplicateId(_))));
    }
}
