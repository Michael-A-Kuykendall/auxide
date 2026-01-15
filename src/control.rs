//! Control message types for main → RT communication.
//!
//! These messages are sent via lock-free SPSC queue from the main thread
//! to the RT audio callback. They enable parameter updates, gate triggers,
//! and other control operations without blocking.
//!
//! # Design Philosophy
//!
//! All messages are:
//! - Fixed-size (no heap allocation)
//! - Copy (can be sent across threads)
//! - Self-contained (no references or pointers)
//!
//! The RT callback drains the control queue each buffer and applies updates.

use crate::graph::NodeId;
use rtrb::{Consumer, Producer, RingBuffer};

/// Capacity for control message queue.
/// Should handle bursts of MIDI events (e.g., chord presses).
pub const CONTROL_QUEUE_CAPACITY: usize = 256;

/// Creates a new control message queue pair.
///
/// Returns (producer for main thread, consumer for RT).
pub fn new_control_queue() -> (Producer<ControlMsg>, Consumer<ControlMsg>) {
    RingBuffer::new(CONTROL_QUEUE_CAPACITY)
}

/// Control messages sent from main thread to RT callback.
#[derive(Debug, Clone, Copy)]
pub enum ControlMsg {
    /// Set a node's gain parameter.
    SetGain {
        node: NodeId,
        /// Gain value (0.0 = silent, 1.0 = unity)
        gain: f32,
    },

    /// Set a node's frequency parameter.
    SetFrequency {
        node: NodeId,
        /// Frequency in Hz
        hz: f32,
    },

    /// Trigger a gate (for envelopes).
    TriggerGate {
        node: NodeId,
        /// true = note on, false = note off
        on: bool,
    },

    /// Set a generic parameter by index.
    SetParam {
        node: NodeId,
        /// Parameter index (node-specific)
        param_idx: u8,
        /// Parameter value
        value: f32,
    },

    /// Set filter cutoff frequency.
    SetFilterCutoff {
        node: NodeId,
        /// Cutoff frequency in Hz
        hz: f32,
    },

    /// Set filter resonance (Q).
    SetFilterResonance {
        node: NodeId,
        /// Resonance (0.0 to 1.0 typical, higher for self-oscillation)
        q: f32,
    },

    /// Set oscillator waveform (if node supports it).
    SetWaveform {
        node: NodeId,
        /// Waveform index (node-specific mapping)
        waveform: u8,
    },

    /// Set detune in cents.
    SetDetune {
        node: NodeId,
        /// Detune in cents (-100 to +100 typical)
        cents: f32,
    },

    /// Set pan position.
    SetPan {
        node: NodeId,
        /// Pan position (-1.0 = left, 0.0 = center, 1.0 = right)
        pan: f32,
    },

    /// Immediately silence a node (emergency mute).
    Mute {
        node: NodeId,
    },

    /// Remove mute from a node.
    Unmute {
        node: NodeId,
    },

    /// All notes off (for all nodes that support it).
    AllNotesOff,

    /// Reset all parameters to defaults.
    Reset,
}

impl ControlMsg {
    /// Returns the target node ID, if this message targets a specific node.
    pub fn target_node(&self) -> Option<NodeId> {
        match self {
            ControlMsg::SetGain { node, .. } => Some(*node),
            ControlMsg::SetFrequency { node, .. } => Some(*node),
            ControlMsg::TriggerGate { node, .. } => Some(*node),
            ControlMsg::SetParam { node, .. } => Some(*node),
            ControlMsg::SetFilterCutoff { node, .. } => Some(*node),
            ControlMsg::SetFilterResonance { node, .. } => Some(*node),
            ControlMsg::SetWaveform { node, .. } => Some(*node),
            ControlMsg::SetDetune { node, .. } => Some(*node),
            ControlMsg::SetPan { node, .. } => Some(*node),
            ControlMsg::Mute { node } => Some(*node),
            ControlMsg::Unmute { node } => Some(*node),
            ControlMsg::AllNotesOff => None,
            ControlMsg::Reset => None,
        }
    }

    /// Returns a human-readable description (for debugging).
    pub fn description(&self) -> &'static str {
        match self {
            ControlMsg::SetGain { .. } => "SetGain",
            ControlMsg::SetFrequency { .. } => "SetFrequency",
            ControlMsg::TriggerGate { .. } => "TriggerGate",
            ControlMsg::SetParam { .. } => "SetParam",
            ControlMsg::SetFilterCutoff { .. } => "SetFilterCutoff",
            ControlMsg::SetFilterResonance { .. } => "SetFilterResonance",
            ControlMsg::SetWaveform { .. } => "SetWaveform",
            ControlMsg::SetDetune { .. } => "SetDetune",
            ControlMsg::SetPan { .. } => "SetPan",
            ControlMsg::Mute { .. } => "Mute",
            ControlMsg::Unmute { .. } => "Unmute",
            ControlMsg::AllNotesOff => "AllNotesOff",
            ControlMsg::Reset => "Reset",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_msg_is_copy() {
        let msg = ControlMsg::SetGain {
            node: NodeId(0),
            gain: 0.5,
        };
        let msg2 = msg; // Copy
        assert!(matches!(msg2, ControlMsg::SetGain { .. }));
    }

    #[test]
    fn test_control_queue_roundtrip() {
        let (mut tx, mut rx) = new_control_queue();

        tx.push(ControlMsg::SetGain {
            node: NodeId(0),
            gain: 0.5,
        })
        .unwrap();
        tx.push(ControlMsg::TriggerGate {
            node: NodeId(1),
            on: true,
        })
        .unwrap();

        let msg1 = rx.pop().unwrap();
        let msg2 = rx.pop().unwrap();

        assert!(matches!(msg1, ControlMsg::SetGain { gain, .. } if (gain - 0.5).abs() < 0.001));
        assert!(matches!(msg2, ControlMsg::TriggerGate { on: true, .. }));
    }

    #[test]
    fn test_target_node() {
        let msg = ControlMsg::SetGain {
            node: NodeId(42),
            gain: 1.0,
        };
        assert_eq!(msg.target_node(), Some(NodeId(42)));

        let msg = ControlMsg::AllNotesOff;
        assert_eq!(msg.target_node(), None);
    }
}
