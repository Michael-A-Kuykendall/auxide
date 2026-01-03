use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::{render_offline, Runtime};

#[test]
fn dsp_silence_propagation() {
    // Graph with Dummy (outputs 0) -> Gain -> OutputSink
    let mut graph = Graph::new();
    let dummy = graph.add_node(NodeType::Dummy);
    let gain = graph.add_node(NodeType::Gain { gain: 2.0 });
    let sink = graph.add_node(NodeType::OutputSink);
    graph
        .add_edge(auxide::graph::Edge {
            from_node: dummy,
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
    let output = render_offline(&mut runtime, 64).unwrap();
    assert!(output.iter().all(|&s| s == 0.0), "Silence should propagate");
}
