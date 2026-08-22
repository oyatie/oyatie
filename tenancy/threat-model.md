---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: tenancy
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-tenancy + ops-security
deciders: council-architecture, ops-security, axis-tenancy, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + NIST SP 800-154
related_adrs: [ADR-0018, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0123, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
review_cadence: quarterly + on every Layer-A or Layer-B architecture change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC6.7, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.18, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.5.34, A.8.2, A.8.3, A.8.4, A.8.5, A.8.7, A.8.11, A.8.12, A.8.14, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.24, A.8.25, A.8.26, A.8.27, A.8.28, A.8.32, A.8.33, A.8.34"
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 22, 25, 26, 28, 30, 32, 33, 34, 35, 36, 44-50"
suggested_frameworks_by_pack:
  pack-kr: ["KR-ISMS-P §2.1-2.12 (보호조치)", "KR PIPA Arts. 15/17/18/22-2/23/23-2/24/25/28/29/29-2/33/33-2/34/36", "KR 전자문서법 Arts. 5/6/7"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308 (Administrative)", "§164.310 (Physical)", "§164.312 (Technical)", "§164.314 (Organizational)", "§164.316 (Policies)", "§164.502 (Permitted Uses)", "§164.514 (De-id)"]
  pack-eu: ["GDPR Arts. 25 + 32 + 35 + 44-50", "eIDAS 910/2014 (when DSR certificates qualify as AdES/QES)", "NIS2 2022/2555", "DORA 2022/2554 (financial-services)"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24/26-2/27"]
  pack-sg: ["PDPA 2012 §11-26 (Protection / Retention / Transfer)", "MAS-TRM v2021 §11-12", "PDPC Notice on Data Breach 2021"]
  pack-au: ["Privacy Act 1988 APP 1-13 (esp. APP 6, 8, 11, 12)", "APRA-CPS 234 §29-44", "OAIC NDB scheme"]
  pack-in: ["DPDPA 2023 §6-12", "RBI Master Direction on Outsourcing of IT Services 2023"]
  pack-br: ["LGPD Arts. 6, 7, 11, 14, 18, 33, 46, 48", "BACEN Res. 4.893/2021"]
  pack-ae: ["UAE PDPL Federal Decree-Law No. 45/2021 Arts. 5/6/9/15/23"]
  pack-ksa: ["PDPL Royal Decree M/19/2021 Arts. 4-9", "SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Threat Model: tenancy µservice

## Purpose

Identify, classify, and mitigate threats to the tenancy µservice's confidentiality, integrity, availability, and privacy posture. **The tenancy substrate is the load-bearing isolation authority for every other oyatie µservice; a compromise here is simultaneously a compromise of every tenant.** This document is the canonical security artifact reviewed by SOC 2 Type 2 examiners, ISO 27001 auditors, GDPR DPAs, KR PIPC reviewers, and HIPAA OCR investigators at first-tenant onboarding in every pack.

The stakes are higher than any other µservice in oyatie: an RLS bypass, JWT-key compromise, or cell-assignment misroute can expose every tenant simultaneously. Authored at that bar.

## Scope

### In-scope

All components introduced by Bominal ADR-0018 (tenancy + RLS posture, inherited) and ADR-0131 (per-microservice flat layout) for the tenancy µservice, deployed in pack-pinned Kubernetes clusters with a dedicated **Postgres + Citus + Patroni HA stack** (the persistence substrate IS this µservice; cf. observability where the substrate is Grafana stack):

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| Postgres 16 (primary metadata store) | `tenancy-tenant-lifecycle-*` (10 crates) |
| Citus 12.x (multi-tenant sharding extension) | `tenancy-isolation-policy-*` (9 crates) |
| Patroni (HA management for Postgres + Citus coordinator) | `tenancy-cell-assignment-*` (8 crates) |
| Valkey (cell-assignment + tenant-validate cache) | `tenancy-dsr-cascade-*` (8 crates) |
| OpenBao (JWT signing key + DB password secrets) | RLS policy YAML at `microservices/tenancy/policy/rls/*.yaml` |
| sqlx migration runner (schema + RLS DDL emission) | Cedar policy fragments at `microservices/tenancy/policy/*.cedar` |

### Out-of-scope

- Threats to the underlying Kubernetes cluster, container runtime, or hyperscaler IaaS layer — owned by the `cloud-k8s` µservice's threat model.
- Threats to OpenBao secret-manager itself — owned by the `cloud-secrets` µservice's threat model; inherited as upstream + cited here.
- Threats to GitHub Actions runners — owned by the `governance` µservice (CI substrate) threat model.
- Threats to other µservices' tenant-scoped data (their own threat models cover; this document covers only the isolation authority itself + the RLS contract surface).
- Threats to Bominal-side tenancy counterparts — separate Bominal threat-model; oyatie inherits decisions per `feedback_bominal_inheritance_precedence.md`.

## Trust Boundaries

```text
┌─ Internet ─────────────────────────────────────────────────────────────────┐
│                                                                            │
│   Tenant operators (admin UI)        Customer applications                 │
│         │                                  │                               │
│         │ (HTTPS, OIDC, mTLS)              │ (per-tenant JWT)              │
│         ▼                                  ▼                               │
│  ┌─ Public ingress (Envoy/Istio gateway) ──────────────────────────────┐   │
│  │  - TLS termination                                                  │   │
│  │  - WAF (rate-limit + OWASP CRS)                                     │   │
│  │  - DDOS protection                                                  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │                                             │
└──────────────────────────────│─────────────────────────────────────────────┘
                               ▼
┌─ Pack-pinned tenancy cluster ──────────────────────────────────────────────┐
│                                                                            │
│  Trust boundary 1: External → Cluster ingress                              │
│                                                                            │
│  ┌─ tenant-lifecycle-rest ──────┐    ┌─ dsr-cascade-rest ─┐                │
│  │  OIDC tenant-scoped writes   │    │  DSR submission   │                 │
│  └──────────────────────────────┘    └───────────────────┘                 │
│             │                                                              │
│  Trust boundary 2: JWT issuance + verification boundary (LOAD-BEARING)     │
│             │                                                              │
│  ┌─ isolation-policy-{rest,worker} ────────────────────────────────────┐   │
│  │  - JwtIssuer (signs JWT with Ed25519 key from OpenBao)              │   │
│  │  - JwtVerifier (verifies JWT against advertised public-key fingerprint)│
│  │  - SigningKeyStore (OpenBao client; HSM-backed where available)     │   │
│  │  - Key rotation 30d; old pubkey valid 30d grace; Workflow event     │   │
│  │    JwtSigningKeyRotated advertises new fingerprint                  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│             │                                                              │
│  Trust boundary 3: Per-tenant Postgres RLS boundary (LOAD-BEARING)         │
│             │                                                              │
│  ┌─ Postgres + Citus (multi-tenant) ──────────────────────────────────┐    │
│  │  - SET LOCAL app.current_tenant_id = $1 on every connection        │    │
│  │  - FORCE ROW LEVEL SECURITY on all tenant-bound tables             │    │
│  │  - CREATE POLICY ... USING (tenant_id = current_setting(...))      │    │
│  │  - No superuser bypass code path in any tenancy-adjacent crate     │    │
│  │  - Citus shard distribution by tenant_id (consistent-hash)         │    │
│  │  - Patroni HA: 1 primary + 2 sync replicas + (optional 2 async)    │    │
│  └────────────────────────────────────────────────────────────────────┘    │
│             │                                                              │
│  Trust boundary 4: Cell-assignment routing boundary                        │
│             │                                                              │
│  ┌─ cell-assignment-worker ───────────────────────────────────────────┐    │
│  │  - Reads CellHealth probe; writes Citus pg_dist_shard placements   │    │
│  │  - Consistent-hash on TenantId → ShardKey                          │    │
│  │  - Rebalance via Citus coordinator (transactional shard moves)     │    │
│  │  - Valkey cache for tenant→cell hot reads (≤ 2ms)                  │    │
│  └────────────────────────────────────────────────────────────────────┘    │
│             │                                                              │
│  Trust boundary 5: DSR cross-µservice fan-out boundary                     │
│             │                                                              │
│  ┌─ dsr-cascade-worker ───────────────────────────────────────────────┐    │
│  │  - Receives DsrRequest                                             │    │
│  │  - Emits TenantDeletionRequested to every µservice (Workflow)      │    │
│  │  - Collects ErasureReceipts                                        │    │
│  │  - Aggregates Merkle root → ProofOfErasure                         │    │
│  │  - Sealed by audit-chain µservice                                  │    │
│  └────────────────────────────────────────────────────────────────────┘    │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

Five trust boundaries:
1. **External → Cluster ingress** (TLS, WAF, DDoS).
2. **JWT issuance + verification** (load-bearing; every µservice's tenant-trust origin).
3. **Per-tenant Postgres RLS** (load-bearing; the database row-level isolation).
4. **Cell-assignment routing** (Citus shard placement; cell-health driven).
5. **DSR cross-µservice fan-out** (regulator-facing erasure proof).

## Assets & Data Classification

Per Bominal ADR-0028 (audit-chain + data-class taxonomy) and the `check-data-class` LEAN lane.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| Tenant metadata (`tenant_id`, status, jurisdiction, plan tier, created_at) | `SENSITIVE_PIPA_ART23` + `BEHAVIORAL_TENANT_PRODUCT` | High | per pack legal min; default 7y after deletion completes (for erasure-receipt retention) | Postgres + Citus pack-pinned |
| Tenant identifier mapping (`canonical_tenant_id` ↔ tenant_id) | `SENSITIVE_PIPA_ART23` (KR PIPA Art. 23 with auxiliary data → sensitive) | Critical | OpenBao tenant-resolver | OpenBao |
| RLS policy text + `tenant-bound-table` registry | `INTERNAL_ONLY` (policy text); `SECRET` when carrying tenant identifiers | Medium | git history (declarative) + DB state | `microservices/tenancy/policy/rls/*.yaml` + Postgres `pg_policies` |
| Cell-assignment table (tenant → cell, shard_key) | `BEHAVIORAL_TENANT_PRODUCT` | Medium | indefinite; rebalanced on cell changes | Citus pg_dist_* tables + Valkey cache |
| JWT signing keys (Ed25519, per-pack, per-environment) | `SECRET` | Critical | OpenBao with 30d rotation + HSM-backed where available | OpenBao |
| JWT public-key fingerprints (advertised via Workflow) | `INTERNAL_ONLY` | Low | per-fingerprint rotation history | Workflow event log + Ontology |
| Postgres replication password | `SECRET` | Critical | OpenBao with 90d rotation | OpenBao |
| Citus coordinator → worker authentication | `SECRET` | Critical | OpenBao rotation | OpenBao |
| Patroni REST API token | `SECRET` | Critical | OpenBao | OpenBao |
| DSR request records | `AUDIT` + `SENSITIVE_PIPA_ART23` | High | ≥ 7y after request completion; immutable | Postgres + audit-chain |
| Erasure receipts (per-µservice) | `AUDIT` | High | ≥ 7y; immutable; Merkle leaf | Postgres + audit-chain |
| Proof-of-erasure certificates | `AUDIT` (regulator-disclosable) | High | indefinite | audit-chain + `microservices/tenancy/evidence/dsr/` |
| Cedar policy fragments | `INTERNAL_ONLY` | Medium | git history | `microservices/tenancy/policy/*.cedar` |
| Tenant onboarding consent records | `AUDIT` + `PII_IDENTIFYING` (operator identity) | High | per pack legal min; default 7y | Postgres + audit-chain |
| Valkey cache contents (tenant validate results) | `BEHAVIORAL_TENANT_PRODUCT` | Medium (transient) | 60s TTL | Valkey in-memory |
| Audit-chain seals (per lifecycle event) | `AUDIT` | High | indefinite; immutable | audit-chain µservice |
| Schema-migration records (per tenant activation) | `AUDIT` | Medium | indefinite | Postgres |

## Actors

| Actor | Trust level | Authentication | Capability |
|---|---|---|---|
| External tenant operator (human) | Untrusted external | OIDC + MFA via Application Shell | CRUD own tenant's lifecycle (within plan tier); read own status |
| Customer application (machine) | Untrusted external | Per-tenant JWT issued by tenancy | Inherit tenant_id from JWT; no direct tenancy admin |
| Platform operator (oyatie internal) | Trusted internal | OIDC + MFA + JIT elevation via OpenBao | Create / suspend / delete any tenant (with 2-person rule for delete) |
| DPO (council-privacy chair) | Trusted internal | OIDC + MFA | Approve DSR cascades; verify proof-of-erasure |
| Internal µservice (workload) | Semi-trusted internal | SPIFFE identity + JWT for caller-tenant-context | Read own-tenant data via RLS; cannot pivot to other tenants |
| Workflow event consumer | Trusted internal | mTLS in mesh | Consume `TenantActivated`, `TenantDeletionRequested`, etc. |
| External auditor (SOC 2 / ISO 27001 / HIPAA / KR-PIPC / GDPR DPA) | Read-only external on time-boxed window | OIDC + MFA + JIT short-lived token via OpenBao | Read scoped subset; cannot pivot to non-scope tenants |
| Regulator (KR PIPC investigator, EU DPA, etc.) | Read-only external + legal compulsion | OIDC + MFA + JIT + legal-process bound | Read proof-of-erasure; read DSR cascade history for scoped tenants |
| Attacker — opportunistic | Untrusted | none | Scans + low-skill exploitation; assume always present |
| Attacker — targeted | Untrusted | none | Sophisticated; supply-chain awareness; assume present for prod-tier surfaces |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure RLS policy / cell assignment / migration script |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case threat actor for confidentiality; mitigated by 2-person rule + audit-chain + separation-of-duties |

## STRIDE Threat Catalog

Each threat carries: ID; category; asset; description; likelihood (L/M/H); impact (L/M/H); risk score; mitigations; owner; residual risk; framework controls.

### Spoofing (S)

**T-S-01 — Attacker forges a JWT with another tenant's `tenant_id` claim**
- Asset: JWT issuance + verification boundary (the load-bearing isolation primitive)
- Likelihood: M / Impact: H (cross-tenant data exposure) / Risk: **H (CRITICAL)**
- Mitigations:
  - JWTs signed with Ed25519 by per-pack, per-environment private key stored in OpenBao (HSM-backed where available).
  - JWT verifiers in every µservice check signature against the advertised public-key fingerprint; refresh on `JwtSigningKeyRotated` Workflow event.
  - 30d signing-key rotation; 30d grace for old pubkey to verify in-flight tokens; rotation event audit-chain-sealed.
  - JWT `iss` claim bound to pack + env; cross-pack JWTs rejected by verifier.
  - JWT `exp` claim ≤ 1h sliding window; refresh via separate refresh-token path; revocation list synced via Workflow.
  - LEAN check `governance-jwt-key-fingerprint-advertised`: refuses key rotation without a fingerprint Workflow event.
  - Pen-test: synthetic JWT forgery attempt annually; should fail with audit-emitted `jwt_signature_invalid_total`.
- Owner: ops-security + axis-tenancy
- Residual: L (signing-key compromise required + audit visibility on attempt)
- Frameworks: SOC 2 CC6.1, CC6.2, CC6.6; ISO 27001 A.5.17, A.8.5, A.8.7, A.8.24; GDPR Art. 32(1)(b)(c); KR PIPA Art. 29 + Art. 29-2 (encryption)

**T-S-02 — Tenant operator impersonates another tenant via session hijack**
- Asset: tenant admin UI session
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - OIDC + MFA required; session cookies HttpOnly + SameSite=Strict + Secure.
  - Session token tied to client IP + UA fingerprint; mismatch triggers re-auth.
  - Cedar policy `tenant-scope.cedar` enforces tenant operator can only act on own tenant; cross-tenant attempts logged + Cedar-denied.
  - Session TTL ≤ 4h; idle timeout ≤ 30min.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2; ISO 27001 A.5.15, A.8.5; GDPR Art. 32; HIPAA §164.312(d)

**T-S-03 — Service spoofs `slo-engine-worker` SPIFFE identity to fake `RlsPolicyInstalled` events**
- Asset: Workflow event integrity
- Likelihood: L / Impact: M / Risk: **M**
- Mitigations:
  - Every Workflow event signed by emitter SPIFFE identity; consumer validates against expected emitter for that event type.
  - `RlsPolicyInstalled` only authorised from `spiffe://oyatie/tenancy/isolation-policy-worker`; consumer rejection + audit-emit on mismatch.
- Owner: axis-tenancy + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.17, A.8.7; GDPR Art. 32(1)(b)

**T-S-04 — Attacker impersonates platform operator to create / delete tenants**
- Asset: platform-operator admin path
- Likelihood: L / Impact: H (mass tenant deletion would be a catastrophic availability + privacy incident) / Risk: **H**
- Mitigations:
  - Platform-operator OIDC + MFA + JIT elevation via OpenBao for any create/delete operation.
  - 2-person rule for delete (CLI requires second signature from ops-security operator).
  - Mass-deletion anomaly detection: `tenancy_delete_rate{}` over 5m > N triggers Sev-1 page.
  - Soft-delete with 30d recovery window default (Open Question 4 resolution in IP-009); hard-delete after grace.
- Owner: ops-security + council-privacy
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6, CC7.4; ISO 27001 A.5.17, A.5.27, A.8.4; GDPR Art. 32; KR PIPA Art. 29

**T-S-05 — Auditor JIT token reused outside engagement window or against non-scoped tenants**
- Asset: auditor read scope (Cedar `auditor-scope.cedar`)
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Auditor JIT tokens scoped to specific tenants + engagement window TTL ≤ 4h.
  - Cedar policy verifies window + scope on every request; deny outside.
  - Every auditor read audit-chain-emitted.
  - mTLS client cert pinned to the auditor firm's gateway during engagement.
- Owner: ops-security + council-privacy
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2, CC8.1; ISO 27001 A.5.15, A.8.2, A.8.3, A.8.34; GDPR Art. 28; HIPAA §164.308(a)(4)(ii)(B)

### Tampering (T)

**T-T-01 — RLS policy tampering — attacker disables RLS on tenant-bound table**
- Asset: Postgres `pg_policies` + RLS DDL state
- Likelihood: M / Impact: H (would expose every row to every tenant; **catastrophic blast radius**) / Risk: **H (CRITICAL)**
- Mitigations:
  - LEAN check `governance-rls-no-superuser-bypass` (NEW): refuses any superuser-context code path in tenancy-adjacent crates; PR-time enforcement.
  - LEAN check `governance-rls-force-on-tenant-tables` (NEW): refuses tenant-bound table migrations without `FORCE ROW LEVEL SECURITY`; PR-time enforcement.
  - All RLS policy mutations via declarative YAML in `microservices/tenancy/policy/rls/<table>.yaml` + PR review by CODEOWNERS (axis-tenancy + ops-security).
  - Continuous DB-state validator (runs every 5min): compares live `pg_policies` to declared YAML; drift triggers Sev-1 + auto-rollback if config-as-code drift detected.
  - Postgres role separation: app role cannot `ALTER POLICY` or `ALTER TABLE ... DISABLE ROW LEVEL SECURITY`; only `tenancy-admin-jit` role can, and JIT elevation requires 2-person rule.
  - Audit-chain seal on every RLS DDL execution.
- Owner: axis-tenancy + ops-security
- Residual: L (catastrophic-blast-radius residual kept ≤ M with multi-layer detection)
- Frameworks: SOC 2 CC6.1, CC6.6, CC7.1, CC8.1; ISO 27001 A.5.15, A.5.17, A.8.2, A.8.3, A.8.4, A.8.12, A.8.32; GDPR Art. 25, Art. 32; HIPAA §164.312(a)(1), §164.312(c)(1); KR PIPA Art. 29

**T-T-02 — JWT verifier accepts tampered claims (algorithm-confusion attack)**
- Asset: JWT verifier path in every µservice
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Verifier hardcodes algorithm `EdDSA` (Ed25519); refuses `alg=none` and `alg=HS*` (HMAC-confusion attack defence).
  - Verifier library: `jsonwebtoken` ≥ 9.x with explicit `Algorithm::EdDSA` whitelist.
  - Unit test in every µservice's JWT verifier crate: asserts `alg=none` + `alg=HS256` + `alg=RS256` all rejected.
  - Pen-test annually with alg-confusion attack vectors.
- Owner: ops-security + axis-tenancy
- Residual: L
- Frameworks: SOC 2 CC6.1, CC8.1; ISO 27001 A.5.17, A.8.7, A.8.24; GDPR Art. 32; OWASP API Top 10 (2023) #2

**T-T-03 — Schema-migration runner tampered to skip RLS DDL during tenant activation**
- Asset: sqlx migration runner + RLS install step
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Migration runner signed binary; signature validated at exec time.
  - Activation worker emits `TenantActivated` event ONLY after a post-migration validation step confirms RLS is `forced=true` on every tenant-bound table; failure emits `TenantActivationFailed` with rollback.
  - Synthetic-tenant activation drill weekly: full cycle + cross-tenant probe; should always show zero cross-tenant rows.
- Owner: axis-tenancy + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.1, CC8.1; ISO 27001 A.8.32, A.8.33; GDPR Art. 32

**T-T-04 — Cell-assignment tampering — attacker reroutes tenant T to a different cell's shard**
- Asset: Citus `pg_dist_shard` + `pg_dist_placement`
- Likelihood: L / Impact: H (could mix tenants' data on one shard if naive routing) / Risk: **M**
- Mitigations:
  - Citus distribution by `tenant_id` is the hash key; routing is deterministic per `tenant_id`; tampering the cell-assignment cache (Valkey) cannot expose cross-tenant rows because RLS still gates row reads.
  - Cell-assignment writes audit-chain-sealed; live-state validator compares Valkey to Postgres source-of-truth every 1min.
  - `CellRebalanceStarted` / `Completed` events: integrity checksums computed before + after; mismatch aborts rebalance + alerts.
- Owner: axis-tenancy + ops-sre-reliability
- Residual: L (defence-in-depth: RLS is the load-bearing primitive; cell-assignment tampering does not bypass RLS)
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.8.12, A.8.32; GDPR Art. 32

**T-T-05 — DSR cascade receipt forgery — attacker forges an `ErasureReceipt` to spoof completeness**
- Asset: DSR receipt aggregation
- Likelihood: L / Impact: H (false proof-of-erasure = regulatory fraud) / Risk: **M**
- Mitigations:
  - Every `ErasureReceipt` signed by emitting µservice's SPIFFE identity; consumer (`dsr-cascade-worker`) validates signature against expected emitter set (derived from `MicroserviceRegistered` events).
  - Missing-receipt SLA timer: if expected µservice doesn't emit within 30d window, escalate Sev-2 + halt proof-of-erasure emission until receipt obtained or manual-override (DPO + ops-security + 2-person rule + audit-chain seal).
  - Synthetic DSR drill quarterly: full cascade across all M01 µservices; verify proof aggregates correctly.
- Owner: council-privacy + axis-tenancy
- Residual: L
- Frameworks: SOC 2 CC4.1, CC8.1; ISO 27001 A.5.27, A.5.28, A.8.15, A.8.34; GDPR Art. 17, Art. 30; KR PIPA Art. 36; HIPAA §164.502

**T-T-06 — Citus shard-move corruption during rebalance**
- Asset: tenant rows during rebalance
- Likelihood: L / Impact: H (data loss) / Risk: **M**
- Mitigations:
  - Citus shard moves are transactional (Citus's built-in shard rebalancer uses logical replication + transactional cut-over).
  - Row-count + per-row checksum (sha256) computed pre + post; mismatch aborts the move + restores from source.
  - Rebalance audit-chain-sealed.
  - Synthetic rebalance drill monthly with integrity validation.
- Owner: axis-tenancy + ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.8.12, A.8.14; GDPR Art. 32(1)(b)(c)

### Repudiation (R)

**T-R-01 — Tenant deletion executed but operator denies authorship**
- Asset: TenantDeletionCompleted event chain
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Every deletion includes `actor=<operator-OIDC-subject>` + `secondary_signer=<ops-security-operator-OIDC-subject>` (2-person rule) + Ed25519 audit-chain seal per Bominal ADR-0028.
  - Audit-chain Merkle proof: tamper-evident.
  - Per-changeset evidence at `microservices/tenancy/evidence/dsr/<dsr_id>.json` carries both signatures + the request body + the proof-of-erasure.
- Owner: axis-tenancy + audit-chain
- Residual: L
- Frameworks: SOC 2 CC4.1, CC8.1; ISO 27001 A.5.27, A.5.28, A.8.15; GDPR Art. 5(2), Art. 30; eIDAS 910/2014 (AdES when applicable)

**T-R-02 — Tenant suspended by operator but operator denies authorship**
- Asset: TenantSuspended event
- Likelihood: L / Impact: L / Risk: **L**
- Mitigations:
  - Same operator-attribution + audit-chain Ed25519 seal as T-R-01.
  - Suspension is reversible (TenantResumed); accidental suspension recoverable.
- Owner: axis-tenancy
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.5.27; GDPR Art. 30

**T-R-03 — RLS policy installed but author denies authorship**
- Asset: RlsPolicyInstalled event + git history
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - All RLS policy YAML changes via signed git commits per `branch-protection.yaml` (required signed commits on `dev`).
  - PR review record + commit signature + audit-chain seal on install execution.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.8.34; GDPR Art. 5(2)

**T-R-04 — JWT signing-key rotation executed but actor denies authorship**
- Asset: JwtSigningKeyRotated event + OpenBao audit log
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - Rotation triggered by OpenBao rotation cron (system identity); manual rotation requires 2-person rule (ops-security + axis-tenancy).
  - OpenBao audit log carries rotation actor + timestamp; cross-checked with audit-chain seal.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC4.1, CC8.1; ISO 27001 A.5.17, A.8.34

### Information Disclosure (I)

**T-I-01 — RLS bypass via superuser-context code path**
- Asset: tenant data rows
- Likelihood: L / Impact: H (catastrophic; **simultaneous breach of every tenant**) / Risk: **H (CRITICAL)**
- Mitigations:
  - LEAN check `governance-rls-no-superuser-bypass` (NEW; PR-time): refuses any code that connects as a superuser to a tenant-bound database OR sets `bypass_rls=true` on a connection. The check uses `cargo-deny` advisory + AST-grep patterns for `SET ROLE postgres`, `SET LOCAL row_security = off`, and direct `bypassrls`-flagged connections.
  - Postgres role design: app role (`tenancy_app`) has no `bypassrls`; only `tenancy-admin-jit` role can, and JIT issuance requires 2-person rule + audit-chain seal.
  - Continuous-compliance scan: weekly Postgres role-attribute audit; any role with `bypassrls=true` in non-JIT state triggers Sev-1.
  - Pen-test: attempt to read cross-tenant rows via every code path; should fail.
- Owner: axis-tenancy + ops-security
- Residual: L (multi-layer; bypass requires both Postgres role compromise AND CI lane evasion)
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.5.18, A.8.2, A.8.3, A.8.4, A.8.12; GDPR Arts. 5(1)(f), 25, 32; KR PIPA Arts. 23, 29; HIPAA §164.312(a)(1)

**T-I-02 — RLS predicate bypass via SQL injection in tenant_id-bearing query**
- Asset: tenant data rows
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - All tenant-id-bearing queries use parameterised `$1` placeholders (sqlx prepared statements); never string-formatted.
  - LEAN check `check-sql-injection-via-format-string`: AST-grep for `format!("...tenant_id = '{}'", ...)` pattern; refuses.
  - Even if injection succeeded, RLS at row level would still block (defence-in-depth).
  - Static analysis via `sqlx::query!` macro: validates SQL at compile time.
- Owner: axis-tenancy + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.8.28, A.8.32; GDPR Art. 32; OWASP Top 10 #3 (Injection)

**T-I-03 — Cell-assignment cache poisoning leaks cross-tenant signal**
- Asset: Valkey cache contents
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - Valkey cache stores (tenant_id → cell_id, shard_key); never row data.
  - Cache poisoning would route to wrong shard, but RLS still gates row reads — no data leak.
  - Cache write authority: only `cell-assignment-worker` SPIFFE identity; mTLS Valkey access; ACL per principal.
- Owner: axis-tenancy + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.12

**T-I-04 — Tenant metadata exposed via error message stack trace**
- Asset: HTTP 5xx response bodies
- Likelihood: M / Impact: M (could expose tenant_id, jurisdiction, internal state) / Risk: **M**
- Mitigations:
  - Error response shape: structured `{error, request_id, message}`; never `details` carrying tenant context to anonymous callers.
  - `request_id` correlates server-side logs; full diagnostics behind operator login.
  - Tracing redactor strips `tenant_id` claim from outbound trace span attributes by default (matches observability OTel redactor).
  - Per-pack DSR enforcement: tenant_id never echoed cross-tenant; per-tenant operator can see own tenant_id only.
- Owner: axis-tenancy
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.8.11, A.8.12; GDPR Art. 5(1)(f), Art. 32

**T-I-05 — Tenant deletion incomplete — residual data in one µservice exposed via aggregation**
- Asset: tenant data across all µservices
- Likelihood: M (engineering discipline gap; some µservice's DSR handler is buggy / missing) / Impact: H / Risk: **M-H**
- Mitigations:
  - DSR cascade SLA timer: per-µservice receipt within 30d; missing receipt = halt-and-escalate.
  - LEAN check `governance-dsr-handler-conformance`: every µservice must register a DSR handler in its catalog record; PR-time enforced.
  - Quarterly DSR drill: synthetic tenant created + deleted across all µservices; cumulative proof should match expected µservice count.
  - Receipt aggregation surfaces missing-µservice with the µservice name + reason; DPO sign-off required for proof-of-erasure emission with missing receipt + alternative-measure (e.g., data already deleted by retention policy).
- Owner: council-privacy + axis-tenancy + every workload µservice owner
- Residual: M (engineering discipline residual)
- Frameworks: SOC 2 CC4.1, CC6.7; ISO 27001 A.5.27, A.8.15; GDPR Art. 17, Art. 30; KR PIPA Art. 36; DPDPA §12; LGPD Art. 18

**T-I-06 — Secret (JWT signing key / Postgres password / Patroni token) leaked via logs**
- Asset: OpenBao-managed secrets emitted accidentally
- Likelihood: M / Impact: H (cascades to broad compromise) / Risk: **H**
- Mitigations:
  - Secret-scanner CI lane (`governance-evidence-secret-scan` — already exists) scans every commit + log emission for known secret patterns.
  - OpenBao SecretReference materialisation never logs the raw secret; wraps in `Secret<T>` type with stripped `Debug` impl.
  - OTel SDK redactor strips known-secret patterns at emission time.
  - Rotation policy: 30d for JWT signing keys, 30d for API tokens, 90d for Postgres passwords (rotate before leaked secret expires).
  - Secret-leak runbook: detection → immediate rotation → forensic trace → engineering education.
- Owner: ops-security + cloud-secrets
- Residual: M (human-error baseline; never fully eliminated)
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.17, A.8.7, A.8.12; GDPR Art. 32(1)(a)(b)(c)(d)

**T-I-07 — Cross-tenant inference via timing side-channel on tenant-validate hot path**
- Asset: tenant-validate response timings
- Likelihood: L / Impact: L (could leak presence/absence of a tenant_id) / Risk: **L**
- Mitigations:
  - Constant-time path for cache-hit (≤ 5 ms) vs cache-miss (≤ 20 ms); attacker can distinguish miss from hit but not own-tenant vs other-tenant.
  - Validate path returns `{valid: bool}`; never echoes the queried `tenant_id` back unless OIDC says the caller owns it.
- Owner: axis-tenancy
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.12

### Denial of Service (D)

**T-D-01 — Tenant-validate hot path overload (single tenant burst-writes)**
- Asset: validate path
- Likelihood: H / Impact: H (validate-down = every µservice down) / Risk: **H (CRITICAL)**
- Mitigations:
  - Per-tenant rate limits at WAF + ingress: max RPS per tenant; excess returns 429.
  - Valkey cache absorbs hot path: cache-hit ratio target ≥ 99 %.
  - Validate-path HA: min 3 replicas per cell; HPA on CPU + p99 latency; max 100 replicas.
  - Postgres + Citus query path bounded; never unbounded scan.
  - Self-SLO on validate availability: 99.99 % monthly; burn-rate alarm at 2×.
- Owner: ops-sre-reliability + axis-tenancy
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6, A.8.14; GDPR Art. 32(1)(c)

**T-D-02 — Patroni HA failure causes Postgres primary outage**
- Asset: Postgres primary
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Patroni topology: 1 primary + 2 sync replicas + (optional 2 async for hyperscaler-tier packs); auto-failover ≤ 10s.
  - Patroni split-brain detection via DCS (etcd / Consul); quorum-based leader election; cells in 3 separate AZs.
  - Synthetic failover drill quarterly: induce primary loss; verify validate hot path availability stays ≥ 99.99% during failover.
  - Read replicas absorb read load during failover.
- Owner: ops-sre-reliability + axis-tenancy
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.14; GDPR Art. 32(1)(c)

**T-D-03 — Citus coordinator outage halts all writes**
- Asset: Citus coordinator
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Coordinator runs HA via Patroni (analogous to primary HA).
  - Read path can proceed against worker nodes directly for some queries; write path requires coordinator.
  - Failover ≤ 30s.
- Owner: ops-sre-reliability + axis-tenancy
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.14

**T-D-04 — DSR cascade flood (attacker submits N DSR requests to exhaust)**
- Asset: DSR cascade worker capacity
- Likelihood: L / Impact: M / Risk: **M**
- Mitigations:
  - DSR submission rate-limited per tenant operator (max N DSRs / day).
  - DSR cascade worker HPA on queue depth.
  - Bot-detection at WAF (CAPTCHA challenge on suspicious DSR-submission rate).
- Owner: ops-sre-reliability + axis-tenancy
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.6

**T-D-05 — Tenant-activation work queue exhausted (mass-onboarding attack)**
- Asset: activation worker pool
- Likelihood: L / Impact: M / Risk: **M**
- Mitigations:
  - Activation rate-limited per ingress (max N activations / hour per IP).
  - Activation worker HPA on queue depth; max 50 replicas per cell.
  - Concurrent activations capped at 1000 per cell (per capacity-model.md).
- Owner: ops-sre-reliability + axis-tenancy
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6

### Elevation of Privilege (E)

**T-E-01 — Postgres role escalation: app role gains `bypassrls`**
- Asset: Postgres role attributes
- Likelihood: L / Impact: H / Risk: **H (CRITICAL)**
- Mitigations:
  - Postgres roles managed via declarative IaC in `microservices/tenancy/iac/terraform/postgres-rbac.tf`; PR-reviewed; CI lane validates declared-vs-live state.
  - Continuous DB-state validator: weekly role-attribute audit; alerts on any role with `bypassrls=true` outside expected JIT state.
  - Role creation requires DBA-JIT elevation via OpenBao (2-person rule).
- Owner: ops-security + axis-tenancy
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.2, A.8.4

**T-E-02 — Cedar policy escape via crafted manifest field**
- Asset: Cedar policy evaluation
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cedar v4+ used; no known template-escape vectors.
  - Cedar fragments fuzzed at CI time (`check-cedar-fragment-coverage` lane).
  - Field input lengths bounded at REST API; oversized inputs rejected pre-policy.
- Owner: ops-security + axis-tenancy
- Residual: L
- Frameworks: SOC 2 CC6.1, CC8.1; ISO 27001 A.5.15, A.8.28

**T-E-03 — Tenant operator escalates to platform operator**
- Asset: platform-operator scope
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - OIDC issuer enforces principal type (tenant_operator vs platform_operator); Cedar policy + tenancy REST endpoints check principal type before action.
  - Platform-operator JIT requires OpenBao elevation + 2-person rule for sensitive actions (create, delete).
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2; ISO 27001 A.5.15, A.8.2, A.8.4

**T-E-04 — DBA elevation token exfiltrated → arbitrary RLS-policy mutations**
- Asset: DBA JIT token
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - DBA-JIT tokens TTL ≤ 1h; bound to operator OIDC subject.
  - 2-person rule for any RLS-mutating action.
  - Every DBA action audit-chain-sealed; live-state validator detects drift within 5min.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.4, A.8.34

**T-E-05 — Patroni REST API used to force-failover to compromised replica**
- Asset: Patroni cluster control
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Patroni REST API mTLS-only; client cert pinned to ops-sre-reliability JIT identity.
  - REST API exposed only in-cluster; no public ingress.
  - Failover commands audit-chain-sealed.
- Owner: ops-sre-reliability + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6, CC7.1; ISO 27001 A.5.15, A.8.4, A.8.20

## LINDDUN Privacy-Threat Catalog

LINDDUN (Linkability / Identifiability / Non-repudiation / Detectability / Disclosure / Unawareness / Non-compliance) covers privacy-specific threats not fully captured by STRIDE.

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | Tenant operator's OIDC subject ↔ tenant_id | An auditor with both views could correlate operators to tenants across the audit log. | Audit-log retention bounded per pack; operator-id ↔ tenant-id correlation requires JIT + audit-chain seal. | L |
| T-L-02 | Identifiability | hashed `tenant_id` (used cross-µservice) | sha256(canonical_tenant_id ++ salt) may be re-identifiable via auxiliary data for very small tenant populations. | Salt rotation 12mo; small-tenant detection triggers extra DP-noise injection on any cross-tenant aggregate. | L |
| T-L-03 | Non-repudiation | Tenant operator authoring lifecycle changes | Signed commits + audit-chain Ed25519; non-repudiation by design. | – | L |
| T-L-04 | Detectability | DSR submission timing | An external observer (e.g., a public regulator) could infer "tenant X just submitted a DSR" from API timing. | DSR submission masked via constant-time response; outbound `DsrRequest` event timing jittered ±30s. | L |
| T-L-05 | Disclosure | Auditor read scope | Auditor could pivot from scoped tenant A to non-scoped tenant B via shared dashboard. | Auditor Cedar policy enforces scoped_tenants; pen-test annually. | L |
| T-L-06 | Unawareness | End-user (the tenant's user) | The tenant's end-user may not know their account is enrolled in oyatie's tenant database. | Joint-controllership clause in DPA template; tenant must disclose to its end-users per Art. 26. | M |
| T-L-07 | Non-compliance | GDPR Art. 17 right-to-erasure | DSR cascade may have gaps where a µservice's handler is missing. | LEAN check `governance-dsr-handler-conformance` refuses µservice without registered handler; quarterly drill. | M |
| T-L-08 | Linkability | Cell-assignment record ↔ tenant business behavior | Cell load patterns could reveal a tenant's traffic shape (linkability via timing). | Cell-load aggregates published at pack level only; per-tenant cell load is `BEHAVIORAL_TENANT_PRODUCT` (not cross-tenant exposed). | L |
| T-L-09 | Non-repudiation | Workflow event emitter | All Workflow events SPIFFE-signed + audit-chain-sealed; emitter cannot deny. | – | L |
| T-L-10 | Disclosure | Proof-of-erasure exposed to wrong regulator | Regulator A could read tenant T's proof when only Regulator B has jurisdiction. | Cedar policy on regulator-scope.cedar enforces jurisdiction-bound read. | L |
| T-L-11 | Non-compliance | Children's data (DPDPA 2023 §9; pack-in) | DPDPA requires parental consent for children's data; tenancy doesn't directly collect age but inherits via tenant DPA. | Tenant DPA includes child-data clause; tenant attests parental-consent process. | L (residual depends on tenant) |
| T-L-12 | Non-compliance | PHI without BAA (pack-us-healthcare) | A tenant in pack-us-healthcare must sign BAA before ingest; tenancy enforces gate. | Onboarding pre-flights BAA; non-signed tenants pinned to non-PHI pack. | L |

## Mitigations Catalog (cross-reference)

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Postgres RLS `FORCE ROW LEVEL SECURITY` on all tenant-bound tables | Preventive | axis-tenancy + ops-security | `governance-rls-force-on-tenant-tables` lane |
| LEAN check `governance-rls-no-superuser-bypass` | Preventive | axis-tenancy + ops-security | PR-time CI lane |
| JWT Ed25519 signing with OpenBao-managed key + 30d rotation + fingerprint advertise | Preventive | ops-security | `governance-jwt-key-fingerprint-advertised` lane |
| Patroni HA with 3-node minimum + quorum DCS | Preventive (availability) | ops-sre-reliability | Quarterly failover drill |
| Citus shard-by-tenant_id + transactional rebalance | Preventive | axis-tenancy | Monthly rebalance integrity drill |
| 2-person rule for tenant create + delete + DBA-JIT | Preventive (insider) | ops-security | OpenBao audit log + audit-chain seal |
| Cedar policy enforcement on every REST request | Preventive | ops-security + axis-tenancy | `governance-tenancy-cedar-coverage` lane |
| DSR cascade with per-µservice receipts + proof-of-erasure Merkle root | Detective + Non-repudiation | council-privacy + audit-chain | Quarterly DSR drill |
| Audit-chain Ed25519 seal on every lifecycle event | Detective + Non-repudiation | audit-chain | Audit-chain regression tests |
| Per-tenant rate limits at WAF + ingress | Preventive (DoS) | ops-sre-reliability | WAF dashboards |
| Per-pack pinning (no cross-pack movement default) | Preventive (residency) | axis-tenancy + council-privacy | `governance-tenancy-residency-conformance` lane |
| Continuous DB-state validator (Postgres role + RLS policy drift) | Detective | ops-security + axis-tenancy | 5-min cadence + Sev-1 page on drift |
| Soft-delete with 30d recovery window default | Preventive (insider; accidental) | council-privacy + axis-tenancy | Hard-delete after grace; recovery path documented |

## Residual Risk Acceptance

Residual risks above L (low) require explicit acceptance signed by `council-architecture` + `ops-security` + `council-privacy`:

| Risk ID | Residual | Why accepted | Re-review date |
|---|---|---|---|
| T-I-05 (DSR cascade incomplete) | M | Engineering-discipline floor on per-µservice handler completeness; mitigated via LEAN check + quarterly drill + per-µservice handler registry. | Quarterly |
| T-I-06 (secret leak via logs) | M | Human-error baseline; mitigated to acceptable via detection + rotation. | Quarterly |
| T-L-06 (end-user unawareness) | M | Tenant-of-tenant responsibility; joint-controllership clause covers. | Annually |
| T-L-07 (Art. 17 cascade gap) | M | Same as T-I-05; engineering discipline. | Quarterly |

Sign-off:

- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`

## Per-Pack Overlay Sections

### pack-kr (Korea)

Compliance frameworks engaged: KR-ISMS-P + KR PIPA + KR 전자문서법.

Additional considerations:
- **KR PIPA Art. 23 (sensitive personal information)**: `tenant_id` hashed with salt; salt rotation 12mo. Mitigation in T-L-02.
- **KR PIPA Art. 23-2 (sensitive data outside-of-KR transfer)**: pack-kr tenant data stays in KR Postgres cluster; cross-pack replication forbidden by default. Enforced in `multi-region.md`.
- **KR PIPA Art. 29 (technical safeguards)**: every T-*-NN mitigation cross-maps to one of the 12 prescribed safeguards in Art. 29.
- **KR PIPA Art. 29-2 (encryption)**: TLS 1.3 in transit + AES-256-GCM at rest + Ed25519 audit-chain seals.
- **KR PIPA Art. 33 (DPIA)**: tenancy DPIA at `microservices/tenancy/dpia.md` satisfies; engaged.
- **KR PIPA Art. 33-2 (DPO appointment)**: council-privacy chair registered with PIPC.
- **KR PIPA Art. 34 (breach notification)**: 72h to PIPC + 72h to data subjects per `incident-response.md`.
- **KR PIPA Art. 36 (right-to-deletion)**: DSR cascade fulfils within 30d.
- **KR-FSS sector guidance** (financial-services tenants): audit log retention ≥ 5y; encrypted at rest with KMS keys in KR-resident KMS.
- **KR-ISMS-P §2.5 (인적보안)** + **§2.7 (접근통제)**: 2-person rule + JIT elevation map.

### pack-us-healthcare (HIPAA-scoped)

Compliance frameworks engaged: HIPAA + state-level (CCPA / CMIA / etc.) — full mapping in `compliance.md`.

Additional considerations:
- **HIPAA §164.312(a)(1) (access control)**: per-tenant JWT + RLS + Cedar policy.
- **HIPAA §164.312(b) (audit controls)**: audit-chain emission on every PHI-touching operation; retention ≥ 6y per §164.316(b)(2).
- **HIPAA §164.502(b) (minimum-necessary)**: tenant data never echoed cross-tenant; per-tenant minimum scope enforced.
- **HIPAA §164.514 (de-identification)**: hashed tenant_id pattern; pseudonymisation default.
- **HIPAA §164.308(a)(4)(ii)(B) (access authorization)**: auditor tokens scoped per T-S-05.
- **45 CFR §164.504(e) (Business Associate Agreement)**: oyatie acts as BA for HIPAA-scope tenants; BAA template at `microservices/tenancy/legal/baa-template.md`.

### pack-eu (GDPR + EDPB + NIS2 + DORA)

Additional considerations:
- **GDPR Art. 25 (privacy by design)**: pseudonymisation + RLS default + DSR cascade default.
- **GDPR Art. 32 (security of processing)**: every "T-*-NN" mitigation contributes.
- **GDPR Art. 35 (DPIA)**: this threat-model + `dpia.md` together satisfy.
- **GDPR Art. 28 (processor terms)**: oyatie acts as processor for tenant operational data; DPA template at `microservices/tenancy/legal/dpa-template.md`.
- **GDPR Arts. 44–50 (transfers)**: pack-eu data EU-resident; cross-pack forbidden by default; tenant-executed SCCs path.
- **NIS2 (2022/2555)**: when oyatie crosses Annex I/II thresholds, the 24h + 72h + 1mo NIS2 reporting timelines apply.
- **DORA (2022/2554)**: financial-services tenants in pack-eu trigger DORA Chapter II + Chapter III + Chapter VI requirements; ICT-risk register + testing program + third-party-risk policy mapped.
- **eIDAS 910/2014**: Ed25519 audit-chain seals + proof-of-erasure certificates can be presented as AdES (advanced electronic signature); QES requires certified TSP (scheduled-for-distinct-tracked-work per ADR-####).

### pack-jp (APPI)

Additional considerations:
- **APPI Art. 17 (purpose of use)**: declared at tenant-onboarding.
- **APPI Art. 21 (cross-border transfer)**: pack-jp data JP-resident.
- **APPI Art. 27 (sensitive data consent)**: tenant DPA captures consent.
- **APPI Art. 26-2 (breach notification)**: 72h to PPC.

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/tenancy-overlay.md` follow this document's structure with the local PII law's articles substituted in.

## Compliance Cross-Mapping (Globally Enforced)

| Framework | Coverage | Mapping doc |
|---|---|---|
| SOC 2 Type 2 | CC1–CC9 covered; cited inline | `microservices/tenancy/compliance.md` |
| ISO 27001:2022 | Annex A.5–A.8 covered; cited inline | `microservices/tenancy/compliance.md` |
| GDPR | Arts. 5, 6, 9, 13, 14, 17, 22, 25, 26, 28, 30, 32, 33, 34, 35, 44 cited inline | `microservices/tenancy/dpia.md` + `compliance.md` |

## Re-review Triggers

This threat model re-reviews on:

- Any change to the trust boundary diagram (new boundary, removed boundary, modified actor).
- Any Layer-A version upgrade (Postgres / Citus / Patroni) where the upstream release notes mention security fixes.
- Any new pack activation (e.g., first pack-us-healthcare tenant onboarding triggers HIPAA-specific deep-dive).
- Annual scheduled review (Q2 each year).
- Post-incident review (any Sev-1 or Sev-2 incident in tenancy).
- Pen-test or audit finding.

## References

- ADR-0018 (Bominal): Tenancy + RLS posture; inherited.
- ADR-0028 (Bominal): Audit chain (Merkle + Ed25519); inherited.
- ADR-0056: BNF v4.1.
- ADR-0105: 13-layer enum.
- ADR-0117: Cloud-native infrastructure (data residency).
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- ADR-0140: Cedar policy enforcement.
- `microservices/tenancy/PRD.md`.
- `microservices/tenancy/dpia.md`.
- `microservices/tenancy/compliance.md`.
- `microservices/tenancy/policy/rls-isolation.md`.
- `microservices/tenancy/policy/data-residency.md`.
- `microservices/tenancy/incident-response.md`.
- Microsoft Threat Modeling methodology (STRIDE).
- LINDDUN privacy-threat methodology — Wuyts et al., KU Leuven.
- OWASP Top 10 (2021) + OWASP API Top 10 (2023).
- NIST SP 800-154 (Guide to Data-Centric System Threat Modeling).
- Postgres RLS documentation — `postgresql.org/docs/16/ddl-rowsecurity.html`.
- Citus docs — `docs.citusdata.com`.
- Patroni docs — `patroni.readthedocs.io`.
- ICO DPIA template — `ico.org.uk`; CNIL DPIA methodology — `cnil.fr/en/PIA`; PIPC Notice 2020-7.
