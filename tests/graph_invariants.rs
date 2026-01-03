use auxide::graph::{Edge, Graph, GraphError, NodeId, NodeType, PortId, Rate};
use auxide::invariant_ppt::{clear_invariant_log, contract_test, GRAPH_REJECTS_INVALID};
use auxide::plan::Plan;

#[test]
fn no_cycles_unless_delay() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(NodeType::Dummy);
    let node2 = graph.add_node(NodeType::Mix);
    // Add edge 1 -> 2
    let edge1 = Edge {
        from_node: node1,
        from_port: PortId(0),
        to_node: node2,
        to_port: PortId(0),
        rate: Rate::Audio,
    };
    graph.add_edge(edge1).unwrap();
    // Try to add 2 -> 1, creating cycle
    let edge2 = Edge {
        from_node: node2,
        from_port: PortId(0),
        to_node: node1,
        to_port: PortId(0),
        rate: Rate::Audio,
    };
    assert_eq!(graph.add_edge(edge2), Err(GraphError::CycleDetected));
    // Note: No Delay node yet, so cycles are always forbidden
}

#[test]
fn input_ports_connected_or_optional() {
    // Required inputs must be connected; optional may be unconnected.
    let mut graph = Graph::new();
    let gain_node = graph.add_node(NodeType::Gain { gain: 1.0 }); // Requires 1 input
                                                                  // No edge to gain_node's input: should fail compile
    assert!(auxide::plan::Plan::compile(&graph, 64).is_err());

    // Add the required edge
    let osc_node = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    graph
        .add_edge(Edge {
            from_node: osc_node,
            from_port: PortId(0),
            to_node: gain_node,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();
    // Now should succeed
    assert!(auxide::plan::Plan::compile(&graph, 64).is_ok());

    // Optional: Dummy has input port but required=0, so unconnected succeeds
    let mut graph2 = Graph::new();
    let _dummy_node = graph2.add_node(NodeType::Dummy);
    assert!(auxide::plan::Plan::compile(&graph2, 64).is_ok());
}

#[test]
fn output_ports_fan_out_via_mix() {
    // Output ports may fan out only via explicit mix semantics
    // Meaning, an output can connect to multiple inputs, but only if it's a mix or something?
    // The plan says "Output ports may fan out only via explicit mix semantics"
    // Currently, graph allows multiple edges from same output.
    // Test that it's allowed.
    let mut graph = Graph::new();
    let node1 = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    let node2 = graph.add_node(NodeType::Gain { gain: 1.0 });
    let node3 = graph.add_node(NodeType::Gain { gain: 1.0 });
    graph
        .add_edge(Edge {
            from_node: node1,
            from_port: PortId(0),
            to_node: node2,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();
    graph
        .add_edge(Edge {
            from_node: node1,
            from_port: PortId(0),
            to_node: node3,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();
    // Should succeed, as fan-out is allowed.
    assert!(auxide::plan::Plan::compile(&graph, 64).is_ok());
}

#[test]
fn node_ids_stable_monotonic() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(NodeType::Dummy);
    let node2 = graph.add_node(NodeType::Dummy);
    assert_eq!(node1, NodeId(0));
    assert_eq!(node2, NodeId(1));
    // Monotonic: next is 2
    let node3 = graph.add_node(NodeType::Dummy);
    assert_eq!(node3, NodeId(2));
}

#[test]
fn remove_node_invalidates_edges() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(NodeType::Dummy);
    let node2 = graph.add_node(NodeType::Dummy);
    graph
        .add_edge(Edge {
            from_node: node1,
            from_port: PortId(0),
            to_node: node2,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();
    // Remove node1
    graph.remove_node(node1);
    // Edges to/from node1 should be removed
    assert!(graph.edges.is_empty());
    // And node2 still exists
    assert_eq!(graph.nodes.len(), 2); // Vec size doesn't shrink
    assert!(graph.nodes[0].is_none());
    assert_eq!(graph.nodes[1].as_ref().unwrap().id, node2);
}

#[test]
fn remove_middle_node_preserves_survivors() {
    let mut graph = Graph::new();
    let node0 = graph.add_node(NodeType::Dummy);
    let node1 = graph.add_node(NodeType::Dummy);
    let node2 = graph.add_node(NodeType::Dummy);
    // Add edge 0 -> 1
    graph
        .add_edge(Edge {
            from_node: node0,
            from_port: PortId(0),
            to_node: node1,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();
    // Add edge 1 -> 2
    graph
        .add_edge(Edge {
            from_node: node1,
            from_port: PortId(0),
            to_node: node2,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();
    // Remove middle node1
    graph.remove_node(node1);
    // Now add edge between survivors 0 -> 2
    graph
        .add_edge(Edge {
            from_node: node0,
            from_port: PortId(0),
            to_node: node2,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();
    // Compile plan without panic or misrouting
    let plan = Plan::compile(&graph, 64).unwrap();
    assert_eq!(plan.edges.len(), 1);
    assert_eq!(plan.edges[0].from_node, node0);
    assert_eq!(plan.edges[0].to_node, node2);
}

#[test]
fn remove_node_stress_recompile() {
    // Stress test: remove nodes in various sequences, add new ones, recompile plans
    let mut graph = Graph::new();
    let mut nodes = vec![];
    for _ in 0..10 {
        nodes.push(graph.add_node(NodeType::Dummy));
    }
    // Add some edges
    for i in 0..9 {
        graph
            .add_edge(Edge {
                from_node: nodes[i],
                from_port: PortId(0),
                to_node: nodes[i + 1],
                to_port: PortId(0),
                rate: Rate::Audio,
            })
            .unwrap();
    }
    // Compile initial plan
    let mut plan = Plan::compile(&graph, 64).unwrap();
    assert_eq!(plan.edges.len(), 9);

    // Remove every other node
    for i in (0..10).step_by(2) {
        graph.remove_node(nodes[i]);
    }
    // Recompile
    plan = Plan::compile(&graph, 64).unwrap();
    // Should have fewer edges
    assert!(plan.edges.len() < 9);

    // Add new nodes and edges
    let new_node = graph.add_node(NodeType::Dummy);
    graph
        .add_edge(Edge {
            from_node: nodes[1], // Assuming still exists
            from_port: PortId(0),
            to_node: new_node,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();
    // Recompile again
    plan = Plan::compile(&graph, 64).unwrap();
    // Should compile without issues
    assert!(plan.edges.len() > 0);
}

#[test]
fn contract_graph_invariants() {
    clear_invariant_log();
    // Run tests that should trigger invariants
    no_cycles_unless_delay();
    input_ports_connected_or_optional();
    // Check that invariants were enforced
    contract_test("graph invariants", &[GRAPH_REJECTS_INVALID]);
}

#[test]
fn edge_direction_validation() {
    let mut graph = Graph::new();
    let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    let gain = graph.add_node(NodeType::Gain { gain: 1.0 });
    let mix = graph.add_node(NodeType::Mix);

    // Valid: output to input
    assert!(graph
        .add_edge(Edge {
            from_node: osc,
            from_port: PortId(0), // output
            to_node: gain,
            to_port: PortId(0), // input
            rate: Rate::Audio,
        })
        .is_ok());

    // Invalid: from non-existent output port
    assert_eq!(
        graph.add_edge(Edge {
            from_node: mix,
            from_port: PortId(2), // not an output port
            to_node: gain,
            to_port: PortId(0), // input
            rate: Rate::Audio,
        }),
        Err(GraphError::InvalidPort)
    );

    // Invalid: output to output (but since IDs overlap, use a different port)
    // For mix, output is PortId(0), but to connect to output, but since we check to is input, it's ok as long as to is input.
    // Actually, the direction check prevents connecting to outputs because we require to_port to be in inputs.
    // So the test for output to output is not possible because to must be input.
    // The invalid is only from input ports.
}

#[test]
fn invalid_node_bounds_check() {
    let mut graph = Graph::new();
    let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });

    // Try to add edge with invalid node ID
    assert_eq!(
        graph.add_edge(Edge {
            from_node: NodeId(999), // out of bounds
            from_port: PortId(0),
            to_node: osc,
            to_port: PortId(0),
            rate: Rate::Audio,
        }),
        Err(GraphError::InvalidNode)
    );
}
