//! Scene definition structures for JSON DSL
//!
//! Defines the declarative schema for 3D scenes including:
//! - Scene nodes (objects, lights, cameras)
//! - Materials and textures
//! - Physics properties
//! - Animation definitions

use serde::{Deserialize, Serialize};
use nalgebra::{Vector3, Quaternion, UnitQuaternion};
use std::collections::HashMap;

/// Main scene definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneDefinition {
    /// Scene metadata
    pub metadata: SceneMetadata,

    /// Scene nodes (objects, lights, cameras)
    pub nodes: Vec<SceneNode>,

    /// Global scene properties
    pub globals: SceneGlobals,
}

/// Scene metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneMetadata {
    pub version: String,
    pub title: String,
    pub description: Option<String>,
    pub author: Option<String>,
}

/// Global scene properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneGlobals {
    pub background: Color,
    pub fog: Option<Fog>,
    pub shadows: bool,
    pub physics: PhysicsConfig,
}

/// Scene node definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneNode {
    /// Unique identifier for the node
    pub id: String,

    /// Node type
    #[serde(rename = "type")]
    pub node_type: NodeType,

    /// Node name (optional)
    pub name: Option<String>,

    /// Transform properties
    pub transform: Transform,

    /// Node-specific properties
    #[serde(flatten)]
    pub properties: NodeProperties,

    /// Child nodes
    pub children: Vec<SceneNode>,

    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Node type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    /// 3D geometry object
    Geometry,
    /// Light source
    Light,
    /// Camera
    Camera,
    /// Empty node (for grouping)
    Empty,
    /// Particle system
    Particles,
}

/// Node properties (flattened into node)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NodeProperties {
    /// Geometry node properties
    Geometry(GeometryProperties),
    /// Light node properties
    Light(LightProperties),
    /// Camera node properties
    Camera(CameraProperties),
    /// Empty node properties
    Empty(EmptyProperties),
    /// Particle system properties
    Particles(ParticleProperties),
}

/// Geometry node properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometryProperties {
    /// Geometry primitive type
    pub primitive: GeometryPrimitive,

    /// Material definition
    pub material: Material,

    /// Physics properties
    pub physics: Option<PhysicsBody>,
}

/// Geometry primitive types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GeometryPrimitive {
    Box,
    Sphere,
    Cylinder,
    Plane,
    Capsule,
    Cone,
    Torus,
    Custom(String),
}

/// Light node properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightProperties {
    /// Light type
    pub light_type: LightType,

    /// Light color
    pub color: Color,

    /// Light intensity
    pub intensity: f32,

    /// Light range (for point/spot lights)
    pub range: Option<f32>,

    /// Shadow casting
    pub cast_shadows: bool,
}

/// Light types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LightType {
    Directional,
    Point,
    Spot,
}

/// Camera node properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraProperties {
    /// Camera type
    pub camera_type: CameraType,

    /// Field of view (degrees, for perspective)
    pub fov: Option<f32>,

    /// Near clipping plane
    pub near: f32,

    /// Far clipping plane
    pub far: f32,

    /// Orthographic size (for orthographic)
    pub orthographic_size: Option<f32>,
}

/// Camera types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CameraType {
    Perspective,
    Orthographic,
}

/// Empty node properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyProperties {
    // Empty nodes have no specific properties
}

/// Particle system properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleProperties {
    /// Maximum number of particles
    pub max_particles: usize,

    /// Particle lifetime
    pub lifetime: f32,

    /// Emission rate (particles per second)
    pub emission_rate: f32,

    /// Initial velocity
    pub initial_velocity: Vector3<f32>,

    /// Particle size
    pub size: f32,

    /// Particle color
    pub color: Color,

    /// Material for particles
    pub material: Material,
}

/// Material definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    /// Material type
    #[serde(rename = "type")]
    pub material_type: MaterialType,

    /// Base color
    pub color: Color,

    /// Metallic factor
    pub metallic: Option<f32>,

    /// Roughness factor
    pub roughness: Option<f32>,

    /// Emissive color
    pub emissive: Option<Color>,

    /// Texture maps
    pub textures: HashMap<String, String>,
}

/// Material types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MaterialType {
    Standard,
    Lambert,
    Phong,
    Toon,
}

/// Physics body properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsBody {
    /// Physics body type
    #[serde(rename = "type")]
    pub body_type: PhysicsBodyType,

    /// Mass (0 for static bodies)
    pub mass: f32,

    /// Friction coefficient
    pub friction: f32,

    /// Restitution (bounciness)
    pub restitution: f32,

    /// Collision shape
    pub shape: CollisionShape,

    /// Initial velocity
    pub velocity: Option<Vector3<f32>>,

    /// Angular velocity
    pub angular_velocity: Option<Vector3<f32>>,

    /// Forces applied to this body
    pub forces: Vec<Force>,
}

/// Physics body types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PhysicsBodyType {
    Static,
    Dynamic,
    Kinematic,
}

/// Collision shape types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CollisionShape {
    Box { size: Vector3<f32> },
    Sphere { radius: f32 },
    Capsule { radius: f32, height: f32 },
    Cylinder { radius: f32, height: f32 },
    Cone { radius: f32, height: f32 },
    Mesh { mesh_id: String },
}

/// Force definition for physics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Force {
    /// Force type
    #[serde(rename = "type")]
    pub force_type: ForceType,

    /// Force vector/direction
    pub vector: Vector3<f32>,

    /// Force magnitude (alternative to vector)
    pub magnitude: Option<f32>,

    /// Application point (relative to body center)
    pub point: Option<Vector3<f32>>,

    /// Force duration (None for continuous)
    pub duration: Option<f32>,
}

/// Force types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ForceType {
    Constant,
    Impulse,
    Gravity,
    Spring,
    Damping,
}

/// Transform definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    /// Position in 3D space
    pub position: Vector3<f32>,

    /// Rotation (quaternion)
    pub rotation: Quaternion<f32>,

    /// Scale
    pub scale: Vector3<f32>,
}

/// Color definition (RGBA)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Fog definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fog {
    /// Fog type
    #[serde(rename = "type")]
    pub fog_type: FogType,

    /// Fog color
    pub color: Color,

    /// Fog density/intensity
    pub density: f32,

    /// Near distance (for linear fog)
    pub near: Option<f32>,

    /// Far distance (for linear fog)
    pub far: Option<f32>,
}

/// Fog types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FogType {
    Linear,
    Exponential,
    ExponentialSquared,
}

/// Physics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    /// Enable physics simulation
    pub enabled: bool,

    /// Gravity vector
    pub gravity: [f32; 3],

    /// Time step for physics simulation
    pub time_step: f32,

    /// Maximum number of physics substeps
    pub max_substeps: u32,
}

impl Default for SceneDefinition {
    fn default() -> Self {
        Self {
            metadata: SceneMetadata {
                version: "1.0".to_string(),
                title: "Jigen Scene".to_string(),
                description: None,
                author: None,
            },
            nodes: Vec::new(),
            globals: SceneGlobals {
                background: Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 },
                fog: None,
                shadows: true,
                physics: PhysicsConfig {
                    enabled: true,
                    gravity: [0.0, -9.81, 0.0],
                    time_step: 1.0 / 60.0,
                    max_substeps: 10,
                },
            },
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::zeros(),
            rotation: Quaternion::identity(),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }
    }
}
