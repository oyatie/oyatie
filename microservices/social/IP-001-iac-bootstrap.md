---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-001-iac-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social + ops-sre-reliability
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: IaC bootstrap (Helm + Kustomize + OpenTofu)

## Intent

Author the social µservice's deployment substrate: Helm chart for the
core workloads (websocket-gateway, user-profile, follow-graph, post-composition
rest/worker, feed-timeline, reactions, mentions, hashtags, trending-topics,
notifications, content-moderation, search, profile-verification, age-verification,
federation-gateway (off in P01), bookmarks, lists, media-transcode worker
(sandboxed via gVisor)), upstream-dependency charts (Meilisearch),
Kustomize base + per-pack overlays, and Terraform-managed Grafana RBAC.
Versions pinned per LTS policy.

## ChangeSet boundary

One cohesive ChangeSet: 1 Helm chart bundle (social) + 1 shared Kustomize
base + 11 per-pack Kustomize overlays + 1 OpenTofu module for Grafana RBAC.
No code; pure IaC + values. Per-pack secret references via OpenBao.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/social/iac/helm/social/Chart.yaml` | exists | upstream dep meilisearch 0.10.0 LTS |
| `microservices/social/iac/helm/social/values.yaml` | exists | Per-BC replica sizing, OpenBao SecretReferences, LTS pins (Postgres 16, Valkey 8.1 (Redis wire-compat), Meilisearch 0.10, ClamAV 1.x, OPSWAT 5.x, ImageMagick 7.1, ffmpeg 7.x), Cedar v4.2 |
| `microservices/social/iac/helm/social/templates/{deployment,service,hpa,pdb,networkpolicy,servicemonitor,prometheusrule}.yaml` | exists | core Kubernetes resources |
| `microservices/social/iac/kustomize/base/kustomization.yaml` | exists | shared base |
| `microservices/social/iac/kustomize/overlays/pack-{kr,eu,us,us-healthcare,jp,sg,au,in,br,ae,ksa}/kustomization.yaml` | 2 done (kr, us-healthcare), 9 follow | per-pack overlay |
| `microservices/social/iac/terraform/grafana-rbac.tf` | Slice B | Terraform-managed Grafana folder + roles |

## Crate Naming

n/a — IaC only.

## Acceptance Gates

```bash
helm lint microservices/social/iac/helm/social
kubectl --dry-run=client apply -k microservices/social/iac/kustomize/overlays/pack-kr
kubectl --dry-run=client apply -k microservices/social/iac/kustomize/overlays/pack-us-healthcare
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice social
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
```

## Test Plan

- Per Phase-01 IaC class: `helm install --dry-run` + `helm test` per chart.
- E2E: kind cluster; pack-kr overlay; all components Ready within 10min.
- gVisor / Kata sandbox smoke for media-transcode worker.
- Meilisearch smoke: index + query roundtrip.

## Halt Conditions

- Any chart upstream-version drift from LTS pin — escalate to standards.
- OpenBao secret-ref resolution failure — block; engage cloud-secrets µservice.
- ImageMagick / ffmpeg CVE — block; sunset to next pinned LTS.

## Next IP

[`IP-002-cargo-workspace-bootstrap.md`](IP-002-cargo-workspace-bootstrap.md)

## References

- ADR-0139; ADR-0131; ADR-0132.
- `microservices/social/multi-region.md`.
- `microservices/social/capacity-model.md`.
- Meilisearch ops `docs.meilisearch.com`.
- gVisor runtime class docs.
