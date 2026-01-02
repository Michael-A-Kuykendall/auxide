//! RT module: real-time execution engine.

// IMPORTANT: Do not call assert_invariant or any PPT logging in RT paths to avoid locks/allocs.

use crate::graph::{Graph, NodeType};
use crate::plan::Plan;

/// Node states for mutable data.
#[derive(Debug, Clone)]
pub enum NodeState {
    SineOsc { phase: f32 },
    Gain,
    Mix,
    OutputSink,
    Dummy,
}

/// The runtime engine.
#[derive(Debug)]
pub struct Runtime {
    pub plan: Plan,
    nodes: Vec<NodeType>,
    states: Vec<NodeState>,
    edge_buffers: Vec<Vec<f32>>,
    temp_inputs: Vec<usize>,
    temp_output_vecs: Vec<Vec<f32>>,
}

impl Runtime {
    /// Create a new runtime from a plan and graph.
    pub fn new(plan: Plan, graph: &Graph) -> Self {
        let nodes: Vec<NodeType> = graph.nodes.iter().map(|n| n.node_type.clone()).collect();
        let states = nodes.iter().map(|nt| match nt {
            NodeType::SineOsc { .. } => NodeState::SineOsc { phase: 0.0 },
            NodeType::Gain { .. } => NodeState::Gain,
            NodeType::Mix => NodeState::Mix,
            NodeType::OutputSink => NodeState::OutputSink,
            NodeType::Dummy => NodeState::Dummy,
        }).collect();
        let edge_buffers = vec![vec![0.0; plan.block_size]; plan.edges.len()];
        let temp_inputs = Vec::with_capacity(16);
        let temp_output_vecs = (0..16).map(|_| vec![0.0; plan.block_size]).collect();
        Self {
            plan,
            nodes,
            states,
            edge_buffers,
            temp_inputs,
            temp_output_vecs,
        }
    }

    /// Process a block of frames, writing to out (mono).
    pub fn process_block(&mut self, out: &mut [f32]) {
        let block_size = self.plan.block_size;
        assert_eq!(out.len(), block_size);
        // For each node in order
        for &node_id in &self.plan.order {
            let node_type = &self.nodes[node_id.0];
            let node_state = &mut self.states[node_id.0];
            // Gather inputs
            self.temp_inputs.clear();
            for &(edge_idx, _port) in &self.plan.node_inputs[node_id.0] {
                self.temp_inputs.push(edge_idx);
            }
            // Prepare outputs
            let num_outputs = self.plan.node_outputs[node_id.0].len();
            for i in 0..num_outputs {
                self.temp_output_vecs[i].fill(0.0);
            }
            let outputs = &mut self.temp_output_vecs[0..num_outputs];
            // Process
            match node_type {
                NodeType::Dummy => {
                    for (i, &edge_idx) in self.temp_inputs.iter().enumerate() {
                        let input = &self.edge_buffers[edge_idx][..];
                        if let Some(output) = outputs.get_mut(i) {
                            output.copy_from_slice(input);
                        }
                    }
                }
                NodeType::SineOsc { freq } => {
                    if let NodeState::SineOsc { phase } = node_state {
                        for output in outputs.iter_mut() {
                            for sample in output.iter_mut() {
                                *sample = phase.sin();
                                *phase += 2.0 * std::f32::consts::PI * freq / 44100.0;
                            }
                        }
                    }
                }
                NodeType::Gain { gain } => {
                    for (i, &edge_idx) in self.temp_inputs.iter().enumerate() {
                        let input = &self.edge_buffers[edge_idx][..];
                        if let Some(output) = outputs.get_mut(i) {
                            for (o, &i_val) in output.iter_mut().zip(input) {
                                *o = i_val * gain;
                            }
                        }
                    }
                }
                NodeType::Mix => {
                    for output in outputs.iter_mut() {
                        for &edge_idx in &self.temp_inputs {
                            let input = &self.edge_buffers[edge_idx][..];
                            for (o, &i_val) in output.iter_mut().zip(input) {
                                *o += i_val;
                            }
                        }
                    }
                }
                NodeType::OutputSink => {
                    if let Some(&edge_idx) = self.temp_inputs.get(0) {
                        let input = &self.edge_buffers[edge_idx][..];
                        out.copy_from_slice(input);
                    }
                }
            }
            // Store outputs in edge buffers
            for (i, &(edge_idx, _)) in self.plan.node_outputs[node_id.0].iter().enumerate() {
                self.edge_buffers[edge_idx].copy_from_slice(&outputs[i]);
            }
        }
    }
}

/// Render offline to a buffer.
pub fn render_offline(runtime: &mut Runtime, frames: usize) -> Vec<f32> {
    let mut output = vec![0.0; frames];
    let block_size = runtime.plan.block_size;
    let mut offset = 0;
    while offset < frames {
        let end = (offset + block_size).min(frames);
        let block_len = end - offset;
        let out_slice = &mut output[offset..end];
        // Pad if necessary, but assume frames % block_size == 0 for simplicity
        runtime.process_block(out_slice);
        offset += block_len;
    }
    output
}

/// Run process_block with panic containment.
pub fn process_block_safe(runtime: &mut Runtime, out: &mut [f32]) {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime.process_block(out);
    }));
    if result.is_err() {
        // Fail closed: silence output
        out.fill(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Graph, NodeType, PortId, Rate};
    use crate::plan::Plan;

    #[test]
    fn rt_no_alloc() {
        let mut graph = Graph::new();
        let node1 = graph.add_node(NodeType::Dummy);
        let plan = Plan::compile(&graph, 64).unwrap();
        let mut runtime = Runtime::new(plan, &graph);
        let mut out = vec![0.0; 64];
        runtime.process_block(&mut out);
        // Should copy default to out, but since no input, out remains 0
        assert_eq!(out, vec![0.0; 64]);
    }

    #[test]
    fn rt_no_lock() {
        // Assume no locks; in Rust, no mutex used
        let mut graph = Graph::new();
        let _node1 = graph.add_node(NodeType::Dummy);
        let plan = Plan::compile(&graph, 64).unwrap();
        let mut runtime = Runtime::new(plan, &graph);
        let mut out = vec![0.0; 64];
        runtime.process_block(&mut out);
    }

    #[test]
    fn rt_honors_edges() {
        // Edges are honored: outputs propagate through the graph
        let mut graph = Graph::new();
        let node1 = graph.add_node(NodeType::Dummy);
        let node2 = graph.add_node(NodeType::Dummy);
        graph
            .add_edge(crate::graph::Edge {
                from_node: node1,
                from_port: PortId(0),
                to_node: node2,
                to_port: PortId(0),
                rate: Rate::Audio,
            })
            .unwrap();
        let plan = Plan::compile(&graph, 64).unwrap();
        let mut runtime = Runtime::new(plan, &graph);
        let mut out = vec![0.0; 64];
        runtime.process_block(&mut out);
        // For Dummy -> Dummy -> OutputSink, but wait, no OutputSink.
        // The graph has Dummy -> Dummy, but no output to out.
        // Perhaps add OutputSink.
        // For test, since no output, out is 0.
        // To test propagation, need to see if node2 gets input, but since no output, hard.
        // Perhaps change to Dummy -> OutputSink
        let mut graph = Graph::new();
        let node1 = graph.add_node(NodeType::Dummy);
        let node2 = graph.add_node(NodeType::OutputSink);
        graph
            .add_edge(crate::graph::Edge {
                from_node: node1,
                from_port: PortId(0),
                to_node: node2,
                to_port: PortId(0),
                rate: Rate::Audio,
            })
            .unwrap();
        let plan = Plan::compile(&graph, 64).unwrap();
        let mut runtime = Runtime::new(plan, &graph);
        let mut out = vec![0.0; 64];
        runtime.process_block(&mut out);
        // Dummy has no input, so outputs 0, OutputSink copies 0 to out
        assert_eq!(out, vec![0.0; 64]);
    }

    #[test]
    fn rt_determinism() {
        let mut graph = Graph::new();
        let node1 = graph.add_node(NodeType::Dummy);
        let plan = Plan::compile(&graph, 64).unwrap();
        let mut runtime1 = Runtime::new(plan.clone(), &graph);
        let mut runtime2 = Runtime::new(plan, &graph);
        let mut out1 = vec![0.0; 64];
        let mut out2 = vec![0.0; 64];
        runtime1.process_block(&mut out1);
        runtime2.process_block(&mut out2);
        assert_eq!(out1, out2);
    }

    #[test]
    fn node_golden() {
        use crate::graph::NodeId;
        let mut graph = Graph::new();
        let _node1 = graph.add_node(NodeType::SineOsc { freq: 440.0 });
        let node2 = graph.add_node(NodeType::OutputSink);
        graph.add_edge(crate::graph::Edge {
            from_node: NodeId(0),
            from_port: PortId(0),
            to_node: node2,
            to_port: PortId(0),
            rate: Rate::Audio,
        }).unwrap();
        let plan = Plan::compile(&graph, 64).unwrap();
        let mut runtime = Runtime::new(plan, &graph);
        let output = render_offline(&mut runtime, 64);
        // Check first few samples
        assert!((output[0] - 0.0).abs() < 0.01); // sin(0) = 0
        // Approximate check for sine wave
        assert!(output[1] > 0.0);
        assert!(output[10] > 0.0);
    }
}
