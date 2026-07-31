# Auxide Roadmap

## Past Releases

### v0.3.x — Kernel Stabilization & Ecosystem Expansion

| Version | Date | Key Changes |
|---------|------|-------------|
| 0.3.2 | 2026-07-29 | Ecosystem docs; glitch diagnostics; opencode config; public sync |
| 0.3.1 | 2026-01-16 | Updated sponsors; API simplifications |
| 0.3.0 | 2026-01-16 | Removed external node support; port refactor (static slices → Vec<Port>); code cleanup |

### v0.2.x — RT Control & Cross-Crate Wiring

| Version | Date | Key Changes |
|---------|------|-------------|
| 0.2.2 | 2026-01-15 | RT control plane (RuntimeCore/RuntimeControl); lock-free SPSC queues; invariant signaling; dual-license |
| 0.2.1 | 2026-01-07 | Bug fixes; documentation; RT safety verification; cross-crate compatibility |
| 0.2.0 | 2026-01-04 | Trait-based external node hook; NodeType::External; external node state preallocation |

### v0.1.x — Foundation

| Version | Date | Key Changes |
|---------|------|-------------|
| 0.1.1 | 2026-01-03 | Error handling; graph builder getter; invariant clarity |
| 0.1.0 | 2026-01-03 | Initial release: Graph → Plan → Runtime pipeline; RT-safe block processing; deterministic execution |

---

## External Crate Releases

### auxide-dsp

| Version | Date | Key Changes |
|---------|------|-------------|
| 0.2.1 | 2026-07-29 | Ecosystem docs, public sync |
| 0.2.0 | 2026-07-25 | Band-limited oscillators (PolyBLEP); RT safety verification; full DSP node library; builders; golden tests |
| 0.1.1 | 2026-01-07 | Phase modulo guards; documentation; RT safety verification |
| 0.1.0 | 2026-01-05 | Initial release: oscillators, filters, effects, envelopes, LFO, wavetables |

### auxide-io

| Version | Date | Key Changes |
|---------|------|-------------|
| 0.1.3 | 2026-07-29 | Glitch diagnostics; buffer underflow detection; ecosystem docs |
| 0.1.2 | 2026-01-07 | Documentation; error handling; auxide 0.2.1 compatibility; RT safety verification |
| 0.1.1 | 2026-01-03 | RT-safety fix; auxide 0.2.0 compatibility; error recovery |
| 0.1.0 | 2026-01-03 | Initial release: CPAL integration; buffer adaptation; channel routing; error recovery |

### auxide-midi

| Version | Date | Key Changes |
|---------|------|-------------|
| 0.1.2 | 2026-07-29 | Glitch logging in examples; ecosystem docs |
| 0.1.1 | 2026-01-07 | Voice allocator fix; documentation; RT safety verification |
| 0.2.0 | 2026-01-05 | Comprehensive MIDI integration; voice allocator; CC mapping; pitch bend; smoothing utilities |
| 0.1.0 | 2026-01-05 | Initial release: MIDI input; voice allocation; CC mapping; polyphonic support |

---

## Future Plans

### v1.0 — Production Readiness

- **Performance optimizations**: Block processing throughput improvements, cache-friendly layout
- **Stereo support**: Full stereo signal path throughout the graph
- **More built-in nodes**: Expanded DSP node library
- **API stabilization**: 1.0 API guarantees
- **Documentation**: Comprehensive guides, cookbook, API reference

### v1.x+ — Ecosystem Growth

- **auxide-server**: Live, multi-instance, addressable node-graph server
- **auxide-proto**: Wire protocol + client (OSC + WebSocket codecs)
- **auxide-conductor**: Composition, scheduling, and transport layer
- **Plugin system**: External node loading at runtime
- **DAW integration**: VST3 / CLAP plugin format support
- **Community tooling**: Visual graph editor, patch library

---

*This roadmap is a living document. Priorities shift based on sponsor needs and real-world usage. If there's something you need, [open an issue](https://github.com/Michael-A-Kuykendall/auxide/issues) or [become a sponsor](https://github.com/sponsors/Michael-A-Kuykendall).*