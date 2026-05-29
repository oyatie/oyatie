---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: intelligence
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: axis-intelligence + ops-security
deciders: council-architecture, ops-security, axis-intelligence, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP LLM Top 10 (2025) + NIST AI RMF (AI 100-1) + EU AI Act Annex III + MITRE ATLAS
related_adrs: [ADR-0255, ADR-0255-amendment-library-first, ADR-0263, ADR-0296, ADR-0145, ADR-0250]
related_specs: [/specs/intelligence-two-layer-substrate.json]
review_cadence: quarterly + on every BC promotion + on every provider-adapter add
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.5.34, A.8.2, A.8.3, A.8.5, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.25, A.8.26, A.8.27, A.8.28"
  - "ISO/IEC 42001:2023 (AI Management System): all clauses"
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35"
  - "EU AI Act 2024/1689 Arts. 9 (risk-management), 10 (data governance), 12 (record-keeping), 13 (transparency), 14 (human oversight), 15 (accuracy, robustness, cybersecurity), 16 (provider obligations), 27 (FRIA), Annex III (high-risk systems)"
  - "OWASP LLM Top 10 (2025): LLM01..LLM10"
  - "MITRE ATLAS (AI/ML Attack Tactics Landscape)"
  - "NIST AI RMF 1.0 — Govern, Map, Measure, Manage functions"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 15/17/22-2/23/28/29", "KR-AIDA 2026 (AI Industrial Promotion Act drafting in progress)", "KR-FSC AI Risk Management Guidelines (financial sector)"]
  pack-us-healthcare: ["HIPAA §164.308/.312/.314/.316", "FDA Software-as-a-Medical-Device (SaMD) framework for AI/ML", "ONC Information-Blocking Rule (TEFCA + USCDI v3 carve-out for AI outputs)"]
  pack-eu: ["EU AI Act Annex III categories 1, 4, 5, 6, 7", "GDPR Art. 22 (automated decision-making)", "EDPB Guidelines 1/2024 on AI"]
  pack-cn: ["CN PIPL Arts. 24/55", "CN Generative AI Service Provisions (2023)", "CN Algorithm Recommendation Provisions (2022)"]
  pack-uk: ["UK AI Regulation White Paper 2023 + ICO AI Guidance"]
doc_status: published
---

# Threat Model: intelligence µservice

## Purpose

Identify, classify, and mitigate threats to the `intelligence` µservice's confidentiality,
integrity, availability, privacy, and AI-safety posture. As the two-layer AI Substrate per
ADR-0255, `intelligence` is the chokepoint between every product surface and every model
provider; compromise here cascades to every product and every tenant. This document is the
canonical security artifact reviewed by SOC 2 Type 2 examiners, ISO 27001 + ISO 42001 auditors,
GDPR DPAs, the EU AI Office (post-Aug 2026 enforcement of AI Act Annex III), the Korean PIPC,
and equivalent supervisory bodies in every active pack.

## Scope

### In-scope

All components introduced by ADR-0255 (two-layer AI Substrate + Consumer Brand Surface) and
ADR-0263 (audit-tap) and ADR-0296 (sidecar credential-handle), deployed in tenant-cell-eligible
Kubernetes clusters per ADR-0254 (Cloud Hypervisor + Kata isolation):

| Layer-A — substrate | Layer-B — consumer brand UX |
|---|---|
| `model-routing` BC | `brand-ux-surface` BC |
| `providers` BC (16 adapters) | (renders only — no Layer-B threats unique to surface) |
| `guardrails` BC | |
| `eval` BC | |
| `attribution` BC | |
| `credential-resolver` BC | |
| `audit-tap` BC | |

### Out-of-scope

- Threats to the underlying provider APIs themselves (Anthropic, OpenAI, Google, …) — covered
  by each provider's SOC 2 / ISO 27001 / DPA.
- Threats to OpenBao secret-manager itself — owned by the `cloud-secrets` µservice threat model;
  inherited here as upstream.
- Threats to the Kubernetes cluster / container runtime — owned by `cloud-k8s` µservice threat
  model.
- Threats to the audit-chain seal stream itself — owned by `audit-chain` µservice.
- Threats to GitHub Actions runners — owned by `governance` µservice CI substrate.
- Threats to embeddings / fine-tuning storage — owned by `intelligence-embeddings` and
  `intelligence-fine-tuning` µservices (separate scope per ADR-0255 §D).

## Trust Boundaries

```text
┌─ Internet ─────────────────────────────────────────────────────────────────┐
│                                                                            │
│   Consumer / developer / tenant browser    Tenant backend (server-side)    │
│         │                                          │                       │
│         │ (HTTPS + HTTP/3 + QUIC; OIDC)            │ (in-process SDK call) │
│         ▼                                          ▼                       │
│  ┌─ Edge gateway (Envoy/Istio + WAF) ──────────────────────────────────┐   │
│  │  TLS termination + PQ-hybrid (Kyber768 + X25519) per ADR-0253       │   │
│  │  WAF (OWASP CRS + LLM-injection ruleset)                            │   │
│  │  DDoS protection + abuse-defence Cedar gate                         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │                                             │
└──────────────────────────────│─────────────────────────────────────────────┘
                               ▼
┌─ Tenant cell (Cloud Hypervisor + Kata pods per ADR-0254) ──────────────────┐
│                                                                            │
│  Trust boundary 1: External → Cluster ingress                              │
│                                                                            │
│  ┌─ intelligence-rest / -grpc handler ──────────────┐                      │
│  │  OIDC tenant-scoped reads                        │                      │
│  │  audience-tag enforcement                        │                      │
│  └──────────────────────────────────────────────────┘                      │
│                              │                                             │
│  Trust boundary 2: tenant-scope envelope (Cedar evaluator)                 │
│                              │                                             │
│  ┌─ model-routing-kernel ───┐ ┌─ guardrails-usecase ─┐                     │
│  │  dispatch-authorization  │ │  refusal-baseline    │                     │
│  └──────────────────────────┘ └──────────────────────┘                     │
│                              │                                             │
│  Trust boundary 3: credential-resolver → OpenBao sidecar (unix socket)     │
│                              │                                             │
│  ┌─ openbao-sidecar (per pod) ─────────────────────────────────────────┐   │
│  │  Issues CredentialHandle bound to provider+ttl+tenant+audience     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │                                             │
│  Trust boundary 4: provider-adapter → external provider API                │
│                              │                                             │
│  ┌─ providers-adapter-<vendor> (TLS 1.3 + PQ-hybrid; per-tenant key) ──┐   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │                                             │
│  Trust boundary 5: audit-tap-worker → audit-chain seal stream              │
│                              │                                             │
│  ┌─ audit-tap-worker (Ed25519 signing under SPIFFE) ──────────────────┐    │
│  └────────────────────────────────────────────────────────────────────┘    │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

Five trust boundaries:
1. **External → Cluster ingress** (TLS, WAF, DDoS, audience-tag enforcement).
2. **Tenant-scope envelope** (Cedar evaluator; tenant_id binding).
3. **Credential-resolver → OpenBao sidecar** (Unix domain socket; short-lived handles).
4. **Provider-adapter → external provider API** (TLS 1.3 + PQ-hybrid; per-tenant credential).
5. **Audit-tap-worker → audit-chain seal stream** (Ed25519 + SPIFFE).

## Assets & Data Classification

Per Bominal ADR-0028 (audit-chain + data-class taxonomy):

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| Dispatch prompt content | `PII_IDENTIFYING` + `PII_QUASI_IDENTIFIER` + occasionally `PHI` (pack-us-healthcare via BAA) | High | 30d hot for audit; never persisted beyond audit-tap | audit-chain (sealed); transient in provider response stream only |
| Dispatch output content | as above | High | as above | audit-chain (sealed) |
| Provider credentials (`SecretReference`) | `SECRET` | Critical | rotation 30d via OpenBao; never in process memory per ADR-0296 | OpenBao via cloud-secrets µservice |
| `CredentialHandle` (per-call short-lived) | `SECRET` (short-lived) | Critical | TTL ≤ 15 min; invalidated post-call | OpenBao sidecar memory only |
| Dispatch envelope (`audience_tag`, `tenant_id`, modality) | `BEHAVIORAL_TENANT_PRODUCT` | Medium | retained with audit-tap record | audit-chain |
| Refusal decision (`RefusalDecision`) | `BEHAVIORAL_TENANT_PRODUCT` + `AUDIT` | High | 1y default; pack-overlay extends (EU AI Act 6 mo Art. 12; HIPAA 6y) | audit-chain |
| Routing decision (`RoutingDecision`) | `BEHAVIORAL_TENANT_PRODUCT` + `AUDIT` | High | 1y | audit-chain |
| Eval canonicalen-set | `INTERNAL_ONLY` (curated test data; should never carry production PII) | Low | indefinite (eval corpus) | git + S3 (versioned) |
| Eval-online results | `BEHAVIORAL_TENANT_PRODUCT` (aggregated) | Medium | 90d hot + 2y cold | Mimir + audit-chain |
| Per-call cost record | `BEHAVIORAL_TENANT_PRODUCT` + `FINANCIAL` | Medium | per pack legal min; KR 5y | finops µservice (read-side projection) |
| Citation graph (attribution) | `BEHAVIORAL_TENANT_PRODUCT` | Medium | 1y (with dispatch record) | audit-chain |
| Audit-tap records | `AUDIT` (Ed25519 sealed) | High | per pack legal min; HIPAA 6y; EU AI Act Art. 12 6 mo | audit-chain (sealed) |
| Per-tenant model routing preferences | `BEHAVIORAL_TENANT_PRODUCT` | Medium | tenant config lifetime | tenancy µservice |
| Brand-ux-surface telemetry (sparkle render counts, refusal banner views) | `BEHAVIORAL_TENANT_PRODUCT` | Low | 30d | observability (Mimir) |

## Actors

| Actor | Trust level | Authentication | Capability |
|---|---|---|---|
| Consumer end-user (B2C) | Untrusted external | OIDC + (optional) WebAuthn | Issue dispatch via brand-ux-surface; cost-float covered by platform-default |
| Developer (Forge user) | Untrusted external | OIDC + MFA | Issue dispatch via SDK; tenant provider-credential BYOK active (ADR-0255 §D-4) |
| Tenant backend (machine) | Semi-trusted internal | mTLS + tenant-bound service identity (SPIFFE) | Issue dispatch via in-process SDK |
| Workload µservice (in same trust domain) | Semi-trusted internal | mTLS + SPIFFE | Issue dispatch in-process |
| Foundry agent (planning / review / exec / doubt) | Trusted internal | OpenBao service token + audience `internal-foundry` | Issue dispatch with elevated rate limit + platform-default credentials |
| `audit-tap-worker` (long-lived) | Trusted internal | SPIFFE identity | Emit Ed25519-signed audit records into audit-chain |
| `eval-worker` (long-lived) | Trusted internal | SPIFFE identity | Read canonicalen set; emit eval-result metrics |
| Council operators (human) | Trusted internal | OIDC + MFA + JIT elevation via OpenBao | Read all audit-tap records (auditor-scope Cedar fragment); never write |
| External auditor (SOC 2 / ISO 27001 / 42001 / EU AI Office / FDA / HIPAA / PIPC) | Read-only on time-boxed window | OIDC + MFA + JIT short-lived token via OpenBao | Read-only on audit-tap + per-pack policy; cannot pivot to tenant data |
| Attacker — opportunistic | Untrusted | none | Scans + low-skill exploitation; assume always present |
| Attacker — targeted | Untrusted | none | Sophisticated; supply-chain awareness; prompt-injection skilled |
| Attacker — model-poisoning supply chain | Untrusted | varies | Compromises a provider-adapter upstream package or a vLLM weight; assume present |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure refusal policy; mitigated by per-pack-floor invariant + multispectrum review |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case threat actor; mitigated by Cedar least-privilege + audit-chain + 2-person-rule on policy changes |

## STRIDE + OWASP LLM Top 10 + EU AI Act threat catalog

Each threat carries: ID; category; asset; description; likelihood (L/M/H); impact (L/M/H);
risk score (likelihood × impact); mitigations (concrete); owner; residual risk;
frameworks satisfied.

### Spoofing (S)

**T-S-01 — Tenant-A submits dispatch claiming `tenant_id` of Tenant-B**
- Asset: tenant-scope envelope
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - OIDC bearer carries `tenant_id` claim signed by OpenBao.
  - Cedar `dispatch-authorization` gate validates `envelope.tenant_id == principal.tenant_id`;
    mismatch returns 401 + audit-emit `tenant_spoofing_attempt`.
  - `audience_tag` is independently validated against the principal's allowed-audience set.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2, CC6.6; ISO 27001 A.5.15, A.5.17, A.8.2; GDPR Art. 32(1)(a)(b);
  EU AI Act Art. 15 (cybersecurity); OWASP LLM03.

**T-S-02 — Attacker impersonates Foundry agent to obtain platform-default credentials**
- Asset: `internal-foundry` audience tag
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - `internal-foundry` audience is restricted to SPIFFE identities matching
    `spiffe://oyatie/foundry/<role>`; pre-issued at cluster bootstrap; mTLS-only.
  - `dispatch-authorization.cedar` forbids `audience == internal-foundry` for any non-`foundry-*`
    SPIFFE identity.
  - 2-person-rule for editing this Cedar fragment.
- Owner: ops-security + axis-foundry
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.3; EU AI Act Art. 14 (human oversight).

**T-S-03 — Attacker forges audit-tap record (claims a dispatch never happened)**
- Asset: audit-chain seal stream
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Each audit-tap record is Ed25519-signed by `spiffe://oyatie/intelligence/audit-tap-worker`.
  - audit-chain µservice validates the signature against the SPIFFE identity registry.
  - Records form a Merkle chain; gaps are detectable as audit-chain seal-stream discontinuity.
- Owner: axis-intelligence + axis-audit-chain
- Residual: L
- Frameworks: SOC 2 CC7.2; ISO 27001 A.5.34; EU AI Act Art. 12 (record-keeping).

### Tampering (T)

**T-T-01 — Prompt-injection: indirect (untrusted content in retrieval context)**
- Asset: dispatch prompt content
- Likelihood: **H** / Impact: H / Risk: **H** (#1 LLM threat per OWASP LLM01)
- Mitigations:
  - **Caller-side RAG discipline**: the `intelligence` substrate does NOT own retrieval; the caller
    builds context; intelligence dispatches with that context. We pass through caller-tagged
    `untrusted_content` markers (per dispatch envelope schema). guardrails-adapter performs
    role-separation: untrusted markers cause provider system-prompt to instruct the model not to
    follow embedded instructions; post-call classifier checks for jailbreak success.
  - Provider-side input filter (provider's own moderation).
  - Output post-call classifier checks if response contains policy violations triggered by
    injection.
  - Runbook `runbooks/prompt-injection-detected.md`.
- Owner: axis-intelligence + council-privacy
- Residual: M (industry-wide unsolved problem; mitigated, not eliminated)
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.23; EU AI Act Art. 15 (robustness, cybersecurity);
  OWASP LLM01.

**T-T-02 — Prompt-injection: direct (untrusted user input)**
- Asset: dispatch prompt content
- Likelihood: H / Impact: M / Risk: **H**
- Mitigations:
  - System-prompt isolation: user input is delimited (provider-specific delimiters).
  - guardrails post-call classifier.
  - Refusal-baseline Cedar gate catches the obvious cases (CSAM / violence / Annex III) regardless
    of injection success.
- Owner: axis-intelligence
- Residual: M
- Frameworks: as above; OWASP LLM01.

**T-T-03 — Output tampering between provider response and caller**
- Asset: dispatch output content + audit-tap record
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Server-to-server TLS 1.3 + PQ-hybrid per ADR-0253.
  - Audit-tap record signs both prompt-hash and output-hash; tampering breaks the hash.
  - mTLS between intelligence pods and provider adapters.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.6; ISO 27001 A.8.20, A.8.21; GDPR Art. 32(1)(b).

**T-T-04 — Model-poisoning supply-chain (compromised vLLM weight or provider-adapter package)**
- Asset: provider-adapter binary + model weight files
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - SLSA L3 build attestation for every adapter crate.
  - Weight-file SHA pinning for self-hosted vLLM / SGLang / TensorRT-LLM via `models.lock`.
  - Provider-adapter crate dependencies under `cargo deny` allow-list + supply-chain scanning.
- Owner: ops-security + axis-intelligence
- Residual: L
- Frameworks: SLSA L3; ISO 27001 A.5.23, A.8.30; OWASP LLM05; EU AI Act Art. 15.

### Repudiation (R)

**T-R-01 — Tenant denies issuing a dispatch that they did**
- Asset: audit-tap record
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - Audit-tap record signed by intelligence + bound to caller OIDC subject + timestamp.
  - Tenant must dispute via DSR runner; audit-chain seal cryptographically proves the dispatch.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC7.2; ISO 27001 A.5.34; EU AI Act Art. 12.

**T-R-02 — oyatie operator denies refusing a dispatch (false-positive refusal claim)**
- Asset: refusal-decision audit record
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Every refusal is audit-tap-emitted with the refusal reason + the gate that fired.
  - Two-person-rule on Cedar fragment changes; PR review + per-pack legal sign-off for
    `refusal-baseline.cedar` mutations.
- Owner: council-privacy + axis-intelligence
- Residual: L
- Frameworks: as above; EU AI Act Art. 13 (transparency).

### Information disclosure (I)

**T-I-01 — Cross-tenant context leak in provider response**
- Asset: dispatch output content
- Likelihood: L / Impact: H / Risk: **M-H**
- Mitigations:
  - Single-tenant dispatch envelope; no batching across tenants.
  - Provider adapters never reuse HTTP connections across tenants for stateful providers.
  - Per-tenant credential isolation (provider-credential BYOK ⇒ per-tenant credential per ADR-0255 §D-4; even platform-default uses
    per-tenant logical separation in the credential-resolver).
- Owner: axis-intelligence + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.12; GDPR Art. 32(1)(b); KR PIPA Art. 29.

**T-I-02 — Provider credential leak via logging / tracing**
- Asset: `SecretReference` resolution path
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - ADR-0296 sidecar pattern: credentials never enter intelligence process memory; opaque handle
    only.
  - Structured-log redactor strips any field containing `secret_reference` / `credential_handle`.
  - Per-provider adapter test set includes "log-scrape for credential leak" assertion.
- Owner: ops-security + axis-intelligence
- Residual: L
- Frameworks: SOC 2 CC6.6; ISO 27001 A.5.34, A.8.12; GDPR Art. 32(1)(a); OWASP LLM02 (Sensitive
  Information Disclosure).

**T-I-03 — Model-output PII leak (the model generates PII for someone other than the data subject)**
- Asset: dispatch output content
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - Post-call PII classifier (separate from refusal-baseline; flags PII patterns in output).
  - Per-pack PII-detection thresholds (EU + KR stricter than US default).
  - Audit-tap records PII-detection results.
- Owner: council-privacy + axis-intelligence
- Residual: M
- Frameworks: GDPR Art. 5(1)(c) data minimisation; KR PIPA Art. 28; EU AI Act Art. 13; OWASP LLM02.

**T-I-04 — Telemetry leak (prompt/output content lands in observability beyond audit-tap)**
- Asset: prompt + output content
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Observability allow-list: no prompt/output fields in logs/traces/metrics beyond the audit-tap
    record (which lives only in audit-chain).
  - CI lane `oya-governance-no-prompt-in-telemetry` greps for `prompt`/`response_text` in
    observability emit paths.
- Owner: axis-intelligence + axis-observability
- Residual: L
- Frameworks: SOC 2 CC6.6; ISO 27001 A.8.12; GDPR Art. 25.

**T-I-05 — Auditor-pivot attack (external auditor uses scope to read outside engagement window)**
- Asset: audit-tap records
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations: identical to observability auditor-scope.cedar pattern (see `policy/auditor-scope.cedar`).
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.17; GDPR Art. 32; OWASP A01.

### Denial of service (D)

**T-D-01 — Provider rate-limit saturation cascades into dispatch-API outage**
- Asset: dispatch API availability
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - Per-provider QPS budget enforced client-side (token bucket); see `capacity-model.md`.
  - Per-tenant quota (so one tenant cannot starve others).
  - Secondary-provider fallback routing per `provider-routing.cedar`.
  - Runbooks `runbooks/provider-outage-*.md` + `provider-rate-limit-saturation.md`.
- Owner: axis-intelligence
- Residual: L
- Frameworks: SOC 2 A1.1; ISO 27001 A.8.27, A.8.28; EU AI Act Art. 15 (robustness).

**T-D-02 — Refusal-amplification DoS (attacker submits many borderline prompts to trigger refusal-cost)**
- Asset: guardrails CPU + provider call budget
- Likelihood: M / Impact: L / Risk: **L-M**
- Mitigations:
  - Refusal-classifier is pre-call and cheap; provider call only happens after pre-call admit.
  - Per-tenant rate limit on refusal-rate spikes triggers abuse-defence Cedar gate.
- Owner: axis-intelligence + ops-security
- Residual: L
- Frameworks: SOC 2 A1.1; ISO 27001 A.8.27.

### Elevation of privilege (E)

**T-E-01 — Caller elevates audience tag from `consumer` to `internal-foundry` to bypass cost controls**
- Asset: `audience_tag` field in dispatch envelope
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Audience tag is validated against principal's allowed-audience set (signed by OpenBao);
    elevation attempt rejected + audit-emitted.
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.3; ISO 27001 A.5.15; GDPR Art. 32(1)(b).

**T-E-02 — Caller bypasses refusal-baseline via Cedar fragment mutation**
- Asset: `refusal-baseline.cedar` content
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - 2-person-rule + per-pack legal sign-off + multispectrum-review v2.4.0 on every change.
  - CI lane validates the fragment against a canonicalen-set of categories that must remain in refusal.
- Owner: council-privacy + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.3; ISO 27001 A.5.15, A.5.23; EU AI Act Annex III.

### EU AI Act Annex III specific

**T-AIACT-01 — Annex III category 1 (biometric identification): dispatch attempts face-match**
- Asset: dispatch envelope (modality=image + prompt pattern)
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: `eu-ai-act-high-risk.cedar` refuses dispatch when modality=image AND prompt
  matches face-match patterns AND tenant_pack=eu AND no Annex III FRIA on file.
- Owner: council-privacy + ops-legal
- Residual: L
- Frameworks: EU AI Act Art. 27 (FRIA); Annex III cat. 1.

**T-AIACT-02 — Annex III category 4 (employment): dispatch evaluates candidate without FRIA**
- Asset: dispatch envelope (prompt pattern signals candidate evaluation)
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: same gate as above for Annex III cat. 4.
- Owner: council-privacy
- Residual: L
- Frameworks: EU AI Act Annex III cat. 4.

**T-AIACT-03 — Annex III category 5 (essential services credit scoring): dispatch credit decision**
- Asset: dispatch envelope
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations: same gate Annex III cat. 5.
- Owner: council-privacy + ops-legal
- Residual: L
- Frameworks: EU AI Act Annex III cat. 5.

**T-AIACT-04 — Failure to record (Art. 12)**: dispatch happens but no audit-tap record emitted
- Asset: audit-tap emission path
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - SLO `audit-emission-success` ≥ 99.99 %; burn-rate alert pages on regression.
  - Dispatch usecase refuses to return without an audit-tap commit (atomic with response stream).
- Owner: axis-intelligence + ops-security
- Residual: L
- Frameworks: EU AI Act Art. 12.

**T-AIACT-05 — Failure to disclose (Art. 13)**: consumer brand surface omits AI-disclosure
- Asset: `RefusalBanner` + `CostFloorDisclosure` + `SparkleIcon` rendering
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations: brand-ux-surface SDK enforces sparkle-icon presence on every AI-rendered surface;
  axe-pa11y-runner E2E checks for presence at WCAG-AAA contrast.
- Owner: axis-intelligence + design-platform
- Residual: L
- Frameworks: EU AI Act Art. 13; WCAG 2.2 AAA.

## Risk register summary

| Risk class | High | Medium | Low |
|---|---|---|---|
| Spoofing | T-S-01 | T-S-02, T-S-03 | — |
| Tampering | T-T-01, T-T-02 | T-T-04 | T-T-03 |
| Repudiation | — | T-R-01 | T-R-02 |
| Information disclosure | T-I-01 | T-I-02, T-I-03 | T-I-04, T-I-05 |
| DoS | — | T-D-01 | T-D-02 |
| Elevation | — | T-E-02 | T-E-01 |
| EU AI Act | — | T-AIACT-01..05 | — |

High risks (T-S-01, T-T-01, T-T-02) carry hard-refusal Cedar gates + multispectrum review of
every mutation + quarterly chaos drill.

## Per-pack overlay

### pack-kr (KR PIPA + ISMS-P + KR-AIDA preparation)

- KR PIPA Art. 23 (sensitive data) refusal floor: prompts requesting sensitive data (health
  diagnosis, race, religion) require explicit consent grant referenced in dispatch envelope.
- KR-FSC AI Risk Management Guidelines (financial sector): pack-kr financial tenants receive
  additional refusal-baseline overlay.

### pack-us-healthcare (HIPAA + FDA SaMD)

- Dispatch with audience `consumer` is refused when the prompt is health-diagnostic in scope
  unless the calling tenant has a Covered-Entity-BAA in place and is using a BAA-signed provider.
- FDA SaMD scope: outputs that propose medical-device-like diagnosis are refused unless tenant has
  filed a 510(k) and registered the device.

### pack-eu (EU AI Act + GDPR)

- Full Annex III refusal layer (Categories 1–7).
- Art. 12 record-keeping verified by SLO `audit-emission-success` ≥ 99.99 %.
- Art. 13 transparency verified by brand-ux-surface SparkleIcon presence test.
- Art. 14 human-oversight: every Annex III refusal carries a human-review escalation queue.

### pack-cn (CN PIPL + Generative AI Provisions 2023)

- Dispatch from pack-cn tenants is routed only to Alibaba Qwen / Tencent Hunyuan / Baidu ERNIE
  providers.
- Outbound dispatch from CN tenants to US/EU providers is refused.

### pack-au / pack-jp / pack-sg / pack-in / pack-br / pack-ae / pack-ksa

Each pack's overlay at `regional-packs/<pack>/intelligence-overlay.md` carries the local AI
regulator's refusal floor and dispatch routing constraint.

## Verification

- `cargo run -p oya-dev-cli -- gate validate cedar-fragment-coverage --microservice intelligence` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate eu-ai-act-annex-iii-refusal --microservice intelligence` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate audit-tap-emit --microservice intelligence` — exit 0.
- Quarterly chaos drill: induce a prompt-injection success + cross-tenant context leak attempt;
  verify rejection + alerting.
- Annual pen-test against dispatch + credential-resolver path: scheduled Q4 of each calendar year.

## References

- ADR-0255, ADR-0255 amendment, ADR-0263, ADR-0296, ADR-0145, ADR-0250.
- `microservices/intelligence/dpia.md`.
- `microservices/intelligence/policy/*.cedar`.
- `docs/standards/documentation-rigor.md`.
- OWASP LLM Top 10 (2025).
- MITRE ATLAS.
- NIST AI RMF 1.0.
- EU AI Act 2024/1689.
- ISO/IEC 42001:2023.
