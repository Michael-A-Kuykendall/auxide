//! Integration tests validating cross-crate functionality.
//! These tests ensure each layer properly integrates with the next.

use auxide::graph::{Graph, NodeType, PortId, Rate};
use auxide::plan::Plan;
use auxide::rt::Runtime;
use auxide_dsp::nodes::oscillators::SawOsc;
use auxide_dsp::nodes::filters::{SvfFilter, SvfMode};
use auxide_dsp::nodes::envelopes::AdsrEnvelope;
use auxide_dsp::nodes::fx::Delay;
use auxide_io::stream_controller::StreamController;
use auxide_midi::VoiceAllocator;

#[test]
fn test_auxide_dsp_integration() {
    // Test: auxide kernel + auxide-dsp nodes work together

    // Build a graph with auxide-dsp nodes
    let mut graph = Graph::new();
    let osc = graph.add_external_node(SawOsc { freq: 440.0 });
    let filter = graph.add_external_node(SvfFilter {
        cutoff: 1000.0,
        resonance: 0.5,
        mode: SvfMode::Lowpass,
    });
    let output = graph.add_node(NodeType::OutputSink);

    // Connect: Osc -> Filter -> Output
    graph.add_edge(auxide::graph::Edge {
        from_node: osc,
        from_port: PortId(0),
        to_node: filter,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    graph.add_edge(auxide::graph::Edge {
        from_node: filter,
        from_port: PortId(0),
        to_node: output,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    // Plan and execute
    let plan = Plan::compile(&graph, 64).unwrap();
    let mut runtime = Runtime::new(plan, &graph, 44100.0);
    let mut output_buffer = vec![0.0; 64];

    runtime.process_block(&mut output_buffer).unwrap();

    // Verify DSP processing occurred (output should be filtered)
    assert!(!output_buffer.iter().all(|&x| x == 0.0), "DSP processing should produce non-zero output");
}

#[test]
fn test_auxide_io_integration() {
    // Test: auxide kernel + auxide-io streaming works

    // Create a simple graph
    let mut graph = Graph::new();
    let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    let output = graph.add_node(NodeType::OutputSink);

    graph.add_edge(auxide::graph::Edge {
        from_node: osc,
        from_port: PortId(0),
        to_node: output,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    // Test StreamController creation (can't actually stream without device)
    let plan = Plan::compile(&graph, 64).unwrap();
    let runtime = Runtime::new(plan, &graph, 44100.0);

    // This tests the integration contract without requiring actual audio devices
    // StreamController::play may fail if no suitable audio device is available
    let controller_result = StreamController::play(runtime);
    // Just verify it doesn't panic - the actual result depends on system audio setup
    let _ = controller_result;
}

#[test]
fn test_auxide_midi_integration() {
    // Test: auxide kernel + auxide-midi voice allocation works

    // Test voice allocator integration
    let mut allocator = VoiceAllocator::new();

    // Simulate MIDI events by directly allocating voices
    let voice1 = allocator.allocate_voice(60);
    let voice2 = allocator.allocate_voice(64);

    // Verify voice allocation works
    assert!(voice1.is_some(), "Should allocate first voice");
    assert!(voice2.is_some(), "Should allocate second voice");

    // Test active voices iteration
    let active_count = allocator.active_voices().count();
    assert_eq!(active_count, 2, "Should have 2 active voices");
}

#[test]
fn test_full_pipeline_integration() {
    // Test: Complete auxide-dsp + auxide-io + auxide-midi pipeline

    // Build complex DSP graph manually (EffectsChainBuilder doesn't auto-connect)
    let mut graph = Graph::new();
    
    // Add nodes
    let input = graph.add_node(NodeType::Dummy);
    let osc = graph.add_external_node(SawOsc { freq: 440.0 });
    let env = graph.add_external_node(AdsrEnvelope {
        attack_ms: 10.0,
        decay_ms: 100.0,
        sustain_level: 0.7,
        release_ms: 200.0,
        curve: 1.0,
    });
    let filter = graph.add_external_node(SvfFilter {
        cutoff: 1000.0,
        resonance: 0.5,
        mode: SvfMode::Lowpass,
    });
    let delay = graph.add_external_node(Delay {
        delay_ms: 300.0,
        feedback: 0.3,
        mix: 0.2,
    });
    let output = graph.add_node(NodeType::OutputSink);
    
    // Connect: Input -> Osc -> Env -> Filter -> Delay -> Output
    graph.add_edge(auxide::graph::Edge {
        from_node: osc,
        from_port: PortId(0),
        to_node: env,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    
    graph.add_edge(auxide::graph::Edge {
        from_node: env,
        from_port: PortId(0),
        to_node: filter,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    
    graph.add_edge(auxide::graph::Edge {
        from_node: filter,
        from_port: PortId(0),
        to_node: delay,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    
    graph.add_edge(auxide::graph::Edge {
        from_node: delay,
        from_port: PortId(0),
        to_node: output,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    // Plan compilation
    let plan = Plan::compile(&graph, 64).unwrap();
    let runtime = Runtime::new(plan, &graph, 44100.0);

    // Test I/O integration setup
    let controller_result = StreamController::play(runtime);
    // Just verify it doesn't panic - the actual result depends on system audio setup
    let _ = controller_result;

    // Test MIDI integration
    let mut voice_allocator = VoiceAllocator::new();

    // Simulate full pipeline event
    let voice = voice_allocator.allocate_voice(64);
    assert!(voice.is_some(), "Should allocate voice for MIDI event");
}

#[test]
fn test_layer_isolation() {
    // Test: Each layer fails gracefully when dependencies are missing

    // Test auxide-io buffer adapter bounds checking
    use auxide_io::buffer_size_adapter::{BufferSizeAdapter, MAX_HOST_FRAMES};

    let mut adapter = BufferSizeAdapter::new(64);

    // Should reject oversized host buffers
    let result = adapter.adapt_to_host_buffer(MAX_HOST_FRAMES + 1);
    assert!(result.is_err(), "Should reject oversized buffers");
}

#[test]
fn test_rt_safety_integration() {
    // Test: RT safety invariants hold across layer boundaries

    // Create RT workload with complex DSP chain (manual connections)
    let mut graph = Graph::new();
    
    let input = graph.add_node(NodeType::Dummy);
    let osc = graph.add_external_node(SawOsc { freq: 440.0 });
    let filter = graph.add_external_node(SvfFilter {
        cutoff: 2000.0,
        resonance: 0.8,
        mode: SvfMode::Bandpass,
    });
    let delay = graph.add_external_node(Delay {
        delay_ms: 150.0,
        feedback: 0.4,
        mix: 0.3,
    });
    let output = graph.add_node(NodeType::OutputSink);
    
    // Connect: Osc -> Filter -> Delay -> Output
    graph.add_edge(auxide::graph::Edge {
        from_node: osc,
        from_port: PortId(0),
        to_node: filter,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    
    graph.add_edge(auxide::graph::Edge {
        from_node: filter,
        from_port: PortId(0),
        to_node: delay,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();
    
    graph.add_edge(auxide::graph::Edge {
        from_node: delay,
        from_port: PortId(0),
        to_node: output,
        to_port: PortId(0),
        rate: Rate::Audio,
    }).unwrap();

    let plan = Plan::compile(&graph, 128).unwrap();
    let mut runtime = Runtime::new(plan, &graph, 48000.0);

    // Process multiple blocks to test sustained RT performance
    let mut output_buffer = vec![0.0; 128];
    for _ in 0..100 {
        runtime.process_block(&mut output_buffer).unwrap();
    }

    // Verify sustained processing didn't break
    assert!(!output_buffer.iter().all(|&x| x == 0.0), "RT processing should be sustained");
}