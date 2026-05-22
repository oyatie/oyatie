---
doc_class: Implementation-Plan
ip_id: IP-journey-j100-pack-rollout-first-action
journey_ref: docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/
microservice: cloud-iac
status: draft
date: 2026-05-21
---

# IP-journey-j100: Pack rollout first infrastructure action

## A. Problem
J100 measures whether a new tenant can move from onboarding to first useful action. cloud-iac owns the infrastructure side of that first action: render, validate, apply, registry readiness, drift watch, and rollback preparedness.

## B. Approach
Use the existing cloud-iac contract surfaces and SLOs as the first-action gate. `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml` carries the rollout request, `microservices/cloud-iac/slos/iac-render-latency.openslo.yaml` and `microservices/cloud-iac/slos/iac-apply-latency.openslo.yaml` define timing, and `microservices/cloud-iac/runbooks/gitops-reconciler-restart.md` covers first-action sync failure.

## C. Deliverables
- First-action rollout examples in OpenAPI and AsyncAPI contracts.
- SLO references for render/apply latency.
- GitOps reconciler restart runbook reference.
- Registry readiness evidence from `microservices/cloud-iac/dashboards/registry-health.json`.

## D. Implementation
1. Add `tenant_onboarding_ref`, `pack_id`, `first_action_id`, and `registry_ready_ref` to j100 examples.
2. Render the pack module and validate tenant, pack, and residency before apply.
3. Apply infrastructure through the applier only after SLO-gated validation.
4. Mark registry readiness before handing off to the next µservice.
5. Watch GitOps sync and execute the reconciler restart runbook on sync failure.
6. Roll back first-action infrastructure through the rollback bounded context.

## E. Acceptance
- First action cannot complete without registry readiness evidence.
- Render and apply latency SLOs are cited.
- GitOps failure path names `microservices/cloud-iac/runbooks/gitops-reconciler-restart.md`.
- Contract examples avoid generic journey placeholders.

## F. Evidence
- Journey: `docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md`.
- SLOs: `microservices/cloud-iac/slos/iac-render-latency.openslo.yaml`, `microservices/cloud-iac/slos/iac-apply-latency.openslo.yaml`.
- Dashboard: `microservices/cloud-iac/dashboards/registry-health.json`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| ArgoCD / Flux | Keeps first sync visible while adding cloud-iac registry readiness. |
| Terraform Cloud | Adds pack rollout and tenant first-action evidence around apply. |
| Spacelift / Env0 | Matches environment rollout while adding SLO-gated handoff evidence. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/cloud-iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml`, `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml`, `microservices/cloud-iac/contracts/proto/cloud-iac.proto`, `microservices/cloud-iac/IP-journey-j100-pack-rollout-first-action.md`.

## DR posture (per ADR-0343)

- Target source: `microservices/cloud-iac/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `seaweedfs_replicated`, `postgres_wal_g`].
- Surface evidence: `microservices/cloud-iac/runbooks/restore-drill-quarterly.md`, `microservices/cloud-iac/runbooks/seaweedfs-volume-failover.md`, `microservices/cloud-iac/manifest.json`, `microservices/cloud-iac/IP-journey-j100-pack-rollout-first-action.md`.
