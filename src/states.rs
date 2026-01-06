//! Node state definitions for the RT engine.

// IMPORTANT: Do not call assert_invariant or any PPT logging in RT paths to avoid locks/allocs.

use std::any::Any;

/// Node states for mutable data.
#[derive(Debug)]
pub enum NodeState {
    /// Sine oscillator state with phase accumulator.
    SineOsc {
        /// Current phase in radians.
        phase: f32,
    },
    /// Gain node (stateless).
    Gain,
    /// Mix node (stateless).
    Mix,
    /// Output sink (stateless).
    OutputSink,
    /// Dummy passthrough (stateless).
    Dummy,
    /// External node with type-erased state.
    External {
        /// The node's runtime state.
        state: Box<dyn Any + Send>,
    },
}