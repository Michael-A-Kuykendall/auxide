# Design Document

## Goals

- Deliver a Rust-native, statically validated signal-graph DSL + executable plan compiler + deterministic runtime.
- Credible to institutional audiences (e.g., audio engineers, systems programmers).
- Reference domain: audio processing, but generalizable to other real-time signal domains.

## Non-Goals

- General-purpose graph library.
- Support for dynamic graph changes at runtime.
- Built-in audio I/O; integrate with CPAL or similar.
- Serialization in v0.1.0.

## Invariants

- Graphs are immutable once sealed into a Plan.
- Execution is deterministic and real-time safe (no alloc/locks).
- All graphs are DAGs; cycles modeled via stateful nodes.
- Rates are explicit and enforced at build time.

## Architecture

### Core Components

- **Graph**: Correct-by-construction DAG with nodes, ports, edges.
- **Plan**: Compiled execution order + buffers.
- **Runtime**: Block-based pull execution.
- **DSL**: Builder API for ergonomics.

### Execution Model

Block-based pull: output nodes pull from inputs, ensuring determinism.

### Feedback

Via stateful nodes (e.g., DelayLine), not graph cycles.

### RT Guarantees

Enforced by harness + module firewall; no alloc/locks in process_block.