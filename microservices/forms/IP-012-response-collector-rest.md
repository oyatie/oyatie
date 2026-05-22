---
doc_class: ImplementationPlan
milestone: M03-workspace-tier-foundation
phase: P01-forms-foundation
impl_plan_id: IP-012-response-collector-rest
status: pending
execution_unit: ChangeSet
owner: axis-forms
acceptance_lanes: [cargo-test, oya-forms-submission-latency, oya-forms-audit-chain-coverage, oya-forms-prefill-link-integrity]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: Response-collector REST surface

## Intent

Axum-based REST server that hosts the OpenAPI surface (contracts/openapi/forms.openapi.yaml). Anonymous + authenticated submission, validation, captcha verify, audit-chain seal, workflow-trigger fan-out, webhook enqueue.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/forms/src/rest/server.rs` | create |
| `microservices/forms/src/rest/handlers/forms.rs` | create |
| `microservices/forms/src/rest/handlers/responses.rs` | create |
| `microservices/forms/src/rest/handlers/public_submit.rs` | create — anonymous endpoint |
| `microservices/forms/src/rest/handlers/ai_build.rs` | create |
| `microservices/forms/src/rest/handlers/webhooks.rs` | create |
| `microservices/forms/src/rest/middleware/cedar.rs` | create |
| `microservices/forms/src/rest/middleware/rate_limit.rs` | create |
| `microservices/forms/src/rest/middleware/audit_chain.rs` | create |
| `microservices/forms/tests/rest_openapi_conformance.rs` | create |
| `microservices/forms/tests/rest_submission_e2e.rs` | create |

## Acceptance Gates

- OpenAPI conformance: 100% match against contracts/openapi/forms.openapi.yaml.
- Submission p95 ≤ 150ms under k6 load (PRD performance).
- Pre-fill HMAC + TTL: tampered → 401; expired → 410.

## References

- Axum.
- OpenAPI 3.2.0.
- ADR-0028 audit-chain.
- PRD FR-05, FR-18, FR-19, FR-24 and AC-06 / AC-07 / AC-21 / AC-22 / AC-27.
- `microservices/forms/contracts/openapi/forms.openapi.yaml`.
- `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml`.
- `microservices/forms/slos/submission-latency.openslo.yaml`.
- `microservices/forms/runbooks/response-store-corruption.md`.

## Foundation A-G Substance

- A. Product scope: response collector is the authoritative ingress for anonymous, authenticated, and prefilled submissions.
- B. Domain model: collector orchestrates `SubmissionCommand`, `ValidatedResponse`, `AuditSeal`, `WorkflowTrigger`, and `WebhookJob`.
- C. Contracts: OpenAPI is the external contract; AsyncAPI captures submission, failure, webhook, and workflow fan-out events.
- D. Policy: Cedar evaluation, rate limit, captcha, data residency, and prefill HMAC checks happen before response persistence.
- E. Operations: induced downstream failure records audit rows and queues retry/dead-letter work without silently dropping submits.
- F. Observability: emit submission p95, HMAC rejection count, workflow-start failures, webhook enqueue lag, and audit seal failures.
- G. Promotion: OpenAPI conformance, 1000-submission audit-chain corpus, prefill integrity, and workflow fail-closed tests gate done.

## Counterpart Benchmark

- Counterpart: Salesforce Web-to-Lead submission endpoint, HubSpot Forms submissions API, and GitHub issue forms intake.
- Defensible parity claim: Oyatie must accept high-volume public responses while preserving audit-chain and version binding.
- Differentiator: downstream workflow failure is fail-closed and visible rather than silently losing an automation trigger.
- Grep counterpart names: Salesforce Web-to-Lead; HubSpot Forms; GitHub issue forms.

## Remediation Notes

- Added REST, AsyncAPI, SLO, runbook, and PRD bindings to make the ingress plan foundation-grade.
- Added A-G substance for orchestration, contracts, policy, operations, telemetry, and promotion.
- Added counterpart names for mechanical parity review.

## Verification Evidence Required

- OpenAPI conformance proves every documented path, response code, and schema matches implementation.
- 1000-submission corpus proves audit-chain coverage and version binding.
- Induced workflow-engine 500 proves fail-closed behavior and dead-letter visibility.
- Prefill HMAC corpus proves tampered links return 401 and expired links return 410.

## Next IP

[`IP-013-bulk-distribute-worker.md`](IP-013-bulk-distribute-worker.md)
