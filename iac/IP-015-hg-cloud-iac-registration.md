---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-meta-iac-pipeline-substrate
impl_plan_id: IP-015-hg-cloud-iac-registration
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-cloud-governance + axis-cloud-iac
acceptance_lanes: [oya-governance-authority-cohesion, oya-governance-hyperscaler-maturity-claims, oya-cloud-iac-iac-smoke]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: HG-CLOUD-IAC gate registration + bootstrap-paradox resolution

## Intent

Register HG-CLOUD-IAC in `/specs/hyperscaler-gates.json` per ADR-0123 hyperscaler-maturity-claims gate. Resolve the cloud-iac bootstrap paradox (cloud-iac applies its own IaC): minimum-viable substrate first bootstraps via cloud-k8s + cloud-secrets; cloud-iac then applies itself thereafter (parallel to observability self-observability per `microservices/observability/PRD.md` OQ#4).

## ChangeSet boundary

Updates `/specs/hyperscaler-gates.json` + adds a bootstrap-script under `iac/scripts/bootstrap.sh` + a self-apply integration test.

## Concrete File Targets

| Path | Action |
|---|---|
| `/specs/hyperscaler-gates.json` | update — add HG-CLOUD-IAC entry |
| `iac/scripts/bootstrap.sh` | create — initial substrate apply via cloud-k8s + cloud-secrets |
| `iac/tests/e2e/self_apply.rs` | create — verify cloud-iac can apply its own IaC after bootstrap |

## Code Shape

```json
// /specs/hyperscaler-gates.json (added entry)
{
  "id": "HG-CLOUD-IAC",
  "microservice": "cloud-iac",
  "competitors": ["argocd", "flux", "terraform-cloud", "opentofu", "spacelift", "atlantis", "env0", "pulumi-service", "crossplane"],
  "claim_boundary_doc": "iac/competitor-parity-matrix.md",
  "verification_lane": "oya-cloud-iac-iac-smoke",
  "registered_at": "2026-05-17",
  "status": "incubating"
}
```

```bash
#!/usr/bin/env bash
# iac/scripts/bootstrap.sh
# One-time bootstrap of cloud-iac substrate via cloud-k8s + cloud-secrets.
# After this completes successfully, cloud-iac can apply its own subsequent IaC.

set -euo pipefail

PACK="${1:-pack-kr}"

# 1. Verify prerequisites
kubectl --context "cloud-iac-control-plane-${PACK}" cluster-info
openbao status

# 2. Install cloud-k8s base substrate (namespaces, RBAC, network policies)
# This is owned by the cloud-k8s µservice's bootstrap; we assume it ran first.

# 3. Apply cloud-iac substrate via direct helm install (no cloud-iac iac-applier yet)
helm dep update iac/iac/helm/argocd
helm dep update iac/iac/helm/flux
helm dep update iac/iac/helm/opentofu
kubectl apply -k iac/iac/kustomize/overlays/${PACK}

# 4. Wait for ArgoCD + iac-applier-worker + iac-registry-worker pods Ready
kubectl wait --for=condition=Ready pod -l app.kubernetes.io/part-of=oya-cloud-iac --timeout=10m -n cloud-iac

# 5. Bootstrap iac-state-index Postgres schema
kubectl exec -n cloud-iac iac-state-index-pg-0 -- psql -U cloud_iac -f /migrations/0001_initial.sql
kubectl exec -n cloud-iac iac-state-index-pg-0 -- psql -U cloud_iac -f /migrations/0002_provenance.sql
kubectl exec -n cloud-iac iac-state-index-pg-0 -- psql -U cloud_iac -f /migrations/0003_append_only_trigger.sql

# 6. Register cloud-iac µservice in its own registry (now self-tracking)
cloud-native IaC controller/API `register --microservice` workflow

# 7. From this point onward, cloud-iac applies its own substrate via the normal apply path.
echo "Bootstrap complete. cloud-iac now self-managed in ${PACK}."
```

## Acceptance Gates

```bash
# Bootstrap drill on kind cluster
iac/scripts/bootstrap.sh pack-kr

# After bootstrap, verify self-apply works
cargo nextest run -p oya-cloud-iac-iac-applier-app --test self_apply

# HG-CLOUD-IAC gate registers green
cloud-ci/oya-ci governance gate `authority-cohesion` is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `hyperscaler-maturity-claims` is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `cloud-iac-iac-smoke` for --pack pack-kr is green in the branch-protected `oya-ci-required` context
```

## Test Plan

| Test | Verifies |
|---|---|
| `bootstrap_kind_pack_kr` | bootstrap.sh runs end-to-end on kind cluster |
| `self_apply_after_bootstrap` | cloud-iac applies its own next-version IaC successfully |
| `hg_cloud_iac_gate_registers` | HG-CLOUD-IAC appears in hyperscaler-gates.json |
| `competitor_parity_matrix_aligned` | matrix entries match feature-by-feature against competitors |

## Halt Conditions

- Bootstrap requires manual intervention beyond the script — fix.
- Self-apply step fails — fix root cause.
- HG-CLOUD-IAC gate sales-claim rules not enforced — fix.

## References

- ADR-0123 (hyperscaler maturity claim gate).
- ADR-0131.
- `iac/competitor-parity-matrix.md`.
- `microservices/observability/PRD.md` OQ#4 (self-observability bootstrap parallel).
- `/specs/hyperscaler-gates.json`.
