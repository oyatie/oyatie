---
doc_class: PolicySpec
title: Cell-Boundary Specification
microservice: cell
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-cell-substrate
deciders: council-architecture, ops-security, axis-cell-substrate, council-privacy
related_adrs: [ADR-0028, ADR-0117, ADR-0130, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/cell/threat-model.md (Trust Boundary 2/3/4/5, T-S-01..T-S-04, T-I-01)
  - microservices/cell/dpia.md (R-01, R-02, R-03)
  - microservices/cell/policy/tenant-scope.cedar
  - microservices/cell/policy/ci-scope.cedar
  - microservices/cell/policy/auditor-scope.cedar
  - microservices/cell/policy/public-read.cedar
review_cadence: quarterly + on every Kubernetes / Postgres / Cluster API version upgrade
doc_status: published
---

# Cell-Boundary Specification (cell µservice)

## Purpose

Define the load-bearing **hard tenant-isolation** invariants of the cell substrate. This document is the canonical reference for SOC 2 examiners, ISO 27001 auditors, GDPR Art. 32 reviewers, KR PIPA Art. 23 / Art. 29 reviewers, HIPAA §164.312(a)(1) reviewers, and pen-testers asking *"how does the cell substrate prevent tenant-A's workload from accessing tenant-B's data or compute?"*

## Cell Identity Model

### Cell ID derivation

```text
canonical_cell_id   = ULID at lifecycle-manager create-time
hashed_cell_id      = sha256(canonical_cell_id ++ deployment_salt)[..16]
namespace_id        = "cell-" + hashed_cell_id    (used as K8s namespace name)
postgres_schema     = "cell_" + hashed_cell_id    (per-cell logical schema)
s3_prefix           = "cells/" + hashed_cell_id   (per-cell object-storage prefix)
spiffe_id           = "spiffe://oyatie/cell/" + hashed_cell_id
```

Properties:
- `canonical_cell_id` is the OpenBao-bound subject; never appears in user-facing surfaces.
- `deployment_salt` rotates per-pack every 12 months; rotation event audited.
- 16-hex truncation provides ~10¹⁹ collision-free namespace; sufficient for foreseeable cell scale.

### Cell scope enumeration

Each cell carries a `cell_scope` enum at lifecycle-manager creation:

```yaml
cell_scope:
  enum: [shared, dedicated, hipaa-dedicated, sandbox, internal]
  description: |
    - shared: N tenants per cell (default for tenant_scope: trial + production)
    - dedicated: 1 tenant per cell (premium tenants; financial; on-request)
    - hipaa-dedicated: 1 Covered Entity per cell (pack-us-healthcare); BAA-bound
    - sandbox: customer-owned non-prod; isolated from prod-tier
    - internal: oyatie's own services dogfooding cell substrate
```

`cell_scope` is set at cell creation and immutable. Migration from `shared` to `dedicated` requires creating a new cell + migrating the tenant via `tenant-migration` use case.

### Reserved cell IDs

Reserved cell IDs are not issued to customer tenants:

| Reserved cell | Purpose | Write authority | Read authority |
|---|---|---|---|
| `cell:oya-control-plane` | observability + governance + cell substrate themselves | observability + governance + cell substrate workers via SPIFFE | platform operators |
| `cell:oya-warm-pool` | host-pool internal accounting (nodes not yet bound to a customer cell) | host-pool worker only | platform operators |
| `cell:oya-decommissioning-quarantine` | cells in soft-delete window | lifecycle-manager only | ops-security + axis-cell-substrate during quarantine review |

## Cell Multi-Tenancy Invariants

### Invariant CB-01: Per-cell namespace isolation always

Every customer-tenant cell maps to **exactly one** Kubernetes namespace + Postgres logical schema + S3 prefix. NetworkPolicy denies all cross-namespace traffic except via approved mesh-internal control-plane routes. CI lane `oya-cell-boundary` validates the K8s manifests in any workload µservice for cross-namespace references; PRs that violate fail to merge.

### Invariant CB-02: Per-cell credentials bound at issuance

Every credential issued by OpenBao for cell-scoped workloads carries a non-modifiable `bound_cell_id` claim. Postgres adapter validates: `connection.session_cell_id == credential.bound_cell_id`. Mismatch returns auth-error + emits `oya_cell_boundary_credential_mismatch_total` (alert on > 0 over 1m).

### Invariant CB-03: Cross-cell DB connection refusal at Postgres (server-side)

Cross-cell DB queries are refused **server-side at Postgres** before any data is read. Row-level-security (RLS) on every cell-scoped table is keyed on the session's `cell_id` GUC variable, set from the credential's bound claim. Wildcard cell queries are forbidden by policy; even Postgres superuser queries that omit cell-scope require 2-person JIT elevation.

CI lane: `oya-cell-no-cross-cell-query` greps workload µservice query layers for any SQL that selects across cells; presence fails the lane.

### Invariant CB-04: No client-side cell filtering as the only check

Application-layer `WHERE cell_id = $1` selectors are advisory; the server enforces actual scope. Even if a client somehow omits the cell-scope filter, Postgres RLS applies.

This invariant exists to prevent a class of bugs where a client-side library is the only line of defence; the server is the line of defence.

### Invariant CB-05: Per-cell capacity limits

| Limit | Default | Configurable per cell_scope | Enforcement |
|---|---|---|---|
| Max tenants per cell | 100 | shared: 100; dedicated: 1; hipaa-dedicated: 1; sandbox: 50; internal: 1000 | scheduler placement policy |
| Max active workflows per cell | 10k | varies by scope | observability per-cell SLO |
| Max Postgres connections per cell | 100 | varies | PgBouncer pool |
| Max object-storage prefix bytes per cell | 1 TB | varies | retention policy |

Excess at any of these limits triggers cell-rebalance (per `runbooks/cell-rebalance.md`) before user impact.

### Invariant CB-06: Reserved-namespace write authority

K8s namespaces matching the prefixes below have **write authority restricted** to specific identities:

| Namespace prefix | Authorised writer (SPIFFE) | Rejection action |
|---|---|---|
| `cell-*` | `spiffe://oyatie/cell/lifecycle-manager` only | 403 + `oya_cell_namespace_unauthorized_writer_total` |
| `cell-control-plane` | observability + governance + cell substrate operators | 403 |
| `cell-warm-pool-*` | `spiffe://oyatie/cell/host-pool` only | 403 |

### Invariant CB-07: Per-cell read authorization via Cedar

Every read request (REST API, SDK, internal mesh call) passes through a Cedar policy evaluator (per ADR-0140) that enforces:

```cedar
permit (
  principal,
  action in [Action::"read_cell_assignment", Action::"read_cell_metadata"],
  resource is Cell
) when {
  principal has cell_id_scope &&
  resource has cell_id &&
  resource.cell_id in principal.cell_id_scope
};
```

(Full fragment at `policy/tenant-scope.cedar`.) The evaluator runs in `cell-registry-rest`; non-matching reads return 403 + emit `oya_cell_unauthorized_read_attempt_total`.

### Invariant CB-08: Cross-pack assignment refusal

A cell may belong to exactly one pack. Tenant→cell assignment must satisfy `tenant.pack == cell.pack`. Cross-pack assignment attempts return 403 at the scheduler write path; Postgres RLS re-checks pack at commit time. CI fuzz: `oya-cell-cedar-fuzz` injects cross-pack writes and asserts refusal.

## Failure Modes

### FM-CB-01: Workload µservice attempts cross-cell DB query

**Behaviour:** Postgres RLS refuses; query returns 0 rows + emits audit event. Application layer sees empty result + audit event guides root-cause.

**Tenant impact:** None at boundary; potential user-visible empty-result if not handled defensively at workload layer.

**Detection:** `oya_cell_boundary_violation_total > 0` over 5m → Sev-1 page.

**Recovery:** Audit lineage of offending code path; engage workload owner; PR-time lane should have caught — investigate lane gap.

### FM-CB-02: Namespace credential drift (someone manually creates a cross-cell binding)

**Behaviour:** OpenBao audit + LEAN check + ArgoCD drift detector all flag within ≤ 5 min.

**Tenant impact:** Bounded to detection window; auto-rollback restores declared state.

**Detection:** `oya-governance-openbao-conformance` lane + ArgoCD drift alarm.

**Recovery:** Auto-rollback to declared state; root-cause investigation; engage ops-security.

### FM-CB-03: Postgres RLS misconfiguration

**Behaviour:** CI lane refuses merge of migrations that omit RLS clauses; runtime LEAN check asserts RLS active on every cell-scoped table.

**Tenant impact:** Caught pre-deploy.

**Detection:** Migration CI lane + runtime check.

**Recovery:** Migration revert; runtime check fail-closes cell-registry-rest (returns 503) until RLS restored.

### FM-CB-04: SPIFFE identity spoofing (attacker stands up workload with same SA name)

**Behaviour:** SPIFFE attestation includes pod-identity binding (workload UUID + namespace + SA); spoofed workload in same namespace requires cluster-admin which is blocked by RBAC.

**Tenant impact:** Defence-in-depth required: combined with cluster-admin RBAC, Cedar, NetworkPolicy.

**Detection:** SPIFFE attestation log; anomalous SVID issuance alarms.

**Recovery:** Revoke SVID; pod-identity audit; incident response.

### FM-CB-05: Reserved-namespace write from non-authorised source

**Behaviour:** K8s admission webhook (deployed by lifecycle-manager) + Cedar policy reject; audit-emit.

**Tenant impact:** None (write rejected).

**Detection:** `oya_cell_namespace_unauthorized_writer_total > 0` over 5m fires page.

**Recovery:** Trace source; revoke offending credentials; root-cause.

### FM-CB-06: Cross-pack assignment attempt

**Behaviour:** Scheduler refuses at decision time; Postgres RLS re-checks at commit; Cedar refuses at API surface.

**Tenant impact:** None (rejected); residency posture preserved.

**Detection:** `oya_cell_cross_pack_attempt_total > 0` → Sev-1 (residency breach is a regulator-notifiable event class).

**Recovery:** Audit-chain trace; engage council-privacy; if write somehow committed, immediate migration to correct pack + breach-notification chain per `incident-response.md`.

### FM-CB-07: Cell namespace deleted without proper decommission flow

**Behaviour:** lifecycle-manager mandates state-machine transitions; manual kubectl deletion blocked by K8s admission webhook.

**Tenant impact:** Caught pre-deletion.

**Detection:** Admission-webhook denial event.

**Recovery:** Re-create via lifecycle-manager if mistake; investigation if intentional.

## Audit Trail

Every cross-cell boundary event is audit-chain-emitted per Bominal ADR-0028:

| Event | Emitter | Fields | Retention |
|---|---|---|---|
| Cross-cell DB query attempt | Postgres RLS audit | `attempted_cell_id, source_spiffe_id, query_hash, request_id, timestamp` | ≥ 1y (HIPAA pack: 6y) |
| Reserved-namespace write attempt | K8s admission webhook | `namespace, source_spiffe_id, attempted_op, timestamp` | ≥ 1y |
| Unauthorised read attempt | cell-registry-rest (Cedar evaluator) | `principal_id, requested_cell_id, action, resource, timestamp` | ≥ 1y |
| Cell credential issuance | OpenBao | `credential_id, bound_cell_id, issued_by, requestor, ttl, timestamp` | ≥ 1y |
| Cell credential revocation | OpenBao | `credential_id, revoked_by, reason, timestamp` | ≥ 1y |
| Deployment salt rotation | OpenBao + cell admin | `prev_salt_hash, new_salt_hash, rotated_by, timestamp` | indefinite |
| Cross-pack assignment attempt | scheduler / cell-registry | `attempted_pack, principal_pack, tenant_id, timestamp` | indefinite |
| Cell-state transition | lifecycle-manager | `cell_id, prev_state, new_state, executed_by, reason, timestamp` | indefinite |
| Tenant migration | tenant-assignment-worker | `tenant_id, source_cell, target_cell, started_at, completed_at, signature` | indefinite |
| Cell decommission | lifecycle-manager | `cell_id, decommissioned_by_quorum, reason, timestamp` | indefinite |

Audit log lives in Mimir under `tenant:oya-self` + replicated to `audit-chain` µservice for Merkle-tree sealing.

## Per-Pack Overlay

### pack-kr (KR PIPA + ISMS-P)

- KR PIPA Art. 29 (technical safeguards) maps to CB-01..CB-08 + FM-CB-01..FM-CB-07.
- Audit log retention ≥ 1 year per PIPA Enforcement Decree Art. 30; extended to 3 years for `tenant_scope: production`.
- KR PIPA Art. 23 (sensitive data): `(hashed_tenant_id, cell_id)` is sensitive; salt rotation per Art. 29.

### pack-us-healthcare (HIPAA)

- §164.312(a)(1) access-control: CB-01..CB-08 satisfy Unique User Identification + Encryption-and-Decryption.
- §164.312(b) audit controls: Audit Trail table.
- §164.312(e)(1) transmission security: mTLS + TLS 1.3.
- §164.502(b) minimum-necessary: per-cell scope enforces least-data.
- Audit retention ≥ 6 years per §164.316(b)(2).
- BAA per Covered Entity tenant; cells in `hipaa-dedicated` scope.

### pack-eu (GDPR + EDPB)

- Art. 32(1)(a) pseudonymisation: tenant identifiers hashed; cell IDs hashed.
- Art. 32(1)(b)(c) integrity + availability: CB-01..CB-08.
- Art. 32(1)(d) regular testing: annual pen-test + quarterly chaos drill.
- Art. 25 by design and default.
- Arts. 44–50 transfers: forbidden cross-pack; SCC exception only.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/cell-boundary-overlay.md` map local PII law's confidentiality + integrity requirements.

## Verification

- `cargo run -p oya-dev-cli -- gate validate cell-boundary --microservice <ms>` — exit 0 for every workload µservice.
- `cargo run -p oya-dev-cli -- gate validate cell-no-cross-cell-query` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate cell-rls-conformance` — exit 0 (Postgres RLS active on every cell-scoped table).
- `cargo run -p oya-dev-cli -- gate validate cedar-fragment-coverage --microservice cell` — exit 0.
- Annual pen-test against cell-boundary: scheduled in `compliance.md`.
- Quarterly chaos drill: induce cross-cell query attempt + cross-pack write attempt; verify refusal + alerting.

## References

- ADR-0028 (Bominal): audit-chain.
- ADR-0117: cloud-native infrastructure + residency.
- ADR-0130: agentic SLO-gated promotion.
- ADR-0131: per-microservice flat layout.
- ADR-0140: Cedar policy enforcement.
- Bominal ADR-0009 (cell architecture).
- Bominal ADR-0019 (runtime catalog + cell sharding).
- `microservices/cell/threat-model.md`.
- `microservices/cell/dpia.md`.
- `microservices/cell/policy/{tenant-scope, ci-scope, auditor-scope, public-read}.cedar`.
- Kubernetes Multi-Tenancy SIG — `github.com/kubernetes-sigs/multi-tenancy`.
- SPIFFE / SPIRE — `spiffe.io`.
- OpenBao — `openbao.org`.
