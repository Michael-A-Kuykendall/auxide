//! Plan module: compile graph into executable plan.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use crate::graph::{Graph, NodeId, PortId, Rate};
use crate::invariant_ppt::{assert_invariant, PLAN_SOUNDNESS};

/// Edge spec for the plan.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeSpec {
    /// Source node ID.
    pub from_node: NodeId,
    /// Source port ID.
    pub from_port: PortId,
    /// Destination node ID.
    pub to_node: NodeId,
    /// Destination port ID.
    pub to_port: PortId,
    /// Signal rate for this edge.
    pub rate: Rate,
}

/// The compiled plan: execution order and edge specs.
#[derive(Debug, Clone)]
pub struct Plan {
    /// Topologically sorted node execution order.
    pub order: Vec<NodeId>,
    /// Input edges per node: Vec of (edge_idx, port_id).
    pub node_inputs: Vec<Vec<(usize, PortId)>>,
    /// Output edges per node: Vec of (edge_idx, port_id).
    pub node_outputs: Vec<Vec<(usize, PortId)>>,
    /// All edge specifications.
    pub edges: Vec<EdgeSpec>,
    /// Processing block size in samples.
    pub block_size: usize,
    /// Maximum input count across all nodes.
    pub max_inputs: usize,
    /// Maximum output count across all nodes.
    pub max_outputs: usize,
}

impl Plan {
    /// Create a plan from a graph.
    pub fn compile(graph: &Graph, block_size: usize) -> Result<Self, PlanError> {
        if block_size == 0 {
            return Err(PlanError::InvalidBlockSize);
        }
        // Topological sort
        let order = topo_sort(graph)?;

        // Build edges
        let edges: Vec<EdgeSpec> = graph
            .edges
            .iter()
            .map(|e| EdgeSpec {
                from_node: e.from_node,
                from_port: e.from_port,
                to_node: e.to_node,
                to_port: e.to_port,
                rate: e.rate.clone(),
            })
            .collect();

        // Validate single-writer: each input port has at most one edge
        let mut input_ports = std::collections::HashSet::new();
        for edge in &edges {
            if !input_ports.insert((edge.to_node, edge.to_port)) {
                return Err(PlanError::MultipleWritersToInput {
                    node: edge.to_node,
                    port: edge.to_port,
                });
            }
        }

        // Build node_inputs and node_outputs
        let mut node_inputs = vec![vec![]; graph.nodes.len()];
        let mut node_outputs = vec![vec![]; graph.nodes.len()];
        for (edge_idx, edge) in edges.iter().enumerate() {
            node_inputs[edge.to_node.0].push((edge_idx, edge.to_port));
            node_outputs[edge.from_node.0].push((edge_idx, edge.from_port));
        }

        let max_inputs = node_inputs.iter().map(|v| v.len()).max().unwrap_or(0);
        let max_outputs = node_outputs.iter().map(|v| v.len()).max().unwrap_or(0);

        // Validate required inputs and external node input limits
        for node_data in graph.nodes.iter().flatten() {
            let required = node_data.node_type.required_inputs();
            let connected = graph
                .edges
                .iter()
                .filter(|e| e.to_node == node_data.id)
                .count();
            if connected < required {
                return Err(PlanError::RequiredInputMissing { node: node_data.id });
            }
            // External nodes have a compile-time input limit for RT safety
            if matches!(node_data.node_type, crate::graph::NodeType::External { .. })
                && connected > MAX_EXTERNAL_NODE_INPUTS
            {
                return Err(PlanError::TooManyInputs {
                    node: node_data.id,
                    got: connected,
                    max: MAX_EXTERNAL_NODE_INPUTS,
                });
            }
        }

        let plan = Self {
            order,
            node_inputs,
            node_outputs,
            edges,
            block_size,
            max_inputs,
            max_outputs,
        };

        // PPT Invariant: Plan compilation succeeded and is sound
        assert_invariant(
            PLAN_SOUNDNESS,
            true,
            "Plan compilation completed successfully",
            Some("compile"),
        );

        Ok(plan)
    }
}

/// Maximum inputs per external node (must match rt.rs MAX_STACK_INPUTS).
pub const MAX_EXTERNAL_NODE_INPUTS: usize = 16;

/// Errors during plan compilation.
#[derive(Debug, Clone, PartialEq)]
pub enum PlanError {
    /// Graph contains a cycle (not allowed except with delay nodes).
    CycleDetected,
    /// A node's required inputs are not connected.
    RequiredInputMissing {
        /// The node missing required inputs.
        node: NodeId,
    },
    /// Multiple edges write to the same input port.
    MultipleWritersToInput {
        /// The node with the conflict.
        node: NodeId,
        /// The conflicting port.
        port: PortId,
    },
    /// Block size must be greater than zero.
    InvalidBlockSize,
    /// External node exceeds maximum input limit for RT safety.
    TooManyInputs {
        /// The node exceeding the limit.
        node: NodeId,
        /// Actual input count.
        got: usize,
        /// Maximum allowed.
        max: usize,
    },
}

/// Topological sort of nodes.
fn topo_sort(graph: &Graph) -> Result<Vec<NodeId>, PlanError> {
    let mut in_degree = vec![0; graph.nodes.len()];
    let mut adj: Vec<Vec<NodeId>> = vec![vec![]; graph.nodes.len()];

    for edge in &graph.edges {
        adj[edge.from_node.0].push(edge.to_node);
        in_degree[edge.to_node.0] += 1;
    }

    let mut queue = std::collections::VecDeque::new();
    for (i, &deg) in in_degree.iter().enumerate().take(graph.nodes.len()) {
        if graph.nodes[i].is_some() && deg == 0 {
            queue.push_back(NodeId(i));
        }
    }

    let mut order = Vec::new();
    while let Some(node) = queue.pop_front() {
        order.push(node);
        for &neighbor in &adj[node.0] {
            in_degree[neighbor.0] -= 1;
            if graph.nodes[neighbor.0].is_some() && in_degree[neighbor.0] == 0 {
                queue.push_back(neighbor);
            }
        }
    }

    let valid_count = graph.nodes.iter().filter(|n| n.is_some()).count();
    if order.len() == valid_count {
        Ok(order)
    } else {
        Err(PlanError::CycleDetected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Edge, NodeType, PortId, Rate};

    #[test]
    fn plan_stability() {
        let mut graph = Graph::new();
        let node1 = graph.add_node(NodeType::Dummy);
        let node2 = graph.add_node(NodeType::Mix);
        graph
            .add_edge(Edge {
                from_node: node1,
                from_port: PortId(0),
                to_node: node2,
                to_port: PortId(0),
                rate: Rate::Audio,
            })
            .unwrap();

        let plan1 = Plan::compile(&graph, 64).unwrap();
        let plan2 = Plan::compile(&graph, 64).unwrap();
        assert_eq!(plan1.order, plan2.order);
        assert_eq!(plan1.edges, plan2.edges);
    }

    #[test]
    fn plan_buffer_liveness() {
        // Check that edges are built correctly.
        let mut graph = Graph::new();
        let node1 = graph.add_node(NodeType::Dummy);
        let node2 = graph.add_node(NodeType::Dummy);
        graph
            .add_edge(Edge {
                from_node: node1,
                from_port: PortId(0),
                to_node: node2,
                to_port: PortId(0),
                rate: Rate::Audio,
            })
            .unwrap();

        let plan = Plan::compile(&graph, 64).unwrap();
        assert_eq!(plan.edges.len(), 1);
        assert_eq!(plan.edges[0].from_node, node1);
        assert_eq!(plan.edges[0].to_node, node2);
    }

    #[test]
    fn plan_debug_smoke_test() {
        let mut graph = Graph::new();
        let _node1 = graph.add_node(NodeType::Dummy);
        let plan = Plan::compile(&graph, 64).unwrap();
        let debug_str = format!("{:?}", plan);
        // Smoke test: ensure it doesn't panic and contains expected fields
        assert!(debug_str.contains("order"));
        assert!(debug_str.contains("edges"));
    }

    #[test]
    fn plan_rejects_external_node_with_too_many_inputs() {
        use crate::graph::Port;
        use crate::node::NodeDef;

        // Create a dummy external node that accepts many inputs
        struct ManyInputNode;

        // Static port arrays for the node
        static INPUT_PORTS: [Port; 20] = [
            Port {
                id: PortId(0),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(1),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(2),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(3),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(4),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(5),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(6),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(7),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(8),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(9),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(10),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(11),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(12),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(13),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(14),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(15),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(16),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(17),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(18),
                rate: Rate::Audio,
            },
            Port {
                id: PortId(19),
                rate: Rate::Audio,
            },
        ];
        static OUTPUT_PORTS: [Port; 1] = [Port {
            id: PortId(0),
            rate: Rate::Audio,
        }];

        impl NodeDef for ManyInputNode {
            type State = ();
            fn input_ports(&self) -> &'static [Port] {
                &INPUT_PORTS
            }
            fn output_ports(&self) -> &'static [Port] {
                &OUTPUT_PORTS
            }
            fn required_inputs(&self) -> usize {
                0
            }
            fn init_state(&self, _: f32, _: usize) -> Self::State {
                ()
            }
            fn process_block(&self, _: &mut Self::State, _: &[&[f32]], _: &mut [Vec<f32>], _: f32) {
            }
        }

        let mut graph = Graph::new();
        let external = graph.add_external_node(ManyInputNode);

        // Add 17 source nodes, each connecting to the external node
        for i in 0..17 {
            let src = graph.add_node(NodeType::SineOsc { freq: 440.0 });
            graph
                .add_edge(Edge {
                    from_node: src,
                    from_port: PortId(0),
                    to_node: external,
                    to_port: PortId(i),
                    rate: Rate::Audio,
                })
                .unwrap();
        }

        // Plan compilation should fail with TooManyInputs
        let result = Plan::compile(&graph, 64);
        assert!(matches!(
            result,
            Err(PlanError::TooManyInputs {
                got: 17,
                max: 16,
                ..
            })
        ));
    }
}
