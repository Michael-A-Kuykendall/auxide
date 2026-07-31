<div align="center">
  <img src="assets/auxide-logo.png" alt="Auxide Logo" width="400" height="auto" />

  # The RT-Safe Audio Graph Kernel for Rust

  ### Deterministic. Minimal. Real-Time Safe.

  [![Crates.io](https://img.shields.io/crates/v/auxide.svg)](https://crates.io/crates/auxide)
  [![Documentation](https://docs.rs/auxide/badge.svg)](https://docs.rs/auxide)
  [![CI](https://github.com/Michael-A-Kuykendall/auxide/workflows/CI/badge.svg)](https://github.com/Michael-A-Kuykendall/auxide/actions)
  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

  [![💝 Sponsor this project](https://img.shields.io/badge/💝_Sponsor_this_project-ea4aaa?style=for-the-badge&logo=github&logoColor=white)](https://github.com/sponsors/Michael-A-Kuykendall)
</div>

**Auxide will be free forever.** No asterisks. No "free for now." No pivot to paid.

### 💝 Support Auxide's Growth

🚀 **If Auxide helps you build amazing audio tools, consider [sponsoring](https://github.com/sponsors/Michael-A-Kuykendall) — 100% of support goes to keeping it free forever.**

- **$5/month**: Coffee tier ☕ — Eternal gratitude + sponsor badge
- **$25/month**: Bug prioritizer 🐛 — Priority support + name in [SPONSORS.md](SPONSORS.md)
- **$100/month**: Corporate backer 🏢 — Logo placement + monthly office hours
- **$500/month**: Infrastructure partner 🚀 — Direct support + roadmap input

[**🎯 Become a Sponsor**](https://github.com/sponsors/Michael-A-Kuykendall) | See our amazing [sponsors](SPONSORS.md) 🙏

---

## Table of Contents

- [What Is Auxide?](#what-is-auxide)
- [Auxide Ecosystem](#auxide-ecosystem)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Key Features](#key-features)
- [Non-Goals](#non-goals)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## What Is Auxide?

Auxide is a low-level, RT-safe kernel for executing audio graphs deterministically. Unlike full DAWs or plugin hosts, it's a programmable building block — think of it as the engine under the hood.

Audio processing as a **directed acyclic graph (DAG)**:
- **Nodes**: DSP units (oscillators, filters, effects).
- **Edges**: Signal flow between nodes.
- **Execution**: Topological order ensures no feedback loops.

| Feature | Auxide | SuperCollider | Rodio | CPAL |
|---------|--------|---------------|-------|------|
| RT-Safe | ✅ | ❌ | ❌ | ✅ |
| Graph-Based | ✅ | ✅ | ❌ | ❌ |
| Deterministic | ✅ | ❌ | ❌ | ❌ |
| Minimal API | ✅ | ❌ | ✅ | ✅ |
| Rust Native | ✅ | ❌ | ✅ | ✅ |

---

## Auxide Ecosystem

Auxide is not one crate — it's a growing **ecosystem of 7 crates**:

| Crate | Description | Status |
|-------|-------------|--------|
| **[auxide](https://github.com/Michael-A-Kuykendall/auxide)** | Audio graph kernel | active |
| **[auxide-dsp](https://github.com/Michael-A-Kuykendall/auxide-dsp)** | DSP node library | active |
| **[auxide-io](https://github.com/Michael-A-Kuykendall/auxide-io)** | Audio I/O layer | active |
| **[auxide-midi](https://github.com/Michael-A-Kuykendall/auxide-midi)** | MIDI integration | active |
| **auxide-server** | Node-graph server | in development |
| **auxide-proto** | Wire protocol | in development |
| **auxide-conductor** | Composition/transport | in development |

---

## Quick Start

```bash
cargo add auxide
```

Then create your first audio graph — see the [Quick Start Guide](docs/QUICK_START.md) for a complete walkthrough.

Browse all [examples](examples/) or run one directly:

```bash
cargo run --example basic_sine
```

For advanced patterns (fan-out mixing, offline rendering, DSP integration), see [Advanced Examples](docs/ADVANCED_EXAMPLES.md).

---

## Architecture

Auxide's three-phase pipeline ensures reliability:

1. **Graph Building** — Construct your DAG with nodes and edges.
2. **Plan Compilation** — Validate invariants, optimize for execution.
3. **Runtime Execution** — Process audio blocks deterministically.

### Key Invariants

- **Single-writer**: One edge per input port.
- **No cycles**: Acyclic graphs only.
- **Rate compatibility**: Audio/Control rates match.
- **Determinism**: Same inputs → same outputs.

For full architectural details, see [ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

## Key Features

- **RT-Safe**: No allocs/locks in hot paths.
- **Deterministic**: Reproducible output.
- **Minimal**: Small API surface, easy to learn.
- **Extensible**: Add nodes via traits.
- **Tested**: Fuzzing, property tests, benchmarks.
- **Performant**: Low-latency block processing.

### Integration Gate

The four live crates (`auxide`, `auxide-dsp`, `auxide-io`, `auxide-midi`) are linked by path dependencies. A single command proves the whole stack builds, lints, and tests:

<details>
<summary>Cross-crate verification</summary>

```bash
./verify_all.sh        # CI / Linux (or git-bash / WSL on Windows)
pwsh ./verify_all.ps1  # native Windows
```

It `cd`s into each crate and runs `cargo build && cargo test && cargo clippy --all-targets -- -D warnings`, exiting non-zero on the first failure.

The cross-crate smoke test lives in `auxide-midi/tests/integration_gate.rs` (SynthBuilder graph → kernel render → non-zero; plus MIDI voice-pool allocate/release).
</details>

---

## Non-Goals

- GUI or DAW features.
- Plugin formats (VST, etc.).
- Live coding environments.
- Multichannel beyond mono.
- OS audio backends.

Auxide is the foundation — build your tools on top.

---

## Roadmap

- **v0.3**: Stable API with simplified runtime. ✅
- **v1.0**: Performance optimizations, stereo support, more built-in nodes.
- **v1.x+**: Server, protocol, conductor crates.

See the full [Roadmap](ROADMAP.md) for version history and future plans.

---

## Contributing

Auxide is open source but not open contribution. See [CONTRIBUTING.md](CONTRIBUTING.md) for the collaboration policy.

---

## License

MIT