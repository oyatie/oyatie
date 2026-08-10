---
ip_id: IP-024
microservice: tenancy
bounded_context: kyb-kyc
layer: rest
status: planned
related_adrs: [ADR-0253, ADR-0258, ADR-0263, ADR-0243]
---

# IP-024 — KYB-KYC REST + AsyncAPI

## A. Problem

`IP-018` supplies KYB/KYC domain decisions, but tenants and substrate operators need a protocol surface to start verification, submit documents, poll case status, and receive completion events. Without a first-class REST and AsyncAPI contract, activation would be gated by an internal-only domain crate with no auditable tenant operator path.

## B. Approach

Extend `tenancy/contracts/openapi/tenancy.yaml` and `contracts/asyncapi/tenant-events.yaml` with KYB/KYC operations and events backed by `oya-tenancy-kyb-kyc-verifier-domain`. The REST layer enforces Cedar, size limits, data-class labels, audit-chain emission, and HTTP/3 transport posture.

## C. Deliverables

| Artifact | Action | Purpose |
|---|---|---|
| `tenancy/contracts/openapi/tenancy.yaml` | update | Add KYB/KYC routes and schemas. |
| `tenancy/contracts/asyncapi/tenant-events.yaml` | update | Add completion/decline/escalation channels. |
| `microservices/tenancy/src/crates/oya-tenancy-kyb-kyc-rest/Cargo.toml` | create | REST crate. |
| `src/routes.rs` | create | Route handlers for start, documents, status, decision. |
| `src/document_upload.rs` | create | Size, content-type, and data-class guard. |
| `tenancy/capabilities/kyb-kyc-complete.yaml` | align | REST and AsyncAPI evidence. |

## D. Implementation

1. Add `POST /v1/kyb-cases`, `POST /v1/kyb-cases/{case_id}/documents`, `GET /v1/kyb-cases/{case_id}`, and `POST /v1/kyb-cases/{case_id}/decision`.
2. Add schemas for `KybCase`, `KybDocumentSubmission`, `ScreeningResult`, `KybDecision`, and `KybCaseStatus`.
3. Enforce `TenantIdParam` and authenticated principal from the existing OpenAPI security scheme on every route.
4. Reject documents above configured size, unknown media types, and missing data classification before storage handoff.
5. Evaluate `policy/action-authorization.cedar` for start, upload, read, and decision actions.
6. Emit AsyncAPI channels `oya.tenancy.kyb-kyc-completed.v1`, `oya.tenancy.kyb-kyc-declined.v1`, and `oya.tenancy.kyb-kyc-escalated.v1`.
7. Add contract tests proving the OpenAPI operation ids and AsyncAPI channel names match the audit event classes.

## E. Acceptance

- `cargo nextest run -p oya-tenancy-kyb-kyc-rest --all-features`.
- OpenAPI validates and includes all KYB/KYC schemas and routes.
- AsyncAPI validates and includes completion, decline, and escalation channels.
- Default-deny Cedar path returns 403 and emits audit evidence.
- Document upload tests cover size limit, unsupported media type, and allowed path.

## F. Evidence

- `tenancy/contracts/openapi/tenancy.yaml` is the existing tenant REST surface.
- `tenancy/contracts/asyncapi/tenant-events.yaml` is the existing tenant event contract.
- `tenancy/IP-018-kyb-kyc-verifier-domain.md` owns the domain decision model.
- `tenancy/runbooks/kyb-kyc-pipeline-stalled.md` is the operational runbook for stuck verification.

## G. Counterparts

| Counterpart | Relevant capability | Gap this IP closes |
|---|---|---|
| Stripe Identity | Verification session API and webhook completion | Adds tenant-side verification routes and completion events. |
| WorkOS | Organization onboarding APIs | Exposes B2B tenant verification to operators instead of hidden internal state. |
| Auth0 Organizations | Organization lifecycle API | Extends org lifecycle with compliance evidence and audit events. |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `tenancy/IP-024-kyb-kyc-rest-and-async.md` matched `openapi, asyncapi`; contract files `tenancy/contracts/openapi/tenancy.yaml, tenancy/contracts/asyncapi/tenant-events.yaml, tenancy/contracts/proto/tenancy.proto`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `tenancy/IP-024-kyb-kyc-rest-and-async.md` matched `emission`; anchors `tenancy/manifest.json, crates/oya-tenancy-api/src/lib.rs`; type anchor `crates/oya-tenancy-api/src/lib.rs::TenantCreateApiRequest`.
