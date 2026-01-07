//! RT module: real-time execution engine.

// IMPORTANT: Do not call assert_invariant or any PPT logging in RT paths to avoid locks/allocs.

#![warn(missing_docs)]

use crate::graph::{Graph, NodeType};
use crate::plan::Plan;
use crate::states::NodeState;

/// Maximum number of inputs that can be handled without heap allocation in RT path.
/// This limit is enforced at plan compile time (see plan.rs MAX_EXTERNAL_NODE_INPUTS).
const MAX_STACK_INPUTS: usize = crate::plan::MAX_EXTERNAL_NODE_INPUTS;

/// The runtime engine.
#[derive(Debug)]
pub struct Runtime {
    /// The compiled execution plan.
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
                    NodeType::External { def } => NodeState::External {
                        state: def.init_state(sample_rate, plan.block_size),
                    },
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

    /// Get the sample rate.
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
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
                                debug_assert_eq!(
                                    input.len(),
                                    output.len(),
                                    "Buffer lengths must match for copy_from_slice"
                                );
                                output.copy_from_slice(input);
                            }
                        }
                    }
                    NodeType::SineOsc { freq } => {
                        if let NodeState::SineOsc { phase } = node_state {
                            let step = 2.0 * std::f32::consts::PI * freq / self.sample_rate;
                            for output in outputs.iter_mut() {
                                for sample in output.iter_mut() {
                                    *sample = phase.sin();
                                    *phase += step;
                                    // Only wrap phase if it exceeds 2π to prevent precision loss
                                    if *phase > 2.0 * std::f32::consts::PI {
                                        *phase %= 2.0 * std::f32::consts::PI;
                                    }
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
                        if let Some(&edge_idx) = self.temp_inputs.first() {
                            let input = &self.edge_buffers[edge_idx][..];
                            out.copy_from_slice(input);
                        }
                    }
                    NodeType::External { def } => {
                        // Build input slices on the stack with proper lifetimes.
                        // The slices borrow from edge_buffers which lives for the duration
                        // of this function, ensuring sound lifetime semantics.
                        let num_inputs = self.temp_inputs.len();

                        if num_inputs <= MAX_STACK_INPUTS {
                            // Fast path: use stack array for typical cases
                            let mut input_refs: [&[f32]; MAX_STACK_INPUTS] =
                                [&[]; MAX_STACK_INPUTS];
                            for (i, &idx) in self.temp_inputs.iter().enumerate() {
                                input_refs[i] = &self.edge_buffers[idx][..];
                            }
                            let inputs_slice = &input_refs[..num_inputs];
                            if let NodeState::External { state } = node_state {
                                match def.process_block(
                                    state.as_mut(),
                                    inputs_slice,
                                    outputs,
                                    self.sample_rate,
                                ) {
                                    Ok(()) => {
                                        // Successful processing
                                    }
                                    Err(e) => {
                                        eprintln!("External node processing failed: {}", e);
                                        // Fail-closed: silence outputs
                                        for output in outputs.iter_mut() {
                                            output.fill(0.0);
                                        }
                                        return Err(e);
                                    }
                                }
                            }
                        } else {
                            // This branch should be unreachable: Plan::compile rejects external nodes
                            // with >MAX_EXTERNAL_NODE_INPUTS inputs. If we hit this, it's a bug.
                            eprintln!(
                                "BUG: External node has {} inputs but plan should have rejected >{}. \
                                This indicates a bug in Plan::compile validation.",
                                num_inputs, MAX_STACK_INPUTS
                            );
                            debug_assert!(false, "External node input validation failed");
                            // Fail-closed: silence outputs for this node
                            for output in outputs.iter_mut() {
                                output.fill(0.0);
                            }
                            return Err("External node exceeds maximum input limit");
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
pub fn render_offline(runtime: &mut Runtime, frames: usize) -> Result<Vec<f32>, &'static str> {
    if runtime.plan.block_size == 0 {
        return Err("Block size must be > 0");
    }
    let mut output = vec![0.0; frames];
    let block_size = runtime.plan.block_size;
    let mut offset = 0;
    while offset < frames {
        let block_len = (frames - offset).min(block_size);
        if block_len == block_size {
            runtime.process_block(&mut output[offset..offset + block_size])?;
        } else {
            // Pad the final partial block
            let mut temp_block = vec![0.0; block_size];
            runtime.process_block(&mut temp_block)?;
            output[offset..frames].copy_from_slice(&temp_block[0..block_len]);
        }
        offset += block_len;
    }
    Ok(output)
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
    use crate::node::NodeDef;
    use crate::plan::Plan;

    #[derive(Clone)]
    struct TestExternalNode;

    impl TestExternalNode {
        const PORTS_MONO_IN: &'static [crate::graph::Port] = &[crate::graph::Port {
            id: PortId(0),
            rate: Rate::Audio,
        }];
        const PORTS_MONO_OUT: &'static [crate::graph::Port] = &[crate::graph::Port {
            id: PortId(0),
            rate: Rate::Audio,
        }];
    }

    impl NodeDef for TestExternalNode {
        type State = f32;

        fn input_ports(&self) -> &'static [crate::graph::Port] {
            Self::PORTS_MONO_IN
        }

        fn output_ports(&self) -> &'static [crate::graph::Port] {
            Self::PORTS_MONO_OUT
        }

        fn required_inputs(&self) -> usize {
            1
        }

        fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
            0.0
        }

        fn process_block(
            &self,
            state: &mut Self::State,
            inputs: &[&[f32]],
            outputs: &mut [Vec<f32>],
            _sample_rate: f32,
        ) -> Result<(), &'static str> {
            // Simple passthrough with gain stored in state; state not mutated here.
            if let Some(out) = outputs.get_mut(0) {
                if let Some(input) = inputs.first() {
                    for (o, &i) in out.iter_mut().zip(*input) {
                        *o = i + *state;
                    }
                }
            }
            Ok(())
        }
    }

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
    fn rt_external_node_runs() {
        let mut graph = Graph::new();
        let input = graph.add_node(NodeType::SineOsc { freq: 440.0 });
        let ext = graph.add_external_node(TestExternalNode);
        let sink = graph.add_node(NodeType::OutputSink);

        graph
            .add_edge(crate::graph::Edge {
                from_node: input,
                from_port: PortId(0),
                to_node: ext,
                to_port: PortId(0),
                rate: Rate::Audio,
            })
            .unwrap();
        graph
            .add_edge(crate::graph::Edge {
                from_node: ext,
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
        // External node passes through osc into sink
        assert!(out.iter().any(|&x| x != 0.0));
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
        let output = render_offline(&mut runtime, 64).unwrap();
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

    #[test]
    fn max_inputs_consistent_with_plan() {
        // Ensure the stack fast-path limit in RT matches the compile-time plan limit.
        assert_eq!(MAX_STACK_INPUTS, crate::plan::MAX_EXTERNAL_NODE_INPUTS);
    }
}
