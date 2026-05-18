---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-meet-foundation
impl_plan_id: IP-001-iac-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-meet + ops-sre-reliability
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: IaC bootstrap (Helm + Kustomize + OpenTofu)

## Intent

Author the meet µservice's deployment substrate: Helm chart for the core workloads (meet-rest, meeting-instance-rest, participant-worker, recording-worker, transcription-worker, webinar-rest, live-stream-egress-worker, e2e-encryption-worker), upstream-dependency charts (LiveKit 1.6.2, coturn 0.2.0, Whisper.cpp 1.7 + faster-whisper, SRS 6.0, Meilisearch 0.10.0), Kustomize base + per-pack overlays (kr, eu, us, us-hc, us-financial), and Terraform-managed Grafana RBAC. Versions pinned per LTS policy. ffmpeg recording workers run under gVisor `runtimeClassName`. Whisper transcription workers carry GPU node selector. LiveKit runs as StatefulSet for room affinity.

## ChangeSet boundary

One cohesive ChangeSet: 1 Helm chart bundle (meet) + 1 shared Kustomize base + 11 per-pack Kustomize overlays + 1 OpenTofu module for Grafana RBAC. No code; pure IaC + values. Per-pack secret references via OpenBao.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/meet/iac/helm/meet/Chart.yaml` | create | upstream deps livekit-server 1.6.2, coturn 0.2.0, meilisearch 0.10.0, srs 6.0 |
| `microservices/meet/iac/helm/meet/values.yaml` | create | Per-BC replica sizing, OpenBao SecretReferences, GPU node selector for Whisper, gVisor runtimeClassName for ffmpeg |
| `microservices/meet/iac/helm/meet/templates/{deployment,statefulset,service,hpa,pdb,networkpolicy,servicemonitor,prometheusrule}.yaml` | create | core Kubernetes resources |
| `microservices/meet/iac/kustomize/base/kustomization.yaml` | create | shared base |
| `microservices/meet/iac/kustomize/overlays/pack-{kr,eu}/kustomization.yaml` | create | 2 packs at slice; remaining 9 packs follow |
| `microservices/meet/iac/terraform/grafana-rbac.tf` | create | Terraform-managed Grafana folder + roles |

## Crate Naming

n/a — IaC only.

## Acceptance Gates

```bash
helm lint microservices/meet/iac/helm/meet
kubectl --dry-run=client apply -k microservices/meet/iac/kustomize/overlays/pack-kr
kubectl --dry-run=client apply -k microservices/meet/iac/kustomize/overlays/pack-eu
terraform -chdir=microservices/meet/iac/tofu validate
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice meet
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
```

## Test Plan

- Per Phase-01 IaC class: `helm install --dry-run` + `helm test` per chart.
- E2E: kind cluster; pack-kr overlay; all components Ready within 15min.
- LiveKit smoke: spin SFU StatefulSet; SDP offer/answer; ICE candidate selection works.
- coturn smoke: STUN binding request + TURN allocation request.
- ffmpeg gVisor smoke: pod boots with `runtimeClassName: gvisor`; sandbox active.
- Whisper GPU smoke: pod schedules onto GPU node; CUDA visible; model loads.

## Halt Conditions

- Any chart upstream-version drift from LTS pin — escalate to standards.
- OpenBao secret-ref resolution failure — block; engage cloud-secrets µservice.
- LiveKit / coturn / Whisper / ffmpeg upstream CVE — block; sunset to next pinned LTS.
- gVisor not installed in target cluster — block; engage cloud-k8s.

## Next IP

[`IP-002-cargo-workspace-bootstrap.md`](IP-002-cargo-workspace-bootstrap.md)

## References

- ADR-0139; ADR-0131; ADR-0132; ADR-MEET-0001; ADR-MEET-0002.
- `microservices/meet/multi-region.md`.
- `microservices/meet/capacity-model.md`.
- LiveKit OSS docs `docs.livekit.io`.
- coturn upstream `github.com/coturn/coturn`.
- Whisper.cpp `github.com/ggerganov/whisper.cpp`.
- faster-whisper `github.com/SYSTRAN/faster-whisper`.
- Meilisearch ops `docs.meilisearch.com`.
- SRS RTMP `github.com/ossrs/srs`.
- gVisor `gvisor.dev`.
