---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-shared-substrate
phase: P02-anonymous-foundation
impl_plan_id: IP-001-iac-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-anonymous + ops-platform
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-iac-conformance, oya-governance-version-pinning-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: IaC Bootstrap (Helm + Kustomize + per-pack overlays)

## Intent

Author Helm chart + Kustomize base + pack-kr + pack-eu overlays for the `anonymous` µservice. No code; pure IaC. Establishes the deployment substrate before any Rust crate is authored.

## ChangeSet boundary

| Path | Action | Description |
|---|---|---|
| `microservices/anonymous/iac/helm/Chart.yaml` | create | Helm chart manifest; appVersion pinned to 1.0.0 |
| `microservices/anonymous/iac/helm/values.yaml` | create | default values; LTS pins (Postgres 16, Redis 7.2, Meilisearch 0.10.0, rust-bls 0.5, Cedar v4.2) |
| `microservices/anonymous/iac/helm/templates/{deployment,service,hpa,pdb,networkpolicy,servicemonitor,prometheusrule}.yaml` | create | 7 templates; EGRESS-DENY-DEFAULT NetworkPolicy enforces I4 |
| `microservices/anonymous/iac/kustomize/base/{kustomization,namespace}.yaml` | create | shared base |
| `microservices/anonymous/iac/kustomize/overlays/pack-kr/*` | create | pack-kr overlay (Seoul region; KR PIPA Art. 24-2 toggle) |
| `microservices/anonymous/iac/kustomize/overlays/pack-eu/*` | create | pack-eu overlay (Frankfurt+Dublin; EU DSA + EU AI Act Art. 50 toggle) |

## Crate Naming

n/a — IaC only.

## Acceptance criteria

- `helm template microservices/anonymous/iac/helm/` exits 0
- `kustomize build microservices/anonymous/iac/kustomize/overlays/pack-kr/` exits 0
- `kustomize build microservices/anonymous/iac/kustomize/overlays/pack-eu/` exits 0
- LTS version pins match `docs/standards/version-pinning.md`
- NetworkPolicy explicitly denies egress to Google Analytics, Segment, Mixpanel, etc. (I4)
- No secret literals in any manifest; all references via `${openbao:secret/<path>}`

## Risks + Mitigations

| Risk | Mitigation |
|---|---|
| Pack overlay drift from base | per-pack overlay lint at CI |
| Missing FIPS-validated container image | image registry must publish FIPS-validated images (post-IP-007) |
