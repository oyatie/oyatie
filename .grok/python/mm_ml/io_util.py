"""Shared I/O helpers (stdlib)."""

from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional


def now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def utc_date() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%d")


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, data: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")


def append_jsonl(path: Path, row: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as f:
        f.write(json.dumps(row, separators=(",", ":")) + "\n")


def find_repo(start: Path) -> Path:
    cur = start.resolve()
    for p in [cur, *cur.parents]:
        if (p / ".grok/harness/pipeline.json").is_file():
            return p
    return cur


def find_grok(start: Path) -> Path:
    repo = find_repo(start)
    g = repo / ".grok"
    if g.is_dir():
        return g
    # worktree may host kit under cwd
    if (start / ".grok/harness/pipeline.json").is_file():
        return start / ".grok"
    return g


def load_quant_config(grok: Path) -> dict:
    path = grok / "harness" / "quantitative-methods.v1.json"
    if path.is_file():
        return load_json(path)
    return {}


def read_jsonl(path: Path) -> List[dict]:
    if not path.is_file():
        return []
    rows: List[dict] = []
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return rows
