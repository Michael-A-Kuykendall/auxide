#!/usr/bin/env python3
"""
Auxide Brain MCP Server (Chroma-backed search)
Serves: auxide + auxide-dsp + auxide-io + auxide-midi knowledge
"""

import json
import math
import re
import sqlite3
import subprocess
import sys
import hashlib
import shutil
from datetime import datetime
from pathlib import Path
from typing import Any

import chromadb

DATABASE_PATH = Path(__file__).parent / "auxide-brain.db"
CHROMA_PATH = Path(__file__).parent / "chroma_db"
COLLECTION_NAME = "auxide_brain"
INDEX_STATE_PATH = Path(__file__).parent / "chroma_index_state.json"
VECTOR_DIM = 256


def tokenize(text: str) -> list[str]:
    return re.findall(r"[a-zA-Z0-9_]+", (text or "").lower())


def embed_text(text: str, dim: int = VECTOR_DIM) -> list[float]:
    vec = [0.0] * dim
    tokens = tokenize(text)
    if not tokens:
        return vec

    for token in tokens:
        digest = hashlib.sha256(token.encode("utf-8")).digest()
        index = int.from_bytes(digest[:4], "little") % dim
        sign = 1.0 if (digest[4] % 2 == 0) else -1.0
        vec[index] += sign

    norm = math.sqrt(sum(v * v for v in vec))
    if norm > 0:
        vec = [v / norm for v in vec]
    return vec


def chunk_text(text: str, size: int = 900, overlap: int = 150) -> list[str]:
    clean = (text or "").strip()
    if not clean:
        return []
    if len(clean) <= size:
        return [clean]

    chunks = []
    start = 0
    step = max(100, size - overlap)
    while start < len(clean):
        end = min(len(clean), start + size)
        chunk = clean[start:end].strip()
        if chunk:
            chunks.append(chunk)
        if end >= len(clean):
            break
        start += step
    return chunks


def extract_chat_text(raw_content: str) -> str:
    text_parts = []
    try:
        data = json.loads(raw_content)
        requests = data.get("requests", [])
        for req in requests:
            message = req.get("message", {})
            if isinstance(message, dict):
                for part in message.get("parts", []):
                    if isinstance(part, dict) and part.get("text"):
                        text_parts.append(part.get("text"))
                msg_text = message.get("text", "") or message.get("content", "")
                if isinstance(msg_text, str) and msg_text:
                    text_parts.append(msg_text)
            elif isinstance(message, str) and message:
                text_parts.append(message)

            response = req.get("response", {})
            if isinstance(response, dict):
                for part in response.get("parts", []):
                    if isinstance(part, dict) and part.get("text"):
                        text_parts.append(part.get("text"))
                rsp_text = response.get("text", "") or response.get("content", "")
                if isinstance(rsp_text, str) and rsp_text:
                    text_parts.append(rsp_text)
            elif isinstance(response, str) and response:
                text_parts.append(response)
    except Exception:
        pass

    if not text_parts:
        return raw_content
    return "\n".join(text_parts)


class AuxideBrainServer:
    def __init__(self, db_path: Path):
        self.db_path = db_path
        self.conn = sqlite3.connect(str(db_path), timeout=30)
        self.conn.row_factory = sqlite3.Row

        self.chroma, self.collection = self._initialize_chroma()

        self._index_ready = False

    def _create_chroma_client(self) -> tuple[Any, Any]:
        chroma = chromadb.PersistentClient(path=str(CHROMA_PATH))
        collection = chroma.get_or_create_collection(name=COLLECTION_NAME)
        return chroma, collection

    def _create_ephemeral_chroma_client(self) -> tuple[Any, Any]:
        chroma = chromadb.Client()
        collection = chroma.get_or_create_collection(name=COLLECTION_NAME)
        return chroma, collection

    def _probe_persistent_store(self) -> tuple[bool, str]:
        if not CHROMA_PATH.exists():
            return True, ""

        probe = subprocess.run(
            [
                sys.executable,
                "-c",
                (
                    "import chromadb, sys; "
                    "chromadb.PersistentClient(path=sys.argv[1]); "
                    "print('ok')"
                ),
                str(CHROMA_PATH),
            ],
            capture_output=True,
            text=True,
            timeout=30,
        )
        if probe.returncode == 0:
            return True, ""

        details = (probe.stderr or probe.stdout or "Chroma store probe failed").strip()
        return False, details

    def _quarantine_chroma_store(self, reason: BaseException) -> None:
        timestamp = datetime.now().strftime("%Y%m%d-%H%M%S")
        backup_path = CHROMA_PATH.parent / f"{CHROMA_PATH.name}.corrupt-{timestamp}"

        if CHROMA_PATH.exists():
            try:
                CHROMA_PATH.rename(backup_path)
            except Exception:
                shutil.rmtree(CHROMA_PATH, ignore_errors=True)

        if INDEX_STATE_PATH.exists():
            try:
                INDEX_STATE_PATH.unlink()
            except Exception:
                pass

        print(
            f"Recovered from Chroma startup failure by resetting persisted state: {reason}",
            file=sys.stderr,
        )

    def _initialize_chroma(self) -> tuple[Any, Any]:
        probe_ok, probe_message = self._probe_persistent_store()
        if not probe_ok:
            self._quarantine_chroma_store(RuntimeError(probe_message))

        try:
            return self._create_chroma_client()
        except BaseException as exc:
            print(
                f"Persistent Chroma unavailable, falling back to in-memory index: {exc}",
                file=sys.stderr,
            )
            return self._create_ephemeral_chroma_client()

    def _lazy_ensure_index(self) -> None:
        if self._index_ready:
            return
        self._ensure_chroma_index()
        self._index_ready = True

    def _current_db_state(self) -> dict[str, Any]:
        stat = self.db_path.stat()
        return {"db_size": stat.st_size, "db_mtime": int(stat.st_mtime)}

    def _read_index_state(self) -> dict[str, Any]:
        if not INDEX_STATE_PATH.exists():
            return {}
        try:
            return json.loads(INDEX_STATE_PATH.read_text(encoding="utf-8"))
        except Exception:
            return {}

    def _write_index_state(self, state: dict[str, Any]) -> None:
        INDEX_STATE_PATH.write_text(json.dumps(state, indent=2), encoding="utf-8")

    def _reset_collection(self) -> None:
        try:
            self.chroma.delete_collection(COLLECTION_NAME)
        except Exception:
            pass
        self.collection = self.chroma.get_or_create_collection(name=COLLECTION_NAME)

    def _upsert_batch(self, ids: list[str], documents: list[str], metadatas: list[dict[str, Any]], embeddings: list[list[float]]) -> None:
        if ids:
            self.collection.upsert(ids=ids, documents=documents, metadatas=metadatas, embeddings=embeddings)

    def _ensure_chroma_index(self) -> None:
        current_state = self._current_db_state()
        index_state = self._read_index_state()

        if current_state == index_state and self.collection.count() > 0:
            return

        self._reset_collection()
        cursor = self.conn.cursor()

        ids: list[str] = []
        docs: list[str] = []
        metas: list[dict[str, Any]] = []
        embeds: list[list[float]] = []

        def flush() -> None:
            nonlocal ids, docs, metas, embeds
            self._upsert_batch(ids, docs, metas, embeds)
            ids, docs, metas, embeds = [], [], [], []

        cursor.execute("SELECT id, project, file_name, file_path, content, last_modified FROM documents")
        for row in cursor.fetchall():
            source_id = int(row["id"])
            for idx, chunk in enumerate(chunk_text(row["content"], size=950, overlap=200)):
                ids.append(f"doc-{source_id}-{idx}")
                docs.append(chunk)
                metas.append(
                    {
                        "kind": "document",
                        "record_id": source_id,
                        "project": row["project"] or "",
                        "file_name": row["file_name"] or "",
                        "file_path": row["file_path"] or "",
                        "last_modified": str(row["last_modified"] or ""),
                    }
                )
                embeds.append(embed_text(chunk))
                if len(ids) >= 128:
                    flush()

        cursor.execute("SELECT id, project, session_title, source_file, content, message_count FROM chat_sessions")
        for row in cursor.fetchall():
            source_id = int(row["id"])
            base_text = extract_chat_text(row["content"] or "")
            for idx, chunk in enumerate(chunk_text(base_text, size=950, overlap=220)):
                ids.append(f"chat-{source_id}-{idx}")
                docs.append(chunk)
                metas.append(
                    {
                        "kind": "chat",
                        "record_id": source_id,
                        "project": row["project"] or "",
                        "session_title": row["session_title"] or "",
                        "source_file": row["source_file"] or "",
                        "message_count": int(row["message_count"] or 0),
                    }
                )
                embeds.append(embed_text(chunk))
                if len(ids) >= 128:
                    flush()

        flush()
        self._write_index_state(current_state)

    def _vector_search(self, query: str, kind: str, project: str | None, limit: int) -> list[dict[str, Any]]:
        max_results = max(1, min(int(limit or 10), 25))
        where = {"$and": [{"kind": kind}, {"project": project}]} if project else {"kind": kind}

        response = self.collection.query(
            query_embeddings=[embed_text(query)],
            n_results=max_results * 2,
            where=where,
            include=["metadatas", "documents", "distances"],
        )

        metadatas = response.get("metadatas", [[]])[0]
        documents = response.get("documents", [[]])[0]
        distances = response.get("distances", [[]])[0]

        best_by_record: dict[int, dict[str, Any]] = {}
        for meta, doc, distance in zip(metadatas, documents, distances):
            if not isinstance(meta, dict):
                continue
            record_id = int(meta.get("record_id", 0))
            if record_id <= 0:
                continue
            score = max(0.0, 1.0 - float(distance or 0.0))
            existing = best_by_record.get(record_id)
            if existing is None or score > existing["score"]:
                best_by_record[record_id] = {"meta": meta, "snippet": doc, "score": score}

        ordered = sorted(best_by_record.items(), key=lambda item: item[1]["score"], reverse=True)
        results = []
        for record_id, payload in ordered[:max_results]:
            meta = payload["meta"]
            row = {
                "id": record_id,
                "project": meta.get("project", ""),
                "snippet": (payload["snippet"] or "")[:320],
                "score": round(payload["score"], 4),
            }
            if kind == "chat":
                row.update({"session_title": meta.get("session_title", ""), "source_file": meta.get("source_file", ""), "message_count": int(meta.get("message_count", 0))})
            else:
                row.update({"file_name": meta.get("file_name", ""), "file_path": meta.get("file_path", ""), "last_modified": meta.get("last_modified", "")})
            results.append(row)

        return results

    def search_documents(self, query: str, limit: int = 10, project: str = None) -> dict[str, Any]:
        self._lazy_ensure_index()
        results = [] if not query else self._vector_search(query, "document", project, limit)
        return {"query": query, "count": len(results), "results": results}

    def search_chats(self, query: str, limit: int = 10, project: str = None) -> dict[str, Any]:
        self._lazy_ensure_index()
        results = [] if not query else self._vector_search(query, "chat", project, limit)
        return {"query": query, "count": len(results), "results": results}

    def get_document(self, doc_id: int) -> dict[str, Any]:
        cursor = self.conn.cursor()
        cursor.execute("SELECT id, project, file_name, file_path, content, file_size, last_modified, indexed_at FROM documents WHERE id = ?", (doc_id,))
        row = cursor.fetchone()
        if not row:
            return {"error": f"Document {doc_id} not found"}
        return {k: row[k] for k in ("id", "project", "file_name", "file_path", "content", "file_size", "last_modified", "indexed_at")}

    def get_chat(self, chat_id: int) -> dict[str, Any]:
        cursor = self.conn.cursor()
        cursor.execute("SELECT id, project, session_title, source_file, content, message_count, indexed_at FROM chat_sessions WHERE id = ?", (chat_id,))
        row = cursor.fetchone()
        if not row:
            return {"error": f"Chat session {chat_id} not found"}
        return {k: row[k] for k in ("id", "project", "session_title", "source_file", "content", "message_count", "indexed_at")}

    def get_stats(self) -> dict[str, Any]:
        self._lazy_ensure_index()
        cursor = self.conn.cursor()
        cursor.execute("SELECT COUNT(*) as count FROM documents")
        total_docs = int(cursor.fetchone()["count"])
        cursor.execute("SELECT COUNT(*) as count FROM chat_sessions")
        total_chats = int(cursor.fetchone()["count"])
        cursor.execute("SELECT COUNT(DISTINCT project) as count FROM documents")
        projects = int(cursor.fetchone()["count"])
        return {
            "total_documents": total_docs,
            "total_chats": total_chats,
            "projects": projects,
            "vector_chunks": self.collection.count(),
            "database_path": str(self.db_path),
            "database_size_mb": round(self.db_path.stat().st_size / (1024 * 1024), 2),
            "chroma_path": str(CHROMA_PATH),
        }

    def list_recent_documents(self, limit: int = 20, project: str = None) -> dict[str, Any]:
        cursor = self.conn.cursor()
        sql = "SELECT id, project, file_name, file_path, indexed_at FROM documents"
        params: list[Any] = []
        if project:
            sql += " WHERE project = ?"
            params.append(project)
        sql += " ORDER BY indexed_at DESC LIMIT ?"
        params.append(max(1, min(int(limit or 20), 100)))
        cursor.execute(sql, params)
        rows = [{k: row[k] for k in ("id", "project", "file_name", "file_path", "indexed_at")} for row in cursor.fetchall()]
        return {"count": len(rows), "documents": rows}

    def reindex(self) -> dict[str, Any]:
        self._reset_collection()
        self._index_ready = False
        self._ensure_chroma_index()
        self._index_ready = True
        return {"status": "ok", "vector_chunks": self.collection.count()}


def handle_request(server: AuxideBrainServer, request: dict[str, Any]) -> dict[str, Any]:
    method = request.get("method")
    params = request.get("params", {})
    request_id = request.get("id")

    try:
        if method == "initialize":
            return {
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": "auxide-brain", "version": "2.0.0-chroma"},
                },
            }

        if method == "tools/list":
            return {
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {
                    "tools": [
                        {"name": "search_documents", "description": "Vector search docs across auxide, auxide-dsp, auxide-io, and auxide-midi", "inputSchema": {"type": "object", "properties": {"query": {"type": "string"}, "project": {"type": "string"}, "limit": {"type": "number"}}, "required": ["query"]}},
                        {"name": "search_chats", "description": "Vector search chats", "inputSchema": {"type": "object", "properties": {"query": {"type": "string"}, "project": {"type": "string"}, "limit": {"type": "number"}}, "required": ["query"]}},
                        {"name": "get_document", "description": "Get doc by id", "inputSchema": {"type": "object", "properties": {"doc_id": {"type": "number"}}, "required": ["doc_id"]}},
                        {"name": "get_chat", "description": "Get chat by id", "inputSchema": {"type": "object", "properties": {"chat_id": {"type": "number"}}, "required": ["chat_id"]}},
                        {"name": "get_stats", "description": "Get stats", "inputSchema": {"type": "object", "properties": {}}},
                        {"name": "list_recent_documents", "description": "List recent docs", "inputSchema": {"type": "object", "properties": {"project": {"type": "string"}, "limit": {"type": "number"}}}},
                        {"name": "reindex", "description": "Force reindex", "inputSchema": {"type": "object", "properties": {}}},
                    ]
                },
            }

        if method == "tools/call":
            name = params.get("name")
            args = params.get("arguments", {})

            dispatch = {
                "search_documents": lambda: server.search_documents(args.get("query", ""), args.get("limit", 10), args.get("project")),
                "search_chats": lambda: server.search_chats(args.get("query", ""), args.get("limit", 10), args.get("project")),
                "get_document": lambda: server.get_document(args.get("doc_id")),
                "get_chat": lambda: server.get_chat(args.get("chat_id")),
                "get_stats": lambda: server.get_stats(),
                "list_recent_documents": lambda: server.list_recent_documents(args.get("limit", 20), args.get("project")),
                "reindex": lambda: server.reindex(),
            }

            handler = dispatch.get(name)
            if not handler:
                return {"jsonrpc": "2.0", "id": request_id, "error": {"code": -32602, "message": f"Unknown tool: {name}"}}

            result = handler()
            return {"jsonrpc": "2.0", "id": request_id, "result": {"content": [{"type": "text", "text": json.dumps(result, indent=2)}]}}

        if method == "notifications/initialized":
            return {}

        return {"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": f"Method not found: {method}"}}

    except Exception as exc:
        return {"jsonrpc": "2.0", "id": request_id, "error": {"code": -32603, "message": f"Internal error: {str(exc)}"}}


def main() -> None:
    if not DATABASE_PATH.exists():
        print(json.dumps({"error": f"Database not found: {DATABASE_PATH}"}), file=sys.stderr)
        sys.exit(1)

    try:
        server = AuxideBrainServer(DATABASE_PATH)
    except Exception as exc:
        print(json.dumps({"error": f"Failed to initialize server: {str(exc)}"}), file=sys.stderr)
        sys.exit(1)

    for line in sys.stdin:
        try:
            request = json.loads(line)
            response = handle_request(server, request)
            if response:
                print(json.dumps(response), flush=True)
        except json.JSONDecodeError:
            continue
        except Exception as exc:
            print(json.dumps({"jsonrpc": "2.0", "error": {"code": -32700, "message": f"Parse error: {str(exc)}"}}), flush=True)


if __name__ == "__main__":
    main()
