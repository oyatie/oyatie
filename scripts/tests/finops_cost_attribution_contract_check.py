#!/usr/bin/env python3
"""Validate FINOPS-001 cost-attribution contract coverage."""
from __future__ import annotations

import copy
import contextlib
import io
import json
import sys
from pathlib import Path
from typing import Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
SPEC_PATH = REPO_ROOT / "specs" / "finops-cost-attribution.json"

REQUIRED_TOP_LEVEL = {
    "tag_block",
    "canonical_label_block",
    "focus_opencost_rollup",
    "chargeback_formula",
    "anomaly_thresholds",
    "anomaly_detector_fixtures",
    "karpenter_workload_class_cost_labels",
    "regulator_evidence_cadence",
    "claim_controls",
    "nonclaims",
}
REQUIRED_ACCEPTED_ADRS = {"ADR-0174", "ADR-0198", "ADR-0199"}
REQUIRED_PROPOSED_CONTEXT_ADRS = {"ADR-0314", "ADR-0315"}
REQUIRED_K8S_LABELS = {
    "oya.io/tenant-id",
    "oya.io/cost-center",
    "oya.io/workload-class",
    "oya.io/regulatory-pack",
}
REQUIRED_CLOUD_TAGS = {
    "oya:tenant-id",
    "oya:cost-center",
    "oya:workload-class",
    "oya:regulatory-pack",
}
REQUIRED_FOCUS_COLUMNS = {
    "ChargePeriodStart",
    "ChargePeriodEnd",
    "ProviderName",
    "ServiceName",
    "SubAccountId",
    "ResourceId",
    "ConsumedQuantity",
    "ConsumedUnit",
    "EffectiveCost",
    "BillingCurrency",
    "Tags.oya.io/tenant-id",
    "Tags.oya.io/cost-center",
    "Tags.oya.io/workload-class",
    "Tags.oya.io/regulatory-pack",
}
REQUIRED_INPUT_STREAMS = {
    "opencost-kubernetes-allocation",
    "cloud-provider-billing-export",
    "on-prem-rate-card",
}
REQUIRED_ANOMALY_CLASSES = {
    "cost-spike",
    "cost-creep",
    "tenant-budget-headroom",
    "tenant-budget-exhausted",
    "provider-cost-deviation",
}
REQUIRED_NODEPOOL_MAP = {
    "app-tier": "app",
    "batch-tier": "batch",
    "gpu-tier": "gpu",
    "regulatory-tier": "regulatory",
}
REQUIRED_CONSUMER_CONTRACTS = {
    "oya/finops-portal/contracts/focus-export-internal.asyncapi.yaml",
    "oya/finops-portal/contracts/tenant-invoice-public.openapi.yaml",
    "oya/finops-portal/contracts/cost-allocation-policy-internal.proto",
}
REQUIRED_NONCLAIM_SNIPPETS = {
    "no live FOCUS export data generator",
    "no OpenCost exporter deployment or Mimir federation runtime",
    "no Karpenter Helm chart, NodePool, or cloud resource mutation",
    "no ERP or marketplace settlement authority is claimed from Proposed ADR-0314/ADR-0315",
}


def fail(message: str) -> NoReturn:
    print(f"finops cost-attribution contract check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require(condition: object, message: str) -> None:
    if not condition:
        fail(message)


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing {path.relative_to(REPO_ROOT)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON in {path.relative_to(REPO_ROOT)}: {exc}")


def properties(spec: dict, name: str) -> dict:
    props = spec.get("properties", {})
    require(name in props, f"missing properties.{name}")
    value = props[name]
    require(isinstance(value, dict), f"properties.{name} must be an object")
    return value


def nested_props(spec: dict, name: str) -> dict:
    prop = properties(spec, name)
    props = prop.get("properties", {})
    require(isinstance(props, dict), f"properties.{name}.properties must be an object")
    return props


def const_value(obj: dict, path: str) -> object:
    require("const" in obj, f"{path} must declare const")
    return obj["const"]


def default_array(obj: dict, path: str) -> list:
    value = obj.get("default")
    require(isinstance(value, list), f"{path} must declare a default array")
    return value


def validate(spec: dict) -> None:
    require(spec.get("_meta", {}).get("spec_id") == "EXE-FINOPS-COST-ATTRIBUTION", "unexpected spec_id")
    require(spec.get("_meta", {}).get("status") == "Accepted", "spec status must remain Accepted")
    require(REQUIRED_ACCEPTED_ADRS <= set(spec.get("_meta", {}).get("authority_boundary", {}).get("accepted_authority", [])), "accepted ADR authority is incomplete")
    require(REQUIRED_PROPOSED_CONTEXT_ADRS <= set(spec.get("_meta", {}).get("authority_boundary", {}).get("proposed_context_only", [])), "Proposed ADR context boundary is incomplete")
    require("cost-model-contract-only" in spec.get("_meta", {}).get("authority_boundary", {}).get("scope", ""), "authority boundary must stay contract-only")

    required = set(spec.get("required", []))
    require(REQUIRED_TOP_LEVEL <= required, f"top-level required missing {sorted(REQUIRED_TOP_LEVEL - required)}")

    label_props = nested_props(spec, "canonical_label_block")
    label_rows = default_array(label_props["kubernetes_labels"], "canonical_label_block.kubernetes_labels")
    labels_by_key = {row.get("key"): row for row in label_rows}
    require(set(labels_by_key) == REQUIRED_K8S_LABELS, "canonical Kubernetes label default set must match ADR-0199")
    for key, row in labels_by_key.items():
        require(row.get("required") is True, f"{key}: label must be required")
    require(labels_by_key["oya.io/tenant-id"].get("data_class") == "non_pii_ulid", "tenant label must remain non-PII ULID")
    for key in REQUIRED_K8S_LABELS - {"oya.io/tenant-id"}:
        require(labels_by_key[key].get("data_class") == "closed_enum", f"{key}: label must be closed enum")
    cloud_tags = set(default_array(label_props["cloud_resource_tags"], "canonical_label_block.cloud_resource_tags"))
    require(cloud_tags == REQUIRED_CLOUD_TAGS, "canonical cloud tag default set must match ADR-0199")
    aliases = label_props["legacy_adr_0174_aliases"].get("properties", {})
    require(const_value(aliases.get("tenant_id", {}), "legacy_adr_0174_aliases.tenant_id") == "oya.io/tenant-id", "legacy tenant_id alias must map to canonical label")
    require(const_value(aliases.get("cost_center", {}), "legacy_adr_0174_aliases.cost_center") == "oya.io/cost-center", "legacy cost_center alias must map to canonical label")

    rollup_props = nested_props(spec, "focus_opencost_rollup")
    require(const_value(rollup_props["focus_version"], "focus_opencost_rollup.focus_version") == "1.3", "FOCUS version must be 1.3")
    require(const_value(rollup_props["opencost_version"], "focus_opencost_rollup.opencost_version") == "1.110.0", "OpenCost version must be pinned to 1.110.0")
    require(set(default_array(rollup_props["aggregation_keys"], "focus_opencost_rollup.aggregation_keys")) == REQUIRED_K8S_LABELS, "rollup aggregation keys must be the canonical Kubernetes labels")
    input_streams = {row.get("stream_id"): row for row in default_array(rollup_props["input_streams"], "focus_opencost_rollup.input_streams")}
    require(set(input_streams) == REQUIRED_INPUT_STREAMS, "rollup input streams must cover OpenCost, provider bills, and on-prem rate cards")
    for stream_id, row in input_streams.items():
        require(row.get("claim_status") == "contract_only", f"{stream_id}: input stream must be contract_only")
    require(REQUIRED_FOCUS_COLUMNS <= set(default_array(rollup_props["focus_columns"], "focus_opencost_rollup.focus_columns")), "FOCUS columns missing required cost/allocation dimensions")
    consumer_contracts = set(default_array(rollup_props["consumer_contracts"], "focus_opencost_rollup.consumer_contracts"))
    require(consumer_contracts == REQUIRED_CONSUMER_CONTRACTS, "consumer contract references changed unexpectedly")
    for rel in consumer_contracts:
        require((REPO_ROOT / rel).exists(), f"consumer contract does not exist: {rel}")
    output_props = rollup_props["output_dataset"].get("properties", {})
    require(const_value(output_props.get("bucket_template", {}), "focus_opencost_rollup.output_dataset.bucket_template") == "oya-finops-focus-export-shared-<env>", "FOCUS export bucket template mismatch")

    formula_props = nested_props(spec, "chargeback_formula")
    formula_required = set(properties(spec, "chargeback_formula").get("required", []))
    for field in ["formula_text", "source_rollup_ref", "currency", "rounding"]:
        require(field in formula_required, f"chargeback_formula must require {field}")
    formula_text = const_value(formula_props["formula_text"], "chargeback_formula.formula_text")
    for token in ["labelled_spend", "tenant_allocation_ratio", "capability_invocations", "audit_chain_rows", "storage_bytes", "applicable_credits"]:
        require(token in formula_text, f"chargeback formula missing {token}")
    require(const_value(formula_props["source_rollup_ref"], "chargeback_formula.source_rollup_ref") == "focus_opencost_rollup", "chargeback formula must source from FOCUS/OpenCost rollup")

    threshold_required = set(properties(spec, "anomaly_thresholds").get("required", []))
    require({name.replace("-", "_") for name in REQUIRED_ANOMALY_CLASSES} <= threshold_required, "anomaly threshold contract missing required classes")
    fixtures = default_array(properties(spec, "anomaly_detector_fixtures"), "anomaly_detector_fixtures")
    fixture_classes = {row.get("detector_class") for row in fixtures}
    require(fixture_classes == REQUIRED_ANOMALY_CLASSES, "anomaly fixtures must cover every detector class exactly once")
    for row in fixtures:
        require(row.get("expected_alert") is True, f"{row.get('fixture_id')}: expected_alert must be true")
        require(row.get("expected_severity") in {"SEV-2", "SEV-3"}, f"{row.get('fixture_id')}: unexpected severity")

    karpenter_rows = default_array(properties(spec, "karpenter_workload_class_cost_labels"), "karpenter_workload_class_cost_labels")
    nodepool_map = {row.get("nodepool"): row.get("workload_class_label") for row in karpenter_rows}
    require(nodepool_map == REQUIRED_NODEPOOL_MAP, "Karpenter NodePool to workload-class label mapping mismatch")
    for row in karpenter_rows:
        require(row.get("cost_center_label") == "cc-cloud-substrate", f"{row.get('nodepool')}: cost center must be cc-cloud-substrate")
        require(row.get("opencost_dimension") == "label:oya.io/workload-class", f"{row.get('nodepool')}: OpenCost dimension mismatch")

    claim_props = nested_props(spec, "claim_controls")
    for field in [
        "contract_only",
        "no_runtime_focus_export",
        "no_opencost_mimir_federation_runtime",
        "no_karpenter_nodepool_runtime_mutation",
        "no_product_cloud_mutation",
    ]:
        require(const_value(claim_props[field], f"claim_controls.{field}") is True, f"claim_controls.{field} must be true")
    require(set(default_array(claim_props["proposed_adr_context_only"], "claim_controls.proposed_adr_context_only")) == REQUIRED_PROPOSED_CONTEXT_ADRS, "claim controls must keep ADR-0314/ADR-0315 context-only")
    nonclaims = set(default_array(properties(spec, "nonclaims"), "nonclaims"))
    require(REQUIRED_NONCLAIM_SNIPPETS <= nonclaims, "nonclaims missing contract-only/runtime boundaries")


def run_self_tests() -> None:
    baseline = load_json(SPEC_PATH)

    def expect_rejected(label: str, mutator: Callable[[dict], None]) -> None:
        candidate = copy.deepcopy(baseline)
        mutator(candidate)
        try:
            with contextlib.redirect_stderr(io.StringIO()):
                validate(candidate)
        except SystemExit as exc:
            require(exc.code != 0, f"self-test {label!r} exited successfully")
        else:
            fail(f"self-test mutation was accepted: {label}")

    expect_rejected(
        "missing tenant label",
        lambda data: data["properties"]["canonical_label_block"]["properties"]["kubernetes_labels"].update(
            {"default": data["properties"]["canonical_label_block"]["properties"]["kubernetes_labels"]["default"][1:]}
        ),
    )
    expect_rejected(
        "wrong FOCUS version",
        lambda data: data["properties"]["focus_opencost_rollup"]["properties"]["focus_version"].update({"const": "1.2"}),
    )
    expect_rejected(
        "missing OpenCost input stream",
        lambda data: data["properties"]["focus_opencost_rollup"]["properties"]["input_streams"].update(
            {"default": data["properties"]["focus_opencost_rollup"]["properties"]["input_streams"]["default"][1:]}
        ),
    )
    expect_rejected(
        "missing anomaly fixture",
        lambda data: data["properties"]["anomaly_detector_fixtures"].update(
            {"default": data["properties"]["anomaly_detector_fixtures"]["default"][1:]}
        ),
    )
    expect_rejected(
        "wrong Karpenter mapping",
        lambda data: data["properties"]["karpenter_workload_class_cost_labels"]["default"][0].update({"workload_class_label": "batch"}),
    )
    expect_rejected(
        "runtime claim control disabled",
        lambda data: data["properties"]["claim_controls"]["properties"]["no_product_cloud_mutation"].update({"const": False}),
    )
    expect_rejected(
        "Proposed ADR context missing",
        lambda data: data["properties"]["claim_controls"]["properties"]["proposed_adr_context_only"].update({"default": ["ADR-0314"]}),
    )
    print("finops cost-attribution contract self-tests passed")


def main() -> None:
    if "--self-test" in sys.argv[1:]:
        run_self_tests()
    validate(load_json(SPEC_PATH))
    print(f"finops cost-attribution contract check passed: {SPEC_PATH.relative_to(REPO_ROOT)}")


if __name__ == "__main__":
    main()
