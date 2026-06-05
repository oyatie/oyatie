---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: observability
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-observability + ops-security
deciders: council-architecture, ops-security, axis-observability, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + NIST SP 800-154
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/agentic-slo-gated-promotion.json, /specs/per-microservice-flat-layout.json]
review_cadence: quarterly + on every Layer-A or Layer-B architecture change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.8.2, A.8.3, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.25, A.8.26, A.8.27, A.8.28"
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35"
suggested_frameworks_by_pack:
  pack-kr: ["KR-ISMS-P §2.1-2.12 (보호조치)", "KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2", "KR 전자문서법 Arts. 5/6/7"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308 (Administrative Safeguards)", "HIPAA §164.310 (Physical Safeguards)", "HIPAA §164.312 (Technical Safeguards)", "HIPAA §164.314 (Organizational Requirements)", "HIPAA §164.316 (Policies and Procedures)"]
  pack-eu: ["GDPR Arts. 25 + 32 + 35 + 44-50 (transfers)", "eIDAS 910/2014 (when SLO data signed)", "NIS2 2022/2555 (when oyatie hits Annex I/II thresholds)"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24/26-2"]
  pack-sg: ["PDPA 2012 §11-26 (Protection Obligation, Retention Limitation, Transfer Limitation)", "MAS-TRM v2021 §11-12"]
  pack-au: ["Privacy Act 1988 APP 1-13 (esp. APP 6, 8, 11)", "APRA-CPS 234 §29-44 (Information Security)"]
  pack-in: ["DPDPA 2023 §6-10 (consent + notice + processing limits)", "RBI Master Direction on Outsourcing of IT Services 2023"]
  pack-br: ["LGPD Arts. 6, 7, 11, 14, 18, 33, 46, 48", "BACEN Res. 4.893/2021"]
  pack-ae: ["UAE PDPL Federal Decree-Law No. 45/2021 Arts. 5/6/9/15"]
  pack-ksa: ["PDPL Royal Decree M/19/2021 Arts. 4-9", "SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Threat Model: observability µservice

## Purpose

Identify, classify, and mitigate threats to the observability µservice's confidentiality, integrity, availability, and privacy posture. The observability substrate is the gate authority for every oyatie µservice's `dev → staging → production` promotion; a compromise here cascades to every product. This document is the canonical security artifact reviewed by SOC 2 Type 2 examiners, ISO 27001 auditors, and GDPR DPAs at first-tenant onboarding.

## Scope

### In-scope

All components introduced by ADR-0139 (agentic SLO-gated promotion) and ADR-0131 (per-microservice flat layout) for the observability µservice, deployed in a **dedicated observability Kubernetes cluster** (decision confirmed 2026-05-17; matches hyperscaler practice — AWS Managed Prometheus runs in its own VPC; GCP Managed Service for Prometheus runs in its own project; Grafana Labs Cloud runs on dedicated infra):

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| Grafana Alloy (per-µservice OTel collector) | `oya-observability-slo-engine-*` (10 crates) |
| Prometheus + Grafana Mimir (TSDB) | `oya-observability-otel-ingest-*` (5 crates) |
| Grafana Loki (logs) | OpenSLO manifests at `microservices/<ms>/slos/*.openslo.yaml` |
| Grafana Tempo (traces) | Promotion-eligibility metrics in Mimir |
| Grafana Pyroscope (profiles) | Per-component release pointer Git refs |
| Grafana (UI + dashboards) | `oya-governance-promotion-readiness` Jenkins/`oya gate` CI lane |
| Prometheus Alertmanager (routing) | Rollback primitive (signed force-fast-forward) |
| Grafana OnCall (paging) | Canary cohort weighting (service-mesh integration) |

### Out-of-scope

- Threats to the underlying Kubernetes cluster, container runtime, or hyperscaler IaaS layer — owned by the `cloud-k8s` µservice's threat model.
- Threats to the workload µservices themselves (tenancy, ontology, workflow, mail, etc.) — each owns its own threat-model.md.
- Threats to OpenBao secret-manager itself — owned by the `cloud-secrets` µservice's threat model. This document inherits OpenBao threats as upstream and references them.
- Threats to Jenkins CI agents and Forgejo required-check plumbing — owned by the `governance` µservice (CI substrate) threat model.
- Threats to Bominal-side observability counterparts — separate Bominal threat-model; oyatie inherits decisions where applicable per `feedback_bominal_inheritance_precedence.md`.

## Trust Boundaries

```text
┌─ Internet ─────────────────────────────────────────────────────────────────┐
│                                                                            │
│   Tenant operators                Customer applications                    │
│         │                                  │                               │
│         │ (HTTPS, OIDC, mTLS)              │ (per-tenant OTel API key)     │
│         ▼                                  ▼                               │
│  ┌─ Public ingress (Envoy/Istio gateway) ──────────────────────────────┐   │
│  │  - TLS termination                                                  │   │
│  │  - WAF (rate-limit + OWASP CRS)                                     │   │
│  │  - DDOS protection (provider-level + Cloudflare)                    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │                                             │
└──────────────────────────────│─────────────────────────────────────────────┘
                               ▼
┌─ Dedicated observability cluster ──────────────────────────────────────────┐
│                                                                            │
│  Trust boundary 1: External → Cluster ingress                              │
│                                                                            │
│  ┌─ slo-engine-rest ─────────────┐    ┌─ Grafana UI ─────┐                 │
│  │  OIDC tenant-scoped reads     │    │  SSO + RBAC      │                 │
│  └───────────────────────────────┘    └──────────────────┘                 │
│             │                                                              │
│  Trust boundary 2: Per-tenant Mimir multi-tenancy (X-Scope-OrgID header)   │
│             │                                                              │
│  ┌─ Mimir (multi-tenant TSDB) ───────────────────────────────────────┐     │
│  │  - Per-tenant: hashed-customer-id; no cross-tenant query          │     │
│  │  - Reserved tenant 'oya-ci' for promotion-readiness lane reads    │     │
│  │  - Reserved tenant 'oya-self' for observability self-SLOs         │     │
│  └───────────────────────────────────────────────────────────────────┘     │
│  ┌─ Loki  ─┐ ┌─ Tempo ─┐ ┌─ Pyroscope ─┐ ┌─ Alertmanager ─┐ ┌─ OnCall ─┐   │
│  │  same   │ │  same   │ │  same       │ │ webhook routes │ │ rotation │   │
│  │ tenancy │ │ tenancy │ │ tenancy     │ │                │ │ + page   │   │
│  └─────────┘ └─────────┘ └─────────────┘ └────────────────┘ └──────────┘   │
│                                                                            │
│  Trust boundary 3: Workload cluster → observability cluster (OTel send)    │
│             │                                                              │
│  ┌─ Grafana Alloy (per-workload-µservice; mTLS to Mimir/Loki/Tempo) ┐      │
│  │  - Per-µservice OTel API key issued by OpenBao                    │      │
│  │  - X-Scope-OrgID = sha256(tenant_id)[..16]                        │      │
│  │  - Per-µservice OTel collector sidecar OR DaemonSet               │      │
│  └───────────────────────────────────────────────────────────────────┘      │
│                                                                            │
│  Trust boundary 4: CI runner → Mimir read API                              │
│             │                                                              │
│  ┌─ slo-engine-worker (long-lived service) ──────────────────────────┐     │
│  │  - Reads Mimir as tenant 'oya-self' for own SLOs                  │     │
│  │  - Writes eligibility verdict metrics as tenant 'oya-ci' (CI tenant) │  │
│  │  - Emits signed eligibility-changed events to the governance promotion pipeline             │     │
│  └───────────────────────────────────────────────────────────────────┘     │
│                                                                            │
│  Trust boundary 5: Rollback primitive → Git refs API                       │
│             │                                                              │
│  ┌─ Git refs PATCH (signed commits, GitHub API) ─────────────────────┐     │
│  │  - WORKFLOW_PAT scoped to release/<ms>/* refs only                │     │
│  │  - Audit emission: every PATCH writes Ed25519 audit record        │     │
│  └───────────────────────────────────────────────────────────────────┘     │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

Five trust boundaries:
1. **External → Cluster ingress** (TLS, WAF, DDoS).
2. **Per-tenant Mimir multi-tenancy** (`X-Scope-OrgID` enforcement; the load-bearing isolation boundary).
3. **Workload cluster → observability cluster** (mTLS, per-µservice OTel API key).
4. **CI runner → Mimir read API** (reserved `oya-ci` tenant; bounded scopes).
5. **Rollback primitive → Git refs API** (signed; PAT-scoped).

## Assets & Data Classification

Per Bominal ADR-0028 (audit-chain + data-class taxonomy) and the `oya-check-data-class` LEAN lane.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| Metrics (per-µservice SLI samples) | `BEHAVIORAL_TENANT_PRODUCT` | Medium | 30d hot + 24mo cold (Mimir blocks → object storage) | Mimir |
| Logs (per-µservice structured logs) | `BEHAVIORAL_TENANT_PRODUCT` + sometimes `PII_IDENTIFYING` (user-id fields) + occasionally `PHI` (pack-us-healthcare) | High | 14d hot + 12mo cold | Loki |
| Traces (per-µservice request traces) | `BEHAVIORAL_TENANT_PRODUCT` + transient `PII_QUASI_IDENTIFIER` (URLs, IPs in spans) | High | 7d hot + 6mo cold | Tempo |
| Profiles (per-µservice CPU/memory profiles) | `INTERNAL_ONLY` | Low | 14d | Pyroscope |
| OpenSLO manifests | `INTERNAL_ONLY` (manifest text) | Low | append-only git history | `microservices/<ms>/slos/` in repo |
| Promotion-eligibility verdicts | `AUDIT` + `BEHAVIORAL_TENANT_PRODUCT` | High | Mimir 90d hot + 2y cold + per-changeset evidence | Mimir + `microservices/<ms>/evidence/multispectrum/` |
| Release-pointer history | `AUDIT` | High | append-only git refs + Ed25519 audit chain | Git + audit-chain µservice |
| Cedar policy fragments | `INTERNAL_ONLY` (policy text); `SECRET`-class when carrying tenant identifiers | Medium | git history | `microservices/observability/policy/*.cedar` |
| Per-µservice OTel API keys | `SECRET` | Critical | OpenBao with 30d rotation | OpenBao via cloud-secrets µservice |
| Mimir multi-tenant API keys (CI tenant + per-tenant) | `SECRET` | Critical | OpenBao with 30d rotation | OpenBao |
| Ed25519 signing keys (audit-chain emission + ref signing) | `SECRET` | Critical | OpenBao with 90d rotation + HSM-backed where available | OpenBao |
| Grafana OnCall integration tokens | `SECRET` | Critical | OpenBao with 30d rotation | OpenBao |
| Tenant identifiers (hashed customer-id used as `X-Scope-OrgID`) | `SENSITIVE_PIPA_ART23` (under KR PIPA Art. 23 — sensitive personal info via re-identification potential) | High | Salted-hash recorded; raw mapping in OpenBao | OpenBao tenant-resolver |
| Burn-rate alert payloads (Grafana OnCall routing) | `BEHAVIORAL_TENANT_PRODUCT` | Medium | 30d | OnCall + Alertmanager state |
| Audit-chain seal records (per Mimir block flush) | `AUDIT` | High | append-only; immutable | audit-chain µservice |

## Actors

| Actor | Trust level | Authentication | Capability |
|---|---|---|---|
| External tenant operator (human) | Untrusted external | OIDC + MFA via Application Shell | Read own tenant's SLOs, dashboards, OnCall incidents; author OpenSLO manifests via PR review |
| Customer application (machine) | Untrusted external | Per-tenant OTel API key (rotated 30d) | Send OTel signal with own tenant header |
| Workload µservice (in same trust domain) | Semi-trusted internal | mTLS + per-µservice OTel key (rotated 30d) | Send OTel signal as own µservice |
| oyatie CI agent (Jenkins) | Semi-trusted internal | OIDC-bound Jenkins identity + reserved Mimir tenant `oya-ci` read-only API key | Read promotion-eligibility metrics; consume signed `eligibility-changed` promotion events |
| slo-engine-worker (long-lived service) | Trusted internal | OpenBao-issued service-account token | Query Mimir as `oya-self`; write verdicts as `oya-ci`; emit dispatch events |
| Reviewer agent (oya-pr-review lane) | Trusted internal | OIDC-bound CI identity | Read dashboards; refuse merges that violate gate |
| Council-architecture / ops-security operators (human) | Trusted internal | OIDC + MFA + JIT elevation via OpenBao | Admin-level Grafana access (read all tenants); RW on OpenSLO manifests via PR review |
| External auditor (SOC 2 / ISO 27001 / etc.) | Read-only external on a time-boxed window | OIDC + MFA + JIT short-lived token via OpenBao | Read-only on Grafana + audit-chain export; cannot pivot to tenant data |
| Attacker — opportunistic | Untrusted | none | Scans + low-skill exploitation; assume always present |
| Attacker — targeted (financially or geopolitically motivated) | Untrusted | none | Sophisticated; supply-chain awareness; assume present for prod-tier surfaces |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure manifests, alerts, or runbooks (mitigated by PR-review + LEAN gates) |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case threat actor for confidentiality; mitigated by least-privilege + audit-chain + separation-of-duties |

## STRIDE Threat Catalog

Each threat carries: ID; category; asset; description; likelihood (L/M/H); impact (L/M/H); risk score (likelihood × impact); mitigations (concrete); owner; residual risk; framework controls satisfied.

### Spoofing (S)

**T-S-01 — Tenant-A submits OTel signal claiming `X-Scope-OrgID` of Tenant-B**
- Asset: Mimir multi-tenancy boundary
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - OTel API key is per-tenant; key carries the bound tenant ID as a non-modifiable claim (signed by OpenBao).
  - Grafana Alloy is configured to enforce the `X-Scope-OrgID` header from the API-key binding, refusing tenant-supplied overrides.
  - Mimir distributor validates the `X-Scope-OrgID` matches the inbound key's bound-tenant claim; mismatch returns 401 + audit-emit `tenant_spoofing_attempt`.
- Owner: ops-security
- Residual: L (key compromise required + audit visibility on attempt)
- Frameworks: SOC 2 CC6.1, CC6.2, CC6.6; ISO 27001 A.5.15, A.5.17, A.8.2, A.8.3; GDPR Art. 32(1)(a)(b); KR PIPA Art. 29 (technical safeguards)

**T-S-02 — Attacker impersonates CI runner to write false eligibility verdicts**
- Asset: `oya-ci` tenant write path
- Likelihood: M / Impact: H (could permit bad-code promotion or block good-code) / Risk: **H**
- Mitigations:
  - `oya-ci` Mimir API key issued only to the in-cluster `slo-engine-worker` ServiceAccount; not exposed to Jenkins CI agents (runners READ via separate read-only key).
  - Mimir ingester validates: only the `oya-self` and `oya-ci` reserved tenants accept writes for `oya_promotion_*` metric families; other tenants emitting these metric names get rejected and audit-emitted.
  - Recording rules in Mimir cross-check verdict-write signatures against the worker's known SPIFFE identity.
- Owner: axis-observability
- Residual: L
- Frameworks: SOC 2 CC6.1, CC7.1, CC7.4; ISO 27001 A.5.15, A.8.3, A.8.7; GDPR Art. 32(1)(b); pack-kr KR-ISMS-P §2.5

**T-S-03 — Attacker forges Grafana OnCall webhook to suppress / fake incidents**
- Asset: Grafana OnCall integration
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - OnCall webhooks signed with HMAC-SHA256 (shared secret from OpenBao, rotated 30d).
  - Alertmanager → OnCall path is in-cluster only; no external ingress.
  - Two-channel incident corroboration: any verdict transition that fires OnCall ALSO writes a Mimir metric; on-call rotation runbook requires both channels green before declaring "resolved."
- Owner: ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC7.2, CC7.4; ISO 27001 A.5.24, A.5.25; GDPR Art. 33 (breach notification reliability)

**T-S-04 — Attacker impersonates the rollback primitive to revert healthy production**
- Asset: Git refs API; release-pointer history
- Likelihood: L / Impact: H (would constitute an availability attack disguised as rollback) / Risk: **M**
- Mitigations:
  - Rollback PATCH calls require Ed25519 signature from `slo-engine-worker`'s SPIFFE identity; GitHub repository ruleset rejects unsigned writes to `release/*/production`.
  - PAT scope limited to `release/*/staging` and `release/*/production` pattern; cannot touch other refs.
  - Audit-chain emission required pre-write; rollback authority owns "what triggered this rollback?" attribution.
- Owner: axis-observability + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6, CC7.2; ISO 27001 A.5.17, A.8.5, A.8.7; GDPR Art. 32(1)(b)(c)

**T-S-05 — Attacker impersonates an external auditor's JIT token to read all tenants**
- Asset: Auditor read scope
- Likelihood: L / Impact: H (would expose entire tenant dataset) / Risk: **M**
- Mitigations:
  - Auditor JIT tokens scoped to specific tenants (the audit-target subset), not "all tenants."
  - Token TTL ≤ 4 hours; non-renewable without ops-security re-issue.
  - Every auditor read is itself audit-emitted (audit-chain of audits).
  - mTLS client cert pinned to the auditor firm's gateway during the engagement window.
- Owner: ops-security + council-privacy
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2, CC8.1; ISO 27001 A.5.15, A.5.17, A.5.18, A.8.2, A.8.3; GDPR Art. 28 (processor); pack-us-healthcare HIPAA §164.308(a)(4)(ii)(B) (access authorization)

### Tampering (T)

**T-T-01 — OpenSLO manifest tampering via repo push**
- Asset: `microservices/<ms>/slos/*.openslo.yaml`
- Likelihood: M / Impact: H (false SLO target → false eligibility verdict → bad code reaches prod) / Risk: **H**
- Mitigations:
  - All OpenSLO changes require PR review via the `oya-pr-review` lane (per ADR-0139 + ADR-0131 promotion-readiness gating).
  - Per-µservice OpenSLO files protected by CODEOWNERS scoped to `axis-observability + council-architecture`.
  - LEAN check `oya-check-openslo-conformance` (NEW; added to Slice D) validates schema + reasonable burn-rate thresholds (block manifests that set fast-burn > 100% or SLO target < 99%).
  - Schema regression test asserts OpenSLO v1.0 conformance.
- Owner: axis-observability
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.31, A.5.32, A.8.32, A.8.33; GDPR Art. 32(1)(b)

**T-T-02 — Mimir block-storage object tampering (S3-compatible backend)**
- Asset: Mimir cold-tier blocks
- Likelihood: L (cloud-secrets µservice's threat model owns object-storage credentials) / Impact: H / Risk: **M**
- Mitigations:
  - S3 bucket policy: write-once-read-many (WORM) where supported (e.g., S3 Object Lock in Compliance mode).
  - Server-side encryption (SSE-KMS) with key in dedicated KMS keyring; key access logged.
  - Mimir block-validator periodically verifies block SHAs against signed metadata; mismatch quarantines the block + audit-emit.
  - Bucket access via service-account IAM only; no human direct access without ops-security JIT elevation.
- Owner: cloud-secrets + axis-observability
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6, CC8.1; ISO 27001 A.8.11, A.8.12, A.8.24, A.8.25; GDPR Art. 32(1)(a)(b); pack-eu GDPR Art. 44 (transfers — KMS key region locks data-residency)

**T-T-03 — Recording-rule tampering (Mimir rule files)**
- Asset: `oya:current_verdict:by_microservice_env`, `oya:all_eligible:by_sha`, etc.
- Likelihood: L / Impact: H (false aggregations → false gate decisions) / Risk: **M**
- Mitigations:
  - Recording rules versioned in git at `microservices/observability/iac/helm/mimir/recording-rules.yaml`; PR-required for any change.
  - LEAN check `oya-check-mimir-recording-rule-conformance` (NEW; added Slice D) validates rule shapes + expected metric labels.
  - Mimir rule-evaluator emits its own SLI; if rule evaluation latency spikes or error rate climbs, the gate's `slo-engine-worker` fails-closed (verdicts = `held` until evaluator green).
- Owner: axis-observability
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.8.32, A.8.33; GDPR Art. 32(1)(b)

**T-T-04 — Eligibility verdict metric tampering at emission time**
- Asset: `oya_promotion_eligibility_verdict` metric writes
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Verdict writes accompanied by Ed25519 signature in label `signature=<sig>`; CI lane validates signature against worker's known public key before consuming.
  - Mimir distributor validates the metric-emitter's SPIFFE identity matches the worker SA.
  - Tampering attempts produce a `gate_verdict_signature_invalid` Mimir metric; on-call paged.
- Owner: axis-observability
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.1; ISO 27001 A.5.17, A.8.7; GDPR Art. 32(1)(b)(c)

**T-T-05 — Git refs PATCH replay attack (rollback called twice with old SHA)**
- Asset: Per-component release pointer
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Rollback PATCH carries monotonic-counter as part of the signed message; replay fails the signature-time check.
  - audit-chain seal includes a wall-clock and a logical-clock; out-of-order rollbacks are detected on read.
  - Release-pointer ledger (in Mimir as `oya_promotion_release_pointer_*` metrics) carries `(current_sha, prior_sha, executed_at)` tuple; CI lane can verify causality.
- Owner: axis-observability
- Residual: L
- Frameworks: SOC 2 CC7.2; ISO 27001 A.8.20, A.8.21; GDPR Art. 32(1)(b)

### Repudiation (R)

**T-R-01 — Promotion executed but actor denies authorship**
- Asset: PromotionExecuted event chain
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Every PromotionExecuted event includes `actor=<jenkins-build-id>` + worker SPIFFE-identity + Ed25519 audit-chain seal per Bominal ADR-0028.
  - audit-chain seal carries Merkle proof; tamper-evident.
  - Per-changeset evidence at `microservices/<ms>/evidence/multispectrum/<change_id>-<unix_ts>.json` is git-committed; commit signed by author per required-check/PR policy.
- Owner: axis-observability + audit-chain
- Residual: L
- Frameworks: SOC 2 CC4.1, CC8.1; ISO 27001 A.5.27, A.5.28, A.8.15; GDPR Art. 5(2) (accountability), Art. 30 (records of processing); pack-eu eIDAS 910/2014 (qualified signature when applicable)

**T-R-02 — OpenSLO author denies authorship of a manifest change**
- Asset: OpenSLO manifest commits
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - Commits to `microservices/*/slos/*.openslo.yaml` require signed commits plus green Jenkins required checks on `dev`.
  - PR review record + commit signature provide non-repudiation.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.8.34; GDPR Art. 5(2)

**T-R-03 — Rollback executed without traceable trigger**
- Asset: Rollback audit-chain
- Likelihood: L / Impact: H (would weaken rollback-as-safety-net story) / Risk: **M**
- Mitigations:
  - Every rollback emits `RollbackExecuted{microservice, from_sha, to_sha, reason, burn_rate_snapshot, executed_at}`; `reason` is a structured enum (`fast_burn_breach | manual_override | post_mortem_remediation`) and required.
  - Per-changeset evidence regenerated post-rollback.
  - Mimir snapshot at rollback-time persisted to cold-tier with explicit retention extension.
- Owner: axis-observability + ops-security
- Residual: L
- Frameworks: SOC 2 CC7.4, CC8.1; ISO 27001 A.5.26, A.5.27, A.8.15, A.8.16; GDPR Art. 33

### Information Disclosure (I)

**T-I-01 — Cross-tenant query leak via Mimir misconfiguration**
- Asset: Mimir per-tenant data
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Mimir runs with `multitenancy_enabled: true` always; no exception.
  - LEAN check `oya-check-mimir-tenancy-enforced` (NEW; Slice D) validates the Helm chart values + asserts no `tenant=*` wildcard exists in any query frontend.
  - Per-tenant query authorization enforced by Mimir distributor (server-side, not client-side).
  - Penetration test against tenant-boundary scheduled annually + on every Mimir version upgrade.
  - Threat hunt: weekly `oya:cross_tenant_query_attempt:rate` SLO (target = 0).
- Owner: ops-security + axis-observability
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.5.18, A.8.2, A.8.3, A.8.12; GDPR Art. 5(1)(f), Art. 25, Art. 32; pack-kr KR PIPA Art. 23 (sensitive data); pack-us-healthcare HIPAA §164.312(a)(1) (access control)

**T-I-02 — PII leakage through trace span attributes**
- Asset: Tempo traces; per-µservice trace emission
- Likelihood: H (engineers regularly include user-id, email, URL in spans) / Impact: H (GDPR/PIPA violation) / Risk: **H**
- Mitigations:
  - OTel SDK is pre-configured with a redaction processor that strips fields matching common PII patterns (`email`, `phone`, `ssn`, `pin`, `password`, `card`).
  - Custom `data_class` annotation in span emission code; the redactor honours `data_class=PII` → drops the attribute.
  - Sampling reduces blast radius; default span sample = 1% in production for non-error paths.
  - Quarterly trace-content scan with synthetic-PII detector (in CI for every µservice).
  - Tenant-facing dashboard hides span attributes that the redactor missed but the dashboard explicitly displays (`data_class=PII` attributes are masked at view time even if persisted, defense-in-depth).
- Owner: axis-observability + each workload µservice owner
- Residual: M (engineering discipline gap; never fully eliminated)
- Frameworks: SOC 2 CC6.7; ISO 27001 A.8.11, A.8.12, A.8.32; GDPR Art. 5(1)(c) (data minimisation), Art. 25 (privacy-by-design), Art. 32; pack-kr KR PIPA Art. 3 (collection-limitation); pack-us-healthcare HIPAA §164.512(e) (de-identification); pack-jp APPI Art. 17

**T-I-03 — Log volume / cardinality leak revealing tenant business behavior**
- Asset: Loki log streams + metric cardinality
- Likelihood: M / Impact: M (competitive intel via traffic patterns) / Risk: **M**
- Mitigations:
  - Per-tenant log-volume aggregates are themselves `BEHAVIORAL_TENANT_PRODUCT`; not exposed cross-tenant in any view.
  - Cardinality limits enforced per-tenant (Mimir + Loki); excess data is dropped (with an explicit per-tenant SLO).
  - Public dashboards (e.g., the Grafana welcome screen) NEVER include per-tenant metric labels.
- Owner: axis-observability
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.12; GDPR Art. 5(1)(f), Art. 32

**T-I-04 — OnCall paging payload leaks SLI numbers cross-tenant**
- Asset: OnCall webhook payload
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - OnCall integrations are scoped per-tenant; the platform-level OnCall installation routes only the tenant-owned escalations.
  - Payload schema explicitly carries `tenant_id_hashed` (not raw); the human-readable description is the SLO name, not the metric series.
  - Cross-tenant aggregations in OnCall are forbidden by Cedar policy (see `policy/oncall-scope.cedar`).
- Owner: ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15; GDPR Art. 32

**T-I-05 — Aggregated SLO data exposed via public dashboards or API leaks per-tenant identity**
- Asset: `slo-engine-rest` public surface
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - `slo-engine-rest` endpoints require OIDC + scope `slo:read:<tenant>`; no anonymous reads.
  - Aggregated cross-tenant metrics (if ever produced for marketing / leaderboard) are differential-privacy-clean: each per-tenant contribution is ε-bounded with ε ≤ 1, and the per-tenant tag is stripped before aggregation.
- Owner: axis-observability
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.5; GDPR Art. 25, Art. 26

**T-I-06 — Secret (OTel API key / Mimir tenant key / Ed25519 signing key) leaked via logs**
- Asset: OpenBao-managed secrets emitted accidentally
- Likelihood: M (engineers log error context including config dumps) / Impact: H (cascades to broad compromise) / Risk: **H**
- Mitigations:
  - Secret-scanner CI lane (`oya-governance-evidence-secret-scan` — already exists) scans every commit + log emission for known secret patterns.
  - OTel SDK redactor strips known-secret patterns at emission time.
  - OpenBao SecretReference materialisation never logs the raw secret; wraps in a `Secret<T>` type with intentionally-stripped `Debug` impl.
  - Rotation policy: 30d for API keys; 90d for signing keys (rotate-out before leaked secret expires).
  - Secret-leak runbook: detection → immediate rotation → forensic trace of how it leaked → engineering education.
- Owner: ops-security + cloud-secrets
- Residual: M (human error baseline)
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.17, A.8.7, A.8.12; GDPR Art. 32(1)(a)(b)(c)(d)

### Denial of Service (D)

**T-D-01 — Mimir ingester overload via burst-write from one tenant**
- Asset: Mimir ingest path
- Likelihood: H / Impact: H (gate becomes unavailable for everyone if Mimir is down) / Risk: **H**
- Mitigations:
  - Per-tenant rate limits in Mimir distributor: max ingest rate + max active series + max sample rate; defaults sized in `microservices/observability/capacity-model.md`.
  - Per-tenant cardinality limits (max distinct series per tenant); excess returns 429.
  - Mimir distributor horizontal autoscaling; HPA based on CPU + queue depth.
  - Mimir ingester replication factor ≥ 3; loss of one ingester does not break ingest.
  - Backpressure: when ingest queue depth exceeds threshold, slo-engine-worker evaluates from object-storage backups instead of live ingester (eventual consistency for gate decisions, but gate stays available).
- Owner: ops-sre-reliability + axis-observability
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6, A.8.14; GDPR Art. 32(1)(c) (availability)

**T-D-02 — slo-engine-worker outage halts every promotion**
- Asset: Eligibility evaluator
- Likelihood: M / Impact: H (every µservice's promotion held) / Risk: **H**
- Mitigations:
  - Worker runs HA with at least 2 replicas; lease-based leadership election.
  - Worker is stateless beyond the evaluator window; restart-tolerant.
  - Mimir + Alertmanager monitor the worker's own SLOs (self-observability per PRD OQ#4); breach triggers OnCall page within 5min.
  - **Bootstrap fail-closed**: per 2026-05-17 user pick, during cold-start (< 3 evaluator cycles of clean data), the worker emits `verdict=held` for all (microservice, sha, target_env) tuples. CI lane respects this fail-closed default.
  - Manual override: no retired CLI bypass exists. Override requires an incident PR against `dev`, ops-security pre-approval, reviewer APPROVE, green Jenkins CI, and audit-chain evidence; requires 2-person rule.
- Owner: axis-observability
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.14; GDPR Art. 32(1)(c)

**T-D-03 — Alertmanager flood pages everyone (alert storm)**
- Asset: Grafana OnCall + on-call rotation
- Likelihood: M / Impact: M (operational fatigue → ignored real incidents) / Risk: **M**
- Mitigations:
  - Alertmanager inhibition rules: parent alerts (cluster-wide outage) suppress dependent alerts.
  - Alert grouping: same (cluster, severity) group into single notification.
  - Per-route rate limits in OnCall.
  - On-call playbook: silence flood patterns; root-cause then re-enable.
  - SLO on the alert pipeline itself (alert-to-page latency, signal-to-noise) — meta-monitoring per Google SRE Workbook ch. 6.
- Owner: ops-sre-reliability
- Residual: M
- Frameworks: SOC 2 CC7.2, CC7.3; ISO 27001 A.5.24, A.8.6, A.8.16

**T-D-04 — Grafana UI ddos via dashboard query flood**
- Asset: Grafana frontend
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - Grafana per-user query rate limits.
  - Query-frontend caching; expensive queries deduplicated.
  - WAF rate limit at ingress (per-IP, per-OIDC-subject).
- Owner: ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.6

**T-D-05 — Object-storage bucket exhaustion (cold-tier write quota)**
- Asset: S3-compatible cold-tier
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Per-bucket quota alarm at 70% / 85% / 95% utilization; budget alerts to FinOps.
  - Mimir block compaction policy aggressive on hot-tier; cold-tier sees blocks only after compaction.
  - Multi-bucket sharding by tenant prefix when single-bucket approaches limit.
- Owner: ops-sre-reliability + cloud-secrets
- Residual: L
- Frameworks: SOC 2 CC7.1, CC9.1; ISO 27001 A.8.6, A.8.14

### Elevation of Privilege (E)

**T-E-01 — `oya-ci` reserved tenant abused to write into per-tenant metric streams**
- Asset: Mimir tenant boundary
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Mimir ingester enforces: `oya-ci` tenant may only write metric names matching `oya_promotion_*` and `oya_governance_*`; any other metric name from this tenant is rejected.
  - Recording rule asserts `count by (__name__) (oya:ci_metric_namespace_violation:rate) == 0`.
  - Pen-test: attempt to write `mail_inbox_count` as `oya-ci`; should fail with 403.
- Owner: axis-observability
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.1; ISO 27001 A.5.15, A.8.3, A.8.4

**T-E-02 — Grafana UI privilege escalation via misconfigured role**
- Asset: Grafana RBAC
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Grafana role assignments managed via OpenTofu in `microservices/observability/iac/terraform/grafana-rbac.tf`; not via UI.
  - LEAN check asserts Grafana folder permissions match the declared OpenTofu.
  - Admin role JIT only (OpenBao-issued, ≤4h).
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2; ISO 27001 A.5.15, A.5.18, A.8.2, A.8.3

**T-E-03 — slo-engine-worker SA token leaked → arbitrary verdict writes**
- Asset: Worker ServiceAccount token
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - SA token bound to pod identity; cannot be used outside cluster.
  - Token rotation 24h.
  - Mimir distributor validates the token's SPIFFE identity matches the expected worker SA.
  - Network policy: only worker pods may reach Mimir's write API.
- Owner: ops-security + axis-observability
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.5.17, A.8.5, A.8.7

**T-E-04 — Cedar policy escape via crafted manifest field**
- Asset: Cedar policy evaluation
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cedar v3+ used (no template-based escape vectors known in v3).
  - Cedar fragments fuzzed at CI time (`oya-check-cedar-fragment-coverage` lane).
  - Field input lengths bounded at REST API; oversized inputs rejected before policy evaluation.
- Owner: axis-observability + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC8.1; ISO 27001 A.5.15, A.8.28

**T-E-05 — Operator-level access to Mimir admin API used to delete tenant data**
- Asset: Mimir admin API (compaction, deletion, retention modification)
- Likelihood: L (insider-malicious threat) / Impact: H / Risk: **M**
- Mitigations:
  - Admin API requires JIT elevation via OpenBao (2-person rule for delete operations).
  - Every admin call emits audit-chain record; mass-deletion patterns trigger anomaly alert (`oya:mimir_admin_mass_deletion:rate`).
  - Soft-deletion: deletion marks blocks for retention-override; actual delete scheduled-for-distinct-tracked-work 30 days (recovery window).
- Owner: ops-security + axis-observability
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.15, A.5.27, A.8.4, A.8.16; GDPR Art. 17 (right to erasure), Art. 32 (security)

## LINDDUN Privacy-Threat Catalog

LINDDUN (Linkability / Identifiability / Non-repudiation / Detectability / Disclosure / Unawareness / Non-compliance) covers privacy-specific threats not fully captured by STRIDE.

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | Span attributes + log entries | Multiple data points across services can be linked to a single end-user even when no direct identifier is present. | Span sampler + redactor; trace correlation IDs are tenant-scoped, not user-scoped; user-trace correlation requires explicit `data_class=PII_QUASI` annotation. | M |
| T-L-02 | Identifiability | Hashed customer ID `X-Scope-OrgID` | sha256(tenant_id)[..16] may be re-identifiable via auxiliary data (subscriber list of small tenant base). | Salt the hash with a per-deployment secret; rotate salt every 12mo (audit-chain notes the rotation event). | L |
| T-L-03 | Non-repudiation | Tenant operator OpenSLO authoring | Tenants may deny authorship of SLO changes that lead to undesirable promotion gates. | Signed commits; PR audit log; per-changeset evidence is signed. | L |
| T-L-04 | Detectability | Metric emission timing | Burst of writes correlates with tenant business events (e.g., end-of-month batch). | Reasonable; reflects tenant business reality. Auditor disclosure: this is BEHAVIORAL_TENANT_PRODUCT and is expected. Mitigation is jurisdictional consent (tenant agreed to telemetry collection at onboarding). | M |
| T-L-05 | Disclosure | Auditor read access | An auditor scoped to a single tenant could pivot into adjacent tenants via shared dashboards. | Auditor tokens are tenant-scoped at Grafana folder level; folder isolation tested in pen-test annually. | L |
| T-L-06 | Unawareness | End-user (the tenant's user, not the tenant) | The tenant's end-user may not know their behavior is captured by observability for the tenant's operational benefit. | Tenant's onboarding contract includes data-processing-agreement; tenant-of-tenant disclosure is tenant's responsibility per GDPR Art. 26 (joint controllership). | M |
| T-L-07 | Non-compliance | GDPR Art. 17 right-to-erasure on end-user | End-user requests erasure; their identifiers are in traces / logs across multiple Mimir blocks. | DSR cascade per the `oya-dsr-cascade-runner` skill: identifies all blocks containing the identifier, marks for redaction; Mimir + Loki + Tempo support per-series deletion API. SLA: 30 days from request. | M (best-effort with rotation; some traces may have been deleted by retention before DSR) |

## Mitigations Catalog (cross-reference)

Cross-cuts STRIDE + LINDDUN. Each mitigation appears in at least one threat row above.

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Mimir multi-tenancy enforced (`multitenancy_enabled: true`) | Preventive | axis-observability | `oya-check-mimir-tenancy-enforced` lane |
| Per-tenant Mimir API key issued by OpenBao with bound tenant claim | Preventive | cloud-secrets | OpenBao API audit + Mimir distributor logs |
| OTel SDK PII redactor | Preventive | axis-observability + each workload µservice | Synthetic-PII detector CI lane |
| Span / metric `data_class` annotations | Preventive | each workload µservice | `oya-check-data-class` lane |
| Ed25519 audit-chain seal on every gate decision + ref change | Detective + Non-repudiation | audit-chain | Audit-chain regression tests |
| Per-tenant rate limits + cardinality limits | Preventive (DoS) | axis-observability | Mimir distributor metrics |
| Signed commits on OpenSLO manifests | Preventive (tampering) | ops-security | required-check policy enforces |
| 2-person rule for Mimir admin ops + manual gate overrides | Preventive (insider) | ops-security | OpenBao JIT elevation logs |
| Network policy: worker → Mimir write API only | Preventive | ops-sre-reliability | Kubernetes NetworkPolicy review |
| Differential privacy on cross-tenant aggregations | Preventive | axis-observability | DP analysis published in `policy/dp-analysis.md` |
| Soft-deletion + 30d recovery window for Mimir admin deletes | Detective + Recovery | axis-observability | Mass-deletion anomaly alert |
| DSR cascade runner | Preventive (compliance) | council-privacy | DSR queue dashboard SLO |

## Residual Risk Acceptance

Residual risks above L (low) require explicit acceptance signed by `council-architecture` + `ops-security` + `council-privacy`:

| Risk ID | Residual | Why accepted | Re-review date |
|---|---|---|---|
| T-I-02 (PII leakage in traces) | M | Cannot be fully eliminated without prohibiting trace emission; engineering discipline is the load-bearing control. | Quarterly |
| T-I-06 (secret leak via logs) | M | Human-error baseline; mitigated to acceptable via detection + rotation. | Quarterly |
| T-D-03 (alert storm) | M | Operational discipline gap; SLO on alert pipeline itself is the meta-control. | Quarterly |
| T-L-01 (linkability) | M | Inherent to span-level tracing; mitigated to acceptable via sampling + redaction. | Annually |
| T-L-04 (detectability via timing) | M | Tenant business reality; consent at onboarding covers. | Annually |
| T-L-06 (end-user unawareness) | M | Tenant-of-tenant responsibility; joint-controllership clause. | Annually |
| T-L-07 (right-to-erasure best-effort) | M | Subject to retention windows; DSR cascade is best-effort within Mimir/Loki/Tempo retention. | Annually |

Sign-off (this document is RW until council sign-off captured):

- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`

## Per-Pack Overlay Sections

### pack-kr (Korea)

Compliance frameworks engaged: KR-ISMS-P + KR PIPA + KR 전자문서법.

Additional considerations:
- **KR PIPA Art. 23 (sensitive personal information)**: hashed customer IDs are treated as sensitive when paired with auxiliary data; the salt-rotation mitigation in T-L-02 satisfies Art. 23's "additional technical measures" requirement.
- **KR PIPA Art. 29 (technical safeguards)**: every mitigation marked "T-S-01" through "T-E-05" maps to one of the 12 prescribed safeguards in Art. 29 (access control + encryption + integrity verification + audit log retention ≥ 1 year + intrusion-detection + …). Cross-mapped in `microservices/observability/compliance.md` (Slice B).
- **KR PIPA Art. 23-2 (sensitive data outside-of-KR transfer)**: GDPR-Art-44-style cross-border transfer is gated; KR tenant data MUST stay in KR Mimir cluster. Multi-region story (Slice B `multi-region.md`) enforces residency.
- **KR 전자문서법 Art. 5 (electronic document integrity)**: audit-chain Ed25519 seal satisfies the integrity-preservation requirement for promotion records (which are electronic documents under the law's definition).
- **KR-ISMS-P §2.5 (인적보안)** and **§2.7 (접근통제)**: 2-person rule + JIT elevation map directly.

### pack-us-healthcare (HIPAA-scoped)

Compliance frameworks engaged: HIPAA + state-level (CCPA / CMIA / etc. — covered in `microservices/observability/compliance.md`).

Additional considerations:
- **HIPAA §164.312(a)(1) (access control)**: per-tenant Mimir multi-tenancy + Ed25519 audit-chain satisfies Unique User Identification + Emergency Access + Automatic Logoff + Encryption-and-Decryption.
- **HIPAA §164.312(b) (audit controls)**: audit-chain emission on every PHI-touching operation; retention ≥ 6 years (Mimir cold-tier 2y is INSUFFICIENT for HIPAA — overlay extends to 6y for pack-us-healthcare; cost-budget.md (Slice B) reflects).
- **HIPAA §164.502 (minimum-necessary standard)**: traces redact PII / PHI at emission; only the minimum-necessary trace attributes are persisted. Specific PHI redaction rules per `policy/redaction-phi.md` (pack-us-healthcare overlay; spec'd in Slice D).
- **HIPAA §164.308(a)(4)(ii)(B) (access authorization)**: auditor tokens scoped per T-S-05.
- **Business Associate Agreement (BAA)**: when a Covered Entity tenant uses oyatie, the BAA is per-tenant and lives at `microservices/observability/legal/baa-template.md` (pack-us-healthcare overlay; spec'd in Slice D).

### pack-eu

Compliance frameworks engaged: GDPR + eIDAS + NIS2.

Additional considerations:
- **GDPR Art. 25 (privacy-by-design)**: every mitigation tagged in catalog above maps to a Schrems-II-compatible technical-organizational measure.
- **GDPR Art. 35 (DPIA)**: this threat model + the DPIA at `microservices/observability/dpia.md` (Slice A Task A2) together satisfy the DPIA requirement for high-risk processing of EU tenant data.
- **GDPR Art. 28 (processor agreement)**: oyatie acts as processor for tenant SLO data; per-tenant DPA template at `microservices/observability/legal/dpa-template.md` (Slice D).
- **GDPR Art. 32 (security of processing)**: every "T-*-NN" mitigation contributes to the Art. 32 risk-appropriate security posture.
- **GDPR Art. 44–50 (transfers)**: pack-eu Mimir cluster is EU-resident; cross-region replication to KR / US is forbidden by default; allowed only with tenant SCCs in place. Enforced in `multi-region.md` (Slice B).
- **NIS2 (2022/2555)**: when oyatie crosses NIS2 Annex I/II thresholds, the incident-reporting timelines (24h initial + 72h detailed + 1mo final) become mandatory; `incident-response.md` (Slice B) reflects.
- **eIDAS 910/2014**: Ed25519 audit-chain seals are advanced electronic signatures (AdES); when the rollback affects EU-resident transaction records, the seals satisfy Art. 26 AdES requirements.

### pack-jp

Compliance frameworks engaged: APPI (改正個人情報保護法 2022).

Additional considerations:
- **APPI Art. 17 (purpose of use)**: telemetry purpose declared at tenant-onboarding; restricted to operational use.
- **APPI Art. 21 (notification on cross-border transfer)**: pack-jp Mimir cluster is JP-resident.
- **APPI Art. 23 (joint use)**: tenant-of-tenant data joint-use disclosure required.

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Pack-overlay sections authored in the corresponding `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/observability-overlay.md` files; each overlay follows the same structure (citing the local PII law's articles + the local cybersecurity-framework controls) and maps to this document's threat IDs via the cross-mapping table in `microservices/observability/compliance.md` (Slice B).

## Compliance Cross-Mapping (Globally Enforced)

| Framework | Coverage | Mapping doc |
|---|---|---|
| SOC 2 Type 2 (2017 TSC + 2022 PoF) | CC1.x (control environment) ← council sign-off; CC2.x (communication) ← runbooks; CC3.x (risk assessment) ← this threat model; CC4.x (monitoring) ← Mimir + OnCall; CC5.x (control activities) ← LEAN lanes; CC6.x (logical access) ← OIDC + RBAC + Cedar; CC7.x (system operations) ← runbooks + alerting; CC8.x (change management) ← PR review + ChangeSet; CC9.x (risk mitigation) ← residual-risk acceptance | `microservices/observability/compliance.md` (Slice B) |
| ISO 27001:2022 | Annex A.5–A.8 controls covered as cited inline in each threat row | `microservices/observability/compliance.md` (Slice B) |
| GDPR | Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44 cited inline | `microservices/observability/dpia.md` (Slice A Task A2) + `microservices/observability/compliance.md` (Slice B) |

## Re-review Triggers

This threat model re-reviews on:

- Any change to the trust boundary diagram above (new boundary, removed boundary, modified actor).
- Any Layer-A version upgrade (Mimir / Loki / Tempo / Grafana / Alloy / Pyroscope / Alertmanager / OnCall) where the upstream release notes mention security fixes.
- Any new pack activation (e.g., first pack-us-healthcare tenant onboarding triggers HIPAA-specific deep-dive).
- Annual scheduled review (Q2 each year).
- Post-incident review (any Sev-1 or Sev-2 incident in observability or any µservice it gates).
- Pen-test or audit finding.

## References

- ADR-0028 (Bominal): Audit chain (Merkle + Ed25519); inherited.
- ADR-0056: BNF v4.1.
- ADR-0105: 13-layer enum.
- ADR-0117: Cloud-native infrastructure (data residency).
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- ADR-0140: Cedar policy enforcement.
- `microservices/observability/PRD.md`.
- `microservices/observability/dpia.md` (Slice A Task A2).
- `microservices/observability/compliance.md` (Slice B).
- `microservices/observability/policy/tenant-isolation.md` (Slice A Task A3).
- `microservices/observability/policy/data-residency.md` (Slice A Task A4).
- `/specs/agentic-slo-gated-promotion.json`.
- Microsoft Threat Modeling methodology (STRIDE).
- LINDDUN privacy-threat methodology — Wuyts et al., KU Leuven.
- OWASP Top 10 (2021) + OWASP API Top 10 (2023).
- NIST SP 800-154 (Guide to Data-Centric System Threat Modeling).
- Google SRE Workbook ch. 5 (alerting on SLOs) + ch. 6 (eliminating toil).
- Grafana Mimir security model — `grafana.com/docs/mimir/latest/manage/secure/`.
- ICO DPIA template — `ico.org.uk/for-organisations/uk-gdpr-guidance-and-resources/accountability-and-governance/data-protection-impact-assessments-dpias`.
- CNIL DPIA methodology — `cnil.fr/en/PIA`.
