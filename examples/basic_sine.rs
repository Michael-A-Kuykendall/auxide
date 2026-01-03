use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;

fn main() {
    // Create a graph with a sine oscillator connected to output
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

    // Compile the plan
    let plan = Plan::compile(&graph, 64).unwrap();

    // Create runtime
    let mut runtime = Runtime::new(plan, &graph);

    // Process a block
    let mut out = vec![0.0; 64];
    runtime.process_block(&mut out);

    // Print first few samples
    for i in 0..10 {
        println!("Sample {}: {}", i, out[i]);
    }
}