#!/usr/bin/env bash
set -euo pipefail

# Oya VCS admission gate for Foundry agentic pipeline cutover.
# This is intentionally narrower than scripts/check.sh: it proves the VCS
# replacement core, authority cutover metadata, multispectrum wiring, and CI
# branch-protection visibility without requiring a full-workspace run.

VCS_PACKAGES=(
  oya-foundry-vcs-kernel
  oya-foundry-vcs-ast-index-kernel
  oya-foundry-vcs-lockstore-adapter
  oya-foundry-vcs-changebundle-kernel
  oya-foundry-vcs-polyglot-indexer-adapter
  oya-foundry-vcs-test-standard-gate-kernel
  oya-foundry-vcs-promotion-controller-kernel
  oya-foundry-vcs-review-mergequeue-kernel
  oya-foundry-vcs-cli-ratchet-kernel
)

python3 - <<'PY'
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path.cwd()


def fail(message: str) -> None:
    print(f"check-oya-vcs-admission: {message}", file=sys.stderr)
    raise SystemExit(1)


def read_json(path: str):
    try:
        return json.loads((ROOT / path).read_text())
    except Exception as exc:
        fail(f"could not parse {path}: {exc}")

root = read_json("specs/cross-cutting/root-hub-pointers.json")
quick = root.get("agent_quick_start_protocol", {})
quick_text = json.dumps(quick, sort_keys=True)
if "Oya VCS" not in quick_text or "claim" not in quick_text or "promote" not in quick_text:
    fail("root quick-start must make Oya VCS claim/work/verify/done/promote discoverable")
if "step_3_grit_state_transition" in quick:
    fail("root quick-start still exposes grit as the primary step_3 state transition")

sequencing = read_json("specs/cross-cutting/master-plan-sequencing.json")
sequence = sequencing.get("sequence", [])
if "claim_with_oya_vcs" not in sequence or "oya_vcs_promote_or_record_blocker" not in sequence:
    fail("master-plan sequence must route claim and promotion through Oya VCS")
legacy_sequence = {"claim_with_grit_or_scaffold_lock", "close_claim_with_done_bundle", "store_icm_completion_summary"}
if legacy_sequence.intersection(sequence):
    fail(f"master-plan sequence still contains legacy authority steps: {sorted(legacy_sequence.intersection(sequence))}")

must_have = set(sequencing.get("implementation_plan_changeset_contract", {}).get("must_have", []))
if "oya_vcs_claim_scope" not in must_have or "vcs_completion_payload" not in must_have:
    fail("ImplementationPlan contract must require Oya VCS claim scope and VCS completion payload")
if "grit_claim_or_scaffold_lock_scope" in must_have or "icm_completion_payload" in must_have:
    fail("ImplementationPlan contract still requires legacy grit/icm fields")

multispectrum = read_json("specs/cross-cutting/multispectrum-review.json")
agentic = multispectrum.get("enforcement_scopes", {}).get("agentic_flow", {})
agentic_text = json.dumps(agentic, sort_keys=True)
if "Oya VCS" not in agentic_text:
    fail("multispectrum agentic_flow must name Oya VCS as the enforcing state machine")
if "grit done" in agentic_text:
    fail("multispectrum agentic_flow still treats grit done as promotion authority")

vcs = read_json("specs/cross-cutting/gitops-vcs-replacement.json")
admission_ids = {gate.get("id") for gate in vcs.get("gitops_pipeline_integration", {}).get("admission_gates", [])}
for required in {
    "claim-coverage",
    "policy-and-tests",
    "multispectrum-evidence",
    "controller-owned-rebase",
    "merge-queue-ownership",
    "cli-command-surface",
    "provider-evidence-slots",
    "audit-chain-coverage",
}:
    if required not in admission_ids:
        fail(f"Oya VCS admission gate missing {required}")
plan = vcs.get("foundry_agentic_pipeline_integration_plan", {})
if plan.get("closure_authority") != "Oya VCS ChangeBundle -> Promotion -> ReleaseTrain":
    fail("Foundry integration plan must declare Oya VCS closure authority")

current_lane = vcs.get("current_ci_admission_lane", {})
smoke = " ".join(current_lane.get("command_surface_smoke", []))
for required in ("claim", "verify", "done", "promote"):
    if f"oya vcs --format json {required}" not in smoke:
        fail(f"current CI lane must smoke-test oya vcs {required}")
provider_required = set(current_lane.get("provider_evidence_required", []))
expected_providers = {"ci", "github-actions", "trivy", "argo-gitops"}
if provider_required != expected_providers:
    fail(f"current CI lane provider evidence must be {sorted(expected_providers)}")
provider_ref = current_lane.get("provider_evidence_ref", "")
provider_path = provider_ref.split("#", 1)[0]
if not provider_path:
    fail("current CI lane must point at provider evidence")
provider_doc = read_json(provider_path)
provider = provider_doc.get("provider_evidence", {})
slots = provider.get("slots", [])
slot_by_id = {slot.get("id"): slot for slot in slots}
for required in expected_providers:
    slot = slot_by_id.get(required)
    if not slot:
        fail(f"provider evidence missing slot {required}")
    if slot.get("provider_kind") != required:
        fail(f"provider evidence slot {required} has wrong provider kind")
    if slot.get("availability") != "available" or slot.get("decision") != "passed":
        fail(f"provider evidence slot {required} must be available/passed")
    proof_kind = slot.get("proof_kind", "")
    evidence_ref = slot.get("evidence_ref", "")
    live_status = slot.get("live_status", "")
    if not evidence_ref or not proof_kind:
        fail(f"provider evidence slot {required} must name evidence_ref and proof_kind")
    if required in {"trivy", "argo-gitops"} and "fixture" in proof_kind.lower():
        fail(f"provider evidence slot {required} regressed to fixture-only proof: {proof_kind}")
    if "remote-run-not-required" in live_status:
        fail(f"provider evidence slot {required} still says remote-run-not-required")

proof_ref = current_lane.get("provider_execution_proof_ref", "")
proof_path = proof_ref.split("#", 1)[0]
if not proof_path:
    fail("current CI lane must point at provider execution proof")
provider_execution = read_json(proof_path)
proof_slots = provider_execution.get("provider_slots", [])
proof_by_id = {slot.get("id"): slot for slot in proof_slots}
for required in expected_providers:
    slot = proof_by_id.get(required)
    if not slot:
        fail(f"provider execution proof missing slot {required}")
    if slot.get("provider_kind") != required:
        fail(f"provider execution proof slot {required} has wrong provider kind")
    if slot.get("decision") != "passed":
        fail(f"provider execution proof slot {required} must be passed")
    if not slot.get("execution_mode"):
        fail(f"provider execution proof slot {required} must name execution_mode")
    if required in {"trivy", "argo-gitops"} and not slot.get("evidence_digest"):
        fail(f"provider execution proof slot {required} must carry evidence_digest")

branch_protection_text = (ROOT / ".github/branch-protection.yaml").read_text()
if "required_status_checks:" not in branch_protection_text or "- oya-vcs-admission" not in branch_protection_text:
    fail("branch protection must require oya-vcs-admission")
if "- oya-vcs-provider-execution" not in branch_protection_text:
    fail("branch protection must require oya-vcs-provider-execution")

workflow = (ROOT / ".github/workflows/pr-tests.yml").read_text()
if "oya-vcs-admission" not in workflow or "scripts/check-oya-vcs-admission.sh" not in workflow:
    fail("pr-tests workflow must expose the oya-vcs-admission job")
if "scripts/install-trivy-ci.sh" not in workflow:
    fail("pr-tests workflow must install Trivy before Oya VCS admission")

supply_chain = (ROOT / ".github/workflows/oya-foundry-fitness-supply-chain.yml").read_text()
if "oya-vcs-provider-execution" not in supply_chain or "scripts/check-oya-vcs-provider-execution.sh --mode ci" not in supply_chain:
    fail("oya-foundry-fitness-supply-chain workflow must expose the oya-vcs-provider-execution job")

metadata = subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"], text=True)
packages = {pkg["name"] for pkg in json.loads(metadata)["packages"]}
expected = {
    "oya-dev-cli",
    "oya-foundry-vcs-kernel",
    "oya-foundry-vcs-ast-index-kernel",
    "oya-foundry-vcs-lockstore-adapter",
    "oya-foundry-vcs-changebundle-kernel",
    "oya-foundry-vcs-polyglot-indexer-adapter",
    "oya-foundry-vcs-test-standard-gate-kernel",
    "oya-foundry-vcs-promotion-controller-kernel",
    "oya-foundry-vcs-review-mergequeue-kernel",
    "oya-foundry-vcs-cli-ratchet-kernel",
}
missing = sorted(expected - packages)
if missing:
    fail(f"workspace missing Oya VCS packages: {missing}")

audit_chain_text = (ROOT / "evidence/audit-chain.jsonl").read_text()
for path in sorted((ROOT / "evidence/multispectrum").glob("*.json")):
    evidence = json.loads(path.read_text())
    change_id = evidence.get("change_id")
    if not change_id:
        fail(f"multispectrum evidence {path} has no change_id")
    if change_id not in audit_chain_text:
        fail(f"multispectrum evidence {path} missing audit-chain coverage for {change_id}")

print("check-oya-vcs-admission: metadata and authority checks passed")
PY

scripts/check-oya-vcs-provider-execution.sh --mode check

cargo_args=()
for package in "${VCS_PACKAGES[@]}"; do
  cargo_args+=("-p" "${package}")
done

cargo test "${cargo_args[@]}"

cargo test -p oya-dev-cli vcs

cargo run -q -p oya-dev-cli -- vcs --format json claim \
  --agent admission-gate \
  --intent "Oya VCS admission CLI smoke" \
  specs/cross-cutting/gitops-vcs-replacement.json::foundry_agentic_pipeline_integration_plan \
  | python3 -m json.tool >/dev/null

cargo run -q -p oya-dev-cli -- vcs --format json verify \
  --agent admission-gate \
  --evidence evidence/gitops-vcs/oya-vcs-admission-cutover-2026-05-15.json \
  | python3 -m json.tool >/dev/null

cargo run -q -p oya-dev-cli -- vcs --format json done \
  --agent admission-gate \
  --evidence evidence/gitops-vcs/oya-vcs-admission-cutover-2026-05-15.json \
  | python3 -m json.tool >/dev/null

cargo run -q -p oya-dev-cli -- vcs --format json promote \
  --agent admission-gate \
  --bundle bundle_oya_vcs_admission_cutover \
  --environment ci-preview \
  --evidence evidence/gitops-vcs/oya-vcs-admission-cutover-2026-05-15.json \
  | python3 -m json.tool >/dev/null
