---
doc_class: Implementation-Plan
ip_id: IP-journey-j93-in-dpdpa-rbi-overlay
journey_ref: docs/user-journeys/j93-in-dpdpa-rbi-financial-overlay/
microservice: cloud-iac
status: draft
date: 2026-05-21
---

# IP-journey-j93: India DPDPA / RBI financial infrastructure overlay

## A. Problem
J93 requires cloud-iac to prove India financial-tenant infrastructure is pack-aware, auditable, and reversible. The prior stamped IP mixed generic regulatory tasks with no cloud-iac module or policy ownership.

## B. Approach
Use the iac-validator to enforce DPDPA/RBI pack inputs before apply, iac-registry to store state refs, and iac-rollback to compensate failed or revoked overlays. Real anchors are `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml`, `microservices/cloud-iac/policy/tenant-scope.cedar`, and `microservices/cloud-iac/slos/iac-validator-availability.openslo.yaml`.

## C. Deliverables
- India pack validation examples in OpenAPI.
- Denied and permitted validation events in AsyncAPI.
- Registry catalog link to `microservices/cloud-iac/catalog/oya-cloud-iac-iac-registry-usecase.yaml`.
- Rollback runbook link to `microservices/cloud-iac/runbooks/rollback-orchestration.md`.

## D. Implementation
1. Add `jurisdiction_code=IN`, `rbi_overlay_ref`, and `financial_cell_ref` to j93 examples.
2. Deny plans missing the RBI overlay before the applier runs.
3. Attach tenant Cedar decisions to every financial overlay mutation.
4. Register final state with evidence hash and state backend ref.
5. Simulate rollback after RBI overlay revocation.
6. Capture validator SLO evidence for the readiness claim.

## E. Acceptance
- Missing RBI overlay is a validator denial, not a partial apply.
- Registry rows are tenant-scoped and do not leak provider credentials.
- Rollback is documented as append-only compensation.
- Validator availability SLO is part of acceptance evidence.

## F. Evidence
- Journey: `docs/user-journeys/j93-in-dpdpa-rbi-financial-overlay/README.md`.
- SLO: `microservices/cloud-iac/slos/iac-validator-availability.openslo.yaml`.
- Catalog: `microservices/cloud-iac/catalog/oya-cloud-iac-iac-registry-usecase.yaml`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| Spacelift | Adds financial-pack validation tied to Cedar and audit-chain evidence. |
| Terraform Cloud | Adds jurisdiction overlay checks before state mutation. |
| ArgoCD / Flux | Keeps GitOps sync downstream of pack-aware infrastructure approval. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/cloud-iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml`, `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml`, `microservices/cloud-iac/contracts/proto/cloud-iac.proto`, `microservices/cloud-iac/IP-journey-j93-in-dpdpa-rbi-overlay.md`.

## DR posture (per ADR-0343)

- Target source: `microservices/cloud-iac/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOX-404` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `3600` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `seaweedfs_replicated`, `postgres_wal_g`].
- Surface evidence: `microservices/cloud-iac/runbooks/restore-drill-quarterly.md`, `microservices/cloud-iac/runbooks/seaweedfs-volume-failover.md`, `microservices/cloud-iac/manifest.json`, `microservices/cloud-iac/IP-journey-j93-in-dpdpa-rbi-overlay.md`.
