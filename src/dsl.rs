//! DSL module: builder API for graphs.

#![forbid(unsafe_code)]
// #![deny(missing_docs)]

use crate::graph::{Graph, GraphError, NodeId, NodeType, PortId, Rate};
use std::collections::HashMap;

/// Handle to a node in the builder.
#[derive(Debug, Clone, Copy)]
pub struct NodeHandle(pub NodeId);

/// The graph builder.
#[derive(Debug)]
pub struct GraphBuilder {
    graph: Graph,
    node_names: HashMap<String, NodeId>, // For named nodes, optional
}

impl GraphBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            node_names: HashMap::new(),
        }
    }

    /// Add a node with type.
    pub fn node(&mut self, node_type: NodeType) -> NodeHandle {
        let id = self.graph.add_node(node_type);
        NodeHandle(id)
    }

    /// Add a named node.
    pub fn node_named(&mut self, name: &str, node_type: NodeType) -> NodeHandle {
        let handle = self.node(node_type);
        self.node_names.insert(name.to_string(), handle.0);
        handle
    }

    /// Connect two ports.
    pub fn connect(
        &mut self,
        from: NodeHandle,
        from_port: PortId,
        to: NodeHandle,
        to_port: PortId,
        rate: Rate,
    ) -> Result<(), DslError> {
        let edge = crate::graph::Edge {
            from_node: from.0,
            from_port,
            to_node: to.0,
            to_port,
            rate,
        };
        self.graph.add_edge(edge).map_err(DslError::Graph)?;
        Ok(())
    }

    /// Build the graph.
    pub fn build(self) -> Result<Graph, DslError> {
        Ok(self.graph)
    }

    /// Get a node by name.
    pub fn get_node_by_name(&self, name: &str) -> Option<NodeId> {
        self.node_names.get(name).copied()
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// DSL-specific errors.
#[derive(Debug, Clone, PartialEq)]
pub enum DslError {
    Graph(GraphError),
    MissingNode(String),
    UnboundPort,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dsl_equivalence() {
        // Build graph via DSL and manually, check equivalence
        let mut builder = GraphBuilder::new();
        let node1 = builder.node(NodeType::Dummy);
        let node2 = builder.node(NodeType::Dummy);
        builder
            .connect(node1, PortId(0), node2, PortId(0), Rate::Audio)
            .unwrap();
        let dsl_graph = builder.build().unwrap();

        let mut manual_graph = Graph::new();
        let m_node1 = manual_graph.add_node(NodeType::Dummy);
        let m_node2 = manual_graph.add_node(NodeType::Dummy);
        manual_graph
            .add_edge(crate::graph::Edge {
                from_node: m_node1,
                from_port: PortId(0),
                to_node: m_node2,
                to_port: PortId(0),
                rate: Rate::Audio,
            })
            .unwrap();

        // Compare nodes and edges
        assert_eq!(dsl_graph.nodes.len(), manual_graph.nodes.len());
        assert_eq!(dsl_graph.edges.len(), manual_graph.edges.len());
    }

    #[test]
    fn ui_tests() {
        // Test error cases
        let mut builder = GraphBuilder::new();
        let node1 = builder.node(NodeType::Dummy);
        let node2 = builder.node(NodeType::Dummy);
        let err = builder
            .connect(node1, PortId(0), node2, PortId(0), Rate::Control)
            .unwrap_err();
        assert_eq!(err, DslError::Graph(GraphError::RateMismatch));
    }
}
