#!/usr/bin/env python3
"""Fail-closed guard for TLS-001 ECH runtime reconciliation claims.

TLS-001 may encode a <=24h ECH rotation posture for `POST /edge/admission`,
but the current api-gateway ECH runtime inventory still documents a 90-day
rotation cadence. This check permits that mismatch only when the contract and
snapshot explicitly block live-compliance/runtime-promotion claims until accepted
authority elevates or replaces the Proposed ADR-0354 target.
"""
from __future__ import annotations

import json
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any, Callable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
OPENAPI_PATH = REPO_ROOT / "oya" / "api-gateway" / "contracts" / "api-gateway.openapi.yaml"
ECH_CONFIG_PATH = REPO_ROOT / "oya" / "api-gateway" / "iac" / "ech-config.yaml"
SNAPSHOT_PATH = REPO_ROOT / "evidence" / "contract-snapshots" / "tls-001-http3-tls-ech-pqc-posture-20260701.md"

ACCEPTED_CRYPTO_AUTHORITY = "docs/decisions/ADR-0506-aws-lc-rs-canonical-crypto-provider.md"
CONTEXTUAL_TLS_AUTHORITY = "docs/decisions/ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc.md"
EXPECTED_STATUS = "runtime_design_only_until_accepted_authority_elevates_ech_rotation"


class CheckFailure(Exception):
    pass


def fail(message: str) -> NoReturn:
    print(f"TLS-001 ECH runtime reconciliation check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require(condition: object, message: str) -> None:
    if not condition:
        raise CheckFailure(message)


def rel(path: Path) -> str:
    try:
        return str(path.resolve().relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def text(value: object) -> str:
    if isinstance(value, dict):
        return " ".join(text(item) for item in value.values())
    if isinstance(value, list):
        return " ".join(text(item) for item in value)
    return str(value).lower()


def load_yaml(path: Path) -> Any:
    if shutil.which("ruby") is None:
        fail("ruby is required for YAML parsing in this repository check")
    ruby = """
require 'yaml'
require 'json'
require 'date'
begin
  docs = YAML.load_stream(File.read(ARGV.fetch(0)))
  puts JSON.generate(docs.length == 1 ? docs.first : docs)
rescue StandardError => e
  warn "YAML parse failed for #{ARGV.fetch(0)}: #{e.class}: #{e.message}"
  exit 2
end
"""
    result = subprocess.run(
        ["ruby", "-e", ruby, str(path)],
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        fail(result.stderr.strip() or f"failed to parse YAML: {rel(path)}")
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        fail(f"ruby YAML parser returned invalid JSON for {rel(path)}: {exc}")


def route_posture(openapi: dict[str, Any]) -> dict[str, Any]:
    try:
        posture = openapi["paths"]["/edge/admission"]["post"]["x-oyatie-transport-security-posture"]
    except KeyError as exc:
        fail(f"OpenAPI missing TLS-001 transport posture key: {exc}")
    require(isinstance(posture, dict), "transport security posture must be an object")
    return posture


def data_value(config: Any, key: str) -> str:
    docs = config if isinstance(config, list) else [config]
    for doc in docs:
        if isinstance(doc, dict):
            data = doc.get("data")
            if isinstance(data, dict) and key in data:
                return str(data[key])
    fail(f"{rel(ECH_CONFIG_PATH)} missing data.{key}")


def validate(openapi: dict[str, Any], ech_config: Any, snapshot_text: str) -> None:
    posture = route_posture(openapi)
    authority = posture.get("authority", {})
    require(authority.get("accepted") == [ACCEPTED_CRYPTO_AUTHORITY], "accepted authority must remain limited to ADR-0506")
    require(
        CONTEXTUAL_TLS_AUTHORITY in authority.get("contextual_not_binding", []),
        "ADR-0354 must remain contextual_not_binding until accepted/elevated",
    )
    guardrail = text(authority.get("proposed_adr_guardrail", ""))
    require("runtime" in guardrail and "accepted" in guardrail and "authority" in guardrail, "proposed ADR guardrail must block runtime promotion without accepted authority")

    claim_ceiling = text(posture.get("claim_ceiling", ""))
    for marker in ["no runtime", "ech key rotation", "live handshake", "production-readiness claim"]:
        require(marker in claim_ceiling, f"claim ceiling missing {marker!r}")

    ech = posture.get("ech", {})
    require(ech.get("enabled") is True, "posture ECH target must stay enabled")
    require(ech.get("key_rotation_hours_max") == 24, "posture ECH target must stay <=24h")
    require(ech.get("dns_ttl_seconds_max") == 3600, "posture DNS TTL target must stay <=3600s")

    rotation_days = int(data_value(ech_config, "ech-config-rotation-days"))
    target_hours = int(ech["key_rotation_hours_max"])
    mismatch = rotation_days * 24 > target_hours
    require(mismatch, "this reconciliation guard is only meaningful while runtime inventory exceeds the target")

    inventory_text = text(posture.get("current_state_inventory_read_only", []))
    require("90 days" in inventory_text and "reconciled before live ech compliance" in inventory_text, "current-state inventory must explicitly document the 90-day mismatch and live-claim blocker")

    reconciliation = posture.get("runtime_reconciliation_guard")
    require(isinstance(reconciliation, dict), "posture must carry runtime_reconciliation_guard")
    require(reconciliation.get("status") == EXPECTED_STATUS, f"runtime_reconciliation_guard.status must be {EXPECTED_STATUS}")
    require(reconciliation.get("accepted_runtime_authority_present") is False, "guard must record no accepted runtime authority is present")
    require(reconciliation.get("runtime_promotion_blocked") is True, "guard must block runtime promotion")
    require(reconciliation.get("no_live_ech_compliance_claim") is True, "guard must forbid live ECH compliance claims")
    require(reconciliation.get("runtime_inventory_rotation_days") == rotation_days, "guard must echo runtime inventory rotation days")
    require(reconciliation.get("target_rotation_hours_max") == target_hours, "guard must echo target rotation hours")
    require(reconciliation.get("mismatch_class") == "target_stricter_than_runtime_inventory", "guard must classify the stricter target/runtime mismatch")
    require(
        CONTEXTUAL_TLS_AUTHORITY in reconciliation.get("contextual_authority", []),
        "guard must cite ADR-0354 as contextual authority only",
    )
    require(
        ACCEPTED_CRYPTO_AUTHORITY in reconciliation.get("accepted_authority", []),
        "guard must cite ADR-0506 as the accepted crypto-provider authority",
    )
    required_before = text(reconciliation.get("required_before_runtime_promotion", []))
    for marker in ["accepted adr", "root-pointer", "runtime", "rollout", "rollback", "observability"]:
        require(marker in required_before, f"runtime promotion requirements missing {marker!r}")

    snapshot = snapshot_text.lower()
    for marker in [
        "runtime design-only",
        "runtime promotion remains blocked",
        "no live ech compliance claim",
        "90-day rotation",
        "<=24h target",
        "accepted adr/root-pointer authority",
        "rollback",
        "observability",
    ]:
        require(marker in snapshot, f"snapshot missing non-claim/reconciliation marker {marker!r}")


def main() -> None:
    try:
        validate(
            load_yaml(OPENAPI_PATH),
            load_yaml(ECH_CONFIG_PATH),
            SNAPSHOT_PATH.read_text(encoding="utf-8"),
        )
    except FileNotFoundError as exc:
        fail(f"missing file: {exc.filename}")
    except CheckFailure as exc:
        fail(str(exc))
    print("TLS-001 ECH runtime reconciliation check passed")


def run_self_tests() -> None:
    baseline_openapi = load_yaml(OPENAPI_PATH)
    baseline_ech = load_yaml(ECH_CONFIG_PATH)
    baseline_snapshot = SNAPSHOT_PATH.read_text(encoding="utf-8")

    def expect_rejected(label: str, mutator: Callable[[dict[str, Any], Any, str], tuple[dict[str, Any], Any, str]]) -> None:
        openapi = json.loads(json.dumps(baseline_openapi))
        ech = json.loads(json.dumps(baseline_ech))
        snapshot = baseline_snapshot
        openapi, ech, snapshot = mutator(openapi, ech, snapshot)
        try:
            validate(openapi, ech, snapshot)
        except CheckFailure:
            return
        fail(f"self-test mutation was accepted: {label}")

    def remove_guard(openapi: dict[str, Any], ech: Any, snapshot: str) -> tuple[dict[str, Any], Any, str]:
        route_posture(openapi).pop("runtime_reconciliation_guard", None)
        return openapi, ech, snapshot

    def overclaim(openapi: dict[str, Any], ech: Any, snapshot: str) -> tuple[dict[str, Any], Any, str]:
        route_posture(openapi)["claim_ceiling"] = "live ECH compliance claim approved"
        return openapi, ech, snapshot

    def accept_proposed_authority(openapi: dict[str, Any], ech: Any, snapshot: str) -> tuple[dict[str, Any], Any, str]:
        route_posture(openapi)["authority"]["accepted"].append(CONTEXTUAL_TLS_AUTHORITY)
        return openapi, ech, snapshot

    def erase_snapshot_nonclaim(openapi: dict[str, Any], ech: Any, snapshot: str) -> tuple[dict[str, Any], Any, str]:
        return openapi, ech, snapshot.replace("No live ECH compliance claim", "Live ECH compliance claim")

    expect_rejected("missing runtime reconciliation guard", remove_guard)
    expect_rejected("positive live-compliance overclaim", overclaim)
    expect_rejected("Proposed ADR elevated without accepted/root-pointer path", accept_proposed_authority)
    expect_rejected("snapshot live-compliance nonclaim removed", erase_snapshot_nonclaim)
    print("TLS-001 ECH runtime reconciliation self-tests passed")


if __name__ == "__main__":
    if "--self-test" in sys.argv[1:]:
        run_self_tests()
    main()
