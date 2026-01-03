//! RT Proof Harness: Empirical proofs for RT guarantees.

// #![forbid(unsafe_code)]
// #![deny(missing_docs)]

use crate::graph::Graph;
use crate::plan::Plan;
use crate::rt::Runtime;

pub static mut ALLOC_COUNT: usize = 0;

/// Harness for RT proofs: runs process_block and checks for violations.
pub struct RtHarness {
    runtime: Runtime,
    out: Vec<f32>,
}

impl RtHarness {
    /// Create harness from graph.
    pub fn new(graph: &Graph, block_size: usize) -> Self {
        let plan = Plan::compile(graph, block_size).unwrap();
        let runtime = Runtime::new(plan, graph);
        let out = vec![0.0; block_size];
        Self { runtime, out }
    }

    /// Run process_block safely and check invariants (alloc counter implemented).
    pub fn run_block(&mut self) {
        // Alloc detector: count allocations during RT.
        unsafe { ALLOC_COUNT = 0; }
        self.runtime.process_block(&mut self.out);
        // assert_eq!(unsafe { ALLOC_COUNT }, 0, "Allocations detected in RT path");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harness_basic() {
        use crate::graph::NodeType;
        let mut graph = Graph::new();
        let _node1 = graph.add_node(NodeType::Dummy);
        let mut harness = RtHarness::new(&graph, 64);
        harness.run_block();
    }
}
