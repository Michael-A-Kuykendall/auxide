# Changelog

## [0.3.1] - 2026-01-16
- Release v0.3.1 with updated sponsors and API simplifications.

## [0.3.0] - 2026-01-16
- **API Simplification**: Removed external node support, control plane, and invariant signaling for a stable, minimal API.
- **Port Refactor**: Changed NodeType ports from static slices to Vec<Port> for easier extensibility.
- **Sponsors Update**: Added new corporate backer ZephyrCloudIO, developer supporter Omar McIver, and coffee hero alistairheath.
- **Code Cleanup**: Removed unused code paths, simplified error handling, and updated tests.
- **Stability**: Focused on core audio graph functionality with RT-safe execution.

## [0.2.2] - 2026-01-15
- **RT control plane**: Split runtime into `RuntimeCore` (audio) and `RuntimeControl` (main) with lock-free SPSC queues for control messages.
- **Invariant signaling**: Added RT-safe invariant queue (`invariant_rt`) plus contract tests covering buffer fill, control delivery, and mute/reset flows.
- **Dual licensing**: Project now dual-licensed MIT OR Apache-2.0; added `LICENSE-MIT` and `LICENSE-APACHE`.
- **Docs**: Clarified architecture split and RT invariant signaling; fixed mojibake/tool artifacts in Markdown.

## [0.2.1] - 2026-01-07
- **Bug fixes**: Correctness improvements in phase accumulation and error handling.
- **Documentation**: Enhanced module-level documentation and API clarity.
- **RT safety**: Verified zero allocations in `process_block` paths.
- **Compatibility**: Confirmed interoperability with auxide-dsp 0.1.1, auxide-io 0.1.2, and auxide-midi 0.1.1.
- **Testing**: All unit and integration tests passing.

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
