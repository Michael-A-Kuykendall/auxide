#Spell: DSP-Trait-Extension
^ Intent: keep auxide kernel locked while enabling node extension via traits; implement DSP nodes in auxide-dsp using the trait hook

=======================================================
PACKAGE: auxide (0.1.x -> 0.2.0) - add trait-based node extension, keep core semantics
=======================================================

@VersionBump
  : auxide 0.1.x -> 0.2.0
  $ require: Cargo.toml version = "0.2.0"
  $ require: CHANGELOG.md entry for 0.2.0 (notes trait-based node extension point)
  $ forbid: changing semantics of existing NodeType variants
  $ forbid: removing existing NodeType variants
  $ prove: existing_0.1_graphs_compile -> test: backward_compat_examples
  $ prove: core_nodes_snapshot -> test: node_type_snapshot

@NodeTrait
  : define RT-safe node trait (no alloc/lock in process)
  > src/node.rs (new) or similar module
  $ require: pub trait NodeDef {
      type State;
      fn input_ports(&self) -> &'static [Port];
      fn output_ports(&self) -> &'static [Port];
      fn required_inputs(&self) -> usize;
      fn init_state(&self, sample_rate: f32, block_size: usize) -> Self::State;
      fn process_block(
        &self,
        state: &mut Self::State,
        inputs: &[&[f32]],
        outputs: &mut [Vec<f32>],
        sample_rate: f32,
      );
    }
  $ forbid: allocations in process_block
  $ forbid: locks/atomics/arcs in State
  $ forbid: dynamic port counts
  $ prove: process_no_alloc -> test: counting_allocator_trait_nodes
  $ prove: ports_match_process -> test: trait_node_port_count_validation

@KernelIntegration
  : allow trait-based nodes without altering existing NodeType semantics
  > src/graph.rs / src/plan.rs / src/rt.rs
  $ require: Graph accepts NodeType::External(Box<dyn NodeDef + Send + Sync + 'static>)
  $ require: Plan supports External nodes in scheduling
  $ require: Runtime stores External nodes + their State, allocates all buffers at init
  $ require: no alloc/lock in Runtime::process_block for External nodes
  $ require: External nodes use static port metadata (no Vec alloc)
  $ forbid: touching existing node match arms behavior
  $ forbid: adding additional core NodeType variants beyond External
  $ prove: external_node_roundtrip -> test: external_node_compiles_and_runs
  $ prove: core_process_unchanged -> test: core_nodes_goldens

@PortStatics
  : move core port specs to static slices
  > src/graph.rs
  $ require: const PORTS_MONO_IN_MONO_OUT, PORTS_DUAL_IN_MONO_OUT, PORTS_MONO_OUT, etc.
  $ require: core NodeType port methods return &'static [Port]
  $ prove: port_methods_no_alloc -> test: port_query_alloc_tracking
  $ prove: port_tables_snapshot -> test: port_table_snapshot

@Docs
  : document extension point
  > README.md, docs/ARCHITECTURE.md
  $ require: section on trait-based nodes and RT constraints
  $ forbid: claims that traits allow alloc/lock in process
  $ prove: docs_mention_trait_safety -> test: doc_check_trait_section

=======================================================
PACKAGE: auxide-dsp (v0.1.0) - DSP via trait nodes, kernel unchanged
=======================================================

@CrateCreation
  : new_crate -> auxide-dsp/
  $ require: Cargo.toml with name = "auxide-dsp", version = "0.1.0"
  $ require: dependency auxide = "0.2"
  $ require: README.md explaining DSP utilities + trait nodes
  $ require: LICENSE (MIT matching auxide)

@NodeImplementations
  : implement DSP nodes as NodeDef types (not added to core enum)
  > src/nodes/*.rs
  $ require: Oscillators (saw, square, triangle, pulse with polyblep, wavetable, supersaw)
  $ require: Noise (white, pink, brown)
  $ require: Filters (SVF, ladder, comb, formant)
  $ require: Envelopes (ADSR/AR/AD with curves)
  $ require: LFO (multiple waveforms)
  $ require: FX (delay/multitap/chorus/flanger/phaser/reverb/conv reverb)
  $ require: Dynamics (compressor/limiter/gate/expander)
  $ require: Shapers (waveshaper/softclip/hardclip/tube/bitcrusher/DC blocker)
  $ require: Pitch/time (pitch detector, spectral gate, pitch shifter)
  $ require: Utility (ring mod, crossfader, mid/side, stereo width, parameter smoother)
  $ forbid: allocations/locks in process_block
  $ require: all internal Vecs preallocated in init_state; capacities fixed
  $ prove: dsp_nodes_no_alloc -> test: counting_allocator_dsp_nodes
  $ prove: dsp_nodes_block_size_safe -> test: block_size_bounds

@Helpers
  : DSP utilities (no RT alloc)
  > src/lib.rs (or modules)
  $ require: db_to_linear, linear_to_db, freq_to_phase_increment, ms_to_samples, polyblep, linear_interpolate, compute_exponential_coefficient
  $ forbid: allocations in helpers
  $ prove: helpers_pure -> test: helper_purity

@TablesWindows
  : wavetable/window generators (init-time only)
  > src/wavetables.rs, src/windows.rs
  $ require: generate_sine/saw/square/triangle tables
  $ require: hann/hamming/blackman windows
  $ forbid: RT use for generation; intended for init
  $ prove: tables_match_golden -> test: wavetable_golden
  $ prove: windows_sum_correctly -> test: window_overlap_add

@BuilderPatterns
  : graph builders emitting Auxide graphs wired with External nodes
  > src/builders.rs
  $ require: SynthBuilder, EffectsChainBuilder producing Graph + Plan using NodeDef-backed nodes
  $ forbid: hiding RT-safety requirements in APIs
  $ prove: builders_produce_valid_graphs -> test: builder_graph_validation

@TestsExamples
  : coverage for nodes and alloc discipline
  > tests/, examples/
  $ require: allocation guards per node type
  $ require: golden tests for osc/filter/envelope/effects
  $ require: examples: wavetable_synth.rs, delay_line.rs, filter_sweep.rs
  $ forbid: f32 exact equality; use tolerance
  $ prove: all_nodes_tested -> test: dsp_node_test_matrix

@Docs
  : document how to register/use trait nodes
  > README.md, rustdoc
  $ require: explicit note that kernel core is unchanged; DSP lives in auxide-dsp via traits
  $ prove: docs_cover_registration -> test: doc_check_registration
