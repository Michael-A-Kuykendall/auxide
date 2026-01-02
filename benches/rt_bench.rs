use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn bench_process_block(c: &mut Criterion) {
    let mut graph = Graph::new();
    let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    let gain = graph.add_node(NodeType::Gain { gain: 0.5 });
    let out_node = graph.add_node(NodeType::OutputSink);
    graph.add_edge(auxide::graph::Edge { from_node: osc, from_port: PortId(0), to_node: gain, to_port: PortId(0), rate: Rate::Audio }).unwrap();
    graph.add_edge(auxide::graph::Edge { from_node: gain, from_port: PortId(0), to_node: out_node, to_port: PortId(0), rate: Rate::Audio }).unwrap();
    let plan = Plan::compile(&graph, 1024).unwrap();
    let mut runtime = Runtime::new(plan, &graph);
    let mut out = vec![0.0; 1024];

    c.bench_function("process_block_1024", |b| {
        b.iter(|| {
            runtime.process_block(black_box(&mut out));
            black_box(&out);
        })
    });
}

fn bench_multi_node(c: &mut Criterion) {
    let mut graph = Graph::new();
    let osc1 = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    let gain = graph.add_node(NodeType::Gain { gain: 0.5 });
    let osc2 = graph.add_node(NodeType::SineOsc { freq: 880.0 });
    let mix = graph.add_node(NodeType::Mix);
    let out_node = graph.add_node(NodeType::OutputSink);
    graph.add_edge(auxide::graph::Edge { from_node: osc1, from_port: PortId(0), to_node: gain, to_port: PortId(0), rate: Rate::Audio }).unwrap();
    graph.add_edge(auxide::graph::Edge { from_node: gain, from_port: PortId(0), to_node: mix, to_port: PortId(0), rate: Rate::Audio }).unwrap();
    graph.add_edge(auxide::graph::Edge { from_node: osc2, from_port: PortId(0), to_node: mix, to_port: PortId(1), rate: Rate::Audio }).unwrap();
    graph.add_edge(auxide::graph::Edge { from_node: mix, from_port: PortId(0), to_node: out_node, to_port: PortId(0), rate: Rate::Audio }).unwrap();
    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph);
    let mut out = vec![0.0; 64];
    c.bench_function("multi_node_process", |b| {
        b.iter(|| {
            runtime.process_block(black_box(&mut out));
            black_box(&out);
        });
    });
}

criterion_group!(benches, bench_process_block, bench_multi_node);
criterion_main!(benches);
