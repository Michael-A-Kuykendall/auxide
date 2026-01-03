use auxide::graph::{Edge, Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;

#[test]
fn plan_topology_preservation() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(NodeType::Dummy);
    let node2 = graph.add_node(NodeType::Dummy);
    let node3 = graph.add_node(NodeType::Mix);
    graph.add_edge(Edge {
        from_node: node1,
        from_port: PortId(0),
        to_node: node3,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    graph.add_edge(Edge {
        from_node: node2,
        from_port: PortId(0),
        to_node: node3,
        to_port: PortId(1),
        rate: Rate::Audio,
    }).unwrap();

    let plan = Plan::compile(&graph, 64).unwrap();
    // Every graph edge corresponds to exactly one plan edge
    assert_eq!(plan.edges.len(), graph.edges.len());
    // No orphaned edges: all plan edges have corresponding graph edges
    for plan_edge in &plan.edges {
        let corresponding = graph.edges.iter().find(|e| e.from_node == plan_edge.from_node && e.to_node == plan_edge.to_node && e.from_port == plan_edge.from_port && e.to_port == plan_edge.to_port);
        assert!(corresponding.is_some());
    }
    // No duplicated routes: assume no duplicates in plan.edges
    let mut seen = std::collections::HashSet::new();
    for edge in &plan.edges {
        let key = (edge.from_node, edge.from_port, edge.to_node, edge.to_port);
        assert!(!seen.contains(&key), "Duplicated route");
        seen.insert(key);
    }
}