---
ip_id: IP-026
microservice: compliance
bounded_context: compliance-control-mapping
layer: rest
status: planned
related_adrs: [ADR-0253, ADR-0258, ADR-0243, ADR-0209]
---

# IP-026 — control-mapping REST + SDK

## A. Problem

Control mapping is only useful when auditors, tenant compliance admins, and Foundry validators can query it consistently. `contracts/openapi.yaml` exposes generic evidence coverage but not framework/control drill-down, and there is no SDK contract for validators. Commercial counterparts treat control mapping as a primary product surface; Oyatie needs the same surface with tenant isolation and audit-chain references.

## B. Approach

Expose IP-022 through read-only REST endpoints and generated Rust/TypeScript SDKs. The API returns framework roster, controls, collector bindings, satisfaction status, attestation history, and evidence refs. It does not mutate controls; pack publication and mapping writes remain in the domain/pack registry paths.

## C. Deliverables

| Artifact | Change |
|---|---|
| `microservices/compliance/contracts/openapi.yaml` | add framework/control/attestation-history paths |
| `microservices/compliance/catalog/api-rest.yaml` | catalog REST surface |
| `microservices/compliance/sdk-plan.md` | SDK generation and compatibility plan |
| `microservices/compliance/policy/action-authorization.cedar` | authorize compliance-admin and Foundry validator reads |
| `microservices/compliance/dashboards/evidence-coverage.json` | link API-visible coverage to dashboard state |

## D. Implementation

1. Add `GET /v1/frameworks` returning canonical and pack-added frameworks.
2. Add `GET /v1/frameworks/{fid}/controls` with pagination and pack filters.
3. Add `GET /v1/controls/{cid}` returning requirement text, collector ids, responsible microservice, status, and last attestation.
4. Add `GET /v1/controls/{cid}/attestation-history` returning evidence refs and audit seal refs.
5. Enforce `policy/action-authorization.cedar` and tenant scope on every route.
6. Generate Rust and TypeScript SDK clients from the OpenAPI contract and pin SemVer behavior in `sdk-plan.md`.
7. Add tests for forbidden tenant read, framework roster, pack overlay control, stale control status, and SDK snapshot compatibility.
8. Ensure HTTP/3 + ECH + PQC settings match IP-020 where applicable.

## E. Acceptance

- OpenAPI includes all four control-mapping routes and schemas.
- Tenant admins see only their tenant and active packs; Foundry validators get only validation-scoped reads.
- SDK snapshots compile and preserve SemVer-compatible response shapes.
- Evidence refs expose audit seals without embedding raw evidence payload.

## F. Evidence

- `microservices/compliance/contracts/openapi.yaml` is the REST authority.
- `microservices/compliance/sdk-plan.md` is the SDK launch plan.
- `microservices/compliance/competitor-parity-matrix.md` marks pre-mapped controls as table-stakes for Drata, Vanta, Tugboat/OneTrust, AuditBoard, and ServiceNow GRC.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| Drata / Vanta | Matches customer-facing control mapping and evidence drill-down. |
| AuditBoard | Provides control catalogue/API parity with stronger audit seal references. |
| ServiceNow GRC | Gives enterprise integrations a typed API without making ServiceNow the system of record. |

## H. Non-goals and handoff boundaries

- Do not mutate controls, mappings, pack state, or evidence artifacts from this read surface.
- Do not expose raw evidence payloads; return evidence refs and seal refs.
- Do not let Foundry validators read outside validation scope.
- Do not publish SDKs with unstable response fields not covered by `sdk-plan.md`.
- Do not bypass Cedar on cached SDK clients; server-side authorization remains mandatory.

## I. Fixture set

- `list_frameworks_with_pack_overlay.json` proves roster shape.
- `get_soc2_control_detail.json` proves collector/status fields.
- `attestation_history_redacted_refs.json` proves no payload embedding.
- `tenant_cross_read_forbidden.json` proves tenant isolation.
- `typescript_sdk_snapshot.json` and `rust_sdk_snapshot.json` prove generated-client compatibility.

## J. Launch blockers

- REST exposes mutation routes for controls or pack state.
- SDK snapshots omit tenant-scope authorization behavior.
- Control details embed raw evidence payloads.
- Foundry validator reads outside declared validation scope.
- OpenAPI changes break generated Rust or TypeScript clients without a SemVer note.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/compliance/IP-026-control-mapping-rest-and-sdk.md` matched `openapi`; contract files `microservices/compliance/contracts/openapi.yaml, microservices/compliance/contracts/asyncapi.yaml, microservices/compliance/contracts/compliance.proto`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.
