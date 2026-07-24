# Auxide Suite Assessment

Date: 2026-03-06

## Executive Summary

The four-repo Auxide stack is not vapor. The core architecture is coherent and the codebase shows real engineering discipline around deterministic execution, RT-safety, and separation of concerns.

The current state is best described as:

- `auxide`: solid kernel foundation
- `auxide-dsp`: substantial and mostly real DSP library
- `auxide-io`: practical hardware bridge with credible scope
- `auxide-midi`: promising integration layer with one critical architectural gap

The main blocker to a compelling MIDI-first synth workflow is not that MIDI parsing or DSP are missing. It is that runtime control of the external DSP nodes is incomplete, so the parts that matter for a playable synth are frozen after graph compilation.

## Workspace Status

The shared workspace entrypoint is `C:/Users/micha/repos/auxide-workspace.code-workspace`.

It now has:

- explicit folder names for all four repos
- shared MCP registration for `auxide-brain`
- excludes for `target`, `tarpaulin`, and `.mcp` artifacts so the workspace stays usable

The MCP server under `auxide/.mcp/auxide-brain-mcp.py` was also hardened to recover from a bad persisted Chroma store instead of crashing at startup.

## Product Intent By Repo

### auxide

Purpose:
- real-time-safe, deterministic graph kernel
- graph compilation, invariant checking, topological scheduling, runtime execution

What is real:
- graph -> plan -> runtime split exists
- built-in runtime control exists for built-in nodes like `SineOsc` and `Gain`
- external node hook exists through `NodeType::External`

Assessment:
- good solid engineering
- the architecture is focused and not overblown
- this is the strongest part of the stack conceptually

### auxide-dsp

Purpose:
- large DSP node library plugged into the kernel through trait-based external nodes

What is real:
- oscillators, filters, envelopes, effects, dynamics, shapers, utilities, and pitch modules are present
- helper math and builder patterns exist
- prior audit artifacts claim RT verification and broad test coverage

Assessment:
- mostly real, not hand-wavy
- the implementation surface is large enough to matter
- some README and audit language is more absolute than the evidence fully justifies, but this is not empty scaffolding

### auxide-io

Purpose:
- connect Auxide runtimes to audio hardware through CPAL

What is real:
- stream controller, buffer adaptation, channel routing, stream state, and error recovery modules are present
- examples exist for tone playback and mixing

Assessment:
- practical and focused
- less ambitious than DSP, but appropriately scoped and useful
- likely the least conceptually risky repo in the stack

### auxide-midi

Purpose:
- MIDI input, voice allocation, CC mapping, smoothing, and synth workflow glue

What is real:
- `midir` input layer is present
- voice allocator and voice state exist
- CC mapping and smoothing exist
- end-to-end demos exist

Assessment:
- real code, but the repo overstates end-to-end readiness
- the MIDI plumbing exists, but the last mile into expressive synthesis is not fully solved

## What Is Solid

- The stack decomposition is right. Kernel, DSP, I/O, and MIDI belong in separate crates.
- The kernel stays narrow instead of absorbing all DSP complexity.
- External node support is a good design choice for ecosystem growth.
- The codebase is full of concrete modules, examples, and tests rather than empty stubs.
- The MicroFreak-first path is directionally correct because it stress-tests the exact integration points that matter.

## What Is Weak Or Incomplete

### 1. External DSP parameters are not first-class runtime controls

This is the biggest current gap.

What that means:

- built-in nodes can be adjusted at runtime through control messages
- external DSP nodes are configured at graph-build time and then effectively frozen

Impact:

- polyphonic synth examples cannot actually retune oscillators per note in a clean real-time way
- filter cutoff, envelopes, and other synth controls are not wired through as true live controls
- MIDI input works, but expressive instrument behavior is compromised

### 2. Envelope control path is incomplete in the kernel runtime

There are explicit TODOs around gate triggering and all-notes-off release handling in the runtime.

Impact:

- the system can describe envelopes but does not yet have a clean generalized runtime control story for them

### 3. auxide-midi demos are partially honest, partially aspirational

The repo helpfully documents current limitations, but some examples still read more complete than they actually are.

Observed pattern:

- `poly_synth.rs` says it is an end-to-end demo
- the file itself also states that all notes currently play at 440Hz because the graph is immutable after compilation

That is not fatal, but it means the repo is closer to a strong prototype than a finished playable MIDI synth layer.

### 4. Documentation/version drift exists

Some README ecosystem/version text is ahead of or inconsistent with current Cargo versions.

Impact:

- low engineering risk
- medium trust and maintenance risk

### 5. The parent Cargo workspace needed explicit excludes

The parent `C:/Users/micha/repos/Cargo.toml` virtual workspace was sitting above many sibling repositories that also declare their own `[workspace]`. That caused Cargo to report multiple workspace roots under the same tree.

Status:

- fixed by adding explicit `exclude` entries for the sibling workspace roots
- `cargo metadata` now resolves the Auxide suite correctly from the parent workspace

Impact:

- terminal-driven Cargo workflows are usable again for the Auxide suite
- the suite can now be verified from a shared workspace manifest instead of being blocked by unrelated sibling projects

### 6. There is at least one real cross-crate API drift today

After the Cargo workspace fix, `auxide` tests pass, but the broader stack is not fully clean.

Observed failure:

- `cargo test -p auxide` passes
- `cargo test -p auxide-midi` currently fails through `auxide-io`

Current concrete mismatch:

- `auxide-io` imports `RuntimeHandle` from `auxide::rt` and calls `Runtime::sample_rate()`
- the compile signal indicates the current public API expectations between `auxide` and `auxide-io` have drifted

Impact:

- the kernel is in better shape than the end-to-end stack
- cross-crate integration still needs active shake-down even where individual components look mature

## Vapor vs. Substance

### Substance

- kernel architecture
- DSP module surface area
- RT-safety intent and discipline
- CPAL hardware bridge
- MIDI parsing and voice allocation primitives

### Limited or overstated

- fully playable polyphonic MIDI synthesis
- generalized runtime modulation of external DSP nodes
- complete end-to-end synth ergonomics

### Not obviously deprecated

- the codebase does not look abandoned
- the repos show recent audit and cleanup activity
- public GitHub issue lists currently show zero open issues for all four repos

## Competitive Direction vs. SuperCollider

The credible differentiation is not trying to out-artify SuperCollider.

The stronger angle is:

- deterministic graph compilation
- Rust-native correctness and type discipline
- programmable audio tooling for software engineers
- RT-safe infrastructure that can be embedded, tested, and reasoned about

That is a valid product thesis. The current codebase supports that thesis at the kernel and DSP levels more than at the live-performance synth integration layer.

## Most Important Next Moves

### Priority 1: solve runtime control for external DSP nodes

Without this, MIDI remains mostly an input demo.

Likely target:

- introduce a control-parameter path for external nodes
- let external node defs expose stable parameter IDs or control ports
- route MIDI note, gate, pitch bend, velocity, and CC data through the kernel without rebuilding graphs

### Priority 2: make one truly playable reference synth

Recommended target workflow:

- Arturia MicroFreak input
- 4 to 8 voice polyphony
- oscillator frequency per voice
- envelope gate on/off
- filter cutoff and resonance CC mapping
- pitch bend
- audio out through auxide-io

Only one path needs to be excellent first.

### Priority 3: clean documentation to match reality

- align README versions with Cargo manifests
- clearly mark which demos are proofs of concept and which are production-grade
- document the actual current limitations of the MIDI layer and the roadmap to remove them

### Priority 4: fix cargo workflow contamination from the parent repos workspace

The stack needs a predictable way to run `cargo check`, `cargo test`, and examples without parent-workspace interference.

## Recommended Immediate Plan

1. Audit and redesign runtime control for external nodes in `auxide`
2. Refactor `auxide-midi` around that control path instead of around frozen external node instances
3. Stand up a single honest MicroFreak-first synth demo as the reference path
4. Repair the current `auxide` ↔ `auxide-io` API drift and re-run the full suite
5. Then tighten docs, release notes, and examples around what is actually proven

## Bottom Line

This suite is worth continuing. It is not fake work.

The kernel and DSP ideas are good, the repo split is right, and the engineering intent is visible in the code. The main thing separating the current stack from a compelling Rust-native programmable audio environment is the missing live control model for the external DSP layer.

That is a real problem, but it is also a concrete one.