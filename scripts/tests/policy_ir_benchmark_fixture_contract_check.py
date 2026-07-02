#!/usr/bin/env python3
"""Validate the Policy IR benchmark FixtureSuite verdict schema + adapter contracts.

Sub-AC 1 gate for the owned Policy IR benchmark phase: the engine-neutral verdict
schema (typed-obligation and reverse-query result shapes) and the per-engine
adapter contracts for the Core-6 slate must validate BEFORE any conformance
vectors exist. The check is forward-compatible: once the single-lane fixture
stage lands vectors, each vector's request/expected-verdict shapes are validated
against the same schema.

Stdlib only. Implements the JSON-Schema subset the verdict schema actually uses:
$ref (#/$defs/...), oneOf, type, const, enum, properties, required,
additionalProperties (bool | schema), items, pattern, minLength, minItems,
uniqueItems.
"""
from __future__ import annotations

import copy
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any, Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
SUITE_PATH = REPO_ROOT / "specs" / "policy-ir-benchmark-fixture-suite.json"
RUBRIC_PATH = REPO_ROOT / "specs" / "policy-ir-benchmark-rubric.json"

CORE6 = ("cedar", "spicedb", "openfga", "opa_rego", "cel", "biscuit")
REQUIRED_VERDICT_DEFS = {
    "RequestId",
    "EntityRef",
    "TypedScalar",
    "TypedObligation",
    "EmulationDisclosure",
    "DecisionProvenance",
    "ForwardAllowVerdict",
    "ForwardDenyVerdict",
    "ReversePrincipalsVerdict",
    "ReverseResourcesVerdict",
    "RequestContext",
    "ForwardRequest",
    "ReversePrincipalsRequest",
    "ReverseResourcesRequest",
    "FixtureRequest",
}
REQUIRED_ADAPTER_FIELDS = {
    "reference_system",
    "fixture_scope",
    "fixture_1_encoding_required",
    "authoring_surface",
    "request_mapping",
    "typed_obligation_binding",
    "reverse_query_binding",
    "consistency_binding",
    "verdict_normalization_rules",
}
REQUIRED_REQUEST_MAPPING_KEYS = {
    "forward_authorization",
    "reverse_principals",
    "reverse_resources",
}
REQUIRED_NORMALIZATION_RULES = {"N1", "N2", "N3", "N4", "N5"}


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


def canonical_hash(document: dict, prefix: str) -> str:
    candidate = copy.deepcopy(document)
    candidate["content_hash"]["value"] = ""
    payload = json.dumps(
        candidate, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode("utf-8")
    return f"{prefix}sha256:{hashlib.sha256(payload).hexdigest()}"


# --- minimal JSON-Schema subset validator -----------------------------------


class SchemaError(ValueError):
    """Raised when an instance fails schema validation."""


def _type_ok(instance: Any, expected: str) -> bool:
    if expected == "object":
        return isinstance(instance, dict)
    if expected == "array":
        return isinstance(instance, list)
    if expected == "string":
        return isinstance(instance, str)
    if expected == "integer":
        return isinstance(instance, int) and not isinstance(instance, bool)
    if expected == "number":
        return isinstance(instance, (int, float)) and not isinstance(instance, bool)
    if expected == "boolean":
        return isinstance(instance, bool)
    if expected == "null":
        return instance is None
    raise ValueError(f"unsupported schema type: {expected}")


def _deep_eq(left: Any, right: Any) -> bool:
    if isinstance(left, bool) is not isinstance(right, bool):
        return False
    if type(left) is not type(right):
        return False
    return left == right


def _resolve_ref(ref: str, root: dict) -> dict:
    require(ref.startswith("#/"), f"unsupported $ref form: {ref}")
    node: Any = root
    for part in ref[2:].split("/"):
        require(isinstance(node, dict) and part in node, f"dangling $ref: {ref}")
        node = node[part]
    return node


def validate_instance(instance: Any, schema: dict, root: dict, path: str = "$") -> None:
    if "$ref" in schema:
        validate_instance(instance, _resolve_ref(schema["$ref"], root), root, path)
        return
    if "oneOf" in schema:
        matches = 0
        errors: list[str] = []
        for index, sub in enumerate(schema["oneOf"]):
            try:
                validate_instance(instance, sub, root, f"{path}<oneOf:{index}>")
                matches += 1
            except SchemaError as exc:
                errors.append(str(exc))
        if matches != 1:
            raise SchemaError(
                f"{path}: oneOf matched {matches} branches (expected exactly 1); "
                f"branch errors: {errors[:3]}"
            )
        return
    if "const" in schema and not _deep_eq(instance, schema["const"]):
        raise SchemaError(f"{path}: expected const {schema['const']!r}, got {instance!r}")
    if "enum" in schema and not any(_deep_eq(instance, item) for item in schema["enum"]):
        raise SchemaError(f"{path}: {instance!r} not in enum {schema['enum']!r}")
    if "type" in schema and not _type_ok(instance, schema["type"]):
        raise SchemaError(f"{path}: expected type {schema['type']}, got {type(instance).__name__}")
    if isinstance(instance, str):
        if "pattern" in schema and re.search(schema["pattern"], instance) is None:
            raise SchemaError(f"{path}: {instance!r} does not match pattern {schema['pattern']!r}")
        if "minLength" in schema and len(instance) < schema["minLength"]:
            raise SchemaError(f"{path}: shorter than minLength {schema['minLength']}")
    if isinstance(instance, dict):
        for key in schema.get("required", []):
            if key not in instance:
                raise SchemaError(f"{path}: missing required property {key!r}")
        properties = schema.get("properties", {})
        additional = schema.get("additionalProperties", True)
        for key, value in instance.items():
            if key in properties:
                validate_instance(value, properties[key], root, f"{path}.{key}")
            elif additional is False:
                raise SchemaError(f"{path}: additional property {key!r} not allowed")
            elif isinstance(additional, dict):
                validate_instance(value, additional, root, f"{path}.{key}")
    if isinstance(instance, list):
        if "minItems" in schema and len(instance) < schema["minItems"]:
            raise SchemaError(f"{path}: fewer than minItems {schema['minItems']}")
        if schema.get("uniqueItems"):
            seen = set()
            for item in instance:
                marker = json.dumps(item, sort_keys=True)
                if marker in seen:
                    raise SchemaError(f"{path}: duplicate item {item!r} (uniqueItems)")
                seen.add(marker)
        if "items" in schema:
            for index, item in enumerate(instance):
                validate_instance(item, schema["items"], root, f"{path}[{index}]")


# --- contract checks ---------------------------------------------------------


def check_hashes(suite: dict, rubric: dict) -> None:
    require(
        rubric["content_hash"]["value"]
        == canonical_hash(rubric, "rubric-v1:"),
        "rubric content-hash pin does not recompute — grade pinning basis broken",
    )
    suite_pin = suite["content_hash"]["value"]
    require(
        re.fullmatch(r"fixture-suite-v[0-9]+:sha256:[0-9a-f]{64}", suite_pin or ""),
        "fixture-suite content_hash.value missing or malformed",
    )
    require(
        suite_pin == canonical_hash(suite, "fixture-suite-v1:"),
        "fixture-suite content-hash pin does not recompute",
    )


def check_stage_gating(suite: dict) -> None:
    stage = suite["stage_contract"]
    vectors = suite["conformance_vectors"]
    require(isinstance(vectors, list), "conformance_vectors must be an array")
    if stage["vectors_present"] is False:
        require(
            vectors == [],
            "pre-vector invariant violated: vectors_present=false but conformance_vectors is non-empty",
        )
    else:
        require(vectors, "vectors_present=true but conformance_vectors is empty")
    require(
        "single-lane" in stage["single_lane_rule"].lower()
        or "single lane" in stage["single_lane_rule"].lower(),
        "stage_contract must state the single-lane ownership rule",
    )


def check_verdict_schema(suite: dict) -> dict:
    schema = suite["verdict_schema"]
    defs = schema.get("$defs", {})
    missing = REQUIRED_VERDICT_DEFS - set(defs)
    require(not missing, f"verdict_schema missing $defs: {sorted(missing)}")

    deny = defs["ForwardDenyVerdict"]
    require(
        _deep_eq(deny["properties"]["obligations"].get("const"), []),
        "ForwardDenyVerdict must pin obligations to const [] (forbid-wins/empty-on-deny)",
    )
    obligation = defs["TypedObligation"]
    params = obligation["properties"]["params"]
    require(
        params.get("type") == "object"
        and params.get("additionalProperties") == {"$ref": "#/$defs/TypedScalar"},
        "TypedObligation.params must be a typed map of TypedScalar values",
    )
    require(
        set(obligation["required"]) == {"kind", "key", "params"},
        "TypedObligation must require kind/key/params",
    )
    require(
        obligation["properties"]["kind"]["enum"] == ["obligation", "advice"],
        "TypedObligation.kind must be the obligation/advice enum",
    )
    provenance = defs["DecisionProvenance"]
    require(
        set(provenance["properties"]["engine_adapter"]["enum"]) == set(CORE6),
        "DecisionProvenance.engine_adapter enum must be exactly the Core-6 slate",
    )
    require(
        {"decision_id", "engine_adapter", "policy_version", "fixture_content_hash", "emulation"}
        <= set(provenance["required"]),
        "DecisionProvenance must require decision_id/engine_adapter/policy_version/fixture_content_hash/emulation",
    )
    for name in ("ReversePrincipalsVerdict", "ReverseResourcesVerdict"):
        node = defs[name]
        require(
            node["properties"]["completeness"]["enum"] == ["complete", "truncated"],
            f"{name}.completeness must be the complete/truncated enum",
        )
        collection = "principals" if name == "ReversePrincipalsVerdict" else "resources"
        require(
            node["properties"][collection].get("uniqueItems") is True,
            f"{name}.{collection} must declare uniqueItems",
        )
    return schema


def check_adapter_contracts(suite: dict, rubric: dict) -> None:
    contracts = suite["adapter_contracts"]
    require(
        set(contracts) == set(CORE6),
        f"adapter_contracts must cover exactly the Core-6 slate; got {sorted(contracts)}",
    )
    rubric_scopes = {
        row["reference_system"]: row["fixture_scope"]
        for row in rubric["reference_slate"]["spiked_systems"]
    }
    for system in CORE6:
        contract = contracts[system]
        missing = REQUIRED_ADAPTER_FIELDS - set(contract)
        require(not missing, f"{system}: adapter contract missing fields {sorted(missing)}")
        require(
            contract["reference_system"] == system,
            f"{system}: reference_system mismatch",
        )
        require(
            contract["fixture_scope"] == rubric_scopes[system],
            f"{system}: fixture_scope {contract['fixture_scope']!r} diverges from frozen rubric "
            f"{rubric_scopes[system]!r}",
        )
        require(
            set(contract["request_mapping"]) == REQUIRED_REQUEST_MAPPING_KEYS,
            f"{system}: request_mapping must cover all three query kinds",
        )
        for binding_name in ("typed_obligation_binding", "reverse_query_binding"):
            binding = contract[binding_name]
            require(
                binding.get("native_surface") and binding.get("result_shape_binding"),
                f"{system}: {binding_name} must declare native_surface and result_shape_binding",
            )
            require(
                binding.get("disclosure_rule"),
                f"{system}: {binding_name} must declare a disclosure_rule (rubric N/E boundary feed)",
            )
        reverse = contract["reverse_query_binding"]
        require(
            reverse.get("principals_api") and reverse.get("resources_api"),
            f"{system}: reverse_query_binding must bind both reverse directions",
        )
        consistency = contract["consistency_binding"]
        require(
            consistency.get("mechanism") and consistency.get("verdict_fields"),
            f"{system}: consistency_binding must declare mechanism and verdict_fields",
        )
        require(
            set(contract["verdict_normalization_rules"]) == REQUIRED_NORMALIZATION_RULES,
            f"{system}: verdict_normalization_rules must bind exactly {sorted(REQUIRED_NORMALIZATION_RULES)}",
        )
    require(
        contracts["biscuit"]["fixture_1_encoding_required"] is False
        and "delegation" in contracts["biscuit"].get("scope_note", ""),
        "biscuit: must declare fixture_1_encoding_required=false with the delegation-only scope note",
    )
    require(
        contracts["cel"]["fixture_scope"] == "statutory_only"
        and contracts["cel"]["fixture_1_encoding_required"] is True,
        "cel: statutory_only scope must still require Fixture-1 encoding",
    )
    for system in ("cedar", "spicedb", "openfga", "opa_rego"):
        require(
            contracts[system]["fixture_1_encoding_required"] is True,
            f"{system}: full-scope system must require Fixture-1 encoding",
        )
    rule_ids = {rule["rule_id"] for rule in suite["adapter_contract_common"]["rules"]}
    require(
        rule_ids == REQUIRED_NORMALIZATION_RULES,
        f"adapter_contract_common must define exactly rules {sorted(REQUIRED_NORMALIZATION_RULES)}",
    )


def _entity_sort_key(entity: dict) -> tuple:
    return (entity["entity_type"], entity["entity_id"])


def check_examples(suite: dict, schema: dict) -> None:
    examples = suite["contract_validation_examples"]["examples"]
    require(examples, "contract_validation_examples must not be empty")
    seen_ids = set()
    coverage = {
        "valid_forward_allow_with_obligations": False,
        "valid_forward_deny": False,
        "valid_reverse_principals": False,
        "valid_reverse_resources": False,
        "valid_fixture_request": False,
        "invalid_deny_with_obligations": False,
        "invalid_untyped_params": False,
        "invalid_emulated_without_shape": False,
        "invalid_unknown_engine": False,
    }
    for example in examples:
        example_id = example["example_id"]
        require(example_id not in seen_ids, f"duplicate example_id {example_id}")
        seen_ids.add(example_id)
        target = example["target"]
        require(target in {"verdict", "fixture_request"}, f"{example_id}: unknown target {target!r}")
        entry = schema if target == "verdict" else {"$ref": "#/$defs/FixtureRequest"}
        instance = example["instance"]
        try:
            validate_instance(instance, entry, schema)
            outcome_valid = True
            error = ""
        except SchemaError as exc:
            outcome_valid = False
            error = str(exc)
        if example["expect_valid"]:
            require(outcome_valid, f"{example_id}: expected valid but schema rejected it: {error}")
        else:
            require(not outcome_valid, f"{example_id}: expected invalid but schema accepted it")
        if outcome_valid and target == "verdict":
            kind = instance["verdict_kind"]
            if kind == "reverse_principals":
                collection = instance["principals"]
            elif kind == "reverse_resources":
                collection = instance["resources"]
            else:
                collection = None
            if collection is not None:
                require(
                    collection == sorted(collection, key=_entity_sort_key),
                    f"{example_id}: reverse result set violates N1 canonical ordering",
                )
        # coverage bookkeeping
        if example["expect_valid"] and target == "verdict":
            if instance.get("decision") == "allow" and instance.get("obligations"):
                coverage["valid_forward_allow_with_obligations"] = True
            if instance.get("decision") == "deny":
                coverage["valid_forward_deny"] = True
            if instance.get("verdict_kind") == "reverse_principals":
                coverage["valid_reverse_principals"] = True
            if instance.get("verdict_kind") == "reverse_resources":
                coverage["valid_reverse_resources"] = True
        if example["expect_valid"] and target == "fixture_request":
            coverage["valid_fixture_request"] = True
        if not example["expect_valid"] and target == "verdict":
            if instance.get("decision") == "deny" and instance.get("obligations"):
                coverage["invalid_deny_with_obligations"] = True
            obligations = instance.get("obligations")
            if isinstance(obligations, list) and any(
                isinstance(o, dict) and isinstance(o.get("params"), str) for o in obligations
            ):
                coverage["invalid_untyped_params"] = True
            emulation = instance.get("provenance", {}).get("emulation", {})
            if emulation.get("emulated") is True and "adapter_shape" not in emulation:
                coverage["invalid_emulated_without_shape"] = True
            if instance.get("provenance", {}).get("engine_adapter") not in CORE6:
                coverage["invalid_unknown_engine"] = True
    missing = [name for name, hit in coverage.items() if not hit]
    require(not missing, f"contract_validation_examples missing required coverage: {missing}")


def check_vectors(suite: dict, schema: dict) -> None:
    """Forward-compatible: once vectors land, each must carry schema-valid shapes."""
    for index, vector in enumerate(suite["conformance_vectors"]):
        require(isinstance(vector, dict), f"vector[{index}] must be an object")
        require(
            "request" in vector and "expected_verdict" in vector,
            f"vector[{index}] must carry request and expected_verdict",
        )
        try:
            validate_instance(vector["request"], {"$ref": "#/$defs/FixtureRequest"}, schema)
            validate_instance(vector["expected_verdict"], schema, schema)
        except SchemaError as exc:
            fail(f"vector[{index}] fails verdict schema: {exc}")


def validate(suite: dict, rubric: dict, check_hash: bool = True) -> None:
    require(suite["_meta"]["spec_id"] == "POL-IR-BENCH-FIXTURES", "unexpected spec_id")
    require(
        suite["_meta"]["rubric_ref"] == "specs/policy-ir-benchmark-rubric.json",
        "_meta.rubric_ref must pin the frozen rubric document",
    )
    if check_hash:
        check_hashes(suite, rubric)
    check_stage_gating(suite)
    schema = check_verdict_schema(suite)
    check_adapter_contracts(suite, rubric)
    check_examples(suite, schema)
    check_vectors(suite, schema)


def main() -> None:
    suite = load_json(SUITE_PATH)
    rubric = load_json(RUBRIC_PATH)
    validate(suite, rubric)
    vectors = len(suite["conformance_vectors"])
    print(
        "policy IR benchmark fixture schema/contract check passed: "
        f"{SUITE_PATH.relative_to(REPO_ROOT)} "
        f"(engines={len(suite['adapter_contracts'])}, "
        f"examples={len(suite['contract_validation_examples']['examples'])}, "
        f"conformance_vectors={vectors})"
    )


def run_self_tests() -> None:
    suite = load_json(SUITE_PATH)
    rubric = load_json(RUBRIC_PATH)

    def expect_rejected(label: str, mutator: Callable[[dict], None], hash_check: bool = False) -> None:
        candidate = copy.deepcopy(suite)
        mutator(candidate)
        try:
            validate(candidate, rubric, check_hash=hash_check)
        except SystemExit as exc:
            require(exc.code != 0, f"self-test {label!r} exited successfully")
        else:
            fail(f"self-test mutation was accepted: {label}")

    def set_deny_obligations(data: dict) -> None:
        for example in data["contract_validation_examples"]["examples"]:
            if example["example_id"] == "ex-v-002":
                example["instance"]["obligations"] = [
                    {"kind": "obligation", "key": "apply_vat", "params": {"schedule": "x"}}
                ]

    expect_rejected("stale content hash", lambda d: d["stage_contract"].update({"stage_1_scope": "tampered"}), hash_check=True)
    expect_rejected("vectors while vectors_present=false", lambda d: d["conformance_vectors"].append({"request": {}, "expected_verdict": {}}))
    expect_rejected("dropped engine contract", lambda d: d["adapter_contracts"].pop("biscuit"))
    expect_rejected("extra non-slate engine contract", lambda d: d["adapter_contracts"].update({"casbin": d["adapter_contracts"]["cedar"]}))
    expect_rejected("fixture_scope drift from rubric", lambda d: d["adapter_contracts"]["cel"].update({"fixture_scope": "full"}))
    expect_rejected("deny verdict grows obligations", set_deny_obligations)
    expect_rejected("untyped obligation params accepted", lambda d: d["verdict_schema"]["$defs"]["TypedObligation"]["properties"].update({"params": {"type": "string"}}))
    expect_rejected("missing reverse direction binding", lambda d: d["adapter_contracts"]["spicedb"]["reverse_query_binding"].pop("resources_api"))
    expect_rejected("missing disclosure rule", lambda d: d["adapter_contracts"]["openfga"]["typed_obligation_binding"].pop("disclosure_rule"))
    expect_rejected("completeness enum widened", lambda d: d["verdict_schema"]["$defs"]["ReversePrincipalsVerdict"]["properties"]["completeness"].update({"enum": ["complete", "truncated", "partial"]}))
    expect_rejected("unsorted reverse example accepted", lambda d: next(e for e in d["contract_validation_examples"]["examples"] if e["example_id"] == "ex-v-003")["instance"]["principals"].reverse())
    expect_rejected("biscuit fixture-1 requirement flipped", lambda d: d["adapter_contracts"]["biscuit"].update({"fixture_1_encoding_required": True, "scope_note": "delegation subset"}))
    print("policy IR benchmark fixture schema/contract self-tests passed")


if __name__ == "__main__":
    if "--self-test" in sys.argv[1:]:
        run_self_tests()
    main()
