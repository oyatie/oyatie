---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: foundry-guardrails
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry-guardrails + ops-security
deciders: council-architecture, ops-security, axis-foundry-guardrails, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + OWASP LLM Top 10 (2025) + MITRE ATLAS + NIST SP 800-154 + NIST AI RMF 1.0
related_adrs: [ADR-0022, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0130, ADR-0131, ADR-0140]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/agent-operating-contract.json]
review_cadence: quarterly + on every classifier-model rollout + on every Cedar bundle change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC6.7, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.8.2, A.8.3, A.8.5, A.8.7, A.8.11, A.8.12, A.8.16, A.8.23, A.8.25, A.8.26, A.8.27, A.8.28"
  - "GDPR Arts. 5, 6, 9, 22, 25, 28, 30, 32, 35"
  - "EU AI Act Arts. 9 (risk-management), 10 (data + data-governance), 11 (technical-documentation), 12 (record-keeping), 13 (transparency), 14 (human-oversight), 15 (accuracy + robustness + cybersecurity)"
suggested_frameworks_by_pack:
  pack-kr: ["KR-ISMS-P §2.5/§2.7", "KR PIPA Arts. 15/17/23/23-2/29/29-2"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308/§164.312/§164.502/§164.504(e) (BAA)"]
  pack-eu: ["GDPR Arts. 9, 22 (automated-decision protections), 25, 32, 35", "EU AI Act high-risk classifier requirements"]
  pack-jp: ["APPI Arts. 17/21/27"]
  pack-sg: ["PDPA Part III + MAS Notice 644"]
  pack-au: ["Privacy Act 1988 APP 1-13; APRA-CPS 234"]
  pack-in: ["DPDPA 2023 §§6-10"]
  pack-br: ["LGPD Arts. 6/7/11/14/18/33/46"]
  pack-ae: ["UAE PDPL FDL 45/2021 Arts. 5/6/9/15"]
  pack-ksa: ["KSA PDPL RD M/19/2021 Arts. 4-9; SAMA Cybersecurity"]
doc_status: published
---

# Threat Model: foundry-guardrails µservice

## Purpose

Identify, classify, and mitigate threats to the foundry-guardrails confidentiality, integrity, availability, safety, and privacy posture. foundry-guardrails is the single chokepoint between agent traffic and provider traffic; a compromise here cascades to every tenant agent invocation and bypasses every product's safety floor. This document is the canonical security artifact reviewed by SOC 2 / ISO 27001 / HIPAA / GDPR / EU AI Act / KR PIPA examiners at first-tenant onboarding.

## Scope

### In-scope

All components introduced by ADR-0131 (Foundry split) for the foundry-guardrails µservice, deployed in the dedicated foundry Kubernetes cluster (matching foundry-runtime / foundry-providers / foundry-supervisor / foundry-evidence siblings):

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| ONNX-runtime classifier-model-serving (Helm chart) | `oya-foundry-guardrails-prompt-classifier-*` (10 crates) |
| Cedar v4 engine (in-process + sidecar) | `oya-foundry-guardrails-output-validator-*` (9 crates) |
| Postgres HA (rule store + Cedar fragment registry + mutation log) | `oya-foundry-guardrails-autonomy-tier-gate-*` (9 crates) |
| Cosign (classifier-model artifact signing) | `oya-foundry-guardrails-content-safety-rule-engine-*` (9 crates) |
| OpenBao (secret bindings, Cosign keys, LLM-judge provider tokens) | `oya-foundry-guardrails-jailbreak-detector-*` (10 crates) |
| Object storage (Cosign-signed classifier artifacts; per-pack S3) | `oya-foundry-guardrails-ai-slop-detector-*` (9 crates) |

### Out-of-scope

- Threats to the underlying Kubernetes cluster, container runtime, hyperscaler IaaS — owned by `cloud-k8s` µservice threat-model.
- Threats to `foundry-providers` itself (LLM-provider tokens, provider compromise) — that µservice owns its own threat-model.
- Threats to `foundry-runtime` (the caller) — that µservice owns its own threat-model; this document inherits.
- Threats to OpenBao itself — owned by `cloud-secrets`.
- Threats to upstream classifier-model training data (oyatie consumes pre-trained + fine-tuned models; supply-chain threats covered separately in `compliance.md`).

## Trust Boundaries

```text
┌─ Foundry cluster ──────────────────────────────────────────────────────────┐
│                                                                            │
│   foundry-runtime (sole in-cluster caller; mTLS via SPIFFE)                │
│        │                                                                   │
│        ▼                                                                   │
│  Trust boundary 1: foundry-runtime → foundry-guardrails                    │
│        │  per-pod SPIFFE identity                                          │
│        │  mTLS                                                             │
│  ┌─ foundry-guardrails -rest pods ─────────────────────────────────────┐   │
│  │  - OIDC bearer verification (for tenant-direct REST callers)        │   │
│  │  - SPIFFE identity verification (for in-cluster runtime callers)    │   │
│  │  - Cedar policy check before any internal dispatch                  │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│        │                                                                   │
│  Trust boundary 2: rest → usecase → adapter dispatch                       │
│        │  in-process; no network                                           │
│        ▼                                                                   │
│  ┌─ prompt-classifier-usecase ─┐   ┌─ output-validator-usecase ────┐       │
│  │  ensemble: heuristic +      │   │  ensemble: secret-leak +      │       │
│  │  classifier-model +         │   │  exfiltration + slop +        │       │
│  │  LLM-judge fallback         │   │  hallucinated-tool-args       │       │
│  └─────────────────────────────┘   └───────────────────────────────┘       │
│        │                                       │                           │
│  Trust boundary 3: classifier-model-server (in-cluster ONNX serving)       │
│        │                                       ▼                           │
│  ┌─ classifier-model-serving pods (per-pack; per-model) ─────────────┐     │
│  │  - ONNX runtime; pre-loaded artifacts; Cosign-verified at start   │     │
│  │  - per-pod SPIFFE; only -adapter-classifier-model can reach       │     │
│  │  - stateless; no per-request persistence                          │     │
│  └───────────────────────────────────────────────────────────────────┘     │
│                                                                            │
│  Trust boundary 4: rule-store Postgres                                     │
│        │  TLS; mTLS optional; service-account binding via OpenBao          │
│        ▼                                                                   │
│  ┌─ Postgres HA (per-pack; rule definitions + Cedar fragments + audit) ┐   │
│  │  - rule writes require GitHub PR + signed-commit + audit-chain emit │   │
│  │  - reads scoped per (pack, tenant) via Cedar at app layer           │   │
│  │  - per-row tenant + pack columns; cross-tenant queries refused       │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
│  Trust boundary 5: LLM-judge fallback → foundry-providers                  │
│        │  in-cluster mTLS via foundry-providers SDK                        │
│        ▼                                                                   │
│  ┌─ foundry-providers (sibling) ─────────────────────────────────┐         │
│  │  - LLM credentials never resident in guardrails                │         │
│  │  - per-tenant + per-pack budget tracking                       │         │
│  └────────────────────────────────────────────────────────────────┘         │
│                                                                            │
│  Trust boundary 6: GuardrailDecisionEmitted → foundry-evidence + audit-chain│
│        │  AsyncAPI publisher; in-cluster bus                               │
│        ▼                                                                   │
│  ┌─ audit-chain µservice (Ed25519 + Merkle seal) ─┐                        │
│  └────────────────────────────────────────────────┘                        │
└────────────────────────────────────────────────────────────────────────────┘
```

Six trust boundaries:
1. **foundry-runtime → foundry-guardrails** (mTLS + SPIFFE; only sanctioned caller).
2. **rest → usecase → adapter** (in-process Cedar checks).
3. **classifier-model-server** (per-pack ONNX serving; Cosign-verified).
4. **Postgres rule store** (per-pack; tenant + pack columns; PR-only writes).
5. **LLM-judge fallback → foundry-providers** (in-cluster mTLS; provider credentials never resident here).
6. **Audit emission** (AsyncAPI → audit-chain).

## Assets & Data Classification

Per Bominal ADR-0028 + `oya-check-data-class` LEAN lane.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| Inbound prompt text (per-invocation) | `BEHAVIORAL_TENANT_PRODUCT` + transient `PII_IDENTIFYING` / `PHI` (when present) | High | Not persisted by guardrails (in-memory only); persisted only by foundry-evidence under that µservice's retention | foundry-evidence (downstream) |
| Outbound provider output text | `BEHAVIORAL_TENANT_PRODUCT` + transient `PII` / `PHI` | High | Not persisted; same as above | foundry-evidence |
| Classification scores (per-detector outputs) | `INTERNAL_ONLY` | Medium | 30d via observability metric stream | observability (Mimir) |
| GuardrailDecision verdicts | `AUDIT` + `BEHAVIORAL_TENANT_PRODUCT` | High | ≥ 1y; 6y for pack-us-healthcare HIPAA §164.316(b)(2) | audit-chain + foundry-evidence |
| Rule definitions (content-safety rules) | `INTERNAL_ONLY` | Medium | git history; per-row in Postgres with append-only mutation log | Postgres + git |
| Cedar policy fragments | `INTERNAL_ONLY` (policy text) | Medium | git history | git + Postgres registry |
| Classifier-model artifacts (ONNX) | `INTERNAL_ONLY` (weights binary) | Medium-High (tampering risk) | versioned; signed via Cosign; per-pack S3 | object storage |
| Classifier-model training data references | `INTERNAL_ONLY` (metadata only; not the data itself) | Low | per-model card | foundry-evidence model-card store |
| Per-tenant Cedar overlay fragments | `INTERNAL_ONLY` (fragment text) + per-tenant entitlement (`SENSITIVE_PIPA_ART23` correlation) | Medium-High | git history + Postgres registry | git + Postgres |
| OpenBao-managed secrets (Cosign signing key + LLM-judge provider tokens via foundry-providers) | `SECRET` | Critical | OpenBao 30d-90d rotation | OpenBao |
| LLM-judge invocation payload (sent to foundry-providers when fallback used) | `BEHAVIORAL_TENANT_PRODUCT` | High | Not persisted by guardrails; per foundry-providers + foundry-evidence | foundry-evidence |
| Audit-chain seal records | `AUDIT` | High | append-only; immutable | audit-chain |
| Sev-1 jailbreak-success incident records | `AUDIT` + `INTERNAL_ONLY` (incident text) | High | indefinite (incident history) | audit-chain + foundry-evidence |
| Per-tenant FP escalation budget state | `INTERNAL_ONLY` | Low-Medium | per-tenant; per-month rolling | Postgres |

## Actors

| Actor | Trust level | Authentication | Capability |
|---|---|---|---|
| foundry-runtime (in-cluster) | Trusted internal | mTLS + SPIFFE `spiffe://oyatie/foundry-runtime/*` | Submit prompt + output for classification/validation; receive verdict |
| Tenant operator (direct REST; rare) | Untrusted external | OIDC + MFA via Application Shell | Read own tenant's rule overlays; mark FP escalation; read decision history |
| Customer application (machine; via foundry-runtime; never direct) | n/a | n/a (only via foundry-runtime) | n/a |
| Rule author (axis-foundry-guardrails operator) | Trusted internal | OIDC + MFA + signed-commit | Author rule definitions + Cedar fragments via git PR |
| Classifier-model author (axis-foundry-guardrails operator) | Trusted internal | OIDC + MFA + Cosign signing key (JIT via OpenBao) | Publish new classifier-model artifacts with Cosign signature |
| Auditor (external) | Read-only external; time-boxed JIT | OIDC + MFA + short-lived token via OpenBao | Read decision records (own-tenant scope or audit-firm scope) |
| Reviewer agent (oya-pr-review lane) | Trusted internal | OIDC-bound CI identity | Refuse merges that violate gate |
| Ops-security operator | Trusted internal | OIDC + MFA + JIT elevation | Admin operations (Cedar bundle hot-reload; Sev-1 incident response) |
| LLM-judge fallback caller (in-cluster) | Trusted internal | mTLS + SPIFFE; consumes foundry-providers SDK | Invoke LLM-judge for ambiguous classifications |
| Attacker — opportunistic | Untrusted | none | Scans; assume always present |
| Attacker — targeted | Untrusted | none | Sophisticated jailbreak crafting; assume present |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure rule; deploy bad classifier (caught by shadow-mode) |
| Insider — malicious | Trusted internal | OIDC + MFA | Tamper with rule / Cedar fragment / classifier model — mitigated by signed commits + Cosign + audit-chain + 2-person rule |

## STRIDE Threat Catalog

Each threat carries: ID; category; asset; description; likelihood (L/M/H); impact (L/M/H); risk score; mitigations; owner; residual risk; framework controls satisfied.

### Spoofing (S)

**T-S-01 — Attacker forges SPIFFE identity to bypass foundry-guardrails (impersonates foundry-runtime)**
- Asset: rest endpoint authentication boundary
- Likelihood: L / Impact: H (could dispatch to foundry-providers without guardrail) / Risk: **M**
- Mitigations:
  - SPIFFE attestation includes pod-identity + namespace + service-account; spoofing requires cluster-admin (RBAC-blocked).
  - Mesh network policy: only `foundry-runtime/*` SPIFFEs may reach guardrails REST endpoints.
  - REST endpoint enforces SPIFFE allow-list; non-listed SPIFFE → 403 + audit-emit `sso_unknown_caller`.
  - Runtime-guardrails-coupling CI lane asserts the only call-site for guardrails is foundry-runtime.
- Owner: ops-security + axis-foundry-guardrails
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2; ISO 27001 A.5.15, A.8.5

**T-S-02 — Attacker impersonates rule-author identity to push malicious rule into Postgres**
- Asset: rule-store mutation path
- Likelihood: L / Impact: H (could weaken every tenant's safety floor) / Risk: **M**
- Mitigations:
  - All rule mutations go through GitHub PR; signed-commits required (branch-protection).
  - Postgres writer is restricted to a single ServiceAccount; SA token JIT-issued by OpenBao only after PR merge.
  - 2-person rule for rule mutations affecting more than one pack (CODEOWNERS).
  - Rule-mutation audit-chain seal includes PR ID + author SPIFFE + signed-commit SHA; mismatch fails.
- Owner: ops-security + axis-foundry-guardrails
- Residual: L
- Frameworks: SOC 2 CC6.1, CC8.1; ISO 27001 A.5.15, A.5.18, A.8.4, A.8.32; EU AI Act Art. 12 (record-keeping)

**T-S-03 — Attacker spoofs classifier-model artifact (replaces Cosign-signed model with malicious weights)**
- Asset: classifier-model artifact in object storage
- Likelihood: L / Impact: H (poisoned classifier silently passes malicious prompts) / Risk: **M**
- Mitigations:
  - All classifier-model artifacts signed via Cosign at publish time; signature key in OpenBao with JIT issuance.
  - Pod-start integrity check: verify Cosign signature against expected key + verify weight-file SHA matches model-card; mismatch refuses start + emits `classifier_model_integrity_violation`.
  - Object-storage bucket policy: write-only-from-signed-publisher SA; no human direct write.
  - Annual classifier-model supply-chain audit.
- Owner: ops-security + axis-foundry-guardrails
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6, CC8.1; ISO 27001 A.5.23, A.8.7, A.8.25, A.8.28; EU AI Act Art. 15 (cybersecurity)

**T-S-04 — Attacker spoofs GuardrailDecisionEmitted event to mask a real Sev-1 jailbreak**
- Asset: AsyncAPI event chain
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Every emitted event Ed25519-signed by emitter SPIFFE.
  - audit-chain consumer rejects events with bad signature; alerts on rejection rate > 0.
  - Two-channel corroboration: emit to AsyncAPI + write a Postgres mutation-log row; reconciliation job cross-checks.
- Owner: axis-foundry-guardrails + audit-chain
- Residual: L

### Tampering (T)

**T-T-01 — Cedar policy fragment tampering via repo push**
- Asset: Cedar fragment registry
- Likelihood: M / Impact: H (could permit cross-tenant entitlement leakage) / Risk: **H**
- Mitigations:
  - Cedar fragments under CODEOWNERS scoped to axis-foundry-guardrails + ops-security.
  - `oya gate validate cedar-default-deny-enforced` LEAN lane refuses bundles missing the base `forbid` rule.
  - `oya gate validate cedar-fragment-coverage` validates the bundle's per-action coverage matrix.
  - Cedar v4 schema-validation at PR time; runtime hot-reload verifies the new bundle against a golden test set before promote.
- Owner: axis-foundry-guardrails + ops-security
- Residual: L
- Frameworks: SOC 2 CC8.1; ISO 27001 A.5.31, A.5.32, A.8.32, A.8.33; EU AI Act Art. 13 (transparency); ADR-0140

**T-T-02 — Postgres rule-store row tampering via direct DB access**
- Asset: rule definitions table
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - DB write access restricted to the rule-store writer ServiceAccount; no human DB-admin without 2-person rule + JIT.
  - Per-row append-only mutation log; updates emit a new row (no in-place mutation), with prior-row pointer.
  - Postgres audit extension (pgaudit) logs every DDL/DML; logs replicated to audit-chain.
  - Daily reconciliation: live rule rows vs git-source-of-truth diff; mismatch alerts.
- Owner: ops-security + axis-foundry-guardrails
- Residual: L

**T-T-03 — Classifier-model weight tampering at rest in object storage**
- Asset: ONNX artifact
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Object-storage bucket policy: WORM where supported (S3 Object Lock Compliance mode).
  - SSE-KMS encryption; key in pack-scoped KMS keyring.
  - Pod-start Cosign-verification (T-S-03).
  - Periodic block validator runs against object-store blocks.
- Owner: ops-security + cloud-secrets

**T-T-04 — Adversarial prompt evades classifier (intentional model-input-evasion attack)**
- Asset: classifier ensemble decision
- Likelihood: H (this is the attacker's daily job) / Impact: H / Risk: **H**
- Mitigations:
  - Multi-detector ensemble (heuristic + classifier + LLM-judge) — no single point of failure.
  - Canonicalisation pre-pass strips whitespace + zero-width + homoglyph + base64 obfuscation before classifier inference.
  - Continuous red-team: golden-fixture catalogue of known jailbreaks rerun on every PR; new patterns folded in monthly.
  - LLM-as-judge fallback invoked when ensemble disagreement; expensive but bounds residual risk.
  - Per-tenant jailbreak-attempt-rate SLO; spike triggers per-tenant review.
- Owner: axis-foundry-guardrails
- Residual: M (cat-and-mouse; never fully eliminable)
- Frameworks: EU AI Act Art. 15 (robustness + cybersecurity); MITRE ATLAS AML.T0043 (Craft Adversarial Data); OWASP LLM01 (Prompt Injection)

**T-T-05 — Multi-turn drift attack: incremental prompt steering across many turns evades single-turn classification**
- Asset: per-invocation classifier (which sees only the current turn)
- Likelihood: M / Impact: H / Risk: **M-H**
- Mitigations:
  - Per-session "drift signal" carried by foundry-runtime in the prompt context; guardrails treats `session_turn_count` + `topic_distance_from_first_turn` as features.
  - Jailbreak ensemble includes a session-aware classifier-model variant when `session_turn_count > 5`.
  - Per-session block-rate SLO; high block-rate-in-late-turn triggers session quarantine.
- Owner: axis-foundry-guardrails + axis-foundry-runtime
- Residual: M

**T-T-06 — Classifier-model rollout tampering (bad model promoted to enforce-mode without shadow phase)**
- Asset: classifier-model deploy pipeline
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - IP-014 enforces shadow→enforce; promote-to-enforce LEAN lane refuses without ≥ 7d shadow + shadow-vs-enforce-delta review sign-off.
  - Shadow-mode metrics emitted: `oya_guardrails_shadow_decision_total{model="<v>", verdict="<v>"}`; observability dashboard surfaces drift.
- Owner: axis-foundry-guardrails

### Repudiation (R)

**T-R-01 — Guardrail issued block; tenant disputes the block reason**
- Asset: GuardrailDecision audit trail
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - Every block emits `block_reason` enum + `cedar_policy_ids[]` + `classifier_model_versions{}` + `evidence_hash`.
  - Decision record signed by emitter SPIFFE; audit-chain Merkle proof binds to time.
  - Tenant can query decision detail via REST (own-tenant scope) for last 30d (subject to retention).
- Owner: axis-foundry-guardrails
- Residual: L
- Frameworks: SOC 2 CC4.1; ISO 27001 A.5.27, A.8.15; EU AI Act Art. 13 (transparency); GDPR Art. 22 (automated decision explanation)

**T-R-02 — Sev-1 jailbreak success: detector missed it; rule-author denies awareness**
- Asset: jailbreak detector history
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Every Sev-1 jailbreak issue auto-allocates incident ID; post-mortem template auto-generated.
  - Detector version + classifier-model version pinned to incident record.
  - Rule-author dashboard shows incident queue; SLA on triage.
- Owner: axis-foundry-guardrails

**T-R-03 — Rule mutation executed; mutating author denies authorship**
- Asset: rule mutation history
- Likelihood: L / Impact: L / Risk: **L**
- Mitigations:
  - Rule mutations land via GitHub PR; signed-commits enforced; CODEOWNERS approver recorded.
  - Postgres mutation log row signed by author SPIFFE.

### Information Disclosure (I)

**T-I-01 — Prompt-content leakage via classifier-model logs / OTel traces**
- Asset: in-flight prompt text
- Likelihood: M (engineers log prompt context for debugging) / Impact: H (PII / PHI exposure) / Risk: **H**
- Mitigations:
  - Prompt text NEVER logged at INFO level; DEBUG-level only with explicit `data_class=PII_REDACTED` mode that hashes the prompt.
  - OTel SDK redactor in guardrails pod strips prompt content from span attributes (replaces with `prompt_hash`).
  - Classifier-model inference is in-process; prompt text passes through TLS to classifier-model-serving but is not stored.
  - `data_class` annotation on every prompt-bearing struct; lane refuses INFO-level logging of `BEHAVIORAL_TENANT_PRODUCT` content.
  - Periodic synthetic-PII detection: synthetic prompts emitted; verify no leakage to Loki / Tempo / Mimir.
- Owner: axis-foundry-guardrails + each developer
- Residual: M (engineering discipline gap)
- Frameworks: SOC 2 CC6.7; ISO 27001 A.8.11, A.8.12; GDPR Art. 5(1)(c) Art. 25 Art. 32; KR PIPA Art. 3; HIPAA §164.502(b); EU AI Act Art. 10 (data governance)

**T-I-02 — Classifier-model inversion: attacker extracts training data via crafted prompts**
- Asset: classifier-model artifact
- Likelihood: L / Impact: M-H (depending on what's in training data) / Risk: **M**
- Mitigations:
  - Classifier-model card documents training-data provenance; no PII / PHI in training corpus.
  - Model output is verdict-only (boolean + score), not free-form generation; inversion attack surface is bounded.
  - For LLM-as-judge fallback (which DOES generate free-form), the prompt template enforces no-data-disclosure prompt structure.
- Owner: axis-foundry-guardrails
- Residual: L
- Frameworks: MITRE ATLAS AML.T0024 (Model Stealing); EU AI Act Art. 15 (cybersecurity)

**T-I-03 — Cross-tenant rule disclosure (tenant-A operator reads tenant-B Cedar overlay)**
- Asset: per-tenant Cedar overlay
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - REST endpoint enforces Cedar policy `tenant-scope.cedar`: principal.tenant_id == resource.tenant_id.
  - Postgres rule-store enforces row-level security (Postgres RLS) on `tenant_id` column.
  - LEAN lane verifies all rule-store queries pass through the typed `RuleStore` port (no raw SQL).
- Owner: axis-foundry-guardrails

**T-I-04 — Sev-1 jailbreak success — provider output containing PHI / sensitive content reached tenant before block**
- Asset: provider output (post-classification)
- Likelihood: L (defence-in-depth: pre+post classification both) / Impact: H / Risk: **M**
- Mitigations:
  - Post-output validator is the second line of defence against same prompt; if pre-classifier misses, post-validator must catch.
  - On detection, output is rewritten with redaction; original-output stored in foundry-evidence under tighter ACL.
  - Sev-1 incident auto-allocated; rule-author + ensemble model retrained on the failure case.
- Owner: axis-foundry-guardrails

**T-I-05 — Secret leak in provider output (provider hallucinates a real API key / token)**
- Asset: provider output content
- Likelihood: M (LLMs occasionally regurgitate training-data secrets) / Impact: H / Risk: **H**
- Mitigations:
  - Output-validator includes a secret-leak detector (regex + entropy + known-secret-pattern matchers); strong-match → block + Sev-1 incident.
  - Secret-pattern library shared with `oya-foundry-fitness-evidence-secret-scan` lane.
  - If the secret matches an oyatie-known secret (OpenBao audit-emit), trigger rotation pipeline.
- Owner: ops-security + axis-foundry-guardrails
- Frameworks: OWASP LLM06 (Sensitive Information Disclosure)

**T-I-06 — Logged classifier scores reveal sensitive prompt features (side-channel)**
- Asset: classifier score telemetry to observability
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Score telemetry coarsened (5-bucket quantization, not raw float) before emission.
  - Per-tenant score histograms are `BEHAVIORAL_TENANT_PRODUCT` and only available to the tenant + auditor.

### Denial of Service (D)

**T-D-01 — Classifier-model-serving overload via burst from one tenant**
- Asset: classifier-serving pool
- Likelihood: H / Impact: H (every tenant's pre-invocation blocked if pool exhausted; fail-closed = every invocation refused) / Risk: **H**
- Mitigations:
  - Per-tenant rate limit at REST layer: 1000 RPS soft / 5000 RPS hard per autonomy tier.
  - HPA on classifier-serving: scale on CPU + p99 latency; min 4 / max 200 per pack.
  - Per-pod queue depth ≤ 50; excess returns 429 with `Retry-After` hint.
  - Pre-warmed pool: 2 standby per model.
  - Fail-closed posture: when classifier serving outage, gate refuses every invocation (per ADR-0022 effective-ceiling).
- Owner: ops-sre-reliability + axis-foundry-guardrails
- Residual: L

**T-D-02 — LLM-judge fallback budget exhaustion (attacker crafts many ambiguous prompts)**
- Asset: per-tenant LLM-judge budget (via foundry-providers)
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - Per-tenant LLM-judge fallback budget: 100/hour soft, 500/hour hard.
  - Budget exhaustion = fail-closed for ambiguous prompts (return block + budget-exceeded reason).
  - Per-tenant overage alarm; rule-author review.
- Owner: axis-foundry-guardrails + axis-foundry-providers

**T-D-03 — Cedar engine evaluation overload (large policy bundle, large input)**
- Asset: Cedar engine
- Likelihood: L / Impact: M / Risk: **L-M**
- Mitigations:
  - Cedar input size bounded at REST layer (max prompt 10MB; max output 10MB).
  - Cedar evaluation timeout: 10ms per invocation; timeout → fail-closed.
  - Cedar engine sharded per pack; no single global bottleneck.

**T-D-04 — Postgres rule-store query overload (cardinality explosion in rule)**
- Asset: Postgres rule-store
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - Rule definitions cached in-pod with 5s TTL; rule-store reads coalesced.
  - Postgres connection pool sized per `capacity-model.md`; circuit-breaker at 80% pool utilization.
  - Cardinality limits at rule-author PR-time: max 10k rules per pack.

**T-D-05 — Fail-closed cascade: classifier outage → every invocation blocked → tenant outage**
- Asset: tenant-facing availability
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Multi-detector ensemble: heuristic detector runs even if classifier outage (degraded confidence; flagged in verdict).
  - Per-tenant emergency bypass entitlement (Cedar): high-trust tenants can use heuristic-only mode for ≤ 1h during classifier outage; emits Sev-2 + per-tenant audit-chain seal.
  - SRE runbook: classifier-model-rollback.md restores prior model in ≤ 5 min.
- Owner: ops-sre-reliability + axis-foundry-guardrails

### Elevation of Privilege (E)

**T-E-01 — Autonomy-tier-gate bypass: caller submits a forged `tier_claim` and gate accepts**
- Asset: ADR-0022 effective ceiling
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - `tier_claim` is computed server-side from (principal SPIFFE, tenant.configured_tier, capability.min_required, pack.cap, subject.class_cap); NEVER read from request body.
  - Cedar policy on the gate refuses any request where the computed ceiling < the requested tier.
  - 100% test coverage on effective-ceiling computation (per ADR-0022 §"Effective-ceiling resolution").
- Owner: axis-foundry-guardrails + ops-security
- Frameworks: ADR-0022; EU AI Act Art. 14 (human-oversight)

**T-E-02 — Cedar policy bug allows cross-tenant rule mutation**
- Asset: rule-store mutation authorization
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cedar fragments fuzzed at CI time (`oya-check-cedar-fragment-coverage` lane).
  - Default-deny base + per-action permit fragments; deny-overrides semantics catch broken permits.
  - Annual external pen-test against Cedar policy.

**T-E-03 — Rule-author SA token compromise → unauthorized rule push**
- Asset: rule-store writer ServiceAccount token
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - SA token bound to pod identity; 24h rotation.
  - Mutation also requires PR merge (token alone insufficient).

**T-E-04 — Classifier-model-serving pod RCE → execute arbitrary code in pack-scoped cluster**
- Asset: classifier-model pod
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - ONNX runtime sandboxed (no syscall expansion); pod runs as non-root.
  - Pod has no outbound network (NetworkPolicy denies egress except mTLS to caller).
  - SBOM scanning (Trivy + Grype) on every classifier-model image at build time.
- Frameworks: OWASP LLM Top 10 (Insecure Plugin Design); EU AI Act Art. 15

**T-E-05 — Adversarial prompt induces tool call to a forbidden tool (e.g., file delete)**
- Asset: capability + tool authorization at runtime
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Note: tool authorization is in foundry-runtime's threat model; guardrails covers the upstream half: the prompt classifier flags `tool_invocation_intent` features and the output validator catches `hallucinated_tool_args`.
  - Cross-µservice: foundry-runtime's per-capability Cedar policy is the final gate on tools.
- Owner: foundry-runtime + axis-foundry-guardrails (joint)

## LINDDUN Privacy-Threat Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | Prompt hash + decision history | Same end-user's prompts across sessions can be linked via prompt-hash even if not stored | Prompt-hash salt rotated per-pack 12mo; hash is one-way; cross-pack linkage impossible | L |
| T-L-02 | Identifiability | Per-tenant Cedar overlay name | Tenant overlay name reveals tenant identity if exposed cross-tenant | Tenant overlay names hashed at API surface; raw names only in Postgres | L |
| T-L-03 | Non-repudiation | Author SPIFFE on rule-mutation seal | Rule author cannot deny authorship; this is the desired outcome | n/a (intended property) | n/a |
| T-L-04 | Detectability | Classifier-model invocation timing | Burst of LLM-judge fallbacks correlates with tenant business event (suspicious-pattern surge) | Reasonable; BEHAVIORAL_TENANT_PRODUCT; consent at onboarding | M |
| T-L-05 | Disclosure | Auditor pivot from one tenant to another via shared decision dashboards | Same risk as observability auditor-scope; inherits mitigations | Cedar `auditor-scope.cedar` tenant-scoped; pen-test annually | L |
| T-L-06 | Unawareness | End-user (tenant's user) unaware their prompts are classified | Joint controllership; tenant must disclose in privacy notice | Joint-controllership cascade in DPA template | M |
| T-L-07 | Non-compliance | GDPR Art. 22 — automated decision explanation requirement | Block decision is automated; tenant may invoke Art. 22 explanation right | Decision record carries `block_reason` enum + `cedar_policy_ids[]` + `classifier_model_versions{}`; explanation is structured | L |

## Mitigations Catalog (cross-reference)

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Multi-detector ensemble (heuristic + classifier + LLM-judge) | Preventive | axis-foundry-guardrails | jailbreak golden-fixture lane |
| Cedar v4 + default-deny + per-tenant overlays | Preventive | axis-foundry-guardrails + ops-security | `oya gate validate cedar-default-deny-enforced` |
| Cosign-signed classifier-model artifacts | Preventive (tampering) | ops-security + axis-foundry-guardrails | pod-start verification + LEAN |
| Per-tenant rate limits | Preventive (DoS) | axis-foundry-guardrails | observability metrics |
| Signed-commits on rule + Cedar fragment PRs | Preventive (tampering) | ops-security | branch-protection.yaml |
| Audit-chain Ed25519 seal on every decision | Detective + Non-repudiation | audit-chain | regression tests |
| Shadow→enforce rule rollout | Preventive (regression) | axis-foundry-guardrails | IP-014 |
| Prompt redaction in logs (data_class annotation) | Preventive (disclosure) | every developer | `oya-check-data-class` lane |
| OTel redactor in guardrails pod | Preventive | axis-foundry-guardrails | synthetic-PII drill |
| Per-tenant FP escalation budget | Detective (over-aggressive policy) | axis-foundry-guardrails | tenant dashboard |
| 2-person rule on cross-pack rule mutation | Preventive (insider) | ops-security | CODEOWNERS |
| Per-pack residency (no cross-pack rule replication) | Preventive (data-residency) | axis-foundry-guardrails | `gate validate cross-pack-replication-forbidden` |

## Residual Risk Acceptance

Residual risks above L require explicit acceptance by council-architecture + ops-security + council-privacy:

| Risk ID | Residual | Why accepted | Re-review |
|---|---|---|---|
| T-T-04 (Adversarial prompt evasion) | M | Cat-and-mouse; bounded by ensemble + red-team cadence | Quarterly |
| T-T-05 (Multi-turn drift) | M | Session-aware classifier addresses; some drift attacks still bypass | Quarterly |
| T-I-01 (PII leakage in logs) | M | Engineering discipline floor; same as observability T-I-02 | Quarterly |
| T-L-04 (Detectability via timing) | M | Inherent in tenant business reality | Annually |
| T-L-06 (End-user unawareness) | M | Tenant-of-tenant responsibility | Annually |

Sign-off (RW until council sign-off):
- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`

## Per-Pack Overlay Sections

### pack-kr (KR-ISMS-P + KR PIPA + 전자문서법)

- **KR PIPA Art. 23 (sensitive personal information)**: PHI / sensitive-data prompt content treated under Art. 23 sensitivity; pack-kr default rules block high-confidence sensitive content unless tenant has explicit Cedar overlay (rare for KR; requires PIPA Art. 23(2) explicit consent at tenant onboarding).
- **KR PIPA Art. 29 (technical safeguards)**: every mitigation maps to one of the 12 prescribed safeguards.
- **KR-ISMS-P §2.5 (인적보안) + §2.7 (접근통제)**: 2-person rule + JIT elevation.
- **전자문서법 Art. 5 (electronic document integrity)**: audit-chain Ed25519 seal satisfies for decision records.

### pack-us-healthcare (HIPAA-scoped)

- **HIPAA §164.502(a) Permitted Uses (TPO)**: guardrail operations satisfy the "Operations" prong; PHI processing requires BAA on file.
- **§164.502(b) Minimum Necessary**: prompt + output handled in-memory only; not persisted by guardrails.
- **§164.308(a)(4)(ii)(B) Access Authorization**: Cedar policy enforces per-tenant scope.
- **§164.312(a)(1) Access Control**: identical implementation to observability tenant-isolation.
- **§164.312(b) Audit Controls**: audit-chain emission on every PHI-touching decision; retention ≥ 6y for HIPAA-tagged tenants (cost-budget reflects).
- **45 CFR §164.504(e) (Business Associate Agreement)**: oyatie operates as Business Associate; BAA at `legal/baa-template.md` (Slice D).

### pack-eu (GDPR + EU AI Act + EDPB + NIS2)

- **GDPR Art. 22 (automated decision-making)**: guardrail block decisions ARE automated-decisions-affecting-individuals. We provide:
  - Art. 22(3) right to meaningful information about the logic (decision-detail REST endpoint).
  - Art. 22(3) right to contest (FP escalation budget; rule-author review).
  - Art. 22(3) right to human intervention (tenant operator can override per Cedar entitlement; emits audit-chain seal).
- **EU AI Act Art. 9 (risk-management)**: this threat model + DPIA + compliance.md is the risk-management system.
- **EU AI Act Art. 10 (data + data-governance)**: classifier-model training-data provenance documented; no PII in training corpus.
- **EU AI Act Art. 11 (technical-documentation)**: this document + PRD + model-cards in foundry-evidence.
- **EU AI Act Art. 12 (record-keeping)**: audit-chain seals satisfy.
- **EU AI Act Art. 13 (transparency)**: block_reason + cedar_policy_ids in decision detail.
- **EU AI Act Art. 14 (human-oversight)**: FP escalation + tenant manual override + per-tenant Cedar entitlement.
- **EU AI Act Art. 15 (accuracy + robustness + cybersecurity)**: ensemble + Cosign + shadow→enforce + red-team cadence.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlay sections at `regional-packs/<pack>/foundry-guardrails-overlay.md`.

## Compliance Cross-Mapping

| Framework | Coverage | Mapping doc |
|---|---|---|
| SOC 2 Type 2 | CC1–CC9 controls covered as cited inline | `microservices/foundry-guardrails/compliance.md` |
| ISO 27001:2022 | Annex A.5–A.8 covered inline | `microservices/foundry-guardrails/compliance.md` |
| GDPR + EU AI Act | Arts. 5, 6, 9, 22, 25, 28, 30, 32, 35; AI Act Arts. 9-15 | `microservices/foundry-guardrails/dpia.md` + `compliance.md` |
| HIPAA | §164.308 / §164.312 / §164.502 / §164.504(e) | `compliance.md` |
| KR PIPA | Arts. 15/17/23/23-2/29/29-2 | `compliance.md` |

## Re-review Triggers

- Cedar policy bundle change.
- Classifier-model rollout (any model, any pack).
- Trust-boundary diagram change.
- Sev-1 jailbreak success (always triggers a re-review of T-T-04 / T-T-05).
- Annual scheduled review.
- Post-incident.
- Pen-test or audit finding.

## References

- ADR-0022: Autonomy ceiling.
- ADR-0028 (Bominal): Audit chain.
- ADR-0140: Cedar policy substrate.
- ADR-0131: Per-microservice flat layout (Foundry split).
- `microservices/foundry-guardrails/PRD.md`.
- `microservices/foundry-guardrails/dpia.md`.
- `microservices/foundry-guardrails/compliance.md`.
- `docs/quality/ai-slop-defense/ai-slop-failure-mode-catalogue.md`.
- Microsoft Threat Modeling methodology (STRIDE).
- LINDDUN — Wuyts et al., KU Leuven.
- OWASP Top 10 (2021) + OWASP LLM Top 10 (2025).
- MITRE ATLAS (Adversarial Threat Landscape for Artificial-Intelligence Systems).
- NIST SP 800-154 + NIST AI RMF 1.0.
- EU AI Act (Regulation 2024/1689).
- AWS Bedrock Guardrails security model — `docs.aws.amazon.com/bedrock/latest/userguide/guardrails-security.html`.
- Microsoft Azure AI Content Safety security model — `learn.microsoft.com/azure/ai-services/content-safety/concepts/`.
- NVIDIA NeMo Guardrails security model — `github.com/NVIDIA/NeMo-Guardrails/blob/main/docs/security.md`.
