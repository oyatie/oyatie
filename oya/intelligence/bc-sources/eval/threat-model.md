---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: foundry-eval
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry + ops-security
deciders: council-architecture, ops-security, axis-foundry, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + OWASP LLM Top 10 (2024) + NIST SP 800-154 + NIST AI RMF (AI 100-1)
related_adrs: [ADR-0024, ADR-0026, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/per-microservice-flat-layout.json]
review_cadence: quarterly + on every Layer-A or Layer-B architecture change + on every new eval-set cohort
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.8.2, A.8.3, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.25, A.8.26, A.8.27, A.8.28"
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35"
  - "EU AI Act Arts. 9 (risk management), 10 (data governance), 15 (accuracy + robustness), 17 (logging)"
suggested_frameworks_by_pack:
  pack-kr: ["KR-ISMS-P §2.1-2.12", "KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2", "KR 신용정보법 (when finance capability evaluated)"]
  pack-us-healthcare: ["HIPAA §164.308 / §164.310 / §164.312 / §164.314 / §164.316", "HHS de-identification guidance (safe-harbor + expert-determination)"]
  pack-eu: ["GDPR Arts. 25 + 32 + 35 + 44-50", "EU AI Act Arts. 9 / 10 / 15 / 17 (high-risk AI)", "NIS2 2022/2555"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24"]
  pack-sg: ["PDPA Part III + IV", "MAS-TRM v2021"]
  pack-au: ["Privacy Act 1988 APP 1-13", "APRA-CPS 234"]
  pack-in: ["DPDPA 2023 §6-10", "RBI Master Direction on Outsourcing of IT Services 2023"]
  pack-br: ["LGPD Arts. 6, 7, 11, 14, 18, 33, 46, 48", "BACEN Res. 4.893/2021"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021 Arts. 5/6/9/15"]
  pack-ksa: ["PDPL Royal Decree M/19/2021 Arts. 4-9", "SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Threat Model: foundry-eval µservice

## Purpose

Identify, classify, and mitigate threats to the foundry-eval µservice's confidentiality, integrity, availability, and privacy posture. The eval substrate is the gate authority for every capability's publish, every routing-preference change, every model-upgrade decision, and every in-house cutover. A compromise here cascades to model-decisioning quality across the entire oyatie surface. This document is the canonical security artifact reviewed by SOC 2 Type 2 examiners, ISO 27001 auditors, GDPR DPAs, and EU AI Act Conformity Assessment Bodies at first-tenant onboarding.

## Scope

### In-scope

All components introduced by ADR-0024 (eval harness + replay) and ADR-0026 (in-house cutover gate) for the foundry-eval µservice, deployed in a **dedicated foundry-eval Kubernetes cluster** with a GPU-eligible node pool (decision aligned with hyperscaler practice — AWS SageMaker Evaluations, Anthropic internal eval clusters, Patronus/Braintrust hosted infra all isolate eval substrate from runtime):

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| Postgres (eval-set metadata; via CloudNativePG) | `oya-foundry-eval-eval-set-registry-*` |
| ClickHouse (parity analytics, week-partitioned MergeTree) | `oya-foundry-eval-eval-runner-*` (11 crates) |
| SeaweedFS / S3-compatible (baseline-outputs + replay traces, per-subject-keyed envelope) | `oya-foundry-eval-parity-analyzer-*` |
| KMS (per-tenant KEK + per-subject DEK wrap) | `oya-foundry-eval-replay-engine-*` |
| Sigstore Cosign + Rekor (eval-set signature verification) | `oya-foundry-eval-baseline-output-store-*` |
| Kubernetes Job controller + GPU node pool (case dispatch; gVisor / Kata sandbox) | Eval-set manifests at `microservices/intelligence-eval/eval-sets/<capability>/v<n>.evalset.yaml` |
| Argo Workflows (nightly + on-demand orchestration) | Per-component release pointer Git refs |

### Out-of-scope

- Threats to the underlying Kubernetes cluster, container runtime, or hyperscaler IaaS layer — owned by `cloud-k8s` µservice threat model.
- Threats to the provider model APIs themselves (OpenAI / Anthropic / Google / xAI / internal) — provider-side; mitigated indirectly via per-eval-set signed manifests + Cosign verify.
- Threats to `foundry-runtime` invocation path — owned by foundry-runtime threat model; we trust the runtime's autonomy-ceiling + sandbox gates per ADR-0022 + ADR-0023.
- Threats to `foundry-evidence` (audit chain) — owned by foundry-evidence threat model; we emit, do not own the seal substrate.
- Threats to OpenBao secret-manager itself — owned by `cloud-secrets` threat model.

## Trust Boundaries

```text
┌─ Internet ─────────────────────────────────────────────────────────────────┐
│                                                                            │
│   Capability owner (engineer)        Tenant operator (read-only verdicts)  │
│         │                                  │                               │
│         │ (HTTPS + OIDC + MFA)             │ (HTTPS + OIDC; scope-bound)   │
│         ▼                                  ▼                               │
│  ┌─ Public ingress (Envoy / Istio gateway) ─────────────────────────────┐  │
│  │  - TLS termination                                                   │  │
│  │  - WAF (rate-limit + OWASP CRS + OWASP LLM Top 10 patterns)          │  │
│  │  - DDOS protection                                                   │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                              │                                             │
└──────────────────────────────│─────────────────────────────────────────────┘
                               ▼
┌─ Dedicated foundry-eval cluster ────────────────────────────────────────────┐
│                                                                             │
│  TB-1: External → Cluster ingress                                           │
│                                                                             │
│  ┌─ eval-runner-rest ─────────────┐    ┌─ parity-analyzer-rest ─────┐       │
│  │  OIDC tenant-scoped reads      │    │  per-capability scoped     │       │
│  │  + capability-owner write      │    │                            │       │
│  └────────────────────────────────┘    └────────────────────────────┘       │
│             │                                                               │
│  TB-2: Per-tenant data isolation (Cedar + KEK-per-tenant boundary)          │
│             │                                                               │
│  ┌─ Postgres (eval-set metadata; row-level security per tenant) ────────┐   │
│  │  - Per-tenant query filtering enforced via RLS                       │   │
│  │  - Schema migrations PR-gated                                        │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│  ┌─ ClickHouse (parity analytics; tenant-id partition + RBAC) ──────────┐   │
│  │  - DP-noise on cross-tenant aggregates                               │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│  ┌─ S3 (baseline-outputs + replay traces; per-subject-keyed envelope) ────┐   │
│  │  - DEK-per-subject wrapped by per-tenant KEK; KEK in KMS             │   │
│  │  - Shred = delete DEK; record remains encrypted-and-unreplayable     │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  TB-3: foundry-eval cluster → foundry-runtime invocation (eval-time)        │
│             │                                                               │
│  ┌─ GPU runner pool (Kubernetes Jobs; gVisor sandbox) ──────────────────┐   │
│  │  - Per-case ephemeral pod; no shared CUDA context                    │   │
│  │  - Per-case egress allowlist (provider model APIs only)              │   │
│  │  - Per-case OTel API key issued by OpenBao; rotated per-pod          │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  TB-4: foundry-eval → foundry-providers (route resolution)                  │
│             │ (mTLS; SPIFFE identity)                                       │
│             │                                                               │
│  TB-5: foundry-eval → foundry-evidence (audit emission)                     │
│             │ (Ed25519 signed events)                                       │
│             │                                                               │
│  TB-6: CI runner → publish-gate read API                                    │
│             │ (reserved CI scope; read-only)                                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

Six trust boundaries:
1. **External → Cluster ingress** (TLS, WAF, DDoS, OWASP LLM patterns).
2. **Per-tenant data isolation** (Cedar + KEK-per-tenant + Postgres RLS + ClickHouse partition).
3. **eval cluster → foundry-runtime invocation** (eval-time only; gVisor sandbox per case).
4. **foundry-eval → foundry-providers** (mTLS + SPIFFE; route resolution).
5. **foundry-eval → foundry-evidence** (Ed25519 signed events).
6. **CI runner → publish-gate read API** (read-only; reserved scope).

## Assets & Data Classification

Per Bominal ADR-0028 (audit-chain + data-class taxonomy) and the `oya-check-data-class` LEAN lane.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| Eval-set manifests (per-capability YAML) | `INTERNAL_ONLY` (text); references may carry `PHI_SYNTHETIC` | Low text; Medium for synthetic-PHI | append-only git history + Postgres metadata | repo + Postgres |
| Baseline outputs (per-case expected outcome) | `BEHAVIORAL_TENANT_PRODUCT` + may carry tenant-derived `PII_QUASI_IDENTIFIER` | High | per-subject-keyed encrypted; 24mo non-shred retention; pack-us-healthcare 6y | S3 + KMS |
| Eval-run results (per-case actual outcome) | `BEHAVIORAL_TENANT_PRODUCT` + transient `PII_QUASI_IDENTIFIER` | High | 90d hot + 24mo cold; pack-us-healthcare 6y | S3 + ClickHouse |
| Parity reports | `BEHAVIORAL_TENANT_PRODUCT` + `AUDIT` | High | week-partitioned ClickHouse; 24mo | ClickHouse |
| Replay traces (sampled production invocations) | `BEHAVIORAL_TENANT_PRODUCT` + occasionally `PII_IDENTIFYING` (where source µservice emitted) + per-pack `PHI` / `SENSITIVE_PIPA_ART23` | Critical | 24mo non-shred + per-subject DEK shred on DSR | S3 + KMS |
| Per-subject DEKs | `SECRET` + `AUDIT` | Critical | rotation 90d + shred on DSR | KMS |
| Per-tenant KEKs | `SECRET` | Critical | rotation 365d + HSM-backed where available | KMS |
| Provider API keys (eval-time invocation) | `SECRET` | Critical | OpenBao 30d rotation | OpenBao |
| ClickHouse credentials | `SECRET` | High | OpenBao 90d rotation | OpenBao |
| Cosign verification keys (Rekor public) | `INTERNAL_ONLY` | Low | rotation per Sigstore policy | Rekor + repo |
| Cosign signing keys (eval-set authoring) | `SECRET` | Critical | OpenBao 90d rotation + HSM-backed | OpenBao + capability-owner workstations (hardware token) |
| EvalRun + ParityVerdict + ReplayDivergence events | `AUDIT` | High | append-only; immutable | foundry-evidence + audit-chain |
| InHouseCutoverEligible verdict | `AUDIT` + `BEHAVIORAL_TENANT_PRODUCT` | High | immutable | foundry-evidence |
| EU AI Act §15 + §17 evidence payloads | `AUDIT` | High | immutable; tied to per-EvalRun | foundry-evidence |

## Actors

| Actor | Trust level | Authentication | Capability |
|---|---|---|---|
| Capability owner (human; engineering team) | Semi-trusted internal | OIDC + MFA + hardware-token (for Cosign sign) | Author eval-sets; sign; trigger ad-hoc runs |
| Tenant operator (human) | Untrusted external | OIDC + MFA via Application Shell | Read own-tenant verdict history; refuse upgrades on stale eval |
| `foundry-runtime` (machine) | Semi-trusted internal | mTLS + SPIFFE | Request publish-gate verdict; receive `EvalRunCompleted` |
| `foundry-providers` (machine) | Semi-trusted internal | mTLS + SPIFFE | Receive `ParityVerdictEmitted` + `InHouseCutoverEligible` |
| `foundry-evidence` (machine) | Trusted internal (audit-chain) | mTLS + SPIFFE + Ed25519 verify | Sink for all audit emissions |
| `tenancy` (machine) | Semi-trusted internal | mTLS + SPIFFE | Sends `EraseSubjectRequested`; consumes `EvalSubjectShred` |
| oyatie CI runner (GitHub Actions) | Semi-trusted internal | `WORKFLOW_PAT` + reserved publish-gate read scope | Read publish-gate verdict; refuse capability publish PR on miss |
| `eval-runner-worker` (long-lived) | Trusted internal | OpenBao-issued service-account token | Dispatch cases; emit verdicts; write S3 + ClickHouse |
| `replay-engine-worker` (long-lived) | Trusted internal | OpenBao SA token + per-subject-DEK unwrap permission | Sample traces; replay; emit divergence |
| Council-architecture / ops-security operators (human) | Trusted internal | OIDC + MFA + JIT elevation via OpenBao | Admin Grafana; RW on policy fragments via PR review |
| External auditor (SOC 2 / ISO 27001 / EU AI Act CAB) | Read-only external; time-boxed | OIDC + MFA + JIT short-lived | Read-only audit-chain + EU AI Act §15+§17 evidence export |
| Attacker — opportunistic | Untrusted | none | Scans + low-skill exploitation |
| Attacker — targeted (model-decision sabotage) | Untrusted | none | Sophisticated; attempts to bias eval verdicts; assume present for prod-tier |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure eval-set; mitigated by PR review + LEAN gates |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case for confidentiality + integrity; mitigated via least-privilege + audit-chain + 2-person rule |

## STRIDE Threat Catalog

Each threat carries: ID; category; asset; description; likelihood (L/M/H); impact (L/M/H); risk; mitigations; owner; residual; frameworks.

### Spoofing (S)

**T-S-01 — Capability owner forges Cosign signature on eval-set manifest**
- Asset: Eval-set manifests
- Likelihood: L / Impact: H (false publish-gate pass; bad capability ships) / Risk: **M**
- Mitigations:
  - Cosign signatures verified at registry-read time via Sigstore Rekor public-log inclusion proof; tampered manifests fail at usecase-layer load.
  - Signing keys hardware-token-bound (YubiKey) per capability-owner workstation; private key not exfiltrable to a non-attested device.
  - Per-capability CODEOWNERS in repo PRs; signed-commit branch protection on `eval-sets/**`.
  - LEAN check `oya-check-cosign-rekor-inclusion` (NEW) validates inclusion proofs on every eval-set load.
- Owner: ops-security + axis-foundry
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6, CC8.1; ISO 27001 A.5.15, A.5.17, A.8.5, A.8.7; GDPR Art. 32(1)(b)(c); EU AI Act Art. 15 (accuracy via signed evidence)

**T-S-02 — Attacker impersonates eval-runner-worker SA to emit false verdicts**
- Asset: EvalRunCompleted emission
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Verdict emissions signed Ed25519 by worker SPIFFE identity; foundry-evidence rejects mismatched signatures.
  - Mimir gauge `oya_foundry_eval_verdict_signature_invalid_total` paged on > 0.
  - SA token bound to in-cluster pod identity; egress NetworkPolicy restricts worker → foundry-evidence + S3 + ClickHouse + Postgres only.
- Owner: ops-security + axis-foundry
- Residual: L
- Frameworks: SOC 2 CC6.1, CC7.1, CC7.4; ISO 27001 A.5.15, A.8.3, A.8.7; GDPR Art. 32(1)(b); pack-kr KR-ISMS-P §2.5

**T-S-03 — Attacker impersonates `foundry-providers` to receive a fraudulent InHouseCutoverEligible verdict**
- Asset: Cutover verdict consumption
- Likelihood: L / Impact: H (could cause inappropriate cutover) / Risk: **M**
- Mitigations:
  - mTLS + SPIFFE on the consumption path; foundry-providers must present a known SPIFFE ID.
  - Verdict carries Ed25519 signature with replay-counter; foundry-providers verifies + rejects out-of-order.
  - Cutover is reversible: revert preference change triggers a `ReverseCutoverExecuted` event and audit emission within 60s.
- Owner: axis-foundry + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.2; ISO 27001 A.5.17, A.8.7; EU AI Act Art. 15

**T-S-04 — Attacker impersonates DSR cascade to shred non-target subject DEKs**
- Asset: Per-subject DEK store
- Likelihood: L / Impact: H (data loss; cohort continuity breach) / Risk: **M**
- Mitigations:
  - `EraseSubjectRequested` events must be signed by tenancy SPIFFE + carry tenant-OIDC operator co-signature.
  - 2-person rule for shred operations on > 1 subject in a 24h window.
  - Soft-shred: DEK marked-for-deletion 7d before actual KMS delete (recovery window).
  - Shred-attempt-log retained 5y immutable.
- Owner: council-privacy + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.4, CC8.1; ISO 27001 A.5.27, A.5.31, A.8.4; GDPR Art. 17 + Art. 32; pack-us-healthcare HIPAA §164.310(d)(2)(i)

**T-S-05 — Attacker forges GitHub Actions OIDC token to call publish-gate read API**
- Asset: Publish-gate read scope
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - GitHub OIDC issuer pinned; audience claim validated against expected pattern `oyatie-ci`.
  - Token TTL ≤ 15 min.
  - Per-repo scope binding; cross-repo OIDC rejected.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15; GDPR Art. 32

### Tampering (T)

**T-T-01 — Eval-set manifest tampering after signing (signed, then field mutated)**
- Asset: Eval-set manifests
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: per T-S-01 (Cosign verify on read; Rekor inclusion-proof check).
- Owner: ops-security + axis-foundry
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.31, A.8.32, A.8.33; GDPR Art. 32(1)(b); EU AI Act Art. 15

**T-T-02 — Baseline-output tampering at S3 (object body modified post-PUT)**
- Asset: Baseline-output bytes
- Likelihood: L / Impact: H (false regression-detection signal) / Risk: **M**
- Mitigations:
  - SSE-KMS + bucket Object Lock in Compliance mode for baseline-output bucket.
  - Per-object Cosign signature stored alongside object; verify-on-read.
  - Block validator job verifies SHAs monthly; mismatch quarantines + paged.
- Owner: cloud-secrets + axis-foundry
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.8.11, A.8.12, A.8.24, A.8.25; GDPR Art. 32(1)(a)(b); pack-eu GDPR Art. 44

**T-T-03 — Replay-trace S3 object tampering**
- Asset: Replay-trace bytes
- Likelihood: L / Impact: H (replay returns wrong "old" output; falsified divergence) / Risk: **M**
- Mitigations:
  - Same SSE-KMS + Object Lock posture as baseline-outputs.
  - Per-object hash chain (each trace references prior trace by hash within the same (capability, day) bucket).
  - Replay-engine verifies chain on read; broken chain → quarantine + paged.
- Owner: cloud-secrets + axis-foundry
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.8.11, A.8.12; GDPR Art. 32(1)(b); EU AI Act Art. 17 (logging integrity)

**T-T-04 — ClickHouse parity-analytics row tampering**
- Asset: parity_analytics MergeTree
- Likelihood: L / Impact: H (false cohort delta → bad routing decision) / Risk: **M**
- Mitigations:
  - Per-row HMAC computed at INSERT; INSERT path enforces.
  - Read path verifies HMAC; mismatch → row dropped + paged.
  - ClickHouse `parity_analytics` partition is read-only after week-seal; INSERT denied to sealed partitions at the cluster ACL layer.
- Owner: axis-foundry
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.8.32, A.8.33; GDPR Art. 32(1)(b); EU AI Act Art. 15

**T-T-05 — Provider response cache poisoning (eval-time invocation)**
- Asset: Cached provider responses (eval-time)
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Eval-time provider calls bypass cache by default (`Cache-Control: no-store` on the case-dispatcher path).
  - Where caching is allowed (deterministic-seed cases), per-response hash signed; verify-on-read.
- Owner: axis-foundry
- Residual: L
- Frameworks: SOC 2 CC6.6; ISO 27001 A.8.7; EU AI Act Art. 15

### Repudiation (R)

**T-R-01 — Eval-runner-worker emitted verdict but denies authorship**
- Asset: EvalRun audit chain
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Every EvalRunCompleted carries `actor=<worker-SPIFFE>` + Ed25519 audit-chain seal per Bominal ADR-0028.
  - Per-changeset evidence at `microservices/intelligence-eval/evidence/multispectrum/<change_id>-<unix_ts>.json` is git-committed.
- Owner: axis-foundry + foundry-evidence
- Residual: L
- Frameworks: SOC 2 CC4.1, CC8.1; ISO 27001 A.5.27, A.5.28, A.8.15; GDPR Art. 5(2), Art. 30; EU AI Act Art. 17

**T-R-02 — Capability owner denies authorship of an eval-set commit**
- Asset: Eval-set git commits
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - Signed commits required on `microservices/intelligence-eval/eval-sets/**`.
  - Cosign signature on manifest = non-repudiation surface independent of git.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.8.34; GDPR Art. 5(2)

**T-R-03 — Rollback (in-house cutover reverse) without traceable trigger**
- Asset: Cutover-reverse audit chain
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Every cutover-reverse emits `ReverseCutoverExecuted{capability, from_model, to_model, reason, parity_snapshot, executed_at}`; reason is structured enum + required.
  - ClickHouse snapshot at reverse-time persisted with retention extension.
- Owner: axis-foundry + ops-security
- Residual: L
- Frameworks: SOC 2 CC7.4, CC8.1; ISO 27001 A.5.26, A.5.27; GDPR Art. 33; EU AI Act Art. 17

### Information Disclosure (I)

**T-I-01 — Cross-tenant leak via ClickHouse parity-analytics misconfiguration**
- Asset: ClickHouse per-tenant data
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - ClickHouse `parity_analytics` partition includes `tenant_id` as primary-key prefix; RBAC enforces per-tenant row visibility.
  - Cross-tenant aggregates use DP noise (ε ≤ 1 per published aggregate) per `policy/dp-analysis.md`.
  - LEAN check `oya-check-clickhouse-rbac-conformance` (NEW) validates RBAC config at deploy time.
  - Penetration test annually + on every ClickHouse version upgrade.
- Owner: ops-security + axis-foundry
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.5.18, A.8.2, A.8.3, A.8.12; GDPR Art. 5(1)(f), Art. 25, Art. 32; pack-kr KR PIPA Art. 23; pack-us-healthcare HIPAA §164.312(a)(1)

**T-I-02 — Replay trace contains PII / PHI never redacted at source**
- Asset: Tempo-sourced replay traces
- Likelihood: H (source µservice may have emitted unredacted) / Impact: H / Risk: **H**
- Mitigations:
  - Replay-engine refuses to ingest traces lacking `data_class` annotation; raw traces quarantined and dead-lettered.
  - Replay-time secondary redactor strips known PII patterns; redactor-miss rate SLO with monthly drift review.
  - Per-subject-DEK envelope provides DSR-shred surface even when source µservice failed to redact.
  - Quarterly synthetic-PII detector run against replay store; misses → ops-security investigation.
- Owner: axis-foundry + council-privacy + each workload µservice owner
- Residual: M (engineering-discipline gap baseline)
- Frameworks: SOC 2 CC6.7; ISO 27001 A.8.11, A.8.12, A.8.32; GDPR Art. 5(1)(c), Art. 25, Art. 32; pack-kr KR PIPA Art. 3; pack-us-healthcare HIPAA §164.512(e), §164.514

**T-I-03 — Eval-set baseline outputs leak proprietary tenant prompts**
- Asset: Baseline outputs (per-case expected)
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - Per-tenant baseline outputs encrypted with tenant KEK; reads enforced at Cedar policy layer.
  - LEAN check `oya-check-baseline-output-tenant-isolation` (NEW) validates KEK-per-tenant boundary at deploy.
  - Public parity-matrix dashboard publishes only DP-noised cross-tenant deltas; per-tenant baselines never on public surface.
- Owner: axis-foundry + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.5, A.8.12; GDPR Art. 25, Art. 32

**T-I-04 — Eval-run logs reveal which provider failed (competitive intel)**
- Asset: EvalRunCompleted public dashboards
- Likelihood: M / Impact: L (competitive but not regulated) / Risk: **L-M**
- Mitigations:
  - Tenant-facing public dashboard masks provider identity per default; "Provider A" / "Provider B" pseudonyms.
  - Operator-only dashboard reveals provider identity behind RBAC.
- Owner: axis-foundry
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.5

**T-I-05 — Auditor JIT read scoped to "all capabilities" exposes per-tenant evidence**
- Asset: External auditor scope
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Auditor JIT tokens scoped per (tenant, capability subset, time window); TTL ≤ 4h non-renewable.
  - Every auditor read audit-emitted (audit-chain of audits).
  - mTLS client cert pinned to auditor firm gateway during engagement.
- Owner: ops-security + council-privacy
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2, CC8.1; ISO 27001 A.5.15, A.5.17, A.5.18, A.8.2, A.8.3; GDPR Art. 28; pack-us-healthcare HIPAA §164.308(a)(4)(ii)(B)

**T-I-06 — Secret leak via eval-run logs (provider API key in error trace)**
- Asset: OpenBao-managed secrets
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Per-case ephemeral pods; provider API key materialised at pod-start, never logged.
  - Secret-scanner CI lane scans every commit + log emission.
  - Eval-runner SDK strips known-secret patterns via `Secret<T>` Debug shim.
  - Rotation: 30d API keys; 90d signing; 365d KEK.
  - Secret-leak runbook: detect → rotate → forensic trace → education.
- Owner: ops-security + cloud-secrets
- Residual: M (human-error baseline)
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.17, A.8.7, A.8.12; GDPR Art. 32

**T-I-07 — Prompt-injection in eval-case input leaks provider-side info to attacker-controlled output**
- Asset: Eval-run pipeline (OWASP LLM01)
- Likelihood: M (adversarial cohort by design exercises this) / Impact: M / Risk: **M**
- Mitigations:
  - Adversarial cohort is the test; pass criterion = capability refuses or contains the injection per ADR-0024.
  - Per-case egress allowlist (provider model API only); pod cannot exfiltrate to attacker domain.
  - Sandbox isolation per gVisor / Kata.
- Owner: axis-foundry + ops-security
- Residual: L
- Frameworks: OWASP LLM Top 10 (LLM01: Prompt Injection); EU AI Act Art. 15 (robustness)

### Denial of Service (D)

**T-D-01 — GPU runner pool exhaustion (adversarial flood of eval-set submissions)**
- Asset: GPU pool
- Likelihood: M / Impact: H (publish-gate stalls for everyone) / Risk: **H**
- Mitigations:
  - Per-capability-owner rate limit on eval-set submission (10/hour by default).
  - GPU pool autoscaler caps at cluster-budget; excess queued not provisioned.
  - Reserved priority class for publish-gate runs > nightly > on-demand; nightly delayed when publish-gate backlogged.
  - Backpressure: when pool saturated, sdk returns `429 with retry-after`.
- Owner: ops-sre-reliability + axis-foundry
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.6, A.8.14; GDPR Art. 32(1)(c)

**T-D-02 — ClickHouse query flood from nightly orchestrator**
- Asset: ClickHouse parity-analytics
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - ClickHouse read query budget per-source (worker has 100 QPS budget).
  - Query plan validator refuses unbounded scans.
  - Backpressure: nightly orchestrator throttles when ClickHouse latency p99 > 200ms.
- Owner: axis-foundry
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.30, A.8.6

**T-D-03 — Replay-engine worker outage halts all model upgrades**
- Asset: Replay engine
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - 2+ HA replicas; lease-based leadership.
  - Worker stateless beyond sampling window; restart-tolerant.
  - **Bootstrap fail-closed**: during cold-start, replay verdict = `divergence_unknown_held`; model-upgrade gate respects.
  - Manual override: `oya admin model upgrade --skip-replay --reason "<rfc>"` with 2-person rule + audit.
- Owner: axis-foundry
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.14; GDPR Art. 32(1)(c)

**T-D-04 — Postgres eval-set metadata outage**
- Asset: Postgres
- Likelihood: L / Impact: H (publish-gate stalls) / Risk: **M**
- Mitigations:
  - HA primary + 2 sync replicas via Patroni.
  - Cached read path: eval-runner-rest caches eval-set manifests for 5min on Postgres unreachable.
  - Backup: PITR with 5min RPO; daily logical backup to S3.
- Owner: ops-sre-reliability
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.13, A.8.14; GDPR Art. 32(1)(c)

**T-D-05 — KMS rate-limit hit on per-subject DEK ops (shred storm)**
- Asset: KMS
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - DSR cascade processes shred queue at bounded rate (10 shreds/sec default; configurable).
  - KMS pre-provisioned for expected shred volume; alarms at 70% quota.
- Owner: ops-security + axis-foundry
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.6, A.8.14

### Elevation of Privilege (E)

**T-E-01 — Eval-case pod escapes gVisor sandbox to host**
- Asset: GPU runner pod
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - gVisor + Kata where supported (Kata for high-risk capabilities); seccomp + AppArmor + read-only root fs.
  - Per-pod NetworkPolicy restricts egress to provider model API allowlist.
  - Pod-security-admission `restricted` profile.
  - CIS Kubernetes Benchmark continuous compliance.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6, CC8.1; ISO 27001 A.5.15, A.8.2, A.8.3, A.8.7

**T-E-02 — Eval-runner-worker SA token captures cross-cluster privileges via misconfigured ClusterRole**
- Asset: Worker SA
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - SA bound to Role (namespace-scoped); ClusterRole forbidden.
  - LEAN check validates SA permissions are minimum-necessary.
  - Token rotation 24h.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.2

**T-E-03 — Capability owner gains eval-runner-worker permissions via shared role**
- Asset: RBAC
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Capability-owner role = read-only on verdicts + write on `eval-sets/<owned-capabilities>/**`; never write on worker config.
  - 2-person rule for any RBAC change touching worker permissions.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.5.18

**T-E-04 — Cedar policy escape via crafted eval-set manifest field**
- Asset: Cedar policy evaluation
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cedar v4+ (no template-based escape vectors known); fragments fuzzed at CI.
  - Input lengths bounded at REST API + at YAML parse.
- Owner: axis-foundry + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC8.1; ISO 27001 A.5.15, A.8.28

**T-E-05 — Insider deletes eval-run history from ClickHouse to hide a regression**
- Asset: parity_analytics MergeTree
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Sealed partitions read-only; DELETE denied at cluster ACL.
  - 2-person rule for any partition seal-break + JIT elevation via OpenBao.
  - Audit-chain emission on every admin-API call.
  - Soft-delete + 30d recovery window.
- Owner: ops-security + axis-foundry
- Residual: L
- Frameworks: SOC 2 CC6.6, CC8.1; ISO 27001 A.5.15, A.5.27, A.8.4, A.8.16; GDPR Art. 17, Art. 32; EU AI Act Art. 17

### AI-Specific (per NIST AI RMF + OWASP LLM Top 10)

**T-A-01 — Eval-set itself contaminated by training-data overlap (data leakage)**
- Asset: Eval-set baseline cases
- Likelihood: M / Impact: M (artificial pass rate) / Risk: **M**
- Mitigations:
  - Per-capability eval-set carries a `contamination_check_run_at` field + `contamination_score` (per provider's data-cutoff vs case-creation timestamp).
  - LEAN check `oya-check-eval-set-contamination` flags suspect cases.
  - Adversarial cohort + replay cohort are designed post-hoc to be out-of-distribution.
- Owner: axis-foundry
- Residual: M
- Frameworks: NIST AI RMF Govern-1.2; EU AI Act Art. 10 (data-governance)

**T-A-02 — LLM-as-judge bias (judge model trained on same data as one provider being evaluated)**
- Asset: HumanJudged + LLM-judged cohorts
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - Per-quarter judge rotation across two competing top providers + one in-house variant.
  - Per-quarter consistency check: inter-judge κ ≥ 0.7 (Cohen's kappa).
  - Judge identity carried in EvalCaseResult emission for auditability.
- Owner: axis-foundry
- Residual: M
- Frameworks: NIST AI RMF Measure-2.10; EU AI Act Art. 15

**T-A-03 — Adversarial cohort itself adversarially weakened over time (regression-test for the gate)**
- Asset: Adversarial cohort cases
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Quarterly adversarial-cohort refresh: 20% case rotation in / out.
  - Per-quarter audit by external red-team (where contracted) or rotating internal red-team.
  - LEAN check `oya-check-adversarial-cohort-freshness`.
- Owner: ops-security + axis-foundry
- Residual: L
- Frameworks: NIST AI RMF Measure-2.7; EU AI Act Art. 15 (robustness)

## LINDDUN Privacy-Threat Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | Replay traces with span attrs | Multiple cases can be linked to one end-user even without direct identifier | Per-subject-DEK envelope; trace correlation IDs tenant-scoped; user-link requires explicit `data_class=PII_QUASI` annotation | M |
| T-L-02 | Identifiability | Hashed tenant-id in eval-runs | `tenant_id` hash may be re-identifiable via auxiliary data | Salted hash; salt rotation 12mo; audit-chain notes rotation | L |
| T-L-03 | Non-repudiation | Capability owner authoring | Owner may deny authorship of an eval-set leading to a bad publish-gate pass | Signed commits + Cosign + audit-chain | L |
| T-L-04 | Detectability | Eval-run timing | Eval-run cadence correlates with capability publish cadence (operational reality) | Tenant contract discloses operational telemetry | M |
| T-L-05 | Disclosure | Auditor scope | Cross-tenant pivot via shared dashboards | Per-tenant Cedar policy + Grafana folder isolation | L |
| T-L-06 | Unawareness | End-user of tenant capability | End-user may not know their behavior captured for eval | Tenant joint-controllership disclosure | M |
| T-L-07 | Non-compliance | GDPR Art. 17 + KR PIPA Art. 36 | Per-subject DEK shred required within SLA | DSR cascade runner; 30d SLA; per-subject-DEK shred surface | M (best-effort within retention window) |

## Mitigations Catalog (cross-reference)

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Cosign + Rekor inclusion-proof verification | Preventive | ops-security + axis-foundry | `oya-check-cosign-rekor-inclusion` lane |
| Per-tenant KEK + per-subject DEK envelope | Preventive | cloud-secrets + axis-foundry | KMS audit + key-lifecycle dashboard |
| gVisor / Kata sandbox per case pod | Preventive | ops-security | CIS Kubernetes Benchmark scan + pod-security-admission |
| Ed25519 audit-chain seal on every emission | Detective + Non-repudiation | foundry-evidence | Audit-chain regression tests |
| ClickHouse RBAC + DP-noise on cross-tenant aggregates | Preventive | axis-foundry + ops-security | `oya-check-clickhouse-rbac-conformance` lane + DP analysis publication |
| Per-case ephemeral pod + egress allowlist | Preventive | ops-security | NetworkPolicy review |
| Replay-engine fail-closed during cold-start | Preventive | axis-foundry | E2E bootstrap test |
| 2-person rule for shred / admin operations | Preventive | ops-security | OpenBao JIT logs |
| DSR cascade runner + 30d SLA | Preventive (compliance) | council-privacy | DSR dashboard SLO |
| Adversarial cohort freshness (quarterly rotation) | Preventive | ops-security + axis-foundry | `oya-check-adversarial-cohort-freshness` lane |
| Judge model rotation + κ ≥ 0.7 consistency | Preventive (AI fairness) | axis-foundry | per-quarter consistency report |

## Residual Risk Acceptance

Residual risks above L require explicit acceptance by council-architecture + ops-security + council-privacy:

| Risk ID | Residual | Why accepted | Re-review |
|---|---|---|---|
| T-I-02 (PII in replay traces) | M | Source-µservice discipline gap baseline; mitigated via secondary redactor + per-subject-DEK envelope | Quarterly |
| T-I-06 (secret leak via logs) | M | Human-error baseline; mitigated via scanner + rotation | Quarterly |
| T-A-01 (eval-set contamination) | M | Industry-wide concern; mitigated via cohort design + rotation | Quarterly |
| T-A-02 (judge bias) | M | Industry-wide concern; mitigated via rotation + κ check | Quarterly |
| T-L-01 (linkability of replay) | M | Inherent to trace-based replay; mitigated via DEK envelope | Annually |
| T-L-04 (detectability via timing) | M | Operational reality | Annually |
| T-L-06 (end-user unawareness) | M | Joint-controllership responsibility | Annually |
| T-L-07 (right-to-erasure best-effort) | M | Retention-window-bounded; DSR cascade best-effort | Annually |

Sign-off:

- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`

## Per-Pack Overlay Sections

### pack-kr

- KR PIPA Art. 23 (sensitive personal info): hashed tenant-id + replay trace per-subject envelope satisfy Art. 23 technical measures.
- KR PIPA Art. 36 (right-to-erasure): DSR cascade SLA 30d aligns with PIPA's "without delay" interpretation per PIPC guidance.
- KR-ISMS-P §2.5 (인적보안) + §2.7 (접근통제): 2-person rule + JIT elevation map directly.
- KR 전자문서법 Art. 5: audit-chain Ed25519 seal satisfies integrity-preservation for eval-run records.

### pack-us-healthcare (HIPAA-scoped)

- HIPAA §164.312(a)(1) (access control): per-tenant KEK + per-subject DEK envelope satisfy Unique-User-Identification + Encryption + Automatic-Logoff.
- HIPAA §164.312(b) (audit controls): audit-chain emission; retention extended to 6y per HIPAA §164.316(b)(2) (cost-budget reflects).
- HIPAA §164.502 (minimum-necessary): eval-set baseline inputs use synthetic-PHI fixtures only; PHI never enters as live data; policy `policy/synthetic-phi-only.md`.
- HIPAA §164.308(a)(4)(ii)(B) (access authorization): auditor JIT tokens scoped per T-I-05.
- BAA: per-tenant; `legal/baa-template.md` overlay.

### pack-eu

- GDPR Art. 25 (privacy-by-design): every mitigation maps to Schrems-II-compatible TOM.
- GDPR Art. 35 (DPIA): this threat model + `dpia.md` satisfy.
- GDPR Art. 28 (processor): per-tenant DPA; `legal/dpa-template.md`.
- GDPR Art. 32 (security of processing): every mitigation contributes.
- GDPR Art. 44-50 (transfers): pack-eu eval-data EU-resident; cross-region replication forbidden by default.
- **EU AI Act Art. 9 (risk management)**: this threat model + risk register satisfy.
- **EU AI Act Art. 10 (data governance)**: eval-set + baseline-output authoring policy + contamination check satisfy.
- **EU AI Act Art. 15 (accuracy + robustness + cybersecurity)**: every EvalRun emission carries §15 evidence schema; adversarial cohort + replay are the §15 robustness evidence.
- **EU AI Act Art. 17 (logging)**: per-eval-run audit-chain emission is the §17 log surface; retention 24mo (extended 6y for high-risk per Art. 17(2)).
- NIS2 2022/2555: when oyatie crosses thresholds, 24h/72h/1mo incident-reporting timelines apply per `incident-response.md`.
- eIDAS 910/2014: Ed25519 audit-chain seals are AdES; satisfies Art. 26 when EU-resident records.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Pack-overlay sections at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/foundry-eval-overlay.md` (when activated); each follows the same structure citing the local PII law + cybersecurity framework + AI-governance framework (e.g., Singapore Model AI Governance Framework, India AI Mission, Brazil ANPD AI guidance).

## Compliance Cross-Mapping (Globally Enforced)

| Framework | Coverage | Mapping doc |
|---|---|---|
| SOC 2 Type 2 (2017 TSC + 2022 PoF) | CC1.x-CC9.x covered as cited inline | `microservices/intelligence-eval/compliance.md` |
| ISO 27001:2022 | Annex A.5-A.8 controls cited inline | `microservices/intelligence-eval/compliance.md` |
| GDPR | Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44 cited inline | `microservices/intelligence-eval/dpia.md` + `microservices/intelligence-eval/compliance.md` |
| EU AI Act (high-risk obligations) | Arts. 9, 10, 15, 17 cited inline | `microservices/intelligence-eval/compliance.md` |
| NIST AI RMF (AI 100-1) | Govern + Map + Measure + Manage functions mapped per cohort | `microservices/intelligence-eval/compliance.md` |
| OWASP LLM Top 10 (2024) | LLM01-LLM10 covered per adversarial cohort + per-case sandbox | `microservices/intelligence-eval/compliance.md` |

## Re-review Triggers

- Any change to trust-boundary diagram (new boundary, removed boundary, modified actor).
- Any Layer-A version upgrade (Postgres / ClickHouse / Cosign / KMS / Kubernetes) with security fix.
- Any new pack activation.
- Annual scheduled review (Q2).
- Post-incident review (Sev-1 / Sev-2).
- Pen-test or audit finding.
- New adversarial cohort pattern published (Anthropic responsible-scaling updates; Apollo Research; UK AISI Inspect).

## References

- ADR-0024 (Foundry eval harness + replay).
- ADR-0026 (In-house AI model substrate roadmap).
- ADR-0028 (Bominal): Audit chain (Merkle + Ed25519).
- ADR-0056 (BNF v4.1).
- ADR-0105 (13-layer enum).
- ADR-0117 (Cloud-native infrastructure; data residency).
- ADR-0139 (Agentic SLO-gated promotion).
- ADR-0131 (Per-microservice flat layout).
- ADR-0132 (Product-platform-and-bundle dissolution).
- ADR-0133 (Industry best-practice conformance program).
- ADR-0140 (Cedar policy enforcement).
- `microservices/intelligence-eval/PRD.md`.
- `microservices/intelligence-eval/dpia.md`.
- `microservices/intelligence-eval/compliance.md`.
- `microservices/intelligence-eval/policy/tenant-isolation.md`.
- `microservices/intelligence-eval/policy/data-residency.md`.
- `/specs/per-microservice-flat-layout.json`.
- Microsoft Threat Modeling methodology (STRIDE).
- LINDDUN privacy-threat methodology — Wuyts et al., KU Leuven.
- OWASP Top 10 (2021) + OWASP API Top 10 (2023) + OWASP LLM Top 10 (2024).
- NIST SP 800-154 (Data-Centric System Threat Modeling).
- NIST AI 100-1 (AI Risk Management Framework).
- Anthropic Responsible Scaling Policy.
- Apollo Research evaluation framework.
- UK AISI Inspect AI framework.
- EU AI Act (Regulation 2024/1689).
