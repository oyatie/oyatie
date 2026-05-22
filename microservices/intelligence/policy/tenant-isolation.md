---
doc_class: PolicySpec
title: Tenant Isolation Specification (intelligence)
microservice: intelligence
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: ops-security + axis-intelligence
related_adrs: [ADR-0244, ADR-0255, ADR-0254, ADR-0248]
related_artifacts:
  - microservices/intelligence/threat-model.md (T-S-01, T-I-01, T-E-01)
  - microservices/intelligence/dpia.md (R-01)
  - microservices/intelligence/policy/dispatch-authorization.cedar
  - microservices/intelligence/policy/byok-gating.cedar
review_cadence: quarterly + on every BC promotion + on every provider-adapter add
doc_status: published
---

# Tenant Isolation Specification (intelligence µservice)

## Purpose

Define how dispatch processing, credential resolution, audit-tap persistence, and provider
routing prevent cross-tenant data flow. This is the canonical isolation artifact reviewed by SOC 2
examiners, ISO 27001 + ISO 42001 auditors, GDPR Art. 32 reviewers, KR PIPA Art. 29 reviewers, and
HIPAA §164.312(a)(1) reviewers.

## Invariants

### TI-INT-01: Single-tenant dispatch envelope

Every dispatch carries exactly one `tenant_id`. The substrate refuses any envelope without a
`tenant_id` or with a `tenant_id` not matching the principal's claim. No batching of dispatches
across tenants is permitted.

### TI-INT-02: Per-tenant credential isolation (provider-credential BYOK, ADR-0255 §D-4)

Provider-credential BYOK is the default for B2B (ADR-0255 §D-4). The `credential-resolver` resolves SecretReferences against
per-tenant OpenBao paths only; CredentialHandles are bound to the resolving tenant + provider +
audience + short TTL (≤ 15 min). Per ADR-0296, the credential never enters intelligence process
memory; only the sidecar can inject it at HTTP-call assembly.

### TI-INT-03: Per-tenant audit-tap isolation

Audit-tap records carry tenant_id; audit-chain seal stream is filtered by tenant_id on every read.
Auditor scope is gated by `auditor-scope.cedar` with explicit `scoped_tenants` set.

### TI-INT-04: Reserved-namespace audience tags

Audience tags `consumer`, `developer`, `internal-foundry` are reserved enum values; the substrate
refuses arbitrary audience strings. Audience tag mutation requires Cedar `dispatch-authorization`
PERMIT match for the principal's allowed-audience set.

### TI-INT-05: Per-tenant cost-cap enforcement

Per-tenant daily + monthly cost caps enforced by `dispatch-authorization.cedar` consumer-budget
and tenant-cost-cap predicates; breach refuses dispatch with `CostCapExceeded`.

### TI-INT-06: Reserved cell topology

Per ADR-0248, intelligence is cell-eligible. Tenants are shuffle-sharded across Tier-3 cells; no
cross-cell traffic for tenant payloads.

### TI-INT-07: No prompt/output content in observability

Prompt + output content never lands in observability (Mimir/Loki/Tempo). Only the audit-tap
records (in audit-chain) carry hashes + classification + meta — never raw content.

## Tenant identity model

```text
canonical_tenant_id      = <opaque-string from tenancy µservice>
hashed_tenant_id         = sha256(canonical_tenant_id ++ deployment_salt)[..16]
tenant_id (envelope use) = "tenant:" + hashed_tenant_id
```

## Failure modes (isolation)

| FM | Behaviour | Detection |
|---|---|---|
| FM-INT-01 | Tenant-A dispatches with tenant_id of B | `dispatch-authorization.cedar` cross-tenant FORBID; metric `oya_intelligence_tenant_spoofing_attempt_total` |
| FM-INT-02 | provider-credential BYOK SecretReference cross-tenant pivot (ADR-0255 §D-4) | `byok-gating.cedar` bound-tenant FORBID; metric `oya_intelligence_byok_cross_tenant_pivot_total` |
| FM-INT-03 | Audit-tap read attempt cross-tenant | `auditor-scope.cedar` scoped-tenants FORBID; metric |
| FM-INT-04 | Cell-cross traffic for tenant payload | cell µservice's cross-cell-deny; metric |

## Audit trail

| Event | Emitter | Fields | Retention |
|---|---|---|---|
| Tenant spoofing attempt | dispatch handler | tenant_a, tenant_b, principal_id, timestamp | 1y default; pack-extends |
| provider-credential BYOK cross-tenant pivot (ADR-0255 §D-4) | credential-resolver | tenant_a, tenant_b, principal_id, secret_path, timestamp | 1y default |
| Audit-tap cross-tenant read | audit-tap-worker | principal_id, requested_tenant, scoped_tenants, timestamp | 1y default |

## Verification

- `cargo run -p oya-dev-cli -- gate validate intelligence-tenant-isolation` — exit 0.
- Quarterly chaos drill: induce cross-tenant dispatch attempt; verify refusal.
- Annual pen-test against tenant boundary.

## References

- ADR-0244, ADR-0255, ADR-0254, ADR-0248, ADR-0296.
- `microservices/intelligence/policy/dispatch-authorization.cedar`.
- `microservices/intelligence/policy/byok-gating.cedar`.
- `microservices/intelligence/policy/auditor-scope.cedar`.
- `microservices/intelligence/threat-model.md`.
- GDPR Art. 32; KR PIPA Art. 29; HIPAA §164.312(a)(1); ISO 27001:2022 A.5.15 + A.8.12.
