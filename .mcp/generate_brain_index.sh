#!/usr/bin/env bash
# generate_brain_index.sh — Dynamic version.
# Discovers project names from the DB, generates .mcp/brain-index/ reference files.
# These are NOT auto-loaded — the AI reads them on-demand via file access.
# Pure bash + sqlite3. No Python. Run after every brain rebuild.
set -euo pipefail

# Auto-discover sqlite3: check PATH, then look next to this script, then common locations
if ! command -v sqlite3 &>/dev/null; then
  SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
  if [[ -f "$SCRIPT_DIR/sqlite3.exe" ]]; then
    export PATH="$SCRIPT_DIR:$PATH"
  elif [[ -f "$SCRIPT_DIR/sqlite3" ]]; then
    export PATH="$SCRIPT_DIR:$PATH"
  fi
fi

MAX_LINES=95
DB="${1:-$(dirname "$0")/*.db}"
# If glob, pick first match
if [[ "$DB" == *"*"* ]]; then
  for f in $DB; do DB="$f"; break; done
fi
OUT_DIR="$(cd "$(dirname "$0")" && pwd)/brain-index"
TMPDIR_GEN=$(mktemp -d)
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

trap 'rm -rf "$TMPDIR_GEN"' EXIT

if [[ ! -f "$DB" ]]; then
  echo "ERROR: Database not found: $DB" >&2; exit 1
fi
mkdir -p "$OUT_DIR"

# Strip Windows \r from all sqlite3 output
q() { sqlite3 -separator '|' "$DB" "$1" | tr -d '\r'; }

# ── Auto-split emitter ────────────────────────────────────────
split_and_emit() {
  local content_file="$1" base_name="$2" desc="$3"
  local total_lines
  total_lines=$(wc -l < "$content_file" | tr -d ' \r')
  local effective_max=$((MAX_LINES - 5))

  if [[ "$total_lines" -le "$effective_max" ]]; then
    { printf -- '---\napplyTo: "**"\ndescription: "%s"\n---\n\n' "$desc"; cat "$content_file"; } > "$OUT_DIR/${base_name}.instructions.md"
    return
  fi

  local part=1 current_lines=0 part_file=""
  part_file="$TMPDIR_GEN/${base_name}_part${part}.body"
  : > "$part_file"

  start_new_part() {
    part=$((part + 1))
    part_file="$TMPDIR_GEN/${base_name}_part${part}.body"
    : > "$part_file"
    current_lines=0
  }

  while IFS= read -r line; do
    if [[ "$line" == "## "* && "$current_lines" -gt 15 ]]; then
      if [[ "$current_lines" -gt $((effective_max - 15)) ]]; then
        start_new_part
      fi
    fi
    if [[ "$current_lines" -ge "$effective_max" ]]; then
      if [[ -z "$line" || "$line" == "- "* || "$line" == "| "* ]]; then
        start_new_part
        if [[ -n "$line" ]]; then
          echo "$line" >> "$part_file"
          current_lines=1
          continue
        fi
      fi
    fi
    echo "$line" >> "$part_file"
    current_lines=$((current_lines + 1))
  done < "$content_file"

  local total_parts=$part
  for ((p=1; p<=total_parts; p++)); do
    local body="$TMPDIR_GEN/${base_name}_part${p}.body"
    local suffix
    if [[ "$total_parts" -eq 1 ]]; then suffix=""; else suffix=$(printf "\\$(printf '%03o' $((96 + p)))"); fi
    local out_name="${base_name}${suffix}"
    local part_desc="${desc} (part ${p}/${total_parts})"
    [[ "$total_parts" -eq 1 ]] && part_desc="$desc"
    { printf -- '---\napplyTo: "**"\ndescription: "%s"\n---\n\n' "$part_desc"; cat "$body"; } > "$OUT_DIR/${out_name}.instructions.md"
  done
}

# ── Discover projects dynamically ─────────────────────────────
PROJECTS=$(q "SELECT DISTINCT project FROM documents ORDER BY project;")
TOTAL_DOCS=$(q "SELECT count(*) FROM documents;")
TOTAL_CHATS=$(q "SELECT count(*) FROM chat_sessions;")
EARLIEST=$(q "SELECT min(substr(created_at,1,10)) FROM chat_sessions;")
LATEST=$(q "SELECT max(substr(created_at,1,10)) FROM chat_sessions;")

# ════════════════════════════════════════════════════════════
# SECTION 0: Overview
# ════════════════════════════════════════════════════════════
{
cat << EOF
---
applyTo: "**"
description: "Brain index overview — auto-generated card catalog"
---

# Brain Index — Overview

**Generated**: ${TIMESTAMP} | **DB**: $(basename "$DB")

| Metric | Count |
|--------|-------|
| Total documents | ${TOTAL_DOCS} |
| Total chats | ${TOTAL_CHATS} |
| Chat range | ${EARLIEST} to ${LATEST} |
EOF

# Per-project counts
echo "$PROJECTS" | while read -r proj; do
  dc=$(q "SELECT count(*) FROM documents WHERE project='${proj}';")
  cc=$(q "SELECT count(*) FROM chat_sessions WHERE project='${proj}';")
  echo "| ${proj} | ${dc} docs, ${cc} chats |"
done

cat << 'USAGE'

## Usage

Scan index files for matching entries, then call MCP brain tools with specific terms.

1. Find topic/filename in index files
2. Call `search_documents` or `search_chats` with those terms
3. Use `get_document`/`get_chat` by ID for full content

## Tools

`search_documents` `search_chats` `get_document` `get_chat` `get_stats` `list_recent_documents` `reindex`
USAGE
} > "$OUT_DIR/brain-index-00-overview.instructions.md"
echo "  [0] overview: done"

# ════════════════════════════════════════════════════════════
# SECTIONS 1-N: Per-project document catalogs
# ════════════════════════════════════════════════════════════
sec_num=1
echo "$PROJECTS" | while read -r proj; do
  dc=$(q "SELECT count(*) FROM documents WHERE project='${proj}';")
  safe_name=$(echo "$proj" | tr ' /' '--')
  {
  echo "# Brain Index: ${proj} Documents (${dc} files)"
  echo ""
  q "
  SELECT
    CASE WHEN instr(file_path, '/') > 0
         THEN substr(file_path, 1, instr(file_path, '/'))
         ELSE '(root)' END as dir,
    file_name,
    CASE
      WHEN file_size > 1048576 THEN cast(file_size/1048576 as text)||'MB'
      WHEN file_size > 1024 THEN cast(file_size/1024 as text)||'KB'
      ELSE cast(file_size as text)||'B'
    END as sz
  FROM documents WHERE project='${proj}' ORDER BY dir, file_name;
  " | {
    prev_dir=""
    while IFS='|' read -r dir fname sz; do
      if [[ "$dir" != "$prev_dir" ]]; then
        dcount=$(q "SELECT count(*) FROM documents WHERE project='${proj}' AND CASE WHEN instr(file_path,'/') > 0 THEN substr(file_path,1,instr(file_path,'/')) ELSE '(root)' END = '${dir}';")
        [[ -n "$prev_dir" ]] && echo ""
        echo "## ${dir} (${dcount} files)"
        echo ""
        prev_dir="$dir"
      fi
      echo "- ${fname} (${sz})"
    done
  }
  } > "$TMPDIR_GEN/sec_${safe_name}.txt"
  padnum=$(printf "%02d" "$sec_num")
  split_and_emit "$TMPDIR_GEN/sec_${safe_name}.txt" "brain-index-${padnum}-${safe_name}-docs" "Brain index — ${proj} documents"
  echo "  [${sec_num}] ${proj} docs: done"
  sec_num=$((sec_num + 1))
done

# ════════════════════════════════════════════════════════════
# CHATS section
# ════════════════════════════════════════════════════════════
{
echo "# Brain Index: Chat Sessions (${TOTAL_CHATS} total)"
echo ""
echo "$PROJECTS" | while read -r proj; do
  # Also check chat_sessions for projects not in documents
  true
done
# Use all distinct projects from both tables
ALL_CHAT_PROJECTS=$(q "SELECT DISTINCT project FROM chat_sessions ORDER BY project;")
echo "$ALL_CHAT_PROJECTS" | while read -r proj; do
  cnt=$(q "SELECT count(*) FROM chat_sessions WHERE project='${proj}';")
  echo "## ${proj} (${cnt} sessions)"
  echo ""
  echo "| ID | Title | Date | Msgs |"
  echo "|----|-------|------|------|"
  q "SELECT id, COALESCE(session_title,'(untitled)'), COALESCE(substr(created_at,1,10),'?'), COALESCE(message_count,0) FROM chat_sessions WHERE project='${proj}' ORDER BY created_at DESC;" | while IFS='|' read -r sid title cdate msgs; do
    echo "| ${sid} | ${title} | ${cdate} | ${msgs} |"
  done
  echo ""
done
} > "$TMPDIR_GEN/sec_chats.txt"
# Figure out next section number: count projects + 1 for overview
proj_count=$(echo "$PROJECTS" | wc -l | tr -d ' \r')
chat_sec=$((proj_count + 1))
chat_padnum=$(printf "%02d" "$chat_sec")
split_and_emit "$TMPDIR_GEN/sec_chats.txt" "brain-index-${chat_padnum}-chats" "Brain index — chat sessions"
echo "  [${chat_sec}] chats: done"

# ════════════════════════════════════════════════════════════
# TOPICS section
# ════════════════════════════════════════════════════════════
topic_sec=$((chat_sec + 1))
topic_padnum=$(printf "%02d" "$topic_sec")
{
echo "# Brain Index: Topic Clusters"
echo ""
echo "Use these keywords for targeted MCP brain queries."
echo ""

print_topic() {
  local label="$1" dw="$2" cw="$3"
  local dc cc
  dc=$(q "SELECT count(*) FROM documents WHERE ${dw};")
  cc=$(q "SELECT count(*) FROM chat_sessions WHERE ${cw};")
  if [[ "$dc" -gt 0 || "$cc" -gt 0 ]]; then
    echo "## ${label} (${dc} docs, ${cc} chats)"
    echo ""
    if [[ "$dc" -gt 0 ]]; then
      q "SELECT file_name FROM documents WHERE ${dw} ORDER BY file_name LIMIT 12;" | while read -r fn; do
        echo "- ${fn}"
      done
    fi
    if [[ "$cc" -gt 0 ]]; then
      echo ""
      echo "Chats:"
      q "SELECT session_title FROM chat_sessions WHERE ${cw} AND session_title IS NOT NULL ORDER BY created_at DESC LIMIT 4;" | while read -r t; do
        echo "- ${t}"
      done
    fi
    echo ""
  fi
}

# Generic topics that work across any shimmy/pounce/other projects
print_topic "Architecture / Design" \
  "upper(file_name) LIKE '%ARCHITECTURE%' OR upper(file_name) LIKE '%DESIGN%'" \
  "upper(session_title) LIKE '%ARCHITECTURE%' OR upper(session_title) LIKE '%DESIGN%'"
print_topic "Audit / Forensics" \
  "upper(file_name) LIKE '%AUDIT%' OR upper(file_name) LIKE '%FORENSIC%'" \
  "upper(session_title) LIKE '%AUDIT%' OR upper(session_title) LIKE '%FORENSIC%'"
print_topic "Benchmarks / Performance" \
  "upper(file_name) LIKE '%BENCH%' OR upper(file_name) LIKE '%PERF%' OR upper(file_name) LIKE '%LATENCY%'" \
  "upper(session_title) LIKE '%BENCH%' OR upper(session_title) LIKE '%PERF%' OR upper(session_title) LIKE '%LATENCY%'"
print_topic "Build / CI / Deploy" \
  "upper(file_name) LIKE '%BUILD%' OR upper(file_name) LIKE '%DEPLOY%' OR upper(file_name) LIKE '%CI%'" \
  "upper(session_title) LIKE '%BUILD%' OR upper(session_title) LIKE '%DEPLOY%' OR upper(session_title) LIKE '%CI%'"
print_topic "Checklists / Runbooks" \
  "upper(file_name) LIKE '%CHECKLIST%' OR upper(file_name) LIKE '%RUNBOOK%'" \
  "upper(session_title) LIKE '%CHECKLIST%' OR upper(session_title) LIKE '%RUNBOOK%'"
print_topic "Cloud / AWS / VM" \
  "upper(file_name) LIKE '%CLOUD%' OR upper(file_name) LIKE '%AWS%' OR upper(file_name) LIKE '%VM%' OR upper(file_name) LIKE '%HETZNER%' OR upper(file_name) LIKE '%LAMBDA%'" \
  "upper(session_title) LIKE '%CLOUD%' OR upper(session_title) LIKE '%AWS%' OR upper(session_title) LIKE '%VM%' OR upper(session_title) LIKE '%HETZNER%' OR upper(session_title) LIKE '%LAMBDA%'"
print_topic "MEV / Arbitrage" \
  "upper(file_name) LIKE '%MEV%' OR upper(file_name) LIKE '%ARB%'" \
  "upper(session_title) LIKE '%MEV%' OR upper(session_title) LIKE '%ARB%'"
print_topic "Migration / Upgrade" \
  "upper(file_name) LIKE '%MIGRAT%' OR upper(file_name) LIKE '%UPGRADE%'" \
  "upper(session_title) LIKE '%MIGRAT%' OR upper(session_title) LIKE '%UPGRADE%'"
print_topic "OCR / Vision / AI" \
  "upper(file_name) LIKE '%OCR%' OR upper(file_name) LIKE '%VISION%' OR upper(file_name) LIKE '%AI%' OR upper(file_name) LIKE '%MODEL%'" \
  "upper(session_title) LIKE '%OCR%' OR upper(session_title) LIKE '%VISION%' OR upper(session_title) LIKE '%AI%' OR upper(session_title) LIKE '%MODEL%'"
print_topic "P2P / Networking" \
  "upper(file_name) LIKE '%P2P%' OR upper(file_name) LIKE '%PEER%' OR upper(file_name) LIKE '%NETWORK%'" \
  "upper(session_title) LIKE '%P2P%' OR upper(session_title) LIKE '%PEER%' OR upper(session_title) LIKE '%NETWORK%'"
print_topic "Product / Strategy" \
  "upper(file_name) LIKE '%PRODUCT%' OR upper(file_name) LIKE '%STRATEGY%'" \
  "upper(session_title) LIKE '%PRODUCT%' OR upper(session_title) LIKE '%STRATEGY%'"
print_topic "Security / Keys / Auth" \
  "upper(file_name) LIKE '%SECUR%' OR upper(file_name) LIKE '%AUTH%' OR upper(file_name) LIKE '%KEY%' OR upper(file_name) LIKE '%CRYPT%'" \
  "upper(session_title) LIKE '%SECUR%' OR upper(session_title) LIKE '%AUTH%' OR upper(session_title) LIKE '%KEY%' OR upper(session_title) LIKE '%CRYPT%'"
print_topic "Session Handoffs" \
  "upper(file_name) LIKE '%SESSION%' OR upper(file_name) LIKE '%HANDOFF%'" \
  "upper(session_title) LIKE '%SESSION%' OR upper(session_title) LIKE '%HANDOFF%'"
print_topic "Shimmy Core / FSE" \
  "upper(file_name) LIKE '%SHIMMY%' OR upper(file_name) LIKE '%FSE%' OR upper(file_name) LIKE '%BEAD%'" \
  "upper(session_title) LIKE '%SHIMMY%' OR upper(session_title) LIKE '%FSE%' OR upper(session_title) LIKE '%BEAD%'"
print_topic "Testing / QA" \
  "upper(file_name) LIKE '%TEST%' OR upper(file_name) LIKE '%QA%' OR upper(file_name) LIKE '%SPEC%'" \
  "upper(session_title) LIKE '%TEST%' OR upper(session_title) LIKE '%QA%' OR upper(session_title) LIKE '%SPEC%'"

} > "$TMPDIR_GEN/sec_topics.txt"
split_and_emit "$TMPDIR_GEN/sec_topics.txt" "brain-index-${topic_padnum}-topics" "Brain index — topic clusters"
echo "  [${topic_sec}] topics: done"

# ════════════════════════════════════════════════════════════
# TIMELINE section
# ════════════════════════════════════════════════════════════
timeline_sec=$((topic_sec + 1))
timeline_padnum=$(printf "%02d" "$timeline_sec")
{
echo "# Brain Index: Timeline"
echo ""
echo "## Documents by Month"
echo ""
q "SELECT COALESCE(substr(last_modified,1,7),'unknown'), project, count(*) FROM documents GROUP BY 1,2 ORDER BY 1 DESC, 2;" | while IFS='|' read -r m p c; do
  echo "- **${m}** ${p}: ${c} docs"
done
echo ""
echo "## Chats by Month"
echo ""
q "SELECT COALESCE(substr(created_at,1,7),'unknown'), project, count(*) FROM chat_sessions GROUP BY 1,2 ORDER BY 1 DESC, 2;" | while IFS='|' read -r m p c; do
  echo "- **${m}** ${p}: ${c} sessions"
done
echo ""
echo "## Recent Documents (last 20)"
echo ""
q "SELECT file_name, project, COALESCE(substr(last_modified,1,10),'?') FROM documents ORDER BY last_modified DESC LIMIT 20;" | while IFS='|' read -r f p d; do
  echo "- ${d} | ${p} | ${f}"
done
echo ""
echo "## Recent Chats (last 20)"
echo ""
q "SELECT COALESCE(session_title,'(untitled)'), project, COALESCE(substr(created_at,1,10),'?') FROM chat_sessions ORDER BY created_at DESC LIMIT 20;" | while IFS='|' read -r t p d; do
  echo "- ${d} | ${p} | ${t}"
done
} > "$TMPDIR_GEN/sec_timeline.txt"
split_and_emit "$TMPDIR_GEN/sec_timeline.txt" "brain-index-${timeline_padnum}-timeline" "Brain index — timeline"
echo "  [${timeline_sec}] timeline: done"

# ── Summary ────────────────────────────────────────────────
echo ""
echo "=== Brain Index Generated ==="
echo "Output: ${OUT_DIR}"
total_files=0
total_lines_all=0
for f in "$OUT_DIR"/brain-index-*.instructions.md; do
  lc=$(wc -l < "$f" | tr -d ' \r')
  total_lines_all=$((total_lines_all + lc))
  total_files=$((total_files + 1))
  status="ok"
  [[ "$lc" -gt "$MAX_LINES" ]] && status="OVER LIMIT"
  echo "  $(basename "$f"): ${lc} lines [${status}]"
done
echo "Total: ${total_files} files, ${total_lines_all} lines"
echo "Timestamp: ${TIMESTAMP}"
