//! JSON DSL module for declarative 3D scene description
//!
//! Provides structures and parsing for JSON-based scene definitions
//! similar to A-Frame or Three.js declarative syntax.

pub mod parser;
pub mod scene;

pub use parser::*;
pub use scene::*;
