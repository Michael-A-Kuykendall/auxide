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

## Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --status in_progress  # Claim work
bd close <id>         # Complete work
bd sync               # Sync with git
```

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

