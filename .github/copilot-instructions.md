# Copilot Instructions for Auxide Crate Development

## Core Guiding Methodology: Sorcery

Sorcery is a **design doctrine for enforceable architecture handoff**, not a tool or framework. It binds high-context intent into "test-bound spells" using symbolic glyph notation, ensuring correctness through executable tests rather than semantic parsing. Spells are "sealed" only when complete (no `?` questions, all obligations bound to tests). The process separates high-context casting (design) from low-context invocation (implementation), preventing intent evaporation.

### Sorcery Intuition
- **Casting:** Compress reasoning into terse, complete spells with requirements (`$ require`), forbids (`$ forbid`), and proofs (`$ prove` → tests).
- **Invocation:** Expand spells into code + tests, verified by runtime evidence.
- **Asymmetry:** Casting is high-context (architect); invocation low-context (agents). Spells declare "what must hold" and "what must never happen" so agents act correctly without guessing.
- **Test-Bound:** No parsing; verification via tests. Forbids are first-class; incomplete spells block sealing.
- **Cleanup Rule:** Spells stay in planning docs; production code excludes glyphs.

### Full Glyph Notation Reference
| Symbol | Meaning | Example |
|:------:|---------|---------|
| `#` | Spell name | `#Spell: Tokenize` |
| `^` | Intent (required!) | `^ produce stable tokens` |
| `@` | Component/Entity | `@Tokenizer` |
| `:` | Input → Output | `: utf8 -> tokens` |
| `$` | Obligation (require/forbid/prove) | `$ prove: deterministic -> test: det` |
| `~` | Assumption | `~ valid_utf8` |
| `>` | Dependency | `> @Tokenizer` |
| `?` | Open question (blocks sealing) | `? performance reqs` |

**Incantation Patterns:** Use existing glyphs for all concepts (e.g., `$ require:` for existence, `$ forbid:` for prohibitions, `$ prove:` for test-evidence).

## Auxide Plan Outline (Sorcery-Driven)

Auxide is a Rust-native, statically validated signal-graph DSL + executable plan compiler + deterministic runtime for audio processing, credible to institutional audiences. Target users: Rust systems engineers; audio as reference domain.

### Charter Spell
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

### Core Spells (Foundation)
- **Graph_Core:** Represent correct-by-construction signal graph with explicit rates/typed ports. Requires NodeId, PortId, Edge, Graph structs; Rate enum; forbids implicit coercion; proves rejects mismatches/cycles/stable ordering.
- **Plan_Compiler:** Compile graph into executable plan with buffers/schedule. Requires topo sort/buffer analysis/deterministic debug; forbids runtime traversal/serialization commitment; proves determinism/buffer soundness/debug stability.
- **Execution_Model:** Block-based pull execution. Requires pull; forbids push in v0.1.0; proves output-driven.
- **Feedback_Model:** DAG graph with stateful nodes for feedback. Requires DAG/stateful nodes; forbids cycles; proves DelayLine enables feedback.
- **RT_Engine:** Execute plan in RT (no alloc/locks). Requires block API/arenas; forbids alloc/locks/panics; proves no-alloc/no-lock/containment/determinism.
- **Diagnostics_Offthread:** Surface RT errors via lock-free queue. Requires queue; forbids alloc in enqueue; proves nonblocking.
- **RT_Proof_Harness:** Empirical RT proofs via harness. Requires alloc/lock detectors; proves harness works.

### DSL Spells (Ergonomics)
- **DSL_Builder:** Fluent graph construction. Requires GraphBuilder/NodeHandle/Port/connect API; forbids required macros; proves equivalence/clear errors.
- **DSL_Validation:** Reject invalid graphs with diagnostics. Requires error codes/context; proves actionable errors.

### Reference Domain Spells (Audio Proof)
- **Audio_Adapter:** CPAL integration for RT proof. Requires CPAL stream/sample handling; forbids alloc in callback; proves plays sine/callback no-alloc.
- **Reference_Nodes:** Minimal nodes (SineOsc/Gain/Mix/OutputSink). Requires nodes; forbids creep; proves math correct.

### Institutional Credibility Spells (Proof + Repro)
- **Benchmarks:** Reproducible latency/throughput evidence. Requires criterion/report; forbids claims without scripts; proves benches run.
- **Documentation:** Auditable design/usage. Requires README/DESIGN/SAFETY; proves examples compile.
- **Release_Gates:** Prevent premature claims. Requires fmt/clippy/tests/UI/golden; forbids release without green proves; proves gates enforced.
- **Claim_Control:** Back claims with tests. Requires manifest mapping; proves claims backed.

### Resolved Decisions (Locked)
1. **Execution:** Block-based pull (output-driven) for determinism.
2. **Feedback:** DAG + stateful nodes (e.g., DelayLine) for effects without cycles.
3. **Serialization:** None in v0.1.0; deterministic Debug + optional later.
4. **DSL:** Pure Rust builder + typed ports; macro optional for sugar.
5. **RT Guarantees:** Harness + module firewall (deny alloc paths, Clippy).
6. **RT Errors:** Fail-closed (silence) + lock-free ring buffer for diagnostics.

### Test Tiers
- **Tier A (Seal Blockers):** RT no-alloc/locks, determinism, cycle/rate rejection.
- **Tier B (Credibility):** Golden renders, plan snapshots, bench smoke.
- **Tier C (Nice-to-Have):** Extended polish, serialization, more nodes.
v0.1.0: Tier A + small Tier B.

### Execution Slices
1. **Graph Truth:** Implement Graph_Core + tests.
2. **Plan is Contract:** Plan_Compiler + buffer tests.
3. **RT Engine:** RT_Engine + harness.
4. **DSL:** DSL_Builder + diagnostics.
5. **Audio Proof:** Audio_Adapter + nodes + golden render.
6. **Institutional Package:** Benchmarks/docs/CI; publish 0.1.0.

### Definition of Done (v0.1.0)
- All prove tests green (Tier A+B).
- Deterministic offline rendering.
- process_block no-alloc/no-lock verified.
- README/DESIGN/SAFETY present; examples compile.
- Benchmarks with scripts/raw numbers.

**Always reference this for intent, obligations, and test bindings. Cast new spells for extensions. Seal only when complete.**

## PPT Invariant Guide Integration

The PPT Invariant Guide (ppt_invariant_guide.md) provides a layered test system combining Predictive Property-Based Testing (PPT) with runtime invariant enforcement. Use it for high-churn projects to maintain semantic integrity:

- **Layers**: E-Test (exploration), P-Test (property), C-Test (contract).
- **Invariants**: Embed `assert_invariant(condition, message, context)` in code for runtime checks and contract tracking.
- **Contract Tests**: Use `contract_test(name, invariants)` to enforce critical rules post-refactor.
- **Integration**: Implement `invariant_ppt.rs` module for auxide to apply this system, ensuring properties and invariants are tested alongside sorcery proves.

Reference ppt_invariant_guide.md for setup and expansion ideas.