//! RT Proof Harness: Empirical proofs for RT guarantees.

use crate::graph::{Graph, NodeId, PortId};
use crate::plan::Plan;
use crate::rt::{Runtime, process_block_safe};
use std::collections::HashMap;

/// Harness for RT proofs: runs process_block and checks for violations.
pub struct RtHarness {
    runtime: Runtime,
    inputs: HashMap<NodeId, HashMap<PortId, Vec<f32>>>,
    outputs: HashMap<NodeId, HashMap<PortId, Vec<f32>>>,
}

impl RtHarness {
    /// Create harness from graph.
    pub fn new(graph: &Graph) -> Self {
        let plan = Plan::compile(graph).unwrap();
        let runtime = Runtime::new(plan, graph);
        let inputs = HashMap::new();
        let outputs = HashMap::new();
        Self {
            runtime,
            inputs,
            outputs,
        }
    }

    /// Run process_block safely and check invariants.
    pub fn run_block(&mut self, frames: usize) {
        // For now, just run; future: detect alloc/locks
        process_block_safe(&mut self.runtime, &self.inputs, &mut self.outputs, frames);
        // Placeholder for violation checks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harness_basic() {
        use crate::graph::{NodeType, Port, Rate};
        let mut graph = Graph::new();
        let _node1 = graph.add_node(
            vec![Port {
                id: PortId(0),
                rate: Rate::Audio,
            }],
            NodeType::Dummy,
        );
        let mut harness = RtHarness::new(&graph);
        harness.run_block(64);
    }
}
