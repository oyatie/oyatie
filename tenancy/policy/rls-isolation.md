---
doc_class: PolicySpec
title: RLS Isolation Specification (the load-bearing isolation primitive)
microservice: tenancy
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-tenancy
deciders: council-architecture, ops-security, axis-tenancy, council-privacy
related_adrs: [ADR-0018, ADR-0028, ADR-0117, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - microservices/tenancy/threat-model.md (Trust Boundary 3, T-T-01, T-I-01, T-I-02, T-E-01)
  - microservices/tenancy/dpia.md (R-01)
  - microservices/tenancy/policy/tenant-scope.cedar
  - microservices/tenancy/policy/ci-scope.cedar
  - microservices/tenancy/policy/auditor-scope.cedar
  - microservices/tenancy/policy/public-read.cedar
review_cadence: quarterly + on every Postgres / Citus major-version upgrade
doc_status: published
---

# RLS Isolation Specification (tenancy µservice)

## Purpose

Define the **load-bearing isolation invariants** of the tenancy substrate. RLS (Postgres Row-Level Security) is the primary mechanism by which oyatie guarantees that tenant-A's queries cannot return tenant-B's rows — even under SQL injection, even under broken application logic, even under misconfigured caches, even under malicious code paths in any consuming µservice.

This document is the authoritative reference for SOC 2 examiners (CC6.1 / CC6.2 / CC6.6 / CC8.1), ISO 27001 auditors (A.5.15 / A.8.2 / A.8.3 / A.8.12 / A.8.32), GDPR Art. 32 reviewers, KR PIPA Art. 23 / Art. 29 reviewers, and HIPAA §164.312(a)(1) reviewers asking *"how does oyatie prevent tenant-A from seeing tenant-B's rows?"*

**The answer is RLS + JWT + Cedar defence-in-depth, with RLS as the load-bearing primitive.** Without RLS, the other two layers would be a single point of failure. With RLS, each layer must fail independently for a breach to occur.

## Tenant Identity Model

### Tenant ID derivation

```text
canonical_tenant_id   = <opaque-string-issued-at-onboarding>   (OpenBao-bound)
hashed_tenant_id      = sha256(canonical_tenant_id ++ deployment_salt)[..16]
JWT claim "tenant_id" = "tenant:" + hashed_tenant_id           (carried on every request)
Postgres `app.current_tenant_id` setting = "tenant:" + hashed_tenant_id  (SET LOCAL on every connection)
```

Properties:
- `canonical_tenant_id` is the OpenBao-bound subject; tenancy never exposes the raw value to operators / auditors via UI.
- `deployment_salt` is a per-pack secret rotated every 12mo. Rotation event captured in audit-chain.
- `hashed_tenant_id` truncation to 16 hex chars (64 bits) provides ~10¹⁹ collision-free namespace.
- The mapping `canonical_tenant_id → hashed_tenant_id` is recoverable only by OpenBao's tenant-resolver service.

### Reserved tenant IDs

The following tenant IDs are RESERVED and never issued as customer tenant IDs. Postgres + tenancy enforces this at issuance.

| Reserved tenant | Purpose | Write authority | Read authority |
|---|---|---|---|
| `tenant:oya-ci` | Promotion-readiness CI lane + governance lanes | tenancy-isolation-policy-worker SPIFFE | CI runners via short-lived OpenBao-issued keys |
| `tenant:oya-self` | tenancy µservice self-observability | tenancy platform components | tenancy platform operators |
| `tenant:oya-aggregate` | Anonymised cross-pack aggregates (DP-noise) | tenancy aggregator job | Public dashboards (non-sensitive) |
| `tenant:oya-system` | System-internal seed data (RLS policy templates, plan-tier defaults) | tenancy migration runner | tenancy-internal reads only |

Any operation referencing `tenant:oya-*` from a source other than the authorised SPIFFE identity is **rejected at the tenant-lifecycle-rest** layer with HTTP 403 + audit-emit `oya_tenancy_reserved_id_violation_total` (alert on > 0 over 5m).

### Tenant scope enumeration

```yaml
tenant_scope:
  enum: [trial, production, sandbox, internal]
  description: |
    - trial: pre-paid evaluation; 30-day TTL; capped capacity
    - production: paying customer; SLA-bound; primary-region pinned per pack
    - sandbox: customer-owned non-prod; isolated from prod-tier alerting
    - internal: oyatie's own services dogfooding
```

`tenant_scope` drives capacity-allocator + retention-policy decisions but does NOT relax isolation boundaries — every tenant scope shares identical RLS invariants.

## Postgres RLS Invariants

### Invariant RLS-01: `FORCE ROW LEVEL SECURITY` on every tenant-bound table

Every Postgres table carrying tenant-scoped rows MUST have:

```sql
ALTER TABLE <table> ENABLE ROW LEVEL SECURITY;
ALTER TABLE <table> FORCE ROW LEVEL SECURITY;
```

The `FORCE` modifier ensures RLS applies even to the table owner; without `FORCE`, table-owner connections (often the app role in lax setups) bypass RLS.

CI lane `oya-governance-rls-force-on-tenant-tables` validates this at PR-time: any migration creating a tenant-bound table without `FORCE ROW LEVEL SECURITY` fails the lane.

### Invariant RLS-02: Per-table CREATE POLICY

Every tenant-bound table has at least one policy of the shape:

```sql
CREATE POLICY tenant_isolation
  ON <table>
  USING (tenant_id = current_setting('app.current_tenant_id')::text);
```

Properties:
- The predicate references `current_setting('app.current_tenant_id')` (not a JWT directly; tenancy's adapter layer sets the setting via `SET LOCAL` on connection checkout, after JWT verification).
- The predicate is `text =` comparison (not bytea / integer); avoids unintended coercion.
- One policy named `tenant_isolation` per table (consistent naming for auditability).
- Additional policies (e.g., per-role read-only views for auditors) layer on top with explicit policy names; never override or relax the `tenant_isolation` policy.

CI lane `oya-governance-rls-policy-shape` (lightweight): asserts every tenant-bound table has a policy named `tenant_isolation` with the canonical predicate.

### Invariant RLS-03: `SET LOCAL app.current_tenant_id = $1` on every connection checkout

The tenancy adapter layer (and every workload µservice's data adapter) MUST execute:

```sql
SET LOCAL app.current_tenant_id = $1;
```

at the start of every transaction, where `$1` is the JWT-derived tenant_id. The connection-pool checkout hook emits this statement before yielding the connection to application code.

- `SET LOCAL` (not `SET`) ensures the setting is scoped to the transaction; transaction commit/rollback clears it.
- The setting is parameterised; SQL injection in tenant_id is impossible.

Code pattern (Rust + sqlx; canonical):

```rust
async fn checkout_tenant_scoped(pool: &PgPool, tenant_id: &TenantId) -> Result<PoolConnection, ...> {
    let mut conn = pool.acquire().await?;
    conn.execute(
        sqlx::query("SET LOCAL app.current_tenant_id = $1")
            .bind(tenant_id.as_str())
    ).await?;
    Ok(conn)
}
```

CI lane `oya-governance-tenant-context-setlocal-present`: AST-grep across adapter crates; refuses any DB-checkout path that doesn't emit `SET LOCAL app.current_tenant_id`.

### Invariant RLS-04: No `bypassrls` on the app role

The Postgres app role used by all workload µservices' adapters MUST NOT carry the `bypassrls` attribute. Verified via:

```sql
SELECT rolname, rolbypassrls FROM pg_roles WHERE rolname = 'tenancy_app';
-- expected: rolbypassrls = false
```

Only the `tenancy-admin-jit` role (issued via OpenBao JIT elevation; 2-person rule; ≤1h TTL; audit-chain-sealed) carries `bypassrls`, and it is used only by:
- The migration runner during schema evolution (limited to migration time; never application traffic).
- DBA emergency interventions (extreme; audit-chain emission + post-incident review).

CI lane `oya-governance-rls-no-superuser-bypass` (NEW; load-bearing): AST-grep refuses:
- `SET ROLE postgres` in any tenancy-adjacent crate (including all µservices that consume tenant-scoped data).
- `SET LOCAL row_security = off` anywhere.
- Direct `bypassrls`-flagged connection setup.
- Any `psql -U postgres` or analogous superuser-context shell command in application code.

### Invariant RLS-05: Migrations enforce RLS at creation time

The tenancy schema-migration runner (sqlx + custom validator) refuses to commit a migration that:
- Creates a tenant-bound table without `ENABLE ROW LEVEL SECURITY`.
- Creates a tenant-bound table without `FORCE ROW LEVEL SECURITY`.
- Creates a tenant-bound table without at least one `CREATE POLICY` referencing `app.current_tenant_id`.
- Creates a column named `tenant_id` on a non-tenant-bound table without explicit waiver in `microservices/tenancy/policy/rls/waivers.yaml`.

Tenant-bound is determined by a declarative manifest at `microservices/tenancy/policy/rls/<table>.yaml`:

```yaml
table: workflow.workflows
tenant_bound: true
tenant_id_column: tenant_id
policies:
  - name: tenant_isolation
    using: "tenant_id = current_setting('app.current_tenant_id')::text"
    force: true
data_class: BEHAVIORAL_TENANT_PRODUCT
owner_microservice: workflow
```

### Invariant RLS-06: Continuous DB-state validator

A `tenancy-rls-state-validator` cron (every 5min) compares the declarative YAML manifests against live `pg_policies` + `pg_class.relrowsecurity` + `pg_class.relforcerowsecurity`:
- Any drift (live policy missing / not forced / predicate mismatch) → emits `oya_tenancy_rls_drift_total{table=<name>}` metric → fires Sev-1 page + auto-rollback via ArgoCD to last-green Helm/manifest state.
- The validator's own metric is monitored; validator-down for ≥ 2min triggers Sev-2 (gate fails-closed for any RLS-mutating PR until validator recovers).

### Invariant RLS-07: No client-side filter substitutes for RLS

`WHERE tenant_id = $1` selectors in app code are **advisory**. The server enforces the actual scope. Even if a query omits the tenant filter, RLS at row level blocks cross-tenant rows.

This invariant exists to prevent the class of bug where a client-side library is the only line of defence.

### Invariant RLS-08: Reserved-namespace write authority

Tables in schemas matching the prefixes below have **write authority restricted** to specific tenancy components:

| Schema prefix | Authorised writer (SPIFFE identity) | Purpose |
|---|---|---|
| `tenancy.*` | `spiffe://oyatie/tenancy/*` only | tenant metadata, lifecycle, cell-assignment |
| `audit_chain.*` | `spiffe://oyatie/audit-chain/*` only | sealed audit records |
| `system.*` | `spiffe://oyatie/tenancy/migration-runner` only | system-seed data |

Non-reserved schemas (everything else; per-workload µservice schemas) follow normal per-tenant scope; tenant-owned data lives in tenant-scoped tables in those schemas.

## JWT Validation Invariants

Complement to RLS — JWT validation gates application admission BEFORE the connection is checked out. RLS is the last line of defence; JWT is the first.

### Invariant JWT-01: Algorithm-confusion defence

Verifiers MUST accept only `alg=EdDSA`; refuse `alg=none`, `alg=HS*` (HMAC), and `alg=RS*` (RSA) by explicit whitelist.

### Invariant JWT-02: Issuer + audience binding

JWT `iss` claim must equal `oya-tenancy-<pack>-<env>`; `aud` claim must equal `oyatie-internal`. Mismatch returns 401 + audit-emit.

### Invariant JWT-03: Expiration window

JWT `exp` ≤ 1h from issuance. Refresh tokens via separate path with stricter binding.

### Invariant JWT-04: Signing-key rotation

JWT signing keys rotated every 30d (or on compromise suspicion). Old pubkey valid for 30d grace to verify in-flight tokens. Rotation event emits `JwtSigningKeyRotated` Workflow event + audit-chain seal.

CI lane `oya-governance-jwt-key-fingerprint-advertised`: refuses key rotation without fingerprint event.

## Cedar Policy Enforcement

Cedar policies (per ADR-0140) layer on top of RLS + JWT to enforce role-based + scope-based authorisation:

- `policy/tenant-scope.cedar`: tenant operators can act only on their own tenant.
- `policy/ci-scope.cedar`: CI principals can only touch reserved tenants.
- `policy/auditor-scope.cedar`: auditors are tenant-scoped + window-bound + read-only.
- `policy/public-read.cedar`: anonymous reads are limited to specific public-class resources.

Cedar evaluator runs in `tenant-lifecycle-rest` and `dsr-cascade-rest`; non-matching requests return 403 + emit `oya_tenancy_cedar_deny_total`.

## Failure Modes (cross-ref `failure-modes.md`)

### FM-01: RLS policy drift (live state diverges from declared YAML)

**Behaviour:** Continuous validator detects drift; emits `oya_tenancy_rls_drift_total > 0`; fires Sev-1 page; auto-rollback via ArgoCD.

**Tenant impact:** Brief window between drift and detection (≤5min); RLS still effective if `FORCE` is intact (which validator ensures).

**Detection:** `oya-tenancy-rls-state-validator` cron metric.

**Recovery:** Auto-rollback to last green state; ops-security incident; root-cause analysis of cause (CI lane evasion? live DB mutation by JIT-elevated DBA?).

### FM-02: Migration runner skips RLS DDL

**Behaviour:** Activation worker post-migration validator detects missing RLS; emits `TenantActivationFailed`; tenant remains in `Created` state (not activated); ops paged.

**Tenant impact:** Activation blocked; no data exposure (table never accessible to traffic until activation succeeds).

**Detection:** Activation post-check.

**Recovery:** Migration runner re-executed with manual oversight; if persistent, RLS YAML manifest correctness verified.

### FM-03: JWT verifier accepts forged token

**Behaviour:** Algorithm-confusion test (T-T-02) fails synthetic forgery; would catch in pen-test.

**Tenant impact:** Theoretical cross-tenant exposure; mitigated by JWT-01 invariant + alg whitelist + unit tests in every verifier.

**Detection:** Pen-test; synthetic forgery drill.

**Recovery:** Verifier patch; emergency-merge sign-off; tenant impact assessment per breach-notification chain if exposure occurred.

### FM-04: Postgres role `bypassrls` set on app role

**Behaviour:** Weekly role-attribute audit detects; Sev-1 page; emergency role-revoke + investigation.

**Tenant impact:** Cross-tenant exposure window between set time and detection (≤7d worst case; instrumented to ≤1h via real-time event hook).

**Detection:** Weekly `oya-tenancy-postgres-role-audit` job + real-time pg_event_trigger on `ALTER ROLE`.

**Recovery:** Revoke `bypassrls`; engage ops-security; breach-notification chain if exposure confirmed.

### FM-05: Cell-assignment cache poisoning

**Behaviour:** Misroutes tenant to wrong shard; RLS still blocks rows; no exposure.

**Tenant impact:** Possible request errors during inconsistent state; no data leak.

**Detection:** Valkey-vs-Postgres state validator; 1min cadence.

**Recovery:** Cache invalidation; root-cause check on cache write path.

## Audit Trail

Every isolation boundary event is audit-chain-emitted per Bominal ADR-0028:

| Event | Emitter | Fields | Retention |
|---|---|---|---|
| RLS policy install (CREATE POLICY) | tenancy migration runner | `table, policy_name, predicate, force, applied_by, change_id` | ≥ 7y (or longer per pack legal; HIPAA 6y; KR-FSS 5y) |
| RLS policy drift detected | tenancy rls-state-validator | `table, expected_policy, live_policy, detected_at` | ≥ 7y |
| RLS policy drift auto-rollback | ArgoCD + tenancy | `table, prior_state, restored_state, applied_at` | ≥ 7y |
| Postgres role attribute mutation | tenancy postgres-role-audit + pg_event_trigger | `role, attribute, prev, new, mutated_by, mutated_at` | indefinite |
| Tenant spoofing attempt (JWT forgery) | tenancy isolation-policy-rest | `attempted_tenant_id, source_ip, source_spiffe, request_id, timestamp` | ≥ 1y |
| Reserved-tenant operation attempt | tenancy tenant-lifecycle-rest | `attempted_tenant_id, source, action, timestamp` | ≥ 1y |
| JWT signing-key rotation | tenancy isolation-policy-worker + OpenBao | `prev_fingerprint, new_fingerprint, rotated_by, timestamp` | indefinite |
| `tenancy-admin-jit` role issuance | OpenBao tenant-resolver | `role, granted_to, ttl, requestor, timestamp` | ≥ 1y |
| Cedar policy deny | tenant-lifecycle-rest / dsr-cascade-rest | `principal_id, action, resource, policy_fragment, timestamp` | ≥ 1y |
| Deployment salt rotation | OpenBao + tenancy admin | `prev_salt_hash, new_salt_hash, rotated_by, timestamp` | indefinite |

The audit log itself stored in audit-chain µservice (Merkle-sealed) + replicated to per-pack Postgres for redundancy. Audit-of-audits: every audit-log read is itself audited.

## Per-Pack Overlay

### pack-kr (KR PIPA + ISMS-P)

- KR PIPA Art. 29 (technical safeguards) maps to RLS-01..RLS-08 + JWT-01..JWT-04 + Cedar enforcement.
- Audit log retention: ≥ 1y per PIPA Enforcement Decree Art. 30; extended to 3y for `tenant_scope: production` (KR-FSS sectoral guidance) and 5y for KR-FSS-regulated tenants.
- KR PIPA Art. 23 (sensitive personal information) — hashed `tenant_id` with auxiliary is sensitive; salt rotation per Art. 29.
- KR PIPA Art. 23-2 (cross-border sensitive transfer): pack-kr enforced.
- KR PIPA Art. 29-2 (encryption requirement): TLS 1.3 in transit + AES-256-GCM at rest + Ed25519 audit-chain seals.

### pack-us-healthcare (HIPAA)

- §164.312(a)(1) (access control) mapping: RLS-01..RLS-08 + JWT-01..JWT-04 + Cedar.
- §164.312(b) (audit controls) mapping: full Audit Trail table; retention ≥ 6y per §164.316(b)(2).
- §164.312(c)(1) (integrity): Ed25519 audit-chain seals on every state transition.
- §164.312(e)(1) (transmission security): mTLS + TLS 1.3 for all inter-service traffic.
- §164.502(b) (minimum necessary): RLS enforces per-tenant minimum-data access.
- §164.514 (de-identification): hashed tenant_id pattern; pseudonymisation default.
- BAA with Covered Entity tenants documented in `legal/baa-template.md`.

### pack-eu (GDPR + EDPB)

- Art. 32(1)(a) pseudonymisation: hashed `tenant_id` is pseudonymous; canonical-tenant-id never exposed to operators / auditors via UI.
- Art. 32(1)(b) confidentiality + integrity: enforced via RLS-01..RLS-08 + audit-chain.
- Art. 32(1)(c) availability: enforced via Patroni HA + per-tenant rate limits.
- Art. 32(1)(d) regular testing: annual pen-test + quarterly chaos drill + weekly synthetic cross-tenant probe.
- Art. 25 by design and default: RLS + JWT + pseudonymisation default-enabled.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Each pack's overlay at `regional-packs/<pack>/tenancy-rls-overlay.md` maps the local PII law's confidentiality + integrity requirements to RLS-01..RLS-08.

## Verification

- `cargo run -p oya-dev-cli -- gate validate rls-no-superuser-bypass --microservice tenancy` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate rls-force-on-tenant-tables --microservice tenancy` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate tenant-context-setlocal-present --microservice tenancy` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate cedar-fragment-coverage --microservice tenancy` — exit 0.
- Weekly synthetic cross-tenant probe: tenant-A authenticated, attempts cross-tenant read of tenant-B's rows; expected: zero rows returned across all paths.
- Quarterly chaos drill: induce RLS drift (controlled); verify validator + auto-rollback fire within 5min.
- Annual pen-test against RLS boundary: scheduled Q4 of each calendar year (October 1 cycle) coinciding with ISO 27001 surveillance audit; documented in `runbooks/rls-pentest.md`.

## References

- ADR-0018 (Bominal): tenancy + RLS posture; inherited.
- ADR-0028 (Bominal): audit-chain.
- ADR-0117: cloud-native infrastructure + residency.
- ADR-0139: agentic SLO-gated promotion.
- ADR-0131: per-microservice flat layout.
- ADR-0140: Cedar policy enforcement.
- `microservices/tenancy/threat-model.md` §"Trust Boundaries" + T-T-01 + T-I-01 + T-I-02 + T-E-01.
- `microservices/tenancy/dpia.md` R-01.
- `microservices/tenancy/policy/{tenant-scope, ci-scope, auditor-scope, public-read}.cedar`.
- `microservices/tenancy/policy/data-residency.md`.
- Postgres RLS docs — `postgresql.org/docs/16/ddl-rowsecurity.html`.
- Citus multi-tenant docs — `docs.citusdata.com/en/stable/use_cases/multi_tenant.html`.
- OWASP Top 10 (2021) #1 Broken Access Control; #3 Injection.
- NIST SP 800-53 AC-3 (access enforcement).
