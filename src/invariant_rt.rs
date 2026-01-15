//! RT-safe invariant signaling for real-time audio paths.
//!
//! This module provides a two-tier invariant system:
//! - **Tier 1 (RT-safe)**: Lock-free signaling of invariant IDs from audio callback
//! - **Tier 2 (Non-RT)**: Verification and contract testing on main thread
//!
//! # Design Philosophy
//!
//! RT code **signals facts**. Non-RT code **judges correctness**.
//!
//! Unlike traditional `assert_invariant` which uses locks, this system:
//! - Never allocates in the RT path
//! - Never locks in the RT path
//! - Never panics in the RT path
//! - Uses lock-free SPSC queues for cross-thread communication
//!
//! # Example
//!
//! ```ignore
//! // RT callback signals an invariant was checked
//! signal_invariant(&invariant_tx, INV_SAMPLE_BUFFER_FILLED);
//!
//! // Main thread verifies contracts
//! let signals = drain_invariant_signals(&mut invariant_rx);
//! assert!(signals.contains(&INV_SAMPLE_BUFFER_FILLED));
//! ```

use rtrb::{Consumer, Producer, RingBuffer};

// ============================================================================
// RT-Safe Invariant IDs (Tier 1)
// ============================================================================
// These are integer IDs, not strings. No allocation, no formatting.

/// Parameter update was received and applied in RT callback.
pub const INV_PARAM_UPDATE_DELIVERED: u8 = 1;

/// Sample buffer was completely filled (no underrun).
pub const INV_SAMPLE_BUFFER_FILLED: u8 = 2;

/// Voice allocation stayed within pool bounds.
pub const INV_VOICE_ALLOCATION_BOUND: u8 = 3;

/// Gate trigger was honored (envelope state changed).
pub const INV_GATE_TRIGGER_HONORED: u8 = 4;

/// Control message was processed without error.
pub const INV_CONTROL_MSG_PROCESSED: u8 = 5;

/// RT callback executed without panic.
pub const INV_RT_CALLBACK_CLEAN: u8 = 6;

// ============================================================================
// Invariant Signal Queue
// ============================================================================

/// Capacity for invariant signal queue.
/// Should be large enough to hold signals from multiple buffer callbacks
/// between main thread drains.
pub const INVARIANT_QUEUE_CAPACITY: usize = 256;

/// Creates a new invariant signal queue pair.
///
/// Returns (producer for RT, consumer for main thread).
pub fn new_invariant_queue() -> (Producer<u8>, Consumer<u8>) {
    RingBuffer::new(INVARIANT_QUEUE_CAPACITY)
}

/// Signals an invariant was checked in the RT path.
///
/// # RT Safety
/// - No allocation
/// - No locking
/// - No panics
/// - If queue is full, signal is dropped (preferable to blocking)
#[inline]
pub fn signal_invariant(tx: &mut Producer<u8>, id: u8) {
    // push() returns Err if full - we drop silently rather than block
    let _ = tx.push(id);
}

/// Signals an invariant with a count (for batched operations).
///
/// # RT Safety
/// Same guarantees as `signal_invariant`.
#[inline]
pub fn signal_invariant_n(tx: &mut Producer<u8>, id: u8, count: usize) {
    for _ in 0..count.min(16) {
        // Cap at 16 to prevent RT stalls
        let _ = tx.push(id);
    }
}

// ============================================================================
// Non-RT Verification (Tier 2)
// ============================================================================

/// Drains all pending invariant signals from the queue.
///
/// Call this from the main thread to collect signals for contract verification.
pub fn drain_invariant_signals(rx: &mut Consumer<u8>) -> Vec<u8> {
    let mut signals = Vec::with_capacity(INVARIANT_QUEUE_CAPACITY);
    while let Ok(id) = rx.pop() {
        signals.push(id);
    }
    signals
}

/// Counts occurrences of each invariant ID in a signal list.
pub fn count_invariant_signals(signals: &[u8]) -> [usize; 256] {
    let mut counts = [0usize; 256];
    for &id in signals {
        counts[id as usize] += 1;
    }
    counts
}

/// Contract verification: asserts that required invariants were signaled.
///
/// # Panics
/// Panics if any required invariant was not signaled at least once.
#[cfg(any(test, feature = "ppt"))]
pub fn contract_test_rt(contract_name: &str, signals: &[u8], required: &[u8]) {
    let counts = count_invariant_signals(signals);
    let mut missing = Vec::new();

    for &id in required {
        if counts[id as usize] == 0 {
            missing.push(invariant_name(id));
        }
    }

    if !missing.is_empty() {
        let present: Vec<&str> = signals
            .iter()
            .map(|&id| invariant_name(id))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        panic!(
            "RT Contract '{}' missing invariants: {:?}. Present: {:?}",
            contract_name, missing, present
        );
    }
}

/// Maps invariant ID to human-readable name (for diagnostics only).
pub const fn invariant_name(id: u8) -> &'static str {
    match id {
        INV_PARAM_UPDATE_DELIVERED => "PARAM_UPDATE_DELIVERED",
        INV_SAMPLE_BUFFER_FILLED => "SAMPLE_BUFFER_FILLED",
        INV_VOICE_ALLOCATION_BOUND => "VOICE_ALLOCATION_BOUND",
        INV_GATE_TRIGGER_HONORED => "GATE_TRIGGER_HONORED",
        INV_CONTROL_MSG_PROCESSED => "CONTROL_MSG_PROCESSED",
        INV_RT_CALLBACK_CLEAN => "RT_CALLBACK_CLEAN",
        _ => "UNKNOWN",
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invariant_queue_roundtrip() {
        let (mut tx, mut rx) = new_invariant_queue();

        signal_invariant(&mut tx, INV_SAMPLE_BUFFER_FILLED);
        signal_invariant(&mut tx, INV_PARAM_UPDATE_DELIVERED);
        signal_invariant(&mut tx, INV_SAMPLE_BUFFER_FILLED);

        let signals = drain_invariant_signals(&mut rx);
        assert_eq!(signals.len(), 3);
        assert_eq!(
            signals,
            vec![
                INV_SAMPLE_BUFFER_FILLED,
                INV_PARAM_UPDATE_DELIVERED,
                INV_SAMPLE_BUFFER_FILLED
            ]
        );
    }

    #[test]
    fn test_count_invariant_signals() {
        let signals = vec![
            INV_SAMPLE_BUFFER_FILLED,
            INV_SAMPLE_BUFFER_FILLED,
            INV_PARAM_UPDATE_DELIVERED,
        ];
        let counts = count_invariant_signals(&signals);
        assert_eq!(counts[INV_SAMPLE_BUFFER_FILLED as usize], 2);
        assert_eq!(counts[INV_PARAM_UPDATE_DELIVERED as usize], 1);
        assert_eq!(counts[INV_GATE_TRIGGER_HONORED as usize], 0);
    }

    #[test]
    fn test_contract_passes_when_invariants_present() {
        let signals = vec![INV_SAMPLE_BUFFER_FILLED, INV_PARAM_UPDATE_DELIVERED];
        // Should not panic
        contract_test_rt(
            "basic contract",
            &signals,
            &[INV_SAMPLE_BUFFER_FILLED, INV_PARAM_UPDATE_DELIVERED],
        );
    }

    #[test]
    #[should_panic(expected = "missing invariants")]
    fn test_contract_fails_when_invariants_missing() {
        let signals = vec![INV_SAMPLE_BUFFER_FILLED];
        contract_test_rt(
            "incomplete contract",
            &signals,
            &[INV_SAMPLE_BUFFER_FILLED, INV_PARAM_UPDATE_DELIVERED],
        );
    }

    #[test]
    fn test_queue_handles_overflow_gracefully() {
        let (mut tx, mut rx) = new_invariant_queue();

        // Fill beyond capacity
        for _ in 0..INVARIANT_QUEUE_CAPACITY + 100 {
            signal_invariant(&mut tx, INV_SAMPLE_BUFFER_FILLED);
        }

        let signals = drain_invariant_signals(&mut rx);
        // Should have exactly capacity, overflow dropped
        assert_eq!(signals.len(), INVARIANT_QUEUE_CAPACITY);
    }

    #[test]
    fn test_invariant_names() {
        assert_eq!(invariant_name(INV_PARAM_UPDATE_DELIVERED), "PARAM_UPDATE_DELIVERED");
        assert_eq!(invariant_name(INV_SAMPLE_BUFFER_FILLED), "SAMPLE_BUFFER_FILLED");
        assert_eq!(invariant_name(255), "UNKNOWN");
    }
}
