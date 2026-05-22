---
ip_id: IP-020
microservice: compliance
bounded_context: regulator-audit-evidence
layer: rest
status: planned
related_adrs: [ADR-0243, ADR-0253, ADR-0258, ADR-0263]
---

# IP-020 — regulator-audit-evidence REST

## A. Problem

External auditors and regulators need scoped evidence access, but the current REST contract only exposes generic evidence and DSAR paths. The auditor portal must not become a broad admin API. This IP creates a regulator/auditor REST surface that is engagement-scoped, Cedar-gated, HTTP/3-capable, and audit-sealed on every view/export.

## B. Approach

Extend `contracts/openapi.yaml` with engagement-scoped read paths backed by `oya-compliance-regulator-audit-evidence-rest`. Every request carries engagement id, tenant id, framework/pack scope, time window, and principal identity. `policy/auditor-scope.cedar` decides whether metadata-only or payload download is allowed. SeaweedFS and audit-chain are accessed through existing evidence ports, not directly from handlers.

## C. Deliverables

| Artifact | Change |
|---|---|
| `microservices/compliance/catalog/oya-compliance-regulator-audit-evidence-rest.yaml` | REST catalog row |
| `microservices/compliance/contracts/openapi.yaml` | add `/engagements/{eid}/evidence`, `/control-mapping`, and `/exports` paths |
| `microservices/compliance/policy/auditor-scope.cedar` | engagement-window and artifact-scope authorization |
| `microservices/compliance/runbooks/regulator-evidence-export-failure.md` | failure response for export errors |
| `microservices/compliance/dashboards/regulator-engagement-activity.json` | view/export/revoke telemetry |

## D. Implementation

1. Add OpenAPI schemas for `RegulatorEngagement`, `ScopedEvidenceRecord`, `EvidenceExportRequest`, and `EvidenceExportReceipt`.
2. Enforce `policy/auditor-scope.cedar` on every handler before evidence lookup.
3. Add `GET /v1/engagements/{eid}/evidence` for scoped listing and `GET /v1/engagements/{eid}/evidence/{evid}` for metadata/payload access.
4. Add `POST /v1/engagements/{eid}/exports` to create signed export bundles and `GET /exports/{xid}` for download.
5. Emit `oya.compliance.regulator-engagement-view`, `export-requested`, `export-ready`, and `export-failed`.
6. Wire HTTP/3 Alt-Svc, ECH, and PQC settings to `iac/edge-waf.yaml` and `iac/pqc-cert.yaml`.
7. Add tests for inactive engagement, out-of-window artifact, metadata-only scope, payload scope, export failure, and revoke.

## E. Acceptance

- No regulator endpoint returns evidence without an active engagement.
- Auditor payload downloads are optional and Cedar-scoped; metadata-only engagements cannot fetch blobs.
- Every evidence view and export emits audit evidence.
- HTTP/3 defaults with h2 fallback and no policy bypass.

## F. Evidence

- `microservices/compliance/contracts/openapi.yaml` is the current REST authority.
- `microservices/compliance/policy/auditor-scope.cedar` is the local authorization surface.
- `microservices/compliance/PRD.md` requires auditor self-service portal access and seal verification.
- Competitors: Drata, Vanta, AuditBoard, ServiceNow GRC all offer auditor access; Oyatie differentiates on verifiable audit-chain evidence.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| Drata / Vanta | Matches auditor portal expectations while avoiding third-party evidence custody. |
| AuditBoard | Provides engagement-scoped evidence exports with stronger tamper evidence. |
| AWS Audit Manager | Parallels auditor-view API patterns inside Oyatie's tenant-scoped control plane. |

## H. Non-goals and handoff boundaries

- Do not create or mutate controls from regulator endpoints; IP-026 is read-only and IP-022 owns mapping.
- Do not expose raw payload downloads unless `policy/auditor-scope.cedar` grants payload scope.
- Do not let expired engagements continue through cached sessions.
- Do not use this surface for tenant self-service DSAR; DSAR remains under `/dsar/*`.
- Do not bypass HTTP policy on h2 fallback; h2 and h3 must hit the same Cedar path.

## I. Fixture set

- `inactive_engagement_list_denied.json` proves closed engagement denial.
- `metadata_only_payload_download_denied.json` proves scoped evidence access.
- `in_window_artifact_visible.json` proves happy path.
- `out_of_window_artifact_hidden.json` proves engagement-window filtering.
- `export_failure_emits_event.json` proves runbook trigger.

## J. Launch blockers

- Closed engagements can list evidence through cached credentials.
- Metadata-only grants can download payloads.
- Export bundles omit audit-chain seal references.
- HTTP/2 fallback bypasses the same Cedar policy path.
- View events are missing tenant, engagement, artifact, or principal ids.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/compliance/IP-020-regulator-audit-evidence-rest.md` matched `openapi`; contract files `microservices/compliance/contracts/openapi.yaml, microservices/compliance/contracts/asyncapi.yaml, microservices/compliance/contracts/compliance.proto`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.
