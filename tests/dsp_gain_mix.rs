use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::{Runtime, render_offline};

#[test]
fn dsp_gain_mix_algebra() {
    // Test Gain(0) -> silence
    let mut graph = Graph::new();
    let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    let gain = graph.add_node(NodeType::Gain { gain: 0.0 });
    let sink = graph.add_node(NodeType::OutputSink);
    graph
        .add_edge(auxide::graph::Edge {
            from_node: osc,
            from_port: PortId(0),
            to_node: gain,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();
    graph
        .add_edge(auxide::graph::Edge {
            from_node: gain,
            from_port: PortId(0),
            to_node: sink,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();

    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph, 44100.0);
    let output = render_offline(&mut runtime, 64);
    assert!(
        output.iter().all(|&s| s.abs() < 0.01),
        "Gain(0) should produce silence"
    );

    // Test Gain(1) -> passthrough
    let mut graph2 = Graph::new();
    let osc2 = graph2.add_node(NodeType::SineOsc { freq: 440.0 });
    let gain2 = graph2.add_node(NodeType::Gain { gain: 1.0 });
    let sink2 = graph2.add_node(NodeType::OutputSink);
    graph2
        .add_edge(auxide::graph::Edge {
            from_node: osc2,
            from_port: PortId(0),
            to_node: gain2,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();
    graph2
        .add_edge(auxide::graph::Edge {
            from_node: gain2,
            from_port: PortId(0),
            to_node: sink2,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();

    let plan2 = Plan::compile(&graph2, 64).unwrap();
    let mut runtime2 = Runtime::new(plan2, &graph2, 44100.0);
    let output2 = render_offline(&mut runtime2, 64);

    // Compare to direct osc
    let mut graph3 = Graph::new();
    let osc3 = graph3.add_node(NodeType::SineOsc { freq: 440.0 });
    let sink3 = graph3.add_node(NodeType::OutputSink);
    graph3
        .add_edge(auxide::graph::Edge {
            from_node: osc3,
            from_port: PortId(0),
            to_node: sink3,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();

    let plan3 = Plan::compile(&graph3, 64).unwrap();
    let mut runtime3 = Runtime::new(plan3, &graph3, 44100.0);
    let output3 = render_offline(&mut runtime3, 64);

    for (a, b) in output2.iter().zip(output3.iter()) {
        assert!((a - b).abs() < 0.01, "Gain(1) should passthrough");
    }

    // Test Mix(N) == sum(inputs)
    let mut graph4 = Graph::new();
    let osc4a = graph4.add_node(NodeType::SineOsc { freq: 440.0 });
    let osc4b = graph4.add_node(NodeType::SineOsc { freq: 440.0 });
    let mix = graph4.add_node(NodeType::Mix);
    let sink4 = graph4.add_node(NodeType::OutputSink);
    graph4
        .add_edge(auxide::graph::Edge {
            from_node: osc4a,
            from_port: PortId(0),
            to_node: mix,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();
    graph4
        .add_edge(auxide::graph::Edge {
            from_node: osc4b,
            from_port: PortId(0),
            to_node: mix,
            to_port: PortId(1),
            rate: Rate::Audio,
        })
        .unwrap();
    graph4
        .add_edge(auxide::graph::Edge {
            from_node: mix,
            from_port: PortId(0),
            to_node: sink4,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();

    let plan4 = Plan::compile(&graph4, 64).unwrap();
    let mut runtime4 = Runtime::new(plan4, &graph4, 44100.0);
    let output4 = render_offline(&mut runtime4, 64);

    // Sum of two oscs
    for i in 0..64 {
        assert!(
            (output4[i] - 2.0 * output3[i]).abs() < 0.01,
            "Mix should sum inputs"
        );
    }
}
