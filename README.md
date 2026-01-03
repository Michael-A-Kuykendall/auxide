# Auxide

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
let mut runtime = Runtime::new(plan, &graph);
let mut out = vec![0.0; 64];
runtime.process_block(&mut out);
```

## Non-Goals

- GUI
- DAW
- Plugin formats
- Live coding
- Multichannel beyond mono
- Runtime graph mutation

## Proofs

- [ARCHITECTURE.md](.docs/ARCHITECTURE.md): System design.
- [RT_RULES.md](.docs/RT_RULES.md): Real-time constraints.
- [INVARIANTS.md](.docs/INVARIANTS.md): Proven properties.
- [FAQ.md](.docs/FAQ.md): Common questions.

## License

MIT