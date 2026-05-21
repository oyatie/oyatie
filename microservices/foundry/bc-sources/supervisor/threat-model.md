---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: foundry-supervisor
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry-control-plane + ops-security
deciders: council-architecture, ops-security, axis-foundry-control-plane, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + NIST SP 800-154 + EU AI Act Annex IV (high-risk-system risk-management)
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0139, ADR-0131, ADR-0133, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/foundry-supervisor-control-plane.json, /specs/per-microservice-flat-layout.json]
review_cadence: quarterly + on every Operator / Postgres / Valkey / Cedar version upgrade + on every new regulatory pack
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC7.1, CC7.2, CC7.4, CC8.1, CC9.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.18, A.5.23, A.5.25, A.5.26, A.5.27, A.5.30, A.5.31, A.5.33, A.8.2, A.8.3, A.8.4, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.24, A.8.25, A.8.26, A.8.27, A.8.28, A.8.32, A.8.34"
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35"
  - "EU AI Act 2024/1689 — Annex III §1-8 (high-risk systems), Art. 9 (risk management), Art. 12 (record-keeping), Art. 14 (human oversight), Art. 15 (accuracy, robustness, cybersecurity)"
suggested_frameworks_by_pack:
  pack-kr: ["KR-ISMS-P §2.1-2.12", "KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308, §164.310, §164.312, §164.314, §164.316"]
  pack-eu: ["EU AI Act 2024/1689 (Annex III + Title III)", "GDPR Arts. 25 + 32 + 35", "NIS2 2022/2555"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24/26-2"]
  pack-sg: ["PDPA 2012 §11-26", "MAS-TRM v2021 §11-12"]
  pack-au: ["Privacy Act 1988 APP 1-13", "APRA-CPS 234"]
  pack-in: ["DPDPA 2023 §6-10", "RBI Master Direction on IT Outsourcing 2023"]
  pack-br: ["LGPD Arts. 6, 7, 11, 14, 18, 33, 46, 48"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021 Arts. 5/6/9/15"]
  pack-ksa: ["PDPL Royal Decree M/19/2021 Arts. 4-9", "SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Threat Model: foundry-supervisor µservice

## Purpose

Identify, classify, and mitigate threats to the foundry-supervisor's confidentiality, integrity, availability, and EU AI Act high-risk-system safety posture. The supervisor is the control-plane authority that deploys and stops every agent capability in oyatie; a compromise here cascades to every tenant's agentic surface. This document is the canonical security artifact reviewed by SOC 2 Type 2 examiners, ISO 27001 auditors, GDPR DPAs, and EU AI Act notified-bodies at first-tenant onboarding.

## Scope

### In-scope

| Layer-A (substrate) | Layer-B (oyatie-owned) |
|---|---|
| PostgreSQL HA cluster (fleet-state, deployment history, entitlement store) | `oya-foundry-supervisor-agent-fleet-lifecycle-*` (11 crates) |
| Valkey Cluster (kill-switch state, supervision-event-bus stream) | `oya-foundry-supervisor-capability-deployment-*` (10 crates) |
| Kubernetes Operator (controller-runtime; kube-rs) + CRDs (`Agent`, `AgentDeployment`, `AutonomyPolicy`, `KillSwitch`) | `oya-foundry-supervisor-autonomy-policy-enforcement-*` (9 crates) |
| Cedar v4 evaluator runtime | `oya-foundry-supervisor-supervision-event-bus-*` (7 crates) |
| OpenBao SecretReference (autonomy-entitlement materialisation) | `oya-foundry-supervisor-kill-switch-circuit-breaker-*` (9 crates) |
| Istio mesh + SPIFFE identities | Capability-definition manifests in tenant-owned git repos |
| Grafana Mimir self-SLO ingestion (per ADR-0139) | Audit-chain seal records |

### Out-of-scope

- Threats to the underlying Kubernetes cluster / IaaS — owned by `cloud-k8s` µservice's threat model.
- Threats to `foundry-runtime` execution-plane workers — owned by that µservice's threat model; this µservice inherits "runtime cannot self-promote" as upstream invariant.
- Threats to OpenBao itself — owned by `cloud-secrets` µservice.
- Threats to `observability` Mimir cluster — owned by that µservice; this document references but does not redefine.

## Trust Boundaries

```text
┌─ Internet ─────────────────────────────────────────────────────────────────┐
│                                                                            │
│  Tenant operators                                                          │
│         │ (HTTPS + OIDC + MFA + Cedar tenant-scope.cedar)                  │
│         ▼                                                                  │
│  ┌─ Public ingress (Envoy/Istio gateway; WAF + DDoS) ────────────────────┐ │
│  └──────────────────────────────────────────────────────────────────────┘ │
│                       │                                                    │
└──────────────────────│────────────────────────────────────────────────────┘
                       ▼
┌─ Foundry-supervisor namespace (dedicated K8s ns per pack region) ──────────┐
│                                                                            │
│  Trust boundary 1: External → Cluster ingress (TLS, WAF, OIDC)             │
│  ┌─ supervisor-rest ──────────────┐  ┌─ admin ops-portal ─────┐            │
│  │ Cedar tenant-scope.cedar       │  │ Cedar auditor-scope    │            │
│  │ (default-deny)                 │  │  (JIT)                 │            │
│  └────────────────────────────────┘  └────────────────────────┘            │
│             │                                                              │
│  Trust boundary 2: REST → Postgres (per-tenant shard; mTLS; SPIFFE)        │
│             │                                                              │
│  ┌─ Postgres HA (master + replica) ─────────────────────────────────┐      │
│  │  - per-tenant row-level-security on every table                 │      │
│  │  - OpenBao-issued per-pod credentials (rotated 30d)             │      │
│  │  - WAL archival to encrypted S3 (data-residency-pinned)         │      │
│  └─────────────────────────────────────────────────────────────────┘      │
│                                                                            │
│  Trust boundary 3: REST + Worker → Valkey Cluster (mTLS; ACL tokens)        │
│  ┌─ Valkey Cluster (3 shards × 2 replicas) ───────────────────────────┐    │
│  │  - Kill-switch state (engaged/disengaged per scope)              │    │
│  │  - Supervision-event-bus stream                                  │    │
│  │  - Per-pod ACL token (rotated 30d)                               │    │
│  └─────────────────────────────────────────────────────────────────┘      │
│                                                                            │
│  Trust boundary 4: Operator → Kubernetes API (per-tenant ns RBAC)          │
│  ┌─ Foundry Operator (controller-runtime; kube-rs) ────────────────┐      │
│  │  - Watches CRDs (Agent, AgentDeployment, KillSwitch,             │      │
│  │    AutonomyPolicy) ONLY in foundry-tenant-* namespaces           │      │
│  │  - SPIFFE: spiffe://oyatie/foundry-supervisor/operator           │      │
│  │  - 2-person rule for any controller pod restart in prod          │      │
│  └─────────────────────────────────────────────────────────────────┘      │
│                                                                            │
│  Trust boundary 5: Supervisor → foundry-runtime (mTLS + SPIFFE)            │
│  - Per-tenant runtime invocation through Cedar autonomy precondition       │
│  - Kill-switch propagation via CRD watch + Valkey pub-sub redundancy        │
│                                                                            │
│  Trust boundary 6: Supervision-event-bus → foundry-evidence (audit-chain)  │
│  - Each event Ed25519-signed at supervisor; audit-chain verifies + seals   │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

Six trust boundaries:
1. External → Cluster ingress (TLS, WAF, OIDC + Cedar).
2. REST → Postgres (per-tenant RLS + mTLS).
3. REST/Worker → Valkey (ACL + mTLS).
4. Operator → K8s API (per-tenant-ns RBAC).
5. Supervisor → foundry-runtime (mTLS + SPIFFE + autonomy precondition).
6. Supervision-event-bus → foundry-evidence (Ed25519 + Merkle seal).

## Assets & Data Classification

Per Bominal ADR-0028 (audit-chain + data-class taxonomy) and the `oya-check-data-class` LEAN lane.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| Fleet-state rows (per-tenant agent counts, capability versions) | `BEHAVIORAL_TENANT_PRODUCT` | High | 2y per pack (per `data-residency.md`) | Postgres |
| Capability definitions (per-tenant) | `BEHAVIORAL_TENANT_PRODUCT` + `INTERNAL_ONLY` | Medium | append-only git history | Tenant-owned git repos |
| Autonomy entitlements (per-tenant autonomy ceiling grants + expiration) | `SENSITIVE_PIPA_ART23` + `AUDIT` | Critical | 5y per pack (KR-FSS sector default) | Postgres + OpenBao |
| Deployment history (admit, canary phase, verdict, rollback) | `AUDIT` + `BEHAVIORAL_TENANT_PRODUCT` | High | 2y (HIPAA pack: 6y) | Postgres |
| Kill-switch state (engaged/disengaged + scope + reason) | `AUDIT` + `BEHAVIORAL_TENANT_PRODUCT` | Critical | 2y (HIPAA pack: 6y) | Valkey + Postgres archive |
| Supervision events (Ed25519-signed) | `AUDIT` | Critical | indefinite (audit-chain immutable) | Valkey Stream → audit-chain µservice |
| Cedar policy fragments | `INTERNAL_ONLY` (text); `SECRET` when carrying tenant identifiers | Medium | git history | `microservices/foundry-supervisor/policy/*.cedar` |
| Postgres credentials | `SECRET` | Critical | OpenBao 30d rotation | OpenBao |
| Valkey ACL tokens | `SECRET` | Critical | OpenBao 30d rotation | OpenBao |
| Supervisor Ed25519 signing keys | `SECRET` | Critical | OpenBao 90d rotation; HSM-backed where available | OpenBao |
| Operator SA token | `SECRET` | Critical | K8s default 24h rotation | K8s |
| SPIFFE SVID (per-pod identity) | `SECRET` | Critical | 1h auto-rotation | SPIRE server |

## Actors

| Actor | Trust level | Authentication | Capability |
|---|---|---|---|
| Tenant operator (human) | Untrusted external | OIDC + MFA via Application Shell | List own fleet; engage own-scope kill-switch; request capability admit via PR |
| Tenant agent (machine) | Untrusted external | OIDC client-credentials + per-tenant SPIFFE bridge | Invoke `foundry-runtime` — supervisor gates via autonomy precondition |
| Workload µservice (Workflow Studio, Application Shell agents) | Semi-trusted internal | mTLS + SPIFFE | Subscribe to supervision events for own scope; cannot invoke kill-switch directly |
| `foundry-runtime` worker | Semi-trusted internal | mTLS + SPIFFE | Receive kill-switch broadcasts; query autonomy precondition |
| `foundry-evidence` | Trusted internal | mTLS + SPIFFE | Read supervision-event-bus; seal audit-chain |
| `foundry-guardrails` | Trusted internal | mTLS + SPIFFE | Publish `GuardrailViolation` events; consumer-side relationship |
| Supervisor operator (Kubernetes Operator) | Trusted internal | SPIFFE: `spiffe://oyatie/foundry-supervisor/operator` | RW on CRDs in `foundry-tenant-*` namespaces; deny on `default` + other µservice namespaces |
| Supervisor REST + worker pods | Trusted internal | SPIFFE | Read/write Postgres + Valkey; emit signed events |
| ops-security on-call (human) | Trusted internal | OIDC + MFA + JIT via OpenBao | Engage fleet-wide kill-switch (2-person rule); rotate signing keys |
| DPO / council-privacy (human) | Trusted internal | OIDC + MFA | Read all audit-chain records; never write fleet state |
| External auditor | Read-only external (time-boxed) | OIDC + JIT short-lived | Read audit-chain + dashboards within scoped tenants per `auditor-scope.cedar` |
| Attacker — opportunistic | Untrusted | none | Scan + low-skill exploit |
| Attacker — targeted | Untrusted | none | Supply-chain aware; assume present for prod surfaces |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfig (mitigated by PR review + LEAN gates) |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case threat; mitigated by least-privilege + audit-chain + separation-of-duties + 2-person rule |

## STRIDE Threat Catalog

### Spoofing (S)

**T-S-01 — Attacker forges tenant identity to deploy a malicious capability**
- Asset: capability-deployment admit path
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations: OIDC + Cedar `tenant-scope.cedar` enforces principal.tenant_id matches resource.tenant_id; SPIFFE identity verified at REST ingress; signed PR review on capability YAML before admit accepts.
- Owner: ops-security + axis-foundry-control-plane
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2; ISO 27001 A.5.15, A.8.2, A.8.3; GDPR Art. 32; EU AI Act Art. 15

**T-S-02 — Attacker impersonates supervisor controller to bypass autonomy precondition**
- Asset: supervisor → foundry-runtime invocation path
- Likelihood: M / Impact: H (would let unauthorized capabilities execute) / Risk: **H**
- Mitigations: per-call mTLS with SPIFFE identity verification on both ends; foundry-runtime refuses any caller that's not `spiffe://oyatie/foundry-supervisor/*`; recording of caller SPIFFE on every event.
- Owner: ops-security + axis-foundry-control-plane
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6, CC7.1; ISO 27001 A.5.15, A.8.5, A.8.7; EU AI Act Art. 15

**T-S-03 — Attacker forges Ed25519 signature on supervision event**
- Asset: audit-chain integrity
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: Ed25519 key in OpenBao (90d rotation, HSM-backed where available); audit-chain Merkle verification cross-checks signature against known supervisor public key on seal-time; tampered events fail seal.
- Owner: ops-security
- Residual: L
- Frameworks: ISO 27001 A.8.24; GDPR Art. 32(1)(b); EU AI Act Art. 12

**T-S-04 — Attacker impersonates ops-security to engage fleet-wide kill-switch (denial)**
- Asset: kill-switch authority
- Likelihood: L / Impact: H (would constitute availability attack disguised as safety) / Risk: **M**
- Mitigations: fleet-wide engage requires 2-person rule via OpenBao JIT (one ops-security + one council-privacy or council-architecture); every engage emits audit-chain record with both signatures; alarm on lone-actor fleet-wide engage attempt.
- Owner: ops-security + council-architecture
- Residual: L
- Frameworks: SOC 2 CC6.1, CC8.1; ISO 27001 A.5.15, A.5.18; EU AI Act Art. 14

**T-S-05 — Attacker spoofs auditor token to pivot across tenants**
- Asset: auditor read scope
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: auditor JIT tokens scoped to specific tenants only (per `auditor-scope.cedar`); TTL ≤ 4h non-renewable; every auditor read audit-chain-emitted; mTLS client cert pinned during engagement window.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2; ISO 27001 A.5.15, A.5.18

### Tampering (T)

**T-T-01 — Capability YAML tampering via repo push (rogue deployment)**
- Asset: tenant capability definitions
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations: CODEOWNERS on capability files; signed commits; LEAN check `oya-check-capability-yaml-conformance` validates schema + autonomy tier ≤ allowed; PR review by tenant DPO before admit accepts; the supervisor's own admit-loop runs Cedar policy on `principal.tenant_id == resource.tenant_id`.
- Owner: ops-security + axis-foundry-control-plane
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.31, A.5.32, A.8.4, A.8.32; EU AI Act Art. 9

**T-T-02 — Postgres row tampering (direct DB access)**
- Asset: fleet-state + deployment-history rows
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: per-pod OpenBao-issued credentials; row-level-security per tenant; WAL archival to encrypted S3; PgAudit + Mimir alerts on direct SELECT/UPDATE outside the supervisor SA; JIT for any DB admin (2-person rule for tables with `AUDIT` data).
- Owner: ops-security + ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6, CC8.1; ISO 27001 A.5.15, A.8.4, A.8.20; GDPR Art. 32

**T-T-03 — Kill-switch state tampering in Valkey (disengaged while autonomy violated)**
- Asset: Valkey kill-switch state
- Likelihood: L / Impact: H (would let unauthorized agents continue running) / Risk: **H**
- Mitigations: Valkey ACL per-key; AOF replication-factor 2; CRD-watch + Valkey cross-check (CRD is source-of-truth, Valkey cached); state-divergence alert (`oya_kill_switch_state_divergence_total > 0`) fires page; recovery: re-publish from CRD truth.
- Owner: ops-security + axis-foundry-control-plane
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.1; ISO 27001 A.8.16; EU AI Act Art. 14

**T-T-04 — Operator CRD tampering (kubectl manual edit)**
- Asset: AgentDeployment + KillSwitch CRDs
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: K8s admission webhook signed by supervisor; OPA Gatekeeper policy refuses kubectl edits not originating from supervisor SA; audit-log on K8s API every CRD mutation; nightly drift-detector reconciles CRD state vs Postgres.
- Owner: ops-security + ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.15, A.8.32

**T-T-05 — Cedar fragment tampering (policy escape via crafted operand)**
- Asset: Cedar policy text + evaluation
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: Cedar v4 default-deny; fragments versioned in git + PR-reviewed; `oya-check-cedar-fragment-coverage` lane fuzzes every fragment; input-length bounds on REST API before policy eval; semantic-conformance test asserts deny-overrides.
- Owner: ops-security + axis-foundry-control-plane
- Residual: L
- Frameworks: ISO 27001 A.8.28; EU AI Act Art. 15

### Repudiation (R)

**T-R-01 — Capability deployer denies authorship**
- Asset: deployment event chain
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations: signed commits on capability YAML; PR record + Ed25519 audit-chain seal on `CapabilityDeployed` event including actor SPIFFE; per-changeset evidence committed.
- Owner: axis-foundry-control-plane + audit-chain
- Residual: L
- Frameworks: SOC 2 CC4.1, CC8.1; ISO 27001 A.5.27, A.8.34; GDPR Art. 5(2)

**T-R-02 — Kill-switch operator denies engagement**
- Asset: KillSwitchEngaged event chain
- Likelihood: L / Impact: H (would weaken safety-net story) / Risk: **M**
- Mitigations: every engage emits `KillSwitchEngaged{actor, reason, scope, engaged_at}` with Ed25519 signature; 2-person rule for fleet-wide produces two signatures; audit-chain Merkle proof + per-changeset evidence.
- Owner: ops-security + audit-chain
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.27, A.5.28; EU AI Act Art. 12

**T-R-03 — Rollback executed without traceable trigger**
- Asset: DeploymentRolledBack audit-chain
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: every rollback emits structured `reason` enum (`fast_burn_breach`, `slow_burn_breach`, `manual_override`, `eval_regression`, `guardrail_violation`, `post_mortem_remediation`); reason required; rollback-time fleet snapshot persisted; per-changeset evidence regenerated.
- Owner: axis-foundry-control-plane + ops-security
- Residual: L
- Frameworks: SOC 2 CC7.4, CC8.1; ISO 27001 A.5.26, A.5.27, A.8.15, A.8.16; GDPR Art. 33

### Information Disclosure (I)

**T-I-01 — Cross-tenant fleet-state leak via Postgres misconfiguration**
- Asset: per-tenant fleet-state
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations: Postgres row-level-security policy `WHERE tenant_id = current_setting('app.tenant_id')`; supervisor sets the session variable per request; LEAN check `oya-check-postgres-rls-enforced` validates the policy on every table; pen-test exercises cross-tenant query annually.
- Owner: ops-security + axis-foundry-control-plane
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.8.2, A.8.3, A.8.12; GDPR Art. 5(1)(f), Art. 25, Art. 32

**T-I-02 — Autonomy-entitlement leak (sensitive PIPA Art. 23 data)**
- Asset: per-tenant autonomy entitlement records
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: stored in OpenBao with strict access policy; Postgres-side reference is opaque token only (no entitlement details); access through OpenBao tenant-resolver only; auditor reads are tenant-scoped at OpenBao policy level.
- Owner: ops-security + council-privacy
- Residual: L
- Frameworks: GDPR Art. 9 (special category); pack-kr KR PIPA Art. 23

**T-I-03 — Capability YAML leak revealing tenant's automation logic**
- Asset: capability definitions
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations: tenant-owned git repo (tenant decides access); supervisor receives via PR webhook from tenant repo; supervisor never serves raw capability YAML cross-tenant.
- Owner: tenant + axis-foundry-control-plane
- Residual: L
- Frameworks: GDPR Art. 5(1)(f), Art. 32

**T-I-04 — Supervision-event payload leaks SLI numbers cross-tenant**
- Asset: supervision-event-bus stream
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations: Valkey ACL restricts each subscriber to its scoped tenant_id pattern; audit-chain access is privileged role only; per-tenant aggregation in any cross-tenant dashboards is DP-noise-protected (ε ≤ 1).
- Owner: ops-security + axis-foundry-control-plane
- Residual: L

**T-I-05 — Secret leak via control-plane logs**
- Asset: OpenBao-managed secrets accidentally emitted
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations: secret-scanner CI lane on every commit + log emission; OTel SDK redactor strips known secret patterns; `Secret<T>` Rust newtype with stripped Debug impl; OpenBao SecretReference never materialised into logs.
- Owner: ops-security + cloud-secrets
- Residual: M (human-error baseline)
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.17, A.8.7, A.8.12; GDPR Art. 32

### Denial of Service (D)

**T-D-01 — Postgres connection-pool exhaustion (admit-loop overload)**
- Asset: Postgres + admit-loop
- Likelihood: H / Impact: H (control-plane unavailable) / Risk: **H**
- Mitigations: PgBouncer pooled; per-tenant rate-limit on admit endpoint; admit-loop queue depth alarm; HPA on REST + worker; PgBouncer pool-exhaustion triggers degraded-mode (read-only) instead of full outage.
- Owner: ops-sre-reliability + axis-foundry-control-plane
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6, A.8.14; GDPR Art. 32(1)(c)

**T-D-02 — Valkey cluster failure breaks kill-switch state**
- Asset: kill-switch read path
- Likelihood: M / Impact: H (kill-switch latency blown) / Risk: **H**
- Mitigations: Valkey Cluster 3-shard × 2-replica; AOF every-second; CRD watch as authoritative fallback (Valkey is cache); fail-closed: on Valkey unavailability, supervisor returns "engaged" for all scopes within 2 s degradation window (safe default); recovery: AOF restore + CRD reconcile.
- Owner: ops-sre-reliability + axis-foundry-control-plane
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.14; EU AI Act Art. 14 (fail-closed for safety)

**T-D-03 — Supervision-event-bus backlog (deployment storm)**
- Asset: supervision-event-bus stream
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations: Valkey Streams (Redis wire-compat) with consumer groups + at-least-once delivery; bus-lag alarm at p99 > 500 ms; per-publisher rate-limit; backpressure into supervisor worker (worker pauses non-critical work when lag breaches).
- Owner: ops-sre-reliability
- Residual: L

**T-D-04 — Cedar evaluation latency spike (regex-DoS on input)**
- Asset: autonomy-policy-enforcement
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations: Cedar v4 has no user-controllable regex; field-length bounded at REST API; Cedar evaluation timeout 50 ms enforced (cancel + deny); fuzz-test for pathological inputs (`oya-check-cedar-fragment-coverage`).
- Owner: ops-security + axis-foundry-control-plane
- Residual: L
- Frameworks: ISO 27001 A.8.28

**T-D-05 — Kubernetes Operator reconcile-storm (CRD watch flood)**
- Asset: Operator pod resource exhaustion
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations: per-shard sharded reconciler; controller-runtime work-queue with retry backoff; reconcile-rate alarm; HPA on controller CPU; degradation: pause non-critical CRD watches when storm detected; CRD mutating-webhook rate-limit.
- Owner: ops-sre-reliability + axis-foundry-control-plane
- Residual: L

**T-D-06 — Fleet-wide kill-switch by mistake (operator error)**
- Asset: availability
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: 2-person rule for fleet-wide; pre-engage confirmation prompt; 5-second post-engage cancel window; engaging signature requires both human signatures cryptographically (one is insufficient); audit-chain captures both signatures.
- Owner: ops-security + council-architecture
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.18; EU AI Act Art. 14

### Elevation of Privilege (E)

**T-E-01 — Tenant escalates autonomy tier T0→T3 without DPA entitlement**
- Asset: autonomy-policy-enforcement
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations: Cedar default-deny; tier escalation requires explicit `principal.billing_components` claim issued by OpenBao at onboarding; entitlement issuance requires DPO signature; pen-test attempts annually.
- Owner: ops-security + council-privacy
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2; ISO 27001 A.5.15, A.5.18, A.8.2; GDPR Art. 22; EU AI Act Art. 14

**T-E-02 — Supervisor SA token compromised → unauthorized fleet writes**
- Asset: supervisor ServiceAccount
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: SA token bound to pod identity; 24h rotation; network policy: only supervisor pods may reach Postgres/Redis write endpoints; Postgres + Valkey validate SPIFFE identity matches expected SA.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.17, A.8.5, A.8.7

**T-E-03 — Kubernetes Operator privilege escalation (RBAC misconfig)**
- Asset: Operator RBAC
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: Operator ClusterRole limited to `Agent`, `AgentDeployment`, `AutonomyPolicy`, `KillSwitch` CRDs across `foundry-tenant-*` namespaces; no `*` verbs; Terraform-managed RBAC; LEAN check asserts RBAC matches declared state.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.5.18, A.8.4

**T-E-04 — Capability YAML escapes intended autonomy tier (smuggling)**
- Asset: capability admit-loop
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: admit-loop parses YAML and bounds the autonomy_level field to ENUM (`T0 | T1 | T2 | T3`); higher-than-tenant-entitled tiers refused; LEAN check on schema; PR review enforces.
- Owner: axis-foundry-control-plane + ops-security
- Residual: L

**T-E-05 — Operator-level access used to delete tenant fleet state**
- Asset: Postgres admin operations
- Likelihood: L (insider threat) / Impact: H / Risk: **M**
- Mitigations: DB admin requires JIT (2-person rule for delete); soft-delete with 30-day grace; mass-deletion anomaly alert; recovery via WAL replay.
- Owner: ops-security
- Residual: L

## LINDDUN Privacy-Threat Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | Supervision events | Multiple events per agent could be linked to a single tenant end-user (e.g., capability + cost-runaway pattern reveals workflow shape). | Per-tenant scoping at subscriber level; aggregation at bus-publisher cross-tenant DP-noise. | L-M |
| T-L-02 | Identifiability | Autonomy-entitlement records | Records carry tenant identifiers + DPO authorisation; high re-identification potential within tenant. | Per-tenant access only; auditor scope time-boxed; salted-hash on cross-tenant aggregates. | L |
| T-L-03 | Non-repudiation | Tenant operator capability publishing | Tenants might deny authorship of capability that misbehaves. | Signed commits + PR audit + per-changeset evidence; Ed25519 audit-chain seal. | L |
| T-L-04 | Detectability | Deployment timing | Burst of deployments correlates with tenant business cycles. | Reasonable; declared in tenant DPA. | M |
| T-L-05 | Disclosure | Auditor read access | Auditor could pivot from one tenant to another via supervisor aggregate views. | Auditor scope per-tenant (Cedar); pen-test annually. | L |
| T-L-06 | Unawareness | End-user of tenant | End-user may not know agents acting on their behalf are gated by oyatie supervisor. | Tenant DPA includes joint-controllership clause + transparency cascade. | M |
| T-L-07 | Non-compliance | GDPR Art. 22 (automated decisions) | Autonomy precondition is itself an automated decision with legal effects if it refuses a tenant user's invocation. | Carve-out: operational decision, not solely-automated decision producing legal effects; tenant can manually override with 2-person rule. | L |

## Mitigations Catalog (cross-reference)

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Postgres row-level-security per tenant | Preventive | ops-security | `oya-check-postgres-rls-enforced` lane |
| OpenBao-issued per-pod credentials, 30d rotation | Preventive | ops-security | OpenBao audit log |
| Ed25519 audit-chain seal on every supervision event | Detective + Non-repudiation | audit-chain | audit-chain regression tests |
| Cedar default-deny + per-tenant scope policy | Preventive | ops-security | `oya-check-cedar-fragment-coverage` lane |
| 2-person rule for fleet-wide kill-switch + DB admin + signing-key rotation | Preventive (insider) | ops-security | OpenBao JIT elevation log |
| Per-tenant rate-limit on admit/REST/kill-switch | Preventive (DoS) | axis-foundry-control-plane | REST metrics |
| Kubernetes Operator RBAC scoped to per-tenant ns | Preventive | ops-security | Terraform-state diff |
| Network policy: supervisor → Postgres + Valkey only | Preventive | ops-sre-reliability | K8s NetworkPolicy review |
| CRD admission webhook + drift-detector | Preventive + Detective | axis-foundry-control-plane | drift-detector job |
| Capability YAML LEAN schema lane | Preventive | axis-foundry-control-plane | `oya-check-capability-yaml-conformance` lane |
| Soft-delete + 30d recovery window | Detective + Recovery | ops-sre-reliability | mass-deletion anomaly alert |

## Residual Risk Acceptance

| Risk ID | Residual | Why accepted | Re-review |
|---|---|---|---|
| T-I-05 (secret leak via logs) | M | Human-error baseline; mitigated via detection + rotation. | Quarterly |
| T-L-04 (detectability via timing) | M | Tenant business reality; consent at onboarding. | Annually |
| T-L-06 (end-user unawareness) | M | Tenant joint-controllership cascade. | Annually |

Sign-off (RW until council captures):
- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`

## Per-Pack Overlay Sections

### pack-kr (KR PIPA + ISMS-P)

- KR PIPA Art. 23 (sensitive PI): autonomy entitlement records carry tenant-DPO authority — treated as sensitive; OpenBao-resident with PIPC retention.
- KR PIPA Art. 29 (technical safeguards): every mitigation above maps to one of the 12 prescribed safeguards; cross-mapped in `compliance.md`.
- KR PIPA Art. 23-2 (cross-border): KR tenant fleet state stays in pack-kr.

### pack-us-healthcare (HIPAA-scoped)

- HIPAA §164.312(a)(1): per-tenant RLS + Cedar; audit-chain emission covers Audit Controls.
- HIPAA §164.308(a)(1)(ii)(A): this threat-model + DPIA together satisfy the Risk Analysis requirement.
- HIPAA §164.502(b): minimum-necessary applied to all supervisor APIs; auditor scope tenant-bound.
- HIPAA §164.316(b)(2): retention extended to 6y for pack-us-healthcare audit-chain.
- BAA-required: tenant signs BAA before fleet ingest enabled.

### pack-eu (GDPR + EU AI Act + NIS2 + eIDAS)

- **EU AI Act 2024/1689**: foundry-supervisor is a control-plane for high-risk AI systems (capabilities may fall in Annex III §1–8). This threat-model + DPIA + risk-management plan (compliance.md §"EU AI Act") satisfy Art. 9 (risk management), Art. 12 (record-keeping via audit-chain), Art. 14 (human oversight via 2-person rule on kill-switch), Art. 15 (cybersecurity via this threat-model + LEAN lanes).
- GDPR Art. 25 (PbD): default-deny Cedar + per-tenant fleet scope.
- GDPR Art. 22 (automated decisions): autonomy precondition is operational decision per §"LINDDUN T-L-07".
- NIS2 (2022/2555): when oyatie crosses Annex I/II thresholds, supervisor incidents follow the 24h+72h+1mo reporting timelines.
- eIDAS 910/2014: Ed25519 audit-chain seals satisfy AdES for EU-tenant supervision-event records.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Pack-overlay sections in `regional-packs/<pack>/foundry-supervisor-overlay.md` carry pack-specific legal-citation depth.

## Re-review Triggers

- Any change to a trust boundary, actor list, or CRD shape.
- Any Layer-A version upgrade (Postgres / Valkey / kube-rs / Cedar) where release notes mention security fixes.
- Any new pack activation.
- Annual scheduled review (Q2).
- Post-incident (any Sev-1/2 in foundry-supervisor or in a capability it manages).
- Pen-test or audit finding.
- New high-risk AI Act Annex III sub-domain entering the tenant base.

## References

- ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0139, ADR-0131, ADR-0133, ADR-0140.
- `microservices/foundry-supervisor/PRD.md`.
- `microservices/foundry-supervisor/dpia.md`.
- `microservices/foundry-supervisor/policy/supervisor-isolation.md`.
- `microservices/foundry-supervisor/policy/data-residency.md`.
- `microservices/foundry-supervisor/compliance.md`.
- Microsoft STRIDE; LINDDUN; OWASP Top 10 (2021) + API Top 10 (2023); NIST SP 800-154.
- EU AI Act 2024/1689 — `eur-lex.europa.eu/eli/reg/2024/1689`.
- Cedar v4 — `cedarpolicy.com`.
- Kubernetes Operator pattern — `kubernetes.io/docs/concepts/extend-kubernetes/operator/`.
