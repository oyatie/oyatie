"""UCB1 multi-armed bandit for human-gated model-routing suggestions."""

from __future__ import annotations

import math
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional

from .io_util import load_json, write_json


class UCB1:
    def __init__(self, arms: List[str], *, c: float = 1.414) -> None:
        if not arms:
            raise ValueError("arms required")
        self.arms = list(arms)
        self.c = float(c)
        self.counts = {a: 0 for a in self.arms}
        self.values = {a: 0.0 for a in self.arms}
        self.t = 0

    def select(self) -> str:
        for a in self.arms:
            if self.counts[a] == 0:
                return a
        self.t = max(self.t, sum(self.counts.values()))
        best_a = self.arms[0]
        best_u = -1e300
        for a in self.arms:
            exploit = self.values[a]
            explore = self.c * math.sqrt(math.log(self.t + 1) / self.counts[a])
            u = exploit + explore
            if u > best_u:
                best_u = u
                best_a = a
        return best_a

    def update(self, arm: str, reward: float) -> None:
        if arm not in self.counts:
            self.arms.append(arm)
            self.counts[arm] = 0
            self.values[arm] = 0.0
        self.counts[arm] += 1
        n = self.counts[arm]
        self.values[arm] += (float(reward) - self.values[arm]) / n
        self.t = sum(self.counts.values())

    def to_state(self) -> dict:
        return {
            "algorithm": "ucb1",
            "c": self.c,
            "arms": self.arms,
            "counts": self.counts,
            "values": self.values,
            "t": self.t,
        }

    @classmethod
    def from_state(cls, state: Mapping[str, Any]) -> "UCB1":
        arms = list(state.get("arms") or [])
        b = cls(arms or ["default"], c=float(state.get("c") or 1.414))
        b.counts = {a: int((state.get("counts") or {}).get(a, 0)) for a in b.arms}
        b.values = {a: float((state.get("values") or {}).get(a, 0.0)) for a in b.arms}
        b.t = int(state.get("t") or sum(b.counts.values()))
        return b


def arms_from_routing(routing: Mapping[str, Any]) -> List[str]:
    priors = routing.get("priors") or {}
    arms: List[str] = []
    for role, cfg in priors.items():
        if not isinstance(cfg, dict):
            continue
        provider = cfg.get("provider") or "?"
        model = cfg.get("model") or "?"
        arms.append(f"{role}:{provider}/{model}")
    return arms or ["ORCHESTRATOR:default"]


def load_bandit_state(path: Path, arms: List[str], *, c: float = 1.414) -> UCB1:
    if path.is_file():
        try:
            return UCB1.from_state(load_json(path))
        except (OSError, ValueError, TypeError, KeyError):
            pass
    return UCB1(arms, c=c)


def save_bandit_state(path: Path, bandit: UCB1) -> None:
    write_json(path, bandit.to_state())
