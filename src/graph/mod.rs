//! Graph module for v(vertex), e(edge), i(incident) scene topology
//!
//! Implements graph structures for:
//! - **v**: Vertices representing scene nodes/objects
//! - **e**: Edges representing relationships/connections
//! - **i**: Incidents representing interactions/physics forces

pub mod vertex;
pub mod edge;
pub mod incident;
pub mod scene_graph;

pub use scene_graph::*;
