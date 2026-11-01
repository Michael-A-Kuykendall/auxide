<div align="center">
  <img src="assets/auxide-logo.png" alt="Auxide Logo" width="400" height="auto" />

  # The RT-Safe Audio Graph Kernel for Rust

  ### Deterministic. Minimal. Real-Time Safe.

  [![Crates.io](https://img.shields.io/crates/v/auxide.svg)](https://crates.io/crates/auxide) [![Trans rights](https://pride-badges.pony.workers.dev/static/v1?label=trans%20rights&stripeWidth=6&stripeColors=5BCEFA,F5A9B8,FFFFFF,F5A9B8,5BCEFA)](https://translifeline.org/) [![LGBTQ+ friendly](https://pride-badges.pony.workers.dev/static/v1?label=lgbtq%2B%20friendly&stripeWidth=6&stripeColors=E40303,FF8C00,FFED00,008026,24408E,732982)](https://www.thetrevorproject.org/)
  [![Documentation](https://docs.rs/auxide/badge.svg)](https://docs.rs/auxide)
  [![CI](https://github.com/Michael-A-Kuykendall/auxide/workflows/CI/badge.svg)](https://github.com/Michael-A-Kuykendall/auxide/actions)
  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

  [![Sponsor this project](https://img.shields.io/badge/%E2%9D%A4%EF%B8%8F-Sponsor-ea4aaa?logo=github)](https://github.com/sponsors/Michael-A-Kuykendall)
</div>

**Auxide will be free forever.** No asterisks. No "free for now." No pivot to paid.

### Support

This project is independently maintained and free forever. Sponsorship helps fund ongoing development, certification, compatibility work, and releases.

[Sponsor on GitHub](https://github.com/sponsors/Michael-A-Kuykendall) · [Buy Me a Coffee](https://www.buymeacoffee.com/michaelakuykendall)

Any amount is appreciated. Suggested tiers:

- **$5/month** — Thanks
- **$15/month** — Supporter
- **$50/month** — Backer
- **Custom** — [Email for invoicing](mailto:michaelallenkuykendall@gmail.com)

---

## Table of Contents

- [What Is Auxide?](#what-is-auxide)
- [Auxide Ecosystem](#auxide-ecosystem)
- [Documentation](#documentation)
- [License](#license)

---

## What Is Auxide?

Auxide is a low-level, RT-safe kernel for executing audio graphs deterministically. Unlike full DAWs or plugin hosts, it's a programmable building block — think of it as the engine under the hood.

Audio processing as a **directed acyclic graph (DAG)**:
- **Nodes**: DSP units (oscillators, filters, effects).
- **Edges**: Signal flow between nodes.
- **Execution**: Topological order ensures no feedback loops.

| Feature | Auxide | SuperCollider | Rodio | CPAL |
|---------|--------|---------------|-------|------|
| RT-Safe | ✅ | ❌ | ❌ | ✅ |
| Graph-Based | ✅ | ✅ | ❌ | ❌ |
| Deterministic | ✅ | ❌ | ❌ | ❌ |
| Minimal API | ✅ | ❌ | ✅ | ✅ |
| Rust Native | ✅ | ❌ | ✅ | ✅ |

---

## Auxide Ecosystem

Auxide is an ecosystem of crates:

| Crate | Description | Version |
|-------|-------------|---------|
| **[auxide](https://github.com/Michael-A-Kuykendall/auxide)** | Real-time-safe audio graph kernel | 0.3.2 |
| [auxide-dsp](https://github.com/Michael-A-Kuykendall/auxide-dsp) | DSP nodes library | 0.2.1 |
| [auxide-io](https://github.com/Michael-A-Kuykendall/auxide-io) | Audio I/O layer | 0.1.3 |
| [auxide-midi](https://github.com/Michael-A-Kuykendall/auxide-midi) | MIDI integration | 0.1.2 |

---

## Documentation

- **[Quick Start](docs/QUICK_START.md)** — build your first audio graph.
- **[Advanced Examples](docs/ADVANCED_EXAMPLES.md)** — fan-out mixing, offline rendering, DSP integration.
- **[Architecture](docs/ARCHITECTURE.md)** — graph building, plan compilation, runtime execution, key invariants.
- **[Roadmap](ROADMAP.md)** — version history and future plans.
- **[Contributing](CONTRIBUTING.md)** — collaboration policy.

---

## License

MIT

## Support

This project is a safe space. Trans rights are human rights.

If you or someone you love needs support:

- [The Trevor Project](https://www.thetrevorproject.org/) — 24/7 for LGBTQ+ young people. Call 1-866-488-7386 or text START to 678-678
- [Trans Lifeline](https://translifeline.org/) — peer support run by and for trans people. US: 877-565-8860
- [988 Suicide & Crisis Lifeline](https://988lifeline.org/) — call or text 988
