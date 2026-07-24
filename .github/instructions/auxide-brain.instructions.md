---
applyTo: "**"
description: "Retrieval rules for the Auxide shared MCP brain"
---

# Auxide Brain

This workspace uses a shared MCP memory server (`auxide-brain`) hosted in `auxide/.mcp`.
It indexes 4 repos: auxide, auxide-dsp, auxide-io, and auxide-midi.

## When to Use

Before answering architecture, DSP, audio pipeline, MIDI, realtime, rendering, or historical questions:
1. Call `search_documents` and/or `search_chats` with specific terms.
2. Use `project` when scope is known: `auxide`, `auxide-dsp`, `auxide-io`, `auxide-midi`, or `auxide-suite`.
3. Use `get_document` or `get_chat` by ID for full content.

## Tools

`search_documents` `search_chats` `get_document` `get_chat` `get_stats` `list_recent_documents` `reindex`

## Shared Chats

Shared multi-root workspace chats are stored under `project: auxide-suite`.
Repo-local chats remain under their individual repo names.

## Offline Index

For the file-by-file catalog, read `.mcp/brain-index/brain-index-00-overview.instructions.md`.
Do not read the whole directory at once. Start with the overview, then open only the relevant sections.

## Reference

See `AUXIDE_BRAIN_HANDBOOK.md` for rebuild commands, storage layout, and workspace-only activation rules.