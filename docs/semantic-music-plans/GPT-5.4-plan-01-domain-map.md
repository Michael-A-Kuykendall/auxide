# GPT-5.4 Plan 01: Semantic Music Domain Map

This document captures the first planning draft from GPT-5.4 for the Auxide semantic music system.

Purpose:
- preserve this plan as a distinct author/version artifact
- support later comparison against two additional AI-authored plans
- serve as input for a later amalgamated master plan

---

Below is a domain map derived from the chat, aimed at **defining music semantically in Rust** rather than exposing users to SuperCollider-style plumbing.

# Core thesis

The system should let users **program music**, not audio-server internals.

So the map should center on:

* musical meaning
* typed semantic objects
* deterministic lowering into runtime audio structures

Not on:

* buses
* node IDs
* server trees
* raw parameter arrays
* obscure event taxonomies

---

# Top-level domain map

```text
Music Domain
├── Sound
│   ├── Instrument
│   ├── Patch
│   ├── Voice
│   ├── Oscillator / Generator
│   ├── Filter / Dynamics / FX
│   ├── Envelope
│   ├── Modulation
│   └── Timbre
│
├── Time
│   ├── Tempo
│   ├── Meter
│   ├── Clock
│   ├── Transport
│   ├── Timeline
│   ├── Phrase
│   ├── Section
│   ├── Arrangement
│   └── Automation Lane
│
├── Performance
│   ├── Note Event
│   ├── Gesture
│   ├── Articulation
│   ├── MIDI Input
│   ├── Controller Mapping
│   ├── Scene
│   ├── Macro Control
│   └── Humanization / Variation
│
├── Structure
│   ├── Melody
│   ├── Harmony
│   ├── Chord Progression
│   ├── Rhythm
│   ├── Pattern
│   ├── Motif
│   ├── Variation Rule
│   └── Generative Rule
│
├── Space
│   ├── Route
│   ├── Send
│   ├── Return
│   ├── Bus
│   ├── Group
│   ├── Mix
│   ├── Pan / Width / Position
│   └── Output Topology
│
├── Media
│   ├── Buffer
│   ├── Sample
│   ├── Recording
│   ├── Playback
│   ├── Slicing
│   ├── Stretching
│   ├── Analysis
│   └── Resampling
│
├── Execution
│   ├── Semantic IR
│   ├── Lowering
│   ├── Runtime Graph
│   ├── Voice Allocation
│   ├── Scheduling
│   ├── Parameter Addressing
│   ├── Automation Engine
│   └── Real-time Control Plane
│
└── Authoring
    ├── Rust API
    ├── DSL / Macros
    ├── Presets
    ├── Project / Session
    ├── Live Coding Surface
    ├── Debug / Inspect
    ├── Testing / Rendering
    └── Pro / Private Features
```

---

# The layers that should exist

## 1. Runtime substrate

This is the current Auxide-style lower layer.

Purpose:

* deterministic audio execution
* DSP graph execution
* RT-safe control delivery
* audio IO
* MIDI IO

Rough crates:

* `auxide` — kernel/runtime
* `auxide-dsp` — DSP units
* `auxide-io` — hardware/audio bridge
* `auxide-midi` — MIDI/control bridge

This layer should be **implementation**, not user experience.

---

## 2. Semantic music layer

This is the missing layer the chat is really pointing toward.

Purpose:

* define music as typed Rust objects
* describe musical intent, not plumbing
* compile to the runtime substrate

Possible crate names:

* `auxide-music`
* `auxide-compose`
* `oxide-music`

This is where the real domain map lives.

---

## 3. Authoring / Pro layer

This sits above the semantic layer.

Purpose:

* live authoring
* session tooling
* higher-level workflows
* premium/private capabilities

Possible names:

* `auxide-pro`
* `oxide-pro`

This is where you put the advanced productized experience.

---

# The real domain objects

## Sound domain

These are the semantic objects for timbre and signal identity.

```rust
Instrument
Patch
VoiceModel
OscillatorKind
FilterModel
EnvelopeShape
Lfo
ModSource
EffectChain
Macro
```

Questions this domain answers:

* What kind of thing is making sound?
* How does its timbre behave?
* What parameters are musically meaningful?
* What controls should a performer see?

Important distinction:

* **Patch** = reusable sound definition
* **Voice** = live instance of that patch

That split is mandatory.

---

## Time domain

Time should not be “sample offsets everywhere.” It should be musical first.

```rust
Tempo
Meter
Beat
Bar
Subdivision
Clock
Transport
Phrase
Section
Arrangement
AutomationCurve
```

Questions this domain answers:

* When does something happen?
* How long is it in musical time?
* What repeats?
* What varies over bars or beats?

This is where SuperCollider-style clocks/tasks/patterns get replaced by something typed and legible.

---

## Performance domain

This domain represents interaction and real-time play.

```rust
NoteOn
NoteOff
Aftertouch
PitchBend
CcMessage
Gesture
Articulation
PerformanceState
Scene
ControllerMap
```

Questions:

* What did the player do?
* How is hardware mapped to musical intent?
* What is expressive vs structural?
* What is global vs per-voice?

This is where the MicroFreak-first workflow belongs.

---

## Structure domain

This is composition logic.

```rust
Note
Interval
Scale
Mode
Chord
Voicing
Progression
RhythmPattern
MelodicPhrase
Motif
Variation
GenerativeRule
```

Questions:

* What notes and harmonies exist?
* How are phrases organized?
* What should repeat, mutate, invert, transpose, humanize?

This is the layer that beats SC’s “cuneiform” feel if done well.

---

## Space domain

Users should not have to think in raw buses, but the system still needs spatial/routing semantics.

```rust
Route
Send
Return
MixBus
Group
Pan
StereoField
SpatialPosition
OutputTarget
```

Questions:

* Where does sound go?
* What shares effects?
* What is grouped?
* What reaches speakers, headphones, stems, recorder?

Expose the **meaning**, not the wire protocol.

---

## Media domain

This handles samples and recorded material.

```rust
BufferId
SampleAsset
Clip
Recorder
PlaybackMode
Slice
GranularRegion
AnalysisResult
```

Questions:

* What audio assets exist?
* How are they loaded, sliced, stretched, analyzed, replayed?

This should be first-class, but separate from pure synthesis.

---

## Execution domain

This is the compiler/runtime bridge.

```rust
SemanticProject
SemanticIr
LoweringPlan
RuntimePatch
RuntimeVoice
ParamAddress
AutomationEvent
SchedulerEvent
CompiledScene
```

Questions:

* How does high-level music become executable runtime objects?
* How are params addressed uniformly?
* How do semantic phrases become schedules and control events?

This is where the stack either becomes real or falls apart.

---

# The most important architectural rule

## Users author semantic intent

## The system lowers to execution detail

That means:

User writes:

* “play this phrase with this instrument”
* “open brightness over 8 bars”
* “make this monophonic with glide”
* “map knob 1 to filter brightness”
* “send snare to plate reverb”
* “humanize timing slightly”

The system handles:

* node graphs
* bus layout
* voice instantiation
* parameter transport
* control messages
* scheduling
* resource lifetime

That is the entire win.

---

# Rust shape for the semantic layer

A good model is:

```text
Authoring API
    ↓
Semantic AST / IR
    ↓
Normalization
    ↓
Lowering / Compilation
    ↓
Runtime Plan
    ↓
Auxide execution
```

So the semantic layer should have at least these subdomains:

```text
auxide-music
├── sound
├── time
├── performance
├── structure
├── space
├── media
├── ir
├── lower
└── api
```

---

# Minimal Rust type sketch

Not implementation, just shape:

```rust
pub struct Song {
    pub transport: TransportSpec,
    pub sections: Vec<Section>,
    pub scenes: Vec<Scene>,
}

pub struct Section {
    pub name: String,
    pub duration: MusicalDuration,
    pub phrases: Vec<PhraseBinding>,
}

pub struct PhraseBinding {
    pub instrument: InstrumentId,
    pub phrase: Phrase,
    pub articulation: ArticulationPlan,
    pub automation: Vec<AutomationLane>,
}

pub struct InstrumentDef {
    pub patch: PatchDef,
    pub performance: PerformanceModel,
    pub routing: RoutingPlan,
}

pub enum Phrase {
    Notes(Vec<NoteEvent>),
    Pattern(PatternDef),
    Generator(GeneratorRule),
}

pub struct AutomationLane {
    pub target: SemanticTarget,
    pub curve: Curve,
    pub span: TimeSpan,
}

pub enum SemanticTarget {
    Brightness,
    Timbre,
    Cutoff,
    Resonance,
    Macro(u16),
    SendLevel(String),
}
```

That is the right flavor: **musical targets first**, raw parameter IDs later.

---

# What belongs in the kernel vs semantic layer

## Kernel / runtime

Keep here:

* RT-safe graph execution
* stable control plane
* external node runtime control
* scheduling primitives
* param addressing
* voice lifecycle primitives
* routing primitives
* buffers
* automation execution

## Semantic layer

Put here:

* phrases
* patterns
* harmony
* articulation
* scenes
* sections
* arrangements
* controller meaning
* musical macros
* high-level patch semantics

The semantic layer should compile down into the kernel, not duplicate it.

---

# Backward map from current state

From the pasted chat, current reality is roughly:

## Already partly real

* runtime kernel
* DSP library
* IO bridge
* MIDI/control beginnings

## Not yet sufficient

* external DSP runtime control
* true synth-definition / synth-instance split
* stable live parameter model
* semantic composition layer
* scheduling/event model at music level
* bus/routing ergonomics
* sample/buffer workflow maturity

So the domain map is not just conceptual. It also implies the next dependency order.

---

# Dependency order

## Phase 1 — execution truth

Before anything pretty:

* runtime control for external DSP nodes
* stable param addressing
* voice lifecycle
* note/gate/release correctness
* IO integration correctness

## Phase 2 — playable semantics

Then:

* instrument definitions
* patch/voice split
* controller mapping by meaning
* phrase playback
* automation lanes
* routing model

## Phase 3 — music semantics

Then:

* harmony/rhythm/pattern domain
* sections/scenes/arrangements
* generative rules
* variations
* sample/media domain

## Phase 4 — authoring / pro

Then:

* live coding surface
* advanced orchestration tools
* premium/private workflow features

---

# The key insight from your chat

The correct target is **not**:

> “Rebuild SuperCollider in Rust exactly.”

The correct target is:

> “Capture SuperCollider’s power, but replace its user-facing chaos with a semantic music system in Rust.”

That means the domain map should preserve capability while radically improving clarity.

---

# Recommended crate map

```text
auxide            -> runtime engine
auxide-dsp        -> DSP building blocks
auxide-io         -> hardware/audio IO
auxide-midi       -> MIDI/control ingestion
auxide-music      -> semantic music model + IR + lowering
auxide-pro        -> private/pro authoring + advanced workflows
```

If you want the cleanest next move, I would turn this into a **formal domain catalog** with:

* entities
* relationships
* ownership boundaries
* crate placement
* “public vs private” split

or go one step more concrete and draft the first `auxide-music` module tree.