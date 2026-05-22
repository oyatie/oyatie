---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-team-channels-dm-threads
impl_plan_id: IP-001-iac-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-messenger + ops-sre-reliability
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: IaC bootstrap (Helm + Kustomize + OpenTofu)

## Intent

Author the messenger µservice's deployment substrate: Helm chart for the
core workloads (websocket-gateway, channel-store, message-stream rest/worker,
presence, file-attachment-worker, mention-router), upstream-dependency charts
(LiveKit + coturn + Meilisearch), Kustomize base + per-pack overlays, and
Terraform-managed Grafana RBAC. Versions pinned per LTS policy.

## ChangeSet boundary

One cohesive ChangeSet: 1 Helm chart bundle (messenger) + 1 shared Kustomize
base + 11 per-pack Kustomize overlays + 1 OpenTofu module for Grafana RBAC.
No code; pure IaC + values. Per-pack secret references via OpenBao.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/messenger/iac/helm/messenger/Chart.yaml` | exists | upstream deps livekit-server 1.6.2, coturn 0.2.0, meilisearch 0.10.0 |
| `microservices/messenger/iac/helm/messenger/values.yaml` | exists | Per-BC replica sizing, OpenBao SecretReferences |
| `microservices/messenger/iac/helm/messenger/templates/{deployment,service,hpa,pdb,networkpolicy,servicemonitor,prometheusrule}.yaml` | exists | core Kubernetes resources |
| `microservices/messenger/iac/kustomize/base/kustomization.yaml` | exists | shared base |
| `microservices/messenger/iac/kustomize/overlays/pack-{kr,eu,us,us-healthcare,jp,sg,au,in,br,ae,ksa}/kustomization.yaml` | 2 done, 9 follow | per-pack overlay |
| `microservices/messenger/iac/terraform/grafana-rbac.tf` | exists | Terraform-managed Grafana folder + roles |

## Crate Naming

n/a — IaC only.

## Acceptance Gates

```bash
helm lint microservices/messenger/iac/helm/messenger
kubectl --dry-run=client apply -k microservices/messenger/iac/kustomize/overlays/pack-kr
kubectl --dry-run=client apply -k microservices/messenger/iac/kustomize/overlays/pack-us-healthcare
terraform -chdir=microservices/messenger/iac/tofu validate
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice messenger
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
```

## Test Plan

- Per Phase-01 IaC class: `helm install --dry-run` + `helm test` per chart.
- E2E: kind cluster; pack-kr overlay; all components Ready within 10min.
- LiveKit smoke: spin SFU; SDP offer/answer; ICE candidate selection works.
- coturn smoke: STUN binding request + TURN allocation request.

## Halt Conditions

- Any chart upstream-version drift from LTS pin — escalate to standards.
- OpenBao secret-ref resolution failure — block; engage cloud-secrets µservice.
- LiveKit / coturn upstream CVE — block; sunset to next pinned LTS.

## Next IP

[`IP-002-cargo-workspace-bootstrap.md`](IP-002-cargo-workspace-bootstrap.md)

## References

- ADR-0139; ADR-0131; ADR-0132.
- `microservices/messenger/multi-region.md`.
- `microservices/messenger/capacity-model.md`.
- LiveKit OSS docs `docs.livekit.io`.
- coturn upstream `github.com/coturn/coturn`.
- Meilisearch ops `docs.meilisearch.com`.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/messenger/IP-001-iac-bootstrap.md` matched `multi-region`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), KR-PIPA-2023-amendment(14400s/900s), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/messenger/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/messenger/slos/attachment-scan-freshness.openslo.yaml`, `microservices/messenger/slos/mention-fanout.openslo.yaml`, `microservices/messenger/slos/message-send-availability.openslo.yaml`, `microservices/messenger/slos/message-send-latency.openslo.yaml`, `microservices/messenger/policy/auditor-scope.cedar`.
