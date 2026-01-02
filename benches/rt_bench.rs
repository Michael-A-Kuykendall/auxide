use auxide::graph::{Graph, NodeType, Port, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::collections::HashMap;

fn bench_process_block(c: &mut Criterion) {
    let mut graph = Graph::new();
    let node1 = graph.add_node(
        vec![Port {
            id: PortId(0),
            rate: Rate::Audio,
        }],
        NodeType::SineOsc {
            freq: 440.0,
            phase: 0.0,
        },
    );
    let plan = Plan::compile(&graph).unwrap();
    let mut runtime = Runtime::new(plan, &graph);
    let mut inputs = HashMap::new();
    inputs.insert(node1, HashMap::from([(PortId(0), vec![1.0; 1024])]));
    let mut outputs = HashMap::new();
    outputs.insert(node1, HashMap::from([(PortId(0), vec![0.0; 1024])]));

    c.bench_function("process_block_1024", |b| {
        b.iter(|| {
            runtime.process_block(black_box(&inputs), black_box(&mut outputs), 1024);
            let sum: f32 = outputs[&node1][&PortId(0)].iter().sum();
            black_box(sum);
        })
    });
}

criterion_group!(benches, bench_process_block);
criterion_main!(benches);
