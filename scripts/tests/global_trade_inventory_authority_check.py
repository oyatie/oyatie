#!/usr/bin/env python3
"""Validate that oya/global-trade inventory stays metadata-only under the PRD authority."""
from __future__ import annotations

import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, NoReturn

REPO_ROOT = Path(__file__).resolve().parents[2]
PRD_PATH = REPO_ROOT / "specs" / "microservices" / "global-trade.json"
MANIFESTS_INDEX_PATH = REPO_ROOT / "specs" / "microservices" / "manifests-index.json"
GLOBAL_TRADE_ROOT = REPO_ROOT / "oya" / "global-trade"
DOMAIN_CRATE_SEGMENT = "crates/oya-global-trade-compliance-domain/"

AUTHORITY_MARKERS = (
    "specs/microservices/global-trade.json",
    "metadata-only prd",
    "inventory/provenance/planned-only",
)

TEXT_SUFFIXES = {
    ".cedar",
    ".hcl",
    ".json",
    ".md",
    ".proto",
    ".tf",
    ".yaml",
    ".yml",
}

CONTRACT_RUNTIME_PATTERNS = [
    (re.compile(r"(?im)^\s*post\s*:"), "OpenAPI mutating POST path"),
    (re.compile(r"(?i)accepted\s+with\s+signed\s+audit\s+event"), "signed audit-event response"),
    (re.compile(r"(?im)^\s*service\s+GlobalTradeService\s*\{"), "gRPC service surface"),
    (re.compile(r"(?im)^\s*rpc\s+Mutate"), "gRPC mutate RPC"),
    (re.compile(r"(?i)\.events\.v1"), "runtime event channel"),
]

IAC_RUNTIME_PATTERNS = [
    (re.compile(r"(?im)^\s*apiVersion\s*:\s*apps/v1"), "Kubernetes workload apiVersion"),
    (re.compile(r"(?im)^\s*kind\s*:\s*Deployment\b"), "Kubernetes Deployment"),
    (re.compile(r"(?im)^\s*replicas\s*:\s*\d+"), "Kubernetes replica declaration"),
    (re.compile(r"(?im)^\s*resource\s+\""), "Terraform/OpenTofu resource block"),
    (re.compile(r"(?i)registry\.example/oya-global-trade"), "container image runtime reference"),
]

SLO_DASHBOARD_RUNTIME_PATTERNS = [
    (re.compile(r"(?i)metricSource"), "measured metric source"),
    (re.compile(r"(?i)sum\s*\(\s*rate\s*\("), "Prometheus rate query"),
    (re.compile(r"(?im)^\s*target\s*:\s*0\.\d+"), "SLO target claim"),
    (re.compile(r"(?i)\"expr\"\s*:\s*\""), "dashboard query expression"),
]

RUNBOOK_RUNTIME_PATTERNS = [
    (re.compile(r"(?i)\bkubectl\b"), "kubectl runtime procedure"),
    (re.compile(r"(?i)\bargocd\b"), "ArgoCD runtime procedure"),
    (re.compile(r"(?i)\boya\s+(metrics|observability)\b"), "runtime observability CLI procedure"),
    (re.compile(r"(?i)alert\s+global-trade"), "live alert trigger"),
    (re.compile(r"(?i)worker\s+queue\s+drains"), "live worker-queue verification"),
]

ROOT_OVERCLAIM_PATTERNS = [
    (re.compile(r"(?i)SAP\s+GTS\s+parity\s+microservice"), "SAP GTS parity microservice claim"),
    (re.compile(r"(?i)HTTP/3\s+is\s+the\s+default\s+edge\s+transport"), "HTTP/3 default runtime posture"),
    (re.compile(r"(?i)\"status\"\s*:\s*\"reserved-wave"), "reserved-wave manifest status as authority"),
    (re.compile(r"(?i)deployment_shape"), "deployment shape claim"),
    (re.compile(r"(?i)pod_runtime_tier"), "pod runtime tier claim"),
]


@dataclass(frozen=True)
class Finding:
    path: Path
    summary: str


def fail(message: str) -> NoReturn:
    print(f"global-trade inventory authority check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def require(condition: object, message: str) -> None:
    if not condition:
        fail(message)


def rel(path: Path) -> str:
    return str(path.relative_to(REPO_ROOT))


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        fail(f"missing {rel(path)}")
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON in {rel(path)}: {exc}")


def is_scanned_file(path: Path) -> bool:
    if not path.is_file() or path.suffix not in TEXT_SUFFIXES:
        return False
    relative = rel(path)
    if DOMAIN_CRATE_SEGMENT in f"{relative}/":
        return False
    return relative.startswith("oya/global-trade/")


def has_authority_boundary(text: str) -> bool:
    lower = text.lower()
    return all(marker in lower for marker in AUTHORITY_MARKERS)


def has_early_authority_boundary(text: str) -> bool:
    early_text = "\n".join(text.splitlines()[:20])
    return has_authority_boundary(early_text)


def scan_patterns(path: Path, text: str, patterns: Iterable[tuple[re.Pattern[str], str]]) -> list[Finding]:
    findings: list[Finding] = []
    for pattern, label in patterns:
        if pattern.search(text):
            findings.append(Finding(path, label))
    return findings


def validate_authority_sources() -> None:
    prd = load_json(PRD_PATH)
    meta = prd.get("_meta", {})
    require(meta.get("status") == "preview", "global-trade PRD status must remain preview")
    require("metadata-only" in meta.get("purpose", "").lower(), "global-trade PRD purpose must remain metadata-only")
    non_goals = " ".join(str(item) for item in prd.get("non_goals", [])).lower()
    for marker in [
        "no live denied-party",
        "government list downloads",
        "customs-authority filing",
        "broker workflow",
        "document archive runtime",
        "workflow execution",
        "runtime audit-chain emission",
        "cloud deployment",
        "sap gts",
    ]:
        require(marker in non_goals, f"global-trade PRD non_goals must preserve {marker!r}")

    manifests_index = load_json(MANIFESTS_INDEX_PATH)
    names = {row.get("name") for row in manifests_index.get("microservices", []) if isinstance(row, dict)}
    require("global-trade" not in names, "manifests-index must not promote global-trade to manifest authority")


def validate_inventory_files() -> None:
    findings: list[Finding] = []
    scanned = 0
    for path in sorted(GLOBAL_TRADE_ROOT.rglob("*")):
        if not is_scanned_file(path):
            continue
        scanned += 1
        text = path.read_text(encoding="utf-8", errors="replace")
        relative = rel(path)
        if not has_authority_boundary(text):
            findings.append(Finding(path, "missing metadata-only PRD inventory/provenance/planned-only authority boundary"))
        elif not has_early_authority_boundary(text):
            findings.append(Finding(path, "authority boundary must appear within the first 20 lines before provenance content"))
        if "/contracts/" in relative:
            findings.extend(scan_patterns(path, text, CONTRACT_RUNTIME_PATTERNS))
        if "/iac/" in relative:
            findings.extend(scan_patterns(path, text, IAC_RUNTIME_PATTERNS))
        if "/slos/" in relative or "/dashboards/" in relative:
            findings.extend(scan_patterns(path, text, SLO_DASHBOARD_RUNTIME_PATTERNS))
        if "/runbooks/" in relative:
            findings.extend(scan_patterns(path, text, RUNBOOK_RUNTIME_PATTERNS))
        if relative in {"oya/global-trade/README.md", "oya/global-trade/manifest.json"}:
            findings.extend(scan_patterns(path, text, ROOT_OVERCLAIM_PATTERNS))

    require(scanned > 0, "no global-trade inventory files were scanned")
    if findings:
        preview = "\n".join(f"- {rel(f.path)}: {f.summary}" for f in findings[:40])
        overflow = "" if len(findings) <= 40 else f"\n... {len(findings) - 40} more finding(s)"
        fail(f"{len(findings)} inventory authority finding(s):\n{preview}{overflow}")
    print(f"global-trade inventory authority check passed: {scanned} metadata-only inventory file(s) scanned")


def run_self_tests() -> None:
    boundary = "specs/microservices/global-trade.json metadata-only PRD inventory/provenance/planned-only"
    require(has_authority_boundary(boundary), "authority marker self-test should pass")
    require(not has_authority_boundary("runtime implementation"), "missing marker self-test should fail")
    late_boundary = "\n".join(["runtime-looking provenance content"] * 21 + [boundary])
    require(has_authority_boundary(late_boundary), "late authority boundary self-test should preserve broad marker detection")
    require(not has_early_authority_boundary(late_boundary), "late authority boundary self-test should fail early-boundary rule")
    contract_findings = scan_patterns(Path("contracts/openapi-v1.yaml"), f"{boundary}\npaths:\n  /x:\n    post:\n      responses:\n        '202':\n          description: Accepted with signed audit event\n", CONTRACT_RUNTIME_PATTERNS)
    require(len(contract_findings) >= 2, "contract overclaim self-test should reject mutating signed-audit surface")
    iac_findings = scan_patterns(Path("iac/k8s-deployment.yaml"), f"{boundary}\napiVersion: apps/v1\nkind: Deployment\nreplicas: 3\n", IAC_RUNTIME_PATTERNS)
    require(len(iac_findings) >= 2, "IAC overclaim self-test should reject deployment surface")
    slo_findings = scan_patterns(Path("slos/service.openslo.yaml"), f"{boundary}\ntarget: 0.999\nquery: sum(rate(x[5m]))\n", SLO_DASHBOARD_RUNTIME_PATTERNS)
    require(len(slo_findings) >= 2, "SLO overclaim self-test should reject measured SLO surface")
    print("global-trade inventory authority self-tests passed")


def main() -> None:
    if "--self-test" in sys.argv[1:]:
        run_self_tests()
    validate_authority_sources()
    validate_inventory_files()


if __name__ == "__main__":
    main()
