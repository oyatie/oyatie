---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: cell
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-cell-substrate + ops-security
deciders: council-architecture, ops-security, axis-cell-substrate, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + NIST SP 800-154
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0139, ADR-0131, ADR-0140]
related_specs: [/specs/per-microservice-flat-layout.json]
review_cadence: quarterly + on every cell-substrate architecture change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.18, A.5.23, A.5.26, A.5.31, A.8.2, A.8.3, A.8.5, A.8.7, A.8.11, A.8.12, A.8.20, A.8.21, A.8.22, A.8.23"
  - "GDPR Arts. 5, 6, 9, 25, 28, 30, 32, 33"
suggested_frameworks_by_pack:
  pack-kr: ["KR-ISMS-P §2.5-2.10 (보호조치)", "KR PIPA Arts. 17/22-2/23/24/28/29"]
  pack-us-healthcare: ["HIPAA §164.308 (Administrative)", "§164.310 (Physical)", "§164.312 (Technical Safeguards)"]
  pack-eu: ["GDPR Arts. 25/32/35", "NIS2 (when applicable)"]
  pack-jp: ["APPI Arts. 17/18/20/23"]
  pack-sg: ["PDPA Part III"]
  pack-au: ["Privacy Act 1988 APP 1/6/8/11", "APRA CPS 234"]
  pack-in: ["DPDPA 2023 §6-10"]
  pack-br: ["LGPD Arts. 6/7/11/46"]
  pack-ae: ["UAE PDPL 45/2021 Arts. 5/6/9/15"]
  pack-ksa: ["PDPL M/19/2021 Arts. 4-9", "SAMA Cybersecurity Framework"]
doc_status: published
---

# Threat Model: cell µservice

## Purpose

Identify, classify, and mitigate threats to the cell substrate's confidentiality, integrity, availability, and privacy posture. The cell substrate is the load-bearing **hard tenant-isolation** primitive; a compromise here is the most severe class of incident in oyatie (Sev-1 always). This document is the canonical security artifact reviewed by SOC 2 Type 2 examiners, ISO 27001 auditors, GDPR DPAs, KR PIPC, and HIPAA OCR at first-tenant onboarding in each pack.

## Scope

### In-scope

All components introduced by this PRD across the 5 BCs (cell-registry, tenant-assignment, scheduler, lifecycle-manager, host-pool):

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| Kubernetes (per pack) | `oya-cell-cell-registry-*` (9 crates) |
| Kubernetes Cluster API (cell CRDs) | `oya-cell-tenant-assignment-*` (11 crates) |
| Postgres (per-pack registry shard) | `oya-cell-scheduler-*` (8 crates) |
| PgBouncer | `oya-cell-lifecycle-manager-*` (9 crates) |
| OCI Block / Object storage (cell-resident PV + prefix) | `oya-cell-host-pool-*` (8 crates) |
| SPIFFE / SPIRE (per-cell SVID issuance) | `registry/cell-assignment.jsonl` |
| Cedar v4 (boundary policy evaluator) | `oya-cell-boundary` LEAN lane |

### Out-of-scope

- Threats to the underlying hyperscaler IaaS — owned by `cloud-k8s` and `cloud-iac` threat models.
- Threats to OpenBao itself — owned by `cloud-secrets` threat model.
- Threats inside individual workload µservices (tenancy, ontology, …) — each owns its own model.
- Threats to the audit-chain signing infrastructure — owned by `audit-chain` threat model.

## Trust Boundaries

```text
┌─ External ──────────────────────────────────────────────────────────────────┐
│                                                                             │
│   Customer applications              Tenant operators                       │
│         │                                  │                                │
│         │ (per-tenant API key)             │ (OIDC + MFA)                   │
│         ▼                                  ▼                                │
│  ┌─ Public ingress (Envoy/Istio gateway) ───────────────────────────────┐   │
│  │   - TLS termination                                                  │   │
│  │   - Per-pack endpoint routing                                        │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                              │                                              │
└──────────────────────────────│──────────────────────────────────────────────┘
                               ▼
┌─ Per-pack cell-set ─────────────────────────────────────────────────────────┐
│                                                                             │
│  Trust boundary 1: External → cluster ingress                               │
│  Trust boundary 2: cell-registry-rest → cell-registry-adapter-postgres      │
│           (mTLS + Cedar policy + per-pack shard pin)                        │
│                                                                             │
│  ┌─ cell-registry-rest ─┐    ┌─ tenant-assignment-rest ─┐                   │
│  │  OIDC + Cedar policy │    │  OIDC + Cedar policy     │                   │
│  └──────────────────────┘    └──────────────────────────┘                   │
│             │                              │                                │
│             ▼                              ▼                                │
│  ┌─ Postgres (per-pack registry) ──────────────────────────────────────┐    │
│  │  - Logical-schema-per-cell isolation                                │    │
│  │  - Row-level-security on cell_assignments table                     │    │
│  │  - Per-cell SA credential bound to cell_id at issuance              │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  Trust boundary 3: scheduler → tenant-assignment write path                 │
│  Trust boundary 4: lifecycle-manager → Kubernetes Cluster API + Postgres    │
│  Trust boundary 5: host-pool → Kubernetes node API + hyperscaler IaaS API   │
│                                                                             │
│  ┌─ Per-cell namespace (K8s) ──────────────────────────────────────────┐    │
│  │  - One namespace per cell; NetworkPolicy denies cross-namespace     │    │
│  │  - Per-cell ServiceAccount with SPIFFE SVID = spiffe://.../cell-<id>│    │
│  │  - Per-cell Postgres schema; per-cell OpenBao binding               │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

Five trust boundaries:

1. **External → cluster ingress** (TLS, WAF, DDoS).
2. **cell-registry-rest → adapter-postgres** (Cedar policy evaluator; per-pack shard).
3. **scheduler → tenant-assignment write path** (per-pack write authority; cross-pack writes refused).
4. **lifecycle-manager → K8s Cluster API + Postgres** (cell CRUD authority; 2-person rule on decommission).
5. **host-pool → K8s node API + hyperscaler IaaS API** (node provisioning + drain authority).

## Assets & Data Classification

Per Bominal ADR-0028.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| Cell-assignment records `(tenant_id, cell_id, scope, assigned_at)` | `SENSITIVE_PIPA_ART23` (re-identification vector for small tenants) + `AUDIT` | Critical | append-only ledger; ≥ 2y hot + indefinite cold per pack-overlay | Postgres + `registry/cell-assignment.jsonl` |
| Cell metadata `(cell_id, pack, region, state, capacity_envelope)` | `INTERNAL_ONLY` + `BEHAVIORAL_TENANT_PRODUCT` when joined | High | hot | Postgres registry |
| Migration plans + checkpoints | `AUDIT` + `BEHAVIORAL_TENANT_PRODUCT` | High | ≥ 90d hot + 2y cold | Postgres + audit-chain |
| Host inventory `(host_id, pack, region, pool_state)` | `INTERNAL_ONLY` | Medium | hot until drained + 90d archive | Postgres |
| Per-cell SPIFFE SVIDs | `SECRET` | Critical | rotated 24h | SPIRE |
| Per-cell Postgres credentials | `SECRET` | Critical | rotated 30d | OpenBao |
| Per-cell OpenBao path binding | `SECRET` | Critical | rotated 30d | OpenBao |
| Cell-decommission deletion record | `AUDIT` | Critical | indefinite | audit-chain |
| Cell-boundary-violation event | `AUDIT` | Critical | indefinite | audit-chain + Mimir |
| Cedar policy fragments | `INTERNAL_ONLY` | Medium | git history | `policy/*.cedar` |

## Actors

| Actor | Trust level | Authentication | Capability |
|---|---|---|---|
| External tenant operator | Untrusted external | OIDC + MFA | Read own cell-assignment via slo-engine + cell-rest; no write authority |
| Customer application | Untrusted external | Per-tenant API key | No direct cell API access (routed via tenancy / workload µservices only) |
| Workload µservice | Semi-trusted internal | mTLS + SPIFFE SVID | Read cell-assignment for own work; never writes |
| `tenancy` µservice | Trusted internal | SPIFFE SVID | Reads cell-assignment on hot path; receives `TenantOnboarded` write path |
| `scheduler` worker | Trusted internal | SPIFFE SVID + OpenBao SA token | Writes placement decisions; binds tenant→cell |
| `lifecycle-manager` worker | Trusted internal | SPIFFE SVID + K8s SA token | Cell CRUD; declarative state-machine driver |
| `host-pool` worker | Trusted internal | SPIFFE SVID + IaaS API credential | Provisions / drains nodes |
| ops-security + axis-cell-substrate operators | Trusted internal | OIDC + MFA + OpenBao JIT | 2-person rule on decommission; Postgres admin via JIT |
| External auditor | Read-only external (time-boxed) | OIDC + MFA + OpenBao JIT | Read tenant-isolation evidence; cannot pivot |
| Attacker — opportunistic / targeted | Untrusted | none | Scans + targeted exploitation |
| Insider — accidental / malicious | Trusted internal | OIDC + MFA | Misconfigure scheduler / mass-decommission (mitigated by 2-person rule + audit-chain) |

## STRIDE Threat Catalog

Each row: ID; category; asset; description; likelihood (L/M/H); impact (L/M/H); risk score; mitigations; owner; residual risk; framework controls satisfied.

### Spoofing (S)

**T-S-01 — Workload µservice impersonates `scheduler` to write cell-assignment**

- Asset: cell-assignment write path
- Likelihood: M / Impact: H (could place a tenant in the wrong cell or pack — tenant-isolation breach + residency breach) / Risk: **H**
- Mitigations:
  - Write path requires SPIFFE SVID matching `spiffe://oyatie/cell/scheduler` or `spiffe://oyatie/cell/tenant-assignment-worker`; mismatch returns 403 + audit-emit.
  - Cedar policy fragment at `policy/cell-boundary.md` refuses writes from any other principal.
  - Postgres row-level-security additionally constrains write rows to the principal's pack.
- Owner: ops-security + axis-cell-substrate
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2, CC6.6; ISO 27001 A.5.15, A.5.17, A.8.2, A.8.3; GDPR Art. 32(1)(a)(b); KR PIPA Art. 29

**T-S-02 — Attacker impersonates `lifecycle-manager` to decommission a cell**

- Asset: cell-decommission authority
- Likelihood: L / Impact: H (mass tenant outage; potential data destruction at Postgres schema drop) / Risk: **H**
- Mitigations:
  - Decommission requires 2-person rule via OpenBao JIT elevation; the `lifecycle-manager` worker SVID is not sufficient alone.
  - Cell-decommission emits `CellDecommissioned` event signed Ed25519; the audit-chain validates the operator quorum.
  - Soft-deletion: ≥ 30d hold before actual Postgres schema drop or S3-prefix removal (`runbooks/cell-decommission.md`).
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.15, A.5.17, A.5.27, A.8.4

**T-S-03 — Host-pool SA token leaked → attacker drains production hosts**

- Asset: host-pool drain primitive
- Likelihood: L / Impact: H (DoS via mass drain) / Risk: **M**
- Mitigations:
  - SA token bound to pod identity; cannot be used outside cluster (per K8s 1.29+ projected token semantics).
  - Token rotation 24h; OpenBao audit log every issuance.
  - Drain ops emit `HostDrainStarted` event; ops dashboard alerts on rate > 1 drain/min (anomalous mass drain pattern).
  - Rate-limit at K8s API: max 1 concurrent drain per pack.
- Owner: ops-security + cloud-k8s
- Residual: L
- Frameworks: SOC 2 CC6.1, CC7.1; ISO 27001 A.5.15, A.8.5, A.8.7

**T-S-04 — Cross-pack write attempt via Cedar bypass**

- Asset: per-pack residency boundary
- Likelihood: L / Impact: H (residency breach — KR PIPA Art. 28 / GDPR Art. 44 violation) / Risk: **H**
- Mitigations:
  - Cedar fragment at `policy/data-residency.md` is the load-bearing residency check.
  - Postgres row-level-security re-checks pack at write commit.
  - `oya-cell-boundary` lane includes a Cedar fuzz harness asserting cross-pack writes refused.
  - Server-side enforcement at Postgres (not client-side) — even if Cedar misconfigured, RLS catches.
- Owner: ops-security + axis-cell-substrate + council-privacy
- Residual: L
- Frameworks: SOC 2 CC6.6; ISO 27001 A.5.14, A.5.23; GDPR Art. 32, Art. 44; KR PIPA Art. 28

### Tampering (T)

**T-T-01 — `cell-assignment.jsonl` ledger tampering via direct git push**

- Asset: `registry/cell-assignment.jsonl`
- Likelihood: L / Impact: H (could re-bind a tenant to a wrong cell historically) / Risk: **M**
- Mitigations:
  - Ledger is append-only; union-merge driver enforces no-rewrite at git layer.
  - CODEOWNERS scoped; branch protection on `dev` requires signed commits.
  - LEAN check `oya-cell-ledger-monotonic` validates new lines append (no edit / delete) at PR time.
  - Audit-chain Ed25519 seal per line; tampering breaks the chain.
- Owner: axis-cell-substrate
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.31, A.8.32, A.8.33

**T-T-02 — Postgres registry row tampering (DBA-level access misuse)**

- Asset: cell_assignments + cells tables
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - No human-DBA SSH; all admin via OpenBao JIT (2-person rule for writes).
  - Postgres audit extension logs every write; logs streamed to Loki under `tenant:oya-self`.
  - Periodic `pg_dump` SHA validation against audit-chain expected state; mismatch quarantines.
  - Row-level-security restricts writes to bound principal's pack.
- Owner: ops-security + axis-cell-substrate
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6, CC8.1; ISO 27001 A.8.11, A.8.24, A.8.25; GDPR Art. 32(1)(a)(b)

**T-T-03 — Cell CRD tampering via direct kubectl edit**

- Asset: K8s Cluster API CRDs (Cluster, Machine, MachineSet)
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - K8s RBAC: only `lifecycle-manager` SA may edit cell CRDs; human direct edits blocked except via OpenBao JIT 2-person.
  - GitOps drift: ArgoCD watches CRD declared state; drift alerts within 5 min.
  - Audit log on every CRD mutation; correlated with `lifecycle-manager` event emission.
- Owner: cloud-k8s + axis-cell-substrate
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.15, A.5.18, A.8.4

**T-T-04 — Scheduler decision tampering (false binpack score)**

- Asset: placement decision quality
- Likelihood: L / Impact: M (over-pack one cell; tenant migration churn) / Risk: **M**
- Mitigations:
  - Scheduler is stateless; decisions reproducible by replay from registry state.
  - Decisions emit `PlacementDecisionMade` event including `binpack_score` signature; CI replay validates determinism.
  - Quarterly placement-quality audit: balance-of-utilisation across cells in pack.
- Owner: axis-cell-substrate
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.32

### Repudiation (R)

**T-R-01 — Operator decommissions a cell, denies authorship**

- Asset: cell-decommission audit
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - 2-person rule via OpenBao JIT; both operators captured in audit-chain record.
  - Ed25519 seal on `CellDecommissioned` event includes both operator SPIFFE IDs.
  - Per-changeset evidence file at `microservices/cell/evidence/multispectrum/*` git-committed.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC4.1, CC8.1; ISO 27001 A.5.27, A.5.28, A.8.15; GDPR Art. 5(2)

**T-R-02 — Migration completed but operator denies authorship**

- Asset: migration audit-chain
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - Every migration emits `TenantMigrated{tenant_id, source_cell, target_cell, executed_by_spiffe, reason}` sealed Ed25519.
  - Migration checkpoints persisted in Postgres; replay reconstructs decision path.
- Owner: axis-cell-substrate
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.8.34

### Information Disclosure (I)

**T-I-01 — Cross-cell data leak via Postgres connection misrouting**

- Asset: per-tenant data in per-cell Postgres schema
- Likelihood: M / Impact: H (tenant-isolation breach, Sev-1) / Risk: **H**
- Mitigations:
  - Per-cell Postgres credentials bind to cell_id non-modifiably at OpenBao issuance.
  - PgBouncer per-cell pool; pools never share connections.
  - Row-level-security on every cell-scoped table.
  - LEAN check `oya-cell-boundary` greps workload µservices for any cross-cell DB ref at PR time.
  - Runtime: every workload µservice DB connection emits `cell_id` label; cardinality alert if connection-pool labels mismatch.
- Owner: ops-security + axis-cell-substrate
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.2, A.8.3, A.8.12; GDPR Arts. 5(1)(f), 25, 32; KR PIPA Art. 23; HIPAA §164.312(a)(1)

**T-I-02 — Cell-assignment ledger leaks tenant identity via small-set re-identification**

- Asset: `(tenant_id, cell_id)` pairs
- Likelihood: M / Impact: H (KR PIPA Art. 23 sensitive-data exposure) / Risk: **H**
- Mitigations:
  - `tenant_id` recorded is hashed-customer-id (per `microservices/observability/policy/tenant-isolation.md` model).
  - Cell-assignment reads enforce Cedar tenant-scope; cross-tenant reads refused.
  - Public dashboards never expose `cell_id` adjacency (which tenant lives next to which) — DP-noise injected if ever surfaced.
- Owner: ops-security + council-privacy
- Residual: M (small-tenant residual; mitigated by salt rotation)
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.12; GDPR Arts. 5(1)(f), 32; KR PIPA Art. 23

**T-I-03 — Migration plan leaks tenant business behavior (timing of migration ≈ scale signal)**

- Asset: `MigrationPlan` event payload
- Likelihood: M / Impact: M (competitive intel via timing) / Risk: **M**
- Mitigations:
  - Per-tenant migration events visible only to that tenant + operators (Cedar policy).
  - Aggregate migration-rate publicly disclosed only with DP noise.
- Owner: council-privacy
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.12; GDPR Art. 5(1)(f)

**T-I-04 — Host-pool inventory leak (which hyperscaler instance types are used)**

- Asset: host inventory metadata
- Likelihood: L / Impact: L (competitive info on infra; minimal tenant impact) / Risk: **L**
- Mitigations:
  - Internal-only data class; not exposed to tenants or auditors.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1

**T-I-05 — Secret leak (per-cell Postgres credential) via logs**

- Asset: OpenBao-managed cell credentials
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Secret-scanner CI lane (`oya-foundry-fitness-evidence-secret-scan`).
  - `Secret<T>` type strips `Debug`; OTel SDK redactor strips known patterns.
  - Rotation 30d; leak rotation < 60s via OpenBao.
- Owner: ops-security + cloud-secrets
- Residual: M
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.17, A.8.7

### Denial of Service (D)

**T-D-01 — Host-pool exhaustion via burst placement requests**

- Asset: scheduler placement availability
- Likelihood: M / Impact: H (new tenant onboarding halts) / Risk: **H**
- Mitigations:
  - Warm pool ≥ 2 nodes per pack; HPA pre-scales on placement queue depth.
  - Rate-limit placement requests: max N concurrent placements per pack.
  - Fail-open: when warm pool empty, placement returns `provisioning` state with ETA; tenant onboarding shows progress.
  - Capacity alarm at 70% / 85% / 95% pool consumption.
- Owner: ops-sre-reliability + cloud-k8s
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6, A.8.14; GDPR Art. 32(1)(c)

**T-D-02 — Postgres registry primary outage halts every cell-assignment read**

- Asset: cell-registry hot path
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Postgres HA: streaming primary + ≥ 2 replicas per pack.
  - Read replica failover within 30s; write quorum.
  - In-process cache in cell-registry-rest: 60s TTL; absorbs short outages.
  - Workload-side cache: per-µservice client SDK caches `cell_id` for 60s; absorbs read-replica outage.
- Owner: cloud-k8s + axis-cell-substrate
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.14

**T-D-03 — Migration race: two operators migrate same tenant simultaneously**

- Asset: migration consistency
- Likelihood: M / Impact: H (split-brain — tenant ends up in indeterminate cell state) / Risk: **H**
- Mitigations:
  - Postgres advisory lock per tenant during migration; second migration blocks.
  - Idempotency key per migration plan; second invocation observes existing plan + joins.
  - Runbook `tenant-migration.md` mandates checking active migration before starting.
- Owner: axis-cell-substrate
- Residual: L
- Frameworks: SOC 2 CC7.1, CC8.1; ISO 27001 A.8.32

**T-D-04 — Cluster API control-plane outage halts cell create / delete**

- Asset: lifecycle-manager hot path
- Likelihood: M / Impact: H (no new cells until restored) / Risk: **M**
- Mitigations:
  - Cluster API HA: ≥ 3 replicas per management cluster.
  - Existing cells unaffected (only new-cell flow halted).
  - Fail-closed: when control-plane unreachable, lifecycle-manager queues requests; flushes on recovery.
- Owner: cloud-k8s
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.14

**T-D-05 — Cell-boundary lane flooding (PR spam)**

- Asset: CI capacity
- Likelihood: L / Impact: L / Risk: **L**
- Mitigations:
  - Lane is fast (< 30s typical); spam absorbed.
  - Per-PR rate limit on GitHub Actions concurrency.
- Owner: axis-foundry
- Residual: L
- Frameworks: SOC 2 CC7.1

### Elevation of Privilege (E)

**T-E-01 — `scheduler` worker abused to write cell-assignment for unauthorised tenant**

- Asset: cell-assignment write path
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cedar policy: scheduler may only write within its bound pack.
  - Postgres row-level-security re-checks pack.
  - Audit-chain seal on every write; anomalous-pattern detection.
- Owner: axis-cell-substrate
- Residual: L
- Frameworks: SOC 2 CC6.6; ISO 27001 A.8.3

**T-E-02 — Cell SA token leaked → attacker reads cross-cell data**

- Asset: per-cell SPIFFE SVID
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - SVID bound to pod identity + cell namespace; cannot be used outside.
  - SVID TTL 24h.
  - NetworkPolicy denies cross-namespace traffic; even leaked SVID cannot pivot.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.5.17, A.8.5

**T-E-03 — K8s admin elevation used to mass-delete cell CRDs**

- Asset: cell CRD authority
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - K8s admin role gated by 2-person OpenBao JIT.
  - Audit-chain seal on every CRD mutation.
  - Soft-delete: CRD marked `decommissioning` first; actual delete only after ≥ 30d.
- Owner: ops-security + cloud-k8s
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.15, A.8.4

**T-E-04 — Cedar policy escape via crafted assignment field**

- Asset: Cedar policy evaluator
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cedar v4 (no known template-escape vectors).
  - Cedar fuzz at CI time (`oya-check-cedar-fragment-coverage`).
  - Field input lengths bounded at REST layer; oversized inputs refused pre-policy.
- Owner: axis-cell-substrate + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC8.1; ISO 27001 A.5.15, A.8.28

**T-E-05 — Postgres-superuser elevation deletes cell-assignment ledger**

- Asset: ledger durability
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Postgres superuser gated by 2-person OpenBao JIT.
  - `registry/cell-assignment.jsonl` git-committed; even Postgres-side delete recoverable from git + audit-chain.
  - Soft-deletion: superuser delete marks rows; actual delete scheduled-for-distinct-tracked-work 30d.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6; ISO 27001 A.5.27, A.8.4

## LINDDUN Privacy-Threat Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | `(tenant_id, cell_id)` pair | Same tenant across multiple cell rebalances over time is linkable via tenant_id, enabling inference of business growth. | Per-tenant DPA discloses operational telemetry; aggregate disclosure DP-bounded. | M |
| T-L-02 | Identifiability | Cell adjacency map | Knowing which tenants share a cell could re-identify small tenants. | Cell adjacency never exposed cross-tenant; ops dashboards admin-only. | L |
| T-L-03 | Non-repudiation | Operator decommission action | Operators may deny mass decommission. | 2-person rule + Ed25519 audit-chain seal. | L |
| T-L-04 | Detectability | Migration event timing | Migration timing correlates with tenant business events (e.g., end-of-quarter scale-up). | Reasonable; reflects business reality; disclosed at onboarding. | M |
| T-L-05 | Disclosure | Auditor read of cell topology | An auditor scoped to one tenant could infer infrastructure scale. | Auditor tokens scoped to tenant data; infra topology has separate read-scope. | L |
| T-L-06 | Unawareness | End-user (tenant's user) | End-user unaware their data lives in a specific cell. | Tenant DPA discloses operational architecture in summary. | L |
| T-L-07 | Non-compliance | GDPR Art. 17 erasure on tenant | Tenant deprovisioning must cascade through cell schema drop + S3 prefix delete. | DSR cascade documented in `compliance.md`; cell-decommission runbook references. | L |

## Mitigations Catalog (cross-reference)

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| SPIFFE SVID per cell/principal with bound identity claim | Preventive | ops-security | SVID issuance log |
| Postgres row-level-security on cell_assignments | Preventive | axis-cell-substrate | RLS regression test |
| Cedar policy fragments per scope (boundary / residency / public / auditor) | Preventive | ops-security | Cedar fuzz lane |
| 2-person rule on cell-decommission + Postgres superuser + K8s admin | Preventive | ops-security | OpenBao JIT audit |
| Ed25519 audit-chain seal on every write | Detective + Non-repudiation | audit-chain | Audit-chain regression tests |
| Soft-deletion (≥ 30d) for cells + DB rows | Recovery | axis-cell-substrate | Mass-delete anomaly alert |
| `oya-cell-boundary` LEAN lane | Preventive | axis-foundry | PR-time refusal |
| Per-cell namespace + NetworkPolicy + per-cell credentials | Preventive | ops-security + cloud-k8s | Network-policy regression test |
| Idempotency keys + advisory locks on migration | Preventive (race) | axis-cell-substrate | Race regression test |
| GitOps drift detector on cell CRDs | Detective | cloud-k8s | ArgoCD drift alert |
| OpenBao secret rotation (24h/30d cadences) | Preventive | cloud-secrets | OpenBao audit |

## Residual Risk Acceptance

| Risk ID | Residual | Why accepted | Re-review |
|---|---|---|---|
| T-I-02 (small-tenant re-identifiability via cell adjacency) | M | Salt-rotation + cell-adjacency-never-exposed mitigates to acceptable; remaining residual inherent to multi-tenancy economics. | Annually |
| T-L-01 (linkability across rebalances) | M | DPA disclosure + DP on aggregate; cannot fully eliminate without inhibiting operational telemetry. | Annually |
| T-L-04 (detectability via migration timing) | M | Business reality; consent at onboarding. | Annually |
| T-I-05 (secret leak via logs) | M | Human-error baseline; mitigated to acceptable via detection + rotation. | Quarterly |

Sign-off (RW until council captures):

- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`

## Per-Pack Overlay Sections

### pack-kr

- **KR PIPA Art. 23** (sensitive-data): tenant↔cell binding is sensitive; salt rotation per Art. 29.
- **KR PIPA Art. 28** (cross-border): cell never crosses pack; enforced server-side at Postgres RLS.
- **KR-ISMS-P §2.7 (접근통제)**: 2-person rule + JIT + SPIFFE.
- **KR-ISMS-P §2.10 (시스템 보안)**: NetworkPolicy + per-cell namespace.

### pack-us-healthcare

- **HIPAA §164.312(a)(1) (access control)**: per-cell SVID + Cedar + Postgres RLS satisfies Unique User Identification + Automatic Logoff + Encryption.
- **HIPAA §164.312(b) (audit controls)**: audit-chain Ed25519 on every cell event; retention ≥ 6y per §164.316(b)(2).
- **HIPAA §164.502 (minimum necessary)**: workload µservices read only own-cell scope.
- **BAA** with Covered Entity tenants: per-tenant; cell is part of BAA-eligible region pool (us-ashburn-1 HIPAA-eligible only).

### pack-eu

- **GDPR Art. 25** (privacy-by-design): cell-boundary enforced server-side.
- **GDPR Art. 32**: every mitigation contributes to risk-appropriate posture.
- **GDPR Arts. 44–50** (transfers): forbidden cross-pack; SCC-only exception.
- **NIS2** when applicable: incident-reporting timelines per `incident-response.md`.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/cell-overlay.md` map local PII law's confidentiality + integrity requirements to T-*-NN.

## Compliance Cross-Mapping (Globally Enforced)

| Framework | Coverage | Mapping doc |
|---|---|---|
| SOC 2 Type 2 | CC1–CC9 as cited inline | `compliance.md` |
| ISO 27001:2022 | Annex A.5–A.8 as cited | `compliance.md` |
| GDPR | Arts. 5, 6, 25, 28, 30, 32, 33 as cited | `dpia.md` + `compliance.md` |

## Re-review Triggers

- Any change to trust-boundary diagram above.
- Any K8s / Postgres / Cluster API version upgrade with security relevance.
- Any new pack activation.
- Annual scheduled review (Q2).
- Post-incident review (any Sev-1 / Sev-2 cell incident).
- Pen-test or audit finding.

## References

- Bominal ADR-0009; Bominal ADR-0019.
- ADR-0028 (audit-chain); ADR-0056 (BNF); ADR-0105 (layer enum); ADR-0117 (cloud-native infra); ADR-0139 (SLO gate); ADR-0131 (per-µservice); ADR-0140 (Cedar).
- `microservices/cell/PRD.md`; `microservices/cell/dpia.md`; `microservices/cell/compliance.md`.
- `microservices/cell/policy/{cell-boundary, data-residency, tenant-scope, ci-scope, auditor-scope, public-read}.md|cedar`.
- Microsoft STRIDE; LINDDUN methodology; OWASP Top 10 2021; NIST SP 800-154.
- Kubernetes Cluster API — `cluster-api.sigs.k8s.io`.
- SPIFFE / SPIRE — `spiffe.io`.
