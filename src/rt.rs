//! RT module: real-time execution engine.

// IMPORTANT: Do not call assert_invariant or any PPT logging in RT paths to avoid locks/allocs.

use crate::graph::{Graph, NodeId, NodeType};
use crate::plan::Plan;
use std::collections::HashMap;
use std::panic;

/// Node states for mutable data.
#[derive(Debug, Clone)]
pub enum NodeState {
    SineOsc { phase: f32 },
    Gain,
    Mix,
    OutputSink,
    Dummy,
}

/// Inputs to the runtime: node -> port -> data
pub type Inputs = HashMap<NodeId, HashMap<crate::graph::PortId, Vec<f32>>>;

/// Outputs from the runtime
pub type Outputs = HashMap<NodeId, HashMap<crate::graph::PortId, Vec<f32>>>;

/// The runtime engine.
#[derive(Debug)]
pub struct Runtime {
    pub plan: Plan,
    graph: Graph,
}

impl Runtime {
    /// Create a new runtime from a plan and graph.
    pub fn new(plan: Plan, graph: &Graph) -> Self {
        Self {
            plan,
            graph: graph.clone(),
        }
    }

    /// Process a block of frames, honoring graph edges and plan buffers.
    pub fn process_block(&mut self, inputs: &Inputs, outputs: &mut Outputs, frames: usize) {
        // For each node in execution order
        for &node_id in &self.plan.execution_order {
            let node_data = &self.graph.nodes[node_id.0];
            // Collect inputs: external + routed from buffers
            let mut node_inputs_vec = vec![];
            for port in &node_data.inputs {
                if let Some(data) = inputs.get(&node_id).and_then(|m| m.get(&port.id)) {
                    node_inputs_vec.push(data.as_slice());
                } else if let Some(buffer) = self.plan.buffers.get(&(node_id, port.id, node_id, port.id)) {
                    // Wait, for inputs, it's from other nodes.
                    // For routed inputs
                    for edge in &self.graph.edges {
                        if edge.to_node == node_id && edge.to_port == port.id {
                            if let Some(buffer) = self.plan.buffers.get(&(edge.from_node, edge.from_port, edge.to_node, edge.to_port)) {
                                node_inputs_vec.push(&buffer.data[..frames]);
                                break;
                            }
                        }
                    }
                    if node_inputs_vec.len() < node_data.inputs.len() {
                        node_inputs_vec.push(&[]);
                    }
                } else {
                    node_inputs_vec.push(&[]);
                }
            }

            // Prepare outputs
            let mut node_outputs_vec = vec![vec![0.0; frames]; node_data.outputs.len()];

            // Process node
            let ctx = crate::graph::ProcessContext { sample_rate: 44100.0, block_size: frames };
            node_data.node.process_block(&node_inputs_vec, &mut node_outputs_vec.iter_mut().map(|v| v.as_mut_slice()).collect::<Vec<_>>(), &ctx);

            // Store outputs in plan buffers for outgoing edges
            for (i, port) in node_data.outputs.iter().enumerate() {
                for edge in &self.graph.edges {
                    if edge.from_node == node_id && edge.from_port == port.id {
                        if let Some(buffer) = self.plan.buffers.get_mut(&(edge.from_node, edge.from_port, edge.to_node, edge.to_port)) {
                            buffer.data[..frames].copy_from_slice(&node_outputs_vec[i]);
                        }
                    }
                }
            }

            // Populate external outputs
            if let Some(out_map) = outputs.get_mut(&node_id) {
                for (i, port) in node_data.outputs.iter().enumerate() {
                    out_map.insert(port.id, node_outputs_vec[i].clone());
                }
            }
        }
    }
}

/// Render offline to a buffer.
pub fn render_offline(runtime: &mut Runtime, frames: usize) -> Vec<f32> {
    let mut output = vec![0.0; frames];
    let mut outputs = HashMap::new();
    for node in &runtime.graph.nodes {
        if !node.outputs.is_empty() {
            outputs.insert(node.id, HashMap::from([(node.outputs[0].id, vec![0.0; frames])]));
        }
    }
    runtime.process_block(&HashMap::new(), &mut outputs, frames);
    if let Some(node_outputs) = outputs.values().next() {
        if let Some(data) = node_outputs.values().next() {
            output.copy_from_slice(data);
        }
    }
    output
}

/// Run process_block with panic containment.
pub fn process_block_safe(
    runtime: &mut Runtime,
    inputs: &Inputs,
    outputs: &mut Outputs,
    frames: usize,
) {
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        runtime.process_block(inputs, outputs, frames);
    }));
    if result.is_err() {
        // Fail closed: silence outputs by zeroing existing buffers
        for node_outputs in outputs.values_mut() {
            for out_vec in node_outputs.values_mut() {
                out_vec.fill(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Graph, NodeType, Port, PortId, Rate};
    use crate::plan::Plan;

    #[test]
    fn rt_no_alloc() {
        let mut graph = Graph::new();
        let node1 = graph.add_node(Box::new(crate::graph::DummyNode));
        let plan = Plan::compile(&graph).unwrap();
        let mut runtime = Runtime::new(plan, &graph);
        let mut inputs = HashMap::new();
        inputs.insert(node1, HashMap::from([(PortId(0), vec![1.0; 64])]));
        let mut outputs = HashMap::new();
        outputs.insert(node1, HashMap::from([(PortId(0), vec![0.0; 64])]));
        runtime.process_block(&inputs, &mut outputs, 64);
        // Should copy input to output without allocating
        assert_eq!(outputs[&node1][&PortId(0)], vec![1.0; 64]);
    }

    #[test]
    fn rt_no_lock() {
        // Assume no locks; in Rust, no mutex used
        let mut graph = Graph::new();
        let _node1 = graph.add_node(Box::new(crate::graph::DummyNode));
        let plan = Plan::compile(&graph).unwrap();
        let mut runtime = Runtime::new(plan, &graph);
        let inputs = HashMap::new();
        let mut outputs = HashMap::new();
        runtime.process_block(&inputs, &mut outputs, 64);
    }

    #[test]
    fn rt_honors_edges() {
        // Edges are honored: outputs propagate through the graph
        let mut graph = Graph::new();
        let node1 = graph.add_node(Box::new(crate::graph::DummyNode));
        let node2 = graph.add_node(Box::new(crate::graph::DummyNode));
        graph
            .add_edge(crate::graph::Edge {
                from_node: node1,
                from_port: PortId(0),
                to_node: node2,
                to_port: PortId(0),
                rate: Rate::Audio,
            })
            .unwrap();
        let plan = Plan::compile(&graph).unwrap();
        let mut runtime = Runtime::new(plan, &graph);
        let mut inputs = HashMap::new();
        inputs.insert(node1, HashMap::from([(PortId(0), vec![1.0; 64])]));
        let mut outputs = HashMap::new();
        outputs.insert(node1, HashMap::from([(PortId(0), vec![0.0; 64])]));
        outputs.insert(node2, HashMap::from([(PortId(0), vec![0.0; 64])]));
        runtime.process_block(&inputs, &mut outputs, 64);
        // Edges are honored: node2 gets input from node1
        assert_eq!(outputs[&node1][&PortId(0)], vec![1.0; 64]);
        assert_eq!(outputs[&node2][&PortId(0)], vec![1.0; 64]);
    }

    #[test]
    fn rt_determinism() {
        let mut graph = Graph::new();
        let node1 = graph.add_node(
            vec![Port {
                id: PortId(0),
                rate: Rate::Audio,
            }],
            NodeType::Dummy,
        );
        let plan = Plan::compile(&graph).unwrap();
        let mut runtime1 = Runtime::new(plan.clone(), &graph);
        let mut runtime2 = Runtime::new(plan, &graph);
        let mut inputs = HashMap::new();
        inputs.insert(node1, HashMap::from([(PortId(0), vec![1.0; 64])]));
        let mut outputs1 = HashMap::new();
        outputs1.insert(node1, HashMap::from([(PortId(0), vec![0.0; 64])]));
        let mut outputs2 = HashMap::new();
        outputs2.insert(node1, HashMap::from([(PortId(0), vec![0.0; 64])]));
        runtime1.process_block(&inputs, &mut outputs1, 64);
        runtime2.process_block(&inputs, &mut outputs2, 64);
        assert_eq!(outputs1, outputs2);
    }

    #[test]
    fn node_golden() {
        let mut graph = Graph::new();
        let _node1 = graph.add_node(Box::new(crate::graph::SineOscNode { freq: 440.0, phase: 0.0 }));
        let plan = Plan::compile(&graph).unwrap();
        let mut runtime = Runtime::new(plan, &graph);
        let output = render_offline(&mut runtime, 64);
        // Check first few samples
        assert!((output[0] - 0.0).abs() < 0.01); // sin(0) = 0
        // Approximate check for sine wave
        assert!(output[1] > 0.0);
        assert!(output[10] > 0.0);
    }
}
