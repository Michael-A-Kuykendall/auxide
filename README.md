<img src="https://raw.githubusercontent.com/Michael-A-Kuykendall/auxide/main/assets/auxide-logo.png" alt="Auxide Logo" width="400">

[![Crates.io](https://img.shields.io/crates/v/auxide.svg)](https://crates.io/crates/auxide)
[![Documentation](https://docs.rs/auxide/badge.svg)](https://docs.rs/auxide)
[![CI](https://github.com/Michael-A-Kuykendall/auxide/workflows/CI/badge.svg)](https://github.com/Michael-A-Kuykendall/auxide/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

# Auxide

**A real-time-safe, deterministic audio graph kernel for Rust.**  
Build reliable audio tools, DSP chains, and synthesis engines with a focus on correctness, performance, and simplicity.

## Overview

Auxide is a real-time-safe, deterministic audio graph kernel for Rust. It provides the foundation for building reliable audio tools, DSP chains, and synthesis engines with a focus on correctness, performance, and simplicity.

### Key Features

- **Real-time Safe**: Zero allocations in audio processing paths
- **Deterministic**: Predictable, reproducible audio output
- **Graph-based**: Flexible node-based audio processing architecture
- **Rust Native**: Built for Rust's ownership and borrowing system
- **Extensible**: Plugin architecture for custom nodes

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
auxide = "0.3"
```

## Example

```rust
use auxide::{AudioGraph, NodeType, NodeId, ProcessContext};

// Create a simple audio graph
let mut graph = AudioGraph::new(44100.0, 512);

// Add nodes to the graph
let sine_node = graph.add_node(NodeType::SineOsc { freq: 440.0 })?;
let output_node = graph.add_node(NodeType::Output { channels: 2 })?;

// Connect nodes
graph.connect(sine_node, output_node)?;

// Process audio
let mut buffer = vec![0.0; 512];
graph.process(&mut buffer)?;
```

## Architecture

Auxide uses a pull-based graph execution model where each node processes audio on demand. This ensures:

- No buffer copying between nodes
- Minimal latency
- Efficient CPU usage
- Deterministic execution order

## Ecosystem

Auxide is designed as a modular ecosystem:

- **auxide**: Core kernel (this crate)
- **auxide-dsp**: DSP nodes and utilities
- **auxide-io**: Audio I/O layer
- **auxide-midi**: MIDI integration

## Community & Support

• 🐛 Bug Reports: [GitHub Issues](https://github.com/Michael-A-Kuykendall/auxide/issues)
• 💬 Discussions: [GitHub Discussions](https://github.com/Michael-A-Kuykendall/auxide/discussions)
• 📖 Documentation: [docs.rs](https://docs.rs/auxide)
• 💝 Sponsorship: [GitHub Sponsors](https://github.com/sponsors/Michael-A-Kuykendall)
• 🤝 Contributing: [CONTRIBUTING.md](https://github.com/Michael-A-Kuykendall/auxide/blob/main/CONTRIBUTING.md)
• 📜 Governance: [GOVERNANCE.md](https://github.com/Michael-A-Kuykendall/auxide/blob/main/GOVERNANCE.md)
• 🔒 Security: [SECURITY.md](https://github.com/Michael-A-Kuykendall/auxide/blob/main/SECURITY.md)

## License & Philosophy

MIT License - forever and always.

**Philosophy**: Audio infrastructure should be invisible. Auxide is infrastructure.

**Testing Philosophy**: Reliability through comprehensive validation and formal verification.

**Forever maintainer**: Michael A. Kuykendall  
**Promise**: This will never become a paid product  
**Mission**: Making real-time audio DSP simple and reliable

## Auxide Ecosystem
| Crate | Description | Version |
|-------|-------------|---------|
| **[auxide](https://github.com/Michael-A-Kuykendall/auxide)** | Real-time-safe audio graph kernel | 0.3.0 |
| [auxide-dsp](https://github.com/Michael-A-Kuykendall/auxide-dsp) | DSP nodes library | 0.2.0 |
| [auxide-io](https://github.com/Michael-A-Kuykendall/auxide-io) | Audio I/O layer | 0.2.0 |
| [auxide-midi](https://github.com/Michael-A-Kuykendall/auxide-midi) | MIDI integration | 0.2.0 |
