<p align="center">
  <img src="./auxide-logo.png" alt="Auxide" width="500" />
</p>

A high-performance, Rust-native framework for building real-time audio processing pipelines through a statically validated signal-graph DSL.

## Overview

Auxide enables deterministic execution with real-time safety in mind, making it ideal for institutional-grade applications like professional audio tools, embedded systems, and research-grade DSP.

## Features

- **Statically Validated Graphs**: Correct-by-construction signal graphs with explicit rates and rate-checked ports.
- **Deterministic Runtime**: Block-based pull execution ensuring predictable performance and edge propagation.
- **Real-Time Safe**: No allocations or locks on the audio thread, with empirical proofs via harness.
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
use auxide::graph::NodeType;

let mut builder = GraphBuilder::new();
let sine = builder.node(NodeType::SineOsc { freq: 440.0 });
let gain = builder.node(NodeType::Gain { gain: 0.5 });
builder.connect(sine, auxide::graph::PortId(0), gain, auxide::graph::PortId(0), auxide::graph::Rate::Audio).unwrap();
let graph = builder.build().unwrap();
```

Compile and run:

```rust
use auxide::plan::Plan;
use auxide::rt::Runtime;

let plan = Plan::compile(&graph, 512).unwrap();
let mut runtime = Runtime::new(plan, &graph);
let mut out_block = vec![0.0; 512];
runtime.process_block(&mut out_block);
```

## Documentation

- [DESIGN.md](.docs/DESIGN.md): Goals, invariants, and non-goals.
- [SAFETY.md](.docs/SAFETY.md): RT rules and enforcement.
- [PROOFS.md](.docs/PROOFS.md): Test mappings for all claims.

## Benchmarks

Run `cargo bench` for performance metrics on real workloads.

## License

MIT