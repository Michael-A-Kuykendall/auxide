# GitHub Copilot Instructions for Auxide

## Auxide Shared Brain

This repo participates in a shared MCP memory server named `auxide-brain`.
The server is hosted in `auxide/.mcp` and indexes:
- `auxide`
- `auxide-dsp`
- `auxide-io`
- `auxide-midi`

The shared brain is intended to auto-start only when opening the saved multi-root workspace:
- `C:/Users/micha/repos/auxide-workspace.code-workspace`

Do not assume the brain is active when this repo is opened by itself.

## Retrieval Flow

Before answering architecture, DSP, audio pipeline, MIDI, realtime, rendering, or historical questions:
1. Call `search_documents` and/or `search_chats` with specific terms.
2. Use `project` when scope is known: `auxide`, `auxide-dsp`, `auxide-io`, `auxide-midi`, or `auxide-suite`.
3. Use `get_document` or `get_chat` by ID for full content.

## Shared Chats

Shared multi-root workspace chats are stored under `project: auxide-suite`.
Repo-local chats remain under their individual repo names.

## Tools

`search_documents` `search_chats` `get_document` `get_chat` `get_stats` `list_recent_documents` `reindex`

## Offline Catalog

For the file-by-file catalog, read `.mcp/brain-index/brain-index-00-overview.instructions.md`.
Do not read the entire catalog directory at once. Start with the overview and then open only the relevant sections.

## Reference

See `AUXIDE_BRAIN_HANDBOOK.md` for rebuild commands, storage layout, and workspace-only activation rules.