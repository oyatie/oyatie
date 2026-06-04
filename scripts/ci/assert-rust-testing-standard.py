#!/usr/bin/env python3
"""Validate the Rust testing standard's Buck2 coverage/mutation authority text.

This checker is local/static documentation-drift evidence only. It proves the
recorded Rust testing standard still preserves Buck2-native LLVM source-based
coverage, Tarpaulin non-authority, and dual Cargo+Buck2 local mutation
boundaries. It never implements a coverage runner, runs mutation testing, posts
statuses, mutates branch protection, or claims P0.0 green / Phase-0 completion /
production readiness.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path

DEFAULT_DOC = Path("docs/standards/testing.md")
LIVE_FALSE_FLAGS = {
    "coverage_runner_implemented": False,
    "mutation_lane_implemented": False,
    "live_required_context_execution_proven": False,
    "protected_branch_authority_proven": False,
    "status_mutation_performed": False,
    "p0_0_green": False,
    "phase0_complete": False,
    "production_ready": False,
    "hyperscaler_grade": False,
}
CLAIM_TRUE_RE = re.compile(
    r"\b(coverage_runner_implemented|mutation_lane_implemented|"
    r"live_required_context_execution_proven|protected_branch_authority_proven|"
    r"status_mutation_performed|p0_0_green|phase0_complete|production_ready|"
    r"hyperscaler_grade)\b\s*[:=]\s*true\b",
    re.IGNORECASE,
)


@dataclass(frozen=True)
class Anchor:
    id: str
    tokens: tuple[str, ...]


REQUIRED_ANCHORS = (
    Anchor("buck2_native_llvm_coverage_policy", ("Buck2-native LLVM source-based coverage",)),
    Anchor("coverage_generated_through_buck2", ("Coverage is generated natively through Buck2, not Tarpaulin",)),
    Anchor("tarpaulin_non_canonical", ("Tarpaulin is not the canonical coverage surface",)),
    Anchor("tarpaulin_not_required_ci", ("MUST NOT be added as required CI/PR evidence",)),
    Anchor("rustc_instrument_coverage", ("rustc -C instrument-coverage", "-C instrument-coverage")),
    Anchor("llvm_profile_file", ("LLVM_PROFILE_FILE",)),
    Anchor("profraw_profiles", (".profraw",)),
    Anchor("llvm_profdata", ("llvm-profdata",)),
    Anchor("llvm_cov", ("llvm-cov",)),
    Anchor("buck2_build_id_evidence", ("Buck2 target", "Build ID", "report path")),
    Anchor("delta_and_generated_exclusions", ("changed-file delta", "excluded generated paths")),
    Anchor("dual_cargo_buck2_harness", ("dual Cargo+Buck2",)),
    Anchor("cargo_manifests_retained", ("Cargo.toml", "Cargo.lock")),
    Anchor("cargo_mutants_local", ("cargo mutants",)),
    Anchor("cargo_nextest_local", ("cargo nextest",)),
    Anchor("local_cargo_mutation_advisory", ("Local Cargo mutation output is advisory",)),
    Anchor("buck2_or_cloud_ci_mutation_capture", ("Buck2 target or trusted cloud-ci/oya-ci lane", "captured the mutation run")),
    Anchor("buck2_authority", ("Buck2 `BUCK` targets remain the build/test/CI authority",)),
    Anchor("reindeer_generated_buck", ("reindeer-style generation", "generated-BUCK path")),
    Anchor("raw_cargo_not_authority", ("raw Cargo commands are not CI/build/test authority",)),
    Anchor("buck2_show_output", ("buck2 test //... --show-output",)),
    Anchor("trusted_cloud_ci_inventory", ("trusted cloud-ci/oya-ci Buck2 target inventory",)),
    Anchor("anti_pattern_tarpaulin_authority", ("Adding Tarpaulin as the monorepo coverage authority",)),
    Anchor("anti_pattern_local_cargo_merge_authority", ("Treating local Cargo mutation testing as merge authority",)),
    Anchor("rustc_source", ("https://doc.rust-lang.org/rustc/instrument-coverage.html",)),
    Anchor("llvm_source", ("https://clang.llvm.org/docs/SourceBasedCodeCoverage.html",)),
    Anchor("buck2_commands_source", ("https://buck2.build/docs/users/commands/",)),
    Anchor("buck2_bootstrapping_source", ("https://buck2.build/docs/about/bootstrapping/",)),
    Anchor("cargo_workspace_source", ("https://doc.rust-lang.org/cargo/reference/workspaces.html",)),
    Anchor("reindeer_source", ("https://github.com/facebookincubator/reindeer",)),
    Anchor("cargo_mutants_source", ("https://mutants.rs/",)),
)


def contains_all(text: str, tokens: tuple[str, ...]) -> bool:
    return all(token in text for token in tokens)


def anchor_results(text: str) -> list[dict[str, object]]:
    results: list[dict[str, object]] = []
    for anchor in REQUIRED_ANCHORS:
        present = contains_all(text, anchor.tokens)
        results.append({"id": anchor.id, "present": present, "tokens": list(anchor.tokens)})
    return results


def missing_anchor_failures(results: list[dict[str, object]]) -> list[str]:
    return [f"missing_{result['id']}" for result in results if not result.get("present")]


def tarpaulin_boundary_failures(text: str) -> list[str]:
    failures: list[str] = []
    if "Tarpaulin" not in text:
        failures.append("missing_tarpaulin_boundary_subject")
        return failures
    if "Tarpaulin is not the canonical coverage surface" not in text:
        failures.append("tarpaulin_canonicalized")
    if "MUST NOT be added as required CI/PR evidence" not in text:
        failures.append("tarpaulin_required_ci_boundary_missing")
    for line_no, line in enumerate(text.splitlines(), start=1):
        if re.search(r"Tarpaulin\s+is\s+(?:the\s+)?canonical\b", line):
            failures.append(f"tarpaulin_canonical_claim_line_{line_no}")
        if re.search(r"Tarpaulin\s+.*required\s+CI/PR\s+evidence", line) and "MUST NOT" not in line:
            failures.append(f"tarpaulin_required_ci_claim_line_{line_no}")
    return failures


def local_cargo_mutation_boundary_failures(text: str) -> list[str]:
    failures: list[str] = []
    if "Local Cargo mutation output is advisory" not in text:
        failures.append("local_cargo_mutation_not_advisory")
    if "captured by a Buck2 target or trusted cloud-ci/oya-ci lane" not in text:
        failures.append("local_cargo_mutation_capture_boundary_missing")
    if "Treating local Cargo mutation testing as merge authority" not in text:
        failures.append("local_cargo_mutation_merge_authority_antipattern_missing")
    return failures


def claim_boundary_failures(text: str) -> list[str]:
    failures = [f"forbidden_true_claim_{match.group(1).lower()}" for match in CLAIM_TRUE_RE.finditer(text)]
    forbidden_phrases = {
        "p0_0_green_phrase": ("P0.0", " is green"),
        "phase0_complete_phrase": ("Phase-0", " is complete"),
        "production_ready_phrase": ("production-ready", " now"),
        "hyperscaler_grade_phrase": ("hyperscaler-grade", " now"),
    }
    for label, parts in forbidden_phrases.items():
        if "".join(parts) in text:
            failures.append(f"forbidden_claim_{label}")
    return failures


def validate_doc(path: Path) -> dict[str, object]:
    text = path.read_text()
    anchors = anchor_results(text)
    failures: list[str] = []
    failures.extend(missing_anchor_failures(anchors))
    failures.extend(tarpaulin_boundary_failures(text))
    failures.extend(local_cargo_mutation_boundary_failures(text))
    failures.extend(claim_boundary_failures(text))
    failures = sorted(set(failures))
    return {
        "doc": str(path),
        "authority_boundary": (
            "local/static standards drift evidence only; this checker does not run coverage, "
            "does not run mutation testing, never posts statuses, never mutates branch protection, "
            "and cannot prove live Phase-0 exit authority"
        ),
        "anchor_results": anchors,
        "anchor_count": len(anchors),
        "anchors_present": sum(1 for anchor in anchors if anchor.get("present")),
        "standard_contract_proven": not failures,
        **LIVE_FALSE_FLAGS,
        "verdict": "PASS" if not failures else "FAIL",
        "failures": failures,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--doc", default=str(DEFAULT_DOC), help="testing standard document to validate")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    path = Path(args.doc)
    output = validate_doc(path)
    rendered = json.dumps(output, sort_keys=True)
    if args.json or output["verdict"] == "PASS":
        print(rendered)
    else:
        print(rendered, file=sys.stderr)
    return 0 if output["verdict"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
