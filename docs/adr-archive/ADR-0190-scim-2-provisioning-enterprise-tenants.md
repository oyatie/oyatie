---
id: ADR-0190
status: Superseded
deciders: council-architecture, axis-identity, axis-tenancy
date: 2026-05-18
owner: axis-identity
supersedes: []
superseded_by: [ADR-0700]
related: [ADR-0145, ADR-0187, ADR-0175-tenant-lifecycle-workflow]
related_specs:
  - /specs/microservices/manifest-schema.json
microservice: identity
versions_current_as_of: 2026-05-18
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0190 — SCIM 2.0 inbound provisioning for enterprise tenants; pluggable adapter for non-SCIM HRIS

## Status

Accepted (2026-05-18). Mandates a SCIM 2.0 (RFC 7643 / 7644) endpoint per tenant in the `identity` µservice for inbound user/group lifecycle from enterprise IdPs (Okta, Microsoft Entra ID, Google Workspace, OneLogin, JumpCloud), and a pluggable adapter contract for HRIS systems that do not speak SCIM (Workday, BambooHR, Rippling).

## Context

Enterprise customers refuse to manually maintain user lists across two systems. The contractual posture they buy is: "your IdP / HRIS is the source of truth; oyatie auto-provisions, auto-updates, auto-deprovisions." Without SCIM, every B2B tenant pays per-seat for an integration we don't have; with SCIM the integration is RFC-driven and uniform.

SCIM 2.0 is the de-facto enterprise provisioning standard (Okta, Entra, Workspace, Cloudflare, Stripe, Slack, Notion all ship SCIM). HRIS systems are a different story: Workday speaks Workday-XML; BambooHR speaks REST-but-not-SCIM; Rippling speaks Rippling-API. They DO push user-lifecycle (hire / promote / terminate) — but via vendor-specific shapes. We adapt those at the edge into SCIM-internal.

## Decision

**Identity µservice exposes a SCIM 2.0 RFC 7643/7644 endpoint at `/scim/v2/{tenant}` per tenant. Inbound provisioning from Okta / Entra / Workspace pushes Users + Groups; lifecycle states (active, suspended, deleted) propagate. A pluggable adapter contract (`HrisAdapter` trait) handles non-SCIM HRIS sources by translating to internal SCIM-shaped operations.**

### SCIM 2.0 surface

| Endpoint | Method | Purpose | Auth |
|---|---|---|---|
| `/scim/v2/{tenant}/ServiceProviderConfig` | GET | discovery | none |
| `/scim/v2/{tenant}/Schemas` | GET | core + ext schemas | none |
| `/scim/v2/{tenant}/ResourceTypes` | GET | resource types | none |
| `/scim/v2/{tenant}/Users` | GET, POST | list, create | SCIM bearer |
| `/scim/v2/{tenant}/Users/{id}` | GET, PUT, PATCH, DELETE | CRUD | SCIM bearer |
| `/scim/v2/{tenant}/Groups` | GET, POST | list, create | SCIM bearer |
| `/scim/v2/{tenant}/Groups/{id}` | GET, PUT, PATCH, DELETE | CRUD | SCIM bearer |

Bearer token per tenant is provisioned via Zitadel admin API + stored as `${openbao:secret/identity/{pack}/{tenant}/scim-bearer}`. Tokens rotate every 90 days per ADR-0117.

### Schemas supported

- `urn:ietf:params:scim:schemas:core:2.0:User` (RFC 7643 §4.1)
- `urn:ietf:params:scim:schemas:extension:enterprise:2.0:User` (RFC 7643 §4.3) — manager, costCenter, department, division, employeeNumber, organization
- `urn:ietf:params:scim:schemas:core:2.0:Group` (RFC 7643 §4.2)
- `urn:oyatie:scim:extension:2.0:User` — `tenant_id`, `regulatory_pack`, `acr_floor`, `data_residency_jurisdiction`

### Lifecycle propagation

| SCIM op | Zitadel side effect | Tenancy side effect | Audit event |
|---|---|---|---|
| POST Users (create) | create User in tenant Org; assign default role | create UserMembership; charge per-seat metric | `IdentityUserProvisioned` |
| PATCH Users (active=true) | reactivate User in Zitadel | resume billing | `IdentityUserReactivated` |
| PATCH Users (active=false) | suspend User; revoke sessions; revoke WebAuthn challenges in-flight | suspend billing | `IdentityUserSuspended` |
| DELETE Users | soft-delete user, mark for GDPR-DSR cascade per ADR-0156 | issue tombstone; honor pack retention | `IdentityUserDeleted` |
| POST Groups | create Group in Zitadel | n/a | `IdentityGroupProvisioned` |
| PATCH Groups (members) | update Group membership | recompute Cedar entity-graph | `IdentityGroupMembershipChanged` |

### HRIS adapter contract

Non-SCIM HRIS (Workday, BambooHR, Rippling) integrates via the `HrisAdapter` trait:

```rust
pub trait HrisAdapter: Send + Sync {
    fn pull_hires(&self, since: DateTime<Utc>) -> Result<Vec<HrisHire>, HrisError>;
    fn pull_promotions(&self, since: DateTime<Utc>) -> Result<Vec<HrisChange>, HrisError>;
    fn pull_terminations(&self, since: DateTime<Utc>) -> Result<Vec<HrisTermination>, HrisError>;
}
```

The HRIS poller runs on cadence (default 15min); pulls events; translates to internal SCIM ops; calls Zitadel admin API + emits audit events. The HRIS is one-way push (HRIS → oyatie); user-initiated changes inside oyatie do NOT propagate back.

### SCIM idempotency

Per ADR-0149 idempotency-keys-canonical, SCIM POST requests include `If-Match` / `If-None-Match` headers; PATCH operations use ETag. Replay-safe operations enforce semantic equivalence.

## Alternatives considered

### Build proprietary provisioning API per IdP

Rejected. Five-way integration matrix multiplies fragility.

### Outbound provisioning (push from oyatie to IdP)

Rejected for the steady-state path. Enterprise IdPs are sources of truth; pushing back would create reconciliation hell. Outbound is reserved for the dev-portal account-recovery flow only.

### SCIM 1.1 fallback

Rejected. SCIM 1.1 is deprecated and lacks PATCH semantics.

### Bulk endpoint (RFC 7644 §3.7) MUST-support

Deferred. Bulk endpoint is optional; sized for high-velocity enterprises (>10k users). Implementation gated on adoption signal (≥1 customer requirement).

## Consequences

### Positive

- Enterprise sales motion unblocked: SCIM checklist item satisfied for every standard B2B IdP.
- Lifecycle propagation is automatic; no manual list-keeping.
- Adapter pattern lets us add HRIS sources without new ADRs (only new adapters).

### Negative

- SCIM operational tail (test against 5+ IdPs ongoing; each IdP's SCIM dialect has quirks) — Okta supports `eq` filter, Entra supports `co` filter, Workspace supports a different subset.
- Cross-tenant boundary discipline: SCIM bearer MUST be tenant-scoped; leaking a token = leaking a tenant's full user list.

### Neutral

- SCIM PATCH `replace` semantics differ subtly between Okta and Entra; we implement the union (RFC 7644 §3.5.2.1) and document the dialect quirks per IdP.

## Implementation

- `crates/oya-shared-scim-server-kernel` — `ScimServer` trait + reference impl with axum routes.
- Per-tenant Postgres tables `scim_users`, `scim_groups` indexed on `(tenant_id, external_id)`.
- Bearer token authn: HTTP Basic with constant-time compare against OpenBao-resolved value.
- AsyncAPI events `IdentityUserProvisioned`, `IdentityUserSuspended`, `IdentityGroupMembershipChanged` published to the workflow bus.

## Verification

- `cargo test -p oya-shared-scim-server-kernel` — full RFC 7644 happy path + idempotency + ETag concurrency.
- Conformance: SCIM 2.0 Compliance Test Suite (`scim2-compliance` golang tool) in CI.
- Integration test against Okta SCIM 2.0 mock + Entra SCIM endpoint mock.

## In-house roadmap

Per user directive 2026-05-18, evaluated under in-house policy:

- **Protocol**: SCIM 2.0 (RFC 7642 / 7643 / 7644) is an IETF **standard**. KEEP.
- **Server**: in-house from inception. The `oya-shared-scim-server-kernel` crate provides the `ScimServer` trait + reference impl handling RFC 7644 §3 endpoints with axum routes. Zitadel's SCIM endpoint is consumed only as a write-through target during Phase 0; the inbound HTTP surface seen by Okta / Entra / Workspace is OURS, not Zitadel's. This is the AWS Cognito / Google Workspace SCIM posture: serve our own SCIM, propagate to upstream IdP store.
- **Schema**: RFC 7643 core + RFC 7643 enterprise extension + oyatie extension `urn:oyatie:scim:extension:2.0:User` — in-house schema document.
- **HRIS adapters** (Workday / BambooHR / Rippling): in-house from inception in `oya-identity-hris-adapter-*` crates.
- **SCIM conformance test set**: external (`scim2-compliance` OSS tool) — KEEP as test dependency only.
- **Phase-2 swap delta**: when ADR-0187 advances to Phase 2, the SCIM adapter target swaps from Zitadel admin API to in-house identity-store directly; consumer SCIM clients see no change.

Conclusion: SCIM server is in-house from inception; only the write-through to Zitadel is Phase-0; Phase-2 swap removes that hop entirely.

## Cross-references

- RFC 7642 (SCIM definitions)
- RFC 7643 (SCIM core schema)
- RFC 7644 (SCIM protocol)
- ADR-0187 canonical-oidc-idp-zitadel-primary
- ADR-0175-tenant-lifecycle-workflow
- ADR-0149 idempotency-keys-canonical
- ADR-0156 pii-registry-canonical
