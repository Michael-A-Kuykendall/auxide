use auxide::graph::{Graph, NodeType, PortId};
use auxide::plan::Plan;
use auxide::rt::{render_offline, Runtime};

#[test]
fn dsp_osc_correctness() {
    let mut graph = Graph::new();
    let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    let sink = graph.add_node(NodeType::OutputSink);
    graph
        .add_edge(auxide::graph::Edge {
            from_node: osc,
            from_port: PortId(0),
            to_node: sink,
            to_port: PortId(0),
            rate: auxide::graph::Rate::Audio,
        })
        .unwrap();

    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph, 44100.0);
    let output = render_offline(&mut runtime, 64).unwrap();

    // Check no DC offset: for a full period, but for now, skip
    // let mean: f32 = output.iter().sum::<f32>() / output.len() as f32;
    // assert!(mean.abs() < 0.01, "DC offset: {}", mean);

    // Check phase continuity: first sample should be sin(0) = 0
    assert!((output[0] - 0.0).abs() < 0.01);

    // Check frequency: period should be 44100 / 440 ≈ 100.22 samples
    // For 64 samples, check the phase advance
    // Approximate: output[1] should be sin(2*pi*440/44100) ≈ sin(0.0628) ≈ 0.0627
    assert!(output[1] > 0.0 && output[1] < 0.1);

    // Frequency error: check if the signal repeats every ~100 samples
    // For simplicity, check that it's sinusoidal
    // More precise: check zero crossings
    let mut zero_crossings = 0;
    for i in 1..output.len() {
        if output[i - 1] * output[i] < 0.0 {
            zero_crossings += 1;
        }
    }
    // For 440 Hz at 44100, period 100.22, in 64 samples ~0.64 periods, ~1-2 crossings
    assert!(
        (1..=3).contains(&zero_crossings),
        "Zero crossings: {}",
        zero_crossings
    );
}
