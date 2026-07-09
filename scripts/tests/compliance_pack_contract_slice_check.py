#!/usr/bin/env python3
"""Validate the COMPLIANCE-001 compliance-pack certification/evidence/CMP contract slice."""
from __future__ import annotations

import copy
import json
import re
import sys
from pathlib import Path
from typing import Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
SCHEMA_PATH = REPO_ROOT / "specs" / "compliance-pack-schema.json"
PACK_FIXTURE_PATH = REPO_ROOT / "specs" / "fixtures" / "compliance-pack" / "compliance-001-soc2-cmp-portability.fixture.json"
PORTABILITY_FIXTURE_PATH = REPO_ROOT / "specs" / "fixtures" / "compliance-pack" / "compliance-001-portability-export-manifest.fixture.json"

REQUIRED_RELATED_ADRS = {"ADR-0209", "ADR-0241", "ADR-0250", "ADR-0251", "ADR-0272", "ADR-0276"}
REQUIRED_SCHEMA_PROPERTIES = {
    "signature_contract",
    "evidence_collectors",
    "cell_certification_state",
    "cmp_consent",
    "portability_export",
}
REQUIRED_SIGNATURE_CONTRACT_FIELDS = {
    "canonicalization",
    "signature_algorithm",
    "signature_scope",
    "verification_inputs",
    "claim_boundary",
}
REQUIRED_VERIFICATION_INPUTS = {
    "public_key_ref",
    "canonical_bundle_digest_sha256",
    "signing_principal",
    "signed_at",
}
REQUIRED_COLLECTOR_KINDS = {
    "ci_artifact_hash",
    "deploy_receipt",
    "access_review_snapshot",
    "backup_restore_drill_receipt",
    "vulnerability_scan_report",
    "pen_test_report",
}
REQUIRED_CELL_EVIDENCE_KINDS = REQUIRED_COLLECTOR_KINDS
CMP_CANONICAL_PURPOSES = [
    "necessary",
    "preference",
    "statistics",
    "marketing",
    "personalization",
]
REQUIRED_PORTABILITY_MODES = {"full", "incremental"}
REQUIRED_NONCLAIM_MARKERS = {
    "contract fixture only",
    "no",
    "not",
    "does not",
    "without claiming",
}
FORBIDDEN_ASSERTIVE_PATTERNS = [
    re.compile(pattern)
    for pattern in [
        r"\bproduction\s+ready\b",
        r"\bprod\s+ready\b",
        r"\bruntime\s+(collector|cmp|portability|export|evidence)\s+(is\s+)?(implemented|live|available|ready)\b",
        r"\baudit(or)?\s+accepted\s+(status|evidence|report)\b",
        r"\bcertification\s+(achieved|complete|passed)\b",
        r"\bcertified\s+for\s+launch\b",
        r"\bsoc\s*2\s+(certified|complete|ready|passed)\b",
        r"\btenant\s+activation\s+(ready|enabled|complete)\b",
        r"\bdsr\s+api\s+(ready|available|implemented|live)\b",
    ]
]


def fail(message: str) -> NoReturn:
    print(f"compliance-pack contract slice check failed: {message}", file=sys.stderr)
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


def text(value: object) -> str:
    if isinstance(value, dict):
        return " ".join(text(v) for v in value.values())
    if isinstance(value, (list, tuple, set)):
        return " ".join(text(v) for v in value)
    return str(value).lower()


def normalized(value: object) -> str:
    return re.sub(r"[^a-z0-9]+", " ", text(value)).strip()


def contains_forbidden_assertive_claim(value: object) -> bool:
    haystack = f" {normalized(value)} "
    return any(pattern.search(haystack) for pattern in FORBIDDEN_ASSERTIVE_PATTERNS)


def without_claim_boundary_fields(value: object) -> object:
    if isinstance(value, dict):
        return {
            key: without_claim_boundary_fields(val)
            for key, val in value.items()
            if key not in {"claim_boundary", "claim_ceiling"}
        }
    if isinstance(value, list):
        return [without_claim_boundary_fields(item) for item in value]
    return value


def require_explicit_nonclaim(label: str, value: str) -> None:
    lowered = value.lower()
    require(any(marker in lowered for marker in REQUIRED_NONCLAIM_MARKERS), f"{label}: missing explicit non-claim wording")


def validate_schema(schema: dict) -> None:
    require(schema.get("version") == "1.1.0", "schema version must be bumped to 1.1.0 for COMPLIANCE-001")
    require(schema.get("_meta", {}).get("spec_id") == "EXE-COMPLIANCE-PACK-SCHEMA", "unexpected schema spec_id")
    related = set(schema.get("_meta", {}).get("related_adrs", []))
    require(REQUIRED_RELATED_ADRS <= related, f"schema related_adrs missing {sorted(REQUIRED_RELATED_ADRS - related)}")
    require_explicit_nonclaim("schema._meta.claim_boundary", schema.get("_meta", {}).get("claim_boundary", ""))

    properties = schema.get("properties", {})
    require(REQUIRED_SCHEMA_PROPERTIES <= set(properties), f"schema missing properties {sorted(REQUIRED_SCHEMA_PROPERTIES - set(properties))}")

    signature_contract = properties["signature_contract"]
    require(set(signature_contract.get("required", [])) == REQUIRED_SIGNATURE_CONTRACT_FIELDS, "signature_contract required fields must be exact")
    verification_inputs = signature_contract.get("properties", {}).get("verification_inputs", {})
    require(set(verification_inputs.get("required", [])) == REQUIRED_VERIFICATION_INPUTS, "signature verification input fields must be exact")
    require(signature_contract["properties"]["canonicalization"].get("enum") == ["canonical-cbor-excluding-signature"], "signature canonicalization must stay canonical CBOR excluding signature")
    require(signature_contract["properties"]["signature_algorithm"].get("enum") == ["Ed25519"], "signature algorithm must stay Ed25519")

    collectors = properties["evidence_collectors"].get("items", {})
    require(REQUIRED_SIGNATURE_CONTRACT_FIELDS - {"canonicalization", "signature_algorithm", "signature_scope", "verification_inputs"} <= set(collectors.get("required", [])), "evidence collectors must require claim_boundary")
    require("auditor_publishability" in collectors.get("required", []), "evidence collectors must require auditor_publishability")

    cell_state = properties["cell_certification_state"]
    require({"state", "certification_level_ref", "evidence_required", "launch_gate_policy", "claim_boundary"} <= set(cell_state.get("required", [])), "cell certification state required fields incomplete")
    require("evidence_collecting" in cell_state["properties"]["state"].get("enum", []), "cell certification state must include evidence_collecting")
    require("certified" in cell_state["properties"]["state"].get("enum", []), "cell certification state must include certified as a gated terminal state")
    require("deny-unless-evidence-collecting-sandbox-only" in cell_state["properties"]["launch_gate_policy"].get("enum", []), "cell launch gate must support sandbox-only evidence collection")

    cmp = properties["cmp_consent"]
    require({"canonical_purposes", "strictly_necessary_purpose", "non_necessary_default", "no_accept_all_default", "audit_event_class", "storage_contract", "claim_boundary"} <= set(cmp.get("required", [])), "CMP consent required fields incomplete")
    prefix_consts = [item.get("const") for item in cmp["properties"]["canonical_purposes"].get("prefixItems", [])]
    require(prefix_consts == CMP_CANONICAL_PURPOSES, "CMP canonical purposes must match ADR-0272 order")
    require(cmp["properties"]["non_necessary_default"].get("const") is False, "CMP non-necessary purposes must default false")
    require(cmp["properties"]["no_accept_all_default"].get("const") is True, "CMP must forbid accept-all as the default")

    portability = properties["portability_export"]
    require({"format", "bundle_media_type", "manifest_schema_ref", "supported_modes", "signature_contract", "fixture_ref", "claim_boundary"} <= set(portability.get("required", [])), "portability export required fields incomplete")
    require(portability["properties"]["format"].get("enum") == ["JSON-LD-1.1-tar-gzip"], "portability format must remain JSON-LD 1.1 tar.gz")
    require(portability["properties"]["bundle_media_type"].get("enum") == ["application/gzip+tar"], "portability media type must remain application/gzip+tar")
    sig_props = portability["properties"]["signature_contract"].get("properties", {})
    for key in ["tenant_signature_required", "oyatie_signature_required", "transparency_log_ref_required"]:
        require(sig_props.get(key, {}).get("const") is True, f"portability signature_contract.{key} must be true")

    require(not contains_forbidden_assertive_claim(without_claim_boundary_fields(schema)), "schema contains forbidden assertive positive-claim wording outside claim-boundary fields")


def validate_pack_fixture(fixture: dict, portability_manifest: dict) -> None:
    require(fixture.get("$schema") == "https://docs.oyatie.com/specs/compliance-pack-schema.schema.json", "pack fixture must point at compliance-pack schema")
    require(fixture.get("_meta", {}).get("fixture_id") == "COMPLIANCE-001-SOC2-CMP-PORTABILITY", "unexpected fixture_id")
    require(fixture.get("_meta", {}).get("status") == "contract-fixture-only", "fixture must remain contract-fixture-only")
    require(set(fixture.get("_meta", {}).get("source_adrs", [])) == REQUIRED_RELATED_ADRS, "fixture source ADRs must exactly cover COMPLIANCE-001 sources")
    require_explicit_nonclaim("fixture._meta.claim_boundary", fixture.get("_meta", {}).get("claim_boundary", ""))

    require(fixture.get("pack_id") == "SOC2-T2", "fixture must be the SOC2-T2 pack slice")
    require(fixture.get("version") == "1.1.0", "fixture version must be 1.1.0")
    require("SOC2" in fixture.get("signed_by", "").upper(), "fixture signature principal should identify SOC2 scope")
    require(re.fullmatch(r"[0-9a-f]{128}", fixture.get("signature", "")), "fixture signature must keep Ed25519 hex shape")

    signature_contract = fixture.get("signature_contract", {})
    require(signature_contract.get("canonicalization") == "canonical-cbor-excluding-signature", "fixture signature canonicalization mismatch")
    require(signature_contract.get("signature_algorithm") == "Ed25519", "fixture signature algorithm mismatch")
    require(signature_contract.get("signature_scope") == "bundle-without-signature-field", "fixture signature scope mismatch")
    require(REQUIRED_VERIFICATION_INPUTS <= set(signature_contract.get("verification_inputs", {})), "fixture signature verification inputs incomplete")
    require_explicit_nonclaim("fixture.signature_contract.claim_boundary", signature_contract.get("claim_boundary", ""))

    collectors = fixture.get("evidence_collectors", [])
    collector_kinds = {collector.get("artifact_kind") for collector in collectors}
    require(REQUIRED_COLLECTOR_KINDS <= collector_kinds, f"missing evidence collectors {sorted(REQUIRED_COLLECTOR_KINDS - collector_kinds)}")
    for collector in collectors:
        require(collector.get("framework") == "SOC2-T2", f"{collector.get('collector_id')}: framework must be SOC2-T2")
        require(collector.get("seal_required") is True, f"{collector.get('collector_id')}: seal_required must be true")
        require(collector.get("auditor_publishability") in {"tenant-admin-visible", "external-auditor-room"}, f"{collector.get('collector_id')}: publishability must be auditor/tenant-visible evidence metadata")
        require_explicit_nonclaim(f"collector {collector.get('collector_id')} claim_boundary", collector.get("claim_boundary", ""))

    cell_state = fixture.get("cell_certification_state", {})
    require(cell_state.get("state") == "evidence_collecting", "fixture must remain evidence_collecting, not certified")
    require(cell_state.get("launch_gate_policy") == "deny-unless-evidence-collecting-sandbox-only", "fixture must remain sandbox-only at launch gate")
    cell_evidence = {item.get("evidence_kind") for item in cell_state.get("evidence_required", [])}
    require(REQUIRED_CELL_EVIDENCE_KINDS <= cell_evidence, f"cell certification state missing evidence kinds {sorted(REQUIRED_CELL_EVIDENCE_KINDS - cell_evidence)}")
    require_explicit_nonclaim("cell_certification_state.claim_boundary", cell_state.get("claim_boundary", ""))

    cmp = fixture.get("cmp_consent", {})
    require(cmp.get("canonical_purposes") == CMP_CANONICAL_PURPOSES, "CMP fixture must pin the five ADR-0272 purposes in order")
    require(cmp.get("strictly_necessary_purpose") == "necessary", "CMP strictly necessary purpose must be necessary")
    require(cmp.get("non_necessary_default") is False, "CMP non-necessary purposes must default off")
    require(cmp.get("no_accept_all_default") is True, "CMP must not use accept-all default")
    storage = cmp.get("storage_contract", {})
    for key in ["signature_required", "prior_record_ref_required", "revocation_one_click"]:
        require(storage.get(key) is True, f"CMP storage_contract.{key} must be true")
    require_explicit_nonclaim("cmp_consent.claim_boundary", cmp.get("claim_boundary", ""))

    portability = fixture.get("portability_export", {})
    require(portability.get("format") == "JSON-LD-1.1-tar-gzip", "portability format mismatch")
    require(portability.get("bundle_media_type") == "application/gzip+tar", "portability media type mismatch")
    require(REQUIRED_PORTABILITY_MODES <= set(portability.get("supported_modes", [])), "portability modes must include full and incremental")
    portability_sig = portability.get("signature_contract", {})
    for key in ["tenant_signature_required", "oyatie_signature_required", "transparency_log_ref_required"]:
        require(portability_sig.get(key) is True, f"portability signature_contract.{key} must be true")
    require(portability.get("fixture_ref") == str(PORTABILITY_FIXTURE_PATH.relative_to(REPO_ROOT)), "portability fixture_ref must point to the manifest fixture")
    require_explicit_nonclaim("portability_export.claim_boundary", portability.get("claim_boundary", ""))

    require(portability_manifest.get("export_id") == "pex-fixture-compliance-001-soc2-0001", "portability manifest export_id mismatch")
    require(portability_manifest.get("format", {}).get("document_format") == "JSON-LD-1.1", "portability manifest must use JSON-LD 1.1")
    require(portability_manifest.get("format", {}).get("container") == "tar.gz", "portability manifest must use tar.gz container")
    require(portability_manifest.get("format", {}).get("bundle_media_type") == "application/gzip+tar", "portability manifest media type mismatch")
    require(len(portability_manifest.get("entries", [])) >= 2, "portability manifest must include at least two synthetic entries")
    for entry in portability_manifest.get("entries", []):
        require(entry.get("media_type") == "application/ld+json", f"{entry.get('path')}: entry media type must be JSON-LD")
        require(re.fullmatch(r"[0-9a-f]{64}", entry.get("sha256", "")), f"{entry.get('path')}: entry digest must be SHA-256 hex")
        require_explicit_nonclaim(f"manifest entry {entry.get('path')} claim_boundary", entry.get("claim_boundary", ""))
    signatures = portability_manifest.get("signatures", {})
    require("tenant_ed25519" in signatures and "oyatie_cosign" in signatures, "portability manifest must include tenant and Oyatie signatures")
    require(portability_manifest.get("audit_chain_proof", {}).get("proof_kind") == "merkle-inclusion-fixture", "portability manifest must carry synthetic audit-chain proof")
    require_explicit_nonclaim("portability_manifest.claim_boundary", portability_manifest.get("claim_boundary", ""))

    require(not contains_forbidden_assertive_claim(without_claim_boundary_fields(fixture)), "pack fixture contains forbidden assertive positive-claim wording outside claim-boundary fields")
    require(not contains_forbidden_assertive_claim(without_claim_boundary_fields(portability_manifest)), "portability manifest contains forbidden assertive positive-claim wording outside claim-boundary fields")


def validate(schema: dict, fixture: dict, portability_manifest: dict) -> None:
    validate_schema(schema)
    validate_pack_fixture(fixture, portability_manifest)


def main() -> None:
    validate(load_json(SCHEMA_PATH), load_json(PACK_FIXTURE_PATH), load_json(PORTABILITY_FIXTURE_PATH))
    print(
        "compliance-pack contract slice check passed: "
        f"{SCHEMA_PATH.relative_to(REPO_ROOT)}, "
        f"{PACK_FIXTURE_PATH.relative_to(REPO_ROOT)}, "
        f"{PORTABILITY_FIXTURE_PATH.relative_to(REPO_ROOT)}"
    )


def run_self_tests() -> None:
    baseline_schema = load_json(SCHEMA_PATH)
    baseline_fixture = load_json(PACK_FIXTURE_PATH)
    baseline_manifest = load_json(PORTABILITY_FIXTURE_PATH)

    def expect_rejected(label: str, mutator: Callable[[dict, dict, dict], None]) -> None:
        schema = copy.deepcopy(baseline_schema)
        fixture = copy.deepcopy(baseline_fixture)
        manifest = copy.deepcopy(baseline_manifest)
        mutator(schema, fixture, manifest)
        try:
            validate(schema, fixture, manifest)
        except SystemExit as exc:
            require(exc.code != 0, f"self-test {label!r} exited successfully")
        else:
            fail(f"self-test mutation was accepted: {label}")

    expect_rejected("missing signature contract schema", lambda schema, fixture, manifest: schema["properties"].pop("signature_contract"))
    expect_rejected("missing ADR-0272 source", lambda schema, fixture, manifest: schema["_meta"].update({"related_adrs": [adr for adr in schema["_meta"]["related_adrs"] if adr != "ADR-0272"]}))
    expect_rejected("wrong CMP order", lambda schema, fixture, manifest: fixture["cmp_consent"].update({"canonical_purposes": ["necessary", "statistics", "preference", "marketing", "personalization"]}))
    expect_rejected("CMP accept-all default", lambda schema, fixture, manifest: fixture["cmp_consent"].update({"no_accept_all_default": False}))
    expect_rejected("certified launch overclaim", lambda schema, fixture, manifest: fixture["cell_certification_state"].update({"state": "certified", "launch_gate_policy": "deny-unless-certified-and-pack-installed"}))
    expect_rejected("missing SOC2 collector", lambda schema, fixture, manifest: fixture.update({"evidence_collectors": [collector for collector in fixture["evidence_collectors"] if collector["artifact_kind"] != "pen_test_report"]}))
    expect_rejected("collector lacks seal", lambda schema, fixture, manifest: fixture["evidence_collectors"][0].update({"seal_required": False}))
    expect_rejected("portability missing incremental", lambda schema, fixture, manifest: fixture["portability_export"].update({"supported_modes": ["full"]}))
    expect_rejected("portability fixture path drift", lambda schema, fixture, manifest: fixture["portability_export"].update({"fixture_ref": "specs/fixtures/wrong.json"}))
    expect_rejected("manifest missing Oyatie signature", lambda schema, fixture, manifest: manifest["signatures"].pop("oyatie_cosign"))
    expect_rejected("manifest digest shape", lambda schema, fixture, manifest: manifest["entries"][0].update({"sha256": "not-a-digest"}))
    expect_rejected("runtime overclaim", lambda schema, fixture, manifest: fixture.update({"purpose": "runtime CMP is implemented"}))
    print("compliance-pack contract slice self-tests passed")


if __name__ == "__main__":
    if "--self-test" in sys.argv[1:]:
        run_self_tests()
    main()
