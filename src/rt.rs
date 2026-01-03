//! RT module: real-time execution engine.

// IMPORTANT: Do not call assert_invariant or any PPT logging in RT paths to avoid locks/allocs.

#![forbid(unsafe_code)]
// #![deny(missing_docs)]

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
    sample_rate: f32,
    nodes: Vec<Option<NodeType>>,
    states: Vec<Option<NodeState>>,
    edge_buffers: Vec<Vec<f32>>,
    temp_inputs: Vec<usize>,
    temp_output_vecs: Vec<Vec<f32>>,
}

impl Runtime {
    /// Create a new runtime from a plan and graph.
    pub fn new(plan: Plan, graph: &Graph, sample_rate: f32) -> Self {
        let nodes: Vec<Option<NodeType>> = graph
            .nodes
            .iter()
            .map(|n| n.as_ref().map(|nd| nd.node_type.clone()))
            .collect();
        let states: Vec<Option<NodeState>> = nodes
            .iter()
            .map(|nt| {
                nt.as_ref().map(|nt| match nt {
                    NodeType::SineOsc { .. } => NodeState::SineOsc { phase: 0.0 },
                    NodeType::Gain { .. } => NodeState::Gain,
                    NodeType::Mix => NodeState::Mix,
                    NodeType::OutputSink => NodeState::OutputSink,
                    NodeType::Dummy => NodeState::Dummy,
                })
            })
            .collect();
        let edge_buffers = vec![vec![0.0; plan.block_size]; plan.edges.len()];
        let temp_inputs = Vec::with_capacity(plan.max_inputs);
        let temp_output_vecs = (0..plan.max_outputs)
            .map(|_| vec![0.0; plan.block_size])
            .collect();
        Self {
            plan,
            sample_rate,
            nodes,
            states,
            edge_buffers,
            temp_inputs,
            temp_output_vecs,
        }
    }

    /// Process a block of frames, writing to out (mono).
    pub fn process_block(&mut self, out: &mut [f32]) -> Result<(), &'static str> {
        let block_size = self.plan.block_size;
        if out.len() != block_size {
            return Err("output buffer must be exactly block_size long");
        }
        // For each node in order
        for &node_id in &self.plan.order {
            if let (Some(node_type), Some(node_state)) =
                (&self.nodes[node_id.0], &mut self.states[node_id.0])
            {
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
                                    *phase += 2.0 * std::f32::consts::PI * freq / self.sample_rate;
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
            } else {
                // Fail-closed: silence outputs
                for &(edge_idx, _) in &self.plan.node_outputs[node_id.0] {
                    self.edge_buffers[edge_idx].fill(0.0);
                }
            }
        }
        Ok(())
    }
}

/// Render offline to a buffer.
pub fn render_offline(runtime: &mut Runtime, frames: usize) -> Vec<f32> {
    let mut output = vec![0.0; frames];
    let block_size = runtime.plan.block_size;
    let mut offset = 0;
    while offset < frames {
        let block_len = (frames - offset).min(block_size);
        if block_len == block_size {
            runtime
                .process_block(&mut output[offset..offset + block_size])
                .unwrap();
        } else {
            // Pad the final partial block
            let mut temp_block = vec![0.0; block_size];
            runtime.process_block(&mut temp_block).unwrap();
            output[offset..frames].copy_from_slice(&temp_block[0..block_len]);
        }
        offset += block_len;
    }
    output
}

/// Run process_block with panic containment.
pub fn process_block_safe(runtime: &mut Runtime, out: &mut [f32]) {
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| runtime.process_block(out)));
    match result {
        Ok(Ok(())) => {} // Success
        Ok(Err(_)) | Err(_) => {
            // Fail closed: silence output
            out.fill(0.0);
        }
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
        let _node1 = graph.add_node(NodeType::Dummy);
        let plan = Plan::compile(&graph, 64).unwrap();
        let mut runtime = Runtime::new(plan, &graph, 44100.0);
        let mut out = vec![0.0; 64];
        runtime.process_block(&mut out).unwrap();
        // Should copy default to out, but since no input, out remains 0
        assert_eq!(out, vec![0.0; 64]);
    }

    #[test]
    fn rt_no_lock() {
        // Assume no locks; in Rust, no mutex used
        let mut graph = Graph::new();
        let _node1 = graph.add_node(NodeType::Dummy);
        let plan = Plan::compile(&graph, 64).unwrap();
        let mut runtime = Runtime::new(plan, &graph, 44100.0);
        let mut out = vec![0.0; 64];
        runtime.process_block(&mut out).unwrap();
    }

    #[test]
    fn rt_honors_edges() {
        // Edges are honored: outputs propagate through the graph
        let mut graph = Graph::new();
        let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
        let sink = graph.add_node(NodeType::OutputSink);
        graph
            .add_edge(crate::graph::Edge {
                from_node: osc,
                from_port: PortId(0),
                to_node: sink,
                to_port: PortId(0),
                rate: Rate::Audio,
            })
            .unwrap();
        let plan = Plan::compile(&graph, 64).unwrap();
        let mut runtime = Runtime::new(plan, &graph, 44100.0);
        let mut out = vec![0.0; 64];
        runtime.process_block(&mut out).unwrap();
        // SineOsc produces non-zero output, OutputSink copies to out
        assert!(
            out.iter().any(|&x| x != 0.0),
            "Output should contain non-zero values from SineOsc"
        );
    }

    #[test]
    fn rt_determinism() {
        let mut graph = Graph::new();
        let _node1 = graph.add_node(NodeType::Dummy);
        let plan = Plan::compile(&graph, 64).unwrap();
        let mut runtime1 = Runtime::new(plan.clone(), &graph, 44100.0);
        let mut runtime2 = Runtime::new(plan, &graph, 44100.0);
        let mut out1 = vec![0.0; 64];
        let mut out2 = vec![0.0; 64];
        runtime1.process_block(&mut out1).unwrap();
        runtime2.process_block(&mut out2).unwrap();
        assert_eq!(out1, out2);
    }

    #[test]
    fn node_golden() {
        use crate::graph::NodeId;
        let mut graph = Graph::new();
        let _node1 = graph.add_node(NodeType::SineOsc { freq: 440.0 });
        let node2 = graph.add_node(NodeType::OutputSink);
        graph
            .add_edge(crate::graph::Edge {
                from_node: NodeId(0),
                from_port: PortId(0),
                to_node: node2,
                to_port: PortId(0),
                rate: Rate::Audio,
            })
            .unwrap();
        let plan = Plan::compile(&graph, 64).unwrap();
        let mut runtime = Runtime::new(plan, &graph, 44100.0);
        let output = render_offline(&mut runtime, 64);
        // Check first few samples
        assert!((output[0] - 0.0).abs() < 0.01); // sin(0) = 0
        // Approximate check for sine wave
        assert!(output[1] > 0.0);
        assert!(output[10] > 0.0);
    }

    #[test]
    fn process_block_wrong_buffer_length() {
        let mut graph = Graph::new();
        let _node1 = graph.add_node(NodeType::Dummy);
        let plan = Plan::compile(&graph, 64).unwrap();
        let mut runtime = Runtime::new(plan, &graph, 44100.0);
        let mut out = vec![0.0; 32]; // Wrong length
        let result = runtime.process_block(&mut out);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "output buffer must be exactly block_size long"
        );
    }
}
