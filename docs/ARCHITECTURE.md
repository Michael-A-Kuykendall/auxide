# Architecture

Auxide is a real-time-safe audio graph kernel designed for building deterministic, block-based audio processing tools.

## Core Components

### Graph
- Represents the audio graph as nodes (DSP units) and edges (connections).
- Nodes have types (e.g., oscillators, gains) with defined input/output ports.
- Edges specify data flow between ports, with rates (Audio or Control).

### Plan
- A compiled, immutable execution schedule derived from the graph.
- Performs topological sorting to avoid cycles.
- Validates invariants: required inputs connected, single-writer rule, rate compatibility.
- Precomputes buffer layouts for efficient runtime execution.

### Runtime
- Executes the plan in blocks (e.g., 64-1024 samples).
- Processes nodes in topological order, routing data via preallocated buffers.
- RT-safe: no allocations or locks in the hot path.

## Data Flow
1. Build/modify the graph (may allocate).
2. Compile to a plan (may allocate, validates invariants).
3. Run the runtime with blocks (RT-safe, deterministic).

## Safety Guarantees
- No cycles (except with delays).
- Deterministic output for given inputs.
- Panic-free under normal operation (fuzzed).</content>
<parameter name="filePath">c:/Users/micha/repos/auxide/docs/ARCHITECTURE.md