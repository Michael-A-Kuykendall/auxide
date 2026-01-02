//! Plan module: compile graph into executable plan.

use crate::graph::{Graph, NodeId, PortId};
use std::collections::HashMap;

/// A buffer for signal data.
#[derive(Debug, Clone, PartialEq)]
pub struct Buffer {
    pub data: Vec<f32>, // Assume f32 for audio
}

/// The compiled plan: execution order and buffers.
#[derive(Debug, Clone)]
pub struct Plan {
    pub execution_order: Vec<NodeId>,
    pub buffers: HashMap<(NodeId, PortId, NodeId, PortId), Buffer>, // Per edge buffer
}

impl Plan {
    /// Create a plan from a graph.
    pub fn compile(graph: &Graph) -> Result<Self, PlanError> {
        // Topological sort
        let execution_order = topo_sort(graph)?;

        // Allocate buffers: one per edge
        let mut buffers = HashMap::new();
        for edge in &graph.edges {
            let buffer = Buffer {
                data: vec![0.0; 1024],
            }; // Fixed size for now
            buffers.insert(
                (edge.from_node, edge.from_port, edge.to_node, edge.to_port),
                buffer,
            );
        }

        let plan = Self {
            execution_order,
            buffers,
        };
        Ok(plan)
    }
}

/// Errors during plan compilation.
#[derive(Debug, Clone, PartialEq)]
pub enum PlanError {
    CycleDetected,
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
    for (i, &deg) in in_degree.iter().enumerate() {
        if deg == 0 {
            queue.push_back(NodeId(i));
        }
    }

    let mut order = Vec::new();
    while let Some(node) = queue.pop_front() {
        order.push(node);
        for &neighbor in &adj[node.0] {
            in_degree[neighbor.0] -= 1;
            if in_degree[neighbor.0] == 0 {
                queue.push_back(neighbor);
            }
        }
    }

    if order.len() == graph.nodes.len() {
        Ok(order)
    } else {
        Err(PlanError::CycleDetected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Edge, NodeType, Port, PortId, Rate};

    #[test]
    fn plan_stability() {
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
        graph
            .add_edge(Edge {
                from_node: node1,
                from_port: PortId(0),
                to_node: node2,
                to_port: PortId(0),
                rate: Rate::Audio,
            })
            .unwrap();

        let plan1 = Plan::compile(&graph).unwrap();
        let plan2 = Plan::compile(&graph).unwrap();
        assert_eq!(plan1.execution_order, plan2.execution_order);
        assert_eq!(plan1.buffers, plan2.buffers);
    }

    #[test]
    fn plan_buffer_liveness() {
        // Check that buffers are allocated correctly and no extras.
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
                rate: Rate::Audio,
            }],
            NodeType::Dummy,
        );
        graph
            .add_edge(Edge {
                from_node: node1,
                from_port: PortId(0),
                to_node: node2,
                to_port: PortId(0),
                rate: Rate::Audio,
            })
            .unwrap();

        let plan = Plan::compile(&graph).unwrap();
        assert_eq!(plan.buffers.len(), 1);
        assert!(
            plan.buffers
                .contains_key(&(node1, PortId(0), node2, PortId(0)))
        );
    }

    #[test]
    fn plan_debug_smoke_test() {
        let mut graph = Graph::new();
        let _node1 = graph.add_node(vec![], NodeType::Dummy);
        let plan = Plan::compile(&graph).unwrap();
        let debug_str = format!("{:?}", plan);
        // Smoke test: ensure it doesn't panic and contains expected fields
        assert!(debug_str.contains("execution_order"));
        assert!(debug_str.contains("buffers"));
    }
}
