#!/usr/bin/env python3
"""Measure Cargo workspace target roots against Buck2 rust crate_root mappings.

This AC-0.13 checker is local/static target-coverage evidence only. It does not
run Cargo, build Rust crates, generate source-line coverage, post statuses,
mutate branch protection, or prove Phase-0 exit authority.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
from pathlib import Path
from typing import Any

DEFAULT_SPEC = Path("specs/buck2-cargo-target-coverage.json")
FALSE_CLAIMS = (
    "source_line_coverage_generated",
    "mutation_lane_implemented",
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
)
REQUIRED_URLS = {
    "https://doc.rust-lang.org/cargo/reference/workspaces.html",
    "https://doc.rust-lang.org/cargo/reference/cargo-targets.html",
    "https://buck2.build/docs/users/commands/",
    "https://buck2.build/docs/about/bootstrapping/",
    "https://github.com/facebookincubator/reindeer",
}
CRATE_ROOT_RE = re.compile(r"crate_root\s*=\s*\"([^\"]+)\"")


def load_json(path: Path) -> dict[str, Any]:
    with path.open() as fh:
        data = json.load(fh)
    if not isinstance(data, dict):
        raise TypeError(f"{path}: expected JSON object")
    return data


def load_toml(path: Path) -> dict[str, Any]:
    return tomllib.loads(path.read_text())


def rel(path: Path, root: Path) -> str:
    return path.relative_to(root).as_posix()


def official_urls(spec: dict[str, Any]) -> set[str]:
    refs = spec.get("_meta", {}).get("official_references", [])
    return {ref.get("url") for ref in refs if isinstance(ref, dict) and isinstance(ref.get("url"), str)}


def manual_targets_defined(manifest: dict[str, Any]) -> bool:
    return any(key in manifest for key in ("lib", "bin", "example", "test", "bench"))


def package_edition(manifest: dict[str, Any]) -> str:
    package = manifest.get("package") if isinstance(manifest.get("package"), dict) else {}
    return str(package.get("edition") or "2015")


def auto_discovery_enabled(manifest: dict[str, Any], key: str) -> bool:
    package = manifest.get("package") if isinstance(manifest.get("package"), dict) else {}
    if key in package:
        return bool(package[key])
    # Cargo kept automatic target discovery disabled by default for packages on
    # the 2015 edition when any target was manually listed; newer editions keep
    # discovery enabled unless the package explicitly opts out with auto* = false.
    if package_edition(manifest) == "2015" and manual_targets_defined(manifest):
        return False
    return True


def dedupe_targets(targets: list[dict[str, str]]) -> list[dict[str, str]]:
    deduped: list[dict[str, str]] = []
    seen: set[tuple[str, str]] = set()
    for target in targets:
        key = (target["kind"], target["path"])
        if key in seen:
            continue
        seen.add(key)
        deduped.append(target)
    return deduped


def discovered_src_bin_targets(member_dir: Path) -> list[dict[str, str]]:
    src_bin = member_dir / "src" / "bin"
    targets: list[dict[str, str]] = []
    if not src_bin.is_dir():
        return targets
    for rust_file in sorted(src_bin.glob("*.rs")):
        targets.append({"kind": "bin", "name": rust_file.stem, "path": f"src/bin/{rust_file.name}"})
    for main_rs in sorted(src_bin.glob("*/main.rs")):
        targets.append({"kind": "bin", "name": main_rs.parent.name, "path": f"src/bin/{main_rs.parent.name}/main.rs"})
    return targets


def cargo_target_roots(member_dir: Path, manifest: dict[str, Any]) -> list[dict[str, str]]:
    roots: list[dict[str, str]] = []
    package_name = manifest.get("package", {}).get("name", member_dir.name)
    lib = manifest.get("lib")
    if isinstance(lib, dict) and isinstance(lib.get("path"), str):
        roots.append({"kind": "lib", "name": str(lib.get("name") or package_name), "path": lib["path"]})
    elif auto_discovery_enabled(manifest, "autolib") and (member_dir / "src" / "lib.rs").is_file():
        roots.append({"kind": "lib", "name": str(package_name), "path": "src/lib.rs"})

    bins = manifest.get("bin") if isinstance(manifest.get("bin"), list) else []
    for index, bin_target in enumerate(bins):
        if isinstance(bin_target, dict) and isinstance(bin_target.get("path"), str):
            roots.append(
                {
                    "kind": "bin",
                    "name": str(bin_target.get("name") or package_name or f"bin-{index}"),
                    "path": bin_target["path"],
                }
            )
    if auto_discovery_enabled(manifest, "autobins"):
        if (member_dir / "src" / "main.rs").is_file():
            roots.append({"kind": "bin", "name": str(package_name), "path": "src/main.rs"})
        roots.extend(discovered_src_bin_targets(member_dir))
    return dedupe_targets(roots)


def expand_workspace_members(root: Path, workspace: dict[str, Any]) -> tuple[list[str], list[str]]:
    raw_members = workspace.get("members", []) if isinstance(workspace.get("members"), list) else []
    excludes = {
        (root / item).resolve()
        for item in workspace.get("exclude", [])
        if isinstance(item, str)
    } if isinstance(workspace.get("exclude"), list) else set()
    members: list[str] = []
    failures: list[str] = []
    for member in raw_members:
        if not isinstance(member, str):
            failures.append("workspace_member_not_string")
            continue
        if any(token in member for token in "*?["):
            matches = sorted(path for path in root.glob(member) if path.is_dir() and (path / "Cargo.toml").is_file())
            if not matches:
                failures.append(f"workspace_member_glob_matched_nothing:{member}")
            for path in matches:
                if path.resolve() not in excludes:
                    members.append(rel(path, root))
        else:
            path = root / member
            if path.resolve() not in excludes:
                members.append(member)
    return sorted(dict.fromkeys(members)), failures


def iter_buck_files(root: Path):
    for buck in root.glob("**/BUCK"):
        if not buck.is_file():
            continue
        if any(part in {".git", "buck-out", "target"} for part in buck.parts):
            continue
        yield buck


def buck_crate_roots(root: Path) -> dict[str, list[str]]:
    mappings: dict[str, list[str]] = {}
    for buck in iter_buck_files(root):
        text = buck.read_text(errors="replace")
        for match in CRATE_ROOT_RE.finditer(text):
            target_root = (buck.parent / match.group(1)).resolve().relative_to(root.resolve()).as_posix()
            mappings.setdefault(target_root, []).append(f"{rel(buck, root)}:{match.group(1)}")
    return mappings


def validate_spec(spec: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    boundary = spec.get("claim_boundary") if isinstance(spec.get("claim_boundary"), dict) else {}
    contract = spec.get("measurement_contract") if isinstance(spec.get("measurement_contract"), dict) else {}
    if boundary.get("target_coverage_measured") is not True:
        failures.append("target_coverage_measurement_not_recorded")
    for claim in FALSE_CLAIMS:
        if boundary.get(claim) is not False:
            failures.append(f"forbidden_true_or_missing_claim_{claim}")
    if contract.get("buck2_target") != "//:buck2-cargo-target-coverage-check":
        failures.append("wrong_buck2_target")
    if contract.get("checker") != "scripts/ci/assert-buck2-cargo-target-coverage.py":
        failures.append("wrong_checker_path")
    if contract.get("workspace_manifest") != "Cargo.toml":
        failures.append("wrong_workspace_manifest")
    if contract.get("parent_buck_allowed") is not True:
        failures.append("parent_buck_mapping_not_allowed")
    if "crate_root" not in str(contract.get("buck2_mapping_rule", "")):
        failures.append("missing_crate_root_mapping_rule")
    auto_rule = str(contract.get("cargo_autodiscovery_rule", ""))
    if "autobins" not in auto_rule or "src/bin" not in "\n".join(str(item) for item in contract.get("cargo_target_roots", [])):
        failures.append("missing_cargo_bin_autodiscovery_rule")
    forbidden = "\n".join(str(item) for item in contract.get("forbidden_authority", []))
    if "source-line coverage claims" not in forbidden:
        failures.append("source_line_claim_forbidden_authority_missing")
    if "protected branch authority" not in forbidden:
        failures.append("protected_branch_forbidden_authority_missing")
    for url in REQUIRED_URLS:
        if url not in official_urls(spec):
            failures.append(f"missing_official_reference_{url}")
    automated_chain = "\n".join(str(item) for item in spec.get("automated_chain", []))
    if "buck2 build //:buck2-cargo-target-coverage-check" not in automated_chain:
        failures.append("missing_buck2_target_in_automated_chain")
    return failures


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", default=".", help="repository root")
    parser.add_argument("--cargo-toml", default="Cargo.toml", help="workspace Cargo.toml path relative to repo root")
    parser.add_argument("--spec", default=str(DEFAULT_SPEC), help="coverage contract spec path")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    root = Path(args.repo_root).resolve()
    cargo_toml = Path(args.cargo_toml)
    if not cargo_toml.is_absolute():
        cargo_toml = root / cargo_toml
    spec_path = Path(args.spec)
    if not spec_path.is_absolute():
        spec_path = root / spec_path

    failures: list[str] = []
    if not cargo_toml.is_file():
        failures.append("missing_workspace_manifest")
        workspace_members: list[str] = []
    else:
        workspace = load_toml(cargo_toml).get("workspace", {})
        workspace_members, member_failures = expand_workspace_members(root, workspace if isinstance(workspace, dict) else {})
        failures.extend(member_failures)
        if not workspace_members:
            failures.append("workspace_members_missing")

    spec = load_json(spec_path) if spec_path.is_file() else {}
    if not spec:
        failures.append("missing_contract_spec")
    else:
        failures.extend(validate_spec(spec))
    contract = spec.get("measurement_contract", {}) if isinstance(spec.get("measurement_contract"), dict) else {}
    known_divergences = contract.get("known_divergences", []) if isinstance(contract.get("known_divergences"), list) else []
    known_by_root = {
        item.get("cargo_target_root"): item
        for item in known_divergences
        if isinstance(item, dict) and isinstance(item.get("cargo_target_root"), str)
    }

    buck_roots = buck_crate_roots(root)
    cargo_targets: list[dict[str, Any]] = []
    missing_mappings: list[dict[str, Any]] = []
    for member in workspace_members:
        member_dir = root / member
        member_manifest = member_dir / "Cargo.toml"
        if not member_dir.is_dir() or not member_manifest.is_file():
            failures.append(f"workspace_member_path_missing:{member}")
            continue
        manifest = load_toml(member_manifest)
        for target in cargo_target_roots(member_dir, manifest):
            target_root = (member_dir / target["path"]).resolve().relative_to(root).as_posix()
            record = {"member": member, **target, "cargo_target_root": target_root}
            cargo_targets.append(record)
            if target_root not in buck_roots:
                missing_mappings.append(record)

    allowed_missing = {root for root in known_by_root if root}
    actual_missing = {item["cargo_target_root"] for item in missing_mappings}
    unregistered_missing = sorted(actual_missing - allowed_missing)
    stale_divergences = sorted(allowed_missing - actual_missing)
    if unregistered_missing:
        failures.append("missing_buck2_target_root_mapping")
    if stale_divergences:
        failures.append("stale_known_divergence")
    if known_divergences:
        for item in known_divergences:
            if not isinstance(item, dict) or not item.get("owner") or not item.get("retirement_phase"):
                failures.append("known_divergence_missing_owner_or_retirement")

    result = {
        "authority_boundary": "local/static AC-0.13 target-coverage measurement only; no source-line coverage, status mutation, protected-branch authority, P0.0 green, or Phase-0 completion proven",
        "target_coverage_measured": not failures,
        "source_line_coverage_generated": False,
        "mutation_lane_implemented": False,
        "status_mutation_performed": False,
        "protected_branch_authority_proven": False,
        "live_required_context_execution_proven": False,
        "p0_0_green": False,
        "phase0_complete": False,
        "production_ready": False,
        "hyperscaler_grade": False,
        "workspace_manifest": rel(cargo_toml, root) if cargo_toml.exists() else str(cargo_toml),
        "workspace_member_count": len(workspace_members),
        "cargo_target_root_count": len(cargo_targets),
        "buck2_mapped_target_root_count": len({target["cargo_target_root"] for target in cargo_targets if target["cargo_target_root"] in buck_roots}),
        "buck_file_count": len(list(iter_buck_files(root))),
        "known_divergence_count": len(known_divergences),
        "missing_mappings": missing_mappings,
        "unregistered_missing_target_roots": unregistered_missing,
        "stale_known_divergences": stale_divergences,
        "sample_mappings": [
            {"cargo_target_root": target["cargo_target_root"], "buck2_mappings": buck_roots.get(target["cargo_target_root"], [])[:3]}
            for target in cargo_targets[:10]
        ],
        "verdict": "PASS" if not failures else "FAIL",
        "failures": sorted(set(failures)),
    }
    rendered = json.dumps(result, sort_keys=True)
    if args.json or result["verdict"] == "PASS":
        print(rendered)
    else:
        print(rendered, file=sys.stderr)
    return 0 if result["verdict"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
