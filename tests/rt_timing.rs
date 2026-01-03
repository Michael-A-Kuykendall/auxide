use auxide::graph::{Graph, NodeType};
use auxide::plan::Plan;
use auxide::rt::Runtime;
use std::time::Instant;

#[test]
fn rt_timing_stability() {
    // Worst-case graph: chain of gains
    let mut graph = Graph::new();
    let mut prev = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    for _ in 0..10 {
        let next = graph.add_node(NodeType::Gain { gain: 1.0 });
        graph.add_edge(auxide::graph::Edge {
            from_node: prev,
            from_port: auxide::graph::PortId(0),
            to_node: next,
            to_port: auxide::graph::PortId(0),
            rate: auxide::graph::Rate::Audio,
        }).unwrap();
        prev = next;
    }
    let sink = graph.add_node(NodeType::OutputSink);
    graph.add_edge(auxide::graph::Edge {
        from_node: prev,
        from_port: auxide::graph::PortId(0),
        to_node: sink,
        to_port: auxide::graph::PortId(0),
        rate: auxide::graph::Rate::Audio,
    }).unwrap();

    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph);

    let mut out = vec![0.0; 64];
    let start = Instant::now();
    for _ in 0..1000 {
        runtime.process_block(&mut out);
    }
    let duration = start.elapsed();
    // Assert bounded: less than 1 second for 1000 blocks
    assert!(duration.as_millis() < 1000, "Execution took too long: {:?}", duration);
}