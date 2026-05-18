---
doc_class: PolicySpec
title: Type-Isolation Specification (per-tenant Ontology scope)
microservice: ontology
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-ontology
deciders: council-architecture, ops-security, axis-ontology, council-privacy
related_adrs: [ADR-0006, ADR-0028, ADR-0059, ADR-0106, ADR-0117, ADR-0122, ADR-0131, ADR-0132, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - microservices/ontology/threat-model.md (Trust Boundaries 2-3, T-S-01, T-I-01, T-I-02, T-I-03, T-E-01, T-E-02)
  - microservices/ontology/dpia.md (R-01, R-02, R-15)
  - microservices/ontology/policy/tenant-scope.cedar
  - microservices/ontology/policy/ci-scope.cedar
  - microservices/ontology/policy/auditor-scope.cedar
  - microservices/ontology/policy/public-read.cedar
review_cadence: quarterly + on every Postgres/Citus/ClickHouse/Cedar upgrade
doc_status: published
---

# Type-Isolation Specification (ontology µservice)

## Purpose

Define the load-bearing tenant- + pillar-isolation invariants of the Ontology substrate. Authoritative reference for SOC 2 (CC6.1 / CC6.2 / CC6.6), ISO 27001 (A.5.15 / A.8.2 / A.8.3 / A.8.12 / A.8.32), GDPR Art. 32, KR PIPA Art. 23 / 29, and HIPAA §164.312(a)(1) reviewers asking *"how does Ontology prevent tenant-A from seeing tenant-B's typed entities + how does it prevent org-pillar data from reaching person-pillar contexts?"*

## Tenant Identity Model

### Tenant ID derivation

```text
canonical_tenant_id   = <opaque-string-issued-at-onboarding>  (NOT stored in ontology)
salted_tenant_id      = sha256(canonical_tenant_id ++ deployment_salt)[..32]
app_tenant_id         = "tenant:" + salted_tenant_id          (bound to Postgres session var)
```

Properties:
- `canonical_tenant_id` is the OpenBao-bound subject; ontology never receives the raw value.
- `deployment_salt` is per-Ontology-cluster secret; rotated every 12 months; rotation audit-chained per Bominal ADR-0028.
- 32-hex truncation = 128 bits of entropy; collision-free at foreseeable scale.
- Tenant resolver service (in tenancy µservice) returns the salted form; cached in Valkey with TTL 60 s.

### Pillar context

Per Bominal ADR-0132 (pillars). Every principal carries one of:

| Pillar | Subject | Cedar entity reference |
|---|---|---|
| `org-pillar` | Organisation-owned data (Order, Invoice, Workflow run, etc.) | `Pillar::"org"` |
| `person-pillar` | Person-owned data (Patient, Employee, Customer profile, etc.) | `Pillar::"person"` |

Property-tier matrix (Bominal ADR-0008 / ADR-0119):

| Tier | Default pillar | Example property |
|---|---|---|
| `Tier1Sensitive` | org or person | SSN, medical record, payment card |
| `Tier2Restricted` | org or person | transaction amount, IP address |
| `Tier3Internal` | org | internal metadata, timestamps |
| `Tier4Public` | org | published catalog, public profile |

Cross-pillar reads require an explicit Cedar `CrossPillarGrant` issued via 2-person rule.

### Reserved principals

| Reserved principal | Purpose | Read authority | Write authority |
|---|---|---|---|
| `Service::"ontology-action-engine-worker"` | Action engine internal | reads to evaluate; audit-chained | writes only via permitted Action Types |
| `Service::"ontology-dsr-cascade-runner"` | DSR Art. 17 cascade | reads every Object Type table to scan for subject identifier | tombstones (DELETE + audit-chain) via DSR Action Type |
| `Service::"ontology-audit-chain-worker"` | Outbox-to-Kafka emit + audit seal | reads outbox table; writes audit-chain Merkle | none beyond audit |
| `Service::"ontology-schema-propagation-worker"` | Schema hot-reload | reads schema registry; emits SchemaRegistered events | none beyond schema events |
| `Auditor::"<engagement-id>"` | External auditor JIT | read-only on policy artifacts + scoped tenant subset | none |

Any inbound write attempting to use a reserved principal name from outside the in-cluster SPIFFE identity is rejected at the gateway with HTTP 403 and emits `oya_ontology_reserved_principal_violation_total` (alert on > 0 over 5 m).

## Postgres + Citus Isolation Invariants

### Invariant TI-01: `FORCE ROW LEVEL SECURITY` on every Object Type table

Every Postgres table that stores Object Type instances declares:

```sql
ALTER TABLE <object_type> ENABLE ROW LEVEL SECURITY;
ALTER TABLE <object_type> FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON <object_type>
  USING (tenant_id = current_setting('app.tenant_id')::TEXT)
  WITH CHECK (tenant_id = current_setting('app.tenant_id')::TEXT);
```

`FORCE` is required: without it, the table owner bypasses RLS, which would be a path for the application's Postgres role to read across tenants.

The CI lane `oya-foundry-fitness-ontology-tenancy-isolation` validates every Object Type table for `FORCE ROW LEVEL SECURITY = true`; PRs that introduce a table without it fail to merge.

### Invariant TI-02: `app.tenant_id` session variable bound from JWT only

The application's middleware extracts `tenant_id` from the validated JWT claim (signed by OpenBao) and binds it to `app.tenant_id` per connection:

```rust
// Pseudo:
let tenant_id = jwt.claims.tenant_id; // bound by OpenBao at token issuance
conn.execute("SET LOCAL app.tenant_id = $1", &[&tenant_id]).await?;
// ... subsequent queries on this connection are RLS-scoped ...
```

LEAN check `oya-foundry-fitness-ontology-tenant-binding` greps every adapter for any code path that sets `app.tenant_id` from anything other than `req.auth.tenant_id`; mismatch fails the lane.

### Invariant TI-03: Citus `multi_shard_modify_mode = strict`

```yaml
# microservices/ontology/iac/helm/postgres/values.yaml
citus:
  config:
    multi_shard_modify_mode: strict
    citus.shard_count: 32  # per pack default
    citus.shard_replication_factor: 3
```

Strict mode forbids cross-shard write queries; this prevents an accidental cross-tenant write via misrouted shard. The CI lane validates the Helm chart values + asserts no override in any overlay.

### Invariant TI-04: Per-tenant Postgres role with bounded grants

Each tenant has a Postgres role (issued by OpenBao at onboarding):

```sql
CREATE ROLE tenant_<salted_tenant_id> NOLOGIN;
GRANT USAGE ON SCHEMA ontology TO tenant_<salted_tenant_id>;
-- RLS restricts the role's view; no GRANT ALL on tables.
```

The application's Postgres connection pool uses session-level role switching via `SET ROLE` after the JWT verification, scoping every query.

### Invariant TI-05: No raw cross-tenant SQL in application paths

A LEAN check (`oya-foundry-fitness-no-raw-sql-cross-tenant`) refuses any SQL string in adapter code that:
- contains `WHERE tenant_id = ?` AND `tenant_id` ≠ `app.tenant_id` session var
- contains `tenant_id IN (...)` lists with > 1 element
- bypasses the adapter's `RLS-scoped` connection pool

Bypass is allowed only for the DSR cascade runner and audit-chain worker via explicit `SECURITY DEFINER` PL/pgSQL functions whose body is reviewed by CODEOWNERS (ops-security + axis-ontology).

### Invariant TI-06: Per-tenant Object Type instance budget + write rate

| Limit | Default | Configurable per tenant_scope | Enforcement |
|---|---|---|---|
| Max Object Type instances per tenant | 1 M | trial: 100 k; production: 1 M; sandbox: 50 k; internal: 10 M | Citus shard fill alarm; per-tenant budget table |
| Max Action invocations / min | 60 k | trial: 6 k; production: 60 k; sandbox: 3 k; internal: 600 k | action-engine rate limiter |
| Max Function reads / sec | 1 k | trial: 100; production: 1 k; sandbox: 50; internal: 10 k | function-engine rate limiter |

Excess returns HTTP 429. Per-tenant overage metric (`oya_ontology_tenant_rate_limit_exceeded_total`) feeds the tenant's own dashboard.

### Invariant TI-07: Per-tenant read authorization via Cedar

Every read request (REST, SDK, agent gateway) passes through a Cedar policy evaluator (per ADR-0140) that enforces:

```cedar
permit (
  principal,
  action in [
    Action::"read_object_type",
    Action::"read_link_type",
    Action::"read_function_result",
    Action::"read_action_receipt",
    Action::"read_audit_chain"
  ],
  resource is TenantData
) when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal.max_tier >= resource.property_tier &&
  principal.pillar_kind == resource.pillar_kind ||
  principal has cross_pillar_grant &&
  resource.pillar_kind in principal.cross_pillar_grant.allowed_pillars
};
```

(Full fragment at `policy/tenant-scope.cedar`.) The evaluator runs in every `*-rest` crate; non-matching reads return 403 + emit `oya_ontology_tenant_unauthorized_read_attempt_total`.

## ClickHouse Mirror Isolation Invariants

### Invariant TI-08: ClickHouse row policies

```sql
CREATE ROW POLICY tenant_filter ON ontology.object_type_history
  FOR ALL TO ALL
  USING tenant_id = getSetting('app_tenant_id');
```

ClickHouse row policy is the equivalent of Postgres RLS at the OLAP mirror. Set at every materialised table that mirrors an Object Type.

### Invariant TI-09: Outbox-to-Kafka emits only RLS-cleared rows

The outbox table itself has RLS enabled; the outbox worker reads with `SET LOCAL app.tenant_id` bound to the row's `tenant_id` (the writer's authenticated tenant); cross-tenant outbox emit is impossible.

### Invariant TI-10: Per-tenant ClickHouse query budget

Per-tenant `max_memory_usage`, `max_execution_time` settings prevent one tenant from monopolising query resources.

## Pillar Isolation Invariants

### Invariant TI-11: `pillar_kind` is part of `ObjectTypeSchema`

Every Object Type schema declares its `pillar_kind` (org / person) at registration. Cannot be changed without a schema-evolution PR + 2-person rule.

### Invariant TI-12: Pillar Cedar gate

```cedar
forbid (
  principal,
  action in [Action::"read_object_type", Action::"write_object_type", Action::"invoke_action"],
  resource is TenantData
)
unless {
  resource has pillar_kind &&
  (
    principal.pillar_kind == resource.pillar_kind ||
    (principal has cross_pillar_grant &&
     resource.pillar_kind in principal.cross_pillar_grant.allowed_pillars &&
     context.now <= principal.cross_pillar_grant.expires_at)
  )
};
```

(Full fragment at `policy/pillar.cedar`.) Cross-pillar grants are issued via 2-person rule; TTL ≤ 30 d unless explicitly renewed; audit-chained at issuance + renewal + revocation.

### Invariant TI-13: Cross-tenant Link Type requires explicit grant

The Link Type adapter checks both endpoints' `tenant_id` against `app.tenant_id`. Mismatch returns `CrossTenantLinkDenied` unless a `CrossTenantLinkGrant` is present in the Cedar context. Grants:
- Issued via 2-person rule.
- TTL ≤ 30 d.
- Carry explicit `data_class` ceiling (e.g., grant only permits Tier3Internal links).
- Audit-chained at issuance + use + expiry.

## Failure Modes (excerpt; full list in `failure-modes.md`)

### FM-IS-01: Postgres RLS drift (live mutation disables RLS on a table)

- Behaviour: Postgres roles for application paths cannot disable RLS; superuser JIT 2-person rule is the only path. Continuous Helm-state validator + `pg_dump --schema-only` diff CronJob detects drift hourly.
- Tenant impact: Caught pre-deploy. If runtime mutation: alarms within 1 h.
- Detection: `oya-foundry-fitness-ontology-tenancy-isolation` lane + drift detector.
- Recovery: Auto-rollback to last green Helm state; ops-security incident if intentional.

### FM-IS-02: Citus strict-mode disabled

- Behaviour: Lane refuses merge; live mutation alarms via continuous Helm-state validator.
- Detection: same as FM-IS-01.
- Recovery: same.

### FM-IS-03: Cross-pillar grant issued without 2-person rule

- Behaviour: Cedar `pillar.cedar` requires `signed_by_two_principals` claim; missing → grant invalid.
- Detection: Cedar evaluation returns deny on use; audit-chained alarm fires.
- Recovery: Revoke grant; investigate; root-cause.

### FM-IS-04: Cross-tenant Link Type creation attempt

- Behaviour: Link adapter checks both endpoints; mismatch returns 403; alarm fires.
- Detection: `oya_ontology_cross_tenant_link_denied_total > 0` over 1 min.
- Recovery: Trace caller; revoke API key if intentional.

## Audit Trail

Every cross-tenant or cross-pillar boundary event audit-chain-emitted per Bominal ADR-0028:

| Event | Emitter | Fields | Retention |
|---|---|---|---|
| Tenant spoofing attempt | gateway middleware | `attempted_tenant_id, source_principal, source_ip, source_spiffe_id, request_id, timestamp` | ≥ 1y |
| Reserved-principal misuse attempt | gateway | `principal_name, source_spiffe_id, attempted_action, timestamp` | ≥ 1y |
| Cross-tenant Link Type denied | link-store-adapter | `attempted_link_id, src_tenant, dst_tenant, principal, timestamp` | ≥ 1y |
| Cross-pillar grant issued | pillar.cedar + 2-person rule | `grant_id, principal, allowed_pillars, data_class_cap, expires_at, signed_by[2]` | indefinite |
| Cross-pillar grant used | pillar evaluator | `grant_id, used_at, target_resource_id` | ≥ 1y |
| DSR erasure executed | dsr-cascade-runner | `subject_hash, removed_object_type_count, residual_object_types[], executed_at` | ≥ 1y |
| Deployment salt rotated | ops-security + OpenBao | `prev_salt_hash, new_salt_hash, rotated_by, timestamp` | indefinite |
| Postgres superuser session opened | OpenBao JIT | `principal, ticket_id, opened_at, expires_at` | ≥ 2y |
| Cedar fragment hot-reload | cedar-fragment-coverage | `fragment_path, prev_hash, new_hash, applied_at` | ≥ 2y |

The audit log is itself persisted in the audit-chain Postgres table within Ontology (sealed via Merkle/Ed25519) and mirrored to the platform `audit-chain` µservice for chain-of-chains sealing.

## Per-Pack Overlay

### pack-kr (KR PIPA + ISMS-P)

- KR PIPA Art. 29 (technical safeguards) maps to TI-01..TI-13.
- Audit log retention: ≥ 1y per PIPA Enforcement Decree Art. 30; extended to 3y for production-tier per KR-FSS sector guidance.
- KR PIPA Art. 23 (sensitive personal information): hashed subject ID with auxiliary is sensitive; salt rotation per Art. 29.

### pack-us-healthcare (HIPAA)

- §164.312(a)(1) (access control) mapping: TI-01..TI-13.
- §164.312(b) (audit controls) mapping: Audit Trail table.
- §164.312(e)(1) (transmission security): mTLS + TLS 1.3 for all inter-service traffic.
- §164.502(b) (minimum necessary): tenant-scope + pillar-scope + tier-filter enforce least-data.
- Audit log retention extended to ≥ 6y per §164.316(b)(2).

### pack-eu (GDPR + EDPB)

- Art. 32(1)(a) pseudonymisation: `app.tenant_id` is salted hash; canonical-tenant-id never exposed.
- Art. 32(1)(b) confidentiality + integrity: TI-01..TI-13 + audit-chain.
- Art. 32(1)(c) availability: Postgres HA + per-tenant rate limits.
- Art. 32(1)(d) regular testing: annual pen-test + quarterly chaos drill.
- Art. 25 (by design + default): RLS + pillar + Cedar default-enabled.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/type-isolation-overlay.md` map the local PII law's confidentiality + integrity requirements to TI-01..TI-13.

## Verification

- `cargo run -p oya-dev-cli -- gate validate ontology-tenancy-isolation` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate ontology-tier-enforcement` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate cedar-coverage --microservice ontology` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate no-raw-sql-cross-tenant --microservice ontology` — exit 0.
- Annual pen-test against tenant boundary + pillar boundary: `runbooks/type-isolation-pentest.md`.
- Quarterly chaos drill: induce reserved-principal misuse + cross-tenant link attempt + cross-pillar grant abuse; verify rejection + alerting.

## References

- ADR-0006: Ontology typed-entity layer.
- ADR-0028 (Bominal): Audit chain.
- ADR-0059: Workflow + Ontology adapter layer.
- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0131: Per-microservice flat layout.
- ADR-0132 (Bominal): Pillars.
- ADR-0140: Cedar policy enforcement.
- `microservices/ontology/threat-model.md` §"Trust Boundaries" + T-S-01, T-I-01, T-I-02, T-I-03, T-E-01.
- `microservices/ontology/dpia.md` R-01, R-02, R-15.
- `microservices/ontology/policy/{tenant-scope, ci-scope, auditor-scope, public-read, pillar}.cedar`.
- `microservices/ontology/policy/data-residency.md`.
- Postgres RLS docs — `postgresql.org/docs/16/ddl-rowsecurity.html`.
- Citus multi-tenancy docs — `docs.citusdata.com`.
- Cedar v4 — `cedarpolicy.com`.
- SPIFFE / SPIRE — `spiffe.io`.
- OpenBao — `openbao.org`.
