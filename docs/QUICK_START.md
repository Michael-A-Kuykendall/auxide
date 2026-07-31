# Auxide Quick Start

## Installation

```toml
[dependencies]
auxide = "0.3"
```

For DSP nodes, I/O, or MIDI, add the respective crates:

```toml
auxide-dsp = "0.2"   # oscillators, filters, effects, envelopes
auxide-io  = "0.1"   # live audio streaming (CPAL)
auxide-midi = "0.1"  # MIDI input + polyphonic synth
```

## Your First Audio Graph

```rust
use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;

fn main() {
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

    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph, 44100.0);
    let mut out = vec![0.0; 64];
    runtime.process_block(&mut out).unwrap();

    println!("Generated {} samples of 440Hz sine", out.len());
}
```

## Using the Control Plane (Preferred API)

For real-time control and live streaming, use `RuntimeCore`/`RuntimeControl`:

```rust
use auxide::graph::{Graph, NodeType, PortId, Rate, Edge};
use auxide::plan::Plan;
use auxide::rt::RuntimeCore;

let (mut handle, mut control) = RuntimeCore::new_with_channels(plan, &graph, 44100.0);

// Send control messages from any thread (lock-free SPSC queue)
control.set_frequency(osc_node, 440.0).ok();
control.trigger_gate(osc_node, true).ok();
control.set_gain(osc_node, 0.5).ok();

// Process one block — drains the control queue
let mut out = vec![0.0f32; 64];
handle.process_block(&mut out).unwrap();
```

See [Advanced Examples](ADVANCED_EXAMPLES.md) for offline rendering, external custom nodes, FM modulation, delayed feedback edges, and more.

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

For DSP examples (filters, envelopes, effects, sample playback), see the
[auxide-dsp README](https://github.com/Michael-A-Kuykendall/auxide-dsp).
For live audio streaming, diagnostics, and recording, see the
[auxide-io README](https://github.com/Michael-A-Kuykendall/auxide-io).
For MIDI input, voice allocation, and CC mapping, see the
[auxide-midi README](https://github.com/Michael-A-Kuykendall/auxide-midi).