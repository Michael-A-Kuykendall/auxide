use auxide::graph::{Graph, NodeType};
use auxide::plan::Plan;
use auxide::rt::Runtime;
use auxide::harness::ALLOC_COUNT;

#[test]
fn rt_alloc_invariant() {
    let mut graph = Graph::new();
    let node1 = graph.add_node(NodeType::Dummy);
    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph);

    // Reset counter
    unsafe { ALLOC_COUNT = 0; }
    let initial = unsafe { ALLOC_COUNT };

    // Run 10,000 times
    let mut out = vec![0.0; 64];
    for _ in 0..10_000 {
        runtime.process_block(&mut out);
    }

    let final_count = unsafe { ALLOC_COUNT };
    assert_eq!(initial, final_count, "Allocations detected in RT path");
}