---
doc_class: DPIA
microservice: feature-flags
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0242
  - ADR-0244
  - ADR-0251
  - ADR-0276
  - ADR-0292
companion_docs:
  - microservices/feature-flags/compliance.md
  - microservices/feature-flags/PRD.md
  - microservices/feature-flags/policy/data-residency.md
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# Data Protection Impact Assessment (DPIA) — Feature Flags

## 1. Processing description

The feature-flags µservice processes the following data categories to deliver runtime flag evaluation and experiment assignment:

| Data element | Purpose | Legal basis (GDPR Art. 6) | Retention |
|---|---|---|---|
| `tenant_id` | Flag scoping; prevents cross-tenant data access | Legitimate interest (contract performance) | Duration of tenant contract |
| `principal_id` | Targeting-rule evaluation; percentage rollout hashing | Legitimate interest | 90 days post-flag-sunset |
| `persona_tier` | Targeting rule evaluation | Legitimate interest | Session-bound |
| `cohort_ids[]` | Cohort-based targeting | Legitimate interest | 30 days |
| `user_id` (hashed for rollout bucket) | Deterministic percentage-rollout assignment | Legitimate interest | Not stored; computed on-fly |
| Flag evaluation context (non-PII subset) | Rule predicate evaluation | Legitimate interest | Not stored when `audit_required: false` |
| Audit-chain events (`audit_required: true` flags) | Compliance audit, regulatory evidence | Legal obligation | 7 years (GDPR Art. 5(1)(e) storage limitation exception for regulatory purposes) |
| Experiment assignment records | Statistical attribution; experiment integrity | Legitimate interest + consent for behavioral experiments | Duration of experiment + 90 days |
| IP address (evaluation request) | Abuse-defence scoring | Legitimate interest (security) | 30 days |

## 2. Necessity and proportionality

- `principal_id` and `persona_tier` are the minimum necessary for targeting-rule evaluation. Raw user PII (name, email, phone) MUST NOT be passed in evaluation context. Enforced by Cedar allowlist gate on `EvaluationContext` fields.
- `user_id` for rollout hashing is one-way HMAC'd; original `user_id` is not retained after hash computation.
- Audit events for `audit_required: false` flags contain no PII; only `tenant_id + flag_key + result + timestamp`.
- Experiment records use `experiment_assignment_id` (UUID) not `user_id`; attribution is pseudonymous.

## 3. Risks and mitigations

| Risk | Likelihood | Severity | Mitigation |
|---|---|---|---|
| Cross-tenant flag definition read | Low | High | Cedar default-deny; per-tenant RLS on flags table; Citus shard isolation |
| Cohort inference through differential evaluation responses | Medium | Medium | Evaluation response does not reveal targeting rule structure; Cedar predicate compiled to Wasm (not human-readable in response) |
| Re-identification of users from rollout-bucket + evaluation pattern | Low | Medium | Rollout bucket is per-(tenant, flag, user) hash; salt rotated per experiment; bucket alone does not identify user |
| Audit-required flag leaking sensitive user state | Medium | High | Audit events encrypted per tenant DEK using encryption-key BYOK where enabled (ADR-0251 §D-10); access gated to compliance officer role; 7-year retention with AES-256-GCM encryption at rest |
| Pack-override agent misconfiguration applying wrong overlay | Low | High | Pack overlays are Cedar-gated; dual-control in production; `PackOverrideTamperAttempt` event on anomaly |
| Experiment data used for profiling beyond experiment scope | Medium | High | Experiment records are pseudonymous; cross-experiment correlation blocked by salt rotation; per-pack `behavioral-profiling-flag` forced off by GDPR pack |

## 4. Data subject rights

| Right | Implementation |
|---|---|
| **Art. 15 (Access)** | Per-tenant DSAR export: flag evaluation history for `audit_required: true` flags, experiment assignment records |
| **Art. 17 (Erasure)** | DSR cascade runner (per ADR-0276): deletes `principal_id`-linked records; replaces with tombstone `[ERASED]`; retains aggregate experiment stats (no PII) |
| **Art. 20 (Portability)** | JSON export per `flag-definitions-export-v1.json` schema |
| **Art. 22 (Automated decision)** | Experiment assignment is automated; carries human-readable explanation + appeal URL |
| **Art. 17(3) exception** | Audit events retained beyond erasure request for legal obligation (regulatory compliance) |

## 5. International transfers

- EU tenant data: remains in `eu-cell-1` (Frankfurt/Amsterdam); never transferred to US cells without explicit tenant opt-in + Standard Contractual Clauses.
- KR tenant data: remains in `kr-cell-1`; data-residency hard-stop enforced by Cedar gate + Cilium network policy.
- FedRAMP tenant data: remains in `us-gov-cell-1`; no transfer to commercial cells.

## 6. DPO consultation

DPO consulted on: (a) `audit_required: true` flag retention policy (7 years), (b) experiment pseudonymisation approach (salt rotation + UUID assignment), (c) cohort inference risk assessment. DPO sign-off obtained 2026-05-15.

## 7. Residual risk

Residual risk: **Low**. After mitigations, the primary residual risk is the audit-trail retention period (7 years) creating a long-lived pseudonymous dataset. Mitigated by: per-tenant DEK encryption, compliance-officer-only access (Cedar gate), and automated erasure of all PII-adjacent fields at 90 days post-experiment.
