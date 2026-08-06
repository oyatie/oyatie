"""CLI for quantitative / ML pipeline workers: python -m mm_ml ..."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from .bandit import UCB1, arms_from_routing, load_bandit_state, save_bandit_state
from .features import feature_matrix, load_kpi_rows, summarize_features
from .harness_adapter import assess_kpi_via_harness, route_prior_via_harness
from .io_util import find_grok, load_json, load_quant_config, now, utc_date, write_json
from .reward import reward_from_grade_row


def cmd_reward(args: argparse.Namespace) -> int:
    grok = find_grok(Path(args.cwd))
    cfg = load_quant_config(grok)
    components = (cfg.get("reward") or {}).get("components") or cfg.get("reward")
    if args.run_id:
        grade = load_json(grok / "mm-runs" / args.run_id / "grade.json")
        r = reward_from_grade_row(grade, components=components)
        out = {"run_id": args.run_id, "reward": r, "grade": grade.get("letter"), "ts": now()}
        if args.write:
            write_json(grok / "mm-runs" / args.run_id / "reward.json", out)
        print(json.dumps(out, indent=2))
        return 0
    # from latest kpi line
    kpi = grok / "mm-runs" / "_ledger" / "kpi.jsonl"
    rows = load_kpi_rows(kpi)
    if not rows:
        print(json.dumps({"error": "no kpi rows", "path": str(kpi)}))
        return 2
    row = rows[-1]
    r = reward_from_grade_row(row, components=components)
    print(json.dumps({"reward": r, "row": row, "ts": now()}, indent=2))
    return 0


def cmd_features(args: argparse.Namespace) -> int:
    grok = find_grok(Path(args.cwd))
    kpi = grok / "mm-runs" / "_ledger" / "kpi.jsonl"
    rows = load_kpi_rows(kpi)
    window = int(args.window or 50)
    summary = summarize_features(rows, window=window)
    mats = feature_matrix(rows, window=window)
    out = {"summary": summary, "n_vectors": len(mats), "last": mats[-1] if mats else None}
    if args.write:
        write_json(grok / "mm-runs" / "_ledger" / "feature-summary.json", out)
    print(json.dumps(out, indent=2))
    return 0


def cmd_process(args: argparse.Namespace) -> int:
    """Prefer Developer/harness process_metrics via mm_bridge."""
    grok = find_grok(Path(args.cwd))
    cfg = load_quant_config(grok)
    ext = (cfg.get("external_harness") or {}).get("path") or args.harness_root
    kpi = grok / "mm-runs" / "_ledger" / "kpi.jsonl"
    window = int(args.window or (cfg.get("features") or {}).get("window") or 50)
    out = assess_kpi_via_harness(
        kpi,
        harness_root=Path(ext) if ext else None,
        window=window,
    )
    if args.write:
        write_json(grok / "mm-runs" / "_ledger" / "process-assessment.json", out)
    print(json.dumps(out, indent=2))
    return 0 if out.get("available") else 3


def cmd_bandit(args: argparse.Namespace) -> int:
    grok = find_grok(Path(args.cwd))
    cfg = load_quant_config(grok)
    bcfg = cfg.get("bandit") or {}
    routing = load_json(grok / "harness" / "model-routing.v1.json")
    arms = arms_from_routing(routing)
    state_rel = bcfg.get("state_path") or "mm-runs/_ledger/bandit-state.json"
    state_path = grok / state_rel
    bandit = load_bandit_state(state_path, arms, c=float(bcfg.get("exploration_c") or 1.414))

    # update from recent rewards
    kpi = load_kpi_rows(grok / "mm-runs" / "_ledger" / "kpi.jsonl")
    components = (cfg.get("reward") or {}).get("components")
    # Without per-arm labels, update a synthetic "run" arm with mean reward signal
    if kpi:
        r = reward_from_grade_row(kpi[-1], components=components)
        # map mode to arm family if possible
        arm = bandit.select()
        bandit.update(arm, r)

    suggest = bandit.select()
    save_bandit_state(state_path, bandit)
    out = {
        "suggest_arm": suggest,
        "state": bandit.to_state(),
        "auto_apply": bool(bcfg.get("auto_apply_routing")),
        "policy": "human-gated; never auto-edit model-routing.v1.json",
        "ts": now(),
    }
    if args.write:
        suggest_path = grok / "memory" / "behind" / f"bandit-routing-suggest-{utc_date()}.json"
        write_json(suggest_path, out)
        out["written"] = str(suggest_path)
    print(json.dumps(out, indent=2))
    return 0


def cmd_route_prior(args: argparse.Namespace) -> int:
    grok = find_grok(Path(args.cwd))
    cfg = load_quant_config(grok)
    ext = (cfg.get("external_harness") or {}).get("path") or args.harness_root
    out = route_prior_via_harness(
        args.task_class,
        args.language,
        harness_root=Path(ext) if ext else None,
    )
    print(json.dumps(out, indent=2))
    return 0 if out.get("available") else 3


def cmd_learn_hook(args: argparse.Namespace) -> int:
    """Aggregate SCORE_GRADE/LEARN hook: reward + process + bandit."""
    # chain
    rc = 0
    for fn, ns in (
        (cmd_reward, argparse.Namespace(cwd=args.cwd, run_id=args.run_id or "", write=True)),
        (cmd_features, argparse.Namespace(cwd=args.cwd, window=50, write=True)),
        (cmd_process, argparse.Namespace(cwd=args.cwd, window=50, write=True, harness_root=args.harness_root)),
        (cmd_bandit, argparse.Namespace(cwd=args.cwd, write=True)),
    ):
        code = fn(ns)
        if code not in (0, 3):
            rc = code
    return rc


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="mm-quant: quantitative ML workers for mm-delivery")
    ap.add_argument("--cwd", default=".")
    ap.add_argument(
        "--harness-root",
        default="/Users/jasonlee/Developer/harness",
        help="path to external evidence-gated harness (process_metrics, mm_bridge, router)",
    )
    sub = ap.add_subparsers(dest="cmd", required=True)

    sp = sub.add_parser("reward", help="compute scalar reward from grade or latest KPI")
    sp.add_argument("--run-id", default="")
    sp.add_argument("--write", action="store_true")
    sp.set_defaults(func=cmd_reward)

    sp = sub.add_parser("features", help="feature summary from KPI ledger")
    sp.add_argument("--window", type=int, default=50)
    sp.add_argument("--write", action="store_true")
    sp.set_defaults(func=cmd_features)

    sp = sub.add_parser("process", help="SPC via Developer/harness process_metrics/mm_bridge")
    sp.add_argument("--window", type=int, default=50)
    sp.add_argument("--write", action="store_true")
    sp.set_defaults(func=cmd_process)

    sp = sub.add_parser("bandit", help="UCB1 routing suggest (human-gated)")
    sp.add_argument("--write", action="store_true")
    sp.set_defaults(func=cmd_bandit)

    sp = sub.add_parser("route-prior", help="rank lanes via harness.router priors")
    sp.add_argument("--task-class", default="implementation")
    sp.add_argument("--language", default="rust")
    sp.set_defaults(func=cmd_route_prior)

    sp = sub.add_parser("learn-hook", help="SCORE_GRADE/LEARN aggregate: reward+features+process+bandit")
    sp.add_argument("--run-id", default="")
    sp.set_defaults(func=cmd_learn_hook)

    args = ap.parse_args(argv)
    return int(args.func(args))


if __name__ == "__main__":
    raise SystemExit(main())
