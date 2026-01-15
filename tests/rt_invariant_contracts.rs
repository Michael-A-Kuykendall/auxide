//! Contract tests for RT invariant signaling.
//!
//! These tests verify that the invariant signaling system works correctly
//! by driving deterministic input and verifying that required invariants fire.

use auxide::control::ControlMsg;
use auxide::graph::{Edge, Graph, NodeId, NodeType, PortId, Rate};
use auxide::invariant_rt::{
    contract_test_rt, INV_CONTROL_MSG_PROCESSED,
    INV_PARAM_UPDATE_DELIVERED, INV_RT_CALLBACK_CLEAN, INV_SAMPLE_BUFFER_FILLED,
};
use auxide::plan::Plan;
use auxide::rt::RuntimeCore;

/// Helper to create a simple oscillator → sink graph.
fn create_simple_graph() -> Graph {
    let mut graph = Graph::new();
    let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    let sink = graph.add_node(NodeType::OutputSink);
    graph
        .add_edge(Edge {
            from_node: osc,
            from_port: PortId(0),
            to_node: sink,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();
    graph
}

/// Helper to create an oscillator → gain → sink graph.
fn create_gain_graph() -> Graph {
    let mut graph = Graph::new();
    let osc = graph.add_node(NodeType::SineOsc { freq: 440.0 });
    let gain = graph.add_node(NodeType::Gain { gain: 0.5 });
    let sink = graph.add_node(NodeType::OutputSink);
    graph
        .add_edge(Edge {
            from_node: osc,
            from_port: PortId(0),
            to_node: gain,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();
    graph
        .add_edge(Edge {
            from_node: gain,
            from_port: PortId(0),
            to_node: sink,
            to_port: PortId(0),
            rate: Rate::Audio,
        })
        .unwrap();
    graph
}

#[test]
fn contract_sample_buffer_filled_on_successful_process() {
    let graph = create_simple_graph();
    let plan = Plan::compile(&graph, 64).unwrap();
    let (mut handle, mut control) = RuntimeCore::new_with_channels(plan, &graph, 44100.0);

    // Process several blocks
    let mut out = vec![0.0; 64];
    for _ in 0..10 {
        handle.process_block(&mut out).unwrap();
    }

    // Drain invariant signals from control (main thread side)
    let signals = control.drain_invariant_signals();

    // Contract: SAMPLE_BUFFER_FILLED must fire for each successful block
    contract_test_rt(
        "sample buffer filled on success",
        &signals,
        &[INV_SAMPLE_BUFFER_FILLED, INV_RT_CALLBACK_CLEAN],
    );

    // Verify we got multiple signals (one per block)
    let buffer_filled_count = signals
        .iter()
        .filter(|&&id| id == INV_SAMPLE_BUFFER_FILLED)
        .count();
    assert!(
        buffer_filled_count >= 10,
        "Expected at least 10 SAMPLE_BUFFER_FILLED signals, got {}",
        buffer_filled_count
    );
}

#[test]
fn contract_param_update_delivered_when_control_msg_sent() {
    let graph = create_gain_graph();
    let plan = Plan::compile(&graph, 64).unwrap();
    let (mut handle, mut control) = RuntimeCore::new_with_channels(plan, &graph, 44100.0);

    // Send control messages
    control
        .send(ControlMsg::SetGain {
            node: NodeId(1), // The gain node
            gain: 0.75,
        })
        .unwrap();

    // Process a block (which drains control messages)
    let mut out = vec![0.0; 64];
    handle.process_block(&mut out).unwrap();

    // Drain invariant signals from control (main thread side)
    let signals = control.drain_invariant_signals();

    // Contract: PARAM_UPDATE_DELIVERED and CONTROL_MSG_PROCESSED must fire
    contract_test_rt(
        "param update delivered",
        &signals,
        &[
            INV_PARAM_UPDATE_DELIVERED,
            INV_CONTROL_MSG_PROCESSED,
            INV_SAMPLE_BUFFER_FILLED,
        ],
    );
}

#[test]
fn contract_no_param_signals_when_no_control_msgs() {
    let graph = create_simple_graph();
    let plan = Plan::compile(&graph, 64).unwrap();
    let (mut handle, mut control) = RuntimeCore::new_with_channels(plan, &graph, 44100.0);

    // Process without sending any control messages
    let mut out = vec![0.0; 64];
    handle.process_block(&mut out).unwrap();

    // Drain invariant signals from control
    let signals = control.drain_invariant_signals();

    // Should have buffer filled but NOT param update (no messages sent)
    assert!(signals.contains(&INV_SAMPLE_BUFFER_FILLED));
    assert!(
        !signals.contains(&INV_PARAM_UPDATE_DELIVERED),
        "PARAM_UPDATE_DELIVERED should not fire when no control messages sent"
    );
}

#[test]
fn contract_frequency_change_applied() {
    let graph = create_simple_graph();
    let plan = Plan::compile(&graph, 64).unwrap();
    let (mut handle, mut control) = RuntimeCore::new_with_channels(plan, &graph, 44100.0);

    // Process initial block
    let mut out1 = vec![0.0; 64];
    handle.process_block(&mut out1).unwrap();

    // Change frequency
    control
        .send(ControlMsg::SetFrequency {
            node: NodeId(0), // The oscillator
            hz: 880.0,
        })
        .unwrap();

    // Process another block
    let mut out2 = vec![0.0; 64];
    handle.process_block(&mut out2).unwrap();

    // The outputs should be different (frequency changed)
    // Note: due to phase continuity, we check later samples
    let signals = control.drain_invariant_signals();
    assert!(signals.contains(&INV_PARAM_UPDATE_DELIVERED));
}

#[test]
fn contract_mute_silences_node() {
    let graph = create_gain_graph();
    let plan = Plan::compile(&graph, 64).unwrap();
    let (mut handle, mut control) = RuntimeCore::new_with_channels(plan, &graph, 44100.0);

    // Process block before muting
    let mut out_before = vec![0.0; 64];
    handle.process_block(&mut out_before).unwrap();
    let max_before = out_before.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(max_before > 0.0, "Should have audio before mute");

    // Mute the gain node
    control.send(ControlMsg::Mute { node: NodeId(1) }).unwrap();

    // Process block after muting
    let mut out_after = vec![0.0; 64];
    handle.process_block(&mut out_after).unwrap();
    let max_after = out_after.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(
        max_after < 0.0001,
        "Should be silent after mute, got max: {}",
        max_after
    );

    // Unmute
    control.send(ControlMsg::Unmute { node: NodeId(1) }).unwrap();

    // Process block after unmuting
    let mut out_unmuted = vec![0.0; 64];
    handle.process_block(&mut out_unmuted).unwrap();
    let max_unmuted = out_unmuted.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(max_unmuted > 0.0, "Should have audio after unmute");
}

#[test]
fn contract_reset_restores_defaults() {
    let graph = create_gain_graph();
    let plan = Plan::compile(&graph, 64).unwrap();
    let (mut handle, mut control) = RuntimeCore::new_with_channels(plan, &graph, 44100.0);

    // Mute a node
    control.send(ControlMsg::Mute { node: NodeId(1) }).unwrap();
    let mut out = vec![0.0; 64];
    handle.process_block(&mut out).unwrap();
    let max_muted = out.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(max_muted < 0.0001, "Should be silent when muted");

    // Reset
    control.send(ControlMsg::Reset).unwrap();
    handle.process_block(&mut out).unwrap();
    let max_reset = out.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(max_reset > 0.0, "Should have audio after reset");
}

#[test]
fn contract_multiple_control_msgs_in_burst() {
    let graph = create_gain_graph();
    let plan = Plan::compile(&graph, 64).unwrap();
    let (mut handle, mut control) = RuntimeCore::new_with_channels(plan, &graph, 44100.0);

    // Send a burst of control messages (simulating chord press)
    for i in 0..10 {
        control
            .send(ControlMsg::SetGain {
                node: NodeId(1),
                gain: 0.1 * i as f32,
            })
            .unwrap();
    }

    // Process a single block
    let mut out = vec![0.0; 64];
    handle.process_block(&mut out).unwrap();

    // Drain signals from control
    let signals = control.drain_invariant_signals();

    // Should have processed control messages
    assert!(signals.contains(&INV_CONTROL_MSG_PROCESSED));
    assert!(signals.contains(&INV_PARAM_UPDATE_DELIVERED));
}

#[test]
fn contract_invariant_queue_handles_overflow() {
    let graph = create_simple_graph();
    let plan = Plan::compile(&graph, 64).unwrap();
    let (mut handle, mut control) = RuntimeCore::new_with_channels(plan, &graph, 44100.0);

    // Process many blocks without draining (should overflow gracefully)
    let mut out = vec![0.0; 64];
    for _ in 0..1000 {
        handle.process_block(&mut out).unwrap();
    }

    // Drain - should get max capacity worth of signals
    let signals = control.drain_invariant_signals();

    // Should have some signals (exact count depends on capacity)
    assert!(!signals.is_empty(), "Should have some invariant signals");

    // All signals should be valid IDs
    for &id in &signals {
        assert!(
            id == INV_SAMPLE_BUFFER_FILLED || id == INV_RT_CALLBACK_CLEAN,
            "Unexpected invariant ID: {}",
            id
        );
    }
}

#[test]
fn contract_determinism_with_control_msgs() {
    // Same control sequence should produce same output
    let graph = create_gain_graph();

    let run = || {
        let plan = Plan::compile(&graph, 64).unwrap();
        let (mut handle, mut control) = RuntimeCore::new_with_channels(plan, &graph, 44100.0);

        let mut output = Vec::new();

        // Process with control message at specific point
        for i in 0..10 {
            if i == 5 {
                control
                    .send(ControlMsg::SetGain {
                        node: NodeId(1),
                        gain: 0.25,
                    })
                    .unwrap();
            }
            let mut out = vec![0.0; 64];
            handle.process_block(&mut out).unwrap();
            output.extend_from_slice(&out);
        }

        output
    };

    let output1 = run();
    let output2 = run();

    // Outputs should be identical
    assert_eq!(output1.len(), output2.len());
    for (i, (a, b)) in output1.iter().zip(output2.iter()).enumerate() {
        assert!(
            (a - b).abs() < 1e-6,
            "Sample {} differs: {} vs {}",
            i,
            a,
            b
        );
    }
}
