---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: ontology
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-ontology + ops-security
deciders: council-architecture, ops-security, axis-ontology, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + NIST SP 800-154
related_adrs: [ADR-0006, ADR-0028, ADR-0055, ADR-0056, ADR-0059, ADR-0106, ADR-0107, ADR-0117, ADR-0122, ADR-0123, ADR-0130, ADR-0131, ADR-0132, ADR-0140]
related_specs: [/specs/products/ontology.json, /specs/knowledge-graph-schema.json, /specs/per-microservice-flat-layout.json]
review_cadence: quarterly + on every Cedar / Postgres / ClickHouse / Agent gateway architecture change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.5.34, A.8.2, A.8.3, A.8.4, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.25, A.8.26, A.8.27, A.8.28, A.8.32, A.8.33"
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35"
suggested_frameworks_by_pack:
  pack-kr: ["KR-ISMS-P §2.1-2.12", "KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2/33/34", "KR 전자문서법 Arts. 5/6/7"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308/.310/.312/.314/.316/.502/.514"]
  pack-eu: ["GDPR Arts. 25 + 32 + 35 + 44-50", "eIDAS 910/2014", "NIS2 2022/2555"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24/26-2"]
  pack-sg: ["PDPA 2012 §11-26", "MAS-TRM v2021"]
  pack-au: ["Privacy Act 1988 APP 1-13", "APRA-CPS 234"]
  pack-in: ["DPDPA 2023 §6-13", "RBI Master Direction on IT Outsourcing 2023"]
  pack-br: ["LGPD Arts. 6/7/11/14/18/33/46/48", "BACEN Res. 4.893/2021"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021"]
  pack-ksa: ["KSA PDPL Royal Decree M/19/2021", "SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Threat Model: ontology µservice

## Purpose

Identify, classify, and mitigate threats to the ontology µservice's confidentiality, integrity, availability, and privacy posture. Because Ontology is the canonical information adapter (every µservice's data flows through it per ADR-0059), a compromise here cascades to every product — RLS bypass leaks every tenant, Cedar bypass invokes every Action, audit-chain tamper invalidates every provenance claim. This document is the canonical security artifact reviewed by SOC 2 Type 2 examiners, ISO 27001 auditors, GDPR DPAs, KR PIPC, and HIPAA Covered Entity counsel at first-tenant onboarding.

## Scope

### In-scope

All components introduced by ADR-0006 (typed-entity layer), ADR-0059 (ecosystem adapter), Bominal ADR-0106 (Ontology architecture), Bominal ADR-0107 (agent gateway), and ADR-0131 (per-microservice flat layout) for the ontology µservice, deployed in a **shared substrate Kubernetes cluster** (decision: shared with other Layer-B substrates such as tenancy + audit-chain; matches hyperscaler practice — Palantir Foundry Ontology runs on shared Foundry cluster):

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| PostgreSQL 16 + Citus 12 (sharded by `tenant_id`) | `oya-ontology-object-type-registry-*` |
| ClickHouse 24 (history-mirror; OLAP) | `oya-ontology-link-type-registry-*` |
| Valkey 8 (schema-registry hot cache) | `oya-ontology-action-type-registry-*` |
| Cedar v4 policy engine (Rust SDK) | `oya-ontology-function-type-registry-*` |
| Apache Kafka KRaft (outbox per ADR-0050) | `oya-ontology-entity-store-*` |
| OpenBao (audit-signing key custody) | `oya-ontology-link-store-*` |
| Istio mTLS (inter-pod) | `oya-ontology-function-engine-*` |
| OPA (sidecar fallback; not primary) | `oya-ontology-action-engine-*` |
| | `oya-ontology-cedar-fragment-coverage-*` |
| | `oya-ontology-query-engine-*` (3-layer KG) |
| | `oya-ontology-agent-gateway-*` |
| | `oya-ontology-audit-chain-*` |
| | `oya-ontology-pillar-*` |

### Out-of-scope

- Threats to Kubernetes cluster / OCI substrate — owned by `cloud-k8s` µservice's threat model.
- Threats to OpenBao secret-manager itself — owned by `cloud-secrets`; this document inherits OpenBao threats as upstream.
- Threats to OpenSLO / Prometheus / Mimir — owned by `observability`.
- Threats to Workflow µservice (the action plane) — owned by `workflow` µservice's threat model.
- Threats to Bominal-side Ontology counterpart — inherited per `feedback_bominal_inheritance_precedence`.

## Trust Boundaries

```text
┌─ Internet ─────────────────────────────────────────────────────────────────┐
│                                                                            │
│  Tenant operators        Customer applications        LLM agents           │
│        │                          │                         │              │
│        │ (OIDC + MFA)              │ (per-tenant API key)    │ (Cedar      │
│        ▼                          ▼                          │  autonomy-  │
│                                                              │  tier JWT)  │
│  ┌─ Public ingress (Envoy/Istio gateway) ──────────────────────────────┐   │
│  │  - TLS 1.3 termination                                              │   │
│  │  - WAF (rate-limit + OWASP CRS)                                     │   │
│  │  - DDoS protection (provider + Cloudflare)                          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │                                             │
└──────────────────────────────│─────────────────────────────────────────────┘
                               ▼
┌─ Shared substrate Kubernetes cluster ──────────────────────────────────────┐
│                                                                            │
│  Trust boundary 1: External → Cluster ingress                              │
│                                                                            │
│  ┌─ ontology-*-rest ────────────────┐    ┌─ agent-gateway-rest ─────────┐  │
│  │  OIDC tenant-scoped + Cedar      │    │  LLM JWT + autonomy_tier     │  │
│  │  policy evaluation               │    │  ceiling + Cedar             │  │
│  └──────────────────────────────────┘    └──────────────────────────────┘  │
│             │                                              │               │
│  Trust boundary 2: REST → Cedar Policy Engine (every Action gated)         │
│             │                                              │               │
│  ┌─ Cedar Policy Engine (in-process Rust SDK) ─────────────────────────┐   │
│  │  - default-deny base; per-Action permit fragments                   │   │
│  │  - autonomy_tier ceiling on agent tool-calls                        │   │
│  │  - cross-pillar grant authority                                     │   │
│  │  - jurisdiction overlays per tenant                                 │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│             │                                                              │
│  Trust boundary 3: Cedar → Entity store + Audit chain emit                 │
│             │                                                              │
│  ┌─ Postgres 16 + Citus 12 (sharded by tenant_id) ──────────────────────┐  │
│  │  - FORCE ROW LEVEL SECURITY per Object Type table                    │  │
│  │  - app.tenant_id session var bound from JWT claim before query       │  │
│  │  - Citus shards by tenant_id; cross-shard queries forbidden by       │  │
│  │    config (multi_shard_modify_mode = strict)                         │  │
│  │  - Replicas (RF=3) for HA; PITR backup                               │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│             │                                                              │
│  Trust boundary 4: Postgres → outbox → Kafka → ClickHouse mirror           │
│             │                                                              │
│  ┌─ Kafka KRaft (outbox per ADR-0050) ──────────────────────────────────┐  │
│  │  - per-tenant topic routing                                          │  │
│  │  - audit-chain seal events emitted here                              │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│             │                                                              │
│  ┌─ ClickHouse 24 (OLAP mirror; read-only) ─────────────────────────────┐  │
│  │  - partition by (tenant_id, toYYYYMM(ts))                            │  │
│  │  - row policies for cross-tenant query refusal                       │  │
│  │  - 3-layer KG joins (semantic/kinetic/dynamic)                       │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│             │                                                              │
│  Trust boundary 5: audit-chain → OpenBao Ed25519 signing                   │
│             │                                                              │
│  ┌─ OpenBao Ed25519 key custody ────────────────────────────────────────┐  │
│  │  - per-tenant signing key (HSM-backed where available)               │  │
│  │  - 90d rotation; rotation event audit-chained                        │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

Five trust boundaries:
1. **External → Cluster ingress** (TLS, WAF, DDoS).
2. **REST → Cedar Policy Engine** (every Action evaluated; default-deny).
3. **Cedar → Entity store + Audit chain emit** (RLS bound to JWT tenant claim; audit-chain emit before state mutation).
4. **Postgres → outbox → Kafka → ClickHouse mirror** (durable replication; per-tenant topic routing).
5. **audit-chain → OpenBao Ed25519 signing** (HSM-backed; rotation audit-chained).

## Assets & Data Classification

Per Bominal ADR-0028 (data-class taxonomy) + ADR-0008 (DUB) + `oya-check-data-class` LEAN lane.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| Object Type instances (canonical entities — Patient, Payslip, Order, etc.) | varies per `PropertyTier`: `Tier1Sensitive` ⇒ `SENSITIVE_*` / `PHI` / `PII_IDENTIFYING`; `Tier2Restricted` ⇒ `BEHAVIORAL_TENANT_PRODUCT`; `Tier3Internal` ⇒ `INTERNAL_ONLY`; `Tier4Public` ⇒ `PUBLIC` | High | per data-residency.md retention matrix | Postgres + Citus |
| Link Type instances | inherited from connected Object Types' MAX tier | varies | same as Object Type | Postgres + Citus |
| Action Type invocations + receipts | `AUDIT` | High | ≥ 1y default; ≥ 6y for pack-us-healthcare; ≥ 3y for pack-kr-FSS | Postgres + audit-chain Merkle tree |
| Function evaluation results (cached) | varies by Function | High (per-tenant) | TTL ≤ 5 min hot cache; not persisted long-term | Valkey + ephemeral |
| ClickHouse history-mirror rows | mirror of Object Type tier | varies | 24 mo cold + retention matrix | ClickHouse |
| Audit-chain Merkle nodes + Ed25519 seals | `AUDIT` | Critical | append-only; immutable | audit-chain Postgres + audit-chain µservice (cross-µservice) |
| Cedar policy fragments | `INTERNAL_ONLY` (policy text); `SECRET`-class when carrying tenant identifiers | Medium | git history | `microservices/ontology/policy/*.cedar` |
| Schema-registry cache (Valkey) | `INTERNAL_ONLY` | Low | TTL ≤ 60 s; reconstructable | Valkey |
| Ontology agent gateway LLM tool-call payloads | varies by Function permitted | High (may carry tenant identifiers in tool args) | 30 d (operational); subject to DSR | Loki + Postgres action log |
| Per-tenant Postgres role credentials | `SECRET` | Critical | rotated 30 d | OpenBao |
| Ed25519 audit-chain signing keys (per-tenant or per-pack) | `SECRET` | Critical | rotated 90 d; HSM-backed | OpenBao |
| Tenant identifiers (mapped to Postgres app.tenant_id session var) | `SENSITIVE_PIPA_ART23` | High | mapping in OpenBao; raw never persisted in Ontology | OpenBao |
| KMS-wrapped DEK per tenant per ciphertext property | `SECRET` | Critical | per ADR-0111 ciphertext property | OpenBao/KMS |

## Actors

| Actor | Trust level | Authentication | Capability |
|---|---|---|---|
| External tenant operator (human) | Untrusted external | OIDC + MFA via Application Shell | Read/write own tenant's Object Types per Cedar; author Object Type schemas via PR |
| Customer application (machine) | Untrusted external | Per-tenant API key bound to tenant ID claim | Read/write own tenant's Object Types per Cedar |
| Workload µservice (internal) | Semi-trusted internal | mTLS + per-µservice SPIFFE identity | Read/write own tenant's Object Types via SDK |
| LLM agent (e.g., Claude / GPT) | Untrusted external (semi-trusted via Cedar autonomy_tier) | LLM JWT with autonomy_tier claim | Call Functions / Actions within autonomy_tier ceiling via agent-gateway |
| Workflow µservice (peer adapter) | Trusted internal | mTLS + SPIFFE | Read Ontology Functions; emit ObjectInstanceMutated subscribers |
| ontology-action-engine-worker | Trusted internal | OpenBao-issued SA token | Invoke Cedar; write Object Types; emit audit-chain seals |
| Audit-chain µservice (downstream consumer) | Trusted internal | mTLS | Read seal events; mirror to cross-µservice audit-chain authority |
| Council-privacy + ops-security operators | Trusted internal | OIDC + MFA + JIT elevation via OpenBao | Admin reads; cross-pillar grant approvals; manual override |
| External auditor (SOC 2 / ISO 27001 / HIPAA / KR-PIPC) | Read-only external on time-boxed window | OIDC + MFA + JIT short-lived token | Read-only on policy artifacts + scoped tenant data |
| DSR cascade runner (internal automation) | Trusted internal | SPIFFE identity | Locate + tombstone subject identifiers across Object Types |
| Attacker — opportunistic | Untrusted | none | Scans + low-skill exploitation |
| Attacker — targeted | Untrusted | none | Sophisticated; supply-chain awareness |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure schemas / Cedar fragments (mitigated by PR review + LEAN) |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case threat actor; mitigated by least-privilege + audit-chain + separation-of-duties |

## STRIDE Threat Catalog

Each threat: ID; category; asset; description; likelihood (L/M/H); impact (L/M/H); risk; mitigations; owner; residual; framework coverage.

### Spoofing (S)

**T-S-01 — Tenant-A submits Object Type write claiming `tenant_id` of Tenant-B**
- Asset: Postgres RLS boundary
- L: M / I: H / Risk: **H**
- Mitigations:
  - Per-tenant API key bound to tenant ID claim (signed by OpenBao); JWT verification before request reaches Cedar.
  - `app.tenant_id` Postgres session variable bound from the JWT claim by middleware; raw tenant ID never accepted from request body.
  - `FORCE ROW LEVEL SECURITY` policy on every Object Type table; mismatch → 0 rows; never an exception.
  - LEAN check `oya-foundry-fitness-ontology-tenancy-isolation` greps for any code path that sets `app.tenant_id` from anything other than `req.auth.tenant_id`.
- Owner: ops-security + axis-ontology
- Residual: L (key compromise required + audit visibility on attempt)
- Frameworks: SOC 2 CC6.1, CC6.2, CC6.6; ISO 27001 A.5.15, A.5.17, A.8.2, A.8.3; GDPR Art. 32(1)(a)(b); KR PIPA Art. 29

**T-S-02 — Attacker impersonates `ontology-action-engine-worker` SA token**
- Asset: Action engine write path
- L: L / I: H / Risk: **M**
- Mitigations:
  - Worker SA token bound to pod identity (SPIFFE SVID); cannot be used outside cluster.
  - Token rotation 24 h.
  - Postgres `pg_hba.conf` restricts the worker SA to specific source pods (Kubernetes network policy enforced via Cilium).
  - Action receipts include SPIFFE identity; tampering detected at Cedar evaluation time.
- Owner: ops-security + axis-ontology
- Residual: L
- Frameworks: SOC 2 CC6.1, CC7.1; ISO 27001 A.5.15, A.8.5, A.8.7; GDPR Art. 32(1)(b)

**T-S-03 — LLM agent impersonates higher autonomy tier**
- Asset: agent-gateway autonomy_tier ceiling
- L: M / I: H (would permit Action invocations beyond policy) / Risk: **H**
- Mitigations:
  - LLM JWT issued by `agent-runtime` µservice with autonomy_tier claim bound at issuance; cannot be self-modified.
  - Cedar policy `agent-gateway-scope.cedar` enforces `principal.autonomy_tier >= Action.required_tier`; missing or low tier → 403.
  - Per-Action `required_tier` is part of `ActionTypeSchema` (not configurable per request).
  - Every agent tool-call audit-chained with the actual JWT autonomy_tier; tampering detectable post-hoc.
- Owner: axis-ontology + council-architecture
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.1; ISO 27001 A.5.15, A.8.3; GDPR Art. 22 (automated decision-making); pack-eu EU AI Act Art. 13 (transparency for high-risk AI) when applicable

**T-S-04 — Attacker forges audit-chain seal**
- Asset: Audit-chain Ed25519 signature
- L: L / I: H / Risk: **M**
- Mitigations:
  - Ed25519 signing keys live in OpenBao; never extractable; signing operations via OpenBao's Transit API.
  - Per-tenant keys (HSM-backed where available); rotation event audit-chained.
  - Verifier checks signature against the public key pinned at the time of seal (key rotation chains carry forward).
  - Mass-forgery would require OpenBao compromise; OpenBao is in `cloud-secrets` µservice's threat model.
- Owner: cloud-secrets + axis-ontology + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.2; ISO 27001 A.5.17, A.8.7, A.8.24; GDPR Art. 32(1)(b); pack-eu eIDAS Art. 26

**T-S-05 — External auditor JIT token used to write (privilege escalation)**
- Asset: Auditor read scope
- L: L / I: H / Risk: **M**
- Mitigations:
  - Auditor JIT tokens scoped read-only at Cedar (`auditor-scope.cedar` forbids all write Actions).
  - Token TTL ≤ 4 h; non-renewable without ops-security re-issue.
  - Token scope pinned to specific tenants subset; cross-tenant pivot blocked by `auditor-scope.cedar` forbid clause.
  - Every auditor read audit-chained (audit-of-audits).
- Owner: ops-security + council-privacy
- Residual: L
- Frameworks: SOC 2 CC6.1, CC8.1; ISO 27001 A.5.15, A.8.34; GDPR Art. 28; pack-us-healthcare HIPAA §164.308(a)(4)(ii)(B)

### Tampering (T)

**T-T-01 — Object Type schema tampering via repo push**
- Asset: `microservices/ontology/specs/object-types/*.json` schemas
- L: M / I: H (false schema → false tier classification → property-tier escape) / Risk: **H**
- Mitigations:
  - All Object Type schema changes via PR review with `oya-pr-review` lane.
  - CODEOWNERS for `specs/object-types/` scoped to `axis-ontology + council-privacy`.
  - LEAN check `oya-foundry-fitness-ontology-tier-enforcement` validates: every property declares `data_class` + `property_tier`; refused if absent.
  - Schema regression test asserts every existing tenant's data continues to validate under the new schema.
- Owner: axis-ontology + council-privacy
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.31, A.5.32, A.8.32, A.8.33

**T-T-02 — Cedar policy fragment tampering via repo push**
- Asset: `microservices/ontology/policy/*.cedar`
- L: M / I: H (false permit → unauthorised Action invocation) / Risk: **H**
- Mitigations:
  - All Cedar fragment changes via PR review with `oya-pr-review` + `oya-foundry-fitness-cedar-coverage` lane.
  - LEAN `cedar-coverage` validates: every registered Action Type has at least one permit AND a default-deny clause.
  - Fuzz testing in CI (`oya-check-cedar-fragment-coverage`): random `(principal, action, resource, context)` tuples; deny is expected unless covered by explicit permit.
  - Cedar v4 (no template-based escape vectors known).
- Owner: axis-ontology + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.15, A.8.32; GDPR Art. 32(1)(b)

**T-T-03 — Postgres RLS policy drift (live mutation disables RLS on a table)**
- Asset: Postgres RLS configuration
- L: L (requires DB superuser; gated by JIT elevation) / I: H / Risk: **M**
- Mitigations:
  - Postgres roles for application paths are non-superuser; cannot `ALTER TABLE ... DISABLE ROW LEVEL SECURITY`.
  - Superuser JIT via OpenBao with 2-person rule; every superuser session audited.
  - Continuous Helm-state validator + `pg_dump --schema-only` diff CronJob detects schema drift hourly; mismatch with declared state alarms.
  - LEAN lane `oya-foundry-fitness-ontology-tenancy-isolation` includes a runtime probe that performs a cross-tenant query attempt; non-zero rows returned = lane fail.
- Owner: ops-security + axis-ontology + cloud-infra
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.17, A.8.5, A.8.7, A.8.32; GDPR Art. 32(1)(b)

**T-T-04 — Citus cross-shard query mode loosening**
- Asset: Citus `multi_shard_modify_mode` configuration
- L: L / I: H / Risk: **M**
- Mitigations:
  - Helm chart pins `multi_shard_modify_mode = 'strict'`; cross-shard modify forbidden.
  - LEAN lane validates Helm config hash at deploy + monthly.
  - Cross-shard read queries allowed only for OLAP via ClickHouse mirror; never against Citus directly for write paths.
- Owner: axis-ontology + cloud-infra
- Residual: L
- Frameworks: SOC 2 CC6.6; ISO 27001 A.8.32

**T-T-05 — Audit-chain Merkle node tampering**
- Asset: Audit-chain Postgres + mirror table
- L: L / I: H / Risk: **M**
- Mitigations:
  - Audit-chain rows append-only via Postgres trigger; UPDATE / DELETE refused.
  - Merkle root computed deterministically from row contents; tampering detected on verification.
  - Periodic cross-check against the `audit-chain` µservice's cross-microservice Merkle authority (chain-of-chains).
  - Mass-tampering would require Postgres superuser + OpenBao Ed25519 key extraction; both gated.
- Owner: axis-ontology + audit-chain µservice
- Residual: L
- Frameworks: SOC 2 CC4.1, CC8.1; ISO 27001 A.5.27, A.5.28, A.8.15; GDPR Art. 5(2), Art. 30; pack-eu eIDAS

**T-T-06 — Outbox-to-Kafka tampering (silent skip causes silent missing audit)**
- Asset: Outbox pattern per ADR-0050
- L: L / I: M / Risk: **M**
- Mitigations:
  - Outbox is a Postgres table; same RLS + append-only constraints.
  - Kafka consumer in `audit-chain-worker` confirms outbox consumption via offset commit + completeness check (count(outbox_rows) == count(emitted_seals)).
  - `oya:ontology_audit_chain_completeness:rate == 1.0` SLO; any breach pages.
- Owner: axis-ontology + audit-chain µservice
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.5.27, A.8.15; GDPR Art. 5(2)

### Repudiation (R)

**T-R-01 — Action invoker denies authorship**
- Asset: ActionInvocation receipt
- L: L / I: M / Risk: **L-M**
- Mitigations:
  - Every Action Type invocation receipt carries `actor_principal_spiffe_id` + `idempotency_key` + Ed25519 audit-chain seal.
  - Per-tenant Merkle root + chain-of-chains provides tamper-evident proof.
  - PR commit signature requirement on `Cargo.toml` + schema files (branch-protection).
- Owner: axis-ontology + audit-chain
- Residual: L
- Frameworks: SOC 2 CC4.1, CC8.1; ISO 27001 A.5.27, A.8.34; GDPR Art. 5(2)

**T-R-02 — Schema-registry change denied by author**
- Asset: ObjectTypeSchema authoring history
- L: L / I: M / Risk: **L**
- Mitigations:
  - Commits to `specs/object-types/` require signed commits per branch-protection.
  - PR audit log on every change.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.8.34; GDPR Art. 5(2)

**T-R-03 — DSR erasure executed but cascade is incomplete (gap claim)**
- Asset: DSR cascade audit
- L: M / I: M / Risk: **M**
- Mitigations:
  - DSR runner emits `DsrExecuted{subject_hash, removed_object_type_count, residual_object_types[], executed_at}`; residual list is explicitly recorded.
  - Per-Object-Type completeness check: scan every Citus shard; emit per-tenant completeness manifest.
  - Tenant-facing DSR confirmation includes the manifest hash; tenant can re-verify.
- Owner: council-privacy + axis-ontology
- Residual: M (retention-window edge cases; documented limitation)
- Frameworks: SOC 2 CC4.1, CC8.1; ISO 27001 A.5.34; GDPR Art. 17

### Information Disclosure (I)

**T-I-01 — Cross-tenant read via PromQL-style query (Function bypassing tenant_id filter)**
- Asset: Postgres + ClickHouse cross-tenant boundary
- L: M / I: H / Risk: **H**
- Mitigations:
  - Postgres RLS is the load-bearing control; even if a Function omits `tenant_id`, RLS returns rows for the bound tenant only.
  - ClickHouse row-policies (per-row filter on `tenant_id = currentDatabase()` analog using `getSetting('app_tenant_id')`).
  - LEAN runtime probe in `oya-foundry-fitness-ontology-tenancy-isolation`: synthetic cross-tenant query attempt; non-zero rows = lane fail.
  - Penetration test annually + on every Postgres/Citus/ClickHouse upgrade.
- Owner: ops-security + axis-ontology
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.5.18, A.8.2, A.8.3, A.8.12; GDPR Art. 5(1)(f), Art. 25, Art. 32; pack-kr KR PIPA Art. 23; pack-us-healthcare HIPAA §164.312(a)(1)

**T-I-02 — Cross-tenant Link Type creation (link Object-A in tenant-1 to Object-B in tenant-2)**
- Asset: Link Type cross-tenant boundary
- L: M / I: H / Risk: **H**
- Mitigations:
  - Link Type adapter checks both endpoints' `tenant_id` against `app.tenant_id`; mismatch returns `CrossTenantLinkDenied`.
  - Cedar `CrossTenantLinkGrant` permit required for the rare legitimate cross-tenant link (e.g., marketplace integration); 2-person rule + audit-chained.
  - LEAN lane `oya-foundry-fitness-ontology-cross-tenant-link` greps the adapter for any code path that creates links without dual-tenant check.
- Owner: axis-ontology + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.3; GDPR Art. 5(1)(f), Art. 32; pack-kr KR PIPA Art. 23

**T-I-03 — Cross-pillar leak (org-pillar Object Type read via person-pillar context)**
- Asset: Pillar boundary
- L: M / I: H / Risk: **H**
- Mitigations:
  - `pillar.cedar` evaluator checks `principal.pillar_kind == Action.required_pillar` OR `cross_pillar_grant in principal.grants`.
  - Cross-pillar grants require explicit Cedar entitlement issued via 2-person rule.
  - Every cross-pillar Function read audit-chained.
  - Annual pen-test against pillar boundary.
- Owner: axis-ontology + council-privacy
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.3; GDPR Art. 32; pack-kr KR PIPA Art. 17 + 18 + 23 (sensitive personal info); pack-us-healthcare HIPAA §164.502(b) (minimum necessary)

**T-I-04 — ClickHouse history-mirror leaks pre-RLS-policy data**
- Asset: ClickHouse mirror
- L: L / I: H / Risk: **M**
- Mitigations:
  - ClickHouse row policies enforce per-tenant scope; cross-tenant queries refused server-side.
  - Outbox-to-Kafka emits only RLS-cleared rows; raw Postgres rows never replicated.
  - Periodic integrity check: random tenant; verify ClickHouse rows match Postgres rows for that tenant + no extra.
- Owner: axis-ontology
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.12; GDPR Art. 25

**T-I-05 — Agent gateway leaks per-tenant data via LLM context window**
- Asset: LLM tool-call return payload
- L: M / I: H (LLM provider may retain tool-call data) / Risk: **H**
- Mitigations:
  - LLM provider's data-retention policy reviewed at onboarding; per-tenant DPA captures.
  - Function results passed to LLMs are tier-filtered: Tier1Sensitive properties masked unless Cedar explicitly permits.
  - Agent-gateway rate limit + per-tenant audit emission on every tool-call.
  - Tenant-facing toggle: "allow LLM access to Tier1Sensitive properties" (default OFF; opt-in only).
- Owner: axis-ontology + council-privacy
- Residual: M (LLM-provider trust boundary irreducible)
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.34; GDPR Art. 5(1)(c), Art. 25, Art. 28, Art. 32; pack-eu EU AI Act Art. 26

**T-I-06 — Property-tier escape via Function projection**
- Asset: Function-engine projection
- L: M / I: H (Tier1Sensitive property exposed to Tier4Public reader) / Risk: **H**
- Mitigations:
  - Function evaluator validates: result-shape tier ≤ caller's max-tier ceiling (caller carries `max_tier` claim).
  - LEAN `oya-foundry-fitness-ontology-tier-enforcement`: every Function projection covers all declared `data_class` propagation rules.
  - Property-tier ceiling enforced server-side at projection time; never trusted from request.
- Owner: axis-ontology
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.34, A.8.11; GDPR Art. 5(1)(c); pack-kr KR PIPA Art. 18

**T-I-07 — Audit-chain seal leak (Ed25519 private key exfiltrated)**
- Asset: Ed25519 signing key
- L: L / I: H / Risk: **M**
- Mitigations:
  - Keys live in OpenBao; never extractable.
  - OpenBao Transit API rate limits + per-key audit.
  - 90 d rotation; legacy key shredded after grace.
  - HSM-backed where available (pack-us-healthcare + pack-eu).
- Owner: cloud-secrets + axis-ontology
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.17, A.8.7, A.8.24; GDPR Art. 32(1)(a)

### Denial of Service (D)

**T-D-01 — Function engine OOM via runaway projection**
- Asset: Function engine
- L: H / I: H (Ontology unavailable for everyone if engine pods OOM) / Risk: **H**
- Mitigations:
  - Per-Function timeout (default 5 s); per-tenant concurrency limit.
  - Function evaluator validates query plan EXPLAIN before execution; rejects unbounded scans.
  - HPA scaling on engine CPU.
  - Function cardinality budget per tenant; excess returns 429.
- Owner: ops-sre-reliability + axis-ontology
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6; GDPR Art. 32(1)(c)

**T-D-02 — Postgres ingester saturation via burst Action writes from one tenant**
- Asset: Postgres write path
- L: H / I: H / Risk: **H**
- Mitigations:
  - Per-tenant write-rate limit at action-engine before reaching Postgres.
  - Citus distributor backpressure.
  - Outbox decouples Action receipt from full Postgres commit (best-effort latency; eventual write-ahead consistency).
- Owner: ops-sre-reliability + axis-ontology
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6; GDPR Art. 32(1)(c)

**T-D-03 — Cedar policy infinite-loop / runaway evaluation**
- Asset: Cedar policy engine
- L: L / I: H / Risk: **M**
- Mitigations:
  - Cedar v4 has bounded evaluation by design (no recursion, no loops).
  - Engine timeout (10 ms hard cap); bench tests in CI.
  - LEAN check refuses policies with `forbid` chains exceeding template depth limit.
- Owner: axis-ontology + ops-security
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.28

**T-D-04 — Audit-chain emission lag → Action engine backpressure**
- Asset: Audit-chain emit path
- L: M / I: M (Action writes slow when audit lag spikes) / Risk: **M**
- Mitigations:
  - Outbox decouples emit; Action engine commits to Postgres first, then emits.
  - Audit-chain worker scales on Kafka consumer lag.
  - If audit lag > 60 s, action-engine throttles to 50% to allow drain.
- Owner: axis-ontology + audit-chain
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.6

**T-D-05 — ClickHouse OLAP query DDoS (expensive aggregations)**
- Asset: Query engine 3-layer KG path
- L: M / I: M / Risk: **M**
- Mitigations:
  - Per-tenant ClickHouse query budget (memory + time).
  - Cache 5-min TTL for common joins.
  - Query queue prioritisation per `tenant_scope` (production > trial).
- Owner: ops-sre-reliability + axis-ontology
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.6

**T-D-06 — Agent gateway flood via runaway LLM tool-call loop**
- Asset: agent-gateway
- L: M / I: H / Risk: **H**
- Mitigations:
  - Per-LLM-session rate limit (default 100 calls/min).
  - Cedar autonomy_tier ceiling.
  - Circuit breaker: if per-LLM error rate > 50%, sessions paused for 5 min.
- Owner: axis-ontology
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6

### Elevation of Privilege (E)

**T-E-01 — Cedar policy escape via crafted Function arguments**
- Asset: Cedar evaluation context
- L: L / I: H / Risk: **M**
- Mitigations:
  - Cedar v4 default-deny baseline.
  - Function arguments type-checked against `FunctionTypeSchema` before Cedar evaluation.
  - Input bounds: max nested depth, max string length.
- Owner: axis-ontology + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC8.1; ISO 27001 A.5.15, A.8.28

**T-E-02 — Postgres superuser obtained → cross-tenant read**
- Asset: Postgres superuser role
- L: L / I: H / Risk: **M**
- Mitigations:
  - Superuser access via OpenBao JIT only; 2-person rule.
  - Every superuser session audit-chained.
  - Network policy: superuser-eligible bastion only reachable from ops-security workstation IPs.
  - Audit alert on `pg_stat_activity` anomalies.
- Owner: ops-security + cloud-infra
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.15, A.8.4

**T-E-03 — Pillar-context spoof via header injection**
- Asset: `principal.pillar_kind` claim
- L: L / I: H / Risk: **M**
- Mitigations:
  - Pillar context bound from JWT claim at gateway; never accepted from request body.
  - LEAN check refuses any code path that sets pillar context from header or body.
- Owner: axis-ontology + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.3

**T-E-04 — Action Type registration loophole (register a permissive Action and immediately invoke it)**
- Asset: Action Type registry write path
- L: L / I: H / Risk: **M**
- Mitigations:
  - Action Type registration requires Cedar fragment update via PR (no in-API registration of new Action Types).
  - Tenant-defined Actions register via signed manifest; signature verified.
  - Fragment-coverage lane fails the PR if any new Action lacks a permit.
- Owner: axis-ontology + ops-security
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.31, A.8.32

**T-E-05 — Cross-pillar grant issued without 2-person rule**
- Asset: Cross-pillar grant authority
- L: L (insider-malicious) / I: H / Risk: **M**
- Mitigations:
  - 2-person rule for cross-pillar grants enforced at Cedar (grant must carry 2 signatory claims).
  - Audit-chained; grant TTL ≤ 30 d unless explicitly renewed.
  - Operator JIT elevation via OpenBao.
- Owner: ops-security + council-privacy
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.15, A.5.27

## LINDDUN Privacy-Threat Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | Object Type instances across tenants | Aggregated cross-tenant aggregates may permit linkage of subject behavior across tenant boundaries | Differential privacy on cross-tenant aggregates (ε ≤ 1); per-tenant aggregates default | M |
| T-L-02 | Identifiability | Hashed `subject_id` in Object Types (e.g., a `Patient` carrying hashed national-id) | Hash plus auxiliary may re-identify | Salted hash; per-tenant salt; rotated annually; audit-chain notes rotation | L |
| T-L-03 | Non-repudiation | Action invocation receipt | Tenant denies authorship; receipt is the load-bearing proof | Signed Ed25519 receipt + Merkle chain; chain-of-chains across µservices | L |
| T-L-04 | Detectability | Object Type write timing | Burst of writes correlates with tenant business events (end-of-month batch, marketing campaign) | Inherent to BEHAVIORAL_TENANT_PRODUCT class; tenant onboarding DPA discloses | M |
| T-L-05 | Disclosure | Cross-tenant Link Type (legitimate marketplace integration) | A link grant unintentionally exposes adjacent tenant's data | Cedar `CrossTenantLinkGrant` with explicit `data_class` cap + property-mask | L |
| T-L-06 | Unawareness | End-user of tenant application | End-user not aware their behavior captured by Ontology | Tenant DPA mandates upstream disclosure to end-users (joint controllership per GDPR Art. 26) | M |
| T-L-07 | Non-compliance | GDPR Art. 17 right-to-erasure | End-user requests erasure; identifiers spread across Object Types, Links, audit-chain seals | DSR cascade scans every Object Type; tombstones identifiers; audit-chain entries remain (immutable) but identifiers in mutable Object Types removed; SLA 30 d | M (best-effort within retention) |
| T-L-08 | Non-compliance | Pillar boundary violation (org-pillar data accessed via person-pillar context) | Violates Bominal ADR-0132 pillar separation | Cedar pillar enforcement + audit-chained cross-pillar grants | L |
| T-L-09 | Linkability | LLM agent context window correlation | LLM provider may correlate tool-call payloads across sessions to infer subject identity | Provider DPA + tier-filtered tool-call results; tenant opt-in for Tier1Sensitive access | M |

## Mitigations Catalog (cross-reference)

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Postgres `FORCE ROW LEVEL SECURITY` | Preventive | axis-ontology + cloud-infra | `oya-foundry-fitness-ontology-tenancy-isolation` lane |
| Citus `multi_shard_modify_mode = strict` | Preventive | axis-ontology | Helm config hash check |
| Cedar v4 default-deny baseline | Preventive | axis-ontology + ops-security | `oya-foundry-fitness-cedar-coverage` lane |
| Cedar fragment-coverage CI lane | Preventive | axis-ontology | per-Action permit + default-deny |
| Per-Action autonomy_tier ceiling | Preventive | axis-ontology | runtime Cedar evaluator |
| Pillar Cedar fragment (`pillar.cedar`) | Preventive | axis-ontology + council-privacy | annual pen-test pillar boundary |
| Ed25519 audit-chain seal per Action | Detective + Non-repudiation | axis-ontology + audit-chain | Merkle root verification |
| Outbox-to-Kafka pattern | Preventive | axis-ontology | completeness rate SLO |
| Property-tier propagation in Function projection | Preventive | axis-ontology | `oya-foundry-fitness-ontology-tier-enforcement` |
| DSR cascade runner | Preventive (compliance) | council-privacy + axis-ontology | DSR queue SLO |
| Per-tenant API key + JWT bound tenant claim | Preventive | ops-security + cloud-secrets | OpenBao audit |
| Per-tenant ClickHouse row policy | Preventive | axis-ontology | ClickHouse policy review |
| Tier-filtered LLM tool-call payload | Preventive | axis-ontology + council-privacy | agent-gateway-scope.cedar |
| 2-person rule for cross-pillar + cross-tenant grants | Preventive (insider) | ops-security + council-privacy | OpenBao JIT logs |
| Per-tenant Ed25519 signing keys (HSM where available) | Preventive | cloud-secrets | OpenBao rotation audit |
| LEAN runtime probes (synthetic cross-tenant query) | Detective | axis-ontology | CI lane + production canary |

## Residual Risk Acceptance

Residual risks above L (low) require explicit acceptance signed by `council-architecture` + `ops-security` + `council-privacy`:

| Risk ID | Residual | Why accepted | Re-review date |
|---|---|---|---|
| T-I-05 (LLM context leak) | M | LLM-provider trust boundary irreducible; tier-filtered + opt-in mitigations bring it to acceptable | Quarterly |
| T-R-03 (DSR cascade gap) | M | Retention-window edge cases; documented limitation in DPA | Annually |
| T-L-01 (linkability cross-tenant) | M | DP injection bounds the leak; full elimination would defeat the cross-tenant analytics value | Annually |
| T-L-04 (detectability via timing) | M | Tenant business reality; DPA covers | Annually |
| T-L-06 (end-user unawareness) | M | Tenant-of-tenant responsibility | Annually |
| T-L-07 (right-to-erasure best-effort) | M | Subject to retention windows; DSR best-effort within Mimir/Postgres retention | Annually |
| T-L-09 (LLM session correlation) | M | LLM-provider trust boundary; pragmatic mitigation via opt-in | Quarterly |

Sign-off:
- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`

## Per-Pack Overlay Sections

### pack-kr (Korea — PIPA + ISMS-P)

- **PIPA Art. 23 (sensitive personal information)**: every Object Type carrying `Tier1Sensitive` properties classified under Art. 23; salt-rotation per T-L-02; mirror of pack-kr `oya-ontology-cedar-fragment-coverage` reviewed annually.
- **PIPA Art. 29 (technical safeguards)**: T-S-01..T-E-05 mitigations map directly to the 12 prescribed safeguards.
- **PIPA Art. 17 + 18 (purpose limitation)**: Function `data_class` propagation ensures use limited to declared purpose.
- **KR 전자문서법 Art. 5**: Ed25519 audit-chain seals satisfy electronic-document integrity.
- **KR-ISMS-P §2.5 + §2.7**: 2-person rule + JIT elevation.

### pack-us-healthcare (HIPAA-scoped)

- **HIPAA §164.312(a)(1) (access control)**: Postgres RLS + Cedar + per-tenant API key satisfy Unique-User Identification + Encryption-and-Decryption.
- **HIPAA §164.312(b) (audit controls)**: audit-chain seal on every PHI-touching Object Type write + Action.
- **HIPAA §164.502 (minimum necessary)**: Function projection tier-filters PHI properties; Cedar permits per role.
- **HIPAA §164.502(a) (TPO)**: Action Type registry tags TPO scope.
- **BAA template**: per-tenant BAA at `microservices/ontology/legal/baa-template.md` (Slice D follow-up).

### pack-eu (GDPR + EDPB + NIS2 + eIDAS)

- **GDPR Art. 25 (privacy by design)**: every mitigation here maps to a Schrems-II-compatible TOM.
- **GDPR Art. 22 (automated decision-making)**: agent-gateway autonomy_tier explicit; tenant DPA carve-out.
- **GDPR Art. 32 (security of processing)**: every T-*-NN mitigation contributes to Art. 32 posture.
- **GDPR Art. 35 (DPIA)**: paired with `dpia.md`.
- **GDPR Art. 44–50 (transfers)**: pack-eu Ontology cluster is EU-resident; cross-region replication forbidden by default.
- **NIS2**: incident reporting timelines.
- **eIDAS 910/2014**: Ed25519 seals are AdES.

### pack-jp (APPI)

- **APPI Art. 17 (purpose of use)**: declared at tenant onboarding.
- **APPI Art. 21 (cross-border transfer)**: pack-jp residency.
- **APPI Art. 27 (sensitive data consent)**: tenant DPA captures.

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/ontology-overlay.md` carry the local PII law's articles + cybersecurity-framework controls. Each maps to T-*-NN via this document's cross-mapping in `microservices/ontology/compliance.md`.

## Compliance Cross-Mapping (Globally Enforced)

| Framework | Coverage | Mapping doc |
|---|---|---|
| SOC 2 Type 2 | CC1.x..CC9.x covered as cited inline | `microservices/ontology/compliance.md` |
| ISO 27001:2022 | A.5.x..A.8.x covered as cited inline | `microservices/ontology/compliance.md` |
| GDPR | Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44 cited inline | `microservices/ontology/dpia.md` + `compliance.md` |

## Re-review Triggers

- Any change to trust-boundary diagram (new actor, removed actor, modified boundary).
- Any Layer-A upgrade (Postgres / Citus / ClickHouse / Cedar / Kafka / Istio) with upstream security notes.
- Any new pack activation (engages distinct legal frameworks).
- Annual scheduled review (Q2 each year).
- Post-incident review (any Sev-1 / Sev-2 in ontology or any µservice it serves).
- Pen-test or audit finding.

## References

- ADR-0006 (Ontology typed-entity layer).
- ADR-0028 (Bominal — audit chain).
- ADR-0055 + ADR-0122 (Ontology naming).
- ADR-0056 (BNF v4.1).
- ADR-0059 (Workflow + Ontology adapter layer).
- ADR-0106 (Bominal — Ontology architecture).
- ADR-0107 (Bominal — agent gateway).
- ADR-0117 (residency).
- ADR-0123 (hyperscaler maturity claim gate).
- ADR-0130 (SLO gate).
- ADR-0131 (per-microservice flat layout).
- ADR-0132 (Bominal — pillars).
- ADR-0140 (Cedar policy enforcement).
- `microservices/ontology/PRD.md`.
- `microservices/ontology/dpia.md`.
- `microservices/ontology/policy/type-isolation.md`.
- `microservices/ontology/policy/data-residency.md`.
- `/specs/products/ontology.json`.
- Microsoft STRIDE; LINDDUN (KU Leuven); OWASP Top 10 (2021); NIST SP 800-154.
- Palantir Foundry Ontology security model — `palantir.com/docs/foundry/ontology/security`.
- AWS Cedar — `cedarpolicy.com`.
- ICO DPIA template — `ico.org.uk`.
- CNIL DPIA methodology — `cnil.fr/en/PIA`.
