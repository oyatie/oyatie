---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: workflow-engine
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-workflow + ops-security
deciders: council-architecture, ops-security, axis-workflow, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + NIST SP 800-154
related_adrs: [ADR-0028, ADR-0035, ADR-0056, ADR-0103, ADR-0105, ADR-0117, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145), ADR-0148]
related_specs: [/specs/microservices/workflow.json, /specs/per-microservice-flat-layout.json]
review_cadence: quarterly + on every engine substrate change OR new event-type registration
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC6.7, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.8.2, A.8.3, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.25, A.8.26, A.8.27, A.8.28"
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35"
suggested_frameworks_by_pack:
  pack-kr: ["KR-ISMS-P §2.1-2.12", "KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2", "KR 전자문서법 Arts. 5/6/7"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308 / §164.310 / §164.312 / §164.314 / §164.316"]
  pack-eu: ["GDPR Arts. 25 + 32 + 35 + 44-50", "eIDAS 910/2014", "NIS2 2022/2555"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24/26-2"]
  pack-sg: ["PDPA 2012 §11-26", "MAS-TRM v2021 §11-12"]
  pack-au: ["Privacy Act 1988 APP 1-13", "APRA-CPS 234 §29-44"]
  pack-in: ["DPDPA 2023 §6-10", "RBI Master Direction on Outsourcing 2023"]
  pack-br: ["LGPD Arts. 6, 7, 11, 14, 18, 33, 46, 48", "BACEN Res. 4.893/2021"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021 Arts. 5/6/9/15"]
  pack-ksa: ["PDPL Royal Decree M/19/2021 Arts. 4-9", "SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Threat Model: workflow-engine µservice

## Purpose

Identify, classify, and mitigate threats to the workflow-engine µservice's confidentiality, integrity, availability, and privacy posture. The engine is the cross-µservice orchestration adapter; a compromise here cascades to every product. This document is the canonical security artifact reviewed by SOC 2 Type 2 examiners, ISO 27001 auditors, and GDPR DPAs at first-tenant onboarding.

## Scope

### In-scope

All components introduced by the workflow-engine PRD + PHASE-01:

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| Postgres + Citus (durable run state, outbox, spec store) | `oya-workflow-engine-spec-store-*` (9 crates) |
| Valkey (ephemeral lease state, subscription registry) | `oya-workflow-engine-execution-engine-*` (12 crates) |
| ClickHouse (run-history analytics replica) | `oya-workflow-engine-state-machine-*` (6 crates) |
| Object storage (large step payloads) | `oya-workflow-engine-event-bus-*` (11 crates) |
|  | `oya-workflow-engine-replay-debugger-backend-*` (11 crates) |
|  | Workflow specs at tenant-uploaded paths |
|  | Per-tenant event-bus topic namespaces |
|  | Audit-chain seals over runs |

### Out-of-scope

- Threats to the underlying Kubernetes cluster, container runtime, or hyperscaler IaaS — owned by `cloud-k8s` threat model.
- Threats to Postgres / Valkey / ClickHouse infrastructure layer — owned by `cloud-iac` µservice threat model; this document inherits.
- Threats to the Studio visual editor — owned by `workflow-studio` threat model.
- Threats to OpenBao secret manager — owned by `cloud-secrets` threat model.
- Threats to the consuming µservices' workflow specs themselves — each owns its own threat model; the engine inherits spec-content threats via signature verification.

## Trust Boundaries

```text
┌─ Internet ─────────────────────────────────────────────────────────────────┐
│   Tenant operators (via Studio)              Customer applications         │
│         │                                          │                       │
│         │ (HTTPS, OIDC, mTLS)                      │ (per-tenant SDK keys) │
│         ▼                                          ▼                       │
│  ┌─ Public ingress (Envoy/Istio) ───────────────────────────────────────┐  │
│  │  - TLS + WAF + DDoS                                                  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────┼──────────────────────────────────────────┘
                                  ▼
┌─ workflow-engine cluster (per-cell, per-pack) ─────────────────────────────┐
│                                                                            │
│  Trust boundary 1: External → engine ingress (REST + gRPC)                 │
│                                                                            │
│  ┌─ execution-engine-rest ────┐  ┌─ event-bus-rest ────┐                   │
│  │ OIDC tenant-scoped         │  │ pub/sub + replay    │                   │
│  └────────────────────────────┘  └─────────────────────┘                   │
│                                                                            │
│  Trust boundary 2: Per-tenant Citus partition (tenant_id partition key)    │
│                                                                            │
│  ┌─ Postgres + Citus (run state + spec store + outbox) ────────────┐       │
│  │  - tenant_id-partitioned distributed tables                     │       │
│  │  - Row-level security (RLS) on top of Citus                     │       │
│  │  - Per-tenant connection pool                                   │       │
│  └─────────────────────────────────────────────────────────────────┘       │
│  ┌─ Valkey (ephemeral) ─┐ ┌─ ClickHouse (analytics) ─┐                      │
│  │ tenant-prefixed key │ │ tenant_id partition key  │                      │
│  └─────────────────────┘ └──────────────────────────┘                      │
│                                                                            │
│  Trust boundary 3: Engine worker → step body execution (sandbox)           │
│                                                                            │
│  ┌─ execution-engine-worker (one worker owns one run via Valkey lease) ┐    │
│  │  - Step body executed in Wasmtime sandbox (plugin substrate ADR-0037) │ │
│  │  - No host filesystem / network unless explicitly granted by spec  │   │
│  │  - Memory + CPU bounded per execution                              │   │
│  │  - Deterministic-step contract enforced at compile time           │    │
│  └────────────────────────────────────────────────────────────────────┘    │
│                                                                            │
│  Trust boundary 4: event-bus subscriber → published event                  │
│                                                                            │
│  ┌─ event-bus subscription enforcement ───────────────────────────────┐    │
│  │  - tenant_id binding from SDK auth ≠ requested filter tenant_id    │    │
│  │  - subscription state in Valkey with tenant prefix                  │    │
│  │  - cross-tenant subscribe attempts audit-emitted + denied          │    │
│  └────────────────────────────────────────────────────────────────────┘    │
│                                                                            │
│  Trust boundary 5: Audit chain emission                                    │
│                                                                            │
│  ┌─ audit-chain-emitter (in-process; signs every state transition) ─┐      │
│  │  - Ed25519 signing key from OpenBao (rotated 90d)                │      │
│  │  - Merkle-chain over per-tenant per-run event sequence          │      │
│  └──────────────────────────────────────────────────────────────────┘      │
└────────────────────────────────────────────────────────────────────────────┘
```

Five trust boundaries:
1. **External → engine ingress** (TLS, WAF, DDoS, OIDC).
2. **Per-tenant Citus partition + RLS** (the load-bearing isolation boundary).
3. **Engine worker → step body** (Wasmtime sandbox; determinism contract).
4. **Event-bus subscriber → published event** (tenant-binding subscription enforcement).
5. **Audit-chain emission** (Ed25519 signing; non-repudiation).

## Assets & Data Classification

Per Bominal ADR-0028 (audit-chain + data-class taxonomy) and `oya-check-data-class` LEAN lane.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| Workflow specs (compiled IR) | `INTERNAL_ONLY` | Low-Medium | append-only git history + Postgres versions; 24mo of versions retained | Postgres + repo |
| Run state (current + checkpoints) | `BEHAVIORAL_TENANT_PRODUCT` | High | 90d hot + 24mo cold (ClickHouse + Postgres) | Postgres (authoritative); ClickHouse (replica) |
| Step payloads (input + output) | `BEHAVIORAL_TENANT_PRODUCT` + transient `PII_IDENTIFYING` (user-id fields) + occasionally `PHI` (pack-us-healthcare clinical workflows) | High | 14d hot + per-pack retention overlay | Postgres + object storage for large payloads |
| Event log (typed events) | `BEHAVIORAL_TENANT_PRODUCT` + sometimes `PII_QUASI_IDENTIFIER` | High | 90d hot + 24mo cold for replay | Postgres outbox + ClickHouse replica |
| Workflow-event subscriptions (registry) | `INTERNAL_ONLY` | Low | live in Valkey; reconstructable from Postgres on cold-start | Valkey + Postgres |
| Spec signing keys (Ed25519) | `SECRET` | Critical | OpenBao 90d rotation | OpenBao |
| Audit-chain Ed25519 signing keys | `SECRET` | Critical | OpenBao 90d rotation; HSM-backed where available | OpenBao |
| Per-tenant SDK API keys | `SECRET` | Critical | OpenBao 30d rotation | OpenBao |
| Postgres connection credentials (per-cell) | `SECRET` | Critical | OpenBao 30d rotation | OpenBao |
| Audit-chain seals (per run) | `AUDIT` | High | append-only; immutable | audit-chain µservice |
| Ephemeral step lease leases (Valkey) | `BEHAVIORAL_TENANT_PRODUCT` | Medium | TTL ≤ 5min per lease | Valkey |
| Hashed tenant ID (used in topic namespace + RLS) | `SENSITIVE_PIPA_ART23` (potential re-identification with auxiliary data) | High | salted; rotation 12mo | OpenBao tenant-resolver |

## Actors

| Actor | Trust level | Authentication | Capability |
|---|---|---|---|
| External tenant operator (human, via Studio) | Untrusted external | OIDC + MFA | Submit specs (PR-authored); start/pause/resume/cancel own runs; view debugger |
| Customer application (machine, embeds SDK) | Untrusted external | Per-tenant SDK API key | Publish + subscribe to own tenant's events; start runs |
| Workload µservice (in same trust domain) | Semi-trusted internal | mTLS + SPIFFE identity | Publish typed events on event-bus; subscribe |
| Studio editor (sibling µservice; in same cluster) | Semi-trusted internal | mTLS + SPIFFE identity | Submit specs; query run state; invoke replay debugger |
| Engine worker (in-process) | Trusted internal | OpenBao-issued ServiceAccount token + SPIFFE | Read/write run state; emit audit-chain seals; dispatch steps |
| Outbox relay worker | Trusted internal | SPIFFE | Read outbox + publish to event-bus subscribers |
| Replay-debugger-backend worker | Trusted internal | SPIFFE | Read event log; replay against ClickHouse |
| Reviewer agent (oya-pr-review lane) | Trusted internal | OIDC-bound CI identity | Read specs at PR-review time; refuse spec changes that violate gate |
| Council operators (human) | Trusted internal | OIDC + MFA + JIT via OpenBao | Admin operations on spec store; emergency override on runs (2-person rule + audit) |
| External auditor | Read-only external, time-boxed | OIDC + MFA + JIT short-lived token | Read audit-chain export; cannot pivot to tenant run state |
| Attacker — opportunistic | Untrusted | none | Scans + low-skill exploitation; always present |
| Attacker — targeted | Untrusted | none | Sophisticated; supply-chain awareness |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure spec or event registration (mitigated by PR review + LEAN gates) |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case threat actor; mitigated by least-privilege + audit-chain + 2-person rule |

## STRIDE Threat Catalog

### Spoofing (S)

**T-S-01 — Tenant-A publishes event claiming to be from Tenant-B**
- Asset: event-bus tenant boundary
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - SDK API key is per-tenant; key carries bound `tenant_id` claim signed by OpenBao.
  - Engine event-bus REST enforces the inbound key's bound tenant claim; refuses any `tenant_id` override in the event payload.
  - Server-side stamping: engine overwrites the event's `tenant_id` field with the authenticated tenant; client cannot spoof.
  - Mismatch attempts return 401 + audit-emit `engine_tenant_spoofing_attempt`.
- Owner: axis-workflow + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2, CC6.6; ISO 27001 A.5.15, A.5.17, A.8.2, A.8.3; GDPR Art. 32(1)(a)(b); KR PIPA Art. 29

**T-S-02 — Forged spec signature: tenant-A submits spec signed with claimed-but-revoked key**
- Asset: workflow spec signature
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Signing keys are OpenBao-managed with revocation list; spec verification consults the live revocation list on every read.
  - Spec-store rejects any spec whose signing key is revoked OR not in the tenant's allowed signer set.
  - Revocation propagation latency monitored as an SLI (`oya_workflow_engine_revocation_propagation_lag`).
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC7.1; ISO 27001 A.5.17, A.8.7; GDPR Art. 32(1)(b)

**T-S-03 — Attacker impersonates engine worker via SA token leak**
- Asset: engine-worker SPIFFE identity
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - SA token bound to pod identity; cannot be used outside cluster.
  - Token rotation 24h.
  - Postgres + Valkey + ClickHouse all validate the SPIFFE identity matches expected engine-worker SA.
  - Network policy: only engine pods may reach Postgres write endpoints.
- Owner: ops-security + axis-workflow
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.5.17, A.8.5, A.8.7

**T-S-04 — Studio impersonates engine internal REST for privileged operations**
- Asset: spec-store + execution-engine REST
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - mTLS between Studio and engine; SPIFFE identity bound.
  - Cedar policy denies privileged operations from non-engine-internal identities.
  - 2-person rule enforced on operator-override paths via JIT elevation.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.3

**T-S-05 — Attacker spoofs an audit-chain emission to forge run history**
- Asset: audit-chain seal sequence
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Audit-chain emission only from engine-internal in-process emitter; OpenBao-signed key never exits the process.
  - Merkle chain detects out-of-order or duplicate seals at verification time.
  - Per-run seal sequence is monotonic; gaps trigger anomaly alert.
- Owner: audit-chain + axis-workflow
- Residual: L
- Frameworks: SOC 2 CC4.1, CC6.6, CC8.1; ISO 27001 A.5.28, A.8.15; GDPR Art. 5(2)

### Tampering (T)

**T-T-01 — Workflow spec tampering at submit time**
- Asset: spec-store entries
- Likelihood: M / Impact: H (false spec → bad run sequence → tenant impact) / Risk: **H**
- Mitigations:
  - All spec submissions are signature-verified; tampered specs refused at read time.
  - LEAN check `oya-governance-workflow-spec-signature-verification` validates the signature path is exercised on every spec read in unit tests.
  - PR review required for production-tier spec promotion (`release/workflow-engine/production` ref).
- Owner: axis-workflow
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.31, A.5.32, A.8.32, A.8.33; GDPR Art. 32(1)(b)

**T-T-02 — Run state corruption via concurrent write (single-writer bypass)**
- Asset: WorkflowRun Postgres row
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Single-writer invariant enforced via Valkey lease per (tenant, run_id); only the lease-holder may write.
  - Postgres optimistic concurrency check (`version` column); lease + version both required for write.
  - Lease TTL ≤ 5min; expired leases trigger lease-extension or transparent failover.
- Owner: axis-workflow
- Residual: L
- Frameworks: SOC 2 CC7.1, CC8.1; ISO 27001 A.8.32

**T-T-03 — Event log tampering (outbox row mutation)**
- Asset: outbox event log
- Likelihood: L / Impact: H (replay would produce wrong step sequence) / Risk: **M**
- Mitigations:
  - Outbox rows are append-only; INSERT-only; UPDATE/DELETE refused by Postgres trigger.
  - Each outbox row carries Ed25519 signature over event payload + tenant_id + run_id + sequence_num.
  - Replay verifies signatures; signature mismatch quarantines the event + audit-emit.
- Owner: axis-workflow
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.1, CC8.1; ISO 27001 A.5.28, A.8.32

**T-T-04 — State-machine transition rule tampering (in-database invariant lookup)**
- Asset: TransitionRule rows
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Transition rules are derived from the signed spec; they are not separately writable at runtime.
  - State-machine refuses transitions whose rule version-SHA doesn't match the run's pinned spec version.
- Owner: axis-workflow
- Residual: L
- Frameworks: SOC 2 CC7.1, CC8.1; ISO 27001 A.8.32

**T-T-05 — Replay-attack on durable-execution: replay old event to re-trigger side effect**
- Asset: Event-bus subscription delivery
- Likelihood: M / Impact: H (could double-execute a payment, message, etc.) / Risk: **H**
- Mitigations:
  - Events carry idempotency keys; the engine refuses to dispatch a step body twice for the same (run_id, step_index, idempotency_key).
  - Subscribers MUST be idempotent — surfaced as a contract at `docs/standards/workflow-event-consumer.md`.
  - Deterministic-replay invariant: replays produce the same sequence, but side-effecting steps (HTTP, DB write) are gated through a side-effect ledger that ignores replay-flagged executions.
  - Replay-flagged events are explicitly marked `replayed=true` on the bus; subscribers can refuse them.
- Owner: axis-workflow + each subscriber
- Residual: L (idempotency discipline floor; mitigated by contract + tests)
- Frameworks: SOC 2 CC7.2; ISO 27001 A.8.20, A.8.21; GDPR Art. 32(1)(b)

**T-T-06 — Spec store version downgrade attack (force older version with known bug)**
- Asset: spec-store version selection
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Runs pin to a specific `workflow_version_sha` at run-start time; downgrade impossible mid-run.
  - Promotion to `release/workflow-engine/production` is forward-only; rolling-back a spec version requires explicit operator action + audit.
- Owner: axis-workflow
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.32

### Repudiation (R)

**T-R-01 — Tenant operator denies authorship of a destructive run cancellation**
- Asset: WorkflowCancelled event
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Every operator-initiated transition (pause/resume/cancel/signal) requires OIDC-bound identity + recorded in audit-chain with actor identity.
  - 2-person rule for destructive operations (cancel running production-tier workflow); both signatures recorded.
- Owner: ops-security + axis-workflow
- Residual: L
- Frameworks: SOC 2 CC4.1, CC8.1; ISO 27001 A.5.27, A.5.28, A.8.15

**T-R-02 — Spec author denies authorship of a manifest change**
- Asset: WorkflowSpec submission
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - Signed commits to spec PRs required per branch-protection.
  - Spec carries signer identity in its signature record.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.8.34; GDPR Art. 5(2)

**T-R-03 — Engine emits state transition without traceable trigger**
- Asset: Audit-chain run history
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Every state transition carries a `cause` field (event-ID + correlation-ID); audit-chain seal includes the cause.
  - Engine refuses to emit transitions without a cause field.
- Owner: axis-workflow
- Residual: L
- Frameworks: SOC 2 CC7.4, CC8.1; ISO 27001 A.5.26, A.5.27, A.8.15

### Information Disclosure (I)

**T-I-01 — Cross-tenant run state leak via Citus partition bypass**
- Asset: WorkflowRun Postgres rows
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Citus partition + Row-Level Security (RLS) BOTH enforce tenant isolation; defense-in-depth.
  - Per-tenant Postgres connection pool — connection's session variable carries tenant_id; RLS predicate reads it.
  - LEAN check `oya-governance-citus-rls-enforced` validates schema + policies on every PR.
  - Per-tenant query audit via Postgres extension `pgaudit`.
  - Penetration test against tenant boundary annually.
- Owner: ops-security + axis-workflow
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.5.18, A.8.2, A.8.3, A.8.12; GDPR Art. 5(1)(f), Art. 25, Art. 32

**T-I-02 — PII leakage through step payload logging**
- Asset: Step payload fields (input/output)
- Likelihood: H (engineers log step input/output for debug) / Impact: H / Risk: **H**
- Mitigations:
  - Step payloads classified with `data_class` annotation in the spec; engine refuses to log `PII`-class fields.
  - OTel-emission redactor strips PII patterns at emission time.
  - Sampling: step-payload logs are sampled at 0.1% in production; redactor still applies to sampled logs.
  - Quarterly synthetic-PII drill validates redactor effectiveness.
- Owner: axis-workflow + each workload µservice owner
- Residual: M (engineering discipline floor)
- Frameworks: SOC 2 CC6.7; ISO 27001 A.8.11, A.8.12, A.8.32; GDPR Art. 5(1)(c), Art. 25, Art. 32

**T-I-03 — Event bus subscription leak: tenant-A subscribes and receives tenant-B events**
- Asset: event-bus subscription delivery
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Subscription bound to authenticated tenant at create-time; server-side filter enforces tenant_id match on every delivery.
  - Cross-tenant subscribe attempts denied + audit-emitted.
  - Bus delivery layer carries the subscription's tenant_id; mismatch on delivery refused.
  - Threat hunt: weekly `oya_workflow_engine_cross_tenant_subscribe_attempt:rate` SLO (target = 0).
- Owner: axis-workflow + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.2, A.8.3; GDPR Art. 25, Art. 32

**T-I-04 — Replay-debugger leaks payload across tenant boundaries**
- Asset: replay-debugger-backend read-side
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - replay-debugger-backend REST scoped per-tenant; OIDC + Cedar policy enforces.
  - ClickHouse query frontend enforces tenant_id partition predicate.
- Owner: axis-workflow + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.3; GDPR Art. 32

**T-I-05 — Secret leakage in step payload context (engineer accidentally puts password in step input)**
- Asset: Step payload
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Step inputs classified at spec authoring time; `SECRET`-class fields stored as OpenBao SecretReference, not raw.
  - Engine refuses runs whose spec carries `SECRET`-class fields as plaintext.
  - Secret-scanner CI lane scans every event payload for known secret patterns at emission time.
- Owner: ops-security + cloud-secrets
- Residual: M (human-error baseline)
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.17, A.8.7, A.8.12; GDPR Art. 32

**T-I-06 — Aggregated run-history analytics leak per-tenant identity via ClickHouse**
- Asset: replay-debugger-backend analytics
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Analytics queries require OIDC + scope `replay:read:<tenant>`.
  - Cross-tenant aggregations (if produced for marketing / benchmarks) use differential-privacy with ε ≤ 1.
  - Per-tenant tag stripped before aggregation.
- Owner: axis-workflow
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.5; GDPR Art. 25

### Denial of Service (D)

**T-D-01 — Per-tenant run flood overwhelms shared engine workers**
- Asset: engine-worker capacity
- Likelihood: H / Impact: H / Risk: **H**
- Mitigations:
  - Per-tenant run-start rate limit + per-tenant active-run cap; refuse `429 Too Many Requests` above cap.
  - Fair-share scheduling: one tenant cannot starve another via long-running workflows.
  - HPA on engine workers; min 3 replicas, max 200.
  - Pre-warmed pool of 10 standby pods; cold-start ≤ 500ms.
- Owner: ops-sre-reliability + axis-workflow
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6, A.8.14; GDPR Art. 32(1)(c)

**T-D-02 — State-machine deadlock: two runs each wait on the other**
- Asset: cross-run dependencies
- Likelihood: M / Impact: H (runs stuck indefinitely) / Risk: **H**
- Mitigations:
  - Deadlock detection in execution-engine: cycle detection on cross-run wait-graph; tear down cycle by failing the youngest run.
  - SLA timer fires on stuck runs; auto-cancellation with audit-chain emission.
  - Per-tenant deadlock-rate SLI; threshold triggers OnCall page.
- Owner: axis-workflow
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.14, A.8.20

**T-D-03 — Event-bus backpressure: subscriber slow → queue grows → engine OOM**
- Asset: outbox + subscription delivery
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Per-subscription consumer-lag SLI; lag > 60s pages OnCall.
  - Backpressure protocol: subscriber receives flow-control signal; engine drops or buffers based on subscription's declared policy.
  - Disk-backed buffer for outbox (Postgres); engine doesn't OOM on slow subscribers.
  - Slow-subscriber quarantine: subscriptions with chronic lag > 10× threshold are dropped after 1h + tenant notified.
- Owner: ops-sre-reliability + axis-workflow
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.6, A.8.14

**T-D-04 — Durable-execution replay storm after engine cold-start**
- Asset: engine worker re-warm path
- Likelihood: M / Impact: H (cold-start could attempt to resume all 10k+ in-flight runs simultaneously) / Risk: **H**
- Mitigations:
  - Resume cadence rate-limited at cold-start: replays at 100 runs/s/worker, gradually ramping.
  - In-flight run resumption distributed across workers via consistent-hash on run_id; no thundering herd.
  - Pre-warmed pool absorbs early load; HPA ramps to handle steady-state.
- Owner: axis-workflow + ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.6, A.8.14

**T-D-05 — Postgres lock contention: hot tenant's run state row contention starves others**
- Asset: WorkflowRun Postgres rows
- Likelihood: M / Impact: M-H / Risk: **M-H**
- Mitigations:
  - Per-tenant Citus partition isolates lock domains.
  - Optimistic concurrency control (no long-held locks).
  - Step state updates use row-level locking with timeout; timeout triggers retry with backoff.
  - Hot-row detection SLI: rows with > 10 contended waits/sec trigger investigation.
- Owner: axis-workflow + cloud-iac
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.6, A.8.14

**T-D-06 — Valkey lease coordinator outage halts step dispatch**
- Asset: Valkey (lease coordinator)
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Valkey HA via Sentinel; min 3 replicas; automatic failover.
  - Fallback to Postgres advisory locks when Valkey is unhealthy (latency degraded but availability preserved).
  - Per-cell Valkey cluster; cell-level isolation.
- Owner: ops-sre-reliability + axis-workflow
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6, A.8.14

### Elevation of Privilege (E)

**T-E-01 — Wasm step body escapes sandbox to host filesystem / network**
- Asset: Wasmtime sandbox boundary
- Likelihood: L (Wasmtime is mature) / Impact: H / Risk: **M**
- Mitigations:
  - Wasmtime configured with no filesystem capabilities; no network host functions; no clock except deterministic-replay-safe clock.
  - WASI capabilities explicit per-step; spec authoring requires explicit capability declarations; engine refuses specs requesting capabilities outside the tenant's allowed set.
  - Memory + CPU bounded per execution (gas limits).
  - Fuzz-test the sandbox boundary quarterly.
- Owner: axis-workflow + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.3, A.8.4, A.8.28

**T-E-02 — Operator JIT elevation abused to mass-cancel tenant production runs**
- Asset: operator-override path
- Likelihood: L (insider-malicious) / Impact: H / Risk: **M**
- Mitigations:
  - 2-person rule required for cancel of production-tier runs; audit-chain emission.
  - Mass-cancel pattern detection: > 10 cancels/min triggers anomaly alert.
  - Soft-cancel: cancellation marks the run for terminal state; recovery window 30min during which an op can rollback the cancel.
- Owner: ops-security + axis-workflow
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.15, A.5.27, A.8.4, A.8.16

**T-E-03 — Spec compiler privilege escalation via crafted spec field**
- Asset: SpecCompiler logic
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Spec parser is fuzzed at CI time (`oya-governance-workflow-spec-fuzz`).
  - Spec parser bounded input lengths; rejects oversized fields before compilation.
  - Compiler runs in process isolation from the durable store.
- Owner: axis-workflow + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC8.1; ISO 27001 A.5.15, A.8.28

**T-E-04 — Cedar policy bypass via subscription-create with synthetic tenant_id**
- Asset: event-bus subscription registry
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cedar v3+ used; field length bounded; fuzzing at CI.
  - Server-side stamping of tenant_id from auth token (client cannot supply); defence-in-depth over Cedar.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.28

## LINDDUN Privacy-Threat Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | Step payloads + event log | Multiple data points across runs can be linked to a single end-user even when no direct identifier present. | Payload classification + redactor; correlation IDs are tenant-scoped, not user-scoped. | M |
| T-L-02 | Identifiability | Hashed tenant ID in topic namespace | sha256(tenant_id)[..16] may be re-identifiable via auxiliary data. | Salted hash; salt rotated 12mo; audit-chain notes rotation. | L |
| T-L-03 | Non-repudiation | Tenant operator authorship of spec changes | Tenant may deny authorship. | Signed commits; PR audit log. | L |
| T-L-04 | Detectability | Run timing patterns | Tenant business-event timing correlates with workflow run starts. | Expected: this is BEHAVIORAL_TENANT_PRODUCT. Consent at onboarding. | M |
| T-L-05 | Disclosure | Replay-debugger access by auditors | Auditor scoped to one tenant could pivot via shared debugger UI. | Auditor tokens tenant-scoped at debugger folder level. | L |
| T-L-06 | Unawareness | End-user (tenant's user) unaware of workflow processing | Tenant's end-user may not know operational data is captured. | Tenant DPA includes upstream-disclosure clause; joint controllership per Art. 26. | M |
| T-L-07 | Non-compliance | GDPR Art. 17 right-to-erasure cascade | End-user erasure across runs / payloads / event log / ClickHouse replica. | DSR cascade per `oya-dsr-cascade-runner` skill; 30-day SLA from request. | M (best-effort within retention) |

## Mitigations Catalog

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Per-tenant Citus partition + RLS | Preventive | axis-workflow | `oya-governance-citus-rls-enforced` lane |
| Per-tenant SDK API key with bound claim | Preventive | cloud-secrets | OpenBao audit + engine REST logs |
| Server-side tenant_id stamping on inbound events | Preventive | axis-workflow | LEAN check on event-bus rest |
| Ed25519 spec signature verification | Preventive | axis-workflow | `oya-governance-workflow-spec-signature-verification` lane |
| Ed25519 audit-chain seals | Detective + Non-repudiation | audit-chain | Audit-chain regression tests |
| Outbox idempotency keys | Preventive (replay-attack) | axis-workflow | Replay determinism test suite |
| Single-writer Valkey lease per run | Preventive (race conditions) | axis-workflow | Concurrent-writer integration test |
| Wasmtime sandbox per step | Preventive | axis-workflow | Sandbox fuzz tests |
| Step-payload `data_class` annotations | Preventive | each spec author | `oya-check-data-class` lane |
| Per-tenant rate limits + active-run caps | Preventive (DoS) | axis-workflow | Engine REST metrics |
| 2-person rule for operator overrides | Preventive (insider) | ops-security | OpenBao JIT elevation logs |
| DSR cascade runner | Preventive (compliance) | council-privacy | DSR queue dashboard SLO |
| Cross-tenant subscribe attempt SLI | Detective | axis-workflow | Mimir alert |
| Mass-cancel anomaly alert | Detective | ops-security | Mimir alert |
| Network policy: engine → Postgres / Valkey / ClickHouse only | Preventive | ops-sre-reliability | Kubernetes NetworkPolicy review |

## Residual Risk Acceptance

| Risk ID | Residual | Why accepted | Re-review date |
|---|---|---|---|
| T-I-02 (PII in step payloads) | M | Cannot fully eliminate without prohibiting payload logging. Engineering discipline floor. | Quarterly |
| T-I-05 (secret in payload) | M | Human-error baseline; mitigated via detection + rotation. | Quarterly |
| T-T-05 (replay-attack on subscribers) | M-L | Idempotency discipline is the load-bearing control. | Quarterly |
| T-L-01 (linkability) | M | Inherent to step-payload tracing. | Annually |
| T-L-04 (detectability) | M | Tenant business reality; consent at onboarding. | Annually |
| T-L-06 (end-user unawareness) | M | Tenant-of-tenant joint-controllership responsibility. | Annually |
| T-L-07 (right-to-erasure best-effort) | M | Bounded by retention windows. | Annually |

Sign-off (this document is RW until council sign-off captured):

- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`

## Per-Pack Overlay Sections

### pack-kr (Korea)

- KR PIPA Art. 23 (sensitive PII): hashed tenant IDs are sensitive when paired with auxiliary; salt-rotation in T-L-02 satisfies Art. 23.
- KR PIPA Art. 29 (technical safeguards): every T-*-NN mitigation maps to one of the 12 prescribed safeguards. Cross-mapped in `compliance.md`.
- KR PIPA Art. 23-2 (cross-border transfer): KR tenant data stays in pack-kr cluster.
- KR-ISMS-P §2.7 (접근통제) + §2.5 (인적보안): 2-person rule + JIT elevation map directly.

### pack-us-healthcare (HIPAA)

- HIPAA §164.312(a)(1) (access control): per-tenant Citus + RLS + Ed25519 audit-chain.
- HIPAA §164.312(b) (audit controls): audit-chain emission on every state transition; retention ≥ 6y for pack-us-healthcare runs.
- HIPAA §164.502 (minimum-necessary): step payloads redact PHI at emission.
- HIPAA §164.504(e) (Business Associate Agreement): oyatie operates as BA for HIPAA-scope tenants; BAA at `microservices/workflow-engine/legal/baa-template.md`.

### pack-eu (GDPR + EDPB + NIS2)

- GDPR Art. 25: every mitigation mapped to Schrems-II-compatible technical-organizational measure.
- GDPR Art. 35 DPIA: this threat model + the DPIA at `dpia.md` satisfy DPIA for high-risk processing.
- GDPR Art. 32: every T-*-NN mitigation contributes to Art. 32 security posture.
- GDPR Arts. 44-50: pack-eu cluster EU-resident; cross-region replication forbidden by default.
- NIS2 2022/2555: when oyatie crosses Annex I/II thresholds, 24h + 72h + 1mo reporting timelines apply; `incident-response.md` reflects.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Pack-overlay sections at `regional-packs/<pack>/workflow-engine-overlay.md`; each follows the same structure with local PII law's articles + local cybersecurity-framework controls; maps to this document's threat IDs via cross-mapping in `compliance.md`.

## Compliance Cross-Mapping

| Framework | Coverage | Mapping doc |
|---|---|---|
| SOC 2 Type 2 (2017 TSC + 2022 PoF) | CC1.x through CC9.x covered as cited inline | `microservices/workflow-engine/compliance.md` |
| ISO 27001:2022 | Annex A.5-A.8 controls cited inline | `microservices/workflow-engine/compliance.md` |
| GDPR | Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44 cited inline | `microservices/workflow-engine/dpia.md` + `compliance.md` |

## Re-review Triggers

- Any change to the trust boundary diagram (new boundary, removed boundary, modified actor).
- Any Layer-A version upgrade (Postgres / Citus / Valkey / ClickHouse) where upstream release notes mention security fixes.
- New event type registered to the event-bus (each new event is a new spec-content surface).
- New pack activation.
- Annual scheduled review (Q2 each year).
- Post-incident review (any Sev-1 or Sev-2 incident in workflow-engine or in a µservice it routes events for).
- Pen-test or audit finding.

## References

- ADR-0028 (Bominal): Audit chain; inherited.
- ADR-0035 (Bominal): Workflow engine (hybrid SM + DAG); inherited.
- ADR-0103 (Bominal): Workflow hexagonal migration; inherited.
- ADR-0056: BNF v4.1.
- ADR-0105: 13-layer enum.
- ADR-0117: Cloud-native infrastructure (data residency).
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout + workflow unbundle.
- ADR-0140: Cedar policy enforcement.
- `microservices/workflow-engine/PRD.md`.
- `microservices/workflow-engine/dpia.md`.
- `microservices/workflow-engine/compliance.md`.
- `/specs/microservices/workflow.json`.
- Microsoft Threat Modeling methodology (STRIDE).
- LINDDUN privacy-threat methodology — Wuyts et al., KU Leuven.
- OWASP Top 10 (2021) + OWASP API Top 10 (2023).
- NIST SP 800-154.
- Temporal security model — `docs.temporal.io/security`.
