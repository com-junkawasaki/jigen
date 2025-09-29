//! JSON DSL module for declarative 3D scene description
//!
//! Provides structures and parsing for JSON-based scene definitions
//! similar to A-Frame or Three.js declarative syntax.

pub mod parser;
pub mod scene;

pub use parser::*;
pub use scene::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_parser_creation() {
        let parser = SceneParser::new();
        // Parser should be created without issues
        assert!(true);
    }

    #[test]
    fn test_physics_config_creation() {
        let config = PhysicsConfig {
            enabled: true,
            gravity: [0.0, -9.81, 0.0],
            time_step: 1.0 / 60.0,
            max_substeps: 10,
        };

        assert_eq!(config.gravity, [0.0, -9.81, 0.0]);
        assert_eq!(config.time_step, 1.0 / 60.0);
        assert_eq!(config.max_substeps, 10);
    }
}
