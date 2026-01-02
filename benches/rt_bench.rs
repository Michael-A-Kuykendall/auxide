use auxide::graph::{Graph, NodeType, Port, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::collections::HashMap;

fn bench_process_block(c: &mut Criterion) {
    let mut graph = Graph::new();
    let node1 = graph.add_node(NodeType::SineOsc {
        freq: 440.0,
        phase: 0.0,
    });
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

fn bench_multi_node(c: &mut Criterion) {
    let mut graph = Graph::new();
    let osc1 = graph.add_node(NodeType::SineOsc { freq: 440.0, phase: 0.0 });
    let gain = graph.add_node(NodeType::Gain { gain: 0.5 });
    let osc2 = graph.add_node(NodeType::SineOsc { freq: 880.0, phase: 0.0 });
    let mix = graph.add_node(NodeType::Mix);
    graph.add_edge(auxide::graph::Edge { from_node: osc1, from_port: PortId(0), to_node: gain, to_port: PortId(0), rate: Rate::Audio }).unwrap();
    graph.add_edge(auxide::graph::Edge { from_node: gain, from_port: PortId(0), to_node: mix, to_port: PortId(0), rate: Rate::Audio }).unwrap();
    graph.add_edge(auxide::graph::Edge { from_node: osc2, from_port: PortId(0), to_node: mix, to_port: PortId(1), rate: Rate::Audio }).unwrap();
    let plan = Plan::compile(&graph).unwrap();
    let mut runtime = Runtime::new(plan, &graph);
    let mut inputs = HashMap::new();
    let mut outputs = HashMap::new();
    outputs.insert(mix, HashMap::from([(PortId(0), vec![0.0; 64])]));
    c.bench_function("multi_node_process", |b| {
        b.iter(|| {
            runtime.process_block(&inputs, &mut outputs, 64);
            black_box(&outputs);
        });
    });
}

criterion_group!(benches, bench_process_block, bench_multi_node);
criterion_main!(benches);
