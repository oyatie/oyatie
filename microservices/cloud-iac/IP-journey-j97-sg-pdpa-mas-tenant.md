---
doc_class: Implementation-Plan
ip_id: IP-journey-j97-sg-pdpa-mas-tenant
journey_ref: docs/user-journeys/j97-sg-pdpa-mas-singapore-tenant/
microservice: cloud-iac
status: draft
date: 2026-05-21
---

# IP-journey-j97: Singapore PDPA / MAS infrastructure tenant

## A. Problem
J97 requires cloud-iac to provision infrastructure that can prove Singapore PDPA/MAS tenancy boundaries, operational resilience, and incident drill readiness. The old IP repeated generated task tables without actual cloud-iac anchors.

## B. Approach
Bind provisioning to `microservices/cloud-iac/policy/iac-isolation.md`, `microservices/cloud-iac/runbooks/restore-drill-quarterly.md`, and `microservices/cloud-iac/slos/iac-rollback-latency.openslo.yaml`. The validator enforces pack and cell posture; applier mutates only after signed validation.

## C. Deliverables
- Singapore tenant examples in `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml`.
- Incident/rollback event examples in `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml`.
- Isolation and restore drill evidence references.
- Registry binding through `microservices/cloud-iac/catalog/oya-cloud-iac-iac-registry-kernel.yaml`.

## D. Implementation
1. Add `sg_pdpa_pack_ref`, `mas_profile`, and `operational_resilience_drill_ref` to j97 examples.
2. Validate isolation policy and state backend locality before apply.
3. Emit readiness and rollback events with audit hashes.
4. Record the applied state in iac-registry.
5. Verify restore drill before tenant activation.
6. Deny readiness if rollback SLO evidence is missing.

## E. Acceptance
- Singapore examples cite isolation, restore, rollback SLO, and registry artifacts.
- The applier cannot override validator denial.
- Rollback evidence is tied to `microservices/cloud-iac/slos/iac-rollback-latency.openslo.yaml`.
- No generic `TASK-###` or `FM-###` rows remain.

## F. Evidence
- Journey: `docs/user-journeys/j97-sg-pdpa-mas-singapore-tenant/README.md`.
- SLO: `microservices/cloud-iac/slos/iac-rollback-latency.openslo.yaml`.
- Runbook: `microservices/cloud-iac/runbooks/restore-drill-quarterly.md`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| Spacelift | Adds MAS/PDPA restore-drill evidence to policy-gated IaC. |
| Terraform Cloud | Adds tenant activation readiness beyond workspace apply history. |
| ArgoCD / Flux | Adds rollback SLO and registry evidence beyond sync state. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/cloud-iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml`, `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml`, `microservices/cloud-iac/contracts/proto/cloud-iac.proto`, `microservices/cloud-iac/IP-journey-j97-sg-pdpa-mas-tenant.md`.

## DR posture (per ADR-0343)

- Target source: `microservices/cloud-iac/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `seaweedfs_replicated`, `postgres_wal_g`].
- Surface evidence: `microservices/cloud-iac/runbooks/restore-drill-quarterly.md`, `microservices/cloud-iac/runbooks/seaweedfs-volume-failover.md`, `microservices/cloud-iac/manifest.json`, `microservices/cloud-iac/IP-journey-j97-sg-pdpa-mas-tenant.md`.
