use auxide::graph::{Graph, NodeType};
use auxide::plan::Plan;
use auxide::rt::Runtime;
use proptest::prelude::*;

proptest! {
    #[test]
    fn rt_no_panic_fuzz(_dummy in 0..10) {
        // For now, simple fuzz: run on a few different graphs
        let graphs = vec![
            {
                let mut g = Graph::new();
                g.add_node(NodeType::Dummy);
                g
            },
            {
                let mut g = Graph::new();
                let n1 = g.add_node(NodeType::SineOsc { freq: 440.0 });
                let n2 = g.add_node(NodeType::OutputSink);
                g.add_edge(auxide::graph::Edge {
                    from_node: n1,
                    from_port: auxide::graph::PortId(0),
                    to_node: n2,
                    to_port: auxide::graph::PortId(0),
                    rate: auxide::graph::Rate::Audio,
                }).unwrap();
                g
            },
        ];

        for graph in graphs {
            if let Ok(plan) = Plan::compile(&graph, 64) {
                let mut runtime = Runtime::new(plan, &graph, 44100.0);
                let mut out = vec![0.0; 64];
                // This should not panic
                runtime.process_block(&mut out).unwrap();
            }
        }
    }
}