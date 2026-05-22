---
doc_class: Implementation-Plan
ip_id: IP-journey-j94-sox404-public-company-controls
journey_ref: docs/user-journeys/j94-sox-404-public-company-controls/
microservice: cloud-iac
status: draft
date: 2026-05-21
---

# IP-journey-j94: SOX 404 infrastructure change-control evidence

## A. Problem
J94 needs evidence that infrastructure changes affecting financial reporting controls are authorized, reviewed, applied, and reversible. cloud-iac owns the render/validate/apply trail and must make that trail SOX-auditable.

## B. Approach
Use existing cloud-iac contracts and dashboards to bind change-control evidence: `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml`, `microservices/cloud-iac/dashboards/apply-success-rate.json`, `microservices/cloud-iac/runbooks/state-lock-break.md`, and `microservices/cloud-iac/slos/slsa-provenance-completeness.openslo.yaml`.

## C. Deliverables
- SOX change ticket fields in OpenAPI examples.
- Apply evidence events in AsyncAPI.
- State-lock recovery references.
- SLSA provenance acceptance for every control-affecting apply.

## D. Implementation
1. Add `change_ticket_ref`, `control_id`, `reviewer_principal_id`, and `provenance_ref` to j94 examples.
2. Require validator success and reviewer principal before apply.
3. Record apply result and provenance hash in iac-registry.
4. Deny direct state-lock breaks unless runbook evidence is attached.
5. Exercise rollback for a failed control-affecting apply.
6. Add negative acceptance for missing reviewer or unsigned module.

## E. Acceptance
- Control-affecting applies require ticket, reviewer, provenance, and tenant scope.
- State-lock recovery cites `microservices/cloud-iac/runbooks/state-lock-break.md`.
- SLSA provenance SLO is named in acceptance evidence.
- No SOX row claims manual approval outside the cloud-iac evidence chain.

## F. Evidence
- Journey: `docs/user-journeys/j94-sox-404-public-company-controls/README.md`.
- Dashboard: `microservices/cloud-iac/dashboards/apply-success-rate.json`.
- SLO: `microservices/cloud-iac/slos/slsa-provenance-completeness.openslo.yaml`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| Terraform Cloud | Adds SOX control IDs and signed provenance to plan/apply evidence. |
| Spacelift | Matches approval gates while binding them to Oyatie audit-chain events. |
| Atlantis | Moves PR-time approval into a registry-backed control evidence trail. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/cloud-iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml`, `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml`, `microservices/cloud-iac/contracts/proto/cloud-iac.proto`, `microservices/cloud-iac/IP-journey-j94-sox404-public-company-controls.md`.

## DR posture (per ADR-0343)

- Target source: `microservices/cloud-iac/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOX-404` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `3600` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `seaweedfs_replicated`, `postgres_wal_g`].
- Surface evidence: `microservices/cloud-iac/runbooks/restore-drill-quarterly.md`, `microservices/cloud-iac/runbooks/seaweedfs-volume-failover.md`, `microservices/cloud-iac/manifest.json`, `microservices/cloud-iac/IP-journey-j94-sox404-public-company-controls.md`.
