---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-notes-foundation
impl_plan_id: IP-001-iac
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-notes + ops-sre-reliability
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: IaC bootstrap (Helm + Kustomize + Terraform)

## Intent

Author the notes µservice's deployment substrate: Helm chart for the core workloads (note-store REST + workers; tag-graph + backlink-graph + checklist + share-link + web-clipper-bridge + collab-edit-broker; import + export workers; ai-assist worker; e2e-key-management) plus upstream-dependency charts (Postgres 16 LTS, Redis 7.2, Meilisearch 0.10.0, Loro broker container), Kustomize base + 11 per-pack overlays, and Terraform-managed Grafana RBAC.

## ChangeSet boundary

One cohesive ChangeSet: 1 Helm chart bundle (notes) + 1 shared Kustomize base + 11 per-pack Kustomize overlays + 1 Terraform module for Grafana RBAC. No code; pure IaC + values.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/notes/iac/helm/notes/Chart.yaml` | exists |
| `microservices/notes/iac/helm/notes/values.yaml` | exists |
| `microservices/notes/iac/helm/notes/templates/{deployment,service,hpa,pdb,networkpolicy,servicemonitor,prometheusrule}.yaml` | exists |
| `microservices/notes/iac/kustomize/base/kustomization.yaml` | exists |
| `microservices/notes/iac/kustomize/overlays/pack-{kr,eu,us,us-healthcare,jp,sg,au,in,br,ae,ksa}/kustomization.yaml` | per-pack |
| `microservices/notes/iac/terraform/grafana-rbac.tf` | exists |

## Acceptance Gates

```bash
helm lint microservices/notes/iac/helm/notes
kubectl --dry-run=client apply -k microservices/notes/iac/kustomize/overlays/pack-kr
kubectl --dry-run=client apply -k microservices/notes/iac/kustomize/overlays/pack-us-healthcare
terraform -chdir=microservices/notes/iac/terraform validate
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice notes
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
```

## Test Plan

- helm install --dry-run + helm test per chart.
- E2E: kind cluster; pack-kr overlay; all components Ready within 10 min.
- Postgres + Redis + Meilisearch + Loro broker smoke tests.

## Halt Conditions

- Any chart upstream-version drift from LTS pin — escalate to standards.
- OpenBao secret-ref resolution failure — block.
- Loro broker CVE — block; sunset to next pinned LTS.

## Next IP

[`IP-002-cargo-workspace-bootstrap.md`](IP-002-cargo-workspace-bootstrap.md)

## References

- ADR-0130; ADR-0131; ADR-0132.
- `microservices/notes/multi-region.md`.
- `microservices/notes/capacity-model.md`.
