<img src="assets/auxide-logo.png" alt="Auxide Logo" width="400">

Auxide is a real-time-safe, deterministic audio graph kernel for building audio tools.

## Minimal Example

```rust
use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;

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
## Fan-Out/Mix Example

```rust
use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;

let mut graph = Graph::new();
let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
let gain1 = graph.add_node(NodeType::Gain { gain: 0.5 });
let gain2 = graph.add_node(NodeType::Gain { gain: 0.3 });
let mixer = graph.add_node(NodeType::Mixer);
let sink = graph.add_node(NodeType::OutputSink);

// Fan out: osc to both gains
graph.add_edge(auxide::graph::Edge {
    from_node: osc,
    from_port: PortId(0),
    to_node: gain1,
    to_port: PortId(0),
    rate: Rate::Audio,
}).unwrap();
graph.add_edge(auxide::graph::Edge {
    from_node: osc,
    from_port: PortId(0),
    to_node: gain2,
    to_port: PortId(0),
    rate: Rate::Audio,
}).unwrap();

// Mix back together
graph.add_edge(auxide::graph::Edge {
    from_node: gain1,
    from_port: PortId(0),
    to_node: mixer,
    to_port: PortId(0),
    rate: Rate::Audio,
}).unwrap();
graph.add_edge(auxide::graph::Edge {
    from_node: gain2,
    from_port: PortId(0),
    to_node: mixer,
    to_port: PortId(1),
    rate: Rate::Audio,
}).unwrap();
graph.add_edge(auxide::graph::Edge {
    from_node: mixer,
    from_port: PortId(0),
    to_node: sink,
    to_port: PortId(0),
    rate: Rate::Audio,
}).unwrap();

let plan = Plan::compile(&graph, 64).unwrap();
let mut runtime = Runtime::new(plan, &graph, 44100.0);
let mut out = vec![0.0; 64];
runtime.process_block(&mut out);
```

## Invariants

- **Single-writer**: Only one edge may write to a given input port.
- **No cycles**: Graphs must be acyclic (except delays).
- **Rate compatibility**: Connected ports must match rates.
- **Determinism**: Same inputs produce same outputs (modulo floating-point).

## Real-Time Safety

RT paths (e.g., `Runtime::process_block`) avoid allocations and locking. Graph building and plan compilation may allocate and are not RT-safe.

## Non-Goals

- GUI or DAW interfaces
- Plugin formats (VST, etc.)
- Live coding or interactive editing
- Multichannel audio (mono only)
- Runtime graph mutation
- OS audio backend integration
- DSP library; provides execution kernel only