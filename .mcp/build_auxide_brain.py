#!/usr/bin/env python3
"""
Auxide Brain Builder
Builds a local SQLite source database for Chroma-backed MCP retrieval.
Indexes: auxide + auxide-dsp + auxide-io + auxide-midi repositories,
         local chat artifacts, and the saved Auxide multi-root workspace chats.
"""

import os
import sqlite3
import hashlib
import json
from pathlib import Path
from datetime import datetime
from urllib.parse import unquote

WORKSPACES = {
    "auxide": Path(r"C:\Users\micha\repos\auxide"),
    "auxide-dsp": Path(r"C:\Users\micha\repos\auxide-dsp"),
    "auxide-io": Path(r"C:\Users\micha\repos\auxide-io"),
    "auxide-midi": Path(r"C:\Users\micha\repos\auxide-midi"),
}

SHARED_WORKSPACE_PROJECT = "auxide-suite"
SHARED_WORKSPACE_FILE = Path(r"C:\Users\micha\repos\auxide-workspace.code-workspace")

DB_PATH = Path(__file__).parent / "auxide-brain.db"
VSCODE_STORAGE = Path(os.environ["APPDATA"]) / "Code" / "User" / "workspaceStorage"

EXCLUDE_DIRS = {
    ".git",
    "target",
    "node_modules",
    "__pycache__",
    ".venv",
    "venv",
    ".next",
    "dist",
    "build",
    "vendor",
    ".mcp",
    "chroma_db",
    "out",
    "coverage",
    ".nyc_output",
    "test-results",
    "ui",
}

SOURCE_EXTENSIONS = (".rs", ".toml", ".lock")

CHAT_NAME_HINTS = ("chat", "conversation", "session", "claude", "grok", "copilot")
OPS_NAME_HINTS = ("audio", "midi", "render", "realtime", "real-time", "benchmark", "latency", "buffer", "device", "plugin", "synth")
OPS_EXTENSIONS = (".json", ".jsonl", ".md", ".txt", ".log", ".sh", ".ps1", ".yaml", ".yml")


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8", errors="ignore")).hexdigest()


def initialize_database(path: Path) -> sqlite3.Connection:
    conn = sqlite3.connect(path)
    cursor = conn.cursor()

    cursor.execute(
        """
        CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_name TEXT NOT NULL,
            content TEXT NOT NULL,
            file_size INTEGER,
            last_modified TIMESTAMP,
            doc_hash TEXT UNIQUE,
            indexed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        """
    )

    cursor.execute(
        """
        CREATE TABLE IF NOT EXISTS chat_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project TEXT NOT NULL,
            source_file TEXT,
            session_id TEXT,
            session_title TEXT,
            content TEXT NOT NULL,
            message_count INTEGER,
            created_at TIMESTAMP,
            session_hash TEXT UNIQUE,
            indexed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        """
    )

    cursor.execute(
        """
        CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
            project, file_path, file_name, content,
            content=documents,
            content_rowid=id
        )
        """
    )

    cursor.execute(
        """
        CREATE VIRTUAL TABLE IF NOT EXISTS chats_fts USING fts5(
            project, session_title, content,
            content=chat_sessions,
            content_rowid=id
        )
        """
    )

    conn.commit()
    return conn


def scan_markdown_files(project_name: str, project_path: Path, conn: sqlite3.Connection) -> int:
    cursor = conn.cursor()
    indexed = 0

    for root, dirs, files in os.walk(project_path):
        dirs[:] = [d for d in dirs if d not in EXCLUDE_DIRS]

        for file_name in files:
            if not file_name.endswith(".md"):
                continue

            file_path = Path(root) / file_name
            try:
                content = file_path.read_text(encoding="utf-8", errors="ignore")
                if not content.strip():
                    continue

                doc_hash = sha256_text(content)
                rel_path = str(file_path.relative_to(project_path)).replace("\\", "/")
                stats = file_path.stat()
                last_modified = datetime.fromtimestamp(stats.st_mtime)

                cursor.execute("SELECT id FROM documents WHERE doc_hash = ?", (doc_hash,))
                if cursor.fetchone():
                    continue

                cursor.execute(
                    """
                    INSERT INTO documents (project, file_path, file_name, content, file_size, last_modified, doc_hash, indexed_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (project_name, rel_path, file_name, content, stats.st_size, last_modified, doc_hash, datetime.now()),
                )
                indexed += 1
            except Exception:
                continue

    conn.commit()
    return indexed


def scan_source_files(project_name: str, project_path: Path, conn: sqlite3.Connection) -> int:
    cursor = conn.cursor()
    indexed = 0

    for root, dirs, files in os.walk(project_path):
        dirs[:] = [d for d in dirs if d not in EXCLUDE_DIRS]

        for file_name in files:
            if not any(file_name.endswith(ext) for ext in SOURCE_EXTENSIONS):
                continue

            file_path = Path(root) / file_name
            try:
                content = file_path.read_text(encoding="utf-8", errors="ignore")
                if not content.strip():
                    continue

                doc_hash = sha256_text(content)
                rel_path = str(file_path.relative_to(project_path)).replace("\\", "/")
                stats = file_path.stat()
                last_modified = datetime.fromtimestamp(stats.st_mtime)

                cursor.execute("SELECT id FROM documents WHERE doc_hash = ?", (doc_hash,))
                if cursor.fetchone():
                    continue

                cursor.execute(
                    """
                    INSERT INTO documents (project, file_path, file_name, content, file_size, last_modified, doc_hash, indexed_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (project_name, rel_path, file_name, content, stats.st_size, last_modified, doc_hash, datetime.now()),
                )
                indexed += 1
            except Exception:
                continue

    conn.commit()
    return indexed


def scan_chat_like_markdown(project_name: str, project_path: Path, conn: sqlite3.Connection) -> int:
    cursor = conn.cursor()
    indexed = 0

    for root, dirs, files in os.walk(project_path):
        dirs[:] = [d for d in dirs if d not in EXCLUDE_DIRS]

        for file_name in files:
            lower = file_name.lower()
            if not lower.endswith((".md", ".txt", ".log", ".json")):
                continue
            if not any(hint in lower for hint in CHAT_NAME_HINTS):
                continue

            file_path = Path(root) / file_name
            try:
                content = file_path.read_text(encoding="utf-8", errors="ignore")
                if len(content.strip()) < 100:
                    continue

                digest = sha256_text(content)
                rel_path = str(file_path.relative_to(project_path)).replace("\\", "/")
                title = file_path.stem.replace("-", " ").replace("_", " ").strip().title()
                msg_count = max(1, content.count("\n\n"))

                cursor.execute("SELECT id FROM chat_sessions WHERE session_hash = ?", (digest,))
                if cursor.fetchone():
                    continue

                cursor.execute(
                    """
                    INSERT INTO chat_sessions (project, source_file, session_id, session_title, content, message_count, created_at, session_hash, indexed_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (project_name, rel_path, file_path.stem, title or file_path.stem, content, msg_count,
                     datetime.fromtimestamp(file_path.stat().st_mtime), digest, datetime.now()),
                )
                indexed += 1
            except Exception:
                continue

    conn.commit()
    return indexed


def scan_operational_artifacts(project_name: str, project_path: Path, conn: sqlite3.Connection) -> int:
    cursor = conn.cursor()
    indexed = 0

    explicit_targets = {
        ".vscode/tasks.json",
        ".vscode/settings.json",
        ".vscode/mcp.json",
    }

    for root, dirs, files in os.walk(project_path):
        dirs[:] = [d for d in dirs if d not in EXCLUDE_DIRS]

        for file_name in files:
            lower = file_name.lower()
            file_path = Path(root) / file_name
            rel_path = str(file_path.relative_to(project_path)).replace("\\", "/")
            rel_lower = rel_path.lower()

            include = rel_lower in explicit_targets
            if not include:
                if not lower.endswith(OPS_EXTENSIONS):
                    continue
                include = any(hint in lower or hint in rel_lower for hint in OPS_NAME_HINTS)
            if not include:
                continue

            try:
                content = file_path.read_text(encoding="utf-8", errors="ignore")
                if len(content.strip()) < 40:
                    continue

                doc_hash = sha256_text(content)
                stats = file_path.stat()
                last_modified = datetime.fromtimestamp(stats.st_mtime)

                cursor.execute("SELECT id FROM documents WHERE doc_hash = ?", (doc_hash,))
                if cursor.fetchone():
                    continue

                cursor.execute(
                    """
                    INSERT INTO documents (project, file_path, file_name, content, file_size, last_modified, doc_hash, indexed_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (project_name, rel_path, file_name, content, stats.st_size, last_modified, doc_hash, datetime.now()),
                )
                indexed += 1
            except Exception:
                continue

    conn.commit()
    return indexed


def find_workspace_ids(project_path: Path) -> list[str]:
    """Find ALL workspace storage IDs that match this project path (including subfolders)."""
    target = str(project_path).lower().replace("\\", "/")
    matches = []

    for workspace_dir in VSCODE_STORAGE.iterdir():
        if not workspace_dir.is_dir():
            continue
        workspace_json = workspace_dir / "workspace.json"
        if not workspace_json.exists():
            continue

        try:
            data = json.loads(workspace_json.read_text(encoding="utf-8", errors="ignore"))
            folder = data.get("folder", "")
            folder_norm = unquote(folder.replace("file:///", "").replace("%3a", ":").replace("%3A", ":")).lower().replace("\\", "/")
            # Exact match OR subfolder of project_path
            if folder_norm == target or folder_norm.startswith(target + "/"):
                matches.append(workspace_dir.name)
        except Exception:
            continue

    return matches


EXTRA_WORKSPACE_IDS = {
    "auxide-suite": ["afc9428c694d764bda85ff4070f8b887"],
}


def find_workspace_file_ids(workspace_file: Path) -> list[str]:
    target = str(workspace_file).lower().replace("\\", "/")
    matches = []

    for workspace_dir in VSCODE_STORAGE.iterdir():
        if not workspace_dir.is_dir():
            continue
        workspace_json = workspace_dir / "workspace.json"
        if not workspace_json.exists():
            continue

        try:
            data = json.loads(workspace_json.read_text(encoding="utf-8", errors="ignore"))
            workspace = data.get("workspace", "")
            workspace_norm = unquote(workspace.replace("file:///", "").replace("%3a", ":").replace("%3A", ":")).lower().replace("\\", "/")
            if workspace_norm == target:
                matches.append(workspace_dir.name)
        except Exception:
            continue

    return matches


def scan_workspace_id_list(project_name: str, workspace_ids: list[str], conn: sqlite3.Connection) -> int:
    cursor = conn.cursor()
    if not workspace_ids:
        return 0

    indexed = 0
    for workspace_id in workspace_ids:
        chat_dir = VSCODE_STORAGE / workspace_id / "chatSessions"
        if not chat_dir.exists():
            continue

        files = list(chat_dir.glob("*.json")) + list(chat_dir.glob("*.jsonl"))

        for file_path in files:
            try:
                if file_path.suffix == ".jsonl":
                    first_line = file_path.read_text(encoding="utf-8", errors="ignore").splitlines()
                    if not first_line:
                        continue
                    wrapper = json.loads(first_line[0])
                    data = wrapper.get("v", wrapper)
                else:
                    data = json.loads(file_path.read_text(encoding="utf-8", errors="ignore"))

                requests = data.get("requests", [])
                if not requests:
                    continue

                content = json.dumps(data, indent=2)
                digest = sha256_text(content)
                session_id = data.get("sessionId", file_path.stem)
                custom_title = data.get("customTitle", "")
                title = custom_title or session_id
                creation_date = data.get("creationDate", 0)
                created_at = datetime.fromtimestamp(creation_date / 1000) if creation_date else datetime.now()

                cursor.execute("SELECT id FROM chat_sessions WHERE session_hash = ?", (digest,))
                if cursor.fetchone():
                    continue

                cursor.execute(
                    """
                    INSERT INTO chat_sessions (project, source_file, session_id, session_title, content, message_count, created_at, session_hash, indexed_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (project_name, file_path.name, session_id, title, content, len(requests), created_at, digest, datetime.now()),
                )
                indexed += 1
            except Exception:
                continue

    conn.commit()
    return indexed


def scan_workspace_chats(project_name: str, project_path: Path, conn: sqlite3.Connection) -> int:
    workspace_ids = find_workspace_ids(project_path)
    for extra in EXTRA_WORKSPACE_IDS.get(project_name, []):
        if extra not in workspace_ids:
            workspace_ids.append(extra)
    return scan_workspace_id_list(project_name, workspace_ids, conn)


def scan_shared_workspace_chats(conn: sqlite3.Connection) -> int:
    workspace_ids = find_workspace_file_ids(SHARED_WORKSPACE_FILE)
    for extra in EXTRA_WORKSPACE_IDS.get(SHARED_WORKSPACE_PROJECT, []):
        if extra not in workspace_ids:
            workspace_ids.append(extra)
    return scan_workspace_id_list(SHARED_WORKSPACE_PROJECT, workspace_ids, conn)


def rebuild_fts_indexes(conn: sqlite3.Connection) -> None:
    cursor = conn.cursor()
    cursor.execute("INSERT INTO documents_fts(documents_fts) VALUES('rebuild')")
    cursor.execute("INSERT INTO chats_fts(chats_fts) VALUES('rebuild')")
    conn.commit()


def main() -> None:
    print("=" * 70)
    print("AUXIDE BRAIN BUILDER")
    print("=" * 70)

    if DB_PATH.exists():
        DB_PATH.unlink()
        print("Removed existing database")

    conn = initialize_database(DB_PATH)

    total_docs = 0
    total_chats = 0

    for project_name, project_path in WORKSPACES.items():
        if not project_path.exists():
            print(f"Skipping missing workspace: {project_name} ({project_path})")
            continue

        print(f"\nIndexing {project_name}...")
        docs = scan_markdown_files(project_name, project_path, conn)
        source_docs = scan_source_files(project_name, project_path, conn)
        ops_docs = scan_operational_artifacts(project_name, project_path, conn)
        chats_local = scan_chat_like_markdown(project_name, project_path, conn)
        chats_vscode = scan_workspace_chats(project_name, project_path, conn)

        total_docs += (docs + source_docs + ops_docs)
        total_chats += (chats_local + chats_vscode)

        print(f"  Documents indexed: {docs}")
        print(f"  Source files indexed: {source_docs}")
        print(f"  Operational artifacts indexed: {ops_docs}")
        print(f"  Chats indexed (local files): {chats_local}")
        print(f"  Chats indexed (VS Code): {chats_vscode}")

    shared_workspace_chats = scan_shared_workspace_chats(conn)
    total_chats += shared_workspace_chats
    print(f"\nShared workspace chats indexed ({SHARED_WORKSPACE_PROJECT}): {shared_workspace_chats}")

    rebuild_fts_indexes(conn)

    cursor = conn.cursor()
    cursor.execute("SELECT COUNT(*) FROM documents")
    doc_count = cursor.fetchone()[0]
    cursor.execute("SELECT COUNT(*) FROM chat_sessions")
    chat_count = cursor.fetchone()[0]

    conn.close()

    # Write Chroma index state file so the MCP server knows this DB is
    # current and does NOT re-index on startup (avoids 60s+ cold-load).
    index_state_path = DB_PATH.parent / "chroma_index_state.json"
    stat = DB_PATH.stat()
    index_state = {"db_size": stat.st_size, "db_mtime": int(stat.st_mtime)}
    index_state_path.write_text(json.dumps(index_state, indent=2), encoding="utf-8")
    print("Wrote chroma_index_state.json (MCP server will skip re-index)")

    print("\n" + "=" * 70)
    print("INDEX COMPLETE")
    print("=" * 70)
    print(f"Total documents: {doc_count}")
    print(f"Total chat sessions: {chat_count}")
    print(f"Database: {DB_PATH}")
    print(f"Database size: {DB_PATH.stat().st_size / (1024 * 1024):.2f} MB")

    # Auto-generate offline brain index files for .mcp/brain-index/
    import subprocess, shutil
    gen_script = DB_PATH.parent / "generate_brain_index.sh"
    if gen_script.exists():
        bash_path = shutil.which("bash")
        if bash_path:
            print("\nGenerating brain index files...")
            # Use relative paths from repo root so bash resolves them correctly
            repo_root = DB_PATH.parent.parent
            script_rel = str(gen_script.relative_to(repo_root)).replace("\\", "/")
            db_rel = str(DB_PATH.relative_to(repo_root)).replace("\\", "/")
            result = subprocess.run(
                [bash_path, script_rel, db_rel],
                capture_output=True, text=True,
                cwd=str(repo_root),
            )
            if result.returncode == 0:
                print(result.stdout)
            else:
                print(f"WARNING: Index generation failed: {result.stderr}")
        else:
            print("\nSkipping index generation (bash not found in PATH)")
    else:
        print(f"\nSkipping index generation ({gen_script} not found)")


if __name__ == "__main__":
    main()
