//! Simple Filter Chain Example
//!
//! Demonstrates chaining gain nodes as a basic low-pass filter approximation.
//! Shows how to build more complex DSP by composing simple nodes.

use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;

fn main() {
    // Approximate a simple low-pass filter with multiple gain stages
    // In a real filter, you'd add custom nodes with state (delays, coefficients)
    let mut graph = Graph::new();

    let input = graph.add_node(NodeType::SineOsc { freq: 1000.0 }); // High freq input
    let gain1 = graph.add_node(NodeType::Gain { gain: 0.8 });
    let gain2 = graph.add_node(NodeType::Gain { gain: 0.6 });
    let gain3 = graph.add_node(NodeType::Gain { gain: 0.4 });
    let output = graph.add_node(NodeType::OutputSink);

    // Chain: input -> gain1 -> gain2 -> gain3 -> output
    graph.add_edge(auxide::graph::Edge {
        from_node: input,
        from_port: PortId(0),
        to_node: gain1,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    graph.add_edge(auxide::graph::Edge {
        from_node: gain1,
        from_port: PortId(0),
        to_node: gain2,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    graph.add_edge(auxide::graph::Edge {
        from_node: gain2,
        from_port: PortId(0),
        to_node: gain3,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    graph.add_edge(auxide::graph::Edge {
        from_node: gain3,
        from_port: PortId(0),
        to_node: output,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph, 44100.0);

    let mut out = vec![0.0; 64];
    runtime.process_block(&mut out).unwrap();

    println!("Generated filtered audio: attenuation chain applied");
}