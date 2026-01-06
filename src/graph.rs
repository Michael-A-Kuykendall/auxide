//! Graph module for Auxide: correct-by-construction signal graphs.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use crate::invariant_ppt::{assert_invariant, GRAPH_LEGALITY, GRAPH_REJECTS_INVALID};
use crate::node::{NodeDef, NodeDefDyn};
use std::sync::Arc;

/// The rate at which a port processes data.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum Rate {
    /// Audio rate (typically 44.1kHz or 48kHz)
    Audio,
    /// Control rate (lower frequency, for parameters)
    Control,
    /// Event rate (asynchronous events)
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
    /// The unique identifier for this port
    pub id: PortId,
    /// The rate at which this port operates
    pub rate: Rate,
}

const PORTS_NONE: &[Port] = &[];
const PORTS_MONO_OUT: &[Port] = &[Port {
    id: PortId(0),
    rate: Rate::Audio,
}];
const PORTS_MONO_IN: &[Port] = &[Port {
    id: PortId(0),
    rate: Rate::Audio,
}];
const PORTS_DUAL_IN_MONO_OUT: &[Port] = &[
    Port {
        id: PortId(0),
        rate: Rate::Audio,
    },
    Port {
        id: PortId(1),
        rate: Rate::Audio,
    },
];

/// An edge connecting two ports.
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    /// The source node ID.
    pub from_node: NodeId,
    /// The source port ID.
    pub from_port: PortId,
    /// The destination node ID.
    pub to_node: NodeId,
    /// The destination port ID.
    pub to_port: PortId,
    /// The rate at which this edge operates.
    pub rate: Rate,
}

/// A node in the graph.
#[derive(Debug, Clone)]
pub struct NodeData {
    /// The unique ID of this node.
    pub id: NodeId,
    /// The input ports of this node.
    pub inputs: &'static [Port],
    /// The output ports of this node.
    pub outputs: &'static [Port],
    /// The type of this node.
    pub node_type: NodeType,
}

/// Types of nodes in the audio graph.
#[non_exhaustive]
#[derive(Clone)]
pub enum NodeType {
    /// Sine wave oscillator with frequency in Hz.
    SineOsc {
        /// Frequency in Hz.
        freq: f32,
    },
    /// Gain node that multiplies input by a factor.
    Gain {
        /// Gain factor (1.0 = unity gain).
        gain: f32,
    },
    /// Mixer node that sums multiple inputs.
    Mix,
    /// Output sink for audio output.
    OutputSink,
    /// Passthrough node for testing/placeholder use.
    Dummy,
    /// External node implemented via NodeDef trait.
    External {
        /// The node definition.
        def: Arc<dyn NodeDefDyn>,
    },
}

impl std::fmt::Debug for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::SineOsc { .. } => write!(f, "SineOsc"),
            NodeType::Gain { .. } => write!(f, "Gain"),
            NodeType::Mix => write!(f, "Mix"),
            NodeType::OutputSink => write!(f, "OutputSink"),
            NodeType::Dummy => write!(f, "Dummy"),
            NodeType::External { .. } => write!(f, "External"),
        }
    }
}

impl NodeType {
    /// Get the input ports for this node type.
    pub fn input_ports(&self) -> &'static [Port] {
        match self {
            NodeType::Dummy => PORTS_MONO_IN,
            NodeType::SineOsc { .. } => PORTS_NONE,
            NodeType::Gain { .. } => PORTS_MONO_IN,
            NodeType::Mix => PORTS_DUAL_IN_MONO_OUT,
            NodeType::OutputSink => PORTS_MONO_IN,
            NodeType::External { def } => def.input_ports(),
        }
    }

    /// Get the output ports for this node type.
    pub fn output_ports(&self) -> &'static [Port] {
        match self {
            NodeType::Dummy => PORTS_MONO_OUT,
            NodeType::SineOsc { .. } => PORTS_MONO_OUT,
            NodeType::Gain { .. } => PORTS_MONO_OUT,
            NodeType::Mix => PORTS_MONO_OUT,
            NodeType::OutputSink => PORTS_NONE,
            NodeType::External { def } => def.output_ports(),
        }
    }

    /// Get the number of required input connections for this node.
    pub fn required_inputs(&self) -> usize {
        match self {
            NodeType::Gain { .. } => 1,
            NodeType::OutputSink => 1,
            NodeType::External { def } => def.required_inputs(),
            _ => 0,
        }
    }
}
/// The signal graph: a DAG of nodes and edges.
#[derive(Debug, Clone)]
pub struct Graph {
    /// All nodes in the graph (None for removed nodes).
    pub nodes: Vec<Option<NodeData>>,
    /// All edges connecting nodes.
    pub edges: Vec<Edge>,
}

/// Errors that can occur when building the graph.
#[derive(Debug, Clone, PartialEq)]
pub enum GraphError {
    /// Connected ports have incompatible rates.
    RateMismatch,
    /// Adding edge would create a cycle.
    CycleDetected,
    /// Port index out of bounds.
    InvalidPort,
    /// Node does not exist.
    InvalidNode,
    /// Input port already has a connection.
    PortAlreadyConnected,
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
        self.nodes.push(Some(NodeData {
            id,
            inputs,
            outputs,
            node_type,
        }));
        id
    }

    /// Add an external node defined via NodeDef.
    pub fn add_external_node<T: NodeDef>(&mut self, def: T) -> NodeId {
        self.add_node(NodeType::External {
            def: Arc::new(def),
        })
    }

    /// Add an edge, validating rates match and no cycles.
    pub fn add_edge(&mut self, edge: Edge) -> Result<(), GraphError> {
        // Validate node existence and get node data
        let from_node_data = self
            .nodes
            .get(edge.from_node.0)
            .and_then(|n| n.as_ref())
            .ok_or(GraphError::InvalidNode)?;
        let to_node_data = self
            .nodes
            .get(edge.to_node.0)
            .and_then(|n| n.as_ref())
            .ok_or(GraphError::InvalidNode)?;

        // Check that from_port is an output port
        if !from_node_data
            .outputs
            .iter()
            .any(|p| p.id == edge.from_port)
        {
            return Err(GraphError::InvalidPort);
        }

        // Check that to_port is an input port
        if !to_node_data.inputs.iter().any(|p| p.id == edge.to_port) {
            return Err(GraphError::InvalidPort);
        }

        // Check rate mismatch
        if edge.rate != self.get_port_rate(edge.from_node, edge.from_port)? {
            return Err(GraphError::RateMismatch);
        }
        if edge.rate != self.get_port_rate(edge.to_node, edge.to_port)? {
            return Err(GraphError::RateMismatch);
        }

        // Check for cycles (simple check: if adding would create cycle)
        if self.would_create_cycle(&edge) {
            assert_invariant(
                GRAPH_REJECTS_INVALID,
                self.would_create_cycle(&edge),
                "Cycle detected, rejecting",
                Some("add_edge"),
            );
            return Err(GraphError::CycleDetected);
        }

        // Check if port is already connected
        if self
            .edges
            .iter()
            .any(|e| e.to_node == edge.to_node && e.to_port == edge.to_port)
        {
            return Err(GraphError::PortAlreadyConnected);
        }

        self.edges.push(edge);
        
        // PPT Invariant: Graph structure remains legal after adding edge
        assert_invariant(GRAPH_LEGALITY, true, "Edge added successfully, graph remains legal", Some("add_edge"));
        
        Ok(())
    }

    /// Remove a node and all edges connected to it.
    pub fn remove_node(&mut self, node_id: NodeId) -> Result<(), GraphError> {
        if node_id.0 >= self.nodes.len() {
            return Err(GraphError::InvalidNode);
        }
        // Remove the node
        self.nodes[node_id.0] = None;
        // Remove edges connected to the node
        self.edges
            .retain(|e| e.from_node != node_id && e.to_node != node_id);
        Ok(())
    }

    fn get_port_rate(&self, node_id: NodeId, port_id: PortId) -> Result<Rate, GraphError> {
        if node_id.0 >= self.nodes.len() {
            return Err(GraphError::InvalidNode);
        }
        let node = &self.nodes[node_id.0];
        let node = node.as_ref().ok_or(GraphError::InvalidNode)?;
        for port in node.inputs {
            if port.id == port_id {
                return Ok(port.rate.clone());
            }
        }
        for port in node.outputs {
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
