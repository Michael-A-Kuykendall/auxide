# FAQ

## What is Auxide?
A minimal, RT-safe audio graph kernel for building audio tools. It executes DSP nodes in a deterministic, block-based manner.

## Is it a DAW/plugin host?
No. It's a low-level kernel; you build hosts/tools on top of it.

## Can I mutate the graph at runtime?
No. Graph changes require recompiling the plan, which is not RT-safe.

## What about multichannel audio?
Not supported. Mono only for v0.1.

## How does it handle timing?
Block-based processing; assumes external clocking (e.g., from an audio backend).

## Is it deterministic?
Yes, for the same inputs and floating-point environment.

## What DSP nodes are included?
Minimal set: oscillators, gains, mixers, silence, output sink. Extend via the API.

## License?
MIT.</content>
<parameter name="filePath">c:/Users/micha/repos/auxide/docs/FAQ.md