#!/usr/bin/env python3
"""Validate AC-0.17 claim-ceiling evidence without live readiness claims.

This checker is local/static fixture evidence only. It evaluates the checked-in
Phase-0 claim/evidence map, the hyperscaler production-readiness claim contract,
and the declared BAD/GREEN fixtures so regulated readiness language cannot be
used without a claim row, a permitted tier, and evidence appropriate to that
tier. It never posts statuses, mutates branch protection, or claims P0.0 green,
Phase-0 completion, production readiness, or hyperscaler-grade readiness.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

DEFAULT_CLAIM_MAP = "specs/phase0-claim-evidence-map.json"
DEFAULT_CONTRACT = "specs/hyperscaler-production-readiness-claim-contract.json"

REQUIRED_ROW_FIELDS = [
    "id",
    "artifact",
    "claim_text",
    "claim_tier",
    "allowed_language_now",
    "regulated_terms",
    "current_evidence",
    "missing_for_next_tier",
    "owner",
]
STRONG_CLAIM_TIERS = {"production_ready", "hyperscaler_grade"}
BUDGET_SIGNALS = (
    "p50",
    "p95",
    "p99",
    "throughput",
    "concurrency target",
    "performance_budget",
    "performance budget",
    "PERF-CAPACITY",
)
MEASURED_RESULT_SIGNALS = (
    "measured_result",
    "measured result",
    "load result",
    "soak result",
    "load/soak result",
    "capacity breakpoint",
)
FORBIDDEN_MECHANICAL_EVIDENCE_RE = re.compile(
    r"(local-only command|legacy oya cli invocation|\boya\s+(?:verify|gate)\b|\blocal\s+oya\b|advisory check|unrequired status|stale sha)",
    re.IGNORECASE,
)
LIVE_FALSE_FLAGS = {
    "claim_ceiling_live": False,
    "protected_branch_authority_proven": False,
    "status_mutation_performed": False,
    "live_required_context_execution_proven": False,
    "p0_0_green": False,
    "phase0_complete": False,
    "production_ready": False,
    "hyperscaler_grade": False,
}


def load_json(path: str | Path) -> dict[str, Any]:
    with Path(path).open() as fh:
        data = json.load(fh)
    if not isinstance(data, dict):
        raise TypeError(f"{path}: expected JSON object")
    return data


def string_list(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [item for item in value if isinstance(item, str)]


def object_list(value: Any) -> list[dict[str, Any]]:
    if not isinstance(value, list):
        return []
    return [item for item in value if isinstance(item, dict)]


def non_empty(value: Any) -> bool:
    if isinstance(value, str):
        return bool(value.strip())
    if isinstance(value, list):
        return bool(value)
    if isinstance(value, dict):
        return bool(value)
    return value is not None


def row_text(row: dict[str, Any]) -> str:
    parts = []
    for value in row.values():
        if isinstance(value, str):
            parts.append(value)
        elif isinstance(value, list):
            parts.extend(item for item in value if isinstance(item, str))
    return "\n".join(parts)


def term_pattern(term: str) -> re.Pattern[str]:
    pieces = [re.escape(piece) for piece in re.split(r"[-\s]+", term.strip().lower()) if piece]
    if not pieces:
        return re.compile(r"a^")
    joined = r"[-\s]+".join(pieces)
    return re.compile(rf"(?<![A-Za-z0-9_]){joined}(?![A-Za-z0-9_])", re.IGNORECASE)


def detected_terms(text: str, vocabulary: set[str]) -> set[str]:
    return {term for term in vocabulary if term_pattern(term).search(text)}


def allowed_tiers(contract: dict[str, Any]) -> set[str]:
    return {tier.get("tier") for tier in object_list(contract.get("claim_tiers")) if isinstance(tier.get("tier"), str)}


def has_signal(text: str, signals: tuple[str, ...]) -> bool:
    text_lower = text.lower()
    return any(signal.lower() in text_lower for signal in signals)


def validate_rows(rows: list[dict[str, Any]], *, text: str, vocabulary: set[str], tiers: set[str]) -> list[str]:
    violations: list[str] = []
    row_ids: set[str] = set()
    duplicate_ids: set[str] = set()
    covered_terms: set[str] = set()

    for row in rows:
        row_id = row.get("id")
        if isinstance(row_id, str) and row_id:
            if row_id in row_ids:
                duplicate_ids.add(row_id)
            row_ids.add(row_id)
        if any(not non_empty(row.get(field)) for field in REQUIRED_ROW_FIELDS):
            violations.append("missing_or_empty_required_field")

        tier = row.get("claim_tier")
        if tier not in tiers:
            violations.append("unknown_claim_tier")

        row_terms = set(string_list(row.get("regulated_terms")))
        covered_terms.update(row_terms)
        if row_terms - vocabulary:
            violations.append("unknown_regulated_term")

        combined = row_text(row)
        if tier == "mechanically_enforced" or "mechanically enforced" in combined.lower():
            if FORBIDDEN_MECHANICAL_EVIDENCE_RE.search(combined):
                violations.append("forbidden_local_or_oya_evidence_for_mechanical_claim")

        strong_claim_tier = tier in STRONG_CLAIM_TIERS
        if strong_claim_tier:
            # Strong readiness tiers need separately attributable evidence
            # buckets. A single prose entry such as
            # "p95 budget and load result" must not satisfy both requirements.
            evidence_entries = string_list(row.get("current_evidence"))
            budget_only_entries = [
                entry
                for entry in evidence_entries
                if has_signal(entry, BUDGET_SIGNALS) and not has_signal(entry, MEASURED_RESULT_SIGNALS)
            ]
            measured_result_only_entries = [
                entry
                for entry in evidence_entries
                if has_signal(entry, MEASURED_RESULT_SIGNALS) and not has_signal(entry, BUDGET_SIGNALS)
            ]
            if not (budget_only_entries and measured_result_only_entries):
                violations.append("performance_claim_without_budget_or_measured_result")

        if tier in {"target_non_claim", "spec_ready"} and not string_list(row.get("missing_for_next_tier")):
            violations.append("missing_next_tier_gap_for_non_live_claim")

    if duplicate_ids:
        violations.append("duplicate_claim_row_id")

    terms_in_text = detected_terms(text, vocabulary)
    if terms_in_text - covered_terms:
        violations.append("regulated_vocabulary_without_claim_row")

    return sorted(set(violations))


def fixture_paths(claim_map: dict[str, Any], explicit: list[str] | None) -> list[str]:
    if explicit:
        return explicit
    fixture_set = claim_map.get("fixture_set") if isinstance(claim_map.get("fixture_set"), dict) else {}
    return string_list(fixture_set.get("all_fixture_paths"))


def expected_from_fixture(fixture: dict[str, Any]) -> tuple[str, set[str], str]:
    expected_verdict = fixture.get("expected_verdict")
    if expected_verdict not in {"GREEN", "RED"}:
        expected_verdict = "RED"
    expected_violations = set(string_list(fixture.get("expected_violations")))
    fixture_id = fixture.get("fixture_id") if isinstance(fixture.get("fixture_id"), str) else "unknown-fixture"
    return expected_verdict, expected_violations, fixture_id


def validate_fixture(path: str, *, vocabulary: set[str], tiers: set[str]) -> dict[str, Any]:
    fixture = load_json(path)
    expected_verdict, expected_violations, fixture_id = expected_from_fixture(fixture)
    observed = set(
        validate_rows(
            object_list(fixture.get("claim_rows")),
            text=fixture.get("text") if isinstance(fixture.get("text"), str) else "",
            vocabulary=vocabulary,
            tiers=tiers,
        )
    )

    fixture_failures: list[str] = []
    if expected_verdict == "GREEN":
        if observed:
            fixture_failures.append(f"{fixture_id}: GREEN claim-ceiling fixture produced violations {sorted(observed)}")
        if expected_violations:
            fixture_failures.append(f"{fixture_id}: GREEN fixture must not list expected_violations")
    else:
        if not observed:
            fixture_failures.append(f"{fixture_id}: RED claim-ceiling fixture must produce violations")
        missing_expected = sorted(expected_violations - observed)
        if missing_expected:
            fixture_failures.append(f"{fixture_id}: expected violations were not observed {missing_expected}")

    return {
        "path": path,
        "fixture_id": fixture_id,
        "expected_verdict": expected_verdict,
        "expected_violations": sorted(expected_violations),
        "observed_violations": sorted(observed),
        "fixture_passed": not fixture_failures,
        "failures": fixture_failures,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--claim-map", default=DEFAULT_CLAIM_MAP)
    parser.add_argument("--contract", default=DEFAULT_CONTRACT)
    parser.add_argument("--fixture", action="append", default=None, help="Fixture path to evaluate; repeat to override claim map fixture_set")
    parser.add_argument("--text", default="", help="Additional authoritative text to scan against the claim rows")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    claim_map = load_json(args.claim_map)
    contract = load_json(args.contract)
    vocabulary = set(string_list(claim_map.get("regulated_vocabulary")))
    tiers = allowed_tiers(contract)
    rows = object_list(claim_map.get("seed_claim_rows"))

    failures: list[str] = []
    claim_map_violations = validate_rows(rows, text=args.text, vocabulary=vocabulary, tiers=tiers)
    failures.extend(f"claim_map:{violation}" for violation in claim_map_violations)

    fixtures = [validate_fixture(path, vocabulary=vocabulary, tiers=tiers) for path in fixture_paths(claim_map, args.fixture)]
    for fixture in fixtures:
        failures.extend(fixture["failures"])

    result = {
        "authority_boundary": "claim-ceiling local/static fixture evidence only; this checker never posts statuses, mutates branch protection, or claims live readiness authority",
        "claim_map": args.claim_map,
        "contract": args.contract,
        "claim_map_summary": {
            "row_count": len(rows),
            "regulated_vocabulary_count": len(vocabulary),
            "allowed_tiers": sorted(tiers),
            "violations": sorted(set(claim_map_violations)),
        },
        "fixture_results": fixtures,
        "local_fixture_contract_proven": not failures,
        "claim_evidence_map_local_static_proven": not failures,
        **LIVE_FALSE_FLAGS,
        "verdict": "PASS" if not failures else "FAIL",
        "failures": failures,
    }
    rendered = json.dumps(result, sort_keys=True)
    if args.json or not failures:
        print(rendered)
    else:
        print(rendered, file=sys.stderr)
    return 0 if not failures else 1


if __name__ == "__main__":
    raise SystemExit(main())
