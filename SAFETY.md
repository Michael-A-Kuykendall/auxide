# Safety Document

## Real-Time Rules

- No heap allocations within `process_block` (requires preallocated output buffers).
- No mutexes, locks, or blocking operations in `process_block`.
- Panics are contained; fail-closed to silence.
- All buffers preallocated off-thread.

## Enforcement

- Test harness checks for allocations (planned; not yet implemented).
- Code review gates forbid alloc paths.
- Clippy lints for RT safety.

## Assumptions

- Host provides real-time thread guarantees.
- No external dependencies in RT path.

## What is Enforced

- Graph validation prevents invalid connections.
- Runtime checks prevent rate mismatches.
- Tests verify determinism and safety.