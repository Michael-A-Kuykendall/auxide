use auxide::graph::{Edge, Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::{render_offline, Runtime};
use hound::{SampleFormat, WavSpec, WavWriter};

// ROMpler demo: proves the full stack renders audio end-to-end.
// Graph -> Plan -> RuntimeCore (legacy Runtime) -> render_offline -> .wav.
//
// NOTE: the single-crate `auxide` today ships oscillator/sink nodes but no
// Sampler node, so this demo drives an oscillator through the full stack to
// produce a non-silent `rompler_demo.wav`. Swap in a Sampler node when the
// IO/sampler work (auxide-io-*) lands.
fn main() {
    let mut graph = Graph::new();
    let osc = graph.add_node(NodeType::SineOsc { freq: 220.0 });
    let sink = graph.add_node(NodeType::OutputSink);
    graph
        .add_edge(Edge {
            from_node: osc,
            from_port: PortId(0),
            to_node: sink,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();

    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph, 44100.0);

    let output = render_offline(&mut runtime, 44100).unwrap();

    let spec = WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };
    let mut writer = WavWriter::create("rompler_demo.wav", spec).unwrap();
    let mut peak = 0.0f32;
    for &s in &output {
        let a = s.abs();
        if a > peak {
            peak = a;
        }
        writer.write_sample(s).unwrap();
    }
    writer.finalize().unwrap();

    println!(
        "ROMpler demo: rendered {} samples (~{:.2}s), peak {:.3}",
        output.len(),
        output.len() as f32 / 44100.0,
        peak
    );
    assert!(peak > 0.0, "ROMpler demo produced a silent file");
}
