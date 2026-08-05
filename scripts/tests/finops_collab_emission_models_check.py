#!/usr/bin/env python3
"""Validate SPEC-FINOPS-EMISSION-BATCH-COLLAB-001 plan/spec-only emission model declarations.

This test is intentionally narrow: it validates the collaboration/communication
service manifests and IP notes touched by Kanban task t_dadd6099. It does not
claim runtime emission instrumentation or live FinOps readiness.
"""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

EXPECTED = {
    "mail": {
        "signal_family": "per_message",
        "mwh": 4.8,
        "co2": 0.0019,
        "coefficients": (0.61, 0.0030, 0.00120, 0.083),
        "must_mention": ["message delivery", "legal hold", "e-discovery"],
    },
    "calendar": {
        "signal_family": "per_request",
        "mwh": 1.7,
        "co2": 0.0007,
        "coefficients": (0.44, 0.0022, 0.00045, 0.041),
        "must_mention": ["event create", "free/busy", "notification fanout"],
    },
    "social": {
        "signal_family": "per_request",
        "mwh": 3.6,
        "co2": 0.0014,
        "coefficients": (0.58, 0.0028, 0.00105, 0.078),
        "must_mention": ["feed render", "reaction", "moderation"],
    },
    "community": {
        "signal_family": "per_request",
        "mwh": 2.9,
        "co2": 0.0012,
        "coefficients": (0.51, 0.0024, 0.00075, 0.052),
        "must_mention": ["post create", "comment", "moderation"],
    },
    "workflow-studio": {
        "signal_family": "per_request",
        "mwh": 3.2,
        "co2": 0.0013,
        "coefficients": (0.57, 0.0027, 0.00055, 0.071),
        "must_mention": ["canvas", "CRDT", "builder action"],
    },
    "docs": {
        "signal_family": "per_request",
        "mwh": 7.4,
        "co2": 0.0030,
        "coefficients": (0.53, 0.0029, 0.00125, 0.064),
        "must_mention": ["document open", "collab cursor", "export PDF"],
    },
    "drive": {
        "signal_family": "storage_byte_hour",
        "mwh": 28.5,
        "co2": 0.0114,
        "coefficients": (0.72, 0.0034, 0.00165, 0.142),
        "must_mention": ["upload", "download", "immutability tier"],
    },
    "forms": {
        "signal_family": "per_request",
        "mwh": 2.1,
        "co2": 0.0008,
        "coefficients": (0.46, 0.0023, 0.00062, 0.049),
        "must_mention": ["form build", "submission", "export CSV"],
    },
    "meet": {
        "signal_family": "media_minutes",
        "mwh": 42.0,
        "co2": 0.0168,
        "coefficients": (0.69, 0.0032, 0.00070, 0.181),
        "must_mention": ["media minute", "live caption", "MLS handshake"],
    },
    "notes": {
        "signal_family": "per_request",
        "mwh": 1.9,
        "co2": 0.0008,
        "coefficients": (0.43, 0.0021, 0.00058, 0.036),
        "must_mention": ["note create", "full-text search", "graph render"],
    },
    "recordings": {
        "signal_family": "media_minutes",
        "mwh": 56.0,
        "co2": 0.0224,
        "coefficients": (0.74, 0.0033, 0.00180, 0.165),
        "must_mention": ["recording playback", "transcript", "legal hold"],
    },
    "sheets": {
        "signal_family": "per_query",
        "mwh": 5.5,
        "co2": 0.0022,
        "coefficients": (0.59, 0.0031, 0.00090, 0.057),
        "must_mention": ["cell edit", "chart render", "recalculation"],
    },
    "sites": {
        "signal_family": "per_request",
        "mwh": 3.8,
        "co2": 0.0015,
        "coefficients": (0.48, 0.0024, 0.00070, 0.069),
        "must_mention": ["page render", "image optimize", "ACME"],
    },
    "slides": {
        "signal_family": "per_request",
        "mwh": 8.8,
        "co2": 0.0035,
        "coefficients": (0.56, 0.0030, 0.00110, 0.074),
        "must_mention": ["deck open", "collab cursor", "export MP4"],
    },
    "tasks": {
        "signal_family": "per_request",
        "mwh": 2.4,
        "co2": 0.0010,
        "coefficients": (0.47, 0.0024, 0.00063, 0.045),
        "must_mention": ["bulk update", "recurring", "dependency cycle"],
    },
    "translate": {
        "signal_family": "per_request",
        "mwh": 9.6,
        "co2": 0.0038,
        "coefficients": (0.66, 0.0032, 0.00068, 0.095),
        "must_mention": ["batch translate", "language detection", "document translate"],
    },
}

TIER_OVERHEAD = {
    "tier_0_kata_clh": 1.4,
    "tier_1_kata_clh": 1.4,
    "tier_2_runc": 1.0,
    "tier_3_runc_edge": 1.05,
}
ADR_EXAMPLE = (0.5, 0.0025, 0.0008, 0.06)


def load_manifest(service: str) -> dict:
    path = ROOT / "oya" / service / "manifest.json"
    with path.open() as fh:
        return json.load(fh)


def assert_close(actual: float, expected: float, label: str) -> None:
    if round(float(actual), 4) != round(float(expected), 4):
        raise AssertionError(f"{label}: expected {expected}, got {actual}")


def check_service(service: str, expected: dict) -> None:
    manifest = load_manifest(service)
    block = manifest.get("sustainability_emission_model")
    if not block:
        raise AssertionError(f"{service}: missing sustainability_emission_model")

    if block.get("pod_runtime_tier_ref") != "self.pod_runtime_tier":
        raise AssertionError(f"{service}: pod_runtime_tier_ref must reference self.pod_runtime_tier")

    power = block.get("power_model", {})
    observed_coefficients = (
        power.get("cpu_watts_per_vcpu_second"),
        power.get("memory_watts_per_gib_second"),
        power.get("storage_watts_per_gib_hour"),
        power.get("network_watts_per_gib"),
    )
    if observed_coefficients == ADR_EXAMPLE:
        raise AssertionError(f"{service}: copied ADR-0344 illustrative coefficients")
    if observed_coefficients != expected["coefficients"]:
        raise AssertionError(f"{service}: coefficient tuple mismatch: {observed_coefficients}")
    if power.get("tier_overhead_factor") != TIER_OVERHEAD:
        raise AssertionError(f"{service}: tier_overhead_factor must retain ADR-0344 defaults")

    price = block.get("price_model", {})
    if price.get("source") != "provider_sku_pricing" or price.get("pin_window_hours") != 1:
        raise AssertionError(f"{service}: price model source/window mismatch")
    expected_binding = f"oya/{service}/IPs/IP-sustainability-emission-model.md#provider-sku-price-binding-plan-only"
    if price.get("binding") != expected_binding:
        raise AssertionError(f"{service}: price binding must be service-local IP reference")

    signal = block.get("workload_signal_source", {})
    if signal.get("signal_family") != expected["signal_family"]:
        raise AssertionError(f"{service}: signal family mismatch: {signal}")
    if signal.get("source_ref") != f"oya/{service}/IPs/IP-sustainability-emission-model.md#workload-signal-source":
        raise AssertionError(f"{service}: signal source_ref mismatch")

    tests = block.get("emission_path_tests")
    required_tests = [
        f"scripts/tests/finops_collab_emission_models_check.py::{service}",
        f"oya/{service}/IPs/IP-sustainability-emission-model.md#red-fixture-contract",
    ]
    if tests != required_tests:
        raise AssertionError(f"{service}: emission_path_tests mismatch: {tests}")

    baseline = block.get("validation_baseline", {})
    assert_close(baseline.get("expected_watt_hours_per_request_p50_mwh"), expected["mwh"], f"{service} mWh")
    assert_close(baseline.get("expected_co2_grams_per_request_p50_at_grid_400gco2_per_kwh"), expected["co2"], f"{service} CO2")
    if baseline.get("tolerance_pct") != 20:
        raise AssertionError(f"{service}: tolerance_pct must remain advisory default 20")

    ips = manifest.get("ips", [])
    ip_entry = next((ip for ip in ips if ip.get("id") == "IP-sustainability-emission-model"), None)
    if not ip_entry:
        raise AssertionError(f"{service}: manifest ips lacks IP-sustainability-emission-model")
    if ip_entry.get("acceptance_status") != "plan-spec-only":
        raise AssertionError(f"{service}: IP acceptance status must preserve non-claim boundary")
    if ip_entry.get("file") != f"oya/{service}/IPs/IP-sustainability-emission-model.md":
        raise AssertionError(f"{service}: IP file path mismatch")

    ip_path = ROOT / "oya" / service / "IPs" / "IP-sustainability-emission-model.md"
    if not ip_path.exists():
        raise AssertionError(f"{service}: missing {ip_path.relative_to(ROOT)}")
    content = ip_path.read_text()
    for phrase in [
        "Plan/Spec-only",
        "No runtime instrumentation",
        "No billing mutation",
        "provider_sku_pricing",
        expected["signal_family"],
        *expected["must_mention"],
    ]:
        if phrase not in content:
            raise AssertionError(f"{service}: IP missing phrase {phrase!r}")


def main() -> None:
    seen_coefficients = set()
    for service, expected in EXPECTED.items():
        check_service(service, expected)
        coefficients = expected["coefficients"]
        if coefficients in seen_coefficients:
            raise AssertionError(f"{service}: duplicate coefficient tuple")
        seen_coefficients.add(coefficients)
    print(f"validated {len(EXPECTED)} collaboration sustainability emission model declarations")


if __name__ == "__main__":
    main()
