"""mm_ml — quantitative / ML / RL helpers for the single mm-delivery pipeline.

Stdlib-first. Optional numpy/sklearn enhance estimators when installed.
Not merge authority; suggestions are human-gated.
"""

from .reward import compute_reward, reward_from_grade_row
from .features import load_kpi_rows, feature_matrix, summarize_features
from .bandit import UCB1, load_bandit_state, save_bandit_state, arms_from_routing

__all__ = [
    "compute_reward",
    "reward_from_grade_row",
    "load_kpi_rows",
    "feature_matrix",
    "summarize_features",
    "UCB1",
    "load_bandit_state",
    "save_bandit_state",
    "arms_from_routing",
]

__version__ = "0.1.0"
