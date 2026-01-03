use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::{Runtime, render_offline};

#[test]
fn offline_render_determinism() {
    let mut graph = Graph::new();
    let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    let sink = graph.add_node(NodeType::OutputSink);
    graph
        .add_edge(auxide::graph::Edge {
            from_node: osc,
            from_port: PortId(0),
            to_node: sink,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();

    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime1 = Runtime::new(plan.clone(), &graph, 44100.0);
    let mut runtime2 = Runtime::new(plan, &graph, 44100.0);

    let output1 = render_offline(&mut runtime1, 64);
    let output2 = render_offline(&mut runtime2, 64);

    assert_eq!(output1, output2, "Offline renders should be identical");
}

#[test]
fn offline_render_partial_block() {
    let mut graph = Graph::new();
    let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    let sink = graph.add_node(NodeType::OutputSink);
    graph
        .add_edge(auxide::graph::Edge {
            from_node: osc,
            from_port: PortId(0),
            to_node: sink,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();

    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph, 44100.0);

    // Test with frames not multiple of block_size
    let frames = 65; // 64 + 1
    let output = render_offline(&mut runtime, frames);
    assert_eq!(
        output.len(),
        frames,
        "Output should have exactly requested frames"
    );
    // Should not panic and produce some output
    assert!(
        output.iter().any(|&x| x != 0.0),
        "Should produce non-zero output"
    );
}
