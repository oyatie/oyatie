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

## Next IP

[`IP-013-bulk-distribute-worker.md`](IP-013-bulk-distribute-worker.md)
