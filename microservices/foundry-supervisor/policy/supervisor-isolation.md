---
doc_class: PolicySpec
title: Supervisor Isolation (Per-Tenant Fleet Boundaries)
microservice: foundry-supervisor
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-foundry-control-plane
deciders: council-architecture, ops-security, axis-foundry-control-plane, council-privacy
related_adrs: [ADR-0028, ADR-0117, ADR-0130, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/foundry-supervisor/threat-model.md (T-I-01, T-S-01, T-T-02, T-E-01)
  - microservices/foundry-supervisor/dpia.md (R-01, R-02, R-14)
  - microservices/foundry-supervisor/policy/tenant-scope.cedar
  - microservices/foundry-supervisor/policy/ci-scope.cedar
  - microservices/foundry-supervisor/policy/auditor-scope.cedar
  - microservices/foundry-supervisor/policy/public-read.cedar
review_cadence: quarterly + on every Postgres/Redis/Operator version upgrade
doc_status: published
---

# Supervisor Isolation (foundry-supervisor µservice)

## Purpose

Define the load-bearing per-tenant fleet-boundary invariants of the foundry-supervisor control plane. This document is the authoritative reference for SOC 2 examiners (CC6.1 / CC6.2 / CC6.6), ISO 27001 auditors (A.5.15 / A.8.2 / A.8.3 / A.8.12), GDPR Art. 32 reviewers, EU AI Act Art. 14 (human oversight) reviewers, KR PIPA Art. 23 / Art. 29 reviewers, and HIPAA §164.312(a)(1) reviewers asking *"how does the supervisor prevent tenant-A's capabilities from acting on tenant-B's fleet?"*

## Tenant Identity Model

### Tenant ID derivation

```text
canonical_tenant_id   = <opaque-string-issued-at-onboarding>   (NOT stored in foundry-supervisor; held in OpenBao)
hashed_tenant_id      = sha256(canonical_tenant_id ++ deployment_salt)[..16]
postgres_tenant_id    = "tenant_" + hashed_tenant_id     (Postgres session variable; bounds RLS)
k8s_namespace         = "foundry-tenant-" + hashed_tenant_id   (per-tenant Kubernetes namespace)
spiffe_pattern        = "spiffe://oyatie/tenants/" + hashed_tenant_id + "/*"   (SPIFFE pattern for tenant SAs)
```

Properties:
- `canonical_tenant_id` is OpenBao-bound; supervisor never receives the raw value.
- `deployment_salt` is per-pack secret, rotated 12 months; rotation event audit-chain-emitted.
- `hashed_tenant_id` 16-hex (64-bit); collision-free namespace for 10⁶ tenants.

### Reserved tenant IDs

Reserved IDs never issued as customer tenant IDs. Supervisor admit-loop refuses any capability YAML whose `tenant_id` matches a reserved value.

| Reserved tenant | Purpose | Write authority | Read authority |
|---|---|---|---|
| `tenant_oya_ci` | CI lane writes (HG-FND-SUP-claim, integration tests) | `slo-engine-worker` SPIFFE + `oya-ci` SA | CI runners via short-lived OpenBao read keys |
| `tenant_oya_self` | Supervisor's own self-observability + self-SLO authoring | supervisor controller pods only | observability + auditor scope |
| `tenant_oya_aggregate` | DP-noise-cleaned cross-tenant aggregates (product metrics) | supervisor aggregator job (ε ≤ 1) | public-read.cedar |

Any inbound write with `tenant_id = tenant_oya_*` from a non-authorised SPIFFE identity is **rejected at REST + Postgres + Redis layers** with HTTP 403 + emits `oya_supervisor_reserved_tenant_violation_total > 0` (alert > 0 over 5m).

### Tenant scope enumeration

```yaml
tenant_scope:
  enum: [trial, production, sandbox, internal]
  description: |
    - trial: pre-paid eval; 30-day TTL; capped fleet size 10 agents; T0/T1 only
    - production: paying customer; SLA-bound; primary-pack pinned
    - sandbox: customer-owned non-prod; isolated from prod-tier alerting
    - internal: oyatie's own µservice dogfooding the supervisor
```

Tenant scope drives capacity + autonomy-tier ceiling but does NOT relax isolation invariants — every scope shares identical isolation invariants below.

## Postgres Isolation Invariants (TI-P-*)

### TI-P-01: Row-Level Security mandatory on every tenant-scoped table

Every Postgres table with a `tenant_id` column has RLS enabled:

```sql
ALTER TABLE fleet_state ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON fleet_state
  USING (tenant_id::text = current_setting('app.tenant_id', true));
```

`oya-check-postgres-rls-enforced` LEAN lane validates every tenant-scoped table has this policy at PR-time.

### TI-P-02: Per-request session variable

Supervisor REST + worker sets `app.tenant_id` per Postgres connection check-out from the PgBouncer pool:

```sql
SET LOCAL app.tenant_id = '<hashed-tenant-id-from-OIDC-claim>';
```

Without this set, RLS denies all rows. Per-request → per-connection isolation via pool checkout.

### TI-P-03: Per-pod credentials with bound-tenant claim (NONE)

Supervisor pods receive OpenBao-issued Postgres credentials with NO bound tenant — the per-request session variable provides the bound tenant. The credential carries the supervisor SA identity, not a tenant identity. (This pattern matches Stripe + Datadog control-plane Postgres usage.)

### TI-P-04: WAL archival encrypted + residency-pinned

Postgres WAL archives to S3-compatible bucket in the pack's region; SSE-KMS with pack-resident KMS keyring. Cross-pack WAL replication forbidden by `data-residency.md`.

### TI-P-05: Audit logs via PgAudit

PgAudit logs every DML against `AUDIT`-class tables (deployment_history, kill_switch_log, autonomy_violation_log). Audit-log volume itself is monitored.

### TI-P-06: Direct DB access locked behind JIT (2-person rule for `AUDIT`)

JIT elevation through OpenBao required for any `psql` access; 2-person rule for tables with `AUDIT`-class data. Every access audit-chain-emitted.

## Redis Isolation Invariants (TI-R-*)

### TI-R-01: ACL per-key pattern

Redis Cluster ACLs scope each connection to keys matching the bound tenant pattern:

```text
user supervisor on >$<openbao-token>
  ~tenant:*:*                # supervisor reads all kill-switch keys (broad; supervisor IS the boundary)
  ~oya-aggregate:*           # supervisor writes aggregate

user tenant-<hashed-id> on >$<per-tenant-bound-token>
  ~tenant:<hashed-id>:*      # tenant-scoped only
  -DELETE
  -FLUSHALL
  -FLUSHDB
```

Tenant-side subscribers receive only their own scoped streams.

### TI-R-02: AOF every-second + replication-factor 2

AOF persistence with `everysec` fsync; 3 shards × 2 replicas; loss of one replica per shard tolerated. Per Redis Cluster reference architecture.

### TI-R-03: Kill-switch state authoritative-truth in CRD (Redis is cache)

Redis kill-switch state is cache + sub-second propagation channel. The Kubernetes CRD (`KillSwitch` custom resource) is the source-of-truth. Periodic CRD-watch reconciliation overwrites Redis if divergence detected. Divergence alarm fires on `oya_kill_switch_state_divergence_total > 0`.

### TI-R-04: Fail-closed on Redis unavailability

When Redis is unreachable for > 2 s, supervisor returns "engaged" for all kill-switch queries (safe default). The runtime-side handshake refuses to invoke a capability with unknown kill-switch state. Recovery: CRD reconcile re-populates Redis.

## Kubernetes Isolation Invariants (TI-K-*)

### TI-K-01: Per-tenant namespace

Each tenant gets a dedicated Kubernetes namespace: `foundry-tenant-<hashed-id>`. The supervisor Operator watches across these namespaces with a label selector.

### TI-K-02: Operator RBAC scoped

The supervisor Operator's ClusterRole permits read/write on `Agent`, `AgentDeployment`, `AutonomyPolicy`, `KillSwitch` CRDs **only** in namespaces matching `^foundry-tenant-.*$`. Refuses access to `default`, `kube-system`, etc.

### TI-K-03: Admission webhook + OPA Gatekeeper

Mutations to supervisor CRDs require origination from the supervisor SA's SPIFFE identity. Manual `kubectl` edits are refused by the admission webhook. OPA Gatekeeper policy enforces this cluster-wide; LEAN check asserts policy is deployed.

### TI-K-04: NetworkPolicy

Per-tenant namespace has a NetworkPolicy: tenant agents may only reach the supervisor REST + foundry-runtime SVCs within their own namespace; cross-namespace traffic is denied except for the supervisor Operator and observability collector.

### TI-K-05: Drift detector

Nightly job compares K8s CRD state vs Postgres deployment history; divergence creates a Sev-2 incident.

## Cedar Per-Tenant Read Authorisation (TI-C-*)

### TI-C-01: Default-deny

`policy/tenant-scope.cedar` opens with `forbid (principal, action, resource);`. Every read requires an explicit permit.

### TI-C-02: principal.tenant_id == resource.tenant_id invariant

Every permit clause for tenant-scoped reads bounds `principal.tenant_id == resource.tenant_id` (Cedar v4 attribute equality).

### TI-C-03: Auditor scope is per-tenant + time-boxed

Auditor JIT tokens carry `scoped_tenants` claim; Cedar refuses reads outside this list. Token TTL ≤ 4 h.

### TI-C-04: CI scope is bounded to oya-ci / oya-self / oya-aggregate

CI principal cannot read customer-tenant data; bounded to reserved-tenant pattern only.

### TI-C-05: Public-read scope is bounded to PUBLIC data_class

Anonymous reads succeed only against resources with `data_class == "PUBLIC"`.

## Failure Modes

### FM-S-01: Postgres master loss

**Behaviour:** PgBouncer reroutes to replica (promoted); ≤ 30 s control-plane availability gap.

**Detection:** `postgres_master_unreachable` alarm + Patroni status.

**Recovery:** Replica promotion (automatic); ops-sre-reliability paged for forensic if pattern suggests compromise.

### FM-S-02: Redis cluster minor partition

**Behaviour:** Cluster mode tolerates 1 replica down per shard; 2 replicas down per shard → fail-closed.

**Detection:** `redis_cluster_health_status` alarm.

**Recovery:** Replica restart; AOF replay; CRD reconcile.

### FM-S-03: Operator pod crashloop

**Behaviour:** Lease-leadership election re-runs; standby controller pod takes over.

**Detection:** `kubernetes_operator_alive == 0` for ≥ 2 min.

**Recovery:** Pod restart; root-cause analysis.

### FM-S-04: Cedar policy drift

**Behaviour:** Cedar fragments versioned in git; deployment via Helm + LEAN lane; live drift is impossible because policy is loaded at pod startup from a ConfigMap.

**Detection:** `oya-check-cedar-fragment-coverage` lane refuses PR; Helm rollback restores prior state.

**Recovery:** Helm rollback + PR fix.

### FM-S-05: Cross-tenant fleet-state leak suspected

**Behaviour:** Pen-test attempts cross-tenant query; succeeds → Sev-1.

**Detection:** Pen-test scheduled annually; `oya_tenant_unauthorized_read_attempt_total > 0` triggers.

**Recovery:** ops-security incident; breach notification chain.

## Audit Trail

Every cross-tenant boundary event audit-chain-emitted per Bominal ADR-0028:

| Event | Emitter | Fields | Retention |
|---|---|---|---|
| RLS denial | Postgres + PgAudit | `principal_sa, attempted_tenant_id, table, query_hash, timestamp` | ≥ 1 y (HIPAA: 6 y) |
| Reserved-tenant write attempt | Supervisor REST | `attempted_tenant_id, source_spiffe_id, timestamp` | ≥ 1 y |
| Kill-switch state divergence | Supervisor Worker | `divergence_scope, redis_state, crd_state, timestamp` | ≥ 1 y |
| Operator RBAC denial | K8s API | `principal_sa, attempted_verb, resource, namespace, timestamp` | ≥ 1 y |
| Cedar policy denial | Supervisor REST | `principal_id, requested_tenant_id, action, resource, timestamp` | ≥ 1 y |
| OpenBao credential issuance + revocation | OpenBao | `key_id, bound_subject, issued_by, ttl` | ≥ 1 y |
| Deployment salt rotation | OpenBao + supervisor admin | `prev_salt_hash, new_salt_hash, rotated_by, timestamp` | indefinite |

Audit log itself lives in observability Mimir under `tenant:oya-self` and is replicated to the `audit-chain` µservice for Merkle sealing.

## Per-Pack Overlay

### pack-kr (PIPA + ISMS-P)

- PIPA Art. 29 maps to TI-P-* + TI-R-* + TI-K-* + TI-C-*.
- Audit log retention: 3 y for `tenant_scope=production` (KR-FSS sector guidance).
- PIPA Art. 23 sensitive PI: autonomy entitlements stored in OpenBao with salt rotation.

### pack-us-healthcare (HIPAA)

- §164.312(a)(1) Access Control: TI-P-01 RLS + TI-C-* Cedar + TI-K-02 RBAC together satisfy.
- §164.312(b) Audit Controls: Audit Trail table.
- §164.502(b) Minimum Necessary: per-tenant scope enforced; cross-tenant queries impossible.
- Audit log retention: ≥ 6 y per §164.316(b)(2).
- BAA with Covered Entity tenants required pre-deploy.

### pack-eu (GDPR + EU AI Act)

- GDPR Art. 32(1)(a) pseudonymisation: hashed `tenant_id` never reverses without OpenBao.
- GDPR Art. 32(1)(b) integrity + confidentiality: TI-P-* + TI-R-* + audit-chain.
- GDPR Art. 32(1)(c) availability: fail-closed kill-switch satisfies "appropriate technical measures" for safety.
- GDPR Art. 25 PbD: default-deny Cedar + per-tenant ns by default.
- EU AI Act Art. 14 (human oversight): 2-person rule on fleet-wide kill-switch + tenant DPO can disengage own scope.
- EU AI Act Art. 15 (cybersecurity): TI-* invariants + supply-chain (cargo deny / Trivy).

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Each pack's overlay at `regional-packs/<pack>/foundry-supervisor-isolation-overlay.md`.

## Verification

- `cargo run -p oya-dev-cli -- gate validate postgres-rls-enforced --microservice foundry-supervisor` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate operator-rbac-conformance --microservice foundry-supervisor` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate cedar-fragment-coverage --microservice foundry-supervisor` — exit 0.
- Annual pen-test of tenant-boundary; documented in `runbooks/tenant-boundary-pentest.md`.
- Quarterly chaos drill: induce reserved-tenant write + cross-tenant query attempt; verify rejection + alerting.

## References

- ADR-0028, ADR-0117, ADR-0130, ADR-0131, ADR-0140.
- `microservices/foundry-supervisor/threat-model.md` T-I-01, T-S-01, T-T-02, T-E-01.
- `microservices/foundry-supervisor/dpia.md` R-01, R-02, R-14.
- `microservices/foundry-supervisor/policy/{tenant-scope, ci-scope, auditor-scope, public-read}.cedar`.
- `microservices/foundry-supervisor/policy/data-residency.md`.
- PostgreSQL RLS docs — `postgresql.org/docs/current/ddl-rowsecurity.html`.
- Redis Cluster spec — `redis.io/docs/management/scaling/`.
- Cedar v4 — `cedarpolicy.com`.
- SPIFFE / SPIRE — `spiffe.io`.
- OpenBao — `openbao.org`.
