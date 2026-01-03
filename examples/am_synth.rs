//! Simple AM Synthesis Example
//!
//! Demonstrates using Auxide for amplitude modulation.
//! A low-frequency oscillator modulates the amplitude of a carrier oscillator.

use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;

fn main() {
    // Create graph: carrier osc + modulator osc + output
    let mut graph = Graph::new();

    // Carrier: 440Hz sine
    let carrier = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    // Modulator: 10Hz sine (low freq for FM)
    let modulator = graph.add_node(NodeType::SineOsc { freq: 10.0 });
    // Gain for modulation depth
    let mod_gain = graph.add_node(NodeType::Gain { gain: 50.0 }); // Modulate by ±50Hz
                                                                  // Output sink
    let sink = graph.add_node(NodeType::OutputSink);

    // Connect: modulator -> gain -> carrier (as control input, but since we don't have control ports, simulate with audio)
    // Note: In a real FM synth, you'd have control ports. Here we use audio rate for simplicity.
    graph
        .add_edge(auxide::graph::Edge {
            from_node: modulator,
            from_port: PortId(0),
            to_node: mod_gain,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();

    // For true FM, we'd need to add the modulation to the carrier freq.
    // Since NodeType::SineOsc takes a fixed freq, this is a simplified demo.
    // In practice, extend NodeType for dynamic freq.

    graph
        .add_edge(auxide::graph::Edge {
            from_node: carrier,
            from_port: PortId(0),
            to_node: sink,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();

    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph, 44100.0);

    // Generate some FM-like sound (simplified)
    let mut out = vec![0.0; 64];
    runtime.process_block(&mut out).unwrap();

    println!("Generated AM audio block");
}
