# Changelog

## [0.3.0] - 2026-01-05
- **Module reorganization**: Extracted `states.rs` from `rt.rs` for better code organization
- **Enhanced PPT API**: Added integration tests and RT no-PPT test coverage
- **Graph invariants**: Improved testing and validation of graph constraints
- **API improvements**: Minor enhancements to graph building and node management

## [0.2.0] - 2026-01-04
- Added trait-based external node hook (`NodeType::External`) with object-safe `NodeDef` adapter.
- Port metadata now uses static slices to avoid allocations in hot paths.
- Runtime preallocates external node state and routes without per-block allocations.
- Backward compatibility preserved for existing nodes and graphs.

## [0.1.1] - 2026-01-03
- **Bug fixes**: Improved error handling in `render_offline` to propagate `process_block` errors instead of panicking.
- **API enhancement**: Added `get_node_by_name` getter to `GraphBuilder` for accessing named nodes.
- **Invariant clarity**: Updated cycle detection assertion message for better readability in PPT mode.
- **Maintenance**: Minor code polish and documentation updates.

## [0.1.0] - 2026-01-03
- Initial release of the audio graph kernel.
- Core architecture: Graph → Plan → Runtime.
- RT-safe block processing with no allocs/locks in hot paths.
- Deterministic execution.
- Invariants: single-writer, no cycles, rate compatibility.
- Minimal DSP nodes: SineOsc, Gain, Mix, OutputSink.
- Comprehensive tests and benchmarks.
- PPT (Predictive Property-Based Testing) system with runtime invariant logging and contract tests.
- **Correctness hardening**: Reject invalid block sizes, enforce edge directions, bounds checks, phase wrapping.
<parameter name="filePath">c:/Users/micha/repos/auxide/CHANGELOG.md