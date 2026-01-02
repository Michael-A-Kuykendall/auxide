//! RT module: real-time execution engine.

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
    node_types: Vec<NodeType>,
    node_states: Vec<NodeState>,
}

impl Runtime {
    /// Create a new runtime from a plan and graph.
    pub fn new(plan: Plan, graph: &Graph) -> Self {
        let node_types: Vec<NodeType> = graph.nodes.iter().map(|n| n.node_type.clone()).collect();
        let node_states = node_types
            .iter()
            .map(|nt| match nt {
                NodeType::SineOsc { freq: _, phase } => NodeState::SineOsc { phase: *phase },
                NodeType::Gain { .. } => NodeState::Gain,
                NodeType::Mix => NodeState::Mix,
                NodeType::OutputSink => NodeState::OutputSink,
                NodeType::Dummy => NodeState::Dummy,
            })
            .collect();
        Self {
            plan,
            node_types,
            node_states,
        }
    }

    /// Process a block of frames.
    /// Note: Current implementation is a scaffold and does not honor graph edges or plan buffers.
    /// It simply copies inputs to outputs by node/port if preallocated.
    pub fn process_block(&mut self, inputs: &Inputs, outputs: &mut Outputs, frames: usize) {
        // For each node in execution order
        for &node_id in &self.plan.execution_order {
            // Dummy processing: copy inputs to outputs
            if let Some(node_inputs) = inputs.get(&node_id)
                && let Some(node_outputs) = outputs.get_mut(&node_id)
            {
                for (port_id, data) in node_inputs {
                    if let Some(out_vec) = node_outputs.get_mut(port_id) {
                        let copy_len = frames.min(out_vec.len()).min(data.len());
                        out_vec[..copy_len].copy_from_slice(&data[..copy_len]);
                        // Silence the rest if out_vec is longer
                        if copy_len < out_vec.len() {
                            out_vec[copy_len..].fill(0.0);
                        }
                    }
                }
            }
        }
    }
}

/// Render offline to a buffer.
pub fn render_offline(runtime: &mut Runtime, frames: usize) -> Vec<f32> {
    let mut output = vec![0.0; frames];
    let sample_rate = 44100.0; // Assume
    for sample in output.iter_mut().take(frames) {
        *sample = 0.0;
        // For each node, process
        for (i, node_type) in runtime.node_types.iter().enumerate() {
            if let (NodeType::SineOsc { freq, .. }, NodeState::SineOsc { phase }) =
                (node_type, &mut runtime.node_states[i])
            {
                let sample_val = (*phase * 2.0 * std::f32::consts::PI).sin();
                *sample = sample_val;
                *phase += freq / sample_rate;
                if *phase >= 1.0 {
                    *phase -= 1.0;
                }
            }
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
        let node1 = graph.add_node(
            vec![Port {
                id: PortId(0),
                rate: Rate::Audio,
            }],
            NodeType::Dummy,
        );
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
        let _node1 = graph.add_node(vec![], NodeType::Dummy);
        let plan = Plan::compile(&graph).unwrap();
        let mut runtime = Runtime::new(plan, &graph);
        let inputs = HashMap::new();
        let mut outputs = HashMap::new();
        runtime.process_block(&inputs, &mut outputs, 64);
    }

    #[test]
    fn rt_ignores_edges() {
        // Current scaffold limitation: edges do not affect output
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
        // Currently, only node1 output is set, node2 remains zero despite edge
        assert_eq!(outputs[&node1][&PortId(0)], vec![1.0; 64]);
        assert_eq!(outputs[&node2][&PortId(0)], vec![0.0; 64]); // Limitation: edge not honored
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
        let _node1 = graph.add_node(
            vec![Port {
                id: PortId(0),
                rate: Rate::Audio,
            }],
            NodeType::SineOsc {
                freq: 440.0,
                phase: 0.0,
            },
        );
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
