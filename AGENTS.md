# Agent Instructions

## 🚫 NO SHORTCUTS — ZERO COMPROMISE

This project demands **the best possible work at all times**. Every acceptance criterion in every bead **must** be fully satisfied before a bead is closed — no deferrals, no "good enough", no follow-up tickets for work that should have been done now. If a bead's AC says "assert decays to ~0 within release", you implement a proper ADSR envelope with a release stage; you don't fake it with a short sample. If the AC says a test must exist, you write it properly. If the AC says clippy-clean, you make it so — then verify.

**Rules:**
1. **Plan → Execute → Verify.** Every step of the accepted plan must be executed, no skipping, no shortcutting, no asking "can we just…"
2. **Quality gates are non-negotiable.** `cargo test`, `cargo clippy -- -D warnings`, lint-as-you-go — run them every time, fix every issue, defer nothing.
3. **No "close and file follow-up"** unless the bead itself explicitly decomposes the work. If a bead's AC isn't met, the bead stays open until it is.
4. **If you think something is too hard or unnecessary, make a concrete engineering argument** — not a convenience argument. You must be able to justify every decision with evidence from the codebase.
5. **Perfection is the baseline.** The code must be correct, idiomatic, well-structured, and complete. No half-measures.

This project is built by someone who cares deeply about quality. Match that standard.

---

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## ⚠️ BAD USAGE — NEVER DO THESE

- **NEVER** read or grep `.beads/issues.jsonl` — the JSONL is a git export, not the source of truth. Use `bd show`/`bd list`/`bd graph`/`bd search` instead.
- **NEVER** touch `.beads/beads.db` or any `.beads/*.db` file directly — the SQLite DB is `bd`'s internal store. Use `bd` CLI for all access.
- **NEVER** use `bd ready` as work authority — it's bugged (parent-blocked heuristic hides foundation and surfaces unrelated). Use `bd graph <epic>` instead.
- **NEVER** trust `bd list --status <open|closed|blocked>` — it returns wrong/clamped sets (e.g., `--status blocked` returns nothing though beads are blocked; `--status closed` falsely lists open beads). Verify status with `bd show <id>`; list blocked with `bd blocked`.
- **NEVER** create a dependency cycle — verify with `bd dep cycles` after every dep change.
- **NEVER** embed full bead docs in AGENTS.md — use `bd onboard` + `bd prime` instead.

## Bead Usage — Correct Commands

### Querying (read-only, always use `bd`)

```bash
bd show <id>              # View full issue details (description, acceptance, design, notes, deps)
bd list                   # Show all open issues
bd list --status open     # Filter by status
bd list --type task       # Filter by type
bd list --label "io"      # Filter by label
bd list --priority 0      # Filter by priority (P0)
bd graph <epic>           # CANONICAL layering — Layer 0 = start here
bd search "text"          # Search issues by text
bd dep list <id>          # Show dependency info
bd dep tree <id>          # Show dependency tree
bd dep cycles             # Check for cycles
bd blocked                # All blocked issues
bd count                  # Issue counts
bd stats                  # Detailed statistics
bd info                   # Database + daemon info
bd lint                   # Template validation (gated AC, success criteria)
```

### Mutating (all through `bd`, never edit files)

```bash
bd create "Title" --type task --priority 2 --parent <epic> --description "..." --acceptance "..." --design "..."
bd q "Title"              # Quick capture, returns ID only (pipeable)
bd update <id> --title "..." --description "..." --acceptance "..." --design "..." --notes "..."
bd update <id> --status in_progress
bd update <id> --status blocked
bd update <id> --add-label "label"
bd close <id> -r "reason"
bd reopen <id>
bd dep <blocker> --blocks <blocked>    # Set dep (blocker first, --blocks blocked)
bd dep add <blocked> <blocker>         # Alternative: "a depends on b" (a is blocked, b is blocker)
bd dep remove <blocked> <blocker>      # Remove dep
bd sync --flush-only                   # After EVERY mutation, before EVERY commit
```

### Conventions (ID scheme)

- Feature beads: `auxide-<domain>-<code>` (domain ∈ dsp | io | midi | server | proto | conductor), e.g. `auxide-io-rfi`.
- Standalone work: `auxide-<code>` (e.g. `auxide-b7x`, `auxide-634`, `auxide-pl0`).
- Epics: `auxide-<name>` (e.g. `auxide-hlf`, `auxide-fxw`).
- NEVER create `aux-` prefixed duplicates of `auxide-` beads (seen: `aux-cpr`/`auxide-cpr`). Prefer the `auxide-` form.

### Anti-Patterns (enforced)

- `bd ready` is BUGGED — use `bd graph` for authoritative layering
- `bd list --status X` is UNRELIABLE — use `bd show <id>` for status and `bd blocked` for the blocked list
- `.beads/issues.jsonl` is a generated export that can be INCOMPLETE (the exporter drops some fields, e.g. `acceptance_criteria` for certain beads) — never read it as source of truth; always use `bd show`
- Before `bd create` from a transcript/session, **search the live db by title first** (`bd search "<title>"`) — recovered beads were duplicated by trusting transcript IDs without checking for an existing same-title bead
- `.beads/beads.db` is `bd`'s internal SQLite store — never touch it
- `opencode session list` is NOT a substitute for `bd` — beads hold the work plan, sessions are transcripts

## 🗺️ REPOSITORY LAYOUT (read before touching code)

Auxide is **seven separate GitHub repos** under `github.com/Michael-A-Kuykendall/`,
not one crate. This `auxide` repo is ONLY the **kernel crate** (`auxide`).
`auxide-dsp` / `auxide-io` / `auxide-midi` are their own repos;
`auxide-server` / `auxide-proto` / `auxide-conductor` are to be created.
They are developed as **sibling directories** and linked via Cargo **path
dependencies** (e.g. `auxide-dsp` does `auxide = { path = "../auxide" }`).
There is NO `[workspace]` aggregator — do NOT merge them into one crate.

A prior session wrongly collapsed everything into this one repo; that was a
mistake. Always work in the correct crate's repo.

➡️ **Authoritative map, clone commands, dependency direction, and go-live
policy:** `docs/REPOSITORIES.md` (in this repo). When a bead needs a crate
that has no repo yet, create the `*-private` repo per that doc.

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

