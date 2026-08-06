"""Reward shaping for RL-style process improvement (stdlib)."""

from __future__ import annotations

from typing import Any, Mapping


DEFAULT_LETTER = {
    "A": 1.0,
    "B": 0.75,
    "C": 0.4,
    "D": 0.15,
    "F": 0.0,
    "?": 0.0,
}


def compute_reward(
    *,
    letter: str,
    ship_ready: bool = False,
    hard_fails: int | list = 0,
    failure_classes: int | list = 0,
    mode: str = "",
    dual_admit: bool = False,
    components: Mapping[str, Any] | None = None,
) -> float:
    cfg = {
        "grade_letter": dict(DEFAULT_LETTER),
        "ship_ready_bonus": 0.15,
        "hard_fail_penalty": -0.5,
        "failure_class_penalty": -0.08,
        "plan_only_cap": 0.55,
        "dual_admit_bonus": 0.1,
        "clip": [-1.0, 1.2],
    }
    if components:
        gl = components.get("grade_letter")
        if isinstance(gl, dict):
            cfg["grade_letter"].update({str(k).upper(): float(v) for k, v in gl.items()})
        for k in (
            "ship_ready_bonus",
            "hard_fail_penalty",
            "failure_class_penalty",
            "plan_only_cap",
            "dual_admit_bonus",
        ):
            if k in components:
                try:
                    cfg[k] = float(components[k])
                except (TypeError, ValueError):
                    pass
        if isinstance(components.get("clip"), (list, tuple)) and len(components["clip"]) == 2:
            cfg["clip"] = [float(components["clip"][0]), float(components["clip"][1])]

    r = float(cfg["grade_letter"].get(str(letter).upper(), 0.0))
    if ship_ready:
        r += float(cfg["ship_ready_bonus"])
    n_hard = len(hard_fails) if isinstance(hard_fails, list) else int(hard_fails or 0)
    n_fc = (
        len(failure_classes)
        if isinstance(failure_classes, list)
        else int(failure_classes or 0)
    )
    if n_hard:
        r += float(cfg["hard_fail_penalty"])
    r += float(cfg["failure_class_penalty"]) * n_fc
    if dual_admit:
        r += float(cfg["dual_admit_bonus"])
    if mode == "plan_only":
        r = min(r, float(cfg["plan_only_cap"]))
    lo, hi = cfg["clip"]
    return max(float(lo), min(float(hi), r))


def reward_from_grade_row(row: Mapping[str, Any], components: Mapping[str, Any] | None = None) -> float:
    return compute_reward(
        letter=str(row.get("letter") or "?"),
        ship_ready=bool(row.get("ship_ready")),
        hard_fails=row.get("hard_fails") or [],
        failure_classes=row.get("failure_classes") or [],
        mode=str(row.get("mode") or ""),
        dual_admit=bool(row.get("dual_admit")),
        components=components,
    )
