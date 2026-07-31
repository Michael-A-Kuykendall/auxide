# Auxide Quick Start

## Installation

```toml
[dependencies]
auxide = "0.3"
```

## Your First Audio Graph

```rust
use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;

fn main() {
    // Build graph
    let mut graph = Graph::new();
    let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    let sink = graph.add_node(NodeType::OutputSink);
    graph.add_edge(auxide::graph::Edge {
        from_node: osc,
        from_port: PortId(0),
        to_node: sink,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    // Compile plan
    let plan = Plan::compile(&graph, 64).unwrap();

    // Run runtime
    let mut runtime = Runtime::new(plan, &graph, 44100.0);
    let mut out = vec![0.0; 64];
    runtime.process_block(&mut out).unwrap();

    println!("Generated {} samples of 440Hz sine", out.len());
}
```

## Running Examples

Clone the repo and explore:

```bash
git clone https://github.com/Michael-A-Kuykendall/auxide.git
cd auxide
cargo run --example basic_sine
```

Available examples in [`examples/`](examples/):
- [`basic_sine`](examples/basic_sine.rs) — Simple oscillator
- [`gain_chain`](examples/gain_chain.rs) — Signal processing chain
- [`mixer`](examples/mixer.rs) — Multi-input mixing
- [`offline_render`](examples/offline_render.rs) — Full buffer rendering
- [`am_synth`](examples/am_synth.rs) — Amplitude modulation demo
- [`filter_chain`](examples/filter_chain.rs) — Basic filter approximation
- [`sequencer`](examples/sequencer.rs) — Note sequencing
- [`rompler_demo`](examples/rompler_demo.rs) — ROMpler synthesis demo
- [`proof_it_works`](examples/proof_it_works.rs) — Proof of concept

---

For advanced usage patterns, see [Advanced Examples](docs/ADVANCED_EXAMPLES.md).