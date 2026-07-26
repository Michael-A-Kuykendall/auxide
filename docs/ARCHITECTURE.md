# auxide Architecture

> Reconstructed 2026-07-26 from the bead plan (hlf/io/dsp/midi beads) and the
> current `auxide` crate, after the original "7-crate stack / 3-layer future /
> contracts" capture was lost. Treat as the durable source of truth; update via
> `bd`/PR, not by editing this prose in isolation.

## Repository layout (verified)

The system is **seven separate GitHub repositories** under
`github.com/Michael-A-Kuykendall/`, NOT one repo. This `auxide` repo is just
the **kernel crate** (`auxide`); `auxide-dsp` / `auxide-io` / `auxide-midi`
are their own repos, and `auxide-server` / `auxide-proto` / `auxide-conductor`
are to be created. Each is developed as a sibling directory and linked via Cargo
**path dependencies** (`auxide-dsp` already does `auxide = { path = "../auxide" }`).
There is **no `[workspace]` aggregator**; do not merge them.

➡️ **Authoritative map, clone commands, and dev-layout rules:**
[`docs/REPOSITORIES.md`](./REPOSITORIES.md). Read it before touching code.

## Current state (verified)

This `auxide` repo (kernel crate) is a **single crate** `auxide`
(Cargo.toml, no `[workspace]`). Its modules today: `control`, `dsl`, `graph`,
`invariant_ppt`, `invariant_rt`, `node`, `plan`, `rt`, `states`. The runtime
renders a `Graph` compiled to a `Plan` via `Runtime`/`render_offline`. The
other six crates live in their own repos (see above).

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

## Current state vs target

The repository is **currently a single crate** (`auxide`, Cargo.toml with no
`[workspace]`). The bead plan (hlf / io / dsp / midi) describes evolving it into
the 7-crate workspace above; the `auxide-hlf.*` beads *are* that work. This is
the plan, not an open decision — the foundation crates get built as those beads
are worked.
