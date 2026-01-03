use auxide::graph::{Edge, Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use proptest::prelude::*;

fn node_type_strategy() -> impl Strategy<Value = NodeType> {
    prop_oneof![
        (0.0f32..20000.0f32).prop_map(|freq| NodeType::SineOsc { freq }),
        (0.0f32..10.0f32).prop_map(|gain| NodeType::Gain { gain }),
        Just(NodeType::Mix),
        Just(NodeType::OutputSink),
        Just(NodeType::Dummy),
    ]
}

fn edge_strategy(num_nodes: usize) -> impl Strategy<Value = (usize, usize, usize, usize)> {
    (0..num_nodes).prop_flat_map(move |from| {
        (0..num_nodes)
            .prop_filter_map("no self", move |to| if to != from { Some(to) } else { None })
            .prop_map(move |to| (from, to, 0, 0))
    })
}

fn graph_strategy() -> impl Strategy<Value = Graph> {
    (2usize..=5) // num_nodes
        .prop_flat_map(|num_nodes| {
            let nodes = prop::collection::vec(node_type_strategy(), num_nodes);
            (Just(num_nodes), nodes)
        })
        .prop_flat_map(|(num_nodes, node_types)| {
            let edges = prop::collection::vec(edge_strategy(num_nodes), 0..=10);
            (Just(node_types), Just(num_nodes), edges)
        })
        .prop_map(|(node_types, _num_nodes, edge_specs)| {
            let mut graph = Graph::new();
            let node_ids: Vec<_> = node_types.into_iter().map(|nt| graph.add_node(nt)).collect();
            for (from_idx, to_idx, from_port, to_port) in edge_specs {
                let from_node = node_ids[from_idx];
                let to_node = node_ids[to_idx];
                let edge = Edge {
                    from_node,
                    from_port: PortId(from_port),
                    to_node,
                    to_port: PortId(to_port),
                    rate: Rate::Audio, // Simplify
                };
                let _ = graph.add_edge(edge); // Ignore errors for now
            }
            graph
        })
}

proptest! {
    #[test]
    fn graph_props_compile_or_fail_deterministically(graph in graph_strategy()) {
        // Compile twice with same graph
        let plan1 = Plan::compile(&graph, 64);
        let plan2 = Plan::compile(&graph, 64);
        // Either both succeed or both fail (determinism)
        prop_assert_eq!(plan1.is_ok(), plan2.is_ok());
        if let (Ok(p1), Ok(p2)) = (plan1, plan2) {
            // If succeed, plans identical
            prop_assert_eq!(p1.order, p2.order);
            prop_assert_eq!(p1.edges, p2.edges);
        }
        // No panics (implicit by running)
    }
}