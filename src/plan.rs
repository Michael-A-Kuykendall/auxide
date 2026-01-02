//! Plan module: compile graph into executable plan.

use crate::graph::{Graph, NodeId, PortId, Rate};

/// Edge spec for the plan.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeSpec {
    pub from_node: NodeId,
    pub from_port: PortId,
    pub to_node: NodeId,
    pub to_port: PortId,
    pub rate: Rate,
}

/// The compiled plan: execution order and edge specs.
#[derive(Debug, Clone)]
pub struct Plan {
    pub order: Vec<NodeId>,
    pub node_inputs: Vec<Vec<(usize, PortId)>>, // (edge_idx, port)
    pub node_outputs: Vec<Vec<(usize, PortId)>>, // (edge_idx, port)
    pub edges: Vec<EdgeSpec>,
    pub block_size: usize,
}

impl Plan {
    /// Create a plan from a graph.
    pub fn compile(graph: &Graph, block_size: usize) -> Result<Self, PlanError> {
        // Topological sort
        let order = topo_sort(graph)?;

        // Build edges
        let edges: Vec<EdgeSpec> = graph.edges.iter().map(|e| EdgeSpec {
            from_node: e.from_node,
            from_port: e.from_port,
            to_node: e.to_node,
            to_port: e.to_port,
            rate: e.rate.clone(),
        }).collect();

        // Build node_inputs and node_outputs
        let mut node_inputs = vec![vec![]; graph.nodes.len()];
        let mut node_outputs = vec![vec![]; graph.nodes.len()];
        for (edge_idx, edge) in edges.iter().enumerate() {
            node_inputs[edge.to_node.0].push((edge_idx, edge.to_port));
            node_outputs[edge.from_node.0].push((edge_idx, edge.from_port));
        }

        let plan = Self {
            order,
            node_inputs,
            node_outputs,
            edges,
            block_size,
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
}
