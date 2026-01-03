//! PPT Invariant System: Runtime invariant enforcement with contract tracking.

#[cfg(feature = "ppt")]
use lazy_static::lazy_static;
#[cfg(feature = "ppt")]
use std::collections::HashSet;
#[cfg(feature = "ppt")]
use std::sync::Mutex;

// Invariant constants for contract tracking (canonical numeric IDs from Auxide Final Spell)
pub const STATE_PIN_COMPLETE: u32 = 1;
pub const INGRESS_VALIDATION: u32 = 2;
pub const GRAPH_LEGALITY: u32 = 3;
pub const GRAPH_REJECTS_INVALID: u32 = 4;
pub const PLAN_SOUNDNESS: u32 = 5;
pub const BUFFER_LIVENESS: u32 = 6;
pub const NODE_SMOKE: u32 = 7;
pub const STATEFUL_NODE_CORRECT: u32 = 8;
pub const EXEC_CORRECTNESS: u32 = 9;
pub const EXEC_MULTI_PORT: u32 = 10;
pub const EXEC_DETERMINISM: u32 = 11;
pub const RT_NO_ALLOC: u32 = 12;
pub const RT_ALLOC_SELFTEST: u32 = 13;
pub const RT_NO_LOCK: u32 = 14;
pub const PPT_RT_SAFE: u32 = 15;
pub const PPT_CONTRACT_COMPLETE: u32 = 16;
pub const PROP_VALID_EXEC: u32 = 17;
pub const PROP_INVALID_REJECT: u32 = 18;
pub const PROP_NO_PANIC: u32 = 19;
pub const EGRESS_INTEGRITY: u32 = 20;
pub const BENCH_VALID: u32 = 21;
pub const RELEASE_SEAL: u32 = 22;

#[cfg(feature = "ppt")]
lazy_static! {
    static ref INVARIANT_LOG: Mutex<HashSet<u32>> = Mutex::new(HashSet::new());
}

#[cfg(feature = "ppt")]
/// Assert an invariant: logs it and panics on failure.
pub(crate) fn assert_invariant(id: u32, condition: bool, message: &str, context: Option<&str>) {
    if !condition {
        let full_message = if let Some(ctx) = context {
            format!("Invariant {} failed: {} (context: {})", id, message, ctx)
        } else {
            format!("Invariant {} failed: {}", id, message)
        };
        eprintln!("{}", full_message);
        panic!("{}", full_message);
    }
    // Log the invariant presence
    INVARIANT_LOG.lock().unwrap().insert(id);
}

#[cfg(not(feature = "ppt"))]
/// Assert an invariant: checks condition and panics on failure.
pub fn assert_invariant(_id: u32, condition: bool, message: &str, _context: Option<&str>) {
    if !condition {
        panic!("Invariant failed: {}", message);
    }
}

#[cfg(feature = "ppt")]
/// Contract test: checks that specified invariants were asserted.
pub fn contract_test(test_name: &str, required_invariants: &[u32]) {
    let log = INVARIANT_LOG.lock().unwrap();
    let mut missing = Vec::new();
    for &inv in required_invariants {
        if !log.contains(&inv) {
            missing.push(inv);
        }
    }
    drop(log); // Drop the lock before panicking
    if !missing.is_empty() {
        panic!(
            "Contract test '{}' failed: invariants not enforced: {:?}",
            test_name, missing
        );
    }
}

#[cfg(not(feature = "ppt"))]
/// Contract test: no-op when PPT feature is disabled.
pub fn contract_test(_test_name: &str, _required_invariants: &[u32]) {}

#[cfg(feature = "ppt")]
/// Clear invariant log (for between test runs).
pub fn clear_invariant_log() {
    INVARIANT_LOG.lock().unwrap().clear();
}

#[cfg(not(feature = "ppt"))]
/// Clear invariant log: no-op when PPT feature is disabled.
pub fn clear_invariant_log() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_invariant_pass() {
        clear_invariant_log();
        assert_invariant(0, 1 + 1 == 2, "Math works", Some("basic"));
        // Should not panic
    }

    #[test]
    #[should_panic]
    fn test_assert_invariant_fail() {
        assert_invariant(0, 1 + 1 == 3, "Math broken", None);
    }

    #[test]
    fn test_contract_test() {
        clear_invariant_log();
        #[cfg(feature = "ppt")]
        {
            INVARIANT_LOG
                .lock()
                .unwrap()
                .insert(PLAN_SOUNDNESS);
            contract_test("example", &[PLAN_SOUNDNESS]);
        }

        #[cfg(not(feature = "ppt"))]
        {
            // When PPT is disabled, contract tests are a no-op.
            contract_test("example", &[PLAN_SOUNDNESS]);
        }
    }
}
