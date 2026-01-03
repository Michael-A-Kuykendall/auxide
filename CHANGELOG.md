# Changelog

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