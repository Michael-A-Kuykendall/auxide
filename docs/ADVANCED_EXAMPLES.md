# Advanced Auxide Examples

## Control Plane — Sending Messages to the Runtime

The `RuntimeCore`/`RuntimeControl` split enables lock-free communication between the main thread and the audio callback:

```rust
use auxide::graph::{Graph, NodeType, PortId, Rate, Edge};
use auxide::plan::Plan;
use auxide::rt::{RuntimeCore, RuntimeControl};
use auxide::control::ControlMsg;

let mut graph = Graph::new();
let osc = graph.add_node(NodeType::SineOsc { freq: 220.0 });
let sink = graph.add_node(NodeType::OutputSink);
graph.add_edge(Edge {
    from_node: osc, from_port: PortId(0),
    to_node: sink, to_port: PortId(0), rate: Rate::Audio,
}).unwrap();
let plan = Plan::compile(&graph, 64).unwrap();

let (mut handle, mut control) = RuntimeCore::new_with_channels(plan, &graph, 44100.0);

// Send control messages — these reach the audio thread lock-free
control.set_frequency(osc, 440.0).ok();
control.set_gain(osc, 0.5).ok();
control.trigger_gate(osc, true).ok();
control.send(ControlMsg::Mute { node: osc }).ok();
control.send(ControlMsg::AllNotesOff).ok();

// Process one block (drains the control queue)
let mut out = vec![0.0f32; 64];
handle.process_block(&mut out).unwrap();
```

### Convenience Methods on RuntimeControl

| Method | Equivalent ControlMsg |
|--------|----------------------|
| `set_frequency(node, hz)` | `SetFrequency { node, hz }` |
| `set_gain(node, gain)` | `SetGain { node, gain }` |
| `trigger_gate(node, on)` | `TriggerGate { node, on }` |
| `mute(node)`, `unmute(node)` | `Mute` / `Unmute` |

## External Nodes — Custom DSP in the Graph

Implement the `NodeDef` trait for custom processing:

```rust
use auxide::node::{NodeDef, Port};
use auxide::graph::{PortId, Rate};

#[derive(Debug, Clone)]
struct MyProcessor { gain: f32 }

impl NodeDef for MyProcessor {
    type State = f32;

    fn input_ports() -> &'static [Port] {
        &[Port { id: PortId(0), rate: Rate::Audio }]
    }
    fn output_ports() -> &'static [Port] {
        &[Port { id: PortId(0), rate: Rate::Audio }]
    }
    fn required_inputs() -> usize { 1 }

    fn init_state(&self, _sample_rate: f32, _block_size: usize) -> Self::State {
        0.0
    }

    fn process_block(&self, state: &mut Self::State,
                     inputs: &[&[f32]], outputs: &mut [Vec<f32>],
                     _sample_rate: f32) {
        // Simple DC blocker: y[n] = x[n] - x[n-1] + 0.995 * y[n-1]
        for i in 0..outputs[0].len() {
            let x = inputs[0][i];
            let y = x - *state + 0.995 * outputs[0][i-1].max(0.0);
            outputs[0][i] = y;
            *state = x;
        }
    }

    fn set_param(&self, state: &mut Self::State, param: u8, value: f32) {
        // Handle canonical PARAM_FREQUENCY, PARAM_CUTOFF, etc.
    }

    fn gate(&self, state: &mut Self::State, on: bool) {
        // Reset state on note-on
    }
}

// Use in a graph:
let node = graph.add_external_node(MyProcessor { gain: 0.8 });
```

## Offline Rendering — Render to Buffer

```rust
use auxide::graph::{Graph, NodeType, PortId, Rate, Edge};
use auxide::plan::Plan;
use auxide::rt::{RuntimeCore, render_offline_handle};

let mut graph = Graph::new();
let osc = graph.add_node(NodeType::SineOsc { freq: 1000.0 });
let sink = graph.add_node(NodeType::OutputSink);
graph.add_edge(Edge {
    from_node: osc, from_port: PortId(0),
    to_node: sink, to_port: PortId(0), rate: Rate::Audio,
}).unwrap();
let plan = Plan::compile(&graph, 1024).unwrap();

let (mut handle, mut control) = RuntimeCore::new_with_channels(plan, &graph, 44100.0);

// Send a frequency change mid-render
control.set_frequency(osc, 880.0).ok();

// Render 1 second of audio — control messages are drained each block
let buffer = render_offline_handle(&mut handle, 44100).unwrap();
// buffer now holds 1s of audio (440 Hz -> 880 Hz at the message point)
```

## Graph-Native Frequency Modulation

The `SineOsc` node has an FM input port (port 0). Patch any signal there for per-sample frequency modulation:

```rust
let mut graph = Graph::new();
let carrier = graph.add_node(NodeType::SineOsc { freq: 200.0 });
let modulator = graph.add_node(NodeType::SineOsc { freq: 440.0 });
let sink = graph.add_node(NodeType::OutputSink);

// Patch modulator output into carrier's FM input (port 0)
graph.add_edge(Edge {
    from_node: modulator, from_port: PortId(0),
    to_node: carrier, to_port: PortId(0), // FM input
    rate: Rate::Audio,
}).unwrap();
graph.add_edge(Edge {
    from_node: carrier, from_port: PortId(0),
    to_node: sink, to_port: PortId(0),
    rate: Rate::Audio,
}).unwrap();
```

Same pattern works for `Gain` node's port 1 (gain modulation).

## Control-Rate Edges (Downsampling)

Connect an audio-rate source to a Control-rate input for block-rate sampling:

```rust
graph.add_edge(Edge {
    from_node: audio_source, from_port: PortId(0),
    to_node: control_target, to_port: PortId(0),
    rate: Rate::Control, // sampled once per block into a presentation buffer
}).unwrap();
```

This is legal per the rate-compat rule: `edge.rate == Control` while `from_rate == Audio`.

## Delayed Edges (1-Block Feedback)

```rust
graph.add_delayed_edge(Edge {
    from_node: output, from_port: PortId(0),
    to_node: input, to_port: PortId(0),
    rate: Rate::Audio,
}).unwrap();
// Excluded from cycle detection; destination reads previous block's value.
```

## Fan-Out and Mixing

Route one signal to multiple processors, then mix back:

```rust
use auxide_dsp::nodes::utility::Mixer;

let mut graph = Graph::new();
let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
let gain1 = graph.add_node(NodeType::Gain { gain: 0.5 });
let gain2 = graph.add_node(NodeType::Gain { gain: 0.3 });
let mixer = graph.add_external_node(Mixer::new(2));
let sink = graph.add_node(NodeType::OutputSink);

// Fan out: osc feeds both gains
for &gain in &[gain1, gain2] {
    graph.add_edge(Edge {
        from_node: osc, from_port: PortId(0),
        to_node: gain, to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
}

// Mix into stereo output
graph.add_edge(Edge { from_node: gain1, from_port: PortId(0),
    to_node: mixer, to_port: PortId(0), rate: Rate::Audio }).unwrap();
graph.add_edge(Edge { from_node: gain2, from_port: PortId(0),
    to_node: mixer, to_port: PortId(1), rate: Rate::Audio }).unwrap();
graph.add_edge(Edge { from_node: mixer, from_port: PortId(0),
    to_node: sink, to_port: PortId(0), rate: Rate::Audio }).unwrap();
```

## PPT Testing — Verify Invariants

The property-based testing system verifies core invariants:

```rust
// Enabled with the "ppt" feature:
// auxide = { features = ["ppt"] }

use auxide::invariant_ppt::assert_invariant;

assert_invariant(
    auxide::invariant_ppt::GRAPH_LEGALITY, // ID 3
    graph.is_valid(),
    "graph must be valid after construction",
    &graph,
);

// Contract tests bundle required invariants:
auxide::invariant_ppt::contract_test(
    "graph_construction",
    &[3, 4, 5] // GRAPH_LEGALITY, PLAN_COMPLETENESS, PLAN_SOUNDNESS
);
```

## Using the Registry + DSL

Define named patch presets:

```rust
use auxide::registry::{Registry, param_or};
use auxide_dsp::registry::register_dsp_ugens;

let mut reg = Registry::new();
register_dsp_ugens(&mut reg);

// Create nodes by name with parameter overrides
use std::collections::HashMap;
let mut params = HashMap::new();
params.insert("freq".to_string(), 440.0);
params.insert("pulse_width".to_string(), 0.5);

let node_type = reg.create("pulse", &params).unwrap();

// Or use the GraphBuilder DSL:
use auxide::dsl::GraphBuilder;

let graph = GraphBuilder::new()
    .node_named("osc", reg.create("saw", &HashMap::from([
        ("freq".into(), 220.0)
    ])).unwrap())
    .node_named("out", NodeType::OutputSink)
    .connect("osc", PortId(0), "out", PortId(0), Rate::Audio)
    .build()
    .unwrap();
```

## Complete Live Audio Pipeline

The full stack — kernel → DSP → I/O → MIDI — is demonstrated in the
[`microfreak_synth.rs`](../examples/microfreak_synth.rs) example in
` auxide-midi`. It builds a polyphonic ROMpler graph with `auxide-dsp`
nodes, runs it through `auxide-io`'s SteamController, and drives it from a
physical MIDI controller through `auxide-midi`'s MidiInputHandler.

---

See [Quick Start](QUICK_START.md) for the basics, or browse all
[examples](../examples/).