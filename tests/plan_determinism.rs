use auxide::graph::{Edge, Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;

#[test]
fn plan_deterministic_compilation() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(NodeType::Dummy);
    let node2 = graph.add_node(NodeType::Mix);
    graph
        .add_edge(Edge {
            from_node: node1,
            from_port: PortId(0),
            to_node: node2,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();

    let plan1 = Plan::compile(&graph, 64).unwrap();
    let plan2 = Plan::compile(&graph, 64).unwrap();
    assert_eq!(plan1.order, plan2.order);
    assert_eq!(plan1.edges, plan2.edges);
    // Also buffer indices, but since edges are the same, assume ok.
}

#[test]
fn plan_rejects_zero_block_size() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(NodeType::Dummy);
    let node2 = graph.add_node(NodeType::OutputSink);
    graph
        .add_edge(Edge {
            from_node: node1,
            from_port: PortId(0),
            to_node: node2,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();

    assert!(Plan::compile(&graph, 0).is_err());
}
