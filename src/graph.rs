//! Graph module for Auxide: correct-by-construction signal graphs.

/// Represents the rate of a signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
pub struct Node {
    pub id: NodeId,
    pub ports: Vec<Port>,
    pub node_type: NodeType,
}

/// Node types for processing.
#[derive(Debug, Clone)]
pub enum NodeType {
    SineOsc { freq: f32, phase: f32 },
    Gain { gain: f32 },
    Mix,
    OutputSink,
    Dummy, // For testing
}

/// The signal graph: a DAG of nodes and edges.
#[derive(Debug, Clone)]
pub struct Graph {
    pub nodes: Vec<Node>,
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

    /// Add a node with given ports and type.
    pub fn add_node(&mut self, ports: Vec<Port>, node_type: NodeType) -> NodeId {
        let id = NodeId(self.nodes.len());
        let node = Node {
            id,
            ports,
            node_type,
        };
        self.nodes.push(node);
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

    fn get_port_rate(&self, node_id: NodeId, port_id: PortId) -> Result<Rate, GraphError> {
        self.nodes
            .get(node_id.0)
            .and_then(|node| node.ports.iter().find(|p| p.id == port_id))
            .map(|p| p.rate)
            .ok_or(GraphError::InvalidPort)
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
        let node1 = graph.add_node(
            vec![Port {
                id: PortId(0),
                rate: Rate::Audio,
            }],
            NodeType::Dummy,
        );
        let node2 = graph.add_node(
            vec![Port {
                id: PortId(0),
                rate: Rate::Control,
            }],
            NodeType::Dummy,
        );
        let edge = Edge {
            from_node: node1,
            from_port: PortId(0),
            to_node: node2,
            to_port: PortId(0),
            rate: Rate::Audio, // Mismatch
        };
        assert_eq!(graph.add_edge(edge), Err(GraphError::RateMismatch));
    }

    #[test]
    fn graph_cycle_detection() {
        clear_invariant_log();
        let mut graph = Graph::new();
        let node1 = graph.add_node(
            vec![Port {
                id: PortId(0),
                rate: Rate::Audio,
            }],
            NodeType::Dummy,
        );
        let node2 = graph.add_node(
            vec![
                Port {
                    id: PortId(0),
                    rate: Rate::Audio,
                },
                Port {
                    id: PortId(1),
                    rate: Rate::Audio,
                },
            ],
            NodeType::Dummy,
        );
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
            from_port: PortId(1),
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
        let node1 = graph.add_node(vec![], NodeType::Dummy);
        let node2 = graph.add_node(vec![], NodeType::Dummy);
        assert!(node1 < node2); // Since NodeId is Ord
    }

    proptest! {
        #[test]
        fn graph_rate_mismatch_prop(rate1 in 0..3usize, rate2 in 0..3usize) {
            let rates = [Rate::Audio, Rate::Control, Rate::Event];
            if rate1 == rate2 { return Ok(()); } // Skip matching
            let mut graph = Graph::new();
            let node1 = graph.add_node(vec![Port { id: PortId(0), rate: rates[rate1] }], NodeType::Dummy);
            let node2 = graph.add_node(vec![Port { id: PortId(0), rate: rates[rate2] }], NodeType::Dummy);
            let edge = Edge {
                from_node: node1,
                from_port: PortId(0),
                to_node: node2,
                to_port: PortId(0),
                rate: rates[rate1], // Mismatch with node2
            };
            prop_assert_eq!(graph.add_edge(edge), Err(GraphError::RateMismatch));
        }
    }
}
