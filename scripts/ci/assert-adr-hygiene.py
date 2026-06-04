#!/usr/bin/env python3
"""Validate AC-0.3 ADR numbering, supersession, and active-doc hygiene.

This checker is local/static fixture evidence only. It scans ADR frontmatter for
duplicate ids, verifies the ADR-0377 renumbering and ADR-0511 -> ADR-0513
supersession contract, and lints active docs for stale canonical references to
superseded decisions. It never mutates branch protection, posts statuses,
regenerates the full ADR index, or claims P0.0/Phase-0 completion.
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import re
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any

DEFAULT_REGISTRY = Path("specs/adr-hygiene-registry.json")
DEFAULT_FIXTURE_DIR = Path("specs/fixtures/phase0-adr-hygiene")
FALSE_CLAIMS = (
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "full_adr_index_regenerated",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
)
FIXTURE_FALSE_CLAIMS = (
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "full_adr_index_regenerated",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
)


def load_json(path: Path) -> dict[str, Any]:
    with path.open() as fh:
        data = json.load(fh)
    if not isinstance(data, dict):
        raise TypeError(f"{path}: expected JSON object")
    return data


def string_list(value: Any) -> list[str]:
    return [item for item in value if isinstance(item, str)] if isinstance(value, list) else []


def object_list(value: Any) -> list[dict[str, Any]]:
    return [item for item in value if isinstance(item, dict)] if isinstance(value, list) else []


def display_path(path: Path, root: Path) -> str:
    try:
        return path.relative_to(root).as_posix()
    except ValueError:
        return str(path)


def parse_frontmatter_list(raw: str) -> list[str]:
    value = raw.strip()
    if not value:
        return []
    if value.startswith("[") and value.endswith("]"):
        inner = value[1:-1].strip()
        if not inner:
            return []
        return [item.strip().strip('"\'') for item in inner.split(",") if item.strip()]
    return [value.strip().strip('"\'')]


def parse_adr_frontmatter(path: Path, root: Path) -> dict[str, Any]:
    text = path.read_text(errors="replace")
    fm: dict[str, Any] = {}
    if text.startswith("---\n"):
        end = text.find("\n---", 4)
        if end != -1:
            for line in text[4:end].splitlines():
                if not line.strip() or line.startswith(" ") or ":" not in line:
                    continue
                key, value = line.split(":", 1)
                key = key.strip()
                value = value.strip()
                if key in {"superseded_by", "supersedes", "related"}:
                    fm[key] = parse_frontmatter_list(value)
                else:
                    fm[key] = value.strip('"')
    filename_id = None
    match = re.match(r"(ADR-\d{4})", path.name)
    if match:
        filename_id = match.group(1)
    return {
        "path": display_path(path, root),
        "id": fm.get("id") or filename_id or "<missing-id>",
        "filename_id": filename_id,
        "status": fm.get("status", ""),
        "superseded_by": fm.get("superseded_by", []),
        "renumbered_from": fm.get("renumbered_from", ""),
    }


def repo_decision_records(root: Path) -> list[dict[str, Any]]:
    return [parse_adr_frontmatter(path, root) for path in sorted((root / "docs/decisions").glob("ADR-*.md"))]


def repo_active_documents(root: Path, globs: list[str], exclude_globs: list[str]) -> list[dict[str, str]]:
    docs: list[dict[str, str]] = []
    for pattern in globs:
        for path in sorted(root.glob(pattern)):
            if not path.is_file():
                continue
            rel = display_path(path, root)
            if rel.startswith("docs/decisions/") or rel.startswith("docs/machine-readable/"):
                continue
            if any(fnmatch.fnmatch(rel, excluded) for excluded in exclude_globs):
                continue
            docs.append({"path": rel, "content": path.read_text(errors="replace")})
    # Stable de-dupe when globs overlap.
    seen: set[str] = set()
    unique: list[dict[str, str]] = []
    for doc in docs:
        if doc["path"] in seen:
            continue
        seen.add(doc["path"])
        unique.append(doc)
    return unique


def validate_false_claims(mapping: dict[str, Any], failures: list[str], *, claims: tuple[str, ...] = FALSE_CLAIMS, prefix: str = "") -> None:
    for claim in claims:
        if mapping.get(claim) is not False:
            failures.append(f"{prefix}forbidden_true_or_missing_claim_{claim}")


def compile_patterns(registry: dict[str, Any]) -> list[dict[str, Any]]:
    patterns: list[dict[str, Any]] = []
    for item in object_list(registry.get("superseded_reference_patterns")):
        try:
            regex = re.compile(str(item.get("pattern", "")))
        except re.error as exc:
            patterns.append({"id": item.get("id", "<missing-id>"), "error": str(exc)})
            continue
        patterns.append({**item, "regex": regex})
    return patterns


def validate_dataset(records: list[dict[str, Any]], active_docs: list[dict[str, str]], registry: dict[str, Any], *, enforce_required_records: bool) -> list[str]:
    failures: list[str] = []
    by_id: dict[str, list[str]] = defaultdict(list)
    by_path: dict[str, dict[str, Any]] = {}
    for record in records:
        rid = record.get("id") if isinstance(record.get("id"), str) else "<missing-id>"
        path = record.get("path") if isinstance(record.get("path"), str) else "<missing-path>"
        by_id[rid].append(path)
        by_path[path] = record
        filename_id = record.get("filename_id")
        if filename_id and filename_id != rid:
            failures.append("adr_filename_id_mismatch")
            failures.append(f"adr_filename_id_mismatch:{path}:{filename_id}!={rid}")
    for rid, paths in by_id.items():
        if len(paths) > 1:
            failures.append("duplicate_adr_number")
            failures.append(f"duplicate_adr_number:{rid}:{','.join(sorted(paths))}")

    renumbering = registry.get("renumbering_contract") if isinstance(registry.get("renumbering_contract"), dict) else {}
    kept = renumbering.get("kept_decision_path")
    renumbered = renumbering.get("renumbered_decision_path")
    if enforce_required_records:
        if kept and kept not in by_path:
            failures.append("adr_0377_kept_decision_missing")
        if renumbered:
            record = by_path.get(renumbered)
            if not record:
                failures.append("renumbered_adr_decision_missing")
            else:
                if record.get("id") != renumbering.get("renumbered_to"):
                    failures.append("renumbered_adr_id_mismatch")
                if record.get("renumbered_from") != renumbering.get("renumbered_from"):
                    failures.append("renumbered_adr_missing_renumbered_from")
        for forbidden_path in string_list(renumbering.get("forbidden_live_paths")):
            if forbidden_path in by_path:
                failures.append("forbidden_duplicate_adr_path_present")
    elif renumbered in by_path:
        record = by_path[renumbered]
        if record.get("id") != renumbering.get("renumbered_to"):
            failures.append("renumbered_adr_id_mismatch")
        if record.get("renumbered_from") != renumbering.get("renumbered_from"):
            failures.append("renumbered_adr_missing_renumbered_from")

    for contract in object_list(registry.get("supersession_contracts")):
        decision_id = contract.get("decision_id")
        required = contract.get("required_superseded_by")
        status_contains = contract.get("required_status_contains")
        matches = [record for record in records if record.get("id") == decision_id]
        if not matches:
            if enforce_required_records:
                failures.append(f"supersession_decision_missing:{decision_id}")
            continue
        for record in matches:
            if isinstance(status_contains, str) and status_contains not in str(record.get("status", "")):
                failures.append(f"{decision_id.lower()}_status_not_superseded")
            if isinstance(required, str) and required not in string_list(record.get("superseded_by")):
                failures.append(f"{decision_id.lower()}_missing_superseded_by_{required.lower()}")
                if decision_id == "ADR-0511" and required == "ADR-0513":
                    failures.append("adr_0511_missing_superseded_by_adr_0513")

    for pattern in compile_patterns(registry):
        if "error" in pattern:
            failures.append(f"invalid_superseded_reference_pattern:{pattern['id']}")
            continue
        regex = pattern["regex"]
        pid = str(pattern.get("id", "<missing-pattern-id>"))
        for doc in active_docs:
            content = doc.get("content", "")
            path = doc.get("path", "<missing-path>")
            if regex.search(content):
                failures.append("superseded_reference_active_doc")
                failures.append(f"superseded_reference_active_doc:{pid}:{path}")
    return failures


def validate_registry(root: Path, registry: dict[str, Any]) -> tuple[list[str], dict[str, Any]]:
    failures: list[str] = []
    boundary = registry.get("claim_boundary") if isinstance(registry.get("claim_boundary"), dict) else {}
    for required_true in (
        "adr_hygiene_registry_published",
        "adr_hygiene_fixture_contract_measured",
        "duplicate_adr_0377_resolved",
        "adr_0511_superseded_by_adr_0513",
        "superseded_reference_lint_measured",
    ):
        if boundary.get(required_true) is not True:
            failures.append(f"claim_boundary_missing_true_{required_true}")
    validate_false_claims(boundary, failures)

    records = repo_decision_records(root)
    active_docs = repo_active_documents(root, string_list(registry.get("active_doc_scan_globs")), string_list(registry.get("active_doc_scan_exclude_globs")))
    failures.extend(validate_dataset(records, active_docs, registry, enforce_required_records=True))
    summary = {
        "decision_record_count": len(records),
        "active_doc_scan_count": len(active_docs),
        "superseded_reference_pattern_count": len(object_list(registry.get("superseded_reference_patterns"))),
    }
    return failures, summary


def validate_fixture(root: Path, fixture: dict[str, Any], registry: dict[str, Any]) -> dict[str, Any]:
    fixture_id = fixture.get("fixture_id") if isinstance(fixture.get("fixture_id"), str) else "<missing-fixture-id>"
    expected_verdict = fixture.get("expected_verdict")
    if expected_verdict not in {"GREEN", "RED"}:
        expected_verdict = "RED"
    expected_violations = set(string_list(fixture.get("expected_violations")))
    observed: list[str] = []
    boundary = fixture.get("claim_boundary") if isinstance(fixture.get("claim_boundary"), dict) else {}
    validate_false_claims(boundary, observed, claims=FIXTURE_FALSE_CLAIMS)
    observed.extend(validate_dataset(object_list(fixture.get("decision_records")), object_list(fixture.get("active_documents")), registry, enforce_required_records=False))

    observed_set = set(observed)
    for item in list(observed):
        if item.startswith("duplicate_adr_number:"):
            observed_set.add("duplicate_adr_number")
        if item.startswith("superseded_reference_active_doc:"):
            observed_set.add("superseded_reference_active_doc")
        if item.startswith("adr_filename_id_mismatch:"):
            observed_set.add("adr_filename_id_mismatch")

    fixture_failures: list[str] = []
    if expected_verdict == "GREEN":
        if observed_set:
            fixture_failures.append(f"{fixture_id}: GREEN ADR hygiene fixture produced violations {sorted(observed_set)}")
        if expected_violations:
            fixture_failures.append(f"{fixture_id}: GREEN fixture must not list expected_violations")
    else:
        if not observed_set:
            fixture_failures.append(f"{fixture_id}: RED ADR hygiene fixture must produce violations")
        missing = sorted(expected_violations - observed_set)
        if missing:
            fixture_failures.append(f"{fixture_id}: expected violations were not observed {missing}")
    return {
        "fixture_id": fixture_id,
        "expected_verdict": expected_verdict,
        "expected_violations": sorted(expected_violations),
        "observed_violations": sorted(observed_set),
        "fixture_passed": not fixture_failures,
        "failures": fixture_failures,
    }


def fixture_paths(root: Path, explicit: list[str] | None) -> list[Path]:
    if explicit:
        return [Path(item) if Path(item).is_absolute() else root / item for item in explicit]
    return sorted((root / DEFAULT_FIXTURE_DIR).glob("*.json"))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--registry", default=str(DEFAULT_REGISTRY))
    parser.add_argument("--fixture", action="append", default=None)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    root = Path(args.repo_root).resolve()
    registry_path = Path(args.registry)
    if not registry_path.is_absolute():
        registry_path = root / registry_path

    failures: list[str] = []
    if not registry_path.is_file():
        registry: dict[str, Any] = {}
        registry_summary = {"decision_record_count": 0, "active_doc_scan_count": 0, "superseded_reference_pattern_count": 0}
        failures.append("missing_adr_hygiene_registry")
    else:
        registry = load_json(registry_path)
        registry_failures, registry_summary = validate_registry(root, registry)
        failures.extend(registry_failures)

    fixture_results: list[dict[str, Any]] = []
    for path in fixture_paths(root, args.fixture):
        if not path.is_file():
            failures.append(f"fixture_path_missing:{display_path(path, root)}")
            continue
        result = validate_fixture(root, load_json(path), registry)
        result["path"] = display_path(path, root)
        fixture_results.append(result)
        failures.extend(result["failures"])

    expected_green = sum(1 for item in fixture_results if item["expected_verdict"] == "GREEN")
    expected_red = sum(1 for item in fixture_results if item["expected_verdict"] == "RED")
    result = {
        "authority_boundary": "AC-0.3 local/static ADR hygiene evidence only; no status mutation, live required-context authority, full ADR index regeneration, P0.0 green, Phase-0 completion, production readiness, or hyperscaler-grade readiness proven",
        "adr_hygiene_registry_published": registry.get("claim_boundary", {}).get("adr_hygiene_registry_published") is True,
        "adr_hygiene_fixture_contract_measured": not failures,
        **registry_summary,
        "fixture_count": len(fixture_results),
        "expected_green_fixture_count": expected_green,
        "expected_red_fixture_count": expected_red,
        "fixture_results": fixture_results,
        "status_mutation_performed": False,
        "protected_branch_authority_proven": False,
        "live_required_context_execution_proven": False,
        "full_adr_index_regenerated": False,
        "p0_0_green": False,
        "phase0_complete": False,
        "production_ready": False,
        "hyperscaler_grade": False,
        "verdict": "PASS" if not failures else "FAIL",
        "failures": sorted(set(failures)),
    }
    rendered = json.dumps(result, sort_keys=True)
    if args.json or not failures:
        print(rendered)
    else:
        print(rendered, file=sys.stderr)
    return 0 if not failures else 1


if __name__ == "__main__":
    raise SystemExit(main())
