//! Sequenced Audio Example
//!
//! Demonstrates building a simple sequencer: play notes in sequence.
//! Uses multiple oscillators triggered in time.

use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;

fn main() {
    let sample_rate = 44100.0;
    let block_size = 1024;
    let note_duration = (sample_rate * 0.5) as usize; // 0.5s per note

    // Sequence: C4, E4, G4
    let frequencies = [261.63, 329.63, 392.00];

    let mut output_buffer = Vec::new();

    for &freq in &frequencies {
        // Build graph for each note
        let mut graph = Graph::new();
        let osc = graph.add_node(NodeType::SineOsc { freq });
        let sink = graph.add_node(NodeType::OutputSink);
        graph.add_edge(auxide::graph::Edge {
            from_node: osc,
            from_port: PortId(0),
            to_node: sink,
            to_port: PortId(0),
            rate: Rate::Audio,
        }).unwrap();

        let plan = Plan::compile(&graph, block_size).unwrap();
        let mut runtime = Runtime::new(plan, &graph, sample_rate);

        // Generate note
        let blocks_needed = note_duration / block_size;
        for _ in 0..blocks_needed {
            let mut block = vec![0.0; block_size];
            runtime.process_block(&mut block).unwrap();
            output_buffer.extend(block);
        }
    }

    println!("Generated sequenced audio: {} samples", output_buffer.len());
    // In a real app, play or save this buffer
}