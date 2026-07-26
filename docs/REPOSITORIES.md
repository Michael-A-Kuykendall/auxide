# Auxide Repository Layout

> Authoritative map of where every Auxide crate lives. **New agents MUST read
> this before touching code.** The 7-crate system is a set of **separate
> GitHub repositories** under one org — NOT subdirectories of a single repo. A
> prior session wrongly collapsed everything into one crate; do not repeat that
> mistake. Each crate is its own repo, cloned as a sibling, and linked in dev
> via Cargo path dependencies.

## Org

All repositories live under **`github.com/Michael-A-Kuykendall/`**. The single
`auxide` (kernel) repo you are currently in is just ONE of the seven.

## The seven crates

| Crate (Cargo `name`) | Role | Private dev repo | Public repo (stale) | Status |
|-----------------------|------|------------------|----------------------|--------|
| `auxide`        | Kernel: graph/plan/runtime + control contract | `auxide-private`        | `auxide`        | exists |
| `auxide-dsp`    | DSP UGens (osc/filters/FX/env/mod) | `auxide-dsp-private`    | `auxide-dsp`    | exists |
| `auxide-io`     | Audio device I/O (stream, recovery, devices) | `auxide-io-private`  | `auxide-io`     | exists |
| `auxide-midi`   | MIDI bridge + ROMpler/graph consumers | `auxide-midi-private`  | `auxide-midi`   | exists |
| `auxide-server` | Live, multi-instance, addressable node-graph server | `auxide-server-private` | `auxide-server` (go-live) | **private created** (2026-07-26) |
| `auxide-proto`  | Wire protocol + client (OSC + WebSocket codecs) | `auxide-proto-private` | `auxide-proto` (go-live) | **private created** (2026-07-26) |
| `auxide-conductor` | Composition / scheduling / transport | `auxide-conductor-private` | `auxide-conductor` (go-live) | **private created** (2026-07-26) |

### Dependency direction (who depends on whom)

```
auxide  ──▶  (kernel, depends on nothing internal)
  ▲  ▲  ▲
  │  │  └── auxide-midi
  │  └───── auxide-io
  └──────── auxide-dsp
                │
                ▼
          auxide-server   (depends on auxide + dsp + io + midi)
                │
                ▼
          auxide-conductor (depends on auxide-server)
                │
                ▼
          auxide-proto  (codecs for server/conductor messages; depends on auxide-server)
```

`auxide` is the root dependency. Everything else points at it (in dev, via
`path = "../auxide"`). Nothing points "up" toward the kernel.

## Accessing the repos from this workspace

The `GITHUB_TOKEN` environment variable is already configured in this workspace,
so `gh` and `git` over HTTPS work without further auth.

```bash
# clone any sibling repo into the SAME parent directory as this one
gh repo clone Michael-A-Kuykendall/auxide-dsp-private
gh repo clone Michael-A-Kuykendall/auxide-io-private
gh repo clone Michael-A-Kuykendall/auxide-midi-private
```

This repo (`auxide-private`) already has:
- `origin`  → `https://github.com/Michael-A-Kuykendall/auxide-private.git`
- `public`  → `https://github.com/Michael-A-Kuykendall/auxide.git`

When you clone the others, set the same two-remote convention:
```bash
cd auxide-dsp-private
git remote add public https://github.com/Michael-A-Kuykendall/auxide-dsp.git
```

## Local development layout (path dependencies)

The dev tree is a set of **sibling directories** under one parent (e.g.
`C:\Users\micha\repos`):

```
repos/
  auxide/            (this repo — kernel crate, `auxide`)
  auxide-dsp/        (or auxide-dsp-private)
  auxide-io/
  auxide-midi/
  auxide-server/     (to be created)
  auxide-proto/      (to be created)
  auxide-conductor/  (to be created)
```

Crates depend on the kernel by **relative path**, not by published version,
while developing:

```toml
# auxide-dsp/Cargo.toml  (verified real example)
[dependencies]
auxide = { path = "../auxide" }
```

So `auxide-server` will declare, in its `Cargo.toml`:
```toml
[dependencies]
auxide     = { path = "../auxide" }
auxide-dsp = { path = "../auxide-dsp" }
auxide-io  = { path = "../auxide-io" }
auxide-midi = { path = "../auxide-midi" }
```

Each crate is currently a **standalone package** (own `Cargo.toml`,
`cargo test`/`clippy` run per-crate). There is **no `[workspace]` aggregator
checked in** — they are linked only through path deps. Do not "helpfully"
merge them into one crate or one workspace member list unless a bead explicitly
asks.

## Creating the three new repos (done — pattern for reference)

`auxide-server`, `auxide-proto`, `auxide-conductor` were created as **private
dev repos** on 2026-07-26 (`auxide-server-private`, `auxide-proto-private`,
`auxide-conductor-private`). Their public mirrors (`auxide-server`,
`auxide-proto`, `auxide-conductor`) are created only at the **go-live gate**
(see AGENTS.md): when that crate's scheduled bead work is closed + green
(fmt/clippy/test) + functional review.

For reference, the creation command used was:

```bash
gh repo create Michael-A-Kuykendall/auxide-server-private --private \
  --description "Private development mirror for auxide-server"
gh repo clone Michael-A-Kuykendall/auxide-server-private auxide-server
# scaffold Cargo.toml with path deps to the siblings above, then implement.
# add the public mirror as a second remote for the eventual push:
git -C auxide-server remote add public https://github.com/Michael-A-Kuykendall/auxide-server.git
```

Each new repo was scaffolded green (`cargo build` + `cargo clippy` clean) with
the path deps it needs (server → `auxide` + `auxide-dsp`; conductor →
`auxide` + `auxide-server`; proto → `auxide`), and pushed to its `origin`
(private). Clone the existing private siblings the same way to get a full local
dev tree (sibling directories named by crate, not by repo suffix).

## Go-live / public-sync policy (reminder)

- Public repos (`auxide`, `auxide-dsp`, `auxide-io`, `auxide-midi`) are
  currently **stale** (private is ahead: kernel 18, dsp 19, io 7, midi 15).
- Push `origin → public` only when a crate's scheduled bead work is **closed +
  green + reviewed**. No auto-push.
- Keep the `-private` repo as the day-to-day working remote; the bare public
  repo is a release mirror.

## Quick orientation for a new agent

1. Read `docs/ARCHITECTURE.md` for the three-layer design + contracts.
2. Read this file — know which repo holds the code you are about to touch.
3. Clone the relevant sibling repo(s) into the parent dir; never assume the
   code you need is in this `auxide` repo.
4. Build/test the crate you are in; cross-crate changes go in their own repos
   and are wired via path deps.
