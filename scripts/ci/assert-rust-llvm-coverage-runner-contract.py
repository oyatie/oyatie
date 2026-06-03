#!/usr/bin/env python3
"""Validate the Buck2-native LLVM Rust coverage runner contract.

This checker is local/static contract evidence only. It validates the target
shape for future Buck2-native LLVM source-based coverage runners. It does not
run tests, generate coverage reports, invoke llvm-profdata/llvm-cov, post
statuses, mutate branch protection, or prove P0.0 / Phase-0 exit authority.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

DEFAULT_SPEC = Path("specs/rust-llvm-coverage-runner-contract.json")
FALSE_CLAIMS = (
    "coverage_report_generated",
    "coverage_budget_enforced",
    "status_mutation_performed",
    "protected_branch_authority_proven",
    "live_required_context_execution_proven",
    "p0_0_green",
    "phase0_complete",
    "production_ready",
    "hyperscaler_grade",
)
REQUIRED_OFFICIAL_URLS = (
    "https://doc.rust-lang.org/rustc/instrument-coverage.html",
    "https://clang.llvm.org/docs/SourceBasedCodeCoverage.html",
    "https://buck2.build/docs/users/commands/",
)
REQUIRED_EVIDENCE_FIELDS = (
    "Buck2 target",
    "Buck2 Build ID",
    "report path",
    "changed-file delta",
    "excluded generated paths",
    "coverage budget result",
)
REQUIRED_TOOLCHAIN_TOOLS = ("rustc", "llvm-profdata", "llvm-cov")
RAW_CARGO_RE = re.compile(r"(^|[;&|(`]|\s)cargo\s+([a-z0-9_-]+)(\s|$)", re.IGNORECASE)


def load_json(path: Path) -> dict[str, Any]:
    with path.open() as fh:
        data = json.load(fh)
    if not isinstance(data, dict):
        raise TypeError(f"{path}: expected JSON object")
    return data


def strings(value: Any) -> list[str]:
    if isinstance(value, list):
        return [item for item in value if isinstance(item, str)]
    return []


def official_urls(spec: dict[str, Any]) -> set[str]:
    refs = spec.get("_meta", {}).get("official_references", [])
    urls: set[str] = set()
    if isinstance(refs, list):
        for ref in refs:
            if isinstance(ref, dict) and isinstance(ref.get("url"), str):
                urls.add(ref["url"])
    return urls


def validate(spec: dict[str, Any]) -> dict[str, Any]:
    failures: list[str] = []
    boundary = spec.get("claim_boundary") if isinstance(spec.get("claim_boundary"), dict) else {}
    pipeline = spec.get("coverage_pipeline") if isinstance(spec.get("coverage_pipeline"), dict) else {}
    instrumentation = pipeline.get("instrumentation") if isinstance(pipeline.get("instrumentation"), dict) else {}
    merge = pipeline.get("profile_merge") if isinstance(pipeline.get("profile_merge"), dict) else {}
    export = pipeline.get("report_export") if isinstance(pipeline.get("report_export"), dict) else {}
    toolchain = (
        pipeline.get("toolchain_requirements")
        if isinstance(pipeline.get("toolchain_requirements"), dict)
        else {}
    )
    smoke = pipeline.get("smoke_coverage") if isinstance(pipeline.get("smoke_coverage"), dict) else {}

    if boundary.get("coverage_runner_contract_proven") is not True:
        failures.append("coverage_runner_contract_not_proven")
    for claim in FALSE_CLAIMS:
        if boundary.get(claim) is not False:
            failures.append(f"forbidden_true_or_missing_claim_{claim}")

    if pipeline.get("canonical_surface") != "Buck2-native LLVM source-based coverage":
        failures.append("missing_buck2_native_llvm_canonical_surface")
    noncanonical = pipeline.get("noncanonical_surface")
    if not isinstance(noncanonical, str) or "Tarpaulin" not in noncanonical or "not required CI/PR" not in noncanonical:
        failures.append("tarpaulin_noncanonical_boundary_missing")
    if pipeline.get("buck2_contract_target") != "//:rust-llvm-coverage-runner-contract-check":
        failures.append("wrong_buck2_contract_target")
    if pipeline.get("future_runner_authority") != "trusted cloud-ci/oya-ci Buck2 target inventory":
        failures.append("wrong_future_runner_authority")

    if toolchain.get("ambient_path_llvm_tools_required") is not False:
        failures.append("ambient_path_llvm_tools_not_forbidden")
    if "rustup llvm-tools component" not in str(toolchain.get("local_smoke_llvm_tools_source", "")):
        failures.append("missing_rustup_llvm_tools_source")
    if "trusted cloud-ci/oya-ci Buck2 toolchain inventory" not in str(
        toolchain.get("live_runner_llvm_tools_source", "")
    ):
        failures.append("missing_live_runner_toolchain_inventory_source")
    if "rustc --print sysroot" not in str(toolchain.get("host_tool_path_derivation", "")):
        failures.append("missing_sysroot_tool_path_derivation")
    if "pin" not in str(toolchain.get("pinning_requirement", "")).lower():
        failures.append("missing_toolchain_pinning_requirement")
    required_tools = set(strings(toolchain.get("required_tools")))
    for tool in REQUIRED_TOOLCHAIN_TOOLS:
        if tool not in required_tools:
            failures.append(f"missing_required_toolchain_tool_{tool.replace('-', '_')}")

    if smoke.get("buck2_smoke_target") != "//:rust-llvm-coverage-smoke-check":
        failures.append("missing_buck2_coverage_smoke_target")
    if smoke.get("smoke_script") != "scripts/ci/run-rust-llvm-coverage-smoke.py":
        failures.append("missing_coverage_smoke_script")
    if smoke.get("fixture") != "specs/fixtures/rust-llvm-coverage-smoke/branchy.rs":
        failures.append("missing_coverage_smoke_fixture")
    if smoke.get("fixture_report_generated") is not True:
        failures.append("fixture_report_generation_not_recorded")
    if smoke.get("production_coverage_report_generated") is not False:
        failures.append("production_coverage_false_boundary_missing")
    if "none" not in str(smoke.get("budget_authority", "")).lower():
        failures.append("smoke_budget_authority_not_none")

    evidence_fields = strings(pipeline.get("required_evidence_fields"))
    for field in REQUIRED_EVIDENCE_FIELDS:
        if field not in evidence_fields:
            failures.append(f"missing_required_evidence_field_{field.replace(' ', '_').replace('-', '_').lower()}")

    if instrumentation.get("rustc_flag") != "rustc -C instrument-coverage":
        failures.append("missing_instrument_coverage_flag")
    if instrumentation.get("buck2_rust_rule_field") != "rustc_flags":
        failures.append("missing_buck2_rustc_flags_field")
    if instrumentation.get("profile_env_var") != "LLVM_PROFILE_FILE":
        failures.append("missing_llvm_profile_file_env")
    template = instrumentation.get("profile_template")
    if not isinstance(template, str) or "%m-%p" not in template or not template.endswith(".profraw"):
        failures.append("missing_profile_collision_guard_or_profraw_template")
    if instrumentation.get("profile_collision_guard") != "%m-%p":
        failures.append("missing_profile_collision_guard")
    if instrumentation.get("profile_extension") != ".profraw":
        failures.append("missing_profraw_extension")

    if merge.get("tool") != "llvm-profdata":
        failures.append("missing_llvm_profdata_tool")
    if merge.get("operation") != "merge":
        failures.append("missing_profdata_merge_operation")
    if merge.get("mode") != "-sparse":
        failures.append("missing_sparse_profdata_merge_mode")
    if "*.profraw" not in str(merge.get("input_glob")):
        failures.append("missing_profraw_merge_input_glob")
    if not str(merge.get("output", "")).endswith(".profdata"):
        failures.append("missing_profdata_output")

    if export.get("tool") != "llvm-cov":
        failures.append("missing_llvm_cov_tool")
    formats = set(strings(export.get("formats")))
    for fmt in ("text", "html", "json"):
        if fmt not in formats:
            failures.append(f"missing_llvm_cov_{fmt}_format")
    budget = export.get("delta_budget") if isinstance(export.get("delta_budget"), dict) else {}
    if budget.get("changed_files_line_coverage_minimum") != 80:
        failures.append("wrong_changed_file_delta_budget")
    if budget.get("kernel_domain_absolute_line_coverage_minimum") != 70:
        failures.append("wrong_kernel_domain_budget")
    if budget.get("generated_code_excluded") is not True:
        failures.append("generated_code_exclusion_missing")

    forbidden = "\n".join(strings(pipeline.get("forbidden_authority")))
    if "Tarpaulin as canonical monorepo coverage evidence" not in forbidden:
        failures.append("missing_tarpaulin_forbidden_authority")
    if "candidate-authored target inventory as coverage authority" not in forbidden:
        failures.append("missing_candidate_inventory_forbidden_authority")

    urls = official_urls(spec)
    for url in REQUIRED_OFFICIAL_URLS:
        if url not in urls:
            failures.append(f"missing_official_reference_{url.rsplit('/', 2)[-2] if url.endswith('/') else url.rsplit('/', 1)[-1]}")

    rendered = json.dumps(spec, sort_keys=True)
    if RAW_CARGO_RE.search(rendered):
        failures.append("raw_cargo_command_present_in_contract")

    automated_chain = "\n".join(strings(spec.get("automated_chain")))
    if "buck2 build //:rust-llvm-coverage-smoke-check" not in automated_chain:
        failures.append("missing_smoke_target_in_automated_chain")

    return {
        "authority_boundary": "local/static coverage runner contract only; no coverage report generated and no live required-context authority proven",
        "coverage_runner_contract_proven": not failures,
        "coverage_report_generated": False,
        "coverage_budget_enforced": False,
        "status_mutation_performed": False,
        "protected_branch_authority_proven": False,
        "live_required_context_execution_proven": False,
        "p0_0_green": False,
        "phase0_complete": False,
        "production_ready": False,
        "hyperscaler_grade": False,
        "verdict": "PASS" if not failures else "FAIL",
        "failures": sorted(set(failures)),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--spec", default=str(DEFAULT_SPEC), help="coverage runner contract spec path")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    result = validate(load_json(Path(args.spec)))
    rendered = json.dumps(result, sort_keys=True)
    if args.json or result["verdict"] == "PASS":
        print(rendered)
    else:
        print(rendered, file=sys.stderr)
    return 0 if result["verdict"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
