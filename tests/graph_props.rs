use auxide::graph::{Edge, Graph, NodeId, NodeType, PortId, Rate};
use auxide::plan::Plan;
use proptest::prelude::*;

proptest! {
    #[test]
    fn graph_props_compile_or_fail_deterministically(
        seed in 0..u64::MAX,
    ) {
        // For simplicity, generate a small graph.
        // TODO: Implement full random graph generation.
        // For now, use a fixed graph to test determinism.
        let mut graph = Graph::new();
        let node1 = graph.add_node(NodeType::SineOsc { freq: 440.0 });
        let node2 = graph.add_node(NodeType::Gain { gain: 1.0 });
        graph.add_edge(Edge {
            from_node: node1,
            from_port: PortId(0),
            to_node: node2,
            to_port: PortId(0),
            rate: Rate::Audio,
        }).unwrap();

        // Compile twice
        let plan1 = Plan::compile(&graph, 64);
        let plan2 = Plan::compile(&graph, 64);
        // Either both succeed or both fail
        assert_eq!(plan1.is_ok(), plan2.is_ok());
        if let (Ok(p1), Ok(p2)) = (plan1, plan2) {
            // If succeed, identical
            assert_eq!(p1.order, p2.order);
            assert_eq!(p1.edges, p2.edges);
        }
        // No panics (implicit)
    }
}