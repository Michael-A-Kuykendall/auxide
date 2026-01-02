# Auxide

A high-performance, Rust-native framework for building real-time audio processing pipelines through a statically validated signal-graph DSL.

## Overview

Auxide enables deterministic execution with real-time safety in mind, making it ideal for institutional-grade applications like professional audio tools, embedded systems, and research-grade DSP.

**Note**: Current runtime is a scaffold; edges and buffers are not yet executed — only preallocated I/O copying is supported.

## Features

- **Statically Validated Graphs**: Correct-by-construction signal graphs with explicit rates and rate-checked ports.
- **Deterministic Runtime**: Block-based pull execution ensuring predictable performance.
- **Real-Time Safe**: Designed for no allocations or locks on the audio thread (requires preallocated buffers).
- **Extensible DSL**: Fluent Rust API for building graphs without macros.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
auxide = "0.1"
```

Build a simple graph:

```rust
use auxide::dsl::GraphBuilder;
use auxide::graph::{Port, PortId, Rate, NodeType};

let mut builder = GraphBuilder::new();
let sine = builder.node(vec![Port { id: PortId(0), rate: Rate::Audio }], NodeType::SineOsc { freq: 440.0, phase: 0.0 });
let gain = builder.node(vec![Port { id: PortId(0), rate: Rate::Audio }, Port { id: PortId(1), rate: Rate::Audio }], NodeType::Gain { gain: 0.5 });
builder.connect(sine, PortId(0), gain, PortId(0), Rate::Audio).unwrap();
let graph = builder.build().unwrap();
```

Compile and run:

```rust
use auxide::plan::Plan;
use auxide::rt::Runtime;

let plan = Plan::compile(&graph).unwrap();
let mut runtime = Runtime::new(plan, &graph);
// Process audio...
```

## Documentation

- [DESIGN.md](DESIGN.md): Goals, invariants, and non-goals.
- [SAFETY.md](SAFETY.md): RT rules and enforcement.

## Benchmarks

Run `cargo bench` for performance metrics.

## License

MIT