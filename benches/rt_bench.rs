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
    let mut runtime = Runtime::new(plan, &graph, 44100.0);
    let mut out = vec![0.0; 1024];

    c.bench_function("process_block_1024", |b| {
        b.iter(|| {
            runtime.process_block(black_box(&mut out)).unwrap();
            black_box(&out);
        })
    });
}

fn bench_timing_stability(c: &mut Criterion) {
    // Worst-case graph: chain of gains
    let mut graph = Graph::new();
    let mut prev = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    for _ in 0..10 {
        let next = graph.add_node(NodeType::Gain { gain: 1.0 });
        graph.add_edge(auxide::graph::Edge {
            from_node: prev,
            from_port: PortId(0),
            to_node: next,
            to_port: PortId(0),
            rate: Rate::Audio,
        }).unwrap();
        prev = next;
    }
    let sink = graph.add_node(NodeType::OutputSink);
    graph.add_edge(auxide::graph::Edge {
        from_node: prev,
        from_port: PortId(0),
        to_node: sink,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph, 44100.0);
    let mut out = vec![0.0; 64];

    c.bench_function("rt_timing_stability", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                runtime.process_block(black_box(&mut out)).unwrap();
            }
            black_box(&out);
        })
    });
}

criterion_group!(benches, bench_process_block, bench_timing_stability);
criterion_main!(benches);
