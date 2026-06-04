#!/usr/bin/env python3
"""Validate AC-0.1/P0.6/AC-0.7 service-root classifier seed evidence.

This checker is local/static Buck2 evidence only. It proves that the checked-in
service inventory, structural packet catalog, and RED/GREEN fixtures fail closed
for closed-world root drift, legacy service sprawl, retired REAL status tokens,
duplicate services across roots, and underscore crate names. It never posts
statuses, mutates branch protection, proves post-migration pure split, proves a
full nested crate inventory, or claims P0.0/Phase-0 completion.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

DEFAULT_INVENTORY = Path("specs/service-inventory.json")
DEFAULT_PACKETS = Path("specs/phase0-structural-packets.json")
DEFAULT_FIXTURE_DIR = Path("specs/fixtures/phase0-service-root-classifier")
CLOSED_WORLD_ROOTS = {
    "oya",
    "cloud",
    "services",
    "platforms",
    "packs",
    "regional-packs",
    "libs",
    "microservices",
}
CANONICAL_SERVICE_ROOTS = {"oya", "cloud"}
LEGACY_SERVICE_ROOTS = {"services", "platforms", "microservices"}
PACK_ROOTS = {"packs", "regional-packs"}
TARGET_TREES = {"oya", "cloud", "libs"}
FALSE_CLAIMS = (
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "full_service_inventory_coverage_proven",
    "post_migration_pure_split_proven",
    "structural_shards_executed",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
)
FIXTURE_FALSE_CLAIMS = (
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
)
REQUIRED_ENTRY_FIELDS = (
    "non_test_loc",
    "has_main_rs",
    "has_real_storage_adapter",
    "authoritative_toolchain",
    "builds_green",
    "tests_pass",
    "orphan_test_count",
    "cargo_red_set",
    "source_path",
    "target_path",
    "target_tree",
    "migration_class",
)
REQUIRED_PACKET_FIELDS = (
    "packet_id",
    "sub_item",
    "owner_lane",
    "source_paths",
    "target_paths",
    "structural_path_set",
    "depends_on",
    "max_scope",
    "acceptance",
    "verification_commands",
    "rollback_inverse",
    "evidence_bundle",
    "trunk_checkpoint",
)
REQUIRED_PACKET_FAMILIES = (
    "P0.6a-GC-",
    "P0.6b-SPLIT-oya-",
    "P0.6b-SPLIT-cloud-",
    "P0.6b-SPLIT-libs-",
    "P0.6c-ADR0131-",
    "P0.6d-BNF-",
)
SKIP_DIRS = {".git", "buck-out", "target", "node_modules", ".next", "dist", "build", "__pycache__"}
REAL_TOKEN_RE = re.compile(r"\bREAL\b")


def load_json(path: Path) -> dict[str, Any]:
    with path.open() as fh:
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


def display_path(path: Path, root: Path) -> str:
    try:
        return path.relative_to(root).as_posix()
    except ValueError:
        return str(path)


def root_of(path_value: str) -> str:
    return path_value.split("/", 1)[0]


def skipped(path: Path) -> bool:
    return any(part in SKIP_DIRS for part in path.parts)


def observed_direct_child_dirs(root: Path) -> list[str]:
    observed: list[str] = []
    for root_name in sorted(CLOSED_WORLD_ROOTS):
        base = root / root_name
        if not base.is_dir():
            continue
        for child in sorted(base.iterdir()):
            if child.is_dir() and not child.name.startswith(".") and not skipped(child):
                observed.append(child.relative_to(root).as_posix())
    return observed


def validate_false_claims(mapping: dict[str, Any], failures: list[str], *, claims: tuple[str, ...] = FALSE_CLAIMS, prefix: str = "") -> None:
    for claim in claims:
        if mapping.get(claim) is not False:
            failures.append(f"{prefix}forbidden_true_or_missing_claim_{claim}")


def iter_status_maturity_fields(value: Any, key_path: tuple[str, ...] = ()) -> list[tuple[tuple[str, ...], str]]:
    found: list[tuple[tuple[str, ...], str]] = []
    if isinstance(value, dict):
        for key, child in value.items():
            child_path = (*key_path, str(key))
            lowered = str(key).lower()
            if isinstance(child, str) and ("status" in lowered or "maturity" in lowered):
                found.append((child_path, child))
            found.extend(iter_status_maturity_fields(child, child_path))
    elif isinstance(value, list):
        for index, child in enumerate(value):
            found.extend(iter_status_maturity_fields(child, (*key_path, str(index))))
    return found


def validate_real_status_tokens(value: Any, failures: list[str], *, prefix: str = "") -> None:
    for key_path, text in iter_status_maturity_fields(value):
        if REAL_TOKEN_RE.search(text):
            failures.append(f"{prefix}retired_real_token_live_field")
            failures.append(f"{prefix}retired_real_token_live_field:{'.'.join(key_path)}")


def validate_inventory(root: Path, inventory: dict[str, Any]) -> tuple[list[str], dict[str, Any]]:
    failures: list[str] = []
    boundary = inventory.get("claim_boundary") if isinstance(inventory.get("claim_boundary"), dict) else {}
    if boundary.get("service_inventory_published") is not True:
        failures.append("service_inventory_not_published")
    if boundary.get("service_root_classifier_measured") is not True:
        failures.append("service_root_classifier_not_measured")
    if boundary.get("closed_world_root_classifier_measured") is not True:
        failures.append("closed_world_root_classifier_not_measured")
    validate_false_claims(boundary, failures)
    validate_real_status_tokens(inventory, failures)

    required_fields = tuple(string_list(inventory.get("required_entry_fields"))) or REQUIRED_ENTRY_FIELDS
    if set(required_fields) != set(REQUIRED_ENTRY_FIELDS):
        failures.append("required_entry_fields_drift")

    roots = object_list(inventory.get("closed_world_roots"))
    root_names = {item.get("root") for item in roots if isinstance(item.get("root"), str)}
    for missing in sorted(CLOSED_WORLD_ROOTS - root_names):
        failures.append(f"closed_world_root_missing:{missing}")
    for extra in sorted(root_names - CLOSED_WORLD_ROOTS):
        failures.append(f"closed_world_root_unexpected:{extra}")
    for item in roots:
        root_name = item.get("root")
        if not isinstance(root_name, str):
            failures.append("closed_world_root_missing_name")
            continue
        if item.get("target_tree") not in TARGET_TREES:
            failures.append(f"closed_world_root_invalid_target_tree:{root_name}")
        if root_name in CANONICAL_SERVICE_ROOTS and item.get("allows_new_service_dirs") is not True:
            failures.append(f"canonical_service_root_not_allowed:{root_name}")
        if root_name in LEGACY_SERVICE_ROOTS and item.get("observed_direct_child_dir_count", 0) not in (0, None):
            failures.append(f"legacy_service_root_not_empty:{root_name}")
        if root_name in PACK_ROOTS and item.get("allows_new_service_dirs") is not False:
            failures.append(f"pack_root_allows_service_dirs:{root_name}")

    entries = object_list(inventory.get("inventory_entries"))
    entry_paths: set[str] = set()
    for entry in entries:
        source_path = entry.get("source_path") if isinstance(entry.get("source_path"), str) else "<missing-source-path>"
        for field in REQUIRED_ENTRY_FIELDS:
            if field not in entry:
                failures.append(f"{source_path}:missing_required_entry_field:{field}")
        if source_path == "<missing-source-path>":
            failures.append("entry_source_path_missing")
            continue
        entry_paths.add(source_path)
        entry_root = root_of(source_path)
        if entry_root not in CLOSED_WORLD_ROOTS:
            failures.append(f"service_root_outside_closed_world:{source_path}")
        if entry_root in LEGACY_SERVICE_ROOTS:
            failures.append(f"service_layout_sprawl:{source_path}")
        if not (root / source_path).is_dir():
            failures.append(f"inventory_source_path_missing:{source_path}")
        if entry.get("target_tree") not in TARGET_TREES:
            failures.append(f"{source_path}:invalid_target_tree")
        if not isinstance(entry.get("non_test_loc"), int) or entry.get("non_test_loc") < 0:
            failures.append(f"{source_path}:invalid_non_test_loc")
        for bool_field in ("has_main_rs", "has_real_storage_adapter", "builds_green", "tests_pass"):
            if not isinstance(entry.get(bool_field), bool):
                failures.append(f"{source_path}:invalid_bool_field:{bool_field}")
        if entry.get("builds_green") is True:
            failures.append(f"{source_path}:entry_claims_builds_green_without_live_context")
        if entry.get("tests_pass") is True:
            failures.append(f"{source_path}:entry_claims_tests_pass_without_live_context")
        if not isinstance(entry.get("orphan_test_count"), int) or entry.get("orphan_test_count") < 0:
            failures.append(f"{source_path}:invalid_orphan_test_count")
        if not isinstance(entry.get("cargo_red_set"), list):
            failures.append(f"{source_path}:invalid_cargo_red_set")
        if "_" in Path(source_path).name:
            failures.append(f"underscore_crate_name:{source_path}")

    for path, count in Counter(entry.get("source_path") for entry in entries).items():
        if isinstance(path, str) and count > 1:
            failures.append(f"duplicate_inventory_source_path:{path}")

    observed = set(observed_direct_child_dirs(root))
    missing_entries = sorted(observed - entry_paths)
    extra_entries = sorted(entry_paths - observed)
    for path in missing_entries:
        failures.append(f"service_inventory_entry_missing:{path}")
    for path in extra_entries:
        failures.append(f"service_inventory_entry_without_observed_path:{path}")

    summary = {
        "closed_world_root_count": len(root_names),
        "inventory_entry_count": len(entries),
        "observed_direct_child_dir_count": len(observed),
        "missing_inventory_entry_count": len(missing_entries),
    }
    return failures, summary


def validate_packets(packets_spec: dict[str, Any] | None, failures: list[str], *, packets_override: list[dict[str, Any]] | None = None, prefix: str = "") -> dict[str, Any]:
    if packets_override is None:
        packets_spec = packets_spec or {}
        boundary = packets_spec.get("claim_boundary") if isinstance(packets_spec.get("claim_boundary"), dict) else {}
        if boundary.get("structural_packet_catalog_published") is not True:
            failures.append(f"{prefix}structural_packet_catalog_not_published")
        if boundary.get("service_root_classifier_measured") is not True:
            failures.append(f"{prefix}service_root_classifier_not_measured_in_packets")
        validate_false_claims(boundary, failures, prefix=prefix)
        required_fields = tuple(string_list(packets_spec.get("required_packet_fields"))) or REQUIRED_PACKET_FIELDS
        required_families = tuple(string_list(packets_spec.get("required_packet_families"))) or REQUIRED_PACKET_FAMILIES
        packets = object_list(packets_spec.get("structural_packets"))
    else:
        required_fields = REQUIRED_PACKET_FIELDS
        required_families = REQUIRED_PACKET_FAMILIES
        packets = packets_override

    if set(required_fields) != set(REQUIRED_PACKET_FIELDS):
        failures.append(f"{prefix}required_packet_fields_drift")

    packet_ids: list[str] = []
    for packet in packets:
        packet_id = packet.get("packet_id") if isinstance(packet.get("packet_id"), str) else "<missing-packet-id>"
        packet_ids.append(packet_id)
        for field in REQUIRED_PACKET_FIELDS:
            if field not in packet:
                failures.append(f"{prefix}structural_packet_missing_required_field:{packet_id}:{field}")
        for field in ("source_paths", "target_paths", "structural_path_set", "depends_on", "verification_commands"):
            if not string_list(packet.get(field)):
                failures.append(f"{prefix}structural_packet_empty_list_field:{packet_id}:{field}")
        commands = "\n".join(string_list(packet.get("verification_commands")))
        if "//:service-root-classifier-check" not in commands:
            failures.append(f"{prefix}structural_packet_missing_classifier_command:{packet_id}")
        if re.search(r"(^|\s)(?:bin/)?oya\s+(?:verify|gate)\b", commands, re.IGNORECASE):
            failures.append(f"{prefix}structural_packet_maps_to_oya_cli:{packet_id}")
        for text_field in ("rollback_inverse", "trunk_checkpoint", "acceptance", "max_scope", "evidence_bundle"):
            if not isinstance(packet.get(text_field), str) or not packet.get(text_field, "").strip():
                failures.append(f"{prefix}structural_packet_missing_text_field:{packet_id}:{text_field}")

    for packet_id, count in Counter(packet_ids).items():
        if packet_id != "<missing-packet-id>" and count > 1:
            failures.append(f"{prefix}duplicate_structural_packet_id:{packet_id}")

    for family in required_families:
        if not any(packet_id.startswith(family) for packet_id in packet_ids):
            failures.append(f"{prefix}structural_packet_missing_required_family")
            failures.append(f"{prefix}structural_packet_missing_required_family:{family}")

    return {"structural_packet_count": len(packets), "required_packet_family_count": len(required_families)}


def validate_fixture(fixture: dict[str, Any], inventory_roots: set[str], packets_spec: dict[str, Any]) -> dict[str, Any]:
    fixture_id = fixture.get("fixture_id") if isinstance(fixture.get("fixture_id"), str) else "<missing-fixture-id>"
    expected_verdict = fixture.get("expected_verdict")
    if expected_verdict not in {"GREEN", "RED"}:
        expected_verdict = "RED"
    expected_violations = set(string_list(fixture.get("expected_violations")))
    observed: list[str] = []

    boundary = fixture.get("claim_boundary") if isinstance(fixture.get("claim_boundary"), dict) else {}
    validate_false_claims(boundary, observed, claims=FIXTURE_FALSE_CLAIMS)
    validate_real_status_tokens(fixture.get("live_status_fields", {}), observed)

    candidates = object_list(fixture.get("candidate_paths"))
    inventory_entry_paths = set(string_list(fixture.get("inventory_entry_paths")))
    service_roots_by_id: dict[str, set[str]] = defaultdict(set)
    for candidate in candidates:
        path_value = candidate.get("path") if isinstance(candidate.get("path"), str) else "<missing-path>"
        kind = candidate.get("kind") if isinstance(candidate.get("kind"), str) else "service"
        candidate_root = root_of(path_value)
        if candidate_root not in inventory_roots:
            observed.append("service_root_outside_closed_world")
        if path_value not in inventory_entry_paths:
            observed.append("service_inventory_entry_missing")
        if kind == "service" and candidate_root not in CANONICAL_SERVICE_ROOTS:
            observed.append("service_layout_sprawl")
        if kind == "pack" and candidate_root not in PACK_ROOTS:
            observed.append("service_layout_sprawl")
        crate_name = candidate.get("crate_name") if isinstance(candidate.get("crate_name"), str) else Path(path_value).name
        if "_" in crate_name or "_" in Path(path_value).name:
            observed.append("underscore_crate_name")
        service_id = candidate.get("service_id") if isinstance(candidate.get("service_id"), str) else Path(path_value).name
        service_roots_by_id[service_id].add(candidate_root)

    for roots in service_roots_by_id.values():
        if len(roots) > 1:
            observed.append("duplicate_service_across_roots")

    if "structural_packets" in fixture:
        validate_packets(None, observed, packets_override=object_list(fixture.get("structural_packets")), prefix="")

    observed_set = set(observed)
    fixture_failures: list[str] = []
    if expected_verdict == "GREEN":
        if observed_set:
            fixture_failures.append(f"{fixture_id}: GREEN service-root fixture produced violations {sorted(observed_set)}")
        if expected_violations:
            fixture_failures.append(f"{fixture_id}: GREEN fixture must not list expected_violations")
    else:
        if not observed_set:
            fixture_failures.append(f"{fixture_id}: RED service-root fixture must produce violations")
        missing_expected = sorted(expected_violations - observed_set)
        if missing_expected:
            fixture_failures.append(f"{fixture_id}: expected violations were not observed {missing_expected}")

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
    parser.add_argument("--inventory", default=str(DEFAULT_INVENTORY))
    parser.add_argument("--packets", default=str(DEFAULT_PACKETS))
    parser.add_argument("--fixture", action="append", default=None)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    root = Path(args.repo_root).resolve()
    inventory_path = Path(args.inventory)
    if not inventory_path.is_absolute():
        inventory_path = root / inventory_path
    packets_path = Path(args.packets)
    if not packets_path.is_absolute():
        packets_path = root / packets_path

    failures: list[str] = []
    if not inventory_path.is_file():
        failures.append("missing_service_inventory_spec")
        inventory: dict[str, Any] = {}
        inventory_summary = {"closed_world_root_count": 0, "inventory_entry_count": 0, "observed_direct_child_dir_count": 0, "missing_inventory_entry_count": 0}
    else:
        inventory = load_json(inventory_path)
        inventory_failures, inventory_summary = validate_inventory(root, inventory)
        failures.extend(inventory_failures)

    if not packets_path.is_file():
        failures.append("missing_structural_packets_spec")
        packets_spec: dict[str, Any] = {}
        packet_summary = {"structural_packet_count": 0, "required_packet_family_count": 0}
    else:
        packets_spec = load_json(packets_path)
        packet_summary = validate_packets(packets_spec, failures)

    inventory_roots = {item.get("root") for item in object_list(inventory.get("closed_world_roots")) if isinstance(item.get("root"), str)} or CLOSED_WORLD_ROOTS
    fixture_results: list[dict[str, Any]] = []
    for path in fixture_paths(root, args.fixture):
        if not path.is_file():
            failures.append(f"fixture_path_missing:{display_path(path, root)}")
            continue
        result = validate_fixture(load_json(path), inventory_roots, packets_spec)
        result["path"] = display_path(path, root)
        fixture_results.append(result)
        failures.extend(result["failures"])

    expected_green = sum(1 for item in fixture_results if item["expected_verdict"] == "GREEN")
    expected_red = sum(1 for item in fixture_results if item["expected_verdict"] == "RED")
    result = {
        "authority_boundary": "AC-0.1/P0.6/AC-0.7 local/static service-root classifier evidence only; no status mutation, live required-context authority, post-migration pure split, P0.0 green, Phase-0 completion, production readiness, or hyperscaler-grade readiness proven",
        "service_inventory_published": inventory.get("claim_boundary", {}).get("service_inventory_published") is True,
        "service_root_classifier_measured": not failures,
        "closed_world_root_classifier_measured": inventory.get("claim_boundary", {}).get("closed_world_root_classifier_measured") is True,
        "structural_packet_catalog_published": packets_spec.get("claim_boundary", {}).get("structural_packet_catalog_published") is True,
        **inventory_summary,
        **packet_summary,
        "fixture_count": len(fixture_results),
        "expected_green_fixture_count": expected_green,
        "expected_red_fixture_count": expected_red,
        "fixture_results": fixture_results,
        "status_mutation_performed": False,
        "protected_branch_authority_proven": False,
        "live_required_context_execution_proven": False,
        "full_service_inventory_coverage_proven": False,
        "post_migration_pure_split_proven": False,
        "structural_shards_executed": False,
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
