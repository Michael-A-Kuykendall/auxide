//! PPT Invariant System: Runtime invariant enforcement with contract tracking.

#[cfg(feature = "ppt")]
use lazy_static::lazy_static;
#[cfg(feature = "ppt")]
use std::collections::HashSet;
#[cfg(feature = "ppt")]
use std::sync::Mutex;

// Invariant constants for contract tracking
pub const GRAPH_NO_CYCLES: &str = "Graph has no cycles";
pub const GRAPH_RATE_MATCH: &str = "Graph ports have matching rates";
pub const GRAPH_STABLE_ORDERING: &str = "Graph has stable node ordering";
pub const PLAN_DETERMINISTIC: &str = "Plan compilation is deterministic";
pub const PLAN_BUFFER_SOUNDNESS: &str = "Plan buffers are soundly allocated";
pub const RT_NO_ALLOC: &str = "RT thread performs no allocations";
pub const RT_NO_LOCKS: &str = "RT thread acquires no locks";
pub const RT_CONTAINMENT: &str = "RT execution is contained";
pub const RT_DETERMINISM: &str = "RT execution is deterministic";
pub const DSL_CLEAR_ERRORS: &str = "DSL provides clear error messages";

#[cfg(feature = "ppt")]
lazy_static! {
    static ref INVARIANT_LOG: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

#[cfg(feature = "ppt")]
/// Assert an invariant: logs it and panics on failure.
pub fn assert_invariant(condition: bool, message: &str, context: Option<&str>) {
    if !condition {
        let full_message = if let Some(ctx) = context {
            format!("Invariant failed: {} (context: {})", message, ctx)
        } else {
            format!("Invariant failed: {}", message)
        };
        eprintln!("{}", full_message);
        panic!("{}", full_message);
    }
    // Log the invariant presence
    let key = format!("{}:{}", message, context.unwrap_or(""));
    INVARIANT_LOG.lock().unwrap().insert(key);
}

#[cfg(not(feature = "ppt"))]
/// Assert an invariant: checks condition and panics on failure.
pub fn assert_invariant(condition: bool, message: &str, _context: Option<&str>) {
    if !condition {
        panic!("Invariant failed: {}", message);
    }
}

#[cfg(feature = "ppt")]
/// Contract test: checks that specified invariants were asserted.
pub fn contract_test(test_name: &str, required_invariants: &[&str]) {
    let log = INVARIANT_LOG.lock().unwrap();
    let mut missing = Vec::new();
    for inv in required_invariants {
        let enforced = log.iter().any(|key| key.starts_with(&format!("{}:", inv)));
        if !enforced {
            missing.push(*inv);
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
pub fn contract_test(_test_name: &str, _required_invariants: &[&str]) {}

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
        assert_invariant(1 + 1 == 2, "Math works", Some("basic"));
        // Should not panic
    }

    #[test]
    #[should_panic]
    fn test_assert_invariant_fail() {
        assert_invariant(1 + 1 == 3, "Math broken", None);
    }

    #[test]
    fn test_contract_test() {
        clear_invariant_log();
        #[cfg(feature = "ppt")]
        {
            INVARIANT_LOG
                .lock()
                .unwrap()
                .insert("Test invariant:".to_string());
            contract_test("example", &["Test invariant"]);
        }

        #[cfg(not(feature = "ppt"))]
        {
            // When PPT is disabled, contract tests are a no-op.
            contract_test("example", &["Test invariant"]);
        }
    }
}
