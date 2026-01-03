# Changelog

## [0.1.0] - 2026-01-03
- Initial release of the audio graph kernel.
- Core architecture: Graph → Plan → Runtime.
- RT-safe block processing with no allocs/locks in hot paths.
- Deterministic execution.
- Invariants: single-writer, no cycles, rate compatibility.
- Minimal DSP nodes: SineOsc, Gain, Mixer, Silence, OutputSink.
- Comprehensive tests and benchmarks.</content>
<parameter name="filePath">c:/Users/micha/repos/auxide/CHANGELOG.md