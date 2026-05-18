---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-foundation
phase: P01-shorts-foundation
impl_plan_id: IP-001-iac-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-shorts + ops-sre-reliability + cloud-secrets
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: IaC bootstrap (Helm + Kustomize + Terraform)

## Intent

Author the shorts µservice's deployment substrate: Helm chart for the core
workloads (websocket-gateway, video-upload-rest, video-transcode-worker
sandboxed via gVisor, video-storage-rest, thumbnail-generation-worker,
audio-track-library-rest, audio-attribution-worker, video-composition-worker,
feed-timeline, watch-time-tracking-worker, like-share-comment-worker,
repost-stitch-duet-worker, hashtag-worker, trending-worker,
notifications-worker, content-moderation-worker, copyright-claim-worker
(Chromaprint + DCT sandboxed), accessibility-captions-worker,
creator-analytics-worker, age-gate-rest, parental-controls-rest,
drm-key-server (Widevine+FairPlay+PlayReady; Premium-tier gated),
federation-gateway (off in P01)), upstream-dependency charts (Meilisearch),
Kustomize base + per-pack overlays, and Terraform-managed Grafana RBAC.
Versions pinned per LTS policy.

## ChangeSet boundary

One cohesive ChangeSet: 1 Helm chart bundle (shorts) + 1 shared Kustomize
base + 11 per-pack Kustomize overlays + 1 Terraform module for Grafana RBAC.
No code; pure IaC + values. Per-pack secret references via OpenBao.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/shorts/iac/helm/shorts/Chart.yaml` | exists | upstream dep meilisearch 0.10.0 LTS |
| `microservices/shorts/iac/helm/shorts/values.yaml` | exists | Per-BC replica sizing, OpenBao SecretReferences, LTS pins (Postgres 16, Redis 7.2, Meilisearch 0.10, ClamAV 1.x, OPSWAT 5.x, ffmpeg 7.x, Chromaprint 1.5.1, Cedar v4.2), DRM tier gating |
| `microservices/shorts/iac/helm/shorts/templates/{deployment,service,hpa,pdb,networkpolicy,servicemonitor,prometheusrule}.yaml` | exists | core Kubernetes resources |
| `microservices/shorts/iac/kustomize/base/kustomization.yaml` | exists | shared base |
| `microservices/shorts/iac/kustomize/overlays/pack-{kr,eu,us,us-healthcare,jp,sg,au,in,br,ae,ksa}/kustomization.yaml` | 2 done (kr, eu), 9 follow | per-pack overlay |
| `microservices/shorts/iac/terraform/grafana-rbac.tf` | Slice B | Terraform-managed Grafana folder + roles |

## Crate Naming

n/a — IaC only.

## Acceptance Gates

```bash
helm lint microservices/shorts/iac/helm/shorts
kubectl --dry-run=client apply -k microservices/shorts/iac/kustomize/overlays/pack-kr
kubectl --dry-run=client apply -k microservices/shorts/iac/kustomize/overlays/pack-eu
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice shorts
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
```

## Test Plan

- Per Phase-01 IaC class: `helm install --dry-run` + `helm test` per chart.
- E2E: kind cluster; pack-kr overlay; all components Ready within 10min.
- gVisor sandbox smoke for video-transcode + copyright-claim + content-moderation workers.
- Meilisearch smoke: index + query roundtrip.
- DRM key-server smoke: Widevine + FairPlay + PlayReady provisioning challenge roundtrip in sandbox.

## Halt Conditions

- Any chart upstream-version drift from LTS pin — escalate to standards.
- OpenBao secret-ref resolution failure — block; engage cloud-secrets µservice.
- ffmpeg / Chromaprint / Cedar / DRM CVE — block; sunset to next pinned LTS.

## Next IP

[`IP-002-cargo-workspace-bootstrap.md`](IP-002-cargo-workspace-bootstrap.md)

## References

- ADR-0130; ADR-0131; ADR-0132; ADR-SHORTS-0001; ADR-SHORTS-0004.
- `microservices/shorts/multi-region.md`.
- `microservices/shorts/capacity-model.md`.
- Meilisearch ops `docs.meilisearch.com`.
- gVisor / Kata Container runtime class docs.
- Cloudflare R2 + Workers docs.
- Widevine SecureStop / FairPlay key-server / PlayReady DRM-server vendor docs.
