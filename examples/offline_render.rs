use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::{Runtime, render_offline};

fn main() {
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

    let output = render_offline(&mut runtime, 64);
    println!("Offline render: first sample {}", output[0]);
}
