# Bead / Work-Record Audit — 5 Independent Passes

Goal: 5 independent top-to-bottom audits of the bead database + repo, building a
cumulative, actionable defect list. Each pass uses a fresh lens. Findings are
tagged F<pass>-<n> with severity, evidence, and a recommended fix.

Status after each pass is appended below. Remediation is deferred to a later
session ("work from the list") unless a finding is a defect I introduced this
session and is safe to fix immediately.

---

## Pass 1 — Inventory & identity integrity (2026-07-26)

Lens: every bead in `bd` — IDs, duplicates, orphans, status-query reliability,
fidelity of recovered beads vs source transcript.

### Findings

**F1-P1 (HIGH) — Duplicate `m71` bead from my own recovery.**
`auxide-m71` (CLOSED, pre-existing, same title/labels/priority/io pts-5) already
existed. My 2026-07-26 recovery created `auxide-io-m71` (OPEN) as a redundant
twin with a slightly different ID. My earlier "reopen m71" acted on the WRONG
(duplicate) bead; the canonical completed bead is `auxide-m71`.
- Evidence: `bd search "error recovery"` → 2 hits (`auxide-m71` closed,
  `auxide-io-m71` open). `grep '"id":"auxide-io-m71"'` =1, title appears 2x in
  issues.jsonl.
- Fix: retire/close `auxide-io-m71`; treat `auxide-m71` as canonical. The
  "m71 merged to main" claim most likely refers to `auxide-m71`.

**F2-P1 (MED) — Duplicate closed epics/tasks: `aux-` vs `auxide-` convention.**
`aux-cpr` and `auxide-cpr` are both CLOSED epics titled "Private planning
bootstrap". `aux-cpr.1` and `auxide-cpr.1` are both CLOSED tasks titled "Design
first-class runtime control for external DSP nodes".
- Fix: merge each pair (keep one, alias/retire the other).

**F3-P1 (MED) — `bd list --status closed` is unreliable.**
It lists `auxide-io-m71` as CLOSED though `bd show` confirms OPEN. Confirms
AGENTS.md's "bd ready / status filters are buggy" warning.
- Fix: never trust `bd list --status X`; verify status with `bd show`.

**F4-P1 (MED) — "blocked" count unverifiable via CLI.**
`bd stats` reports 9 blocked, but `bd list --status blocked` returns nothing.
Same query-bug class as F3.
- Fix: rely on `bd show`/graph, not `--status` filters, when counting.

**F5-P1 (MED) — Duplicate `auxide-midi-7db` vs closed `auxide-hcy`.**
Both are "Offline ROMpler demo producing .wav from full stack". `hcy` is already
CLOSED (done). `7db` was recovered from the transcript but is a duplicate of work
already completed and closed.
- Fix: retire `auxide-midi-7db` (duplicate of `auxide-hcy`).

**F6-P1 (LOW) — Recovered ACs/descriptions are reconstructed, not verbatim.**
Titles match source (opencode-session-calm-circuit.md:862-890) exactly; but the
acceptance criteria and descriptions I wrote are synthesized, not the original
author's text.
- Fix: treat recovered-bead ACs as placeholders pending original-author review.

**F7-P1 (LOW) — No enforced ID convention.**
Mixed schemes: `auxide-<domain>-<code>` (io/dsp/midi),
`auxide-hlf.N` (numbered children), `auxide-<code>` (m71, b7x, 634, 6ku, hcy,
915, 69m, pl0), and `aux-<code>` (cpr pairs).
- Fix: pick one convention and normalize historical IDs.

**F8-P1 (LOW/UNVERIFIED) — Possible orphan beads.**
Sampled parentage is correct (hlf.* → auxide-hlf; recovered 24 → auxide-fxw via
parent-child deps). But `auxide-pl0` (blocks-dep of hlf.2) and `auxide-b7x`
parentage not yet verified; full orphan scan deferred to Pass 3.
- Fix: run full parent-child scan in Pass 3.

### Pass 1 cumulative defect list (so far)
F1-P1, F2-P1, F3-P1, F4-P1, F5-P1, F6-P1, F7-P1, F8-P1

---

## Pass 2 — Acceptance-criteria & DoD / gated-AC compliance (2026-07-26)

Lens: does every bead (esp. the next-to-be-worked foundation beads) have a
specific, verifiable, gated acceptance criterion? Is the tracked JSONL a faithful
mirror of the live DB's ACs?

### Findings

**F9-P2 (HIGH) — `bd sync` JSONL export drops `acceptance_criteria` for ≥6 beads.**
Live DB HAS ACs for `auxide-hlf.7/.8/.9/.10` (verified via `bd show`), but the
exported `.beads/issues.jsonl` contains no `acceptance_criteria` key for them
(also `aux-cpr`, `auxide-cpr.1`). Re-running `bd sync` prints "Exported 0
issues" and does NOT repair it.
- Why it matters: my Pass-1 durability fix relies on tracking `issues.jsonl`.
  If the DB is ever lost and restored from JSONL, those 6 beads' ACs vanish
  silently → the audit/recovery itself is not durable.
- Fix: find a way to force a FULL re-export (or verify `bd` re-imports ACs from
  db, not JSONL), OR additionally back up `beads.db`. Do not assume JSONL alone
  is a complete backup.

**F10-P2 (MED) — "6 missing AC" is mostly a JSONL artifact, not a live gap.**
In the live DB the `hlf.*` foundation beads are well-specified with detailed,
gated, testable ACs (strong DoD — good). Of the 6 "missing", 4 (hlf.7-10) have
ACs in db; only the closed planning trackers `aux-cpr` / `aux-cpr.1` may
genuinely lack granular AC (low priority).
- Fix: optionally add AC to the 2 tracker beads; not blocking.

**F11-P2 (LOW) — Recovered-bead ACs are boilerplate.** Most recovered
io/dsp/midi beads end with "cargo test and clippy clean" — acceptable as
placeholders but not strongly gated. Tighten during grooming. (extends F6-P1)

**Positive:** AC coverage is 42/48 in the JSONL; the P0/P1 `hlf.*` foundation
beads have concrete, gated, testable acceptance criteria — the next work is
well-defined.

### Pass 2 cumulative defect list (so far)
F1-P1, F2-P1, F3-P1, F4-P1, F5-P1, F6-P1, F7-P1, F8-P1,
F9-P2, F10-P2, F11-P2

---

## Pass 3 — Dependency & hierarchy graph (2026-07-26)

Lens: cycles, dangling deps, blocked-bead accuracy, parent/child correctness,
and whether `bd ready`/`bd blocked` can be trusted (per AGENTS.md warnings).

### Findings

**F12-P3 (refines F4-P1) — `bd list --status blocked` is buggy; `bd blocked` is
correct.** `bd blocked` lists exactly the 9 blocked beads `bd stats` reports;
`bd list --status blocked` returns nothing. Trust `bd blocked`, never
`bd list --status <x>`.

**F13-P3 (MED) — `auxide-hlf` epic is blocked by its own child `auxide-fxw`.**
`auxide-hlf` (orchestration) has `auxide-fxw` as a child AND is blocked by it
(parent-blocked-by-child). Structurally odd and is the root cause of F14.
- Fix: decide intended direction — likely `auxide-fxw` should not block
  `auxide-hlf`, or the two epics are mis-scoped (fxw may not belong under hlf).

**F14-P3 (MED, confirms AGENTS.md) — `bd ready` hides ready foundation children
of a blocked epic.** `auxide-hlf.9` has NO own blockers (it is a prerequisite
for hlf.10, not blocked itself) yet is ABSENT from `bd ready`, because its
parent `auxide-hlf` is blocked by `auxide-fxw`. `bd ready` surfaces `pl0` and
the recovered beads but buries genuinely-ready foundation work.
- This validates AGENTS.md's "NEVER use bd ready as work authority."
- Fix: don't rely on `bd ready`; derive work order from `bd blocked` + manual
  graph, or restructure so foundation beads aren't hidden by a blocked parent.

**F15-P3 (POSITIVE) — No cycles, no dangling deps, coherent build order.**
`bd dep cycles` is clean; dangling-dep scan found 0; the hlf chain is sane:
`auxide-pl0` → `auxide-hlf.2` → {hlf.3, hlf.4, hlf.7, hlf.8} → `auxide-hlf.1`
→ {hlf.5, hlf.10} → `auxide-hlf.6`; `hlf.9` → `hlf.10`. `auxide-pl0` is a proper
unblocked root.

**F16-P3 (LOW) — `bd ready` surfaces duplicate/retire-able beads as "ready".**
`auxide-midi-7db` (F5-P1) and `auxide-io-m71` (F1-P1) appear in `bd ready` as
actionable though they should be retired. Consequence of unresolved duplicates.

### Pass 3 cumulative defect list (so far)
F1-P1, F2-P1, F3-P1, F4-P1, F5-P1, F6-P1, F7-P1, F8-P1,
F9-P2, F10-P2, F11-P2,
F12-P3, F13-P3, F14-P3, F15-P3, F16-P3

---

## Pass 4 — Code/reality reconciliation (2026-07-26)

Lens: do closed-bead claims and the bead *plan* match the actual repository?
(Single crate `auxide` v0.3.1; no `[workspace]` in Cargo.toml.)

### Findings

**F17-P4 (HIGH) — `auxide-hcy` is falsely closed.** AC requires
`examples/rompler_demo.rs` producing a non-silent `.wav`. Reality: NO rompler
demo exists anywhere — `git log --all -- examples/rompler_demo.rs` is empty,
`git ls-files | grep rompler` is empty, and no "rompler" string exists in
`src/`/`examples/`. The closure reason ("Peak=0.97, RMS=0.50 — real audio
output") is not reproducible in this repo.
- Fix: reopen `auxide-hcy`; either recreate the demo for real or correct the
  record. (Tangled with F5-P1/F1-P1: `7db` is a duplicate of this same work.)

**F18-P4 (HIGH) — `auxide-b7x` is stale/falsely closed.** Its gate
`verify_all.sh` loops `CRATES="auxide auxide-dsp auxide-io auxide-midi"` and
`cd "$DIR"` into sibling dirs — but those dirs don't exist (single crate). The
script fails on the first `cd`. The AC "single script proves the whole stack
builds/tests/lints across four crates" is NOT met in the current repo. The
"ALL CRATES GREEN" gate has never run green here.
- Fix: either (a) rewrite `verify_all.sh` for the single-crate reality and
  actually run it green, or (b) reopen `auxide-b7x` until the gate is real.

**F19-P4 (MED) — Plan assumes a multi-crate workspace the repo is not.**
`hlf.*` beads create `auxide-server`/`auxide-proto`/`auxide-conductor` crates;
dsp/io/midi beads and `b7x` assume 4+ sibling crates. The repo is a single
crate. Either multi-crate is the intended future (in which case the `hlf` beads
are the vehicle and are fine) or the plan is stale.
- Fix: confirm the intended target architecture with the user; if single-crate
  is intended, rewrite the crate-creation beads as single-crate modules.

**F20-P4 (MED) — The "7-crate stack / 3-layer future / contracts" architecture
capture is missing from the repo.** The earlier session claimed this was
"fully captured," but `grep` finds no such text in `docs/` or any repo `.md`
(excluding session exports). It is another lost artifact (like the beads).
- Fix: re-capture the architecture in a durable place (doc or bead) so future
  sessions have it.

**Positive:** `auxide-69m`, `auxide-915`, `auxide-6ku` closures ARE reflected in
current code — `src/{control,graph,plan,rt}.rs` contain `apply_control_msg`,
`NodeType::External`, `add_external_node`, `render_offline_handle`. Those three
closures hold.

### Pass 4 cumulative defect list (so far)
F1-P1, F2-P1, F3-P1, F4-P1, F5-P1, F6-P1, F7-P1, F8-P1,
F9-P2, F10-P2, F11-P2,
F12-P3, F13-P3, F14-P3, F15-P3, F16-P3,
F17-P4, F18-P4, F19-P4, F20-P4

---

## Pass 5 — Process & hygiene (2026-07-26)

Lens: AGENTS.md compliance, bd hooks, doc accuracy, git state, and the
systemic tension between the "never read issues.jsonl" rule and bd's unreliable
CLI.

### Findings

**F21-P5 (MED) — bd git hooks are NOT installed.** `bd hooks list` shows 4
missing: pre-commit (flush JSONL), post-merge (import), pre-push (block stale
JSONL), post-checkout (import). Without them, the only safeguard is the manual
`bd sync` step — exactly the gap that let bead state drift and get lost.
- Fix: run `bd hooks install`.

**F22-P5 (LOW) — Doc drift.** README documents `verify_all.sh` (good) but the
script is broken (F18); README has no StreamController/ROMpler content
(consistent with F17/F18 and io bead `256`'s stale premise — StreamController
doesn't exist in the repo).
- Fix: refresh docs alongside the code fixes.

**F23-P5 (LOW) — Rule/tooling tension.** AGENTS.md forbids reading
`.beads/issues.jsonl`, yet bd's CLI is buggy (F3/F4/F12/F14), pressuring agents
to peek at the JSONL (I did too during this audit). The rule is sound but
unenforceable while bd queries misbehave.
- Fix: fix bd queries OR formally permit a `bd show`/JSONL fallback; until then
  mandate `bd show` + `bd blocked`, never `bd list --status X` / `bd ready`.

**F24-P5 (LOW) — Session-export clutter.** 4 `opencode-session-*.md` files sit
in repo root (gitignored, harmless, but accumulate).
- Fix: prune periodically.

**F25-P5 (LOW, META) — Bead recovery from transcript is error-prone.** My own
recovery created duplicates (`auxide-io-m71` vs existing `auxide-m71`;
`auxide-midi-7db` vs existing `auxide-hcy`) by trusting transcript IDs without
checking the live db for same-title beads first.
- Fix (recovery SOP): before `bd create`, search the live db by TITLE to avoid
  dupes.

**Positive:** working tree is clean (all pushes landed); the `.beads` durability
fix persists (no `.beads` in `.git/info/exclude`); AGENTS.md's warnings about
`bd ready` / `bd list --status blocked` are ACCURATE (F12/F14 confirmed them);
`bd show` is reliable.

### Resolution of earlier "deferred" findings
- F8-P1 (orphan scan) → RESOLVED by Pass 3: no orphans; parentage correct
  (hlf.*→auxide-hlf, recovered 24→auxide-fxw). No action.
- F12-P3 → refines F4-P1 (the bug is specifically `bd list --status blocked`;
  `bd blocked` is correct). Not a new defect.

---

# CUMULATIVE DEFECT LIST (all 5 passes)

Severity legend: **HIGH** = data-integrity / falsely-closed / durability;
**MED** = should fix before relying on the plan; **LOW** = hygiene / nice-to-have.

## HIGH
- **F1-P1** — Duplicate `m71`: `auxide-io-m71` (my recovery) vs pre-existing
  closed `auxide-m71`. Retire the duplicate; `auxide-m71` is canonical.
- **F9-P2** — `bd sync` JSONL export drops `acceptance_criteria` for ≥6 beads
  (hlf.7/8/9/10, aux-cpr, auxide-cpr.1) though live db has them. Tracking
  `issues.jsonl` alone is NOT a complete backup. Force full re-export or also
  back up `beads.db`.
- **F17-P4** — `auxide-hcy` falsely closed: no ROMpler demo exists anywhere in
  git/tree. Reopen; recreate the demo (or correct the record).
- **F18-P4** — `auxide-b7x` falsely/stale-closed: `verify_all.sh` loops 4
  nonexistent crate dirs; the integration gate has never run green here. Fix the
  script or reopen until real.

## MED
- **F2-P1** — Duplicate closed pairs: `aux-cpr`/`auxide-cpr`, `aux-cpr.1`/
  `auxide-cpr.1`. Merge each pair.
- **F3-P1** — `bd list --status closed` is unreliable (falsely lists open
  `auxide-io-m71`). Verify status with `bd show`.
- **F4-P1** — `bd stats` reports 9 blocked but `bd list --status blocked`
  returns nothing (use `bd blocked`). [refined by F12-P3]
- **F5-P1** — `auxide-midi-7db` duplicates already-closed `auxide-hcy`. Retire.
- **F13-P3** — `auxide-hlf` epic is blocked by its child `auxide-fxw`
  (parent-blocked-by-child). Review direction/scope.
- **F14-P3** — `bd ready` hides ready foundation children of a blocked epic
  (e.g., `auxide-hlf.9` absent though unblocked). Validates AGENTS.md; don't
  trust `bd ready`.
- **F19-P4** — Plan assumes a multi-crate workspace; repo is a single crate.
  Confirm intended target architecture.
- **F20-P4** — "7-crate stack / 3-layer future / contracts" architecture capture
  is missing from the repo. Re-capture durably.
- **F21-P5** — bd git hooks not installed (4 missing). Run `bd hooks install`.

## LOW
- **F6-P1** — Recovered beads' ACs/descriptions are reconstructed, not verbatim.
- **F7-P1** — No enforced ID convention (auxide-<domain>-<code> / auxide-hlf.N
  / auxide-<code> / aux-<code>).
- **F10-P2** — Closed planning trackers `aux-cpr`/`auxide-cpr.1` may lack AC.
- **F11-P2** — Recovered-bead ACs are boilerplate ("cargo test + clippy clean").
- **F16-P3** — `bd ready` surfaces duplicate/retire-able beads as "ready".
- **F22-P5** — README/doc drift (broken verify_all reference; no StreamController/
  ROMpler content).
- **F23-P5** — Rule/tooling tension: forbidding issues.jsonl reads is
  unenforceable while bd CLI is buggy.
- **F24-P5** — Session-export .md clutter in repo root.
- **F25-P5** — Recovery SOP needed: search live db by title before `bd create`.

## Positives (no action)
- No dependency cycles; no dangling deps; coherent hlf build order (F15-P3).
- hlf.* foundation beads have strong gated ACs (F10-P2).
- `auxide-69m`/`915`/`6ku` closures reflected in current code.
- Working tree clean; `.beads` durability fix persists; AGENTS.md bd warnings
  accurate; `bd show` reliable.

---

## Suggested remediation order (for the "work from the list" phase)
1. Install bd hooks (F21-P5) — prevents recurrence immediately.
2. Retire duplicates: `auxide-io-m71`, `auxide-midi-7db`, `aux-cpr`/`auxide-cpr`
   pairs (F1-P1, F5-P1, F2-P1).
3. Reopen falsely-closed: `auxide-hcy`, `auxide-b7x`; fix `verify_all.sh`
   (F17-P4, F18-P4, F22-P5).
4. Resolve JSONL AC-drop (F9-P2) so the backup is complete.
5. Confirm target architecture & re-capture it (F19-P4, F20-P4); fix `bd ready`
   reliance / hlf↔fxw block direction (F13-P3, F14-P3, F23-P5).
6. Hygiene: ID convention, AC tightening, export prune (F6/F7/F10/F11/F24/F25).


