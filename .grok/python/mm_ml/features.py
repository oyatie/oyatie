"""Feature extraction from KPI ledger rows (stdlib)."""

from __future__ import annotations

from pathlib import Path
from typing import Any, Dict, List, Mapping, Sequence

from .io_util import read_jsonl
from .reward import reward_from_grade_row


def load_kpi_rows(path: Path) -> List[dict]:
    return read_jsonl(path)


def feature_matrix(
    rows: Sequence[Mapping[str, Any]],
    *,
    window: int = 50,
) -> List[Dict[str, float]]:
    seq = list(rows)[-window:] if window else list(rows)
    letters = ("A", "B", "C", "D", "F")
    out: List[Dict[str, float]] = []
    for r in seq:
        letter = str(r.get("letter") or "?").upper()
        score = r.get("score")
        try:
            score_n = float(score) / 100.0 if score is not None else 0.0
        except (TypeError, ValueError):
            score_n = 0.0
        hard = r.get("hard_fails") or []
        classes = r.get("failure_classes") or []
        feats: Dict[str, float] = {
            "score_norm": max(0.0, min(1.0, score_n)),
            "ship_ready": 1.0 if r.get("ship_ready") else 0.0,
            "hard_fail_count": float(len(hard) if isinstance(hard, list) else 0),
            "failure_class_count": float(len(classes) if isinstance(classes, list) else 0),
            "mode_plan_only": 1.0 if r.get("mode") == "plan_only" else 0.0,
            "reward": reward_from_grade_row(r),
        }
        for L in letters:
            feats[f"letter_{L}"] = 1.0 if letter == L else 0.0
        out.append(feats)
    return out


def summarize_features(rows: Sequence[Mapping[str, Any]], *, window: int = 50) -> dict:
    mats = feature_matrix(rows, window=window)
    if not mats:
        return {"n": 0, "mean_reward": 0.0, "mean_score_norm": 0.0}
    mean_r = sum(m["reward"] for m in mats) / len(mats)
    mean_s = sum(m["score_norm"] for m in mats) / len(mats)
    return {
        "n": len(mats),
        "mean_reward": mean_r,
        "mean_score_norm": mean_s,
        "plan_only_fraction": sum(m["mode_plan_only"] for m in mats) / len(mats),
    }
