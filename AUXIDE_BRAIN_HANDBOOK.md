# Auxide Brain Handbook

## Purpose

This setup creates one shared MCP brain for the Auxide family:
- auxide
- auxide-dsp
- auxide-io
- auxide-midi

The shared brain lives in `auxide/.mcp`, but it is only auto-started by opening `C:/Users/micha/repos/auxide-workspace.code-workspace`.
Opening an individual repo does not auto-start the brain.

## Storage Layout

- Builder: `auxide/.mcp/build_auxide_brain.py`
- MCP server: `auxide/.mcp/auxide-brain-mcp.py`
- Source DB: `auxide/.mcp/auxide-brain.db`
- Chroma store: `auxide/.mcp/chroma_db`
- Offline catalog: `auxide/.mcp/brain-index/`
- Index state: `auxide/.mcp/chroma_index_state.json`

## Chat Sources

The builder ingests chats from two places:
- repo-local VS Code workspaceStorage entries for each repo
- the saved multi-root workspace file `auxide-workspace.code-workspace`

Shared multi-root chats are stored under `project = auxide-suite` so they are indexed once instead of being duplicated across all four repos.

## Rebuild

From `C:/Users/micha/repos/auxide` run:

```bash
./.mcp/build_auxide_brain.py
```

That rebuilds the SQLite source DB, refreshes Chroma state metadata, and regenerates the offline index files in `.mcp/brain-index/`.

## Retrieval

Use the MCP tools:
- `search_documents`
- `search_chats`
- `get_document`
- `get_chat`
- `get_stats`
- `list_recent_documents`
- `reindex`

Use `project: auxide-suite` when you specifically want shared multi-root workspace chats.

## Activation Rule

The MCP server is configured in `auxide-workspace.code-workspace`, not in any repo-local `.vscode/settings.json` file.
That keeps the shared brain attached to the saved multi-root workspace only.