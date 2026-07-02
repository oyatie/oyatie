#!/usr/bin/env python3
"""Validate the Fixture-2 sub-60s revocation-evidence artifact (rubric G4).

Sub-AC gate for the owned Policy IR benchmark phase: the revocation-evidence
artifact produced by benchmarks/policy-ir/revoke_then_check_harness.py must

  1. pin content-hashes that recompute — every measurement record references
     the FROZEN Fixture-2 content-hash (fixture_2_workload_contract subtree
     hash), the FixtureSuite document hash, and the frozen rubric hash;
  2. satisfy the sub-60s criterion — every measured time_to_consistent_deny_ms
     is strictly under the fixture's sub_60s_bound_ms with a coherent sub_60s
     flag and the consistent-deny discipline (>= 3 consecutive denies);
  3. respect the fixture scope bindings — measured engines are exactly
     {cedar, spicedb, openfga, opa_rego, biscuit}; cel is analysis-only with
     the rubric G4 bounded_scope_rule cited; the consistency-semantics
     analysis covers the whole Core-6 slate with citations;
  4. document the measurement topology (environment, node counts, cache
     layers, distribution path, parameters) alongside every number, carry the
     N3 emulation disclosure, and match the fixture's watched check and
     expected pre/post decisions (allow -> deny).

Stdlib only.
"""
from __future__ import annotations

import copy
import hashlib
import json
import sys
from pathlib import Path
from typing import Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
SUITE_PATH = REPO_ROOT / "specs" / "policy-ir-benchmark-fixture-suite.json"
RUBRIC_PATH = REPO_ROOT / "specs" / "policy-ir-benchmark-rubric.json"
EVIDENCE_PATH = (
    REPO_ROOT / "evidence" / "policy-ir-benchmark" / "fixture-2-revocation-evidence.json"
)

CORE6 = {"cedar", "spicedb", "openfga", "opa_rego", "cel", "biscuit"}
REQUIRED_TOPOLOGY_FIELDS = {
    "environment",
    "node_counts",
    "cache_layers",
    "distribution_path",
    "parameters",
}
REQUIRED_ANALYSIS_FIELDS = {
    "model_kind",
    "read_after_write",
    "staleness_model",
    "sub_60s_assessment",
    "citations",
}
MIN_CONSECUTIVE_DENIES = 3
TRANSITION_SURFACING_ENGINES = {"cedar", "spicedb", "opa_rego", "biscuit"}


def fail(message: str) -> NoReturn:
    print(f"FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def require(condition: object, message: str) -> None:
    if not condition:
        fail(message)


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        fail(f"cannot load {path}: {exc}")


def canonical_bytes(document: object) -> bytes:
    return json.dumps(
        document, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode("utf-8")


def fixture_2_hash(suite: dict) -> str:
    subtree = copy.deepcopy(suite["fixture_2_workload_contract"])
    subtree["content_hash"]["value"] = ""
    return "fixture2-v1:sha256:" + hashlib.sha256(canonical_bytes(subtree)).hexdigest()


def whole_doc_hash(document: dict, prefix: str) -> str:
    candidate = copy.deepcopy(document)
    candidate["content_hash"]["value"] = ""
    return f"{prefix}sha256:" + hashlib.sha256(canonical_bytes(candidate)).hexdigest()


def frozen_pins(suite: dict, rubric: dict) -> dict[str, str]:
    require(
        "fixture_2_workload_contract" in suite,
        "FixtureSuite carries no fixture_2_workload_contract — Fixture 2 not landed",
    )
    f2_pin = suite["fixture_2_workload_contract"]["content_hash"]["value"]
    require(
        f2_pin == fixture_2_hash(suite),
        "frozen Fixture-2 content-hash does not recompute from the FixtureSuite",
    )
    suite_pin = suite["content_hash"]["value"]
    require(
        suite_pin == whole_doc_hash(suite, "fixture-suite-v1:"),
        "FixtureSuite content-hash does not recompute",
    )
    rubric_pin = rubric["content_hash"]["value"]
    require(
        rubric_pin == whole_doc_hash(rubric, "rubric-v1:"),
        "rubric content-hash does not recompute",
    )
    return {
        "fixture_2_content_hash": f2_pin,
        "fixture_suite_content_hash": suite_pin,
        "rubric_content_hash": rubric_pin,
    }


def check_measurement(record: dict, pins: dict[str, str], scenario: dict) -> None:
    engine = record.get("engine")
    require(engine in CORE6, f"measurement engine {engine!r} outside the Core-6 slate")
    label = f"measurement[{engine}]"
    require(record.get("measured") is True, f"{label}: measured must be true")

    # (1) frozen content-hash pins
    require(
        record.get("fixture_2_content_hash") == pins["fixture_2_content_hash"],
        f"{label}: does not reference the frozen Fixture-2 content-hash "
        f"(got {record.get('fixture_2_content_hash')!r})",
    )
    require(
        record.get("fixture_suite_content_hash") == pins["fixture_suite_content_hash"],
        f"{label}: stale FixtureSuite content-hash pin",
    )
    require(
        record.get("rubric_content_hash") == pins["rubric_content_hash"],
        f"{label}: stale rubric content-hash pin",
    )

    # (2) sub-60s criterion + consistent-deny discipline
    bound = scenario["sub_60s_bound_ms"]
    require(
        record.get("sub_60s_bound_ms") == bound,
        f"{label}: sub_60s_bound_ms diverges from the fixture scenario bound",
    )
    latency = record.get("time_to_consistent_deny_ms")
    require(
        isinstance(latency, (int, float)) and not isinstance(latency, bool) and latency >= 0,
        f"{label}: time_to_consistent_deny_ms must be a non-negative number",
    )
    require(
        latency < bound,
        f"{label}: time_to_consistent_deny_ms {latency} violates the sub-60s bound {bound}",
    )
    require(
        record.get("sub_60s") is True and (latency < bound) == record["sub_60s"],
        f"{label}: sub_60s flag must be true and consistent with the measured value",
    )
    consecutive = record.get("consecutive_consistent_denies")
    require(
        isinstance(consecutive, int) and consecutive >= MIN_CONSECUTIVE_DENIES,
        f"{label}: needs >= {MIN_CONSECUTIVE_DENIES} consecutive consistent denies",
    )

    # (3) fixture-scenario coherence
    require(
        record.get("revoked_tuple_id") == scenario["revoked_tuple_id"],
        f"{label}: revoked_tuple_id diverges from the fixture revocation scenario",
    )
    watched = scenario["watched_check"]
    got_watch = record.get("watched_check", {})
    for field in ("request_id", "principal", "action", "resource"):
        require(
            got_watch.get(field) == watched[field],
            f"{label}: watched_check.{field} diverges from the fixture scenario",
        )
    require(
        record.get("pre_revocation_decision") == scenario["expected_pre_revocation_decision"],
        f"{label}: pre_revocation_decision must be "
        f"{scenario['expected_pre_revocation_decision']!r}",
    )
    require(
        record.get("post_revocation_decision") == scenario["expected_post_revocation_decision"],
        f"{label}: post_revocation_decision must be "
        f"{scenario['expected_post_revocation_decision']!r}",
    )

    # (4) topology documentation + disclosures
    topology = record.get("topology")
    require(isinstance(topology, dict), f"{label}: missing topology documentation")
    missing = REQUIRED_TOPOLOGY_FIELDS - set(topology)
    require(not missing, f"{label}: topology missing fields {sorted(missing)}")
    require(
        isinstance(topology["node_counts"], dict) and topology["node_counts"],
        f"{label}: topology.node_counts must be a non-empty object",
    )
    require(
        isinstance(topology["cache_layers"], list),
        f"{label}: topology.cache_layers must be an array",
    )
    require(
        isinstance(topology["parameters"], dict) and topology["parameters"],
        f"{label}: topology.parameters must document the reference-model parameters",
    )
    emulation = record.get("emulation")
    require(isinstance(emulation, dict), f"{label}: missing N3 emulation disclosure")
    if emulation.get("emulated") is True:
        require(
            isinstance(emulation.get("adapter_shape"), str) and emulation["adapter_shape"],
            f"{label}: emulated=true without a recorded adapter_shape is invalid",
        )
    else:
        require(
            emulation.get("emulated") is False,
            f"{label}: emulation.emulated must be a boolean disclosure",
        )
    # Consistency-transition surface: cedar bumps the bundle semver; spicedb
    # (ZedToken revision), opa_rego (bundle revision), and biscuit (revocation-id
    # set digest) change their tokens. OpenFGA documents synchronous tuple writes
    # with NO per-write token — the authorization-model id pins the schema, not
    # the tuple snapshot — so no transition is owed there.
    if engine in TRANSITION_SURFACING_ENGINES:
        version_changed = record.get("policy_version_before") != record.get(
            "policy_version_after"
        )
        token_changed = (
            "consistency_token_after" in record
            and record.get("consistency_token_before") != record.get("consistency_token_after")
        )
        require(
            version_changed or token_changed,
            f"{label}: revocation must surface a consistency transition "
            "(token change or policy version bump)",
        )


def validate(evidence: dict, suite: dict, rubric: dict) -> None:
    pins = frozen_pins(suite, rubric)
    require(
        evidence.get("artifact_kind") == "revocation-evidence",
        "artifact_kind must be revocation-evidence",
    )
    require(
        evidence["_meta"]["spec_id"] == "POL-IR-BENCH-F2-REVOCATION-EVIDENCE",
        "unexpected evidence spec_id",
    )
    for key, value in pins.items():
        require(
            evidence["pins"].get(key) == value,
            f"evidence pins.{key} does not match the frozen value",
        )

    scenario = suite["fixture_2_workload_contract"]["revocation_scenario"]
    require(
        evidence.get("sub_60s_bound_ms") == scenario["sub_60s_bound_ms"],
        "evidence sub_60s_bound_ms diverges from the fixture scenario",
    )

    scope = scenario["scope_bindings"]
    measurements = evidence.get("measurements")
    require(isinstance(measurements, list) and measurements, "measurements must be non-empty")
    measured_engines = [record.get("engine") for record in measurements]
    require(
        len(set(measured_engines)) == len(measured_engines),
        "duplicate engine measurement records",
    )
    require(
        sorted(measured_engines) == sorted(scope["measured_engines"]),
        f"measured engines {sorted(measured_engines)} must be exactly the fixture scope "
        f"binding {sorted(scope['measured_engines'])}",
    )
    for record in measurements:
        check_measurement(record, pins, scenario)

    analysis = evidence.get("consistency_semantics_analysis")
    require(isinstance(analysis, dict), "missing consistency_semantics_analysis")
    require(
        set(analysis) == CORE6,
        f"consistency_semantics_analysis must cover exactly the Core-6 slate; "
        f"got {sorted(analysis or {})}",
    )
    for engine, entry in analysis.items():
        missing = REQUIRED_ANALYSIS_FIELDS - set(entry)
        require(not missing, f"analysis[{engine}]: missing fields {sorted(missing)}")
        require(
            isinstance(entry["citations"], list)
            and entry["citations"]
            and all(isinstance(c, str) and c for c in entry["citations"]),
            f"analysis[{engine}]: citations must be a non-empty list of strings",
        )
        for field in REQUIRED_ANALYSIS_FIELDS - {"citations"}:
            require(
                isinstance(entry[field], str) and entry[field].strip(),
                f"analysis[{engine}]: {field} must be non-empty written analysis",
            )

    exclusions = evidence.get("exclusions")
    require(isinstance(exclusions, list), "exclusions must be an array")
    excluded_engines = {entry.get("engine") for entry in exclusions}
    require(
        excluded_engines == set(scope["analysis_only_engines"]),
        f"exclusions must cover exactly the analysis-only engines "
        f"{sorted(scope['analysis_only_engines'])}",
    )
    for entry in exclusions:
        require(
            entry.get("measured") is False,
            f"exclusion[{entry.get('engine')}]: measured must be false",
        )
        require(
            isinstance(entry.get("reason"), str) and entry["reason"].strip(),
            f"exclusion[{entry.get('engine')}]: needs a written reason",
        )
        require(
            "bounded_scope_rule" in entry.get("rubric_citation", ""),
            f"exclusion[{entry.get('engine')}]: must cite the rubric G4 bounded_scope_rule",
        )
    require(
        excluded_engines.isdisjoint(set(measured_engines)),
        "an engine cannot be both measured and analysis-only",
    )


def main() -> None:
    evidence = load_json(EVIDENCE_PATH)
    suite = load_json(SUITE_PATH)
    rubric = load_json(RUBRIC_PATH)
    validate(evidence, suite, rubric)
    latencies = {
        record["engine"]: record["time_to_consistent_deny_ms"]
        for record in evidence["measurements"]
    }
    print(
        "fixture-2 revocation evidence check passed: "
        f"{EVIDENCE_PATH.relative_to(REPO_ROOT)} "
        f"(measured={len(latencies)}, analysis_engines={len(evidence['consistency_semantics_analysis'])}, "
        f"max_time_to_consistent_deny_ms={max(latencies.values()):.1f}, "
        f"bound_ms={evidence['sub_60s_bound_ms']}, "
        f"fixture_2_content_hash={evidence['pins']['fixture_2_content_hash'][:32]}...)"
    )


def run_self_tests() -> None:
    evidence = load_json(EVIDENCE_PATH)
    suite = load_json(SUITE_PATH)
    rubric = load_json(RUBRIC_PATH)

    def expect_rejected(label: str, mutator: Callable[[dict], None]) -> None:
        candidate = copy.deepcopy(evidence)
        mutator(candidate)
        try:
            validate(candidate, suite, rubric)
        except SystemExit as exc:
            require(exc.code != 0, f"self-test {label!r} exited successfully")
        else:
            fail(f"self-test mutation was accepted: {label}")

    def stale_fixture_2_pin(data: dict) -> None:
        stale = "fixture2-v1:sha256:" + "0" * 64
        data["measurements"][0]["fixture_2_content_hash"] = stale

    def over_bound(data: dict) -> None:
        data["measurements"][0]["time_to_consistent_deny_ms"] = 60000.0

    def lying_flag(data: dict) -> None:
        data["measurements"][0]["sub_60s"] = False

    def drop_topology_field(data: dict) -> None:
        data["measurements"][0]["topology"].pop("distribution_path")

    def drop_engine(data: dict) -> None:
        data["measurements"] = [
            record for record in data["measurements"] if record["engine"] != "biscuit"
        ]

    def measure_cel(data: dict) -> None:
        clone = copy.deepcopy(data["measurements"][0])
        clone["engine"] = "cel"
        data["measurements"].append(clone)

    def emulated_without_shape(data: dict) -> None:
        data["measurements"][0]["emulation"] = {"emulated": True}

    def pre_revocation_deny(data: dict) -> None:
        data["measurements"][0]["pre_revocation_decision"] = "deny"

    def too_few_denies(data: dict) -> None:
        data["measurements"][0]["consecutive_consistent_denies"] = 1

    def drop_analysis_engine(data: dict) -> None:
        data["consistency_semantics_analysis"].pop("cel")

    def uncited_analysis(data: dict) -> None:
        data["consistency_semantics_analysis"]["spicedb"]["citations"] = []

    def drop_exclusion_citation(data: dict) -> None:
        data["exclusions"][0]["rubric_citation"] = "somewhere-else"

    def no_consistency_transition(data: dict) -> None:
        record = next(r for r in data["measurements"] if r["engine"] == "spicedb")
        record["consistency_token_after"] = record["consistency_token_before"]
        record["policy_version_after"] = record["policy_version_before"]

    expect_rejected("stale fixture-2 content-hash pin", stale_fixture_2_pin)
    expect_rejected("measurement at the 60s bound", over_bound)
    expect_rejected("sub_60s flag inconsistent with value", lying_flag)
    expect_rejected("topology missing distribution_path", drop_topology_field)
    expect_rejected("missing measured engine (biscuit)", drop_engine)
    expect_rejected("cel measured despite analysis-only scope", measure_cel)
    expect_rejected("emulated disclosure without adapter shape", emulated_without_shape)
    expect_rejected("pre-revocation deny", pre_revocation_deny)
    expect_rejected("fewer than 3 consecutive denies", too_few_denies)
    expect_rejected("analysis missing a Core-6 engine", drop_analysis_engine)
    expect_rejected("analysis without citations", uncited_analysis)
    expect_rejected("exclusion without bounded_scope_rule citation", drop_exclusion_citation)
    expect_rejected("revocation without consistency transition", no_consistency_transition)
    print("fixture-2 revocation evidence self-tests passed")


if __name__ == "__main__":
    if "--self-test" in sys.argv[1:]:
        run_self_tests()
    main()
