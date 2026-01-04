# Contributing to Auxide

Thank you for your interest in Auxide! 🎛️

## Open Source, Not Open Contribution

Auxide is **open source** but **not open contribution**.

- The code is available under the MIT license
- You can fork, modify, use, and learn from it freely
- **Pull requests are not accepted by default**
- Architectural, roadmap, and merge decisions are made by the project maintainer

This model keeps the project coherent, real-time safe, and deterministic without governance overhead.

## How to Propose Work

If you believe you can contribute meaningfully:

1. **Email the maintainer first** at michaelallenkuykendall@gmail.com
2. Describe your background and the specific change you propose
3. If aligned, a scoped collaboration may be arranged
4. Only after discussion will PRs be considered

**Unsolicited PRs will be closed without merge.**

## What We Welcome (after email alignment)
- Bug reports with clear reproduction steps (Issues are fine)
- Security vulnerability reports (email only)
- Documentation fixes (discuss first)
- RT-safety or determinism fixes (discuss first)

## What Is Maintainer-Only
- New features and API design
- Architectural changes
- Dependency updates
- Performance work and DSP kernels

## Code Style (if invited)
- Rust 2021; `cargo fmt` + `cargo clippy -- -D warnings`
- Tests for any behavior change; no regressions to determinism/RT-safety
- Public APIs must include examples

## Code of Conduct
See CODE_OF_CONDUCT.md. Be respectful and inclusive.