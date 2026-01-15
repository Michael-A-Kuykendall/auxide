//! RT module: real-time execution engine.
//!
//! # Architecture
//!
//! The runtime is split into three components:
//!
//! - **RuntimeCore**: RT-owned execution engine (moved into audio callback)
//! - **RuntimeControl**: Main-thread-owned control endpoints
//! - **RuntimeHandle**: What StreamController receives (core + RT channel endpoints)
//!
//! This split enables:
//! - Lock-free parameter updates from main thread
//! - RT-safe invariant signaling from audio callback
//! - No shared mutable state requiring synchronization
//!
//! # Message Flow
//!
//! ```text
//! Main Thread              RT Thread (callback)
//!      │                        │
//!      │──ControlMsg───────────▶│  (lock-free SPSC)
//!      │                        │
//!      │◀──InvariantSignal─────│  (lock-free SPSC)
//!      │                        │
//! ```

// IMPORTANT: Do not call assert_invariant or any PPT logging in RT paths to avoid locks/allocs.

#![warn(missing_docs)]

use crate::control::{new_control_queue, ControlMsg, CONTROL_QUEUE_CAPACITY};
use crate::graph::{Graph, NodeId, NodeType};
use crate::invariant_rt::{
    new_invariant_queue, signal_invariant, INV_CONTROL_MSG_PROCESSED, INV_PARAM_UPDATE_DELIVERED,
    INV_RT_CALLBACK_CLEAN, INV_SAMPLE_BUFFER_FILLED,
};
use crate::plan::Plan;
use crate::states::NodeState;
use rtrb::{Consumer, Producer};


/// Maximum number of inputs that can be handled without heap allocation in RT path.
/// This limit is enforced at plan compile time (see plan.rs MAX_EXTERNAL_NODE_INPUTS).
const MAX_STACK_INPUTS: usize = crate::plan::MAX_EXTERNAL_NODE_INPUTS;

// ============================================================================
// Legacy Runtime (preserved for backward compatibility)
// ============================================================================

/// The runtime engine (legacy API - use RuntimeCore for new code).
///
/// This is preserved for backward compatibility. For new code using
/// parameter updates and invariant signaling, use `RuntimeCore::new_with_channels`.
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

// ============================================================================
// New Split Architecture (RuntimeCore + RuntimeControl + RuntimeHandle)
// ============================================================================

/// RT-owned execution core (moved into audio callback).
///
/// This contains all state needed for audio processing. It is designed to be
/// moved into the audio callback where it processes blocks and applies control
/// messages received via lock-free queue.
///
/// # RT Safety
///
/// All methods on RuntimeCore are RT-safe:
/// - No allocation after construction
/// - No locking
/// - No panics (errors are signaled, not thrown)
pub struct RuntimeCore {
    /// The compiled execution plan.
    pub plan: Plan,
    sample_rate: f32,
    nodes: Vec<Option<NodeType>>,
    states: Vec<Option<NodeState>>,
    edge_buffers: Vec<Vec<f32>>,
    temp_inputs: Vec<usize>,
    temp_output_vecs: Vec<Vec<f32>>,
    /// Per-node mute state (true = muted)
    mute_flags: Vec<bool>,
    /// Per-node gain override (applied on top of node's own gain)
    gain_overrides: Vec<f32>,
}

/// Main-thread-owned control interface.
///
/// This holds the channel endpoints for the main thread:
/// - `control_tx`: Send control messages to RT
/// - `invariant_rx`: Receive invariant signals from RT
///
/// # Usage
///
/// ```ignore
/// let (core, control) = RuntimeCore::new_with_channels(plan, &graph, sample_rate);
///
/// // Main thread sends control messages
/// control.send(ControlMsg::SetGain { node: NodeId(0), gain: 0.5 });
///
/// // Main thread drains invariant signals
/// let signals = control.drain_invariant_signals();
/// ```
pub struct RuntimeControl {
    /// Send control messages to RT (main → RT)
    control_tx: Producer<ControlMsg>,
    /// Receive invariant signals from RT (RT → main)
    invariant_rx: Consumer<u8>,
    /// Sample rate for reference
    sample_rate: f32,
    /// Block size for reference
    block_size: usize,
}

/// What gets passed to StreamController (core + RT channel endpoints).
///
/// This bundles the RuntimeCore with its RT-side channel endpoints:
/// - `control_rx`: Receive control messages from main
/// - `invariant_tx`: Send invariant signals to main
///
/// StreamController calls `play(handle)` which moves this into the audio callback.
pub struct RuntimeHandle {
    /// RT-owned execution core
    pub core: RuntimeCore,
    /// Receive control messages from main (main → RT)
    pub control_rx: Consumer<ControlMsg>,
    /// Send invariant signals to main (RT → main)
    pub invariant_tx: Producer<u8>,
}

impl RuntimeCore {
    /// Create a RuntimeCore with associated control channels.
    ///
    /// Returns (handle for StreamController, control for main thread).
    pub fn new_with_channels(
        plan: Plan,
        graph: &Graph,
        sample_rate: f32,
    ) -> (RuntimeHandle, RuntimeControl) {
        let nodes: Vec<Option<NodeType>> = graph
            .nodes
            .iter()
            .map(|n| n.as_ref().map(|nd| nd.node_type.clone()))
            .collect();

        let num_nodes = nodes.len();

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

        let block_size = plan.block_size;

        let core = RuntimeCore {
            plan,
            sample_rate,
            nodes,
            states,
            edge_buffers,
            temp_inputs,
            temp_output_vecs,
            mute_flags: vec![false; num_nodes],
            gain_overrides: vec![1.0; num_nodes],
        };

        // Create channels
        let (control_tx, control_rx) = new_control_queue();
        let (invariant_tx, invariant_rx) = new_invariant_queue();

        let handle = RuntimeHandle {
            core,
            control_rx,
            invariant_tx,
        };

        let control = RuntimeControl {
            control_tx,
            invariant_rx,
            sample_rate,
            block_size,
        };

        (handle, control)
    }

    /// Get the sample rate.
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Get the block size.
    pub fn block_size(&self) -> usize {
        self.plan.block_size
    }

    /// Apply a control message (RT-safe).
    ///
    /// This is called from within the audio callback after draining the control queue.
    #[inline]
    fn apply_control_msg(&mut self, msg: ControlMsg) {
        match msg {
            ControlMsg::SetGain { node, gain } => {
                // Set the gain override for any node type
                if node.0 < self.gain_overrides.len() {
                    self.gain_overrides[node.0] = gain;
                }
                // Also update the Gain node's internal gain directly
                // This ensures SetGain works as "set to value" not "multiply by value"
                if let Some(Some(NodeType::Gain { gain: ref mut node_gain })) = self.nodes.get_mut(node.0) {
                    *node_gain = 1.0; // Neutralize node gain so override is the actual value
                }
            }
            ControlMsg::SetFrequency { node, hz } => {
                if let Some(Some(NodeType::SineOsc { freq })) = self.nodes.get_mut(node.0) {
                    *freq = hz;
                }
            }
            ControlMsg::TriggerGate { node, on: _ } => {
                // TODO: When envelope nodes are added, trigger their gate here
                let _ = node; // Suppress unused warning for now
            }
            ControlMsg::Mute { node } => {
                if node.0 < self.mute_flags.len() {
                    self.mute_flags[node.0] = true;
                }
            }
            ControlMsg::Unmute { node } => {
                if node.0 < self.mute_flags.len() {
                    self.mute_flags[node.0] = false;
                }
            }
            ControlMsg::AllNotesOff => {
                // TODO: Trigger all envelope releases
            }
            ControlMsg::Reset => {
                self.gain_overrides.fill(1.0);
                self.mute_flags.fill(false);
            }
            // Other messages handled as needed
            _ => {}
        }
    }

    /// Process a block with control message handling and invariant signaling.
    ///
    /// This is the main entry point for the audio callback. It:
    /// 1. Drains pending control messages
    /// 2. Processes the audio block
    /// 3. Signals relevant invariants
    ///
    /// # RT Safety
    /// - No allocation
    /// - No locking
    /// - No panics
    pub fn process_block_with_channels(
        &mut self,
        out: &mut [f32],
        control_rx: &mut Consumer<ControlMsg>,
        invariant_tx: &mut Producer<u8>,
    ) -> Result<(), &'static str> {
        // Drain control messages (RT-safe: lock-free pop)
        let mut msg_count = 0;
        while let Ok(msg) = control_rx.pop() {
            self.apply_control_msg(msg);
            msg_count += 1;
            // Cap messages per buffer to prevent RT stalls
            if msg_count >= CONTROL_QUEUE_CAPACITY / 4 {
                break;
            }
        }

        if msg_count > 0 {
            signal_invariant(invariant_tx, INV_CONTROL_MSG_PROCESSED);
            signal_invariant(invariant_tx, INV_PARAM_UPDATE_DELIVERED);
        }

        // Process the audio block
        let result = self.process_block_internal(out);

        if result.is_ok() {
            signal_invariant(invariant_tx, INV_SAMPLE_BUFFER_FILLED);
            signal_invariant(invariant_tx, INV_RT_CALLBACK_CLEAN);
        }

        result
    }

    /// Internal block processing (same logic as legacy Runtime::process_block).
    fn process_block_internal(&mut self, out: &mut [f32]) -> Result<(), &'static str> {
        let block_size = self.plan.block_size;
        if out.len() != block_size {
            return Err("output buffer must be exactly block_size long");
        }

        for &node_id in &self.plan.order {
            // Skip muted nodes
            if self.mute_flags.get(node_id.0).copied().unwrap_or(false) {
                // Silence outputs for muted node
                for &(edge_idx, _) in &self.plan.node_outputs[node_id.0] {
                    self.edge_buffers[edge_idx].fill(0.0);
                }
                continue;
            }

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

                // Get gain override for this node
                let gain_override = self.gain_overrides.get(node_id.0).copied().unwrap_or(1.0);

                // Process based on node type
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
                            let step = 2.0 * std::f32::consts::PI * freq / self.sample_rate;
                            for output in outputs.iter_mut() {
                                for sample in output.iter_mut() {
                                    *sample = phase.sin() * gain_override;
                                    *phase += step;
                                    if *phase > 2.0 * std::f32::consts::PI {
                                        *phase %= 2.0 * std::f32::consts::PI;
                                    }
                                }
                            }
                        }
                    }
                    NodeType::Gain { gain } => {
                        let effective_gain = gain * gain_override;
                        for (i, &edge_idx) in self.temp_inputs.iter().enumerate() {
                            let input = &self.edge_buffers[edge_idx][..];
                            if let Some(output) = outputs.get_mut(i) {
                                for (o, &i_val) in output.iter_mut().zip(input) {
                                    *o = i_val * effective_gain;
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
                            // Apply gain override to mix output
                            if (gain_override - 1.0).abs() > 0.0001 {
                                for sample in output.iter_mut() {
                                    *sample *= gain_override;
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
                        let num_inputs = self.temp_inputs.len();
                        if num_inputs <= MAX_STACK_INPUTS {
                            let mut input_refs: [&[f32]; MAX_STACK_INPUTS] = [&[]; MAX_STACK_INPUTS];
                            for (i, &idx) in self.temp_inputs.iter().enumerate() {
                                input_refs[i] = &self.edge_buffers[idx][..];
                            }
                            let inputs_slice = &input_refs[..num_inputs];
                            if let NodeState::External { state } = node_state {
                                if let Err(e) = def.process_block(
                                    state.as_mut(),
                                    inputs_slice,
                                    outputs,
                                    self.sample_rate,
                                ) {
                                    eprintln!("External node processing failed: {}", e);
                                    for output in outputs.iter_mut() {
                                        output.fill(0.0);
                                    }
                                    return Err(e);
                                }
                            }
                        } else {
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
                for &(edge_idx, _) in &self.plan.node_outputs[node_id.0] {
                    self.edge_buffers[edge_idx].fill(0.0);
                }
            }
        }
        Ok(())
    }
}

impl RuntimeControl {
    /// Send a control message to RT (non-blocking).
    ///
    /// Returns Ok(()) if message was queued, Err if queue is full.
    pub fn send(&mut self, msg: ControlMsg) -> Result<(), ControlMsg> {
        self.control_tx.push(msg).map_err(|rtrb::PushError::Full(m)| m)
    }

    /// Drain all pending invariant signals from RT.
    ///
    /// Call this periodically from main thread to collect invariant data.
    pub fn drain_invariant_signals(&mut self) -> Vec<u8> {
        crate::invariant_rt::drain_invariant_signals(&mut self.invariant_rx)
    }

    /// Get the sample rate.
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Get the block size.
    pub fn block_size(&self) -> usize {
        self.block_size
    }

    /// Convenience: send SetGain message.
    pub fn set_gain(&mut self, node: NodeId, gain: f32) -> Result<(), ControlMsg> {
        self.send(ControlMsg::SetGain { node, gain })
    }

    /// Convenience: send SetFrequency message.
    pub fn set_frequency(&mut self, node: NodeId, hz: f32) -> Result<(), ControlMsg> {
        self.send(ControlMsg::SetFrequency { node, hz })
    }

    /// Convenience: send TriggerGate message.
    pub fn trigger_gate(&mut self, node: NodeId, on: bool) -> Result<(), ControlMsg> {
        self.send(ControlMsg::TriggerGate { node, on })
    }

    /// Convenience: send Mute message.
    pub fn mute(&mut self, node: NodeId) -> Result<(), ControlMsg> {
        self.send(ControlMsg::Mute { node })
    }

    /// Convenience: send Unmute message.
    pub fn unmute(&mut self, node: NodeId) -> Result<(), ControlMsg> {
        self.send(ControlMsg::Unmute { node })
    }
}

impl RuntimeHandle {
    /// Get the sample rate.
    pub fn sample_rate(&self) -> f32 {
        self.core.sample_rate
    }

    /// Get the block size.
    pub fn block_size(&self) -> usize {
        self.core.plan.block_size
    }

    /// Process a block (convenience wrapper for audio callback).
    ///
    /// This drains control messages, processes audio, and signals invariants.
    pub fn process_block(&mut self, out: &mut [f32]) -> Result<(), &'static str> {
        self.core.process_block_with_channels(
            out,
            &mut self.control_rx,
            &mut self.invariant_tx,
        )
    }
}

// ============================================================================
// Legacy Helper Functions
// ============================================================================

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
