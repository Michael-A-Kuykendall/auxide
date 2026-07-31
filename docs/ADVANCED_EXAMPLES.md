# Advanced Auxide Examples

## Fan-Out and Mixing

Route one signal to multiple processors, then mix back:

```rust
use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;

fn main() {
    let mut graph = Graph::new();
    let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    let gain1 = graph.add_node(NodeType::Gain { gain: 0.5 });
    let gain2 = graph.add_node(NodeType::Gain { gain: 0.3 });
    let mixer = graph.add_node(NodeType::Mix);
    let sink = graph.add_node(NodeType::OutputSink);

    // Fan out: osc feeds both gains
    graph.add_edge(auxide::graph::Edge {
        from_node: osc, from_port: PortId(0),
        to_node: gain1, to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    graph.add_edge(auxide::graph::Edge {
        from_node: osc, from_port: PortId(0),
        to_node: gain2, to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    // Mix attenuated signals
    graph.add_edge(auxide::graph::Edge {
        from_node: gain1, from_port: PortId(0),
        to_node: mixer, to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    graph.add_edge(auxide::graph::Edge {
        from_node: gain2, from_port: PortId(0),
        to_node: mixer, to_port: PortId(1),
        rate: Rate::Audio,
    }).unwrap();
    graph.add_edge(auxide::graph::Edge {
        from_node: mixer, from_port: PortId(0),
        to_node: sink, to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph, 44100.0);
    let mut out = vec![0.0; 64];
    runtime.process_block(&mut out).unwrap();
}
```

This demonstrates **parallel processing** and **signal combination** — core to audio graphs.

## Offline Rendering

Process entire buffers for non-real-time tasks:

```rust
use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;

fn main() {
    let mut graph = Graph::new();
    let osc = graph.add_node(NodeType::SineOsc { freq: 1000.0 });
    let sink = graph.add_node(NodeType::OutputSink);
    graph.add_edge(auxide::graph::Edge {
        from_node: osc, from_port: PortId(0),
        to_node: sink, to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    let plan = Plan::compile(&graph, 1024).unwrap();
    let mut runtime = Runtime::new(plan, &graph, 44100.0);

    // Render 1 second of audio
    let mut buffer = vec![0.0; 44100];
    for chunk in buffer.chunks_mut(1024) {
        runtime.process_block(chunk).unwrap();
    }
}
```

Perfect for batch processing, analysis, or exporting.

---

## Usage Patterns

### Building a Synth
Extend `NodeType` with custom oscillators and filters. Use Auxide for the graph engine. Pair with `auxide-dsp` for a full DSP node library.

### Game Audio
Dynamic graphs for sound design — RT-safe for frame rates.

### Prototyping DSP
Quickly test ideas without real-time constraints.

### Integration
Pair with [`auxide-io`](https://github.com/Michael-A-Kuykendall/auxide-io) for playback, [`auxide-dsp`](https://github.com/Michael-A-Kuykendall/auxide-dsp) for DSP nodes, and [`auxide-midi`](https://github.com/Michael-A-Kuykendall/auxide-midi) for MIDI control.

---

See [Quick Start](QUICK_START.md) for the basics, or browse all [examples](../examples/).