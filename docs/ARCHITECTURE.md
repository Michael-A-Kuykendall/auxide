# auxide Architecture

> Reconstructed 2026-07-26 from the bead plan (hlf/io/dsp/midi beads) and the
> current `auxide` crate, after the original "7-crate stack / 3-layer future /
> contracts" capture was lost. Treat as the durable source of truth; update via
> `bd`/PR, not by editing this prose in isolation.

## Current state (verified)

The repository is a **single crate** `auxide` (Cargo.toml, no `[workspace]`).
Modules today: `control`, `dsl`, `graph`, `invariant_ppt`, `invariant_rt`,
`node`, `plan`, `rt`, `states`. The runtime renders a `Graph` compiled to a
`Plan` via `Runtime`/`render_offline`.

## Target: the 7-crate stack

The work plan (beads `auxide-hlf.*`, `auxide-dsp-*`, `auxide-io-*`,
`auxide-midi-*`) describes an intended **7-crate workspace**:

| Crate | Role | Beads |
|-------|------|-------|
| `auxide`        | Kernel: graph/plan/runtime + control contract | `auxide-hlf.2/.3/.4` |
| `auxide-dsp`    | DSP UGens (filters, FX, envelopes, PitchDetector) | `auxide-dsp-*` |
| `auxide-io`     | Audio device I/O (stream, error recovery, devices) | `auxide-io-*` |
| `auxide-midi`   | MIDI bridge + ROMpler/graph consumers | `auxide-midi-*` |
| `auxide-server` | Live, multi-instance, addressable node-graph server | `auxide-hlf.1` |
| `auxide-proto`  | Wire protocol + client (OSC + WebSocket codecs) | `auxide-hlf.5` |
| `auxide-conductor` | Composition / scheduling / transport | `auxide-hlf.6/.9/.10` |

## Three layers (future)

1. **Kernel layer** — `auxide` + `auxide-dsp` + `auxide-io` + `auxide-midi`:
   the deterministic, real-time-safe audio graph and its node libraries.
2. **Orchestration layer** — `auxide-server` + `auxide-conductor`: a live,
   addressable server exposing the graph, plus pattern/transport composition.
3. **Protocol/client layer** — `auxide-proto`: OSC + WebSocket codecs so
   external clients (and the conductor) drive the server.

## Contracts (stable interfaces)

- **Control contract** (`src/control.rs`): canonical `PARAM_*` indices are the
  single source of truth for runtime parameter routing — `PARAM_FREQUENCY=0`,
  `PARAM_CUTOFF=1`, `PARAM_RESONANCE=2`, `PARAM_WAVEFORM=3`, `PARAM_DETUNE=4`,
  `PARAM_PAN=5`. `ControlMsg` (SetFrequency/SetFilterCutoff/SetFilterResonance/
  SetWaveform/SetDetune/SetPan/TriggerGate/AllNotesOff/Mute/Reset) routes onto
  the matching `NodeDef` method via `RuntimeCore::apply_control_msg`. External
  nodes (e.g. `auxide-dsp`) are driven live through this contract.
- **Graph/Plan contract**: a `Graph` of `NodeType`s compiles to a `Plan`
  (topological order; cycles handled via `FeedbackNode` 1-block delay). The
  `Runtime` executes the plan; `render_offline` renders N frames for tests/demos.
- **Node contract**: `NodeDef::set_param(idx, v)` / `gate(...)` are the uniform
  control surface every node implements.

## Open question (see `auxide-aud.8`)

The beads assume the multi-crate workspace above. If the intended target is
actually a **single crate** (the current state), the `auxide-hlf.*` crate-
creation beads must be rewritten as single-crate modules. This is unresolved and
needs an explicit decision.
