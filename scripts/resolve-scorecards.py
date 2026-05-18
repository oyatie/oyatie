#!/usr/bin/env python3
"""Resolve per-microservice scorecards by merging canonical base + per-µservice overrides.

Per ADR-0064 canonical-base-and-localization-packs and SWEEP-H Slice 3, the
128 per-µservice scorecard JSONs collapsed into:
  - 4 canonical scorecards at specs/microservices/scorecards/canonical/
  - 32 per-µservice overrides at microservices/<ms>/scorecards/overrides.json

This script performs the merge at audit-time: it takes the canonical
scorecard, substitutes <ms> + <chart> placeholders from the override file,
applies any deltas declared in the override, and emits the resolved view.

Usage:
  python3 scripts/resolve-scorecards.py                            # print summary
  python3 scripts/resolve-scorecards.py <ms> <framework>            # emit resolved view
  python3 scripts/resolve-scorecards.py --emit-rollup               # rewrite registry/hyperscaler-scorecards/index.json
"""

from __future__ import annotations

import json
import sys
import argparse
from pathlib import Path
from typing import Any

ROOT = Path("/Users/jasonlee/oyatie")
CANONICAL_DIR = ROOT / "specs" / "microservices" / "scorecards" / "canonical"
MS_ROOT = ROOT / "microservices"
ROLLUP_PATH = ROOT / "registry" / "hyperscaler-scorecards" / "index.json"

MICROSERVICES = [
    "application", "audit-chain", "cell", "community", "observability",
    "ontology", "tenancy", "workflow-engine", "anonymous", "calendar",
    "docs", "drive", "foundry", "forms", "mail", "meet", "messenger",
    "network", "notes", "recordings", "sheets", "shorts", "sites",
    "slides", "social", "tasks", "translate", "workflow-studio",
    "cloud-iac", "cloud-k8s", "cloud-secrets", "governance",
]

FRAMEWORKS = {
    "aws-well-architected": "aws_well_architected",
    "google-sre-prr": "google_sre_prr",
    "cis-k8s-benchmark": "cis_k8s_benchmark",
    "slsa-l3": "slsa_l3",
}


def load_canonical(framework: str) -> dict:
    path = CANONICAL_DIR / f"{framework}.json"
    return json.loads(path.read_text())


def load_overrides(ms: str) -> dict:
    path = MS_ROOT / ms / "scorecards" / "overrides.json"
    if not path.is_file():
        raise FileNotFoundError(f"missing overrides for {ms}: {path}")
    return json.loads(path.read_text())


def substitute_placeholders(obj: Any, ms: str, chart: str) -> Any:
    """Recursively replace <ms> and <chart> placeholders in string values."""
    if isinstance(obj, str):
        return obj.replace("<ms>", ms).replace("<chart>", chart)
    if isinstance(obj, list):
        return [substitute_placeholders(x, ms, chart) for x in obj]
    if isinstance(obj, dict):
        return {k: substitute_placeholders(v, ms, chart) for k, v in obj.items()}
    return obj


def resolve_scorecard(ms: str, framework: str) -> dict:
    """Return the resolved per-µservice scorecard view (canonical + overrides)."""
    canonical = load_canonical(framework)
    overrides = load_overrides(ms)
    chart = overrides.get("chart_name", ms)
    # Substitute placeholders
    resolved = substitute_placeholders(canonical, ms, chart)
    # Rename `evidence_pattern` → `evidence` after substitution for compatibility
    resolved = rename_evidence_pattern(resolved)
    # Apply microservice + summary
    resolved.pop("_placeholders", None)
    resolved.pop("canonical_base", None)
    resolved.pop("overlay_consumers", None)
    resolved["microservice"] = ms
    if overrides.get("summary"):
        resolved["summary"] = overrides["summary"]
    # Apply per-framework overrides
    fw_key = FRAMEWORKS[framework]
    fw_overrides = overrides.get(fw_key, {})
    if fw_overrides.get("overall_status"):
        resolved["overall_status"] = fw_overrides["overall_status"]
    # Apply deltas (evidence_suffix appended; field overrides applied to specific control)
    for delta in fw_overrides.get("deltas", []):
        apply_delta(resolved, framework, delta)
    return resolved


def rename_evidence_pattern(obj: Any) -> Any:
    if isinstance(obj, dict):
        new = {}
        for k, v in obj.items():
            if k == "evidence_pattern":
                new["evidence"] = rename_evidence_pattern(v)
            else:
                new[k] = rename_evidence_pattern(v)
        return new
    if isinstance(obj, list):
        return [rename_evidence_pattern(x) for x in obj]
    return obj


def apply_delta(resolved: dict, framework: str, delta: dict) -> None:
    control = delta.get("control")
    field = delta.get("field")
    value = delta.get("value")
    if not control or not field:
        return
    # Find the control row by id (AWS, CIS) or by key (PRR, SLSA)
    if framework == "aws-well-architected":
        for pillar_key, pillar in resolved.get("pillars", {}).items():
            for c in pillar.get("controls", []):
                if c.get("id") == control:
                    if field == "evidence_suffix":
                        c["evidence"] = f"{c.get('evidence','')} {value}".strip()
                    else:
                        c[field] = value
    elif framework == "cis-k8s-benchmark":
        for cat in resolved.get("categories", {}).values():
            for c in cat.get("controls", []):
                if c.get("id") == control:
                    if field == "evidence_suffix":
                        c["evidence"] = f"{c.get('evidence','')} {value}".strip()
                    else:
                        c[field] = value
    elif framework == "google-sre-prr":
        item = resolved.get("checklist", {}).get(control)
        if item is not None:
            if field == "slo_count":
                item["evidence"] = f"{item.get('evidence','')} count: {value}".strip()
            elif field == "runbook_count":
                item["evidence"] = f"{item.get('evidence','')} count: {value}".strip()
            elif field == "dashboard_count":
                item["evidence"] = f"{item.get('evidence','')} count: {value}".strip()
            elif field == "evidence_suffix":
                item["evidence"] = f"{item.get('evidence','')} {value}".strip()
            else:
                item[field] = value
    elif framework == "slsa-l3":
        for sect in resolved.get("requirements", {}).values():
            if control in sect:
                if field == "evidence_suffix":
                    sect[control]["evidence"] = f"{sect[control].get('evidence','')} {value}".strip()
                else:
                    sect[control][field] = value


def emit_rollup() -> dict:
    """Build the aggregate rollup referencing canonical + overrides."""
    rollup = {
        "$schema": "https://oyatie.dev/schemas/hyperscaler-scorecard-rollup.json",
        "schema_version": "1.1",
        "generated_at": "2026-05-18",
        "sweep": "SWEEP-H",
        "frameworks": [
            "AWS Well-Architected (5 pillars)",
            "Google SRE Production Readiness Review",
            "CIS Kubernetes Benchmark v1.10",
            "SLSA (Supply-chain Levels for Software Artifacts) v1.0",
        ],
        "canonical_base_paths": {
            "aws_well_architected": "specs/microservices/scorecards/canonical/aws-well-architected.json",
            "google_sre_prr": "specs/microservices/scorecards/canonical/google-sre-prr.json",
            "cis_k8s_benchmark": "specs/microservices/scorecards/canonical/cis-k8s-benchmark.json",
            "slsa_l3": "specs/microservices/scorecards/canonical/slsa-l3.json",
        },
        "canonical_authority": "ADR-0064",
        "aggregate_status": "green",
        "microservices": {},
    }
    agg_green = True
    for ms in sorted(MICROSERVICES):
        overrides = load_overrides(ms)
        statuses = {
            "aws_well_architected": overrides.get("aws_well_architected", {}).get("overall_status", "green"),
            "google_sre_prr": overrides.get("google_sre_prr", {}).get("overall_status", "green"),
            "cis_k8s_benchmark": overrides.get("cis_k8s_benchmark", {}).get("overall_status", "green"),
            "slsa_l3": overrides.get("slsa_l3", {}).get("overall_status", "green"),
        }
        ms_green = all(s == "green" for s in statuses.values())
        agg_green = agg_green and ms_green
        rollup["microservices"][ms] = {
            **statuses,
            "overrides_path": f"microservices/{ms}/scorecards/overrides.json",
            "chart_name": overrides.get("chart_name", ms),
        }
    rollup["aggregate_status"] = "green" if agg_green else "yellow"
    return rollup


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--emit-rollup", action="store_true",
                        help="Rewrite registry/hyperscaler-scorecards/index.json")
    parser.add_argument("ms", nargs="?", help="Microservice slug")
    parser.add_argument("framework", nargs="?",
                        choices=list(FRAMEWORKS.keys()),
                        help="Framework name")
    args = parser.parse_args()

    if args.emit_rollup:
        rollup = emit_rollup()
        ROLLUP_PATH.write_text(json.dumps(rollup, indent=2, ensure_ascii=False) + "\n")
        ms_count = len(rollup["microservices"])
        print(f"[ok] wrote {ROLLUP_PATH.relative_to(ROOT)}")
        print(f"     microservices: {ms_count}")
        print(f"     aggregate_status: {rollup['aggregate_status']}")
        return 0

    if args.ms and args.framework:
        resolved = resolve_scorecard(args.ms, args.framework)
        print(json.dumps(resolved, indent=2, ensure_ascii=False))
        return 0

    # Default: validate every (ms, framework) combo resolves without error
    failures = []
    for ms in MICROSERVICES:
        for fw in FRAMEWORKS:
            try:
                resolved = resolve_scorecard(ms, fw)
                if resolved.get("overall_status") != "green":
                    failures.append((ms, fw, "non-green"))
            except Exception as e:
                failures.append((ms, fw, str(e)))
    total = len(MICROSERVICES) * len(FRAMEWORKS)
    print(f"Resolved {total - len(failures)} / {total} scorecards")
    if failures:
        for ms, fw, err in failures:
            print(f"  FAIL {ms} {fw}: {err}")
        return 1
    print("[ok] All canonical+overrides combinations resolve green.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
