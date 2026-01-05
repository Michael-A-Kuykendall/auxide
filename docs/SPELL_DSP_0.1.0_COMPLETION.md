# Spell: DSP-0.1.0-Completion
^ Intent: Complete auxide-dsp 0.1.0 with missing plan features and dream additions for sound engineers; pack the best RT-safe DSP primitives into a comprehensive crate

=======================================================
PACKAGE: auxide-dsp (v0.1.0) - Add missing nodes, enhance examples, audit for completeness
=======================================================

@NoiseExpansion
  : add pink and brown noise generators
  > src/nodes/oscillators.rs
  $ require: PinkNoise NodeDef (filtered white noise for 1/f spectrum)
  $ require: BrownNoise NodeDef (integrated white noise for 1/f^2 spectrum)
  $ forbid: allocations in process_block
  $ require: state preallocates filter buffers
  $ prove: noise_spectra_correct -> test: noise_spectral_analysis
  $ prove: noise_no_alloc -> test: counting_allocator_noise

@FXExpansion
  : add multitap delay and convolution reverb
  > src/nodes/fx.rs
  $ require: MultitapDelay NodeDef (multiple delay taps with individual feedback/gains)
  $ require: ConvolutionReverb NodeDef (FFT-based convolution with impulse response)
  $ require: uses realfft for convolution
  $ forbid: allocations in process_block
  $ require: IR buffer preloaded in init_state
  $ prove: multitap_echoes -> test: multitap_delay_golden
  $ prove: conv_reverb_impulse -> test: convolution_reverb_ir_test
  $ prove: fx_no_alloc -> test: counting_allocator_fx_expansion

@ShaperExpansion
  : add soft clip and tube saturation
  > src/nodes/shapers.rs
  $ require: SoftClip NodeDef (tanh-based soft clipping)
  $ require: TubeSaturation NodeDef (simple diode/tube model saturation)
  $ forbid: allocations in process_block
  $ require: precomputed lookup tables if needed
  $ prove: soft_clip_curve -> test: soft_clip_transfer_function
  $ prove: tube_saturation -> test: tube_saturation_golden
  $ prove: shapers_no_alloc -> test: counting_allocator_shapers

@FilterExpansion
  : add biquad and allpass filters
  > src/nodes/filters.rs
  $ require: BiquadFilter NodeDef (LPF/HPF/BPF/Notch/Peak/Shelf with coeffs)
  $ require: AllpassFilter NodeDef (1st/2nd order allpass)
  $ forbid: allocations in process_block
  $ require: coeffs computed in init_state
  $ prove: biquad_responses -> test: biquad_frequency_response
  $ prove: allpass_phase -> test: allpass_phase_response
  $ prove: filters_no_alloc -> test: counting_allocator_filters

@UtilityExpansion
  : add mid-side processor
  > src/nodes/utility.rs
  $ require: MidSideProcessor NodeDef (encode/decode mid-side stereo)
  $ forbid: allocations in process_block
  $ prove: mid_side_roundtrip -> test: mid_side_encode_decode
  $ prove: utility_no_alloc -> test: counting_allocator_utility

@ExampleUpdates
  : update examples to match plan
  > examples/
  $ require: wavetable_synth.rs (WavetableOsc + filters + envelopes)
  $ require: delay_line.rs (MultitapDelay demo)
  $ require: filter_sweep.rs (BiquadFilter automation)
  $ prove: examples_compile -> test: example_compilation
  $ prove: examples_run -> test: example_execution

@CrateAudit
  : final audit for 0.1.0 completeness
  > Cargo.toml, README.md
  $ require: all plan features implemented
  $ require: README.md lists all node types
  $ require: no compiler warnings
  $ require: all tests pass
  $ forbid: unimplemented placeholders
  $ prove: feature_parity -> test: dsp_feature_matrix
  $ prove: no_warnings -> cargo check
  $ prove: all_tests_pass -> cargo test

@VersionLock
  : lock 0.1.0 with comprehensive tests
  $ require: version = "0.1.0" in Cargo.toml
  $ require: CHANGELOG.md entry for 0.1.0
  $ prove: version_locked -> test: version_check
  $ prove: changelog_updated -> test: changelog_check