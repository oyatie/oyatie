"""Optional import bridge to Developer/harness (process_metrics, mm_bridge, router).

Never lifecycle authority. Fail soft when external harness is absent.
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any, Dict, List, Optional


def _default_harness_root() -> Path:
    return Path("/Users/jasonlee/Developer/harness")


def ensure_harness_path(root: Path | None = None) -> Optional[Path]:
    r = Path(root) if root else _default_harness_root()
    if not r.is_dir():
        return None
    s = str(r.resolve())
    if s not in sys.path:
        sys.path.insert(0, s)
    return r


def assess_kpi_via_harness(
    kpi_path: Path,
    *,
    harness_root: Path | None = None,
    window: int = 50,
) -> Dict[str, Any]:
    root = ensure_harness_path(harness_root)
    if root is None:
        return {
            "available": False,
            "reason": "external harness path missing",
            "fallback": "stdlib mm_ml only",
        }
    try:
        import mm_bridge  # type: ignore
    except ImportError as e:
        return {"available": False, "reason": f"import mm_bridge failed: {e}"}

    summary = mm_bridge.summarize_ledger(kpi_path, window=window)
    summary["available"] = True
    summary["harness_root"] = str(root)
    return summary


def route_prior_via_harness(
    task_class: str,
    language: str = "rust",
    *,
    harness_root: Path | None = None,
) -> Dict[str, Any]:
    root = ensure_harness_path(harness_root)
    if root is None:
        return {"available": False, "reason": "external harness path missing"}
    try:
        import router  # type: ignore
    except ImportError as e:
        return {"available": False, "reason": f"import router failed: {e}"}

    try:
        ranked = router.rank(task_class, language)
    except Exception as e:  # noqa: BLE001 — bridge must not crash pipeline
        return {"available": False, "reason": f"router.rank failed: {e}"}

    # router.rank returns (list[Lane], reason:str) in current harness
    reason = None
    lanes = ranked
    if isinstance(ranked, tuple) and len(ranked) >= 1:
        lanes = ranked[0]
        reason = ranked[1] if len(ranked) > 1 else None

    out_lanes = []
    for L in list(lanes or [])[:8]:
        out_lanes.append(
            {
                "family": getattr(L, "family", None),
                "model": getattr(L, "model", None),
                "channel": getattr(L, "channel", None),
                "effort": getattr(L, "effort", None),
                "graded": getattr(L, "graded", None),
                "note": getattr(L, "note", None),
            }
        )

    return {
        "available": True,
        "task_class": task_class,
        "language": language,
        "reason": reason,
        "ranked": out_lanes,
    }
