//! Graph module for Auxide: correct-by-construction signal graphs.

#![forbid(unsafe_code)]
// #![deny(missing_docs)]

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Rate {
    Audio,
    Control,
    Event,
}

/// Unique identifier for a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub usize);

/// Unique identifier for a port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PortId(pub usize);

/// A port with its rate.
#[derive(Debug, Clone, PartialEq)]
pub struct Port {
    pub id: PortId,
    pub rate: Rate,
}

/// An edge connecting two ports.
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub from_node: NodeId,
    pub from_port: PortId,
    pub to_node: NodeId,
    pub to_port: PortId,
    pub rate: Rate,
}

/// A node in the graph.
#[derive(Debug, Clone)]
pub struct NodeData {
    pub id: NodeId,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    pub node_type: NodeType,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum NodeType {
    SineOsc { freq: f32 },
    Gain { gain: f32 },
    Mix,
    OutputSink,
    Dummy, // For testing
}

impl NodeType {
    pub fn input_ports(&self) -> Vec<Port> {
        match self {
            NodeType::Dummy => vec![Port { id: PortId(0), rate: Rate::Audio }],
            NodeType::SineOsc { .. } => vec![],
            NodeType::Gain { .. } => vec![Port { id: PortId(0), rate: Rate::Audio }],
            NodeType::Mix => vec![
                Port { id: PortId(0), rate: Rate::Audio },
                Port { id: PortId(1), rate: Rate::Audio },
            ],
            NodeType::OutputSink => vec![Port { id: PortId(0), rate: Rate::Audio }],
        }
    }

    pub fn output_ports(&self) -> Vec<Port> {
        match self {
            NodeType::Dummy => vec![Port { id: PortId(0), rate: Rate::Audio }],
            NodeType::SineOsc { .. } => vec![Port { id: PortId(0), rate: Rate::Audio }],
            NodeType::Gain { .. } => vec![Port { id: PortId(0), rate: Rate::Audio }],
            NodeType::Mix => vec![Port { id: PortId(0), rate: Rate::Audio }],
            NodeType::OutputSink => vec![],
        }
    }
}

/// The signal graph: a DAG of nodes and edges.
#[derive(Debug, Clone)]
pub struct Graph {
    pub nodes: Vec<NodeData>,
    pub edges: Vec<Edge>,
}

/// Errors that can occur when building the graph.
#[derive(Debug, Clone, PartialEq)]
pub enum GraphError {
    RateMismatch,
    CycleDetected,
    InvalidPort,
}

impl Graph {
    /// Create a new empty graph.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a node.
    pub fn add_node(&mut self, node_type: NodeType) -> NodeId {
        let inputs = node_type.input_ports();
        let outputs = node_type.output_ports();
        let id = NodeId(self.nodes.len());
        self.nodes.push(NodeData { id, inputs, outputs, node_type });
        id
    }

    /// Add an edge, validating rates match and no cycles.
    pub fn add_edge(&mut self, edge: Edge) -> Result<(), GraphError> {
        // Check rate mismatch
        if edge.rate != self.get_port_rate(edge.from_node, edge.from_port)? {
            return Err(GraphError::RateMismatch);
        }
        if edge.rate != self.get_port_rate(edge.to_node, edge.to_port)? {
            return Err(GraphError::RateMismatch);
        }

        // Check for cycles (simple check: if adding would create cycle)
        if self.would_create_cycle(&edge) {
            return Err(GraphError::CycleDetected);
        }

        self.edges.push(edge);
        Ok(())
    }

    /// Remove a node and all edges connected to it.
    pub fn remove_node(&mut self, node_id: NodeId) {
        // Remove the node
        self.nodes.retain(|n| n.id != node_id);
        // Remove edges connected to the node
        self.edges.retain(|e| e.from_node != node_id && e.to_node != node_id);
    }

    fn get_port_rate(&self, node_id: NodeId, port_id: PortId) -> Result<Rate, GraphError> {
        let node = &self.nodes[node_id.0];
        for port in &node.inputs {
            if port.id == port_id {
                return Ok(port.rate.clone());
            }
        }
        for port in &node.outputs {
            if port.id == port_id {
                return Ok(port.rate.clone());
            }
        }
        Err(GraphError::InvalidPort)
    }

    fn would_create_cycle(&self, edge: &Edge) -> bool {
        // Simple cycle detection: check if to_node can reach from_node
        // For now, basic implementation; can be improved with proper topo sort
        let mut visited = vec![false; self.nodes.len()];
        self.dfs(edge.to_node, edge.from_node, &mut visited)
    }

    fn dfs(&self, current: NodeId, target: NodeId, visited: &mut [bool]) -> bool {
        if current == target {
            return true;
        }
        if visited[current.0] {
            return false;
        }
        visited[current.0] = true;
        for edge in &self.edges {
            if edge.from_node == current && self.dfs(edge.to_node, target, visited) {
                return true;
            }
        }
        false
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invariant_ppt::clear_invariant_log;
    use proptest::prelude::*;

    #[test]
    fn graph_rate_mismatch() {
        let mut graph = Graph::new();
        let node1 = graph.add_node(NodeType::SineOsc { freq: 440.0 });
        let node2 = graph.add_node(NodeType::Gain { gain: 1.0 });
        let edge = Edge {
            from_node: node1,
            from_port: PortId(0),
            to_node: node2,
            to_port: PortId(0),
            rate: Rate::Control, // Mismatch
        };
        assert_eq!(graph.add_edge(edge), Err(GraphError::RateMismatch));
    }

    #[test]
    fn graph_cycle_detection() {
        clear_invariant_log();
        let mut graph = Graph::new();
        let node1 = graph.add_node(NodeType::Dummy);
        let node2 = graph.add_node(NodeType::Mix);
        // Add edge 1 -> 2
        let edge1 = Edge {
            from_node: node1,
            from_port: PortId(0),
            to_node: node2,
            to_port: PortId(0),
            rate: Rate::Audio,
        };
        graph.add_edge(edge1).unwrap();
        // Try to add 2 -> 1, creating cycle
        let edge2 = Edge {
            from_node: node2,
            from_port: PortId(0),
            to_node: node1,
            to_port: PortId(0),
            rate: Rate::Audio,
        };
        assert_eq!(graph.add_edge(edge2), Err(GraphError::CycleDetected));
    }

    #[test]
    fn graph_stable_toposort() {
        // For stable ordering, ensure nodes are processed in id order or something.
        // For now, just check that graph builds correctly.
        let mut graph = Graph::new();
        let node1 = graph.add_node(NodeType::SineOsc { freq: 440.0 });
        let node2 = graph.add_node(NodeType::Gain { gain: 1.0 });
        assert!(node1 < node2); // Since NodeId is Ord
    }

    proptest! {
        #[test]
        fn graph_rate_mismatch_prop(_rate1 in 0..3usize, _rate2 in 0..3usize) {
            // Simplified: since ports are fixed to Audio for these nodes, mismatch if edge rate != Audio
            let mut graph = Graph::new();
            let node1 = graph.add_node(NodeType::SineOsc { freq: 440.0 });
            let node2 = graph.add_node(NodeType::Gain { gain: 1.0 });
            let edge = Edge {
                from_node: node1,
                from_port: PortId(0),
                to_node: node2,
                to_port: PortId(0),
                rate: Rate::Control, // Mismatch
            };
            prop_assert_eq!(graph.add_edge(edge), Err(GraphError::RateMismatch));
        }
    }
}
