use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;

fn main() {
    // Mix two oscillators
    let mut graph = Graph::new();
    let osc1 = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    let osc2 = graph.add_node(NodeType::SineOsc { freq: 880.0 });
    let mix = graph.add_node(NodeType::Mix);
    let sink = graph.add_node(NodeType::OutputSink);

    graph
        .add_edge(auxide::graph::Edge {
            from_node: osc1,
            from_port: PortId(0),
            to_node: mix,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();
    graph
        .add_edge(auxide::graph::Edge {
            from_node: osc2,
            from_port: PortId(0),
            to_node: mix,
            to_port: PortId(1),
            rate: Rate::Audio,
        })
        .unwrap();
    graph
        .add_edge(auxide::graph::Edge {
            from_node: mix,
            from_port: PortId(0),
            to_node: sink,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();

    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph, 44100.0);

    let mut out = vec![0.0; 64];
    runtime.process_block(&mut out).unwrap();

    println!("Mixer output: {}", out[0]);
}
