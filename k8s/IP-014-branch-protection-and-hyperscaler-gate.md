---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-onprem-k8s-substrate
impl_plan_id: IP-014-branch-protection-and-hyperscaler-gate
status: pending
execution_unit: ChangeSet
owner: axis-foundry + axis-cloud
acceptance_lanes: [yaml-lint, branch-protection-conformance, governance-authority-cohesion, governance-hyperscaler-maturity-claims]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014: branch-protection + HG-CLOUD-K8S registration

## Intent

Add cloud-k8s-specific BLOCKER lanes to `.github/branch-protection.yaml` (`cloud-k8s-iac-smoke`, `check-cis-k8s-benchmark`, `check-cosign-admission`, `check-etcd-encryption`, `check-istio-strict-mtls`) and register `HG-CLOUD-K8S` in `/specs/hyperscaler-gates.json` per ADR-0123.

## ChangeSet boundary

Cross-cutting changes:
- `.github/branch-protection.yaml` updated
- `/specs/hyperscaler-gates.json` registers HG-CLOUD-K8S
- `/specs/cloud-k8s-cluster-state.json` (NEW) — cluster state machine
- `microservices/cloud-k8s/iac/lanes/` — lane definitions

## Concrete File Targets

| Path | Action |
|---|---|
| `.github/branch-protection.yaml` | update — add 5 BLOCKER lanes on dev + staging |
| `/specs/hyperscaler-gates.json` | update — register HG-CLOUD-K8S with claims-bound criteria |
| `/specs/cloud-k8s-cluster-state.json` | create — cluster state machine spec |
| `microservices/cloud-k8s/iac/lanes/{cis-k8s-benchmark,cosign-admission,etcd-encryption,istio-strict-mtls,iac-smoke}.yaml` | create — per-lane definitions |

## Code Shape

`.github/branch-protection.yaml` diff:

```yaml
branches:
  dev:
    required_status_checks:
      # ADDED by IP-014:
      - cloud-k8s-iac-smoke
      - check-cis-k8s-benchmark
      - check-cosign-admission
      - check-etcd-encryption
      - check-istio-strict-mtls
  staging:
    required_status_checks:
      # ADDED:
      - cloud-k8s-iac-smoke
      - check-cis-k8s-benchmark
      - check-cosign-admission
      - check-etcd-encryption
      - check-istio-strict-mtls
```

`/specs/hyperscaler-gates.json` excerpt:

```json
{
  "gates": [
    {
      "id": "HG-CLOUD-K8S",
      "owner_team": "axis-cloud",
      "registered_artifact": "microservices/cloud-k8s/PRD.md",
      "claim_boundary_doc": "microservices/cloud-k8s/competitor-parity-matrix.md",
      "criteria": [
        {"id": "HG-CLOUD-K8S-01", "name": "vanilla-upstream-kubernetes", "verifier": "cargo run -p dev-cli -- gate validate version-pinning-conformance"},
        {"id": "HG-CLOUD-K8S-02", "name": "cis-k8s-benchmark-v1.9", "verifier": "check-cis-k8s-benchmark"},
        {"id": "HG-CLOUD-K8S-03", "name": "nsa-cisa-k8s-hardening-v1.2", "verifier": "check-nsa-k8s-hardening"},
        {"id": "HG-CLOUD-K8S-04", "name": "istio-strict-mtls-mesh-wide", "verifier": "check-istio-strict-mtls"},
        {"id": "HG-CLOUD-K8S-05", "name": "etcd-kms-envelope-encryption", "verifier": "check-etcd-encryption"},
        {"id": "HG-CLOUD-K8S-06", "name": "cosign-admission-enforced", "verifier": "check-cosign-admission"},
        {"id": "HG-CLOUD-K8S-07", "name": "kubernetes-api-proxy-only-path", "verifier": "check-kubernetes-api-proxy-only-path"},
        {"id": "HG-CLOUD-K8S-08", "name": "per-pack-cluster-boundary", "verifier": "check-data-residency-conformance"},
        {"id": "HG-CLOUD-K8S-09", "name": "cluster-bootstrap-30min-p99", "verifier": "e2e drill"},
        {"id": "HG-CLOUD-K8S-10", "name": "node-join-5min-p99", "verifier": "e2e drill"}
      ]
    }
  ]
}
```

## Acceptance Gates

```bash
yamllint .github/branch-protection.yaml
jq -r '.gates[] | select(.id == "HG-CLOUD-K8S")' /specs/hyperscaler-gates.json
cargo run -p dev-cli -- gate validate authority-cohesion
cargo run -p dev-cli -- gate validate hyperscaler-maturity-claims
cargo run -p dev-cli -- gate validate branch-protection-conformance
```

## Test Plan

- branch-protection.yaml: `gh api repos/<org>/<repo>/branches/dev/protection` reflects the new lanes
- HG-CLOUD-K8S registers green: `cargo run -p dev-cli -- gate list --owner axis-cloud` shows HG-CLOUD-K8S
- All 10 HG criteria have verifier exit-0 capability

## Halt Conditions

- Any HG criterion's verifier not implementable — re-scope
- Any lane added without paired runbook — refuse

## Next IP

[`IP-015-observability-slo-and-authority-cohesion.md`](IP-015-observability-slo-and-authority-cohesion.md)

## References

- ADR-0123 (HG gate); ADR-0121.
- `.github/branch-protection.yaml`; `/specs/hyperscaler-gates.json`.
