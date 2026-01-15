# Release Notes

## 0.2.2 (2026-01-15)
- RT control plane split (`RuntimeCore`/`RuntimeControl`/`RuntimeHandle`) with lock-free SPSC queues.
- RT invariant signaling queue and contract tests for buffer fill, control delivery, mute/reset.
- Dual licensing: MIT OR Apache-2.0 with `LICENSE-MIT` and `LICENSE-APACHE`.
- Documentation cleanup (architecture, invariants) and removal of stray artifacts.

## 0.2.1 (2026-01-07)
- Phase accumulation and error-handling fixes.
- Expanded module docs and RT safety verification (no allocations in hot paths).
- Compatibility verified with auxide-dsp 0.1.1, auxide-io 0.1.2, auxide-midi 0.1.1.

## 0.2.0 (2026-01-04)
- Added external node hook (`NodeType::External`) and object-safe `NodeDef` adapter.
- Preallocated external node state and routes; static metadata to avoid RT allocations.
- Preserved backward compatibility for existing graphs.

## 0.1.1 (2026-01-03)
- `render_offline` now propagates `process_block` errors.
- `get_node_by_name` added to `GraphBuilder`.
- Clearer cycle detection assertion in PPT mode and minor polish.

## 0.1.0 (2026-01-03)
- Initial release: Graph → Plan → Runtime architecture, RT-safe processing, deterministic execution, single-writer/no-cycle invariants, minimal DSP nodes, PPT system, and correctness hardening.
