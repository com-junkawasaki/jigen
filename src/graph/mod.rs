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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_graph_creation() {
        let scene_graph = SceneGraph::new();
        assert_eq!(scene_graph.graph.node_count(), 0);
        assert_eq!(scene_graph.graph.edge_count(), 0);
    }

    #[test]
    fn test_scene_graph_statistics() {
        let scene_graph = SceneGraph::new();
        let stats = scene_graph.statistics();
        assert_eq!(stats.vertex_count, 0);
        assert_eq!(stats.edge_count, 0);
    }
}
