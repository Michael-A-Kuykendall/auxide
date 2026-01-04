// examples/proof_it_works.rs
use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::{Runtime, render_offline};

fn main() {
    // Build 440Hz sine
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

    let plan = Plan::compile(&graph, 512).unwrap();
    let mut runtime = Runtime::new(plan, &graph, 44100.0);

    // Generate 1 second of audio
    let samples = render_offline(&mut runtime, 44100).unwrap();
    
    // Save to WAV
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create("proof.wav", spec).unwrap();
    for &sample in &samples {
        writer.write_sample((sample * 32767.0) as i16).unwrap();
    }
    writer.finalize().unwrap();
    
    println!("Generated proof.wav - open it and you should hear a 440Hz tone");
}