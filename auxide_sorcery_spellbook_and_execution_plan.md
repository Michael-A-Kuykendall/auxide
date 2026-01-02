# Auxide — Sorcery Spellbook (Test‑Bound) + Execution Plan

## 0) Reading of Sorcery (new test‑bound version)
Sorcery’s newest form replaces “trust me” architecture handoff with **test-bound obligations**: a spell is only complete when it declares (a) what must exist (**require**), (b) what is forbidden (**forbid**), and (c) what must be proven via executable checks (**prove → test**). The casting phase compresses high-context intent into these obligations; the invocation phase is judged by runtime evidence rather than narrative interpretation. Open questions block sealing; sealed spells are “perfect gates” that lower-context agents can invoke without inventing missing intent.

---

## 1) Project Charter Spell

#Spell: Auxide_Charter
^ Intent: deliver a Rust-native, statically validated signal-graph DSL + executable plan compiler + deterministic runtime, credible to institutional audiences.

@Auxide
  : requirements -> crate
  $ require: crate `auxide` with public `graph`, `plan`, `rt`, `dsl` modules
  $ require: DESIGN.md (goals/non-goals/invariants)
  $ require: benches + CI gates
  $ forbid: allocations on real-time thread
  $ forbid: hidden global state
  $ forbid: nondeterministic execution order
  $ prove: deterministic_exec -> test: rt_determinism
  $ prove: no_rt_alloc -> test: rt_no_alloc
  $ prove: doc_examples_compile -> test: doctest_all

~ target_users = Rust systems engineers; audio is reference domain, not the boundary.

---

## 2) Core Spells (Foundation)

#Spell: Graph_Core
^ Intent: represent a correct-by-construction signal graph with explicit rates and typed ports.

@Graph
  : nodes+edges -> validated_dag
  $ require: NodeId, PortId, Edge, Graph structs
  $ require: Rate enum {Audio, Control, Event}
  $ require: Port<T, Rate> typing (or equivalent) to prevent invalid connections
  $ forbid: implicit rate coercion
  $ forbid: graph cycles unless explicitly modeled (future work)
  $ prove: rejects_rate_mismatch -> test: graph_rate_mismatch
  $ prove: rejects_cycles -> test: graph_cycle_detection
  $ prove: stable_node_ordering -> test: graph_stable_toposort

~ graphs are immutable once sealed into a Plan.


#Spell: Plan_Compiler
^ Intent: compile a validated graph into an executable plan with preallocated buffers and deterministic order.

@Planner
  : Graph -> Plan
  $ require: topological sort + execution schedule
  $ require: buffer lifetime analysis + reuse strategy
  $ require: deterministic_plan_debug
  $ forbid: runtime graph traversal on audio thread
  $ forbid: serialization_format_commitment in v0.1.0
  $ prove: plan_is_deterministic -> test: plan_stability
  $ prove: buffer_reuse_soundness -> test: plan_buffer_liveness
  $ prove: debug_is_stable -> test: plan_debug_snapshot

~ compilation may allocate; execution must not.


#Spell: Execution_Model
^ Intent: establish block-based pull execution for deterministic RT scheduling.

@Execution
  : plan -> block_outputs
  $ require: block_pull_execution
  $ forbid: push_execution in v0.1.0
  $ prove: schedule_is_output_driven -> test: plan_output_rooted


#Spell: Feedback_Model
^ Intent: enable feedback effects via stateful nodes without cyclic graphs.

@Feedback
  : stateful_nodes -> feedback_effects
  $ require: graph_is_dag
  $ require: stateful_nodes_enable_feedback_effects
  $ forbid: cyclic_edges in v0.1.0
  $ prove: delayline_enables_feedback -> test: node_delay_golden


#Spell: RT_Engine
^ Intent: execute a Plan in hard real-time constraints (no alloc, no locks), block-based.

@Runtime
  : Plan+Inputs -> Outputs
  $ require: block processing API (frames: N)
  $ require: explicit scratch/buffer arenas created off-thread
  $ forbid: heap alloc within process_block
  $ forbid: mutex/condvar within process_block
  $ forbid: panics escaping audio callback
  $ prove: no_alloc_in_block -> test: rt_no_alloc
  $ prove: no_lock_in_block -> test: rt_no_lock
  $ prove: panic_contained -> test: rt_panic_containment
  $ prove: bitwise_stable_output -> test: rt_determinism

~ If a node errors, fail closed (silence or configured fallback) and surface diagnostic off-thread.


#Spell: Diagnostics_Offthread
^ Intent: surface RT errors off-thread without blocking.

@Diagnostics
  : rt_errors -> offthread_queue
  $ require: lockfree_diag_queue
  $ forbid: allocations in enqueue
  $ prove: diag_queue_nonblocking -> test: rt_diag_queue


#Spell: RT_Proof_Harness
^ Intent: provide empirical proof of RT constraints via test harness.

@Harness
  : rt_code -> alloc_lock_proofs
  $ require: alloc_counter_harness
  $ require: lock_detector_harness (best-effort)
  $ prove: harness_catches_alloc -> test: rt_alloc_harness_selftest

---

## 3) DSL Spells (Ergonomics without Magic)

#Spell: DSL_Builder
^ Intent: provide an embedded DSL that constructs graphs fluently while preserving type/rate correctness.

@DSL
  : Rust_API -> Graph
  $ require: GraphBuilder, NodeHandle, Port<T, Rate>, connect() API
  $ require: ergonomic constants/modulation wiring
  $ forbid: procedural macro required for core usage (optional convenience only)
  $ prove: dsl_constructs_same_graph -> test: dsl_equivalence
  $ prove: compiler_errors_are_clear -> test: ui_tests (compile-fail)

~ primary audience tolerates explicitness; optimize for correctness.


#Spell: DSL_Validation
^ Intent: make invalid graphs unrepresentable or rejected with precise diagnostics.

@Diagnostics
  : invalid_input -> error
  $ require: error codes for cycle, rate mismatch, missing node, unbound port
  $ require: span-ish context from builder steps (best effort)
  $ prove: errors_are_actionable -> test: diag_snapshot

---

## 4) Reference Domain Spells (Audio as Proof, Not Boundary)

#Spell: Audio_Adapter
^ Intent: integrate with a minimal audio backend to prove real-time execution end-to-end.

@Audio
  : cpal_callback -> auxide_runtime
  $ require: CPAL output stream example
  $ require: sample format handling (f32 baseline)
  $ forbid: allocation in callback
  $ prove: plays_sine -> test: offline_golden (render-to-buffer)
  $ prove: callback_no_alloc -> test: rt_no_alloc

~ live audio tests may be flaky; use offline rendering for CI truth.


#Spell: Reference_Nodes
^ Intent: minimal node set to validate the architecture.

@Nodes
  : params -> audio
  $ require: SineOsc
  $ require: Gain
  $ require: Mix (2->1)
  $ require: OutputSink
  $ forbid: feature creep (filters/reverb/granular) until Plan+RT are proven
  $ prove: node_math_correct -> test: node_golden

---

## 5) Institutional Credibility Spells (Proof + Repro)

#Spell: Benchmarks
^ Intent: publish reproducible evidence of latency, throughput, and RT safety properties.

@Bench
  : code -> measurements
  $ require: criterion benches for plan compile + process_block
  $ require: baseline comparisons (optional)
  $ require: report template (README table)
  $ forbid: benchmark claims without scripts
  $ prove: benches_run -> test: ci_bench_smoke


#Spell: Documentation
^ Intent: ensure outsiders can audit intent, invariants, and usage quickly.

@Docs
  : design -> comprehension
  $ require: README with 1-sentence thesis + minimal example
  $ require: DESIGN.md with invariants + non-goals
  $ require: SAFETY.md (RT rules and what is enforced)
  $ prove: docs_examples_compile -> test: doctest_all


#Spell: Release_Gates
^ Intent: prevent reputation damage through premature claims.

@CI
  : commits -> pass/fail
  $ require: fmt + clippy + test
  $ require: compile-fail UI tests for DSL validation
  $ require: offline render golden tests
  $ forbid: release tag unless all Prove obligations are green
  $ prove: gate_enforced -> test: ci_required_checks


#Spell: Claim_Control
^ Intent: ensure public claims are backed by executable evidence.

@Claims
  : claims_manifest -> test_mappings
  $ require: README_claims_mapped_to_tests
  $ prove: claims_backed -> test: claims_manifest_check

---

## 6) Execution Plan (Slices)

### Slice 1 — “Graph Truth” (sealable)
- Implement `Graph_Core` + tests: cycle/rate/stable ordering.
- Output: `auxide::graph` usable and documented.

### Slice 2 — “Plan is a Contract”
- Implement `Plan_Compiler` + buffer liveness tests.
- Output: deterministic `Plan` + debug print.

### Slice 3 — “RT Engine”
- Implement `RT_Engine` + no-alloc/no-lock harness.
- Output: offline block rendering is stable.

### Slice 4 — “DSL”
- Implement `DSL_Builder` + compile-fail diagnostics.
- Output: short fluent examples compile.

### Slice 5 — “Audio Proof”
- Implement `Audio_Adapter` + minimal nodes.
- Output: offline golden render; optional live CPAL demo.

### Slice 6 — “Institutional Package”
- Benchmarks, docs, CI gates; publish 0.1.0 with conservative claims.

---

## 7) Resolved Decisions (locked for sealing v0.1.0)

### 1) Execution model: **Block-based pull (output-driven)**

**Decision:** Pull model, evaluated in fixed-size blocks (`process_block(frames)`).
**Rationale:** Deterministic scheduling, simpler "no alloc/no lock" enforcement, predictable cache behavior. Push is possible later as an adapter.
**Sorcery impact:** Removes "agent assumption" risk; simplifies `Plan` contract.

### 2) Feedback loops: **Disallow cycles in Graph; allow "explicit stateful nodes"**

**Decision:** The core Graph remains a DAG. Cycles are modeled via **stateful nodes** (e.g., `DelayLine`, `OneSampleDelay`, `History`, `Integrator`) that carry internal state across blocks. No graph-level cycles in v0.1.0.
**Rationale:** This is how you keep the core compiler simple and still support legitimate audio feedback patterns. You get the *effect* of feedback without cyclic edges.
**Test-binding:** Prove via golden render that `DelayLine` behaves correctly and does not require cyclic edges.

### 3) Serialization: **None for v0.1.0; deterministic Debug + optional feature later**

**Decision:** Do not commit to RON/JSON/Protobuf in v0.1.0. Provide:
* deterministic `Debug` / `Display` of `Graph` and `Plan`
* stable hashing for plan identity (optional)
Add serialization as `auxide-serde` feature or separate crate later.
**Rationale:** Serialization adds surface area and dependency risk. For institutional credibility, deterministic artifacts + tests are sufficient at v0.1.0.

### 4) DSL ergonomics: **Builder API + typed ports; optional macro only for sugar**

**Decision:** Core DSL is pure Rust: `GraphBuilder`, `NodeHandle`, `Port<T, Rate>`, `connect()` with compile-time constraints where feasible. Add a macro only if it materially improves readability, but it is not required.
**Rationale:** This matches your "institutional" stance: explicit, auditable, minimal magic.

### 5) RT guarantees: **Enforce by harness + "rt" module dependency firewall**

**Decision:** Enforce "no alloc/locks" by:
* a single RT test harness using a counting global allocator (tests only)
* a crate/module boundary: `auxide-rt` forbids common allocation paths by policy (deny-list imports, Clippy + review gate), plus runtime tests
**Rationale:** In Rust, proofs are mostly empirical + discipline. Sorcery's test-binding is the right tool; add a small static "firewall" to reduce foot-guns.

### 6) RT error handling: **Fail-closed + deferred diagnostics channel**

**Decision:** In RT, errors become:
* **silence** (or configured fallback) for the affected node
* a lightweight error code enqueued into a **lock-free ring buffer** drained off-thread
* panics contained: catch-unwind at graph boundary only if you accept the overhead; otherwise forbid panics and treat as abort in debug
**Rationale:** You need an explicit mechanism; otherwise "fail-closed" becomes hand-wavy.

---

## 7.1) Spell Amendments

Add/modify these spells so Sorcery can "seal" v0.1.0 without ambiguity:

### Spell: Execution_Model (new, add to Core Spells)

#Spell: Execution_Model
^ Intent: establish block-based pull execution for deterministic RT scheduling.

@Execution
  : plan -> block_outputs
  $ require: block_pull_execution
  $ forbid: push_execution in v0.1.0
  $ prove: schedule_is_output_driven -> test: plan_output_rooted

### Spell: Feedback_Model (new, add to Core Spells)

#Spell: Feedback_Model
^ Intent: enable feedback effects via stateful nodes without cyclic graphs.

@Feedback
  : stateful_nodes -> feedback_effects
  $ require: graph_is_dag
  $ require: stateful_nodes_enable_feedback_effects
  $ forbid: cyclic_edges in v0.1.0
  $ prove: delayline_enables_feedback -> test: node_delay_golden

### Spell: Serialization (revise existing in Plan_Compiler)

Update #Spell: Plan_Compiler to include:
  $ require: deterministic_plan_debug
  $ forbid: serialization_format_commitment in v0.1.0
  $ prove: debug_is_stable -> test: plan_debug_snapshot

### Spell: Diagnostics_Offthread (new, add to RT_Engine)

#Spell: Diagnostics_Offthread
^ Intent: surface RT errors off-thread without blocking.

@Diagnostics
  : rt_errors -> offthread_queue
  $ require: lockfree_diag_queue
  $ forbid: allocations in enqueue
  $ prove: diag_queue_nonblocking -> test: rt_diag_queue

### Spell: RT_Proof_Harness (new, add to RT_Engine)

#Spell: RT_Proof_Harness
^ Intent: provide empirical proof of RT constraints via test harness.

@Harness
  : rt_code -> alloc_lock_proofs
  $ require: alloc_counter_harness
  $ require: lock_detector_harness (best-effort)
  $ prove: harness_catches_alloc -> test: rt_alloc_harness_selftest

### Spell: Claim_Control (new, add to Institutional Credibility Spells)

#Spell: Claim_Control
^ Intent: ensure public claims are backed by executable evidence.

@Claims
  : claims_manifest -> test_mappings
  $ require: README_claims_mapped_to_tests
  $ prove: claims_backed -> test: claims_manifest_check

---

## 7.2) Test Tiers (to keep high-value, not 50+ vanity checks)

Define **three proof tiers**:

1. **Tier A (Seal blockers):** RT no-alloc/no-lock, determinism, cycle rejection, rate mismatch rejection
2. **Tier B (Credibility):** golden renders for core nodes, plan stability snapshots, bench smoke
3. **Tier C (Nice-to-have):** extended diagnostics polish, serialization, more nodes

v0.1.0 only needs Tier A + a small Tier B set.

---

## 8) Definition of Done (v0.1.0)

- All `prove` tests green.
- Deterministic offline rendering of a reference graph.
- `process_block` verified no-alloc + no-lock.
- README + DESIGN + SAFETY present; examples compile.
- Benchmarks published with scripts and raw numbers.

