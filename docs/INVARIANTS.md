# Invariants

Auxide enforces several invariants to ensure correctness, determinism, and RT safety.

## Graph Invariants
- **No cycles**: The graph must be acyclic, except for cycles involving delay nodes (which are handled specially).
- **Rate compatibility**: Connected ports must have compatible rates (Audio or Control).
- **Port existence**: Edges must reference valid ports on existing nodes.
- **Node stability**: Node IDs are stable and monotonic; removal sets nodes to None but preserves IDs.

## Plan Invariants
- **Required inputs**: All non-optional inputs must be connected.
- **Single-writer**: Each input port may have at most one incoming edge.
- **Topological order**: Execution order respects dependencies.

## Runtime Invariants
- **Determinism**: Same graph/plan/inputs produce same outputs (modulo floating-point).
- **No allocs/locks in RT paths**: `process_block` and related methods are RT-safe.
- **Buffer bounds**: All buffer accesses are bounds-checked at compile time or runtime.

## RT Invariant Signaling (Lock-Free)
- **Channel-based**: RT code signals invariant IDs over a lock-free SPSC queue; main thread drains and verifies.
- **IDs**: `INV_PARAM_UPDATE_DELIVERED`, `INV_SAMPLE_BUFFER_FILLED`, `INV_CONTROL_MSG_PROCESSED`, `INV_RT_CALLBACK_CLEAN`, plus room for future IDs.
- **Non-blocking**: Signals are dropped if queues are full—never block the audio thread.
- **Contracts**: Tests assert required signals via `contract_test_rt` to guarantee RT behavior without locking.

## Violations
Violations are caught at plan compilation and result in explicit errors (e.g., `PlanError::MultipleWritersToInput`).