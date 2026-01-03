use auxide::graph::{Edge, Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;

#[test]
fn plan_isolation() {
    // Compile graph A → plan A
    let mut graph_a = Graph::new();
    let node1 = graph_a.add_node(NodeType::Dummy);
    let node2 = graph_a.add_node(NodeType::OutputSink);
    graph_a
        .add_edge(Edge {
            from_node: node1,
            from_port: PortId(0),
            to_node: node2,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();
    let _plan_a = Plan::compile(&graph_a, 64).unwrap();

    // Mutate graph → compile plan B
    let node3 = graph_a.add_node(NodeType::Gain { gain: 2.0 });
    graph_a
        .add_edge(Edge {
            from_node: node1,
            from_port: PortId(0),
            to_node: node3,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();
    let _plan_b = Plan::compile(&graph_a, 64).unwrap();

    // Assert plan A remains unchanged
    let _plan_a_again = Plan::compile(&graph_a, 64).unwrap(); // Recompile original? Wait, graph_a is mutated.
    // Wait, the test says "Mutate graph → compile plan B"
    // But to check plan A unchanged, I need to compile the original graph again.
    // So, I need to keep the original graph.

    // Let's adjust.
    let mut graph_original = Graph::new();
    let node1 = graph_original.add_node(NodeType::Dummy);
    let node2 = graph_original.add_node(NodeType::OutputSink);
    graph_original
        .add_edge(Edge {
            from_node: node1,
            from_port: PortId(0),
            to_node: node2,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();
    let plan_a = Plan::compile(&graph_original, 64).unwrap();

    // Mutate a copy
    let mut graph_mutated = graph_original.clone();
    let node3 = graph_mutated.add_node(NodeType::Gain { gain: 2.0 });
    graph_mutated
        .add_edge(Edge {
            from_node: node1,
            from_port: PortId(0),
            to_node: node3,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();
    let _plan_b = Plan::compile(&graph_mutated, 64).unwrap();

    // Assert plan A remains unchanged (recompile original)
    let plan_a_check = Plan::compile(&graph_original, 64).unwrap();
    assert_eq!(plan_a.order, plan_a_check.order);
    assert_eq!(plan_a.edges, plan_a_check.edges);

    // plan A still executes correctly
    let mut runtime_a = Runtime::new(plan_a, &graph_original, 44100.0);
    let mut out = vec![0.0; 64];
    runtime_a.process_block(&mut out).unwrap();
    // Should be 0, as Dummy outputs 0
    assert_eq!(out, vec![0.0; 64]);
}
