---
doc_class: PolicySpec
title: Dual-Context Policy — submitter vs tenant operator vs auditor vs CI vs anonymous
microservice: forms
status: Accepted
date: 2026-05-17
owner_team: ops-security + axis-forms + council-privacy
related_artifacts:
  - microservices/forms/policy/tenant-scope.cedar
  - microservices/forms/policy/ci-scope.cedar
  - microservices/forms/policy/auditor-scope.cedar
  - microservices/forms/policy/public-read.cedar
  - microservices/forms/threat-model.md
review_cadence: annually + on any new principal type
doc_status: published
---

# Dual-Context Policy: Five Principal Classes

Forms is unusual among oyatie µservices: it has **five distinct principal classes** with different trust postures, and several actions (notably "submit a response") happen across the trust boundary. This document defines the dual-context model and how Cedar policy fragments compose to enforce it.

## Principal classes

| Class | Cedar entity | Trust | Typical action |
|---|---|---|---|
| TenantOperator | `TenantOperator::<oidc-sub>` | High (authenticated; OIDC + seat entitlement) | Author forms; read aggregates; export responses |
| Submitter (authenticated) | `AuthenticatedSubmitter::<oidc-sub>` | Medium (authenticated submitter; OIDC-only) | Submit responses; read own response (DSR) |
| Submitter (anonymous) | `AnonymousPrincipal::"anonymous"` | Low (no identity) | Submit responses to published-and-anonymous-allowed forms only |
| Auditor | `Auditor::<firm-engagement>` | Time-boxed read-only (JIT token; ≤ 4h TTL; tenant allow-list) | Read audit seals + metadata |
| ServiceAccount | `ServiceAccount::"oya-forms-ci"` | CI scope (no tenant data; no PII; no writes) | Read governance + metadata |

## Composition rules

Policy fragments compose by **inclusion** at the Cedar evaluator:

```
tenant-scope.cedar     applies to TenantOperator + AuthenticatedSubmitter
ci-scope.cedar         applies to ServiceAccount::"oya-forms-ci" only
auditor-scope.cedar    applies to Auditor::* only
public-read.cedar      applies to AnonymousPrincipal::"anonymous" only
```

Each fragment includes its own `forbid (principal, action, resource);` default-deny at the top. Cedar evaluator applies the **union** of fragments; a deny in any matching fragment denies the request (Cedar default-deny semantics).

## Cross-context invariants

1. **No anonymous principal can ever read a response.** Only submit; never read. Even own-response read by anonymous principal requires conversion to authenticated submitter via OIDC + claim-of-ownership.

2. **No auditor principal can ever read PII content.** Only audit seals + metadata. Auditor read of any PII column rejected by `auditor-scope.cedar` FORBID.

3. **No CI principal can ever read tenant content.** Even non-PII form definitions are denied; CI only sees metadata + aggregates.

4. **No tenant operator can read another tenant's data.** Cross-tenant FORBID is defense-in-depth on top of Citus RLS.

5. **PHI requires `pack=pack-us-healthcare` + `baa_signed=true`.** Enforced by FORBID in `tenant-scope.cedar`.

6. **GDPR Art. 9 special-category requires `art9_explicit_consent=true`.** Enforced by FORBID in `tenant-scope.cedar`.

7. **reCAPTCHA configuration is forbidden in pack-eu / pack-kr / pack-us-healthcare.** Per `ADR-FORMS-0002` privacy posture.

## Dual-context handling: anonymous submission

The "submit a response" action is the most-trafficked path; it crosses the trust boundary from anonymous to authenticated-resource-state.

```
AnonymousPrincipal::"anonymous"
        │
        │ Cedar permits via public-read.cedar PERMIT-3 (submit_response)
        │ provided: published + accepting_submissions + anonymous_submission_allowed
        ▼
Forms middleware:
  1. Verify captcha token (hCaptcha / Turnstile / Friendly Captcha)
  2. Verify rate-limit (per-IP + per-form)
  3. Verify HMAC + TTL on pre-filled link (if pre-filled)
  4. Validate field-by-field against form.v1 schema
  5. Evaluate cross-field validation DAG (server-authoritative)
  6. Re-evaluate conditional logic (server-authoritative per ADR-FORMS-0004)
  7. Submitter hash = HMAC(per-form-salt || submitter_identifier_class || raw)
  8. PII encrypted at column level via per-tenant DEK
  9. Audit-chain seal Ed25519-signed
 10. Workflow-trigger fan-out (fail-closed)
 11. Webhook delivery enqueued
        │
        ▼
Resource state: Response::{tenant_id, response_id, ...} written
```

## Dual-context handling: AI-form-build T2

When a TenantOperator invokes AI-form-build T2 with a cross-microservice destination (e.g., "build me a form that emails on submit"):

```
TenantOperator::<oidc-sub>
        │
        │ Cedar permits via tenant-scope.cedar PERMIT-7 (invoke_ai_form_build)
        │ provided: ai_build_consent + has_ai_form_build_entitlement
        │
        │ Cedar permits via tenant-scope.cedar FORBID-exception (T2 cross-µservice)
        │ provided: has_t2_cross_microservice_consent + destination_microservice in allowlist
        ▼
foundry-providers (LLM call routing)
        │
        │ PII redactor + prompt-injection scrub before LLM call
        ▼
LLM (pack-resident provider)
        │
        ▼
Forms dsl-loader validates LLM output:
  - schema-valid form.v1
  - no cross-µservice node references the tenant has no entitlement to
  - no data_class mismatch (PII tagged correctly)
  - no Cedar bypass attempt
        │
        ▼
Draft created (ChangeSet) — reviewer-agent + tenant accept BEFORE save
        │
        ▼
Resource state: FormDraft::{tenant_id, draft_id, ai_build_origin=true, tier=T2, ...}
```

## Verification

- `oya-forms-cedar-default-deny-conformance` — exit 0 on 50-case adversarial corpus.
- `oya-forms-cross-tenant-rls-enforced` — Citus RLS + Cedar defense-in-depth assertion.
- Quarterly chaos drill: induce auditor pivot attempt; expect FORBID.

## References

- All four .cedar fragments in this directory.
- `threat-model.md`.
- ADR-0140 (retired per ADR-0145) Cedar policy.
- ADR-FORMS-0002, ADR-FORMS-0005.
- Cedar v4 semantics documentation — `docs.cedarpolicy.com/`.
