use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;

fn main() {
    // Create a chain: Osc -> Gain -> Gain -> Output
    let mut graph = Graph::new();
    let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    let gain1 = graph.add_node(NodeType::Gain { gain: 0.5 });
    let gain2 = graph.add_node(NodeType::Gain { gain: 0.5 });
    let sink = graph.add_node(NodeType::OutputSink);

    graph.add_edge(auxide::graph::Edge {
        from_node: osc,
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
        to_node: sink,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph);

    let mut out = vec![0.0; 64];
    runtime.process_block(&mut out);

    println!("Gain chain output: {}", out[0]);
}